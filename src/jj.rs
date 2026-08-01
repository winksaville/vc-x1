//! Typed facade over jj read queries.
//!
//! Reads evaluate in-process through jj-lib (`common::load_repo` +
//! `common::resolve_revset`, then `Commit` accessors), with one
//! carve-out: a revset that references a working copy (`@`, `@-`,
//! `ws@`) still spawns `jj`, whose auto-snapshot keeps "right now"
//! questions honest; those reads move in-process with the mutation
//! lift (the jj-lib migration stage of `notes/refactor-20260716.md`).
//! `bookmark_list` / `bookmark_list_all` also still spawn: their
//! consumers parse the CLI listing textually.
//!
//! - `matches` / `rev_exists`: does a revset match / does a
//!   revision resolve.
//! - `chid_of` / `cid_of` / `cid_short_of` / `cids_short_of`:
//!   change / commit ids.
//! - `desc_of` / `is_empty`: description and emptiness.
//! - `is_no_such_revision`: classify jj's unresolvable-revision
//!   error, typed (in-process) or by stderr wording (spawned).
//!
//! Mutations still spawn `jj` at their call sites; the jj-lib
//! migration stage moves them in-process too.

use std::path::Path;

use jj_lib::commit::Commit;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::Repo;
use jj_lib::revset::RevsetResolutionError;
use pollster::FutureExt;

use crate::common;

/// Crate-standard boxed-error result, aliased locally for brevity.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// True when `revset` references a working copy, so the query must
/// go through the spawned CLI's auto-snapshot until the mutation
/// lift moves snapshotting in-process.
///
/// A `@` is working-copy syntax (`@`, `@-`, `ws@`) unless it has
/// symbol characters on *both* sides, which is the remote-bookmark
/// form (`name@remote`).
fn references_working_copy(revset: &str) -> bool {
    let is_symbol_char =
        |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | '+');
    let chars: Vec<char> = revset.chars().collect();
    chars.iter().enumerate().any(|(i, &c)| {
        c == '@'
            && !(i > 0
                && is_symbol_char(chars[i - 1])
                && chars.get(i + 1).copied().is_some_and(is_symbol_char))
    })
}

/// Run `jj log -r <rev> --no-graph -T <template> -R <repo>` and
/// return its stdout (`run` trims surrounding whitespace): the
/// spawn path for working-copy-relative revsets.
fn log_spawn(repo: &Path, rev: &str, template: &str) -> Result<String> {
    common::run(
        "jj",
        &[
            "log",
            "-r",
            rev,
            "--no-graph",
            "-T",
            template,
            "-R",
            &repo.to_string_lossy(),
        ],
        Path::new("."),
    )
}

/// Resolve `revset` in-process and return its commits, in revset
/// order (newest first, matching `jj log`).
fn commits_of(repo: &Path, revset: &str) -> Result<Vec<Commit>> {
    let (workspace, repo_at_head) = common::load_repo(repo)?;
    let ids = common::resolve_revset(&workspace, &repo_at_head, revset)?;
    ids.iter()
        .map(|id| Ok(repo_at_head.store().get_commit(id)?))
        .collect()
}

/// True when `revset` matches at least one commit in `repo`.
///
/// A valid-but-empty revset (e.g. `conflicts()` on a clean repo)
/// is `Ok(false)`; an unresolvable revision is an `Err`, which
/// `rev_exists` folds to `false`.
pub fn matches(repo: &Path, revset: &str) -> Result<bool> {
    if references_working_copy(revset) {
        return Ok(!log_spawn(repo, revset, "\"x\"")?.is_empty());
    }
    let (workspace, repo_at_head) = common::load_repo(repo)?;
    Ok(!common::resolve_revset(&workspace, &repo_at_head, revset)?.is_empty())
}

/// True when `rev` resolves in `repo`.
///
/// Folds jj's unresolvable-revision error (`is_no_such_revision`,
/// either path) to `Ok(false)`; other failures (bad repo path,
/// spawn error) stay `Err`.
pub fn rev_exists(repo: &Path, rev: &str) -> Result<bool> {
    match matches(repo, rev) {
        Ok(found) => Ok(found),
        Err(e) if is_no_such_revision(e.as_ref()) => Ok(false),
        Err(e) => Err(e),
    }
}

/// True when `e` is jj's unresolvable-revision error, in either
/// form: the in-process `RevsetResolutionError::NoSuchRevision`
/// (typed) or the spawned CLI's stderr wording (`doesn't exist`,
/// `No such revision`).
pub fn is_no_such_revision(e: &(dyn std::error::Error + 'static)) -> bool {
    if let Some(res) = e.downcast_ref::<RevsetResolutionError>() {
        return matches!(res, RevsetResolutionError::NoSuchRevision { .. });
    }
    let msg = e.to_string();
    msg.contains("doesn't exist") || msg.contains("No such revision")
}

/// The 12-character change id of `rev`.
pub fn chid_of(repo: &Path, rev: &str) -> Result<String> {
    if references_working_copy(rev) {
        return log_spawn(repo, rev, "change_id.short(12)");
    }
    Ok(commits_of(repo, rev)?
        .iter()
        .map(common::format_chid)
        .collect())
}

/// The full commit id of `rev`.
pub fn cid_of(repo: &Path, rev: &str) -> Result<String> {
    if references_working_copy(rev) {
        return log_spawn(repo, rev, "commit_id");
    }
    Ok(commits_of(repo, rev)?
        .iter()
        .map(|c| c.id().hex())
        .collect())
}

/// The 12-character commit id of `rev`.
pub fn cid_short_of(repo: &Path, rev: &str) -> Result<String> {
    if references_working_copy(rev) {
        return log_spawn(repo, rev, "commit_id.short(12)");
    }
    Ok(cids_short_of(repo, rev)?.concat())
}

/// 12-character commit ids of every commit `revset` resolves to:
/// the multi-result sibling of `cid_short_of` (e.g. the heads of a
/// conflicted bookmark).
pub fn cids_short_of(repo: &Path, revset: &str) -> Result<Vec<String>> {
    if references_working_copy(revset) {
        let out = log_spawn(repo, revset, r#"commit_id.short(12) ++ "\n""#)?;
        return Ok(out
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect());
    }
    Ok(commits_of(repo, revset)?
        .iter()
        .map(|c| {
            let hex = c.id().hex();
            hex[..hex.len().min(12)].to_string()
        })
        .collect())
}

/// The full description (title + body) of `rev`, surrounding
/// whitespace trimmed (matching the spawn path's `run` trim).
pub fn desc_of(repo: &Path, rev: &str) -> Result<String> {
    if references_working_copy(rev) {
        return log_spawn(repo, rev, "description");
    }
    let all: String = commits_of(repo, rev)?
        .iter()
        .map(|c| c.description())
        .collect();
    Ok(all.trim().to_string())
}

/// True when `rev` is empty (no file changes relative to its
/// parent). Strict: exactly one commit must resolve (in-process),
/// or the template must print `true` / `false` (spawned).
pub fn is_empty(repo: &Path, rev: &str) -> Result<bool> {
    if references_working_copy(rev) {
        let out = log_spawn(repo, rev, "empty")?;
        return match out.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            other => Err(format!("jj::is_empty: unexpected template output {other:?}").into()),
        };
    }
    let (workspace, repo_at_head) = common::load_repo(repo)?;
    let ids = common::resolve_revset(&workspace, &repo_at_head, rev)?;
    match ids.as_slice() {
        [id] => {
            let commit = repo_at_head.store().get_commit(id)?;
            Ok(commit.is_empty(repo_at_head.as_ref()).block_on()?)
        }
        other => Err(format!(
            "jj::is_empty: expected exactly one commit for {rev:?}, got {}",
            other.len()
        )
        .into()),
    }
}

/// Run `jj bookmark list <bookmark> -R <repo>` and return its
/// stdout, which is empty when the local bookmark doesn't exist.
pub fn bookmark_list(repo: &Path, bookmark: &str) -> Result<String> {
    common::run(
        "jj",
        &["bookmark", "list", bookmark, "-R", &repo.to_string_lossy()],
        Path::new("."),
    )
}

/// Like `bookmark_list` but with `-a`: includes remote refs,
/// tracked ones indented (`  @origin: ...`), non-tracking at
/// column 0 (`<bookmark>@<remote>: ...`). The input for
/// `common::find_tracked_remote` /
/// `common::find_non_tracking_remote`.
pub fn bookmark_list_all(repo: &Path, bookmark: &str) -> Result<String> {
    common::run(
        "jj",
        &[
            "bookmark",
            "list",
            "-a",
            bookmark,
            "-R",
            &repo.to_string_lossy(),
        ],
        Path::new("."),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{self, Fixture, jj_ok};

    /// Working-copy forms route to the spawn path.
    #[test]
    fn working_copy_revsets_are_detected() {
        assert!(references_working_copy("@"));
        assert!(references_working_copy("@-"));
        assert!(references_working_copy("@ & empty()"));
        assert!(references_working_copy("::@"));
        assert!(references_working_copy("ws@"));
        assert!(references_working_copy("abc123::(@-)"));
    }

    /// Remote-bookmark forms and plain revsets stay in-process.
    #[test]
    fn non_working_copy_revsets_are_not_detected() {
        assert!(!references_working_copy("main"));
        assert!(!references_working_copy("conflicts()"));
        assert!(!references_working_copy("bookmarks(exact:main)"));
        assert!(!references_working_copy("feature@origin"));
        assert!(!references_working_copy("present(refactor-vc-x1@origin)"));
        assert!(!references_working_copy("abc123::main"));
    }

    /// The in-process id/description accessors agree with the
    /// spawned `jj log` templates on the same commit.
    #[test]
    fn in_process_reads_match_spawned_jj() {
        let fx = Fixture::new("jjreads");
        // Pin `@-` to its concrete commit id so every query below
        // is a non-`@` revset (the in-process path under test).
        let rev = test_helpers::cid(&fx.work, "@-");
        assert_eq!(cid_short_of(&fx.work, &rev).unwrap(), rev);
        assert!(cid_of(&fx.work, &rev).unwrap().starts_with(&rev));
        assert_eq!(
            chid_of(&fx.work, &rev).unwrap(),
            test_helpers::chid(&fx.work, "@-")
        );
        assert_eq!(
            desc_of(&fx.work, &rev).unwrap(),
            test_helpers::description(&fx.work, "@-")
        );
        let expected_empty =
            jj_ok(&fx.work, &["log", "-r", &rev, "--no-graph", "-T", "empty"]) == "true";
        assert_eq!(is_empty(&fx.work, &rev).unwrap(), expected_empty);
    }

    /// `matches` / `rev_exists` in-process: valid-but-empty is
    /// `false`, unresolvable folds to `false` via the typed
    /// `NoSuchRevision`, and a real commit id resolves.
    #[test]
    fn matches_and_rev_exists_in_process() {
        let fx = Fixture::new("jjexists");
        let rev = test_helpers::cid(&fx.work, "@-");
        assert!(!matches(&fx.work, "conflicts()").unwrap());
        assert!(rev_exists(&fx.work, &rev).unwrap());
        assert!(!rev_exists(&fx.work, "no-such-bookmark-xyz").unwrap());
    }

    /// `cids_short_of` returns one id per matching commit, exactly
    /// the bookmark-heads shape `sync` consumes.
    #[test]
    fn cids_short_of_returns_bookmark_heads() {
        let fx = Fixture::new("jjheads");
        let rev = test_helpers::cid(&fx.work, "@-");
        jj_ok(&fx.work, &["bookmark", "set", "testbm", "-r", &rev]);
        assert_eq!(
            cids_short_of(&fx.work, "bookmarks(exact:testbm)").unwrap(),
            vec![rev]
        );
        assert!(
            cids_short_of(&fx.work, "bookmarks(exact:absentbm)")
                .unwrap()
                .is_empty()
        );
    }
}

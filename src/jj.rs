//! Typed facade over jj: reads and the publish-path mutations, all
//! in-process through jj-lib (the jj-lib migration stage of
//! `notes/refactor-20260716.md`).
//!
//! Reads resolve through `common::load_repo` +
//! `common::resolve_revset` and `Commit` accessors; a revset that
//! references a working copy (`@`, `@-`, `ws@`) snapshots first
//! (`repo_for_read`), so "right now" questions answer about the
//! filesystem the way the spawned CLI's auto-snapshot did.
//! Mutations run on the `session` module's `RepoSession`: its
//! settings / snapshot / transaction-finish plumbing.
//!
//! - `matches` / `rev_exists`: does a revset match / does a
//!   revision resolve.
//! - `chid_of` / `cid_of` / `cid_short_of` / `cids_short_of`:
//!   change / commit ids.
//! - `desc_of` / `is_empty`: description and emptiness.
//! - `is_no_such_revision`: the typed unresolvable-revision test.
//! - `commit` / `describe` / `bookmark_set` / `git_push_bookmark` /
//!   `git_fetch`: the publish-path mutations.
//!
//! Still spawning `jj` elsewhere: `bookmark_list` /
//! `bookmark_list_all` here (their consumers parse the CLI listing
//! textually), and the call sites outside the five verbs
//! (`jj squash`, `jj new`, `jj rebase`, `jj op log` / `op restore`,
//! `jj git clone`, `jj diff --stat`, repo-init plumbing).

use std::path::Path;

use jj_lib::commit::Commit;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::Repo;
use jj_lib::revset::RevsetResolutionError;
use pollster::FutureExt;

use crate::common;

mod session;

/// Crate-standard boxed-error result, aliased locally for brevity.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// `desc` with the trailing newline jj normalizes descriptions to.
fn complete_newline(desc: &str) -> String {
    if desc.is_empty() || desc.ends_with('\n') {
        desc.to_string()
    } else {
        format!("{desc}\n")
    }
}

/// Snapshot the working copy, then update the wc commit's
/// description and start a new empty change on top (`jj commit`).
pub fn commit(repo: &Path, desc: &str) -> Result<()> {
    let mut ses = session::RepoSession::open(repo)?;
    ses.snapshot()?;
    let wc_commit = ses.wc_commit()?;
    let name = ses.workspace.workspace_name().to_owned();
    let mut tx = ses.start_tx();
    let new_commit = tx
        .repo_mut()
        .rewrite_commit(&wc_commit)
        .set_description(complete_newline(desc))
        .write()
        .block_on()?;
    let new_wc_commit = tx
        .repo_mut()
        .new_commit(vec![new_commit.id().clone()], wc_commit.tree())
        .write()
        .block_on()?;
    tx.repo_mut().edit(name, &new_wc_commit).block_on()?;
    ses.finish_tx(tx, &format!("commit {}", wc_commit.id().hex()))
}

/// Snapshot, then rewrite `rev`'s description and rebase its
/// descendants (`jj describe -r <rev> -m <desc>`).
pub fn describe(repo: &Path, rev: &str, desc: &str) -> Result<()> {
    let mut ses = session::RepoSession::open(repo)?;
    ses.snapshot()?;
    let commit = one_commit(&ses, rev)?;
    let desc = complete_newline(desc);
    if commit.description() == desc {
        return Ok(());
    }
    let mut tx = ses.start_tx();
    tx.repo_mut()
        .rewrite_commit(&commit)
        .set_description(desc)
        .write()
        .block_on()?;
    tx.repo_mut().rebase_descendants().block_on()?;
    ses.finish_tx(tx, &format!("describe commit {}", commit.id().hex()))
}

/// Snapshot, then point local bookmark `name` at `rev`
/// (`jj bookmark set <name> -r <rev>`).
pub fn bookmark_set(repo: &Path, name: &str, rev: &str) -> Result<()> {
    use jj_lib::op_store::RefTarget;
    let mut ses = session::RepoSession::open(repo)?;
    ses.snapshot()?;
    let commit = one_commit(&ses, rev)?;
    let mut tx = ses.start_tx();
    tx.repo_mut()
        .set_local_bookmark_target(name.as_ref(), RefTarget::normal(commit.id().clone()));
    ses.finish_tx(
        tx,
        &format!("point bookmark {} to commit {}", name, commit.id().hex()),
    )
}

/// Snapshot, then push local bookmark `name` to `origin`
/// (`jj git push --bookmark <name>`), force-with-lease against the
/// last-seen remote position.
///
/// Deviation from the CLI: no conflicted / no-description / private
/// preflights on the pushed commits; the push callers validate the
/// commit they publish before calling.
pub fn git_push_bookmark(repo: &Path, name: &str) -> Result<()> {
    use jj_lib::git::{GitPushOptions, GitPushRefTargets, GitSettings};
    use jj_lib::merge::Diff;
    use jj_lib::ref_name::{RefNameBuf, RemoteName};

    let mut ses = session::RepoSession::open(repo)?;
    ses.snapshot()?;
    let remote = RemoteName::new("origin");
    let ref_name: RefNameBuf = name.into();
    let local = ses
        .repo
        .view()
        .get_local_bookmark(&ref_name)
        .as_normal()
        .cloned()
        .ok_or_else(|| format!("bookmark '{name}' is absent or conflicted"))?;
    let remote_ref = ses
        .repo
        .view()
        .get_remote_bookmark(ref_name.to_remote_symbol(remote));
    let before = remote_ref.target.as_normal().cloned();
    if before.as_ref() == Some(&local) {
        return Ok(()); // Already in sync; matches `jj git push` "Nothing changed".
    }
    let ref_updates = GitPushRefTargets {
        bookmarks: vec![(
            ref_name.clone(),
            Diff {
                before,
                after: Some(local),
            },
        )],
        tags: vec![],
    };
    let git_settings = GitSettings::from_settings(&ses.settings)?;
    let mut tx = ses.start_tx();
    let stats = jj_lib::git::push_refs(
        tx.repo_mut(),
        git_settings.to_subprocess_options(),
        remote,
        &ref_updates,
        &mut session::DebugCallback,
        &GitPushOptions::default(),
    )?;
    if !stats.all_ok() {
        return Err(format!(
            "git push of bookmark '{name}' failed: rejected {:?}, remote rejected {:?}",
            stats.rejected, stats.remote_rejected
        )
        .into());
    }
    ses.finish_tx(tx, &format!("push bookmark {name} to git remote origin"))
}

/// Snapshot, then fetch all branches from `remote` and import the
/// results (`jj git fetch --remote <remote>`). Returns one line per
/// changed remote bookmark, empty when the fetch changed nothing.
///
/// Deviation from the CLI: the branch expression is always "all
/// branches" rather than the remote's configured fetch refspecs,
/// which is what a standard `origin` resolves to anyway.
pub fn git_fetch(repo: &Path, remote: &str) -> Result<Vec<String>> {
    use jj_lib::git::{GitFetch, GitFetchRefExpression, GitSettings, expand_fetch_refspecs};
    use jj_lib::ref_name::RemoteName;
    use jj_lib::str_util::StringExpression;

    let mut ses = session::RepoSession::open(repo)?;
    ses.snapshot()?;
    let remote = RemoteName::new(remote);
    let expanded = expand_fetch_refspecs(
        remote,
        GitFetchRefExpression {
            bookmark: StringExpression::all(),
            tag: StringExpression::none(),
        },
    )?;
    let git_settings = GitSettings::from_settings(&ses.settings)?;
    let import_options = ses.import_options()?;
    let mut tx = ses.start_tx();
    let mut fetch = GitFetch::new(
        tx.repo_mut(),
        git_settings.to_subprocess_options(),
        &import_options,
    )?;
    fetch.fetch(remote, expanded, &mut session::DebugCallback, None, None)?;
    let stats = fetch.import_refs().block_on()?;
    drop(fetch);
    let lines: Vec<String> = stats
        .changed_remote_bookmarks
        .iter()
        .map(|(symbol, _)| format!("bookmark {symbol} updated"))
        .collect();
    if !tx.repo().has_changes() {
        return Ok(lines);
    }
    tx.repo_mut().rebase_descendants().block_on()?;
    ses.finish_tx(tx, &format!("fetch from git remote {}", remote.as_str()))?;
    Ok(lines)
}

/// Resolve `rev` against an open engine to exactly one commit.
fn one_commit(ses: &session::RepoSession, rev: &str) -> Result<jj_lib::commit::Commit> {
    let ids = common::resolve_revset(&ses.workspace, &ses.repo, rev)?;
    match ids.as_slice() {
        [id] => Ok(ses.repo.store().get_commit(id)?),
        other => Err(format!(
            "expected exactly one commit for {rev:?}, got {}",
            other.len()
        )
        .into()),
    }
}

/// True when `revset` references a working copy, so the query must
/// snapshot before resolving (an op-store write) to see the
/// filesystem as it is right now, like the CLI's auto-snapshot.
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

/// Load `repo` for a read of `revset`: a working-copy-relative
/// revset snapshots first so "right now" questions answer about the
/// filesystem; anything else is a plain read-only load.
fn repo_for_read(
    repo: &Path,
    revset: &str,
) -> Result<(
    jj_lib::workspace::Workspace,
    std::sync::Arc<jj_lib::repo::ReadonlyRepo>,
)> {
    if references_working_copy(revset) {
        let mut ses = session::RepoSession::open(repo)?;
        ses.snapshot()?;
        Ok((ses.workspace, ses.repo))
    } else {
        common::load_repo(repo)
    }
}

/// Resolve `revset` in-process and return its commits, in revset
/// order (newest first, matching `jj log`).
fn commits_of(repo: &Path, revset: &str) -> Result<Vec<Commit>> {
    let (workspace, repo_at_head) = repo_for_read(repo, revset)?;
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
    let (workspace, repo_at_head) = repo_for_read(repo, revset)?;
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

/// True when `e` is jj's unresolvable-revision error, the typed
/// `RevsetResolutionError::NoSuchRevision` (every rev query is
/// in-process now, so the stderr-wording fallback is gone).
pub fn is_no_such_revision(e: &(dyn std::error::Error + 'static)) -> bool {
    matches!(
        e.downcast_ref::<RevsetResolutionError>(),
        Some(RevsetResolutionError::NoSuchRevision { .. })
    )
}

/// The 12-character change id of `rev`.
pub fn chid_of(repo: &Path, rev: &str) -> Result<String> {
    Ok(commits_of(repo, rev)?
        .iter()
        .map(common::format_chid)
        .collect())
}

/// The full commit id of `rev`.
pub fn cid_of(repo: &Path, rev: &str) -> Result<String> {
    Ok(commits_of(repo, rev)?
        .iter()
        .map(|c| c.id().hex())
        .collect())
}

/// The 12-character commit id of `rev`.
pub fn cid_short_of(repo: &Path, rev: &str) -> Result<String> {
    Ok(cids_short_of(repo, rev)?.concat())
}

/// 12-character commit ids of every commit `revset` resolves to:
/// the multi-result sibling of `cid_short_of` (e.g. the heads of a
/// conflicted bookmark).
pub fn cids_short_of(repo: &Path, revset: &str) -> Result<Vec<String>> {
    Ok(commits_of(repo, revset)?
        .iter()
        .map(|c| {
            let hex = c.id().hex();
            hex[..hex.len().min(12)].to_string()
        })
        .collect())
}

/// The full description (title + body) of `rev`, surrounding
/// whitespace trimmed (the shape the spawned `-T description` form
/// returned, which every caller still expects).
pub fn desc_of(repo: &Path, rev: &str) -> Result<String> {
    let all: String = commits_of(repo, rev)?
        .iter()
        .map(|c| c.description())
        .collect();
    Ok(all.trim().to_string())
}

/// True when `rev` is empty (no file changes relative to its
/// parent). Strict: exactly one commit must resolve.
pub fn is_empty(repo: &Path, rev: &str) -> Result<bool> {
    let (workspace, repo_at_head) = repo_for_read(repo, rev)?;
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

    /// `git rev-parse <rev>` in a fixture repo (test inspection of
    /// the colocated git side).
    fn git_rev_parse(repo: &std::path::Path, rev: &str) -> String {
        let out = std::process::Command::new("git")
            .args(["rev-parse", rev])
            .current_dir(repo)
            .output()
            .expect("spawn git");
        assert!(out.status.success(), "git rev-parse {rev} failed");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// In-process commit: description lands on `@-`, a fresh empty
    /// `@` opens, and the colocated git HEAD follows.
    #[test]
    fn in_process_commit_updates_colocated_git() {
        let fx = Fixture::new("jjmut");
        std::fs::write(fx.work.join("mut.txt"), "hello").unwrap();
        commit(&fx.work, "test: in-process commit\n\nbody line").unwrap();
        assert_eq!(
            desc_of(&fx.work, "@-").unwrap(),
            "test: in-process commit\n\nbody line"
        );
        assert!(is_empty(&fx.work, "@").unwrap());
        assert_eq!(
            git_rev_parse(&fx.work, "HEAD"),
            cid_of(&fx.work, "@-").unwrap()
        );
    }

    /// In-process bookmark set exports the ref to the colocated git
    /// repo, and describe rewrites reach both sides.
    #[test]
    fn in_process_bookmark_and_describe_export_to_git() {
        let fx = Fixture::new("jjmutbm");
        bookmark_set(&fx.work, "bmtest", "@-").unwrap();
        assert_eq!(
            git_rev_parse(&fx.work, "refs/heads/bmtest"),
            cid_of(&fx.work, "@-").unwrap()
        );
        describe(&fx.work, "@-", "test: rewritten description").unwrap();
        assert_eq!(
            desc_of(&fx.work, "@-").unwrap(),
            "test: rewritten description"
        );
        assert_eq!(
            git_rev_parse(&fx.work, "refs/heads/bmtest"),
            cid_of(&fx.work, "@-").unwrap()
        );
    }

    /// A working-copy read sees the filesystem as it is right now:
    /// the snapshot-first path is live, not the stale op-store view.
    #[test]
    fn wc_relative_reads_snapshot_first() {
        let fx = Fixture::new("jjwcsnap");
        assert!(is_empty(&fx.work, "@").unwrap());
        std::fs::write(fx.work.join("wcsnap.txt"), "x").unwrap();
        assert!(!is_empty(&fx.work, "@").unwrap());
    }

    /// Working-copy forms route to the snapshot-first path.
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

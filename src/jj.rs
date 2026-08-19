//! Typed facade over jj: reads and the publish-path mutations, all
//! in-process through jj-lib (the jj-lib migration stage of
//! `notes/refactor-20260716.md`).
//!
//! Reads resolve through `common::load_repo` +
//! `common::resolve_revset` and `Commit` accessors; a revset that
//! references a working copy (`@`, `@-`, `ws@`) snapshots first
//! (`repo_for_read`), so "right now" questions answer about the
//! filesystem the way the spawned CLI's auto-snapshot did.
//! Mutations are `session::RepoSession` methods; the verb fns here
//! are one-shot wrappers (open a session, run the verb) for
//! context-less callers. Context-ful callers (push, squash-push,
//! sync) go through `Context::session`, which caches one open
//! session per repo for the whole invocation.
//!
//! - `matches` / `rev_exists`: does a revset match / does a
//!   revision resolve.
//! - `chid_of` / `cid_of` / `cid_short_of` / `cids_short_of`:
//!   change / commit ids.
//! - `desc_of` / `is_empty`: description and emptiness.
//! - `local_bookmark_exists` / `non_tracking_remote_of` /
//!   `has_tracked_remote`: typed bookmark and remote-ref queries
//!   over the view (they replaced parsing `jj bookmark list`
//!   output).
//! - `diff_stat`: a `diff --stat`-shaped summary of `@` against
//!   its parents.
//! - `is_no_such_revision`: the typed unresolvable-revision test.
//! - `commit` / `describe` / `bookmark_set` / `git_push_bookmark`:
//!   one-shot wrappers over the session verbs. `git_fetch` has no
//!   wrapper: its only caller (sync) is context-ful.
//!
//! Still spawning `jj` elsewhere: the call sites outside the five
//! verbs (`jj squash`, `jj new`, `jj rebase`, `jj op log` /
//! `op restore`, `jj git clone`, repo-init plumbing).

use std::path::Path;

use futures::AsyncReadExt as _;
use jj_lib::backend::TreeValue;
use jj_lib::commit::Commit;
use jj_lib::diff::ContentDiff;
use jj_lib::diff::DiffHunkKind;
use jj_lib::matchers::EverythingMatcher;
use jj_lib::merged_tree::TreeDiffIterator;
use jj_lib::object_id::ObjectId;
use jj_lib::ref_name::RefName;
use jj_lib::ref_name::RemoteName;
use jj_lib::repo::Repo;
use jj_lib::revset::RevsetResolutionError;
use pollster::FutureExt;

use crate::common;

pub(crate) mod session;

/// Crate-standard boxed-error result, aliased locally for brevity.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// One-shot `RepoSession::commit`: update the wc commit's
/// description and start a new empty change on top (`jj commit`).
pub fn commit(repo: &Path, desc: &str) -> Result<()> {
    session::RepoSession::open(repo)?.commit(desc)
}

/// One-shot `RepoSession::describe`: rewrite `rev`'s description
/// and rebase its descendants (`jj describe -r <rev> -m <desc>`).
pub fn describe(repo: &Path, rev: &str, desc: &str) -> Result<()> {
    session::RepoSession::open(repo)?.describe(rev, desc)
}

/// One-shot `RepoSession::bookmark_set`: point local bookmark
/// `name` at `rev` (`jj bookmark set <name> -r <rev>`).
pub fn bookmark_set(repo: &Path, name: &str, rev: &str) -> Result<()> {
    session::RepoSession::open(repo)?.bookmark_set(name, rev)
}

/// One-shot `RepoSession::git_push_bookmark`: push local bookmark
/// `name` to `origin` (`jj git push --bookmark <name>`).
pub fn git_push_bookmark(repo: &Path, name: &str) -> Result<()> {
    session::RepoSession::open(repo)?.git_push_bookmark(name)
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

/// True when the local bookmark `name` exists (what
/// `jj bookmark list <name>` printing anything used to mean).
pub fn local_bookmark_exists(repo: &Path, name: &str) -> Result<bool> {
    let (_workspace, repo_at_head) = common::load_repo(repo)?;
    let name = RefName::new(name);
    Ok(repo_at_head.view().get_local_bookmark(name).is_present())
}

/// The first non-tracking remote ref of `bookmark`, if any: a
/// remote ref that exists but is not tracked (the refs the
/// `jj bookmark list -a` listing showed at column 0 as
/// `<bookmark>@<remote>: ...`). Returns the remote's name.
pub fn non_tracking_remote_of(repo: &Path, bookmark: &str) -> Result<Option<String>> {
    let (_workspace, repo_at_head) = common::load_repo(repo)?;
    let name = RefName::new(bookmark);
    for (symbol, remote_ref) in repo_at_head.view().all_remote_bookmarks() {
        if symbol.name == name && remote_ref.target.is_present() && !remote_ref.is_tracked() {
            return Ok(Some(symbol.remote.as_str().to_string()));
        }
    }
    Ok(None)
}

/// True when `bookmark` has a present, tracked remote ref at
/// `remote` (what an indented `@<remote>` entry in the `-a`
/// listing used to mean, synced or divergent-decorated alike).
pub fn has_tracked_remote(repo: &Path, bookmark: &str, remote: &str) -> Result<bool> {
    let (_workspace, repo_at_head) = common::load_repo(repo)?;
    let name = RefName::new(bookmark);
    let remote_ref = repo_at_head
        .view()
        .get_remote_bookmark(name.to_remote_symbol(RemoteName::new(remote)));
    Ok(remote_ref.target.is_present() && remote_ref.is_tracked())
}

/// Per-file numbers behind one `diff_stat` line.
struct FileStat {
    path: String,
    insertions: usize,
    deletions: usize,
    /// Files without countable lines render this tag instead of a
    /// `+`/`-` graph: `(binary)`, `(conflict)`, `(symlink)`, ...
    special: Option<&'static str>,
}

/// Line count of `bytes` the way `diff --stat` counts: every
/// newline-terminated line plus a trailing unterminated one.
fn stat_line_count(bytes: &[u8]) -> usize {
    bytes.split_inclusive(|b| *b == b'\n').count()
}

/// One side's plain-file content for the stat's line counts.
/// `None` when the side has no countable content (absent is
/// `Some(vec![])`, so added and removed files count all lines).
fn stat_side_content(
    store: &std::sync::Arc<jj_lib::store::Store>,
    path: &jj_lib::repo_path::RepoPath,
    value: &Option<jj_lib::backend::TreeValue>,
) -> Result<Option<Vec<u8>>> {
    match value {
        None => Ok(Some(Vec::new())),
        Some(jj_lib::backend::TreeValue::File { id, .. }) => {
            let mut reader = store.read_file(path, id).block_on()?;
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).block_on()?;
            Ok(Some(buf))
        }
        Some(_) => Ok(None),
    }
}

/// Render a `jj diff --stat`-shaped summary of the working-copy
/// commit (`@`) against its parents, in-process: one
/// `<path> | <total> <graph>` line per changed file and the
/// `N files changed, X insertions(+), Y deletions(-)` summary
/// line, which is emitted even when nothing changed (callers
/// depend on the constant summary line, see push's completion
/// sanity check).
pub fn diff_stat(repo: &Path) -> Result<String> {
    const GRAPH_WIDTH: usize = 40;
    let (workspace, repo_at_head) = repo_for_read(repo, "@")?;
    let ids = common::resolve_revset(&workspace, &repo_at_head, "@")?;
    let [id] = ids.as_slice() else {
        return Err(
            format!("jj::diff_stat: expected exactly one commit for @, got {ids:?}").into(),
        );
    };
    let commit = repo_at_head.store().get_commit(id)?;
    let parent_tree = commit.parent_tree(repo_at_head.as_ref()).block_on()?;
    let commit_tree = commit.tree();

    let mut stats: Vec<FileStat> = Vec::new();
    for entry in TreeDiffIterator::new(&parent_tree, &commit_tree, &EverythingMatcher) {
        let diff = entry.values?;
        let before = diff.before.as_resolved().cloned().flatten();
        let after = diff.after.as_resolved().cloned().flatten();
        if matches!(&before, Some(TreeValue::Tree(_))) || matches!(&after, Some(TreeValue::Tree(_)))
        {
            continue;
        }
        if before.is_none() && after.is_none() && diff.before.is_resolved() {
            continue;
        }
        let path = entry.path.as_internal_file_string().to_string();
        let store = repo_at_head.store();
        let mut stat = FileStat {
            path,
            insertions: 0,
            deletions: 0,
            special: None,
        };
        let sides = if diff.before.is_resolved() && diff.after.is_resolved() {
            (
                stat_side_content(store, &entry.path, &before)?,
                stat_side_content(store, &entry.path, &after)?,
            )
        } else {
            (None, None)
        };
        match sides {
            (Some(b), Some(a)) => {
                if b.contains(&0u8) || a.contains(&0u8) {
                    stat.special = Some("(binary)");
                } else {
                    for hunk in ContentDiff::by_line([b.as_slice(), a.as_slice()]).hunks() {
                        if hunk.kind == DiffHunkKind::Different {
                            stat.deletions += stat_line_count(hunk.contents[0]);
                            stat.insertions += stat_line_count(hunk.contents[1]);
                        }
                    }
                }
            }
            _ => {
                stat.special = if !diff.before.is_resolved() || !diff.after.is_resolved() {
                    Some("(conflict)")
                } else {
                    Some("(special)")
                };
            }
        }
        stats.push(stat);
    }

    let path_width = stats
        .iter()
        .map(|s| s.path.chars().count())
        .max()
        .unwrap_or(0);
    let total_width = stats
        .iter()
        .map(|s| (s.insertions + s.deletions).to_string().len())
        .max()
        .unwrap_or(1);
    let mut out = String::new();
    let mut insertions = 0;
    let mut deletions = 0;
    for s in &stats {
        insertions += s.insertions;
        deletions += s.deletions;
        let total = s.insertions + s.deletions;
        let graph = match s.special {
            Some(tag) => tag.to_string(),
            None => {
                let scale = |n: usize| {
                    if total <= GRAPH_WIDTH {
                        n
                    } else {
                        // Round-to-nearest keeps a one-line change visible.
                        (n * GRAPH_WIDTH + total / 2) / total
                    }
                };
                format!(
                    "{}{}",
                    "+".repeat(scale(s.insertions)),
                    "-".repeat(scale(s.deletions))
                )
            }
        };
        out.push_str(&format!(
            "{:<path_width$} | {:>total_width$} {}\n",
            s.path, total, graph
        ));
    }
    let plural = |n: usize, one: &str, many: &str| {
        if n == 1 {
            format!("{n} {one}")
        } else {
            format!("{n} {many}")
        }
    };
    out.push_str(&format!(
        "{} changed, {}(+), {}(-)\n",
        plural(stats.len(), "file", "files"),
        plural(insertions, "insertion", "insertions"),
        plural(deletions, "deletion", "deletions"),
    ));
    Ok(out)
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

    /// The typed bookmark queries read the view: existence is the
    /// local bookmark's presence, and a fixture with no remotes has
    /// neither tracked nor non-tracking remote refs.
    #[test]
    fn bookmark_queries_read_the_view() {
        let fx = Fixture::new("jjbmq");
        bookmark_set(&fx.work, "bmq", "@-").unwrap();
        assert!(local_bookmark_exists(&fx.work, "bmq").unwrap());
        assert!(!local_bookmark_exists(&fx.work, "absent").unwrap());
        assert!(!has_tracked_remote(&fx.work, "bmq", "origin").unwrap());
        assert_eq!(non_tracking_remote_of(&fx.work, "bmq").unwrap(), None);
    }

    /// Tracked and non-tracking remote refs are told apart the way
    /// the retired `-a` listing parsers did: a pushed bookmark is
    /// tracked, an untracked one reports its remote. Fixture setup
    /// drives the real jj against a bare origin (integration-type).
    #[test]
    fn remote_ref_queries_follow_track_state() {
        let fx = Fixture::new("jjbmrq");
        jj_ok(&fx.work, &["bookmark", "create", "bmr", "-r", "@-"]);
        jj_ok(&fx.work, &["git", "push", "--bookmark", "bmr"]);
        assert!(has_tracked_remote(&fx.work, "bmr", "origin").unwrap());
        assert_eq!(non_tracking_remote_of(&fx.work, "bmr").unwrap(), None);
        jj_ok(&fx.work, &["bookmark", "untrack", "bmr@origin"]);
        assert!(!has_tracked_remote(&fx.work, "bmr", "origin").unwrap());
        assert_eq!(
            non_tracking_remote_of(&fx.work, "bmr").unwrap(),
            Some("origin".to_string())
        );
    }

    /// diff_stat counts the working copy's lines the way
    /// `jj diff --stat` did: a clean repo still prints the constant
    /// summary line, adds and edits count per file, and the total
    /// line pluralizes.
    #[test]
    fn diff_stat_counts_wc_changes() {
        let fx = Fixture::new("jjstat");
        let clean = diff_stat(&fx.work).unwrap();
        assert!(
            clean.contains("0 files changed, 0 insertions(+), 0 deletions(-)"),
            "clean stat: {clean}"
        );
        std::fs::write(fx.work.join("a.txt"), "one\ntwo\n").unwrap();
        let s = diff_stat(&fx.work).unwrap();
        assert!(s.contains("a.txt"), "stat: {s}");
        assert!(
            s.contains("1 file changed, 2 insertions(+), 0 deletions(-)"),
            "stat: {s}"
        );
        commit(&fx.work, "test: seed a.txt").unwrap();
        std::fs::write(fx.work.join("a.txt"), "one\nTWO\nthree\n").unwrap();
        let s = diff_stat(&fx.work).unwrap();
        assert!(
            s.contains("1 file changed, 2 insertions(+), 1 deletion(-)"),
            "stat: {s}"
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

    /// bugs.md #1 in-process: a transiently held `.git/index.lock`
    /// no longer fails the mutation; the session's git half retries
    /// until the holder lets go (here, a thread releasing it well
    /// inside the backoff budget).
    #[test]
    fn mutation_survives_transient_index_lock() {
        let fx = Fixture::new("jjlockretry");
        let lock = fx.work.join(".git").join("index.lock");
        std::fs::write(&lock, "").expect("plant index.lock");
        let unlock = std::thread::spawn({
            let lock = lock.clone();
            move || {
                std::thread::sleep(std::time::Duration::from_millis(120));
                std::fs::remove_file(&lock).expect("release index.lock");
            }
        });
        bookmark_set(&fx.work, "lockbm", "@-").expect("bookmark_set retries past the lock");
        unlock.join().expect("join unlock thread");
        assert_eq!(
            git_rev_parse(&fx.work, "refs/heads/lockbm"),
            cid_of(&fx.work, "@-").unwrap()
        );
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

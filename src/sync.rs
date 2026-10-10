//! The `sync` subcommand: fetch + classify + rebase / fast-forward
//! a workspace's repos against their remotes, then reposition `@`
//! onto the synced bookmark.
//!
//! `--bookmark` names a **work-repo** bookmark only: the bot
//! repo is a linear journal on `main` by design, so every
//! per-repo step (tracking preflight, classify, act, reposition)
//! resolves its bookmark via `repo_bookmark`, which pins the
//! bot repo to `main`.
//!
//! Sync is a single atomic operation: verify-then-act happens
//! inside one invocation against one fetch snapshot (a separate
//! check-then-apply pair of runs would race the remote).
//!
//! **Safe by default**: before it fetches, sync finds out what the
//! fetch would do, and holds a repo back only where that would
//! tangle local work (see `look_repos`). A fetch is not a look: it
//! moves a tracked bookmark that is behind, conflicts one that
//! diverged, and, when the remote rewrote a commit the clone has,
//! rebases `@` onto the rewrite and can leave conflict markers in a
//! file. So sync:
//!
//! - asks the remote where each of its bookmarks is, fetching nothing
//!   (`jj::git_ls_remote_heads`), and compares with the last fetch.
//!   A remote that has not moved means nothing to fetch.
//! - downloads the commits of the bookmarks that moved, with no ref
//!   moved (`jj::git_download`), and tells each move apart by
//!   ancestry (`Move`): a fast-forward replaces nothing and so
//!   cannot touch local work, a rewrite can.
//! - holds the repo back when a bookmark that moved has local commits
//!   of its own, or the remote rewrote one and the repo holds local
//!   work (`local_work`). It says why, syncs the other repos, and
//!   ends in an error naming it. `--force` syncs it anyway.
//!
//! **`--dry-run`** does the same finding out and stops there: it
//! reports each repo's state, the local work, and what a sync would
//! do, and moves no bookmark, `@`, or remote-tracking ref.
//!
//! **Stop-on-error**: a failure leaves state where the failing step
//! stopped so the user can inspect it. Each repo's pre-sync op id
//! is captured in memory (`jj::current_op_id`) and the error report
//! prints it with the explicit `jj op restore <op> -R <repo>` undo.
//! Nothing persists across invocations: a stale snapshot adopted
//! later is how the push state file corrupted published history
//! (bugs.md #8), so sync keeps none.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{LevelFilter, debug, info, warn};

use crate::common::resolve_repos;
use crate::context::Context;
use crate::jj;
use crate::options_flags::scope::{Scope, parse_scope};
use crate::subcommand::SubcommandRunner;

/// Fetch and sync a set of repos to their remotes.
///
/// One atomic operation: fetch, classify, fast-forward / rebase the
/// bookmark as needed, then reposition `@` onto it. On any failure
/// the starting state of every repo is restored via `jj op restore`.
///
/// Repo set is resolved from `-R/--repo` + `--scope`:
///
/// - `-R PATH`: workspace root, or a single repo to sync alone.
/// - `--scope=work|agent|work,agent`: keyword role selection,
///   resolved via the workspace root's `.vc-config.toml`.
/// - Neither: `.`, the one repo the command is run in, whichever
///   side of a workspace that is, with no walk up from a
///   subdirectory. Both repos are asked for with `--scope=both`.
#[derive(Args, Debug)]
pub struct SyncArgs {
    /// Suppress all informational output (exit code signals result)
    #[arg(short, long)]
    pub quiet: bool,

    /// Bookmark to sync in the work repo. The session (bot) repo
    /// is a linear journal and always syncs `main`, regardless.
    #[arg(long, default_value = "main")]
    pub bookmark: String,

    /// Remote to sync against
    #[arg(long, default_value = "origin")]
    pub remote: String,

    /// Sync a repo even where sync would hold it back, rebasing what
    /// the sync needs rebased. `--rebase` is the same flag.
    ///
    /// Sync holds a repo back when what the remote did would tangle
    /// local work: a bookmark that moved on the remote has local
    /// commits of its own, or the remote rewrote a bookmark and the
    /// repo holds local work. With `--force` the repo is fetched all
    /// the same, a diverged bookmark is rebased onto its remote, and
    /// a work repo `@` that carries changes onto the synced bookmark.
    /// The bot repo's `@` is never rebased (it always `jj new main`).
    #[arg(long, visible_alias = "rebase")]
    pub force: bool,

    /// Look only: report each repo's local work and where it stands
    /// against its remote, and change nothing.
    ///
    /// Nothing is fetched. The remote is asked which commit its
    /// bookmarks are, the commits of those that moved are downloaded
    /// with no ref moved, and the report says what a sync would do:
    /// nothing, follow the remote, or be held back and why. Exits 0
    /// whenever the look itself succeeded, whatever it found.
    #[arg(long)]
    pub dry_run: bool,

    /// Workspace root, or a single jj repo to sync on its own.
    ///
    /// - `-R PATH` alone: sync just the repo at PATH.
    /// - `-R PATH -s ROLES`: use PATH as the workspace root and
    ///   sync the named side(s).
    #[arg(short = 'R', long = "repo", value_name = "PATH", verbatim_doc_comment)]
    pub repo: Option<PathBuf>,

    /// Which repo(s) of the workspace to sync.
    ///
    /// `SCOPE=work|agent|both`:
    ///
    /// - `work`: sync only the work repo.
    /// - `agent`: sync only the bot repo (errors if no bot repo
    ///   is configured).
    /// - `both` (or `work,agent`): sync both repos.
    ///
    /// Composes with `-R` as the workspace root. With neither,
    /// sync acts on `.`, the one repo it is run in.
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = parse_scope,
        verbatim_doc_comment
    )]
    pub scope: Option<Scope>,
}

/// Inputs to the sync op, flat, owned, clap-free.
///
/// - `quiet`: `-q` / `--quiet`, clamp output to `Warn` for the run.
/// - `bookmark`: bookmark to sync in the work repo (default
///   `main`). The bot repo always syncs `main`.
/// - `remote`: remote to sync against (default `origin`).
/// - `force`: `--force` / `--rebase`, the go to sync a repo sync
///   would hold back (see `holds`), rebasing a diverged bookmark (see
///   `act_on_state`) and a non-empty work repo `@` (see
///   `reposition_work`).
/// - `dry_run`: `--dry-run`, look only (see `look_repos`).
/// - `repo`: `-R/--repo` path (None => discover the workspace
///   root from cwd).
/// - `scope`: `--scope` parsed (None => `.`, unless `repo` names
///   a path).
pub struct SyncParams {
    pub quiet: bool,
    pub bookmark: String,
    pub remote: String,
    pub force: bool,
    pub dry_run: bool,
    pub repo: Option<PathBuf>,
    pub scope: Option<Scope>,
}

impl From<&SyncArgs> for SyncParams {
    /// Convert clap-derived `SyncArgs` into the flat `SyncParams`
    /// (total: every field copies straight over).
    fn from(a: &SyncArgs) -> Self {
        Self {
            quiet: a.quiet,
            bookmark: a.bookmark.clone(),
            remote: a.remote.clone(),
            force: a.force,
            dry_run: a.dry_run,
            repo: a.repo.clone(),
            scope: a.scope.clone(),
        }
    }
}

impl SubcommandRunner for SyncArgs {
    type Params = SyncParams;

    /// Delegate to the existing `From<&SyncArgs>` impl above
    /// (total: never fails).
    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(SyncParams::from(self))
    }

    /// Run the existing `sync` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        sync(ctx, params)
    }
}

/// Relationship between a local bookmark and its remote counterpart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Local and remote point at the same commit.
    UpToDate,
    /// Local is a strict ancestor of remote: fast-forward possible.
    Behind { local: String, remote: String },
    /// Remote is a strict ancestor of local: nothing to pull in.
    Ahead { local: String, remote: String },
    /// Neither is an ancestor of the other: needs rebase.
    Diverged { local: String, remote: String },
    /// Neither is an ancestor of the other, and the fetch moved the
    /// bookmark to the remote's all the same: the remote rewrote the
    /// local commit, and jj followed the rewrite.
    Rewritten { local: String, remote: String },
    /// The bookmark has no `@<remote>` counterpart.
    NoRemote,
}

/// Per-repo context accumulated between the snapshot and action phases.
#[derive(Debug)]
struct RepoCtx {
    path: PathBuf,
    #[allow(dead_code)]
    op_id: String,
    state: State,
}

/// CLI entry point for the `sync` subcommand.
///
/// Thin wrapper over `sync_repos` that resolves `--scope` into a
/// concrete repo list (`.` with neither flag, by the resolver the
/// read commands share) and forwards the rest. Tests call `sync_repos` directly
/// with absolute fixture paths. `ctx` supplies the repo sessions
/// the fetch and fast-forward verbs run on (`Context::session`).
///
/// When `--quiet` is set, the global log filter is temporarily
/// clamped to `Warn` for the duration of the call and restored on
/// return, so `info!` calls throughout sync (plus any
/// subprocess-stderr routed through `common::run`) go dark. Errors
/// still surface at `Warn` / `Error` so script callers don't lose
/// diagnostics.
pub fn sync(ctx: &mut Context, params: &SyncParams) -> Result<(), Box<dyn std::error::Error>> {
    let repos = resolve_repos(params.repo.as_deref(), params.scope.as_ref())?;
    if params.quiet {
        let prev = log::max_level();
        log::set_max_level(LevelFilter::Warn);
        let result = sync_repos(ctx, &repos, params);
        log::set_max_level(prev);
        result
    } else {
        sync_repos(ctx, &repos, params)
    }
}

/// Sync the given repos against their remotes.
///
/// Orchestrates the full flow: tracking preflight on every repo,
/// the look (see `look_repos`), snapshot each repo that is to be
/// fetched its current op id in memory, then hand off to `run_plan`
/// for fetch + classify + act, then reposition `@`.
///
/// The look sorts the repos three ways. One it holds back (see
/// `holds`) is not fetched: the others sync, and the run then ends
/// in an error naming the held ones, so a caller with only the exit
/// status learns they were not synced. One whose remote has not
/// moved and which holds local work is settled as it is, there being
/// nothing to fetch. The rest are fetched. `--force` skips the look
/// and fetches every repo.
///
/// **Stop-on-error**: a failure leaves every repo exactly where the
/// failing step stopped so the user can inspect what happened:
/// nothing is auto-reverted. The error report names each repo's
/// pre-sync op id with the explicit `jj op restore` undo. The
/// snapshots die with the invocation.
///
/// Paths may be relative (resolved against the process cwd) or
/// absolute. Tests use absolute tempdir paths to avoid cwd dependence
/// under parallel `cargo test`.
pub fn sync_repos(
    ctx: &mut Context,
    repos: &[PathBuf],
    params: &SyncParams,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!(
        "sync: enter (bookmark={}, remote={})",
        params.bookmark, params.remote
    );

    // Preflight: verify bookmark tracking on every repo before any
    // fetch/rebase. Implements "Non-tracking-remote bookmark detection",
    // design at:
    //   https://github.com/winksaville/vc-x1/blob/main/notes/chores/chores-06.md#non-tracking-remote-bookmark-detection-design
    for repo in repos {
        let bookmark = repo_bookmark(repo, &params.bookmark);
        if bookmark != params.bookmark {
            info!(
                "{}: bot repo, syncing 'main' ('{}' is a work repo bookmark)",
                repo.display(),
                params.bookmark
            );
        }
        crate::common::verify_tracking(repo, bookmark)?;
    }

    if params.dry_run {
        report_look(&look_repos(repos, params)?, params);
        debug!("sync: exit");
        return Ok(());
    }

    // The gate: find out what a fetch would do, and hold back the
    // repos where it would tangle local work. `settled` are the ones
    // with nothing to fetch.
    let mut held: Vec<PathBuf> = Vec::new();
    let mut settled: Vec<PathBuf> = Vec::new();
    if !params.force {
        for l in look_repos(repos, params)? {
            let r = l.path.display();
            if !l.holds.is_empty() {
                info!("{r}: not synced:");
                for why in &l.holds {
                    info!("{r}:   {why}");
                }
                held.push(l.path.clone());
            } else if l.moves.is_empty() && !l.work.is_empty() {
                info!(
                    "{r}: {} has not moved, nothing to fetch; local work left untouched:",
                    params.remote
                );
                for item in &l.work {
                    info!("{r}:   {item}");
                }
                settled.push(l.path.clone());
            } else if !l.work.is_empty() {
                info!("{r}: holds local work, which this sync leaves untouched:");
                for item in &l.work {
                    info!("{r}:   {item}");
                }
            }
        }
    }

    // In-memory only: the snapshot exists to be printed by this
    // invocation's failure report, never persisted for a later one.
    let mut snapshots: Vec<(PathBuf, String)> = Vec::new();
    for repo in repos
        .iter()
        .filter(|r| !held.contains(r) && !settled.contains(r))
    {
        let op_id = jj::current_op_id(repo)?;
        debug!("{}: op snapshot = {op_id}", repo.display());
        snapshots.push((repo.clone(), op_id));
    }

    // Run the plan, then reposition `@` onto the freshly-synced
    // bookmark. Both live inside the same stop-on-error region: any
    // failure falls through to the report below with state left in
    // place.
    let result = run_plan(ctx, &snapshots, params).and_then(|()| {
        for (repo, _) in &snapshots {
            reposition_at(repo, repo_bookmark(repo, &params.bookmark), params)?;
        }
        Ok(())
    });

    if let Err(e) = &result {
        warn!("sync failed: {e}");
        warn!("stopping: state left as-is for inspection (no auto-revert)");
        warn!("pre-sync op snapshot per repo:");
        for (repo, op_id) in &snapshots {
            warn!("  {}: op {op_id}", repo.display());
        }
        warn!("undo per repo: jj op restore <op> -R <repo>");
        debug!("sync: exit");
        return result;
    }

    debug!("sync: exit");
    if !held.is_empty() {
        let names: Vec<String> = held.iter().map(|r| r.display().to_string()).collect();
        return Err(format!(
            "not synced: {}: commit and push the local work, or rerun with --force",
            names.join(", ")
        )
        .into());
    }
    Ok(())
}

/// What `repo` holds that a fetch could collide with, one line per
/// finding, empty when no fetch can collide with anything.
///
/// Local work alone holds nothing back: a remote that has not moved
/// brings nothing, and one that only fast-forwarded replaces nothing.
/// It decides when the remote rewrote a bookmark (see `holds`).
///
/// The three checks, each a way local work meets what a fetch does:
///
/// - `@` has uncommitted changes: a fetch that follows a rewritten
///   remote rebases `@`, and an edit to the same lines becomes
///   conflict markers in the file. Not asked of the bot repo, whose
///   `@` holds the running session's writes and is never empty.
/// - `@-` is not on the remote: `@` sits on a local-only commit,
///   which a sync would have to rebase with it.
/// - A bookmark holds commits the remote does not have: it is ahead
///   or diverged, and a fetch of a remote that also moved leaves it
///   conflicted.
///
/// With none of the three, every commit a fetch can touch is one
/// the remote already has, so it can lose nothing and the way back
/// is the old commit. Local-only commits no bookmark names are left
/// out of it: nothing a sync does moves them.
fn local_work(repo: &Path, remote: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let on_remote = format!("::remote_bookmarks(remote=exact:\"{remote}\")");
    let mut work = Vec::new();
    if !is_bot_repo(repo) && !at_is_empty(repo)? {
        work.push("@ has uncommitted changes".to_string());
    }
    if !jj::matches(repo, &format!("@- & {on_remote}"))? {
        let parent = jj::cid_short_of(repo, "@-")?;
        work.push(format!("@- ({parent}) is not on {remote}"));
    }
    let ahead = jj::local_bookmarks_at(repo, &format!("bookmarks() ~ {on_remote}"))?;
    if !ahead.is_empty() {
        work.push(format!(
            "bookmark {} has commits {remote} does not",
            ahead.join(", ")
        ));
    }
    Ok(work)
}

/// How one of the remote's bookmarks moved since the last fetch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    /// The remote has a bookmark this repo has not fetched before.
    New,
    /// The remote's commit descends from the last-fetched one: it
    /// only added commits. Nothing is replaced, so no local work can
    /// be moved by fetching it.
    FastForward,
    /// The remote's commit does not descend from the last-fetched
    /// one: history was rewritten, a forced push. A fetch follows the
    /// rewrite and rebases whatever sat on what was replaced.
    Rewrite,
    /// The remote no longer has the bookmark.
    Gone,
}

impl Move {
    /// The word a report uses for the move.
    fn word(self) -> &'static str {
        match self {
            Move::New => "new",
            Move::FastForward => "fast-forward",
            Move::Rewrite => "rewritten",
            Move::Gone => "deleted",
        }
    }
}

/// One repo as a look found it.
///
/// - `bookmark`, `state`: the synced bookmark and where it stands
///   against the remote's.
/// - `work`: the local work the repo holds (see `local_work`).
/// - `moves`: each of the remote's bookmarks that moved since the
///   last fetch, by name. Empty means a fetch would bring nothing.
/// - `holds`: why a sync would hold the repo back, empty when it
///   would not (see `holds`).
#[derive(Debug)]
pub struct Looked {
    pub path: PathBuf,
    pub bookmark: String,
    pub state: State,
    pub work: Vec<String>,
    pub moves: Vec<(String, Move)>,
    pub holds: Vec<String>,
}

/// Look at each repo: what a fetch would bring, what local work it
/// would meet, and so what a sync would do. No bookmark, `@`, or
/// remote-tracking ref is moved.
///
/// What a look does change: it makes the working-copy snapshot any
/// jj command makes, since "does `@` hold changes" is asked of the
/// files as they are now, and it downloads the commits of the
/// remote's bookmarks that moved into the git store, where no ref
/// names them and jj does not see them. They are the commits a fetch
/// would bring.
pub fn look_repos(
    repos: &[PathBuf],
    params: &SyncParams,
) -> Result<Vec<Looked>, Box<dyn std::error::Error>> {
    let mut looked = Vec::new();
    for repo in repos {
        let bookmark = repo_bookmark(repo, &params.bookmark);
        let remote = &params.remote;
        let theirs = jj::git_ls_remote_heads(repo, remote)?;
        let moves = remote_moves(repo, remote, &theirs)?;
        let work = local_work(repo, remote)?;
        let holds = holds(repo, remote, &moves, &work)?;
        let state = look_state(repo, bookmark, remote, theirs.get(bookmark))?;
        looked.push(Looked {
            path: repo.clone(),
            bookmark: bookmark.to_string(),
            state,
            work,
            moves,
            holds,
        });
    }
    Ok(looked)
}

/// Each of `remote`'s bookmarks that moved since the last fetch, by
/// name, with how it moved.
///
/// `theirs` is where the remote's bookmarks are now
/// (`jj::git_ls_remote_heads`), compared with the remote-tracking
/// refs. For a bookmark on a different commit than last fetched, the
/// commit is downloaded with no ref moved and the two are compared
/// by ancestry in the git store: jj has not imported the remote's
/// commit, so a revset cannot ask.
fn remote_moves(
    repo: &Path,
    remote: &str,
    theirs: &std::collections::BTreeMap<String, String>,
) -> Result<Vec<(String, Move)>, Box<dyn std::error::Error>> {
    let ours = jj::remote_bookmark_targets(repo, remote)?;
    let changed: Vec<String> = theirs
        .iter()
        .filter(|(name, head)| ours.get(*name).is_some_and(|last| last != *head))
        .map(|(name, _)| name.clone())
        .collect();
    jj::git_download(repo, remote, &changed)?;

    let mut moves = Vec::new();
    for (name, head) in theirs {
        match ours.get(name) {
            None => moves.push((name.clone(), Move::New)),
            Some(last) if last == head => {}
            Some(last) if jj::git_is_ancestor(repo, last, head)? => {
                moves.push((name.clone(), Move::FastForward));
            }
            Some(_) => moves.push((name.clone(), Move::Rewrite)),
        }
    }
    for name in ours.keys().filter(|name| !theirs.contains_key(*name)) {
        moves.push((name.clone(), Move::Gone));
    }
    moves.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(moves)
}

/// Why a sync would hold `repo` back, one line per reason, empty
/// when it would not.
///
/// A fetch tangles local work in two ways, and each is one reason:
///
/// - A bookmark that moved on the remote has local commits of its
///   own. The fetch leaves it conflicted, pointing at both.
/// - The remote rewrote or deleted a bookmark, and the repo holds
///   local work (see `local_work`). The fetch rebases what sat on the
///   replaced commits, and an edit to the same lines becomes conflict
///   markers. Which commits were replaced is not worked out: any
///   local work at all is taken as at risk.
///
/// A fast-forward and a new bookmark replace nothing, so whatever
/// local work the repo holds, they hold nothing back.
fn holds(
    repo: &Path,
    remote: &str,
    moves: &[(String, Move)],
    work: &[String],
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut why = Vec::new();
    let mut rewrote = Vec::new();
    for (name, moved) in moves {
        if *moved == Move::New {
            continue;
        }
        let own =
            format!("bookmarks(exact:\"{name}\") ~ ::remote_bookmarks(remote=exact:\"{remote}\")");
        if jj::matches(repo, &own)? {
            why.push(format!(
                "bookmark {name} has commits of its own, and {remote}'s has moved ({})",
                moved.word()
            ));
        }
        if matches!(moved, Move::Rewrite | Move::Gone) {
            rewrote.push(format!("{name} ({})", moved.word()));
        }
    }
    if !rewrote.is_empty() && !work.is_empty() {
        why.push(format!(
            "{remote} rewrote history under local work: {}",
            rewrote.join(", ")
        ));
        why.extend(work.iter().map(|item| format!("local work: {item}")));
    }
    Ok(why)
}

/// Where `bookmark` stands against `remote`'s, whose commit `theirs`
/// is, `None` when the remote has no such bookmark.
///
/// Asked of the git store by ancestry, the remote's commit having
/// been downloaded when it moved (see `remote_moves`) and jj not
/// having imported it:
///
/// - The same commit: `UpToDate`.
/// - The local one is an ancestor of the remote's: `Behind`.
/// - The remote's is an ancestor of the local one: `Ahead`.
/// - Neither, and the local bookmark holds no commits of its own:
///   `Rewritten`, the remote having replaced what was fetched.
/// - Neither, and it does: `Diverged`.
///
/// Returns `NoRemote` when the remote has no such bookmark.
fn look_state(
    repo: &Path,
    bookmark: &str,
    remote: &str,
    theirs: Option<&String>,
) -> Result<State, Box<dyn std::error::Error>> {
    let heads = local_bookmark_heads(repo, bookmark)?;
    let Some(head) = theirs else {
        return Ok(State::NoRemote);
    };
    let remote_short: String = head.chars().take(12).collect();
    let local = match heads.as_slice() {
        [] => {
            return Err(format!("{}: bookmark '{bookmark}' does not exist", repo.display()).into());
        }
        [local] => local.clone(),
        // Conflicted: an earlier fetch's divergence.
        _ => {
            return Ok(State::Diverged {
                local: heads.join(","),
                remote: remote_short,
            });
        }
    };
    if head.starts_with(&local) {
        return Ok(State::UpToDate);
    }
    let local_full = jj::cid_of(repo, &local)?;
    let own = format!("{local} ~ ::remote_bookmarks(remote=exact:\"{remote}\")");
    let remote = remote_short;
    Ok(if jj::git_is_ancestor(repo, &local_full, head)? {
        State::Behind { local, remote }
    } else if jj::git_is_ancestor(repo, head, &local_full)? {
        State::Ahead { local, remote }
    } else if jj::matches(repo, &own)? {
        State::Diverged { local, remote }
    } else {
        State::Rewritten { local, remote }
    })
}

/// What a sync would do with a repo a look found as `looked`, in a
/// phrase: the report's last line for the repo, so a reader is not
/// left to work the answer out from the state and the local work.
fn verdict(looked: &Looked, params: &SyncParams) -> String {
    let remote = &params.remote;
    if !looked.holds.is_empty() && !params.force {
        return "a sync would be held back: commit and push the local work, or pass --force"
            .to_string();
    }
    if looked.moves.is_empty() {
        return "a sync would have nothing to do".to_string();
    }
    if let State::Diverged { .. } = looked.state {
        return format!("a sync would rebase '{}' onto {remote}", looked.bookmark);
    }
    if looked.work.is_empty() || params.force {
        format!("a sync would follow {remote}")
    } else {
        format!("a sync would follow {remote} and leave the local work untouched")
    }
}

/// Print what a look found: per repo its bookmark's state against
/// the remote, the remote's other bookmarks that moved, the local
/// work it holds, why a sync would hold it back, and the verdict
/// (see `verdict`), or the one-line summary when every repo is level
/// and holds no local work.
fn report_look(looked: &[Looked], params: &SyncParams) {
    let level = looked
        .iter()
        .all(|l| l.work.is_empty() && l.moves.is_empty() && l.state == State::UpToDate);
    if level {
        let n = looked.len();
        let noun = if n == 1 { "repo is" } else { "repos are" };
        info!("sync: {n} {noun} {UP_TO_DATE_MSG}");
        return;
    }
    let remote = &params.remote;
    for l in looked {
        let r = l.path.display();
        let b = &l.bookmark;
        match &l.state {
            State::UpToDate => info!("{r}: '{b}' is up to date with {remote}"),
            State::NoRemote => info!("{r}: '{b}' is not on {remote}"),
            State::Ahead {
                local,
                remote: theirs,
            } => {
                info!("{r}: '{b}' is ahead of {remote} (local {local} > remote {theirs})")
            }
            State::Behind {
                local,
                remote: theirs,
            } => info!(
                "{r}: '{b}' is behind {remote} by a fast-forward (local {local}, remote {theirs})"
            ),
            State::Rewritten {
                local,
                remote: theirs,
            } => info!("{r}: {remote} rewrote '{b}' (local {local}, remote {theirs})"),
            State::Diverged {
                local,
                remote: theirs,
            } => {
                info!("{r}: '{b}' has diverged from {remote} (local {local} vs remote {theirs})")
            }
        }
        for (name, moved) in l.moves.iter().filter(|(name, _)| name != b) {
            info!("{r}: {remote} also moved '{name}' ({})", moved.word());
        }
        if !l.work.is_empty() {
            info!("{r}: holds local work:");
            for item in &l.work {
                info!("{r}:   {item}");
            }
        }
        if !l.holds.is_empty() && !params.force {
            info!("{r}: a sync would hold it back because:");
            for why in l
                .holds
                .iter()
                .filter(|why| !why.starts_with("local work: "))
            {
                info!("{r}:   {why}");
            }
        }
        info!("{r}: {}", verdict(l, params));
    }
}

/// Clean-case summary tail ("<N> repo(s) ..." prefixed at the emit
/// site). Shared with main.rs's `long_about` so the documented
/// output shape can't drift from the emitted one.
pub const UP_TO_DATE_MSG: &str = "up to date, nothing to sync";

/// Fetch and classify each repo, then act (or not, in verify-only).
///
/// Returns `Err` on the first failure so the caller can stop and
/// report. Partial progress across repos is left in place for
/// inspection (see `sync_repos`).
fn run_plan(
    ctx: &mut Context,
    snapshots: &[(PathBuf, String)],
    params: &SyncParams,
) -> Result<(), Box<dyn std::error::Error>> {
    // Phase 1: fetch + classify silently.
    //
    // Each repo's fetch stderr is captured (rather than streamed to
    // `info!` via `common::run`) so we can decide after classification
    // whether to surface it. `jj git fetch`'s routine "Nothing
    // changed." chatter is the main thing we're suppressing here:
    // if nothing needs action, the user shouldn't see it.
    let mut fetched: Vec<(PathBuf, Vec<String>)> = Vec::new();
    let mut ctxs: Vec<RepoCtx> = Vec::new();
    //
    // The bookmark is read before the fetch: the fetch moves a
    // tracked bookmark that is behind and conflicts one that
    // diverged, so afterwards neither state can be told from the
    // bookmark alone.
    for (repo, op_id) in snapshots {
        let bookmark = repo_bookmark(repo, &params.bookmark);
        let found = local_bookmark_heads(repo, bookmark)?;
        let had_conflicts = has_conflicts(repo)?;
        let lines = fetch_silent(ctx, repo, &params.remote)?;
        fetched.push((repo.clone(), lines));
        let state = classify(repo, &found, bookmark, &params.remote)?;
        // Only the bot repo's `@`, which the gate does not ask to be
        // empty, or a repo synced on the go can get here conflicted.
        if !had_conflicts && has_conflicts(repo)? {
            return Err(format!(
                "{}: the fetch conflicts with local work (the remote rewrote a commit under it)",
                repo.display()
            )
            .into());
        }
        ctxs.push(RepoCtx {
            path: repo.clone(),
            op_id: op_id.clone(),
            state,
        });
    }

    // Ahead is not level either, and a reader of the report wants to
    // hear of an unpushed bookmark, so only up-to-date is quiet.
    let any_action_needed = ctxs
        .iter()
        .any(|c| !matches!(c.state, State::UpToDate | State::NoRemote));

    // Phase 2: emit status. `--quiet` is enforced globally via the
    // log-level clamp in `sync()`, so these `info!` calls are already
    // suppressed in scripts. We just shape the output here.
    if ctxs.is_empty() {
        // Every repo was held back, and the caller has said so.
    } else if !any_action_needed {
        let n = ctxs.len();
        let noun = if n == 1 { "repo is" } else { "repos are" };
        info!("sync: {n} {noun} {UP_TO_DATE_MSG}");
    } else {
        for (repo, lines) in &fetched {
            info!("{}: fetch {}", repo.display(), params.remote);
            for line in lines {
                info!("{line}");
            }
        }
        for ctx in &ctxs {
            log_state(&ctx.path, &ctx.state);
        }
    }

    // Phase 3: act. Repositioning `@` onto the synced bookmark
    // happens after `run_plan` returns (see `sync_repos`), outside
    // the revert region.
    for repo_ctx in &ctxs {
        act_on_state(ctx, repo_ctx, params)?;
    }

    Ok(())
}

/// Fetch `repo` from `remote` without streaming chatter to `info!`.
///
/// The facade's fetch returns one line per changed remote
/// bookmark. The caller decides whether to surface them (action
/// case) or drop them (clean case).
fn fetch_silent(
    ctx: &mut Context,
    repo: &Path,
    remote: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    debug!("fetch --remote {remote} -R {}", repo.display());
    ctx.session(repo)?.git_fetch(remote)
}

/// Reposition `@` onto the freshly-synced bookmark after a successful
/// sync.
///
/// Dispatches on repo role: the caller has already excluded the
/// deprecated verify-only mode:
///
/// - **session (bot) sub-repo** -> always `jj new main`
///   (see `reposition_bot`).
/// - **any other repo** -> move `@` onto the synced `bookmark` under
///   the work-repo safety rules (see `reposition_work`).
fn reposition_at(
    repo: &Path,
    bookmark: &str,
    params: &SyncParams,
) -> Result<(), Box<dyn std::error::Error>> {
    if is_bot_repo(repo) {
        reposition_bot(repo)
    } else {
        reposition_work(repo, bookmark, params.force)
    }
}

/// True when `repo` is the bot sub-repo.
///
/// Thin wrapper over `common::is_bot_dir`: side detection is by
/// location (the parent's config names this dir as its `agent`), not
/// config content. A POR / single-repo workspace is a work repo.
fn is_bot_repo(repo: &Path) -> bool {
    crate::common::is_bot_dir(repo)
}

/// Bookmark to sync for `repo`.
///
/// `--bookmark` is work-repo-only: the bot repo is a
/// linear journal on `main` by design, so it pins `main` regardless
/// of the requested bookmark. Every other repo uses `bookmark`
/// as passed.
fn repo_bookmark<'a>(repo: &Path, bookmark: &'a str) -> &'a str {
    if is_bot_repo(repo) { "main" } else { bookmark }
}

/// Reposition the bot repo's `@` onto `main`.
///
/// The session (`.claude`) repo is a linear journal on `main`, and its
/// `@` normally carries live session writes:
///
/// - `@-` already the `main` tip -> no-op: `@` is where it belongs,
///   live writes stay in the working copy. (An unconditional
///   `jj new main` here would churn an empty `@`'s chid/op every
///   sync, or strand a non-empty `@`'s live writes (and any ochid
///   captured against its chid) on a sibling head.)
/// - Errors when `@-` isn't on `main` (not an ancestor-or-equal of the
///   bookmark): refuse rather than guess.
/// - Otherwise `main` moved: `jj new main` starts a fresh `@` on the
///   bookmark. The prior `@` becomes a sibling head, which is
///   expected for the journal. A conflict is very unlikely given
///   `.claude`'s content. If one ever appears the user resolves it.
fn reposition_bot(repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let parent = jj::cid_short_of(repo, "@-")?;
    let tip = jj::cid_short_of(repo, "main")?;
    if parent == tip {
        debug!("{}: @ already on 'main'", repo.display());
        return Ok(());
    }
    if !jj::matches(repo, &format!("{parent}::main"))? {
        return Err(format!(
            "{}: @- ({parent}) is not on main: refusing to reposition @",
            repo.display()
        )
        .into());
    }
    info!("{}: jj new main", repo.display());
    jj::new_on(repo, "main")?;
    Ok(())
}

/// Reposition the work repo's `@` onto the synced `bookmark`.
///
/// Let `@-` be the parent of `@`:
///
/// - `bookmark == @-` -> already positioned, no-op.
/// - `bookmark` a proper descendant of `@-`, `@` empty ->
///   `jj new bookmark` (jj auto-abandons the old empty `@`).
/// - `bookmark` a proper descendant of `@-`, `@` non-empty -> rebase
///   `@` onto `bookmark`, but only with `rebase` set. Without it the
///   gate has held the repo back before this (see `local_work`), and
///   `@` is left in place all the same should one get here.
/// - `bookmark` not a descendant of `@-` (diverged / `@` ahead) ->
///   leave `@` and inform why it didn't move.
fn reposition_work(
    repo: &Path,
    bookmark: &str,
    rebase: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let parent = jj::cid_short_of(repo, "@-")?;
    let tip = jj::cid_short_of(repo, bookmark)?;
    if tip == parent {
        debug!("{}: @ already on '{bookmark}'", repo.display());
        return Ok(());
    }
    if !jj::matches(repo, &format!("{parent}::{tip}"))? {
        info!(
            "{}: @- ({parent}) is not behind '{bookmark}' ({tip}); leaving @ in place",
            repo.display()
        );
        return Ok(());
    }
    // `bookmark` is a proper descendant of `@-`: safe to move a clean `@`.
    if at_is_empty(repo)? {
        info!("{}: jj new {bookmark}", repo.display());
        jj::new_on(repo, bookmark)?;
        return Ok(());
    }
    // `@` carries changes: rebase only on the go.
    if !rebase {
        info!(
            "{}: @ has changes; left in place (pass --force to rebase onto '{bookmark}')",
            repo.display()
        );
        return Ok(());
    }
    info!("{}: rebasing @ onto '{bookmark}'", repo.display());
    jj::rebase_branch(repo, "@", bookmark)?;
    if has_conflicts(repo)? {
        return Err(format!(
            "{}: rebase of @ onto '{bookmark}' produced conflicts",
            repo.display()
        )
        .into());
    }
    Ok(())
}

/// True when the working-copy commit `@` is empty (no changes).
fn at_is_empty(repo: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    jj::matches(repo, "@ & empty()")
}

/// Perform the mutation corresponding to `repo_ctx.state` (skipped
/// in deprecated verify-only mode).
///
/// - `UpToDate` / `Ahead` / `NoRemote` -> no-op (and no output, the state
///   was already logged by `log_state`).
/// - `Behind` -> `jj bookmark set <b> -r <b>@<remote>` to fast-forward.
///   The fetch has usually moved a tracked bookmark there already, and
///   the set is what covers the one it did not.
/// - `Rewritten` -> no-op: the fetch followed the rewrite.
/// - `Diverged` -> only on the go, `--force`: `jj rebase -b <b> -d
///   <b>@<remote>`, then probe `conflicts()`. A non-empty result means
///   the rebase produced conflicted commits, and the `Err` stops the
///   run with that state in place. Without the go the gate has held
///   the repo back (see `local_work`), and one that gets here anyway
///   is an `Err` too.
fn act_on_state(
    ctx: &mut Context,
    repo_ctx: &RepoCtx,
    params: &SyncParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo = &repo_ctx.path;
    let bookmark = repo_bookmark(repo, &params.bookmark);
    let remote_rev = format!("{}@{}", bookmark, params.remote);
    match &repo_ctx.state {
        State::UpToDate | State::Ahead { .. } | State::NoRemote => Ok(()),
        State::Behind { .. } => {
            info!("{}: setting '{bookmark}' to {remote_rev}", repo.display());
            ctx.session(repo)?.bookmark_set(bookmark, &remote_rev)?;
            Ok(())
        }
        State::Rewritten { .. } => {
            info!("{}: '{bookmark}' followed the rewrite", repo.display());
            Ok(())
        }
        State::Diverged { local, remote } => {
            if !params.force {
                return Err(format!(
                    "{}: '{bookmark}' diverged: rerun with --force to rebase it onto {remote_rev}",
                    repo.display()
                )
                .into());
            }
            // `local` is either a single commit id or a comma-joined list
            // of heads when the bookmark was conflicted before the fetch.
            // Pick the head that isn't the remote: that's the local-only
            // tip.
            let local_head = local
                .split(',')
                .find(|h| *h != remote)
                .unwrap_or(local.as_str());
            info!(
                "{}: rebasing {local_head} onto {remote_rev}",
                repo.display()
            );
            ctx.session(repo)?.rebase_branch(local_head, &remote_rev)?;
            if has_conflicts(repo)? {
                return Err(format!("{}: rebase produced conflicts", repo.display()).into());
            }
            Ok(())
        }
    }
}

/// Print a single-line summary of `state` at `info!` level.
fn log_state(repo: &Path, state: &State) {
    let r = repo.display();
    match state {
        State::UpToDate => info!("{r}: up-to-date"),
        State::NoRemote => info!("{r}: no remote counterpart, skipping"),
        State::Ahead { local, remote } => {
            info!("{r}: ahead (local {local} > remote {remote}); nothing to sync")
        }
        State::Behind { local, remote } => {
            info!("{r}: behind (local {local} < remote {remote}); fast-forward needed")
        }
        State::Diverged { local, remote } => {
            info!("{r}: diverged (local {local} vs remote {remote}); rebase needed")
        }
        State::Rewritten { local, remote } => {
            info!("{r}: rewritten (local {local}, remote {remote}); following the remote")
        }
    }
}

/// Classify the relationship between `bookmark` as found and
/// `bookmark@remote` as fetched.
///
/// `found` is the bookmark's heads read before the fetch (see
/// `local_bookmark_heads`), since the fetch moves a tracked bookmark
/// and the bookmark afterwards no longer says where the repo was.
/// When `found` has multiple heads the bookmark was already
/// conflicted and the repo is `Diverged` by definition. Otherwise we
/// compare the single local head against the single remote commit via
/// two revset-ancestry probes. Neither being an ancestor is
/// `Diverged`, unless the fetch has put the bookmark on the remote's
/// commit regardless, which it does only when that commit is a
/// rewrite of the local one: `Rewritten`.
///
/// Returns `NoRemote` when `<b>@<remote>` does not resolve: the caller
/// logs a skip and moves on.
fn classify(
    repo: &Path,
    found: &[String],
    bookmark: &str,
    remote: &str,
) -> Result<State, Box<dyn std::error::Error>> {
    let local_heads = found.to_vec();
    let remote_rev = format!("{bookmark}@{remote}");
    let Some(remote) = try_commit_id(repo, &remote_rev)? else {
        return Ok(State::NoRemote);
    };
    if local_heads.is_empty() {
        return Err(format!("{}: bookmark '{bookmark}' does not exist", repo.display()).into());
    }
    if local_heads.len() > 1 {
        // Conflicted before the fetch: an earlier fetch's divergence.
        let local = local_heads.join(",");
        return Ok(State::Diverged { local, remote });
    }
    // OK: `len() > 1` arm handled above, `is_empty()` handled above
    let local = local_heads.into_iter().next().unwrap();
    if local == remote {
        return Ok(State::UpToDate);
    }
    let local_is_anc = jj::matches(repo, &format!("{local}::{remote_rev}"))?;
    let remote_is_anc = jj::matches(repo, &format!("{remote_rev}::{local}"))?;
    Ok(match (local_is_anc, remote_is_anc) {
        (true, _) => State::Behind { local, remote },
        (false, true) => State::Ahead { local, remote },
        (false, false) if local_bookmark_heads(repo, bookmark)? == [remote.clone()] => {
            State::Rewritten { local, remote }
        }
        (false, false) => State::Diverged { local, remote },
    })
}

/// Return all commit ids the `bookmark` currently points at.
///
/// Normally a one-element vector. Two or more elements indicate a
/// conflicted bookmark (jj's representation of diverged post-fetch
/// state, where the local bookmark has multiple heads).
fn local_bookmark_heads(
    repo: &Path,
    bookmark: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    jj::cids_short_of(repo, &format!("bookmarks(exact:{bookmark})"))
}

/// Like `jj::cid_short_of`, but `Ok(None)` when the revset doesn't resolve.
///
/// The unresolvable-revision error (`jj::is_no_such_revision`)
/// maps to
/// `Ok(None)` so callers can distinguish "missing" from "other
/// failure".
fn try_commit_id(repo: &Path, rev: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    match jj::cid_short_of(repo, rev) {
        Ok(id) if id.is_empty() => Ok(None),
        Ok(id) => Ok(Some(id)),
        Err(e) if jj::is_no_such_revision(e.as_ref()) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Return `true` when `repo` has any conflicted commits.
fn has_conflicts(repo: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    jj::matches(repo, "conflicts()")
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

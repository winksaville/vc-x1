//! The `sync` subcommand: fetch + classify + rebase / fast-forward
//! a workspace's repos against their remotes, then reposition `@`
//! onto the synced bookmark.
//!
//! `--bookmark` names a **code-repo** bookmark only: the session
//! (bot) repo is a linear journal on `main` by design, so every
//! per-repo step (tracking preflight, classify, act, reposition)
//! resolves its bookmark via `repo_bookmark`, which pins the
//! session repo to `main`.
//!
//! Sync is a single atomic operation — verify-then-act happens
//! inside one invocation against one fetch snapshot (a separate
//! check-then-apply pair of runs would race the remote). The
//! hidden deprecated `--check` flag preserves the old verify-only
//! mode for `push`'s preflight until that is rewired in-process.
//!
//! **Stop-on-error**: a failure leaves state where the failing step
//! stopped so the user can inspect it. Each repo's pre-sync op id
//! is persisted to `.vc-x1/sync-state.toml` (see `state`); the
//! error report points at `vc-x1 revert` to undo explicitly.
//!
//! `current_op_id` / `op_restore` are `pub(crate)` so `push` (and
//! the `revert` subcommand) reuse the same snapshot-and-restore
//! primitives.

use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Args;
use log::{LevelFilter, debug, info, warn};

use crate::common::{default_scope, find_workspace_root, prompt, run, scope_to_repos};
use crate::context::Context;
use crate::desc_helpers::VC_CONFIG_FILE;
use crate::options_flags::scope::{Scope, parse_scope};
use crate::subcommand::SubcommandRunner;
use crate::toml_simple;

/// Fetch and sync a set of repos to their remotes.
///
/// One atomic operation: fetch, classify, fast-forward / rebase the
/// bookmark as needed, then reposition `@` onto it. On any failure
/// the starting state of every repo is restored via `jj op restore`.
///
/// Repo set is resolved from `-R/--repo` + `--scope`:
///
/// - `-R PATH` — workspace root, or a single repo to sync alone.
/// - `--scope=code|bot|code,bot` — keyword role selection,
///   resolved via the workspace root's `.vc-config.toml`.
/// - Neither — workspace-default scope:
///   - dual workspace (`.vc-config.toml` with `other-repo`) → `code,bot`
///   - single-repo workspace (`.vc-config.toml`, no `other-repo`) → `code`
///   - POR (no `.vc-config.toml`) → cwd
#[derive(Args, Debug)]
pub struct SyncArgs {
    /// Verify only — fetch + classify; error if any repo needs
    /// action; no bookmark move, no `@` reposition.
    ///
    /// Deprecated and hidden: kept solely for `push`'s preflight
    /// shell-out until that is rewired in-process (see the
    /// `notes/todo.md` sync follow-up). Note the fetch still
    /// auto-fast-forwards a tracked bookmark — this mode was never
    /// fully read-only.
    #[arg(long, hide = true)]
    pub check: bool,

    /// Suppress all informational output (exit code signals result)
    #[arg(short, long)]
    pub quiet: bool,

    /// Bookmark to sync in the code repo. The session (bot) repo
    /// is a linear journal and always syncs `main`, regardless.
    #[arg(long, default_value = "main")]
    pub bookmark: String,

    /// Remote to sync against
    #[arg(long, default_value = "origin")]
    pub remote: String,

    /// Rebase a non-empty `@` onto the synced bookmark without asking.
    ///
    /// After a successful sync, `@` is repositioned onto the synced
    /// bookmark. When the code repo's `@` carries changes it normally
    /// asks before rebasing; `--rebase` answers yes up front for
    /// non-interactive use. Ignored for the session repo (which
    /// always `jj new main`).
    #[arg(long)]
    pub rebase: bool,

    /// Workspace root, or a single jj repo to sync on its own.
    ///
    /// - `-R PATH` alone — sync just the repo at PATH.
    /// - `-R PATH -s ROLES` — use PATH as the workspace root and
    ///   sync the named side(s).
    #[arg(short = 'R', long = "repo", value_name = "PATH", verbatim_doc_comment)]
    pub repo: Option<PathBuf>,

    /// Which repo(s) of the workspace to sync.
    ///
    /// `SCOPE=code|bot|code,bot`:
    ///
    /// - `code` — sync only the app repo.
    /// - `bot` — sync only the bot repo (errors if no bot repo
    ///   is configured).
    /// - `code,bot` — sync both repos.
    ///
    /// Composes with `-R` as the workspace root. Default depends
    /// on workspace state: dual workspace → `code,bot`;
    /// single-repo workspace or POR → `code`.
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
/// - `quiet`: `-q` / `--quiet` — clamp output to `Warn` for the run.
/// - `bookmark`: bookmark to sync in the code repo (default
///   `main`); the session repo always syncs `main`.
/// - `remote`: remote to sync against (default `origin`).
/// - `check`: hidden deprecated `--check` — verify-only mode
///   (fetch + classify + report, error if action needed; no
///   bookmark move, no `@` reposition). Absent ⇒ the normal
///   atomic sync.
/// - `rebase`: `--rebase` — rebase a non-empty `@` onto the synced
///   bookmark without prompting (code repo only; see
///   `reposition_code`).
/// - `repo`: `-R/--repo` path (None ⇒ discover the workspace
///   root from cwd).
/// - `scope`: `--scope` parsed (None ⇒ resolve via the
///   workspace-default scope at run time).
pub struct SyncParams {
    pub quiet: bool,
    pub bookmark: String,
    pub remote: String,
    pub check: bool,
    pub rebase: bool,
    pub repo: Option<PathBuf>,
    pub scope: Option<Scope>,
}

impl From<&SyncArgs> for SyncParams {
    /// Convert clap-derived `SyncArgs` into the flat `SyncParams`
    /// (total — every field copies straight over).
    fn from(a: &SyncArgs) -> Self {
        Self {
            quiet: a.quiet,
            bookmark: a.bookmark.clone(),
            remote: a.remote.clone(),
            check: a.check,
            rebase: a.rebase,
            repo: a.repo.clone(),
            scope: a.scope.clone(),
        }
    }
}

impl SubcommandRunner for SyncArgs {
    type Params = SyncParams;

    /// Delegate to the existing `From<&SyncArgs>` impl above
    /// (total — never fails).
    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(SyncParams::from(self))
    }

    /// Run the existing `sync` op.
    fn run(ctx: &Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        sync(ctx, params)
    }
}

/// Resolve a `-R`/`--scope` pair into a concrete repo list.
///
/// `-R` and `--scope` compose:
///
/// - neither — workspace-default scope against the discovered root.
/// - `-R PATH` alone — just that repo.
/// - `-s ROLES` alone — roles against the discovered workspace root.
/// - `-R PATH -s ROLES` — roles against `PATH` as the workspace root.
///
/// `pub(crate)` — shared by `sync` and `revert`, which must resolve
/// the same invocation shape to the same repos.
pub(crate) fn resolve_repos(
    repo: &Option<PathBuf>,
    scope: &Option<Scope>,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    match (repo, scope) {
        (None, None) => {
            let root = find_workspace_root();
            scope_to_repos(&default_scope(root.as_deref()), root.as_deref())
        }
        (Some(p), None) => Ok(vec![p.clone()]),
        (None, Some(s)) => scope_to_repos(s, find_workspace_root().as_deref()),
        (Some(p), Some(s)) => scope_to_repos(s, Some(p)),
    }
}

/// Relationship between a local bookmark and its remote counterpart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Local and remote point at the same commit.
    UpToDate,
    /// Local is a strict ancestor of remote — fast-forward possible.
    Behind { local: String, remote: String },
    /// Remote is a strict ancestor of local — nothing to pull in.
    Ahead { local: String, remote: String },
    /// Neither is an ancestor of the other — needs rebase.
    Diverged { local: String, remote: String },
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
/// concrete repo list (falling back to the workspace-default
/// scope) and forwards the rest. Tests call `sync_repos` directly
/// with absolute fixture paths. `ctx` is unused today — present
/// for the uniform subcommand-layer signature.
///
/// When `--quiet` is set, the global log filter is temporarily
/// clamped to `Warn` for the duration of the call and restored on
/// return, so `info!` calls throughout sync (plus any
/// subprocess-stderr routed through `common::run`) go dark. Errors
/// still surface at `Warn` / `Error` so script callers don't lose
/// diagnostics.
pub fn sync(_ctx: &Context, params: &SyncParams) -> Result<(), Box<dyn std::error::Error>> {
    let repos = resolve_repos(&params.repo, &params.scope)?;
    if params.quiet {
        let prev = log::max_level();
        log::set_max_level(LevelFilter::Warn);
        let result = sync_repos(&repos, params);
        log::set_max_level(prev);
        result
    } else {
        sync_repos(&repos, params)
    }
}

/// Sync the given repos against their remotes.
///
/// Orchestrates the full flow: pre-flight clean-check on every repo,
/// snapshot each repo's current op id (persisted to
/// `.vc-x1/sync-state.toml` per repo), then hand off to `run_plan`
/// for fetch + classify + act, then reposition `@`.
///
/// **Stop-on-error**: a failure leaves every repo exactly where the
/// failing step stopped so the user can inspect what happened —
/// nothing is auto-reverted. The error report names each repo's
/// pre-sync op id and points at `vc-x1 revert`, which consumes the
/// persisted snapshots. On full success the snapshots are cleared
/// (a stale file must not become a revert target later).
///
/// Paths may be relative (resolved against the process cwd) or
/// absolute. Tests use absolute tempdir paths to avoid cwd dependence
/// under parallel `cargo test`.
pub fn sync_repos(
    repos: &[PathBuf],
    params: &SyncParams,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!(
        "sync: enter (check={}, bookmark={}, remote={})",
        params.check, params.bookmark, params.remote
    );

    // Preflight: verify bookmark tracking on every repo before any
    // fetch/rebase. Implements "Non-tracking-remote bookmark detection"
    // — design at:
    //   https://github.com/winksaville/vc-x1/blob/main/notes/chores/chores-06.md#non-tracking-remote-bookmark-detection-design
    for repo in repos {
        let bookmark = repo_bookmark(repo, &params.bookmark);
        if bookmark != params.bookmark {
            info!(
                "{}: session repo — syncing 'main' ('{}' is a code repo bookmark)",
                repo.display(),
                params.bookmark
            );
        }
        crate::common::verify_tracking(repo, bookmark)?;
    }

    let mut snapshots: Vec<(PathBuf, String)> = Vec::new();
    for repo in repos {
        let op_id = current_op_id(repo)?;
        debug!("{}: op snapshot = {op_id}", repo.display());
        state::save(
            repo,
            &op_id,
            repo_bookmark(repo, &params.bookmark),
            &params.remote,
        )?;
        snapshots.push((repo.clone(), op_id));
    }

    // Run the plan, then reposition `@` onto the freshly-synced
    // bookmark (skipped in deprecated verify-only mode). Both live
    // inside the same stop-on-error region: any failure falls
    // through to the report below with state left in place.
    let result = run_plan(&snapshots, params).and_then(|()| {
        if !params.check {
            for (repo, _) in &snapshots {
                reposition_at(repo, repo_bookmark(repo, &params.bookmark), params)?;
            }
        }
        Ok(())
    });

    if let Err(e) = &result {
        warn!("sync failed: {e}");
        warn!("stopping — state left as-is for inspection (no auto-revert)");
        warn!("pre-sync op snapshot per repo:");
        for (repo, op_id) in &snapshots {
            warn!("  {}: op {op_id}", repo.display());
        }
        warn!("undo with `vc-x1 revert`, or per repo: jj op restore <op> -R <repo>");
        debug!("sync: exit");
        return result;
    }

    // Full success — the snapshots are no longer revert targets.
    for (repo, _) in &snapshots {
        state::clear(repo)?;
    }

    debug!("sync: exit");
    Ok(())
}

/// Clean-case summary tail ("<N> repo(s) …" prefixed at the emit
/// site). Shared with main.rs's `long_about` so the documented
/// output shape can't drift from the emitted one.
pub const UP_TO_DATE_MSG: &str = "up to date, nothing to sync";

/// Fetch and classify each repo, then act (or not, in verify-only).
///
/// Returns `Err` on the first failure so the caller can stop and
/// report; partial progress across repos is left in place for
/// inspection (see `sync_repos`).
fn run_plan(
    snapshots: &[(PathBuf, String)],
    params: &SyncParams,
) -> Result<(), Box<dyn std::error::Error>> {
    // Phase 1 — fetch + classify silently.
    //
    // Each repo's fetch stderr is captured (rather than streamed to
    // `info!` via `common::run`) so we can decide after classification
    // whether to surface it. `jj git fetch`'s routine "Nothing
    // changed." chatter is the main thing we're suppressing here —
    // if nothing needs action, the user shouldn't see it.
    let mut fetched: Vec<(PathBuf, String)> = Vec::new();
    let mut ctxs: Vec<RepoCtx> = Vec::new();
    for (repo, op_id) in snapshots {
        let stderr = fetch_silent(repo, &params.remote)?;
        fetched.push((repo.clone(), stderr));
        let state = classify(repo, repo_bookmark(repo, &params.bookmark), &params.remote)?;
        ctxs.push(RepoCtx {
            path: repo.clone(),
            op_id: op_id.clone(),
            state,
        });
    }

    let any_action_needed = ctxs
        .iter()
        .any(|c| matches!(c.state, State::Behind { .. } | State::Diverged { .. }));

    // Phase 2 — emit status. `--quiet` is enforced globally via the
    // log-level clamp in `sync()`, so these `info!` calls are already
    // suppressed in scripts; we just shape the output here.
    if !any_action_needed {
        let n = ctxs.len();
        let noun = if n == 1 { "repo is" } else { "repos are" };
        info!("sync: {n} {noun} {UP_TO_DATE_MSG}");
    } else {
        for (repo, stderr) in &fetched {
            info!("{}: fetch {}", repo.display(), params.remote);
            for line in stderr.lines() {
                info!("{line}");
            }
        }
        for ctx in &ctxs {
            log_state(&ctx.path, &ctx.state);
        }
    }

    // Phase 3 — act (subprocess output streams through as usual).
    // `act_on_state` short-circuits in deprecated verify-only mode,
    // making it a true no-op here. Repositioning `@` onto the synced
    // bookmark happens after `run_plan` returns (see `sync_repos`),
    // outside the revert region.
    for ctx in &ctxs {
        act_on_state(ctx, params)?;
    }

    // Phase 4 — verify-only mode is fatal when action would be
    // needed. The normal sync ran the action above and is done.
    if params.check && any_action_needed {
        let n_action = ctxs
            .iter()
            .filter(|c| matches!(c.state, State::Behind { .. } | State::Diverged { .. }))
            .count();
        let noun = if n_action == 1 { "repo" } else { "repos" };
        return Err(format!(
            "sync: {n_action} {noun} need action (see above) — \
             resolve with `vc-x1 sync` and re-run"
        )
        .into());
    }
    Ok(())
}

/// Fetch `repo` from `remote` without streaming subprocess output to
/// `info!`.
///
/// Mirrors what `common::run` would do, but returns stderr to the
/// caller so the caller can decide whether to surface it (verbose /
/// action case) or drop it (clean case). Stdout is dropped — `jj git
/// fetch` doesn't use it. Failure carries stderr in the error message.
fn fetch_silent(repo: &Path, remote: &str) -> Result<String, Box<dyn std::error::Error>> {
    debug!("$ jj git fetch --remote {remote} -R {}", repo.display());
    let output = Command::new("jj")
        .args(["git", "fetch", "--remote", remote, "-R", &repo_str(repo)])
        .output()
        .map_err(|e| format!("failed to run jj git fetch: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stdout.is_empty() {
        debug!("  {stdout}");
    }
    if !output.status.success() {
        return Err(format!("jj git fetch -R {} failed: {stderr}", repo.display()).into());
    }
    Ok(stderr)
}

/// Reposition `@` onto the freshly-synced bookmark after a successful
/// sync.
///
/// Dispatches on repo role — the caller has already excluded the
/// deprecated verify-only mode:
///
/// - **session (bot) sub-repo** → always `jj new main`
///   (see `reposition_session`).
/// - **any other repo** → move `@` onto the synced `bookmark` under
///   the code-repo safety rules (see `reposition_code`).
fn reposition_at(
    repo: &Path,
    bookmark: &str,
    params: &SyncParams,
) -> Result<(), Box<dyn std::error::Error>> {
    if is_session_repo(repo) {
        reposition_session(repo)
    } else {
        reposition_code(repo, bookmark, params.rebase)
    }
}

/// True when `repo` is the session (bot) sub-repo.
///
/// Reads `<repo>/.vc-config.toml`'s `[workspace] path`: the code repo
/// (workspace root) is `"/"`, the session sub-repo is `"/.claude"`. A
/// missing / unreadable config (POR, single-repo workspace) is treated
/// as a code repo.
fn is_session_repo(repo: &Path) -> bool {
    match toml_simple::toml_load(&repo.join(VC_CONFIG_FILE)) {
        Ok(cfg) => {
            toml_simple::toml_get(&cfg, "workspace.path").map(String::as_str) == Some("/.claude")
        }
        Err(_) => false,
    }
}

/// Bookmark to sync for `repo`.
///
/// `--bookmark` is code-repo-only: the session (bot) repo is a
/// linear journal on `main` by design, so it pins `main` regardless
/// of the requested bookmark. Every other repo uses `bookmark`
/// as passed.
fn repo_bookmark<'a>(repo: &Path, bookmark: &'a str) -> &'a str {
    if is_session_repo(repo) {
        "main"
    } else {
        bookmark
    }
}

/// Reposition the session repo's `@` onto `main`.
///
/// The session (`.claude`) repo is a linear journal on `main`, and its
/// `@` normally carries live session writes:
///
/// - `@-` already the `main` tip → no-op: `@` is where it belongs,
///   live writes stay in the working copy. (An unconditional
///   `jj new main` here would churn an empty `@`'s chid/op every
///   sync, or strand a non-empty `@`'s live writes — and any ochid
///   captured against its chid — on a sibling head.)
/// - Errors when `@-` isn't on `main` (not an ancestor-or-equal of the
///   bookmark) — refuse rather than guess.
/// - Otherwise `main` moved: `jj new main` starts a fresh `@` on the
///   bookmark; the prior `@` becomes a sibling head, which is
///   expected for the journal. A conflict is very unlikely given
///   `.claude`'s content; if one ever appears the user resolves it.
fn reposition_session(repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let parent = commit_id(repo, "@-")?;
    let tip = commit_id(repo, "main")?;
    if parent == tip {
        debug!("{}: @ already on 'main'", repo.display());
        return Ok(());
    }
    if !revset_nonempty(repo, &format!("{parent}::main"))? {
        return Err(format!(
            "{}: @- ({parent}) is not on main — refusing to reposition @",
            repo.display()
        )
        .into());
    }
    info!("{}: jj new main", repo.display());
    run(
        "jj",
        &["new", "main", "-R", &repo_str(repo)],
        Path::new("."),
    )?;
    Ok(())
}

/// Reposition the code repo's `@` onto the synced `bookmark`.
///
/// Let `@-` be the parent of `@`:
///
/// - `bookmark == @-` → already positioned; no-op.
/// - `bookmark` a proper descendant of `@-`, `@` empty →
///   `jj new bookmark` (jj auto-abandons the old empty `@`).
/// - `bookmark` a proper descendant of `@-`, `@` non-empty → rebase
///   `@` onto `bookmark`, but only with `rebase` set (else prompt on a
///   TTY; skip + inform when declined or not a TTY).
/// - `bookmark` not a descendant of `@-` (diverged / `@` ahead) →
///   leave `@` and inform why it didn't move.
fn reposition_code(
    repo: &Path,
    bookmark: &str,
    rebase: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let parent = commit_id(repo, "@-")?;
    let tip = commit_id(repo, bookmark)?;
    if tip == parent {
        debug!("{}: @ already on '{bookmark}'", repo.display());
        return Ok(());
    }
    if !revset_nonempty(repo, &format!("{parent}::{tip}"))? {
        info!(
            "{}: @- ({parent}) is not behind '{bookmark}' ({tip}); leaving @ in place",
            repo.display()
        );
        return Ok(());
    }
    // `bookmark` is a proper descendant of `@-` — safe to move a clean `@`.
    if at_is_empty(repo)? {
        info!("{}: jj new {bookmark}", repo.display());
        run(
            "jj",
            &["new", bookmark, "-R", &repo_str(repo)],
            Path::new("."),
        )?;
        return Ok(());
    }
    // `@` carries changes: rebase only on opt-in / confirmation.
    if !rebase && !confirm_rebase(repo)? {
        info!(
            "{}: @ has changes; left in place (pass --rebase to rebase onto '{bookmark}')",
            repo.display()
        );
        return Ok(());
    }
    info!("{}: rebasing @ onto '{bookmark}'", repo.display());
    run(
        "jj",
        &["rebase", "-b", "@", "-d", bookmark, "-R", &repo_str(repo)],
        Path::new("."),
    )?;
    if has_conflicts(repo)? {
        return Err(format!(
            "{}: rebase of @ onto '{bookmark}' produced conflicts",
            repo.display()
        )
        .into());
    }
    Ok(())
}

/// Ask whether to rebase a non-empty `@`, but only on a TTY.
///
/// Returns `Ok(false)` without prompting when stdin isn't a terminal
/// (scripts): the caller then skips + informs rather than blocking on
/// `read_line`. A `y`/`yes` (case-insensitive) answer confirms.
///
/// Under `cargo test` the harness inherits the invoking terminal's
/// stdin, so an in-process test reaching this path would block on
/// `read_line` waiting for the user — the `cfg!(test)` arm pins the
/// non-interactive answer instead.
fn confirm_rebase(repo: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    if cfg!(test) || !std::io::stdin().is_terminal() {
        return Ok(false);
    }
    let ans = prompt(&format!(
        "{}: @ has changes — rebase onto the synced bookmark? [y/N] ",
        repo.display()
    ))?;
    Ok(matches!(ans.to_ascii_lowercase().as_str(), "y" | "yes"))
}

/// True when the working-copy commit `@` is empty (no changes).
fn at_is_empty(repo: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    revset_nonempty(repo, "@ & empty()")
}

/// Perform the mutation corresponding to `ctx.state` (skipped in
/// deprecated verify-only mode).
///
/// - `UpToDate` / `Ahead` / `NoRemote` → no-op (and no output, the state
///   was already logged by `log_state`).
/// - `Behind` → `jj bookmark set <b> -r <b>@<remote>` to fast-forward.
/// - `Diverged` → `jj rebase -b <b> -d <b>@<remote>`, then probe
///   `conflicts()`. A non-empty result means the rebase produced
///   conflicted commits; return `Err` so the outer revert restores the
///   pre-fetch state.
fn act_on_state(ctx: &RepoCtx, params: &SyncParams) -> Result<(), Box<dyn std::error::Error>> {
    let repo = &ctx.path;
    let bookmark = repo_bookmark(repo, &params.bookmark);
    let remote_rev = format!("{}@{}", bookmark, params.remote);
    match &ctx.state {
        State::UpToDate | State::Ahead { .. } | State::NoRemote => Ok(()),
        State::Behind { .. } => {
            if !params.check {
                info!("{}: setting '{bookmark}' to {remote_rev}", repo.display());
                run(
                    "jj",
                    &[
                        "bookmark",
                        "set",
                        bookmark,
                        "-r",
                        &remote_rev,
                        "-R",
                        &repo_str(repo),
                    ],
                    Path::new("."),
                )?;
            }
            Ok(())
        }
        State::Diverged { local, remote } => {
            if !params.check {
                // `local` is either a single commit id or a comma-joined list
                // of heads when the bookmark is conflicted. Pick the head
                // that isn't the remote — that's the local-only tip. The
                // comma-joined path covers the jj post-fetch divergence
                // shape (local bookmark conflicted between old local head
                // and freshly-fetched remote head).
                let local_head = local
                    .split(',')
                    .find(|h| *h != remote)
                    .unwrap_or(local.as_str());
                info!(
                    "{}: rebasing {local_head} onto {remote_rev}",
                    repo.display()
                );
                run(
                    "jj",
                    &[
                        "rebase",
                        "-b",
                        local_head,
                        "-d",
                        &remote_rev,
                        "-R",
                        &repo_str(repo),
                    ],
                    Path::new("."),
                )?;
                if has_conflicts(repo)? {
                    return Err(format!("{}: rebase produced conflicts", repo.display()).into());
                }
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
        State::NoRemote => info!("{r}: no remote counterpart — skipping"),
        State::Ahead { local, remote } => {
            info!("{r}: ahead (local {local} > remote {remote}); nothing to sync")
        }
        State::Behind { local, remote } => {
            info!("{r}: behind (local {local} < remote {remote}); fast-forward needed")
        }
        State::Diverged { local, remote } => {
            info!("{r}: diverged (local {local} vs remote {remote}); rebase needed")
        }
    }
}

/// Return the id of the most recent operation on `repo`.
///
/// Used as the revert target on failure. Exposed at `pub(crate)` so
/// `push` (0.37.0-2+) can reuse the same snapshot pattern for its
/// commit-stage rollback without duplicating the jj invocation.
pub(crate) fn current_op_id(repo: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let out = run(
        "jj",
        &[
            "op",
            "log",
            "--no-graph",
            "-n",
            "1",
            "-T",
            "id.short(12)",
            "-R",
            &repo_str(repo),
        ],
        Path::new("."),
    )?;
    Ok(out.trim().to_string())
}

/// Restore `repo` to the operation identified by `op_id`.
///
/// Thin wrapper around `jj op restore` — drops the caller's returned
/// stdout. Exposed at `pub(crate)` for `push`'s commit-stage
/// rollback and the `revert` subcommand; sync itself no longer
/// auto-reverts (stop-on-error).
pub(crate) fn op_restore(repo: &Path, op_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    run(
        "jj",
        &["op", "restore", op_id, "-R", &repo_str(repo)],
        Path::new("."),
    )?;
    Ok(())
}

/// Classify the relationship between `bookmark` and `bookmark@remote`.
///
/// Uses `bookmarks(<b>)` rather than the bare name so a conflicted
/// bookmark (jj's representation of a diverged fetch) resolves to all
/// of its heads instead of erroring. When the set has multiple heads,
/// the bookmark is conflicted and the repo is `Diverged` by definition.
/// Otherwise we compare the single local head against the single
/// remote commit via two revset-ancestry probes.
///
/// Returns `NoRemote` when `<b>@<remote>` does not resolve — the caller
/// logs a skip and moves on.
fn classify(
    repo: &Path,
    bookmark: &str,
    remote: &str,
) -> Result<State, Box<dyn std::error::Error>> {
    let local_heads = local_bookmark_heads(repo, bookmark)?;
    let remote_rev = format!("{bookmark}@{remote}");
    let Some(remote) = try_commit_id(repo, &remote_rev)? else {
        return Ok(State::NoRemote);
    };
    if local_heads.is_empty() {
        return Err(format!("{}: bookmark '{bookmark}' does not exist", repo.display()).into());
    }
    if local_heads.len() > 1 {
        // Conflicted bookmark — jj's shape for post-fetch divergence.
        let local = local_heads.join(",");
        return Ok(State::Diverged { local, remote });
    }
    let local = local_heads.into_iter().next().unwrap(); // OK: `len() > 1` arm handled above, `is_empty()` handled above
    if local == remote {
        return Ok(State::UpToDate);
    }
    let local_is_anc = revset_nonempty(repo, &format!("{local}::{remote_rev}"))?;
    let remote_is_anc = revset_nonempty(repo, &format!("{remote_rev}::{local}"))?;
    Ok(match (local_is_anc, remote_is_anc) {
        (true, _) => State::Behind { local, remote },
        (false, true) => State::Ahead { local, remote },
        (false, false) => State::Diverged { local, remote },
    })
}

/// Return all commit ids the `bookmark` currently points at.
///
/// Normally a one-element vector; two or more elements indicate a
/// conflicted bookmark (jj's representation of diverged post-fetch
/// state, where the local bookmark has multiple heads).
fn local_bookmark_heads(
    repo: &Path,
    bookmark: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let out = run(
        "jj",
        &[
            "log",
            "-r",
            &format!("bookmarks(exact:{bookmark})"),
            "--no-graph",
            "-T",
            r#"commit_id.short(12) ++ "\n""#,
            "-R",
            &repo_str(repo),
        ],
        Path::new("."),
    )?;
    Ok(out
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

/// Return the short commit id that `rev` resolves to in `repo`.
///
/// Errors when the revset is unresolvable or matches multiple heads
/// — callers that want to tolerate "revset doesn't resolve" should use
/// `try_commit_id`.
fn commit_id(repo: &Path, rev: &str) -> Result<String, Box<dyn std::error::Error>> {
    let out = run(
        "jj",
        &[
            "log",
            "-r",
            rev,
            "--no-graph",
            "-T",
            "commit_id.short(12)",
            "-R",
            &repo_str(repo),
        ],
        Path::new("."),
    )?;
    Ok(out.trim().to_string())
}

/// Like `commit_id`, but `Ok(None)` when the revset doesn't resolve.
///
/// jj reports missing revisions via stderr strings like
/// `Revision \`foo@origin\` doesn't exist` or `No such revision …`;
/// both get mapped to `Ok(None)` so callers can distinguish "missing"
/// from "other subprocess failure".
fn try_commit_id(repo: &Path, rev: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    match commit_id(repo, rev) {
        Ok(id) if id.is_empty() => Ok(None),
        Ok(id) => Ok(Some(id)),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("doesn't exist") || msg.contains("No such revision") {
                Ok(None)
            } else {
                Err(e)
            }
        }
    }
}

/// Return `true` when `revset` matches at least one commit in `repo`.
fn revset_nonempty(repo: &Path, revset: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let out = run(
        "jj",
        &[
            "log",
            "-r",
            revset,
            "--no-graph",
            "-T",
            r#"commit_id.short() ++ "\n""#,
            "-R",
            &repo_str(repo),
        ],
        Path::new("."),
    )?;
    Ok(!out.trim().is_empty())
}

/// Return `true` when `repo` has any conflicted commits.
fn has_conflicts(repo: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    revset_nonempty(repo, "conflicts()")
}

/// Convert a path to a `String` suitable for passing as a subprocess arg.
fn repo_str(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

pub(crate) mod state;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

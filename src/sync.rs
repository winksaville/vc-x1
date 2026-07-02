//! The `sync` subcommand: fetch + classify + (optionally) rebase /
//! fast-forward a workspace's repos against their remotes.
//!
//! Default is `--check` (fetch + report, error if any repo needs
//! action); `--no-check` applies. On any failure every repo is
//! reverted to its pre-sync op via `jj op restore`.
//!
//! `current_op_id` / `op_restore` are `pub(crate)` so `push` reuses
//! the same snapshot-and-restore pattern.

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
/// Default is `--check`: fetch + classify + report, **error** if any
/// repo is `behind` or `diverged`. Pass `--no-check` to actually
/// rebase / fast-forward. On any failure the starting state of every
/// repo is restored via `jj op restore`.
///
/// Scripts and automation should pass `--check` or `--no-check`
/// explicitly rather than relying on the default — defaults can shift,
/// explicit flags lock in the contract.
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
    /// Verify only — fetch + classify; error if any repo needs action.
    /// This is the default; pass explicitly in scripts.
    #[arg(long, conflicts_with = "no_check")]
    pub check: bool,

    /// Apply — fetch + classify, then rebase / fast-forward as needed.
    #[arg(long, conflicts_with = "check")]
    pub no_check: bool,

    /// Suppress all informational output (exit code signals result)
    #[arg(short, long)]
    pub quiet: bool,

    /// Bookmark to sync in each repo
    #[arg(long, default_value = "main")]
    pub bookmark: String,

    /// Remote to sync against
    #[arg(long, default_value = "origin")]
    pub remote: String,

    /// Rebase a non-empty `@` onto the synced bookmark without asking.
    ///
    /// After a successful `--no-check` sync, `@` is repositioned onto
    /// the synced bookmark. When the code repo's `@` carries changes
    /// it normally asks before rebasing; `--rebase` answers yes up
    /// front for non-interactive use. Ignored in `--check` mode and
    /// for the session repo (which always `jj new main`).
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
/// - `bookmark`: bookmark to sync in each repo (default `main`).
/// - `remote`: remote to sync against (default `origin`).
/// - `no_check`: `--no-check` — actually rebase / fast-forward
///   (absent ⇒ check mode: fetch + report only). The `--check`
///   flag is the explicit form of the default and carries no
///   value of its own, so it isn't mirrored here.
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
    pub no_check: bool,
    pub rebase: bool,
    pub repo: Option<PathBuf>,
    pub scope: Option<Scope>,
}

impl From<&SyncArgs> for SyncParams {
    /// Convert clap-derived `SyncArgs` into the flat `SyncParams`.
    /// `--check` is dropped — it's the explicit form of the
    /// default; the op only ever consults `no_check`.
    fn from(a: &SyncArgs) -> Self {
        Self {
            quiet: a.quiet,
            bookmark: a.bookmark.clone(),
            remote: a.remote.clone(),
            no_check: a.no_check,
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

/// Resolve `params` into the concrete repo list `sync_repos`
/// operates on.
///
/// `-R` and `--scope` compose:
///
/// - neither — workspace-default scope against the discovered root.
/// - `-R PATH` alone — sync just that repo.
/// - `-s ROLES` alone — roles against the discovered workspace root.
/// - `-R PATH -s ROLES` — roles against `PATH` as the workspace root.
fn resolve_params_to_repos(
    params: &SyncParams,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    match (&params.repo, &params.scope) {
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
    let repos = resolve_params_to_repos(params)?;
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
/// snapshot each repo's current op id, then hand off to `run_plan` for
/// fetch + classify + (optional) act. On any error, revert every repo
/// to its snapshot op via `jj op restore` so the caller sees an atomic
/// "either it all went through or nothing changed" outcome.
///
/// Paths may be relative (resolved against the process cwd) or
/// absolute. Tests use absolute tempdir paths to avoid cwd dependence
/// under parallel `cargo test`.
pub fn sync_repos(
    repos: &[PathBuf],
    params: &SyncParams,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!(
        "sync: enter (no_check={}, bookmark={}, remote={})",
        params.no_check, params.bookmark, params.remote
    );

    // Preflight: verify bookmark tracking on every repo before any
    // fetch/rebase. Implements "Non-tracking-remote bookmark detection"
    // — design at:
    //   https://github.com/winksaville/vc-x1/blob/main/notes/chores/chores-06.md#non-tracking-remote-bookmark-detection-design
    for repo in repos {
        crate::common::verify_tracking(repo, &params.bookmark)?;
    }

    let mut snapshots: Vec<(PathBuf, String)> = Vec::new();
    for repo in repos {
        let op_id = current_op_id(repo)?;
        debug!("{}: op snapshot = {op_id}", repo.display());
        snapshots.push((repo.clone(), op_id));
    }

    let result = run_plan(&snapshots, params);

    if let Err(e) = &result {
        warn!("sync failed: {e}");
        warn!("reverting all repos to starting state...");
        for (repo, op_id) in &snapshots {
            match op_restore(repo, op_id) {
                Ok(()) => info!("  {}: reverted to op {op_id}", repo.display()),
                Err(re) => warn!("  {}: revert failed: {re}", repo.display()),
            }
        }
        debug!("sync: exit");
        return result;
    }

    // Reposition `@` onto the freshly-synced bookmark. Runs only in
    // apply mode and only after every repo synced cleanly, and sits
    // OUTSIDE the `op_restore` revert region above: a reposition
    // failure (e.g. the session repo's `@-` off main) is surfaced
    // without rolling back the successful fetch / fast-forward.
    if params.no_check {
        for (repo, _) in &snapshots {
            reposition_at(repo, &params.bookmark, params)?;
        }
    }

    debug!("sync: exit");
    Ok(())
}

/// Fetch and classify each repo, then act (or not, in dry-run).
///
/// Returns `Err` on the first failure so the caller can trigger the
/// cross-repo revert; partial success across repos is *not* a valid
/// end state for this command.
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
        let state = classify(repo, &params.bookmark, &params.remote)?;
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
        let noun = if n == 1 { "repo" } else { "repos" };
        info!("sync: {n} {noun}, all bookmarks up-to-date");
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
    // `act_on_state` short-circuits when `params.no_check` is false,
    // so check mode is a true no-op here. Repositioning `@` onto the
    // synced bookmark happens after `run_plan` returns (see
    // `sync_repos`), outside the revert region.
    for ctx in &ctxs {
        act_on_state(ctx, params)?;
    }

    // Phase 4 — check mode is fatal when action would be needed.
    // Apply mode (`--no-check`) ran the action above and is done.
    if !params.no_check && any_action_needed {
        let n_action = ctxs
            .iter()
            .filter(|c| matches!(c.state, State::Behind { .. } | State::Diverged { .. }))
            .count();
        let noun = if n_action == 1 { "repo" } else { "repos" };
        return Err(format!(
            "sync: {n_action} {noun} need action (see above) — \
             resolve with `vc-x1 sync --no-check` and re-run"
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
/// apply-mode sync.
///
/// Dispatches on repo role — the caller has already gated on apply
/// mode:
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

/// Reposition the session repo's `@` onto `main`.
///
/// The session (`.claude`) repo is a linear journal on `main`, and its
/// `@` normally carries live session writes:
///
/// - Errors when `@-` isn't on `main` (not an ancestor-or-equal of the
///   bookmark) — refuse rather than guess.
/// - Otherwise `jj new main` starts a fresh `@` on the bookmark; the
///   prior `@` becomes a sibling head, which is expected for the
///   journal. A conflict is very unlikely given `.claude`'s content;
///   if one ever appears the user resolves it.
fn reposition_session(repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let parent = commit_id(repo, "@-")?;
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
        info!("{}: @ already on '{bookmark}'", repo.display());
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

/// Perform the mutation corresponding to `ctx.state` when `--no-check`.
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
    let remote_rev = format!("{}@{}", params.bookmark, params.remote);
    match &ctx.state {
        State::UpToDate | State::Ahead { .. } | State::NoRemote => Ok(()),
        State::Behind { .. } => {
            if params.no_check {
                info!("{}: fast-forwarding '{}'", repo.display(), params.bookmark);
                run(
                    "jj",
                    &[
                        "bookmark",
                        "set",
                        &params.bookmark,
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
            if params.no_check {
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
            info!("{r}: behind (local {local} < remote {remote}); would fast-forward")
        }
        State::Diverged { local, remote } => {
            info!("{r}: diverged (local {local} vs remote {remote}); would rebase")
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
/// Thin wrapper around `jj op restore`. Called during the failure
/// revert path — drops the caller's returned stdout. Exposed at
/// `pub(crate)` so `push` reuses the same restore call.
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

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

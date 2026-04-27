use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Args;
use log::{LevelFilter, debug, info, warn};

use crate::common::{default_scope, find_workspace_root, run, scope_to_repos};
use crate::scope::{Scope, Side};

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
/// Repo set is resolved (in order):
///
/// - `-R` / `--repo` → exactly that list (back-compat / arbitrary multi-repo).
/// - `--scope=code|bot|code,bot` → dual-repo roles, resolved via the
///   project root's `.vc-config.toml`.
/// - Neither → default scope: `code,bot` when `.vc-config.toml` has a
///   non-empty `[workspace] other-repo`, else `code`. POR (no
///   `.vc-config.toml`) → `code` resolved to cwd.
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

    /// Path to jj repo; repeatable or comma-separated.
    ///
    /// Mutually exclusive with `--scope`. When neither is given the
    /// repo set is derived from `--scope` defaults.
    #[arg(
        short = 'R',
        long = "repo",
        value_name = "PATH",
        conflicts_with = "scope"
    )]
    pub repos: Vec<PathBuf>,

    /// Which repo(s) of the dual-repo to sync.
    ///
    /// `SCOPE=code|bot|code,bot`:
    ///
    /// - `code` — sync only the app repo.
    /// - `bot` — sync only the bot repo (errors if no bot repo
    ///   is configured).
    /// - `code,bot` — sync both repos (the default when both are
    ///   configured).
    #[arg(
        long,
        value_name = "SCOPE",
        value_delimiter = ',',
        verbatim_doc_comment
    )]
    pub scope: Option<Vec<Side>>,
}

/// Split the caller's `-R` flags into a concrete repo list.
///
/// Each value is split on `,` and trimmed, so `-R .,.claude` works
/// identically to `-R . -R .claude`. Caller is responsible for not
/// passing an empty slice — `sync()` consults `--scope` instead.
fn split_repos(raw: &[PathBuf]) -> Vec<PathBuf> {
    raw.iter()
        .flat_map(|p| {
            p.to_string_lossy()
                .split(',')
                .map(|s| PathBuf::from(s.trim()))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Resolve `args` into the concrete repo list `sync_repos` operates on.
///
/// Precedence: explicit `-R` → `--scope` → workspace-default scope.
/// See [`SyncArgs`] for the full contract.
fn resolve_args_to_repos(args: &SyncArgs) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    if !args.repos.is_empty() {
        return Ok(split_repos(&args.repos));
    }
    let workspace_root = find_workspace_root();
    let scope = match &args.scope {
        Some(sides) if sides.is_empty() => return Err("--scope: value is empty".into()),
        Some(sides) => Scope::Roles(sides.clone()),
        None => default_scope(workspace_root.as_deref()),
    };
    scope_to_repos(&scope, workspace_root.as_deref())
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
/// Thin wrapper over `sync_repos` that resolves the `-R` flag into a
/// concrete repo list (falling back to the dual-repo default) and
/// forwards the rest of the args. Tests call `sync_repos` directly
/// with absolute fixture paths.
///
/// When `--quiet` is set, the global log filter is temporarily clamped
/// to `Warn` for the duration of the call and restored on return, so
/// `info!` calls throughout sync (plus any subprocess-stderr routed
/// through `common::run`) go dark. Errors still surface at `Warn` /
/// `Error` so script callers don't lose diagnostics.
pub fn sync(args: &SyncArgs) -> Result<(), Box<dyn std::error::Error>> {
    let repos = resolve_args_to_repos(args)?;
    if args.quiet {
        let prev = log::max_level();
        log::set_max_level(LevelFilter::Warn);
        let result = sync_repos(&repos, args);
        log::set_max_level(prev);
        result
    } else {
        sync_repos(&repos, args)
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
pub fn sync_repos(repos: &[PathBuf], args: &SyncArgs) -> Result<(), Box<dyn std::error::Error>> {
    debug!(
        "sync: enter (no_check={}, bookmark={}, remote={})",
        args.no_check, args.bookmark, args.remote
    );

    // Preflight: verify bookmark tracking on every repo before any
    // fetch/rebase. Implements "Non-tracking-remote bookmark detection"
    // — design at:
    //   https://github.com/winksaville/vc-x1/blob/main/notes/chores-06.md#non-tracking-remote-bookmark-detection-design
    for repo in repos {
        crate::common::verify_tracking(repo, &args.bookmark)?;
    }

    let mut snapshots: Vec<(PathBuf, String)> = Vec::new();
    for repo in repos {
        let op_id = current_op_id(repo)?;
        debug!("{}: op snapshot = {op_id}", repo.display());
        snapshots.push((repo.clone(), op_id));
    }

    let result = run_plan(&snapshots, args);

    if let Err(e) = &result {
        warn!("sync failed: {e}");
        warn!("reverting all repos to starting state...");
        for (repo, op_id) in &snapshots {
            match op_restore(repo, op_id) {
                Ok(()) => info!("  {}: reverted to op {op_id}", repo.display()),
                Err(re) => warn!("  {}: revert failed: {re}", repo.display()),
            }
        }
    }

    debug!("sync: exit");
    result
}

/// Fetch and classify each repo, then act (or not, in dry-run).
///
/// Returns `Err` on the first failure so the caller can trigger the
/// cross-repo revert; partial success across repos is *not* a valid
/// end state for this command.
fn run_plan(
    snapshots: &[(PathBuf, String)],
    args: &SyncArgs,
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
        let stderr = fetch_silent(repo, &args.remote)?;
        fetched.push((repo.clone(), stderr));
        let state = classify(repo, &args.bookmark, &args.remote)?;
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
            info!("{}: fetch {}", repo.display(), args.remote);
            for line in stderr.lines() {
                info!("{line}");
            }
        }
        for ctx in &ctxs {
            log_state(&ctx.path, &ctx.state);
        }
    }

    // Phase 3 — act (subprocess output streams through as usual).
    // `act_on_state` and `ensure_at_on_main` short-circuit when
    // `args.no_check` is false, so check mode is a true no-op here.
    for ctx in &ctxs {
        act_on_state(ctx, args)?;
        ensure_at_on_main(&ctx.path, &args.bookmark, args.no_check)?;
    }

    // Phase 4 — check mode is fatal when action would be needed.
    // Apply mode (`--no-check`) ran the action above and is done.
    if !args.no_check && any_action_needed {
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

/// Ensure `@` is a descendant of `bookmark`, rebasing if not.
///
/// `jj git fetch` fast-forwards a tracked local bookmark to the remote
/// tip when local is a strict ancestor, but leaves `@` behind — still
/// parented on the *pre-fetch* bookmark commit. The `.claude` repo is
/// the motivating case: trailing session writes (e.g. `/exit`'s jsonl
/// tail) sit in `@`, and without this step they'd end up orphaned on a
/// stale branch when the remote advanced.
///
/// Skipped when `main::@` is already non-empty (i.e., `@` is already
/// reachable from `bookmark`). On `--no-check`, runs
/// `jj rebase -b @ -d <bookmark>` which carries all commits between
/// the old parent and `@` forward, then checks `conflicts()` — any
/// conflict in this step trips the outer revert the same way a
/// conflicted `act_on_state` rebase would.
fn ensure_at_on_main(
    repo: &Path,
    bookmark: &str,
    apply: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if revset_nonempty(repo, &format!("{bookmark}::@"))? {
        return Ok(());
    }
    info!("{}: rebasing @ onto '{bookmark}'", repo.display());
    if apply {
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
    }
    Ok(())
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
fn act_on_state(ctx: &RepoCtx, args: &SyncArgs) -> Result<(), Box<dyn std::error::Error>> {
    let repo = &ctx.path;
    let remote_rev = format!("{}@{}", args.bookmark, args.remote);
    match &ctx.state {
        State::UpToDate | State::Ahead { .. } | State::NoRemote => Ok(()),
        State::Behind { .. } => {
            if args.no_check {
                info!("{}: fast-forwarding '{}'", repo.display(), args.bookmark);
                run(
                    "jj",
                    &[
                        "bookmark",
                        "set",
                        &args.bookmark,
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
            if args.no_check {
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
mod tests {
    use super::*;

    /// Default flags: neither `--check` nor `--no-check` set (the
    /// implicit default is check mode), bookmark "main", remote "origin",
    /// no `-R` and no `--scope` (caller will resolve via the
    /// workspace-default scope), `--quiet` off.
    #[test]
    fn parse_defaults() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(flatten)]
            args: SyncArgs,
        }
        let cli = Cli::try_parse_from(["test"]).unwrap();
        assert!(!cli.args.check);
        assert!(!cli.args.no_check);
        assert!(!cli.args.quiet);
        assert_eq!(cli.args.bookmark, "main");
        assert_eq!(cli.args.remote, "origin");
        assert!(cli.args.repos.is_empty());
        assert!(cli.args.scope.is_none());
    }

    /// `-q` / `--quiet` CLI form is honored.
    #[test]
    fn parse_quiet_flag() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(flatten)]
            args: SyncArgs,
        }
        let cli = Cli::try_parse_from(["test", "--quiet"]).unwrap();
        assert!(cli.args.quiet);
        let cli_short = Cli::try_parse_from(["test", "-q"]).unwrap();
        assert!(cli_short.args.quiet);
    }

    /// Overrides: `--no-check`, `--bookmark`, `--remote` all honored.
    #[test]
    fn parse_overrides() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(flatten)]
            args: SyncArgs,
        }
        let cli = Cli::try_parse_from([
            "test",
            "--no-check",
            "--bookmark",
            "dev",
            "--remote",
            "upstream",
        ])
        .unwrap();
        assert!(cli.args.no_check);
        assert!(!cli.args.check);
        assert_eq!(cli.args.bookmark, "dev");
        assert_eq!(cli.args.remote, "upstream");
        assert!(cli.args.repos.is_empty());
    }

    /// `--check` flag parses as the explicit form of the default.
    #[test]
    fn parse_check_flag() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(flatten)]
            args: SyncArgs,
        }
        let cli = Cli::try_parse_from(["test", "--check"]).unwrap();
        assert!(cli.args.check);
        assert!(!cli.args.no_check);
    }

    /// `--check` and `--no-check` together are rejected by clap.
    #[test]
    fn parse_check_no_check_conflict() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(flatten)]
            args: SyncArgs,
        }
        assert!(Cli::try_parse_from(["test", "--check", "--no-check"]).is_err());
    }

    /// Single `-R` passthrough (single-repo project case).
    #[test]
    fn split_repos_single() {
        let raw = vec![PathBuf::from("/tmp/some-repo")];
        assert_eq!(split_repos(&raw), vec![PathBuf::from("/tmp/some-repo")]);
    }

    /// Repeated `-R` flags combine into the final list in order.
    #[test]
    fn split_repos_repeated() {
        let raw = vec![
            PathBuf::from("/a"),
            PathBuf::from("/b"),
            PathBuf::from("/c"),
        ];
        assert_eq!(
            split_repos(&raw),
            vec![
                PathBuf::from("/a"),
                PathBuf::from("/b"),
                PathBuf::from("/c"),
            ]
        );
    }

    /// Comma-separated values inside a single `-R` are split and trimmed.
    #[test]
    fn split_repos_comma_separated() {
        let raw = vec![PathBuf::from(" /a , /b ,/c")];
        assert_eq!(
            split_repos(&raw),
            vec![
                PathBuf::from("/a"),
                PathBuf::from("/b"),
                PathBuf::from("/c"),
            ]
        );
    }

    /// Mixed repeated + comma forms compose naturally.
    #[test]
    fn split_repos_mixed() {
        let raw = vec![PathBuf::from("/a,/b"), PathBuf::from("/c")];
        assert_eq!(
            split_repos(&raw),
            vec![
                PathBuf::from("/a"),
                PathBuf::from("/b"),
                PathBuf::from("/c"),
            ]
        );
    }

    /// `--scope=code` parses into a single-element side list.
    #[test]
    fn parse_scope_code() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(flatten)]
            args: SyncArgs,
        }
        let cli = Cli::try_parse_from(["test", "--scope", "code"]).unwrap();
        assert_eq!(cli.args.scope.as_deref(), Some(&[Side::Code][..]));
    }

    /// `--scope=code,bot` parses into a two-element list.
    #[test]
    fn parse_scope_code_bot() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(flatten)]
            args: SyncArgs,
        }
        let cli = Cli::try_parse_from(["test", "--scope", "code,bot"]).unwrap();
        assert_eq!(
            cli.args.scope.as_deref(),
            Some(&[Side::Code, Side::Bot][..])
        );
    }

    /// `--scope` and `-R` are mutually exclusive (clap conflicts_with).
    #[test]
    fn parse_scope_repo_conflict() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(flatten)]
            args: SyncArgs,
        }
        assert!(Cli::try_parse_from(["test", "--scope", "code", "-R", "/tmp/x"]).is_err());
    }

    /// `-R path` CLI form is accepted for a single repo.
    #[test]
    fn parse_single_repo_flag() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(flatten)]
            args: SyncArgs,
        }
        let cli = Cli::try_parse_from(["test", "-R", "/tmp/x"]).unwrap();
        assert_eq!(cli.args.repos, vec![PathBuf::from("/tmp/x")]);
    }

    /// Repeated `-R` CLI form accumulates into the `repos` vec.
    #[test]
    fn parse_repeated_repo_flag() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(flatten)]
            args: SyncArgs,
        }
        let cli = Cli::try_parse_from(["test", "-R", "/a", "-R", "/b"]).unwrap();
        assert_eq!(
            cli.args.repos,
            vec![PathBuf::from("/a"), PathBuf::from("/b")]
        );
    }
}

#[cfg(test)]
mod integration_tests {
    //! End-to-end tests for `sync_repos` against real dual-repo jj
    //! fixtures. Each test builds an isolated fixture (bare-git
    //! remotes + colocated jj repos) under a unique tempdir via
    //! `Fixture::new` (driving `init::init_with_symlink` in
    //! `--repo-local` mode), then drives the scenario with plain
    //! `jj` subprocess calls. Requires `jj` in `PATH`.
    //!
    //! Fixtures clean themselves up via `Fixture`'s `Drop` impl so a
    //! panicking test still removes its tempdir.

    use super::*;
    use crate::test_helpers::Fixture;
    use std::fs;
    use std::process::Command;

    use crate::common::{default_scope, find_workspace_root_from, scope_to_repos};

    /// Resolver helpers must consume what `init --repo-local` produces.
    ///
    /// Builds a real dual-repo fixture (bare git remotes + colocated jj
    /// repos + the canonical `.vc-config.toml` pair init writes), then
    /// drives the resolver chain `sync()` uses when `-R` is empty:
    ///
    /// - `find_workspace_root_from(&fx.claude)` walks up to `fx.work`
    ///   (proves cwd-portability against init's `path = "/.claude"` /
    ///   `path = "/"` config split).
    /// - `default_scope(Some(&fx.work))` reads the workspace config
    ///   and resolves to the dual-repo default.
    /// - `scope_to_repos` maps each `Scope` shape to the right
    ///   absolute path(s) under the fixture.
    ///
    /// Pure check on the resolver chain — does not invoke `sync()`
    /// itself, since that walks `std::env::current_dir()` and parallel
    /// `cargo test` makes cwd mutation unsafe.
    #[test]
    fn resolver_chain_against_init_repo_local() {
        let fx = Fixture::new("resolver-chain");

        // Walk-up from the bot side lands on the app root.
        assert_eq!(
            find_workspace_root_from(&fx.claude).as_deref(),
            Some(&*fx.work),
            "find_workspace_root should resolve from .claude up to work"
        );
        // Walk-up from the app root finds itself.
        assert_eq!(
            find_workspace_root_from(&fx.work).as_deref(),
            Some(&*fx.work)
        );

        // init --repo-local writes other-repo = ".claude", so the
        // workspace's default scope is dual.
        assert_eq!(
            default_scope(Some(&fx.work)),
            Scope::Roles(vec![Side::Code, Side::Bot])
        );

        // Each scope shape resolves to the right absolute path(s).
        assert_eq!(
            scope_to_repos(&Scope::Roles(vec![Side::Code, Side::Bot]), Some(&fx.work)).unwrap(),
            vec![fx.work.clone(), fx.claude.clone()]
        );
        assert_eq!(
            scope_to_repos(&Scope::Roles(vec![Side::Code]), Some(&fx.work)).unwrap(),
            vec![fx.work.clone()]
        );
        assert_eq!(
            scope_to_repos(&Scope::Roles(vec![Side::Bot]), Some(&fx.work)).unwrap(),
            vec![fx.claude.clone()]
        );

        // sync_repos accepts the resolved list and reports up-to-date
        // — the resolver's output is shaped the way sync expects.
        let resolved =
            scope_to_repos(&Scope::Roles(vec![Side::Code, Side::Bot]), Some(&fx.work)).unwrap();
        sync_repos(&resolved, &apply_args()).expect("sync should succeed on resolved repos");
    }

    /// Run `jj <args> -R <repo>` and assert success; returns trimmed stdout.
    fn jj(repo: &Path, args: &[&str]) -> String {
        let out = Command::new("jj")
            .args(args)
            .arg("-R")
            .arg(repo)
            .output()
            .expect("spawn jj");
        assert!(
            out.status.success(),
            "jj {args:?} failed in {}: {}",
            repo.display(),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// Resolve `rev` to its short commit id in `repo`.
    fn cid(repo: &Path, rev: &str) -> String {
        jj(
            repo,
            &["log", "-r", rev, "--no-graph", "-T", "commit_id.short(12)"],
        )
    }

    /// Sync args with `--no-check` set (apply mode).
    ///
    /// Integration tests pass explicit repo paths through `sync_repos`
    /// directly, so `repos` stays empty here and the CLI-side default
    /// resolution is not exercised by this helper.
    fn apply_args() -> SyncArgs {
        SyncArgs {
            check: false,
            no_check: true,
            quiet: false,
            bookmark: "main".to_string(),
            remote: "origin".to_string(),
            repos: Vec::new(),
            scope: None,
        }
    }

    /// Add a local-only commit on `main` in `repo` (not pushed), then
    /// restore `@` to an empty child so pre-flight still passes.
    ///
    /// Sequence: write file → describe `@` → advance `main` to `@` →
    /// create a fresh empty `@` above it.
    fn add_local_commit(repo: &Path, file: &str, content: &str, msg: &str) {
        fs::write(repo.join(file), content).expect("write local file");
        jj(repo, &["describe", "@", "-m", msg]);
        jj(repo, &["bookmark", "set", "main", "-r", "@"]);
        jj(repo, &["new"]);
    }

    /// Clone `remote_url` into `<base>/<work_name>`, add a commit, push it.
    ///
    /// Used to make the remote advance beyond the fixture's `main`
    /// from a separate working copy. Returns the pushed commit's id.
    fn push_from_clone(
        base: &Path,
        remote_url: &Path,
        work_name: &str,
        file: &str,
        content: &str,
        msg: &str,
    ) -> String {
        let workdir = base.join(work_name);
        let out = Command::new("jj")
            .args(["git", "clone", "--colocate"])
            .arg(remote_url)
            .arg(&workdir)
            .output()
            .expect("spawn jj clone");
        assert!(
            out.status.success(),
            "jj git clone failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        fs::write(workdir.join(file), content).expect("write remote file");
        jj(&workdir, &["describe", "@", "-m", msg]);
        jj(&workdir, &["bookmark", "set", "main", "-r", "@"]);
        jj(&workdir, &["git", "push", "--bookmark", "main"]);
        cid(&workdir, "main")
    }

    /// Scenario 1: fresh fixture, nothing to do — `sync` leaves both
    /// repos untouched.
    #[test]
    fn sync_up_to_date() {
        let fx = Fixture::new("up-to-date");
        let work_main = cid(&fx.work, "main");
        let claude_main = cid(&fx.claude, "main");
        sync_repos(&fx.repos(), &apply_args()).expect("sync should succeed");
        assert_eq!(cid(&fx.work, "main"), work_main);
        assert_eq!(cid(&fx.claude, "main"), claude_main);
    }

    /// Scenario 2a: a non-empty `@` on top of main (simulates `/exit`
    /// trailing session writes in `.claude`) is tolerated when there's
    /// nothing new on the remote. `@`'s commit id changes because jj
    /// snapshots the written file, but its content is preserved and
    /// `@` stays reachable from main.
    #[test]
    fn sync_tolerates_trailing_at_up_to_date() {
        let fx = Fixture::new("trailing-uptodate");
        let pre_main = cid(&fx.claude, "main");
        fs::write(fx.claude.join("trailing.jsonl"), "{\"line\":1}\n").expect("write trailing file");
        sync_repos(&fx.repos(), &apply_args()).expect("sync should succeed");
        assert_eq!(cid(&fx.claude, "main"), pre_main, "main should not move");
        let on_main = jj(
            &fx.claude,
            &[
                "log",
                "-r",
                "main::@",
                "--no-graph",
                "-T",
                r#"commit_id.short(12) ++ "\n""#,
            ],
        );
        assert!(!on_main.trim().is_empty(), "@ should stay on main's line");
        assert_eq!(
            fs::read_to_string(fx.claude.join("trailing.jsonl")).unwrap(),
            "{\"line\":1}\n",
            "trailing content preserved"
        );
    }

    /// Scenario 2b: `@` has trailing writes and the remote advanced
    /// while the session was offline. jj's fetch auto-ff's main but
    /// leaves `@` behind; sync must then rebase `@` onto the new main
    /// so the session tail doesn't end up orphaned. Content is
    /// preserved across the rebase.
    #[test]
    fn sync_rebases_trailing_at_when_main_moves() {
        let fx = Fixture::new("trailing-rebase");
        let remote_claude = fx.base.join("remote-claude.git");
        let remote_head = push_from_clone(
            &fx.base,
            &remote_claude,
            "claude2",
            "remote.md",
            "remote\n",
            "feat: remote-added",
        );
        // Trailing writes on @
        fs::write(fx.claude.join("trailing.jsonl"), "{\"line\":2}\n").expect("write trailing file");
        sync_repos(&fx.repos(), &apply_args()).expect("sync should succeed");
        assert_eq!(
            cid(&fx.claude, "main"),
            remote_head,
            "local main should match remote after auto-ff"
        );
        // @ should now be a descendant of main.
        let on_main = jj(
            &fx.claude,
            &[
                "log",
                "-r",
                "main::@",
                "--no-graph",
                "-T",
                r#"commit_id.short(12) ++ "\n""#,
            ],
        );
        assert!(
            !on_main.trim().is_empty(),
            "@ should be reachable from main after ensure_at_on_main"
        );
        // Trailing content preserved on disk.
        assert_eq!(
            fs::read_to_string(fx.claude.join("trailing.jsonl")).unwrap(),
            "{\"line\":2}\n",
            "trailing content preserved across @ rebase"
        );
    }

    /// Scenario 2c: `@` has trailing writes and local+remote both
    /// modify the same file differently. Rebase produces conflicts;
    /// sync fails and op-restores both repos. The trailing content
    /// must survive the revert — jj documents that op restore rolls
    /// back operation history without clobbering the working copy.
    #[test]
    fn sync_conflict_preserves_trailing_at_on_revert() {
        let fx = Fixture::new("trailing-conflict");
        let remote_claude = fx.base.join("remote-claude.git");
        push_from_clone(
            &fx.base,
            &remote_claude,
            "claude2",
            "shared.txt",
            "remote-version\n",
            "feat: remote shared",
        );
        // Local commit on main with conflicting content
        add_local_commit(
            &fx.claude,
            "shared.txt",
            "local-version\n",
            "feat: local shared (conflicting)",
        );
        // Trailing writes on new @
        fs::write(fx.claude.join("trailing.jsonl"), "{\"line\":3}\n").expect("write trailing file");

        let pre_main = cid(&fx.claude, "main");
        let pre_remote = cid(&fx.claude, "main@origin");

        let err = sync_repos(&fx.repos(), &apply_args())
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("conflicts"),
            "expected conflict error, got: {err}"
        );

        // Post-revert: main and main@origin back to pre-sync.
        assert_eq!(cid(&fx.claude, "main"), pre_main, "main reverted");
        assert_eq!(
            cid(&fx.claude, "main@origin"),
            pre_remote,
            "main@origin reverted"
        );
        // No conflicts remain.
        let conflicts = jj(
            &fx.claude,
            &[
                "log",
                "-r",
                "conflicts()",
                "--no-graph",
                "-T",
                r#"commit_id ++ "\n""#,
            ],
        );
        assert!(
            conflicts.trim().is_empty(),
            "no conflicts should remain after revert"
        );
        // Trailing content preserved on disk.
        assert_eq!(
            fs::read_to_string(fx.claude.join("trailing.jsonl")).unwrap(),
            "{\"line\":3}\n",
            "trailing content preserved across op restore"
        );
    }

    /// Scenario 3: local has commits not yet pushed; sync classifies
    /// `ahead` and leaves the local bookmark alone even under
    /// `--no-check`.
    #[test]
    fn sync_ahead_is_noop() {
        let fx = Fixture::new("ahead");
        add_local_commit(&fx.work, "local.txt", "local\n", "feat: local only");
        let ahead_head = cid(&fx.work, "main");
        sync_repos(&fx.repos(), &apply_args()).expect("sync should succeed");
        assert_eq!(cid(&fx.work, "main"), ahead_head);
    }

    /// Scenario 4: clean divergence — both sides advance main on
    /// different files; sync rebases local onto remote and the result
    /// is conflict-free.
    #[test]
    fn sync_diverged_rebases() {
        let fx = Fixture::new("diverged");
        let remote_code = fx.base.join("remote-code.git");
        let remote_head = push_from_clone(
            &fx.base,
            &remote_code,
            "work2",
            "remote.txt",
            "remote\n",
            "feat: remote only",
        );
        add_local_commit(&fx.work, "local.txt", "local\n", "feat: local only");

        sync_repos(&fx.repos(), &apply_args()).expect("sync should succeed");

        // Remote tracking bookmark now points at the pushed remote commit.
        assert_eq!(
            cid(&fx.work, "main@origin"),
            remote_head,
            "main@origin should match pushed remote commit"
        );
        // Local main is ahead of remote — rebased local commit sits on top.
        let main_after = cid(&fx.work, "main");
        assert_ne!(
            main_after, remote_head,
            "local main should be ahead of remote after rebase"
        );
        // Remote is an ancestor of local post-rebase.
        let anc = jj(
            &fx.work,
            &[
                "log",
                "-r",
                &format!("{remote_head}::{main_after}"),
                "--no-graph",
                "-T",
                r#"commit_id.short(12) ++ "\n""#,
            ],
        );
        assert!(
            !anc.trim().is_empty(),
            "remote should be ancestor of local after rebase"
        );
        // No conflicts.
        let conflicts = jj(
            &fx.work,
            &[
                "log",
                "-r",
                "conflicts()",
                "--no-graph",
                "-T",
                r#"commit_id ++ "\n""#,
            ],
        );
        assert!(conflicts.trim().is_empty(), "no conflicts expected");
    }

    /// Scenario 5: conflicting divergence — both sides modify the
    /// same path differently. Rebase produces conflicts; sync fails
    /// and reverts both repos to their pre-sync state.
    #[test]
    fn sync_diverged_conflict_reverts() {
        let fx = Fixture::new("conflict");
        let remote_code = fx.base.join("remote-code.git");
        push_from_clone(
            &fx.base,
            &remote_code,
            "work2",
            "shared.txt",
            "remote-version\n",
            "feat: remote shared",
        );
        add_local_commit(
            &fx.work,
            "shared.txt",
            "local-version\n",
            "feat: local shared (conflicting)",
        );

        let pre_main = cid(&fx.work, "main");
        let pre_remote = cid(&fx.work, "main@origin");

        let err = sync_repos(&fx.repos(), &apply_args())
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("rebase produced conflicts"),
            "unexpected error: {err}"
        );

        // After revert: main and main@origin are back where they started.
        assert_eq!(cid(&fx.work, "main"), pre_main, "main should be reverted");
        assert_eq!(
            cid(&fx.work, "main@origin"),
            pre_remote,
            "main@origin should be reverted (pre-fetch state)"
        );
        // No conflicts remain.
        let conflicts = jj(
            &fx.work,
            &[
                "log",
                "-r",
                "conflicts()",
                "--no-graph",
                "-T",
                r#"commit_id ++ "\n""#,
            ],
        );
        assert!(
            conflicts.trim().is_empty(),
            "no conflicts should remain after revert"
        );
    }
}

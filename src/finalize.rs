//! The `finalize` subcommand: squash + set-bookmark + push a jj
//! repo, optionally detached so trailing writes land first.
//!
//! Built for the bot to atomically finalize its `.claude` session
//! repo at end-of-step. Every action is opt-in (`--squash`,
//! `--push`, `--detach`, `--delay`). The detached path re-execs
//! `vc-x1 finalize --exec …` so its output and errors survive the
//! parent exiting; failures are dropped as markers under
//! `~/.cache/vc-x1/finalize-status` and surfaced on the next
//! `vc-x1` run.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Args;
use log::{debug, info};

use crate::common::run;
use crate::context::Context;
use crate::options_flags::squash::{SquashOption, SquashSpec};
use crate::subcommand::SubcommandRunner;

/// Directory where the detached finalize child writes failure markers.
/// A subsequent `vc-x1` invocation scans this directory and surfaces
/// any failures to the user, closing the gap that the detached child's
/// non-zero exit isn't observable by the caller.
fn status_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache/vc-x1/finalize-status"))
}

/// Drop a failure marker for the detached child whose non-zero exit
/// the caller can't observe.
fn write_failure_marker(params: &FinalizeParams, err: &str) {
    let Some(dir) = status_dir() else {
        return;
    };
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0); // OK: filesystem-sortable timestamp; 0 on the impossible pre-epoch path
    let pid = std::process::id();
    let path = dir.join(format!("{ns}-{pid}.status"));
    let content = format!(
        "timestamp_ns={ns}\npid={pid}\nrepo={}\nbookmark={}\nerror={err}\n",
        params.repo.display(),
        params.bookmark.as_deref().unwrap_or(""),
    );
    let _ = std::fs::write(&path, content);
}

/// Read, print, and delete any failure markers left by previous detached
/// finalize children. Cheap no-op when the directory doesn't exist.
pub fn surface_previous_failures() {
    let Some(dir) = status_dir() else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("status"))
        .collect();
    paths.sort();
    for path in paths {
        if let Ok(content) = std::fs::read_to_string(&path) {
            eprintln!(
                "warn: previous finalize failure ({}):",
                path.file_stem()
                    .map(|s| s.to_string_lossy())
                    .unwrap_or_default() // OK: filename without extension; empty on unexpected absence
            );
            for line in content.lines() {
                eprintln!("  {line}");
            }
            let _ = std::fs::remove_file(&path);
        }
    }
}

/// Squash, set bookmark, and/or push a jj repo.
///
/// Designed for the bot to atomically finalize its session repo:
/// --detach exits immediately, --delay waits for trailing writes,
/// --squash folds them in, --bookmark + --push sends it upstream.
/// Every flag is opt-in. See README.md for details.
#[derive(Args, Debug)]
pub struct FinalizeArgs {
    /// Path to jj repo
    #[arg(long, default_value = ".")]
    pub repo: PathBuf,

    #[command(flatten)]
    pub squash: SquashOption,

    /// Seconds to wait before squashing
    #[arg(long, default_value_t = 10.0)]
    pub delay: f64,

    /// Existing bookmark to push to remote
    #[arg(long, value_name = "BOOKMARK")]
    pub push: Option<String>,

    /// Detach and run in the background
    #[arg(long)]
    pub detach: bool,

    /// The above `pub detach` is true when `--detach` is passed
    /// to the `finalize` subcommand. The app then `fork`s and
    /// spawns `vc-x1 finalize --exec …`, replacing `--detach`
    /// with `--exec`. The child sees `exec: true` and is routed
    /// through `finalize_exec()` to do the squash/push work in
    /// the background.
    ///
    /// Marked `hide = true` because it's an internal argument and
    /// should not appear in `-h` / `--help` output.
    #[arg(long, hide = true)]
    pub exec: bool,
}

/// Per-invocation finalize inputs — the clap-free shape the
/// `finalize` op works against. Built from `FinalizeArgs` at the
/// binary edge via `TryFrom` (fallible only on repo-path
/// canonicalization — `--squash` is already parsed into a
/// `SquashSpec` by clap). The `--log` path lives on `Context`,
/// not here.
#[derive(Debug)]
pub struct FinalizeParams {
    pub repo: PathBuf,
    pub squash: Option<SquashSpec>,
    pub delay_secs: f64,
    pub bookmark: Option<String>,
    pub push: bool,
    pub detach: bool,
    pub exec: bool,
}

impl TryFrom<&FinalizeArgs> for FinalizeParams {
    type Error = String;

    /// Derive `push`/`bookmark` from `--push` and canonicalize
    /// `--repo` (so the detached child resolves it regardless of
    /// cwd); `--squash` was already parsed by clap.
    fn try_from(a: &FinalizeArgs) -> Result<Self, String> {
        let repo = std::fs::canonicalize(&a.repo)
            .map_err(|e| format!("cannot resolve repo path '{}': {e}", a.repo.display()))?;
        Ok(FinalizeParams {
            repo,
            squash: a.squash.value.clone(),
            delay_secs: a.delay,
            bookmark: a.push.clone(),
            push: a.push.is_some(),
            detach: a.detach,
            exec: a.exec,
        })
    }
}

impl SubcommandRunner for FinalizeArgs {
    type Params = FinalizeParams;

    /// Delegate to the existing `TryFrom<&FinalizeArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        FinalizeParams::try_from(self)
    }

    /// Run the existing `finalize` op.
    fn run(ctx: &Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        finalize(ctx, params)
    }

    /// The detached `finalize --exec` re-entry is the bot's
    /// session-end child; report it as `is_detached_exec=true` so
    /// the trait's default `dispatch` suppresses the banner via
    /// `crate::sb_ide` (the child shouldn't print user-facing
    /// chatter or surface failure markers in its log).
    fn is_detached_exec(params: &Self::Params) -> bool {
        params.exec
    }
}

/// Validate inputs synchronously before detaching.
///
/// Catches the common failure modes up front so the parent can exit
/// with a visible non-zero status — rather than discovering them in
/// the detached child where errors only reach the log file.
fn preflight(params: &FinalizeParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("preflight: checking params");
    let repo_str = params.repo.to_string_lossy();
    let cwd = std::path::Path::new(".");

    // Verify squash revsets resolve to something.
    if let Some(ref sq) = params.squash {
        jj_rev_exists(&repo_str, cwd, &sq.source)
            .map_err(|e| format!("squash source '{}' does not resolve: {e}", sq.source))?;
        jj_rev_exists(&repo_str, cwd, &sq.target)
            .map_err(|e| format!("squash target '{}' does not resolve: {e}", sq.target))?;
    }

    // Refuse to operate on a repo with conflicts.
    let conflicts = run(
        "jj",
        &[
            "log",
            "-r",
            "conflicts()",
            "--no-graph",
            "-T",
            "\"x\"",
            "-R",
            &repo_str,
        ],
        cwd,
    )?;
    if !conflicts.is_empty() {
        return Err(format!("repo '{repo_str}' has conflicts — resolve before finalize").into());
    }

    // Bookmark: existence, tracking, forward-only move, push-target description.
    if let Some(ref bookmark) = params.bookmark {
        let exists = run("jj", &["bookmark", "list", bookmark, "-R", &repo_str], cwd)?;
        if exists.is_empty() {
            return Err(format!("bookmark '{bookmark}' does not exist").into());
        }

        crate::common::verify_tracking(&params.repo, bookmark)?;

        let target_rev = params
            .squash
            .as_ref()
            .map(|sq| sq.target.as_str())
            .unwrap_or("@"); // OK: no squash spec → bookmark will point at current @
        let range = run(
            "jj",
            &[
                "log",
                "-r",
                &format!("{bookmark}::({target_rev})"),
                "--no-graph",
                "-T",
                "\"x\"",
                "-R",
                &repo_str,
            ],
            cwd,
        )?;
        if range.is_empty() {
            return Err(format!(
                "bookmark '{bookmark}' move is not forward — current position is not an \
                 ancestor of '{target_rev}' (would diverge)"
            )
            .into());
        }

        if params.push {
            let desc = run(
                "jj",
                &[
                    "log",
                    "-r",
                    target_rev,
                    "--no-graph",
                    "-T",
                    "description",
                    "-R",
                    &repo_str,
                ],
                cwd,
            )?;
            if desc.trim().is_empty() {
                return Err(format!(
                    "push target '{target_rev}' has no description — push would fail \
                     (run `jj describe -r {target_rev} -R {repo_str}` first)"
                )
                .into());
            }
        }
    }

    log_plan(params)?;

    Ok(())
}

/// Return Ok(()) if the revset resolves to one or more commits in `repo`.
fn jj_rev_exists(repo: &str, cwd: &std::path::Path, rev: &str) -> Result<(), String> {
    run(
        "jj",
        &["log", "-r", rev, "--no-graph", "-T", "\"x\"", "-R", repo],
        cwd,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Log what finalize is about to do so the user sees the plan before detach.
fn log_plan(params: &FinalizeParams) -> Result<(), Box<dyn std::error::Error>> {
    let repo_str = params.repo.to_string_lossy();
    let cwd = std::path::Path::new(".");

    if let Some(ref sq) = params.squash {
        info!(
            "finalize: squash {} → {} in {}",
            sq.source, sq.target, repo_str
        );
    }
    if let Some(ref bookmark) = params.bookmark {
        let target_rev = params
            .squash
            .as_ref()
            .map(|sq| sq.target.as_str())
            .unwrap_or("@"); // OK: same as above
        let current = jj_rev_short(&repo_str, cwd, bookmark).unwrap_or_else(|_| "?".into()); // OK: logging only — fall back to "?" if revset fails
        let target = jj_rev_short(&repo_str, cwd, target_rev).unwrap_or_else(|_| "?".into()); // OK: same
        info!("finalize: set bookmark '{bookmark}' {current} → {target} ({target_rev})");
        if params.push {
            info!("finalize: push '{bookmark}' to remote");
        }
    }

    Ok(())
}

/// Short one-line summary of a revset: `<change_short> <commit_short>`.
fn jj_rev_short(repo: &str, cwd: &std::path::Path, rev: &str) -> Result<String, String> {
    run(
        "jj",
        &[
            "log",
            "-r",
            rev,
            "--no-graph",
            "-T",
            "change_id.shortest(8) ++ \" \" ++ commit_id.shortest(8)",
            "-R",
            repo,
        ],
        cwd,
    )
    .map_err(|e| e.to_string())
}

/// Do the squash / set-bookmark / push work — runs in the detached
/// child when `--detach`, inline otherwise.
fn finalize_exec(params: &FinalizeParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("finalize_exec: starting params={params:?}");
    let repo_str = params.repo.to_string_lossy();
    let cwd = std::path::Path::new(".");

    // Squash if requested
    if let Some(ref sq) = params.squash {
        let delay = std::time::Duration::from_secs_f64(params.delay_secs);
        debug!("finalize_exec: sleeping {delay:?}");
        std::thread::sleep(delay);

        info!("finalize: squashing {} → {}...", sq.source, sq.target);
        run(
            "jj",
            &[
                "squash",
                "--ignore-immutable",
                "--use-destination-message",
                "--from",
                &sq.source,
                "--into",
                &sq.target,
                "-R",
                &repo_str,
            ],
            cwd,
        )?;
    }

    // Set bookmark and push if requested
    if let Some(ref bookmark) = params.bookmark {
        // Verify bookmark exists — don't silently create new ones
        let result = run("jj", &["bookmark", "list", bookmark, "-R", &repo_str], cwd)?;
        if result.is_empty() {
            return Err(format!("bookmark '{bookmark}' does not exist").into());
        }

        let rev = params
            .squash
            .as_ref()
            .map(|sq| sq.target.as_str())
            .unwrap_or("@"); // OK: no squash spec → bookmark points at current @
        info!("finalize: setting bookmark '{bookmark}' to {rev}...");
        run(
            "jj",
            &["bookmark", "set", bookmark, "-r", rev, "-R", &repo_str],
            cwd,
        )?;

        if params.push {
            info!("finalize: pushing '{bookmark}' to origin...");
            run(
                "jj",
                &["git", "push", "--bookmark", bookmark, "-R", &repo_str],
                &params.repo,
            )?;
        }
    }

    info!("finalize: done");
    debug!("finalize_exec: done");
    Ok(())
}

/// Build the argv for the detached re-exec: `finalize --exec …`,
/// mirroring `params` plus the `--log` path so the child logs to
/// the same file.
pub fn build_exec_args(params: &FinalizeParams, log: Option<&Path>) -> Vec<String> {
    let mut args = vec![
        "finalize".to_string(),
        "--exec".to_string(),
        "--repo".to_string(),
        params.repo.to_string_lossy().to_string(),
    ];
    if let Some(ref sq) = params.squash {
        args.push("--squash".to_string());
        args.push(format!("{},{}", sq.source, sq.target));
        args.push("--delay".to_string());
        args.push(params.delay_secs.to_string());
    }
    if let Some(ref bookmark) = params.bookmark {
        args.push("--push".to_string());
        args.push(bookmark.clone());
    }
    if let Some(log) = log {
        args.push("--log".to_string());
        args.push(log.to_string_lossy().to_string());
    }
    args
}

/// Spawn a detached child process with `--exec` and return immediately.
/// The child re-enters `main()`, parses `--exec`, and `finalize()` routes
/// it to `finalize_exec()` where the actual work happens.
fn detach(ctx: &Context, params: &FinalizeParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("detach: parent starting params={params:?}");

    let exe = std::env::current_exe()?;
    let args = build_exec_args(params, ctx.log.as_deref());
    debug!("detach: exe={exe:?} args={args:?}");

    let mut cmd = std::process::Command::new(exe);
    cmd.args(&args).stdin(Stdio::null());

    // Hand the child the user's controlling terminal when one exists so
    // its output stays visible after the parent exits. Inherited pipes
    // are unreliable here — the bash-tool / cron caller typically closes
    // its read end on parent exit, silently dropping child writes. Fall
    // back to null when there is no tty (pipe-invoked / CI / Windows);
    // the log file is authoritative in that case.
    match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
    {
        Ok(tty) => {
            debug!("detach: /dev/tty opened, child output → terminal");
            let out = tty.try_clone()?;
            cmd.stdout(Stdio::from(out));
            cmd.stderr(Stdio::from(tty));
        }
        Err(e) => {
            debug!("detach: /dev/tty unavailable ({e}), child output → null");
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
    }

    let child = cmd.spawn()?;
    debug!("detach: spawned child pid={}", child.id());
    match &ctx.log {
        Some(log) => info!(
            "finalize: detached (pid {}), log: {}",
            child.id(),
            log.display()
        ),
        None => info!("finalize: detached (pid {})", child.id()),
    }
    Ok(())
}

/// Run the `finalize` subcommand: preflight, then either detach a
/// child to do the work or do it inline.
pub fn finalize(ctx: &Context, params: &FinalizeParams) -> Result<(), Box<dyn std::error::Error>> {
    // Nothing to do — show help hint
    if params.squash.is_none() && !params.push {
        return Err("nothing to do (see --help for options)".into());
    }

    debug!("finalize: entry params={params:?}");

    // Validate synchronously before detaching so failures exit with a
    // visible non-zero status rather than hiding in the detached child's log.
    // Skip in --exec (the re-entered child already passed preflight in the parent).
    if !params.exec {
        preflight(params)?;
    }

    let result = if params.detach && !params.exec {
        detach(ctx, params)
    } else {
        finalize_exec(params)
    };
    match &result {
        Ok(()) => debug!("finalize: exit ok"),
        Err(e) => {
            debug!("finalize: exit err={e}");
            // In the detached-child path the caller can't observe our
            // non-zero exit. Drop a failure marker so a later vc-x1
            // invocation can surface the problem.
            if params.exec {
                write_failure_marker(params, &e.to_string());
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::{Cli, Commands};
    use clap::Parser;

    fn parse(args: &[&str]) -> FinalizeArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::Finalize(a)) => a,
            _ => panic!("expected Finalize"),
        }
    }

    fn parse_err(args: &[&str]) -> String {
        Cli::try_parse_from(args).unwrap_err().to_string()
    }

    #[test]
    fn no_args() {
        let args = parse(&["vc-x1", "finalize"]);
        assert_eq!(args.repo, PathBuf::from("."));
        assert!(args.squash.value.is_none());
        assert!(args.push.is_none());
        assert!(!args.detach);
    }

    #[test]
    fn push_only() {
        let args = parse(&["vc-x1", "finalize", "--push", "main"]);
        assert!(args.squash.value.is_none());
        assert_eq!(args.push, Some("main".to_string()));
    }

    fn squash_at() -> SquashSpec {
        SquashSpec {
            source: "@".to_string(),
            target: "@-".to_string(),
        }
    }

    #[test]
    fn all_opts() {
        let cli = Cli::try_parse_from([
            "vc-x1",
            "--log",
            "/tmp/test.log",
            "finalize",
            "--repo",
            ".claude",
            "--squash",
            "@,@-",
            "--push",
            "dev-0.14.0",
            "--delay",
            "2.5",
            "--detach",
        ])
        .unwrap();
        assert_eq!(cli.log, Some(PathBuf::from("/tmp/test.log")));
        if let Some(Commands::Finalize(args)) = cli.command {
            assert_eq!(args.repo, PathBuf::from(".claude"));
            assert_eq!(args.squash.value, Some(squash_at()));
            assert_eq!(args.push, Some("dev-0.14.0".to_string()));
            assert_eq!(args.delay, 2.5);
            assert!(args.detach);
        } else {
            panic!("expected Finalize");
        }
    }

    #[test]
    fn bare_squash() {
        let args = parse(&["vc-x1", "finalize", "--squash", "--push", "main"]);
        assert_eq!(args.squash.value, Some(squash_at()));
        assert_eq!(args.push, Some("main".to_string()));
    }

    #[test]
    fn bad_delay() {
        let err = parse_err(&["vc-x1", "finalize", "--delay", "abc"]);
        assert!(err.contains("invalid value"));
    }

    #[test]
    fn unknown_opt() {
        let err = parse_err(&["vc-x1", "finalize", "--bogus"]);
        assert!(err.contains("--bogus"));
    }

    #[test]
    fn try_from_canonicalizes_repo() {
        let args = parse(&["vc-x1", "finalize", "--repo", ".", "--push", "main"]);
        let params = FinalizeParams::try_from(&args).unwrap();
        assert_eq!(params.repo, std::fs::canonicalize(".").unwrap());
        assert_eq!(params.bookmark, Some("main".to_string()));
        assert!(params.push);
        assert!(params.squash.is_none());
    }

    #[test]
    fn bad_squash() {
        let err = parse_err(&["vc-x1", "finalize", "--squash", "@", "--push", "main"]);
        assert!(err.contains("expected SOURCE,TARGET"), "got: {err}");
    }

    #[test]
    fn build_exec_args_roundtrip() {
        let params = FinalizeParams {
            repo: PathBuf::from(".claude"),
            squash: Some(SquashSpec {
                source: "@".to_string(),
                target: "@-".to_string(),
            }),
            delay_secs: 2.0,
            bookmark: Some("dev-0.14.0".to_string()),
            push: true,
            detach: false,
            exec: false,
        };
        let exec_args = build_exec_args(&params, Some(Path::new("/tmp/test.log")));
        let mut full_args = vec!["vc-x1".to_string()];
        full_args.extend(exec_args);
        let cli = Cli::try_parse_from(full_args).unwrap();
        assert_eq!(cli.log, Some(PathBuf::from("/tmp/test.log")));
        if let Some(crate::Commands::Finalize(args)) = cli.command {
            let parsed = FinalizeParams::try_from(&args).unwrap();
            // repo is canonicalized, so compare the canonical form
            assert_eq!(parsed.repo, std::fs::canonicalize(&params.repo).unwrap());
            assert_eq!(parsed.squash.as_ref().unwrap().source, "@");
            assert_eq!(parsed.squash.as_ref().unwrap().target, "@-");
            assert_eq!(parsed.bookmark, Some("dev-0.14.0".to_string()));
            assert_eq!(parsed.delay_secs, params.delay_secs);
            assert!(parsed.push);
            assert!(parsed.exec);
        } else {
            panic!("expected Finalize");
        }
    }
}

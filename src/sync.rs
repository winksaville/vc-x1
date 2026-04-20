use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info, warn};

use crate::common::run;

/// Fetch and sync both repos (`.` and `.claude`) to their remotes.
///
/// Default is dry-run: reports per-repo state without mutating. Pass
/// `--no-dry-run` to act. On any failure the starting state of both
/// repos is restored via `jj op restore`.
#[derive(Args, Debug)]
pub struct SyncArgs {
    /// Actually perform the sync (default: dry-run)
    #[arg(long)]
    pub no_dry_run: bool,

    /// Bookmark to sync in each repo
    #[arg(long, default_value = "main")]
    pub bookmark: String,

    /// Remote to sync against
    #[arg(long, default_value = "origin")]
    pub remote: String,
}

/// Repos synced, in order. Hardcoded for now — the user's convention.
const REPOS: [&str; 2] = [".", ".claude"];

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
/// Thin wrapper over `sync_repos` that supplies the project's
/// hardcoded repo pair `["." ".claude"]`. Tests call `sync_repos`
/// directly with absolute fixture paths.
pub fn sync(args: &SyncArgs) -> Result<(), Box<dyn std::error::Error>> {
    let repos: Vec<PathBuf> = REPOS.iter().map(PathBuf::from).collect();
    sync_repos(&repos, args)
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
        "sync: enter (no_dry_run={}, bookmark={}, remote={})",
        args.no_dry_run, args.bookmark, args.remote
    );

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
    let mut ctxs: Vec<RepoCtx> = Vec::new();
    for (repo, op_id) in snapshots {
        info!("{}: fetch {}", repo.display(), args.remote);
        run(
            "jj",
            &[
                "git",
                "fetch",
                "--remote",
                &args.remote,
                "-R",
                &repo_str(repo),
            ],
            Path::new("."),
        )?;
        let state = classify(repo, &args.bookmark, &args.remote)?;
        log_state(repo, &state);
        ctxs.push(RepoCtx {
            path: repo.clone(),
            op_id: op_id.clone(),
            state,
        });
    }

    for ctx in &ctxs {
        act_on_state(ctx, args)?;
        ensure_at_on_main(&ctx.path, &args.bookmark, args.no_dry_run)?;
    }

    if !args.no_dry_run {
        info!("dry-run — re-run with --no-dry-run to apply");
    }
    Ok(())
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
/// reachable from `bookmark`). On `--no-dry-run`, runs
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

/// Perform the mutation corresponding to `ctx.state` when `--no-dry-run`.
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
            if args.no_dry_run {
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
            if args.no_dry_run {
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
/// Used as the revert target on failure.
fn current_op_id(repo: &Path) -> Result<String, Box<dyn std::error::Error>> {
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
/// revert path — drops the caller's returned stdout.
fn op_restore(repo: &Path, op_id: &str) -> Result<(), Box<dyn std::error::Error>> {
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

    /// Default flags: dry-run on, bookmark "main", remote "origin".
    #[test]
    fn parse_defaults() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(flatten)]
            args: SyncArgs,
        }
        let cli = Cli::try_parse_from(["test"]).unwrap();
        assert!(!cli.args.no_dry_run);
        assert_eq!(cli.args.bookmark, "main");
        assert_eq!(cli.args.remote, "origin");
    }

    /// Overrides: `--no-dry-run`, `--bookmark`, `--remote` all honored.
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
            "--no-dry-run",
            "--bookmark",
            "dev",
            "--remote",
            "upstream",
        ])
        .unwrap();
        assert!(cli.args.no_dry_run);
        assert_eq!(cli.args.bookmark, "dev");
        assert_eq!(cli.args.remote, "upstream");
    }
}

#[cfg(test)]
mod integration_tests {
    //! End-to-end tests for `sync_repos` against real dual-repo jj
    //! fixtures. Each test builds an isolated fixture (bare-git
    //! remotes + colocated jj repos) under a unique tempdir via
    //! `crate::test_fixture::test_fixture`, then drives the scenario
    //! with plain `jj` subprocess calls. Requires `jj` in `PATH`.
    //!
    //! Fixtures clean themselves up via `Fixture`'s `Drop` impl so a
    //! panicking test still removes its tempdir.

    use super::*;
    use crate::test_fixture::{TestFixtureArgs, test_fixture};
    use std::fs;
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Build a unique tempdir path for a test fixture.
    ///
    /// Combines a nanosecond timestamp with a per-process atomic
    /// counter so parallel tests and same-nanosecond collisions both
    /// yield distinct paths.
    fn unique_base(tag: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("vc-x1-sync-{tag}-{ts}-{n}"))
    }

    /// Owned dual-repo fixture with RAII cleanup.
    struct Fixture {
        base: PathBuf,
        work: PathBuf,
        claude: PathBuf,
    }

    impl Fixture {
        /// Build a fresh fixture in a unique tempdir.
        fn new(tag: &str) -> Self {
            let base = unique_base(tag);
            let args = TestFixtureArgs {
                path: Some(base.clone()),
                with_pending: false,
                use_template: None,
            };
            test_fixture(&args).expect("build test fixture");
            let work = base.join("work");
            let claude = work.join(".claude");
            Fixture { base, work, claude }
        }

        /// Repos to pass to `sync_repos`.
        fn repos(&self) -> Vec<PathBuf> {
            vec![self.work.clone(), self.claude.clone()]
        }
    }

    impl Drop for Fixture {
        /// Remove the fixture tree on drop. Best-effort; a failure here
        /// doesn't fail the test.
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.base);
        }
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

    /// Sync args with `--no-dry-run` set.
    fn apply_args() -> SyncArgs {
        SyncArgs {
            no_dry_run: true,
            bookmark: "main".to_string(),
            remote: "origin".to_string(),
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
    /// `--no-dry-run`.
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

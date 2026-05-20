//! Integration tests for the sync module.
//!
//! End-to-end tests for `sync_repos` against real dual-repo jj
//! fixtures. Each test builds an isolated fixture (bare-git
//! remotes + colocated jj repos) under a unique tempdir via
//! `Fixture::new`, then drives the scenario with plain `jj`
//! subprocess calls. Requires `jj` in `PATH`.
//!
//! Fixtures clean themselves up via `Fixture`'s `Drop` impl so a
//! panicking test still removes its tempdir.

use super::*;
use crate::options_flags::scope::Side;
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
/// - `default_scope(Some(&fx.work))` reads the workspace config and
///   resolves to the dual-repo default.
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
        Scope(vec![Side::Code, Side::Bot])
    );

    // Each scope shape resolves to the right absolute path(s).
    assert_eq!(
        scope_to_repos(&Scope(vec![Side::Code, Side::Bot]), Some(&fx.work)).unwrap(),
        vec![fx.work.clone(), fx.claude.clone()]
    );
    assert_eq!(
        scope_to_repos(&Scope(vec![Side::Code]), Some(&fx.work)).unwrap(),
        vec![fx.work.clone()]
    );
    assert_eq!(
        scope_to_repos(&Scope(vec![Side::Bot]), Some(&fx.work)).unwrap(),
        vec![fx.claude.clone()]
    );

    // sync_repos accepts the resolved list and reports up-to-date
    // — the resolver's output is shaped the way sync expects.
    let resolved = scope_to_repos(&Scope(vec![Side::Code, Side::Bot]), Some(&fx.work)).unwrap();
    sync_repos(&resolved, &apply_params()).expect("sync should succeed on resolved repos");
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

/// Sync params with `--no-check` set (apply mode).
///
/// Integration tests pass explicit repo paths through `sync_repos`
/// directly, so `repo` / `scope` stay `None` here and the CLI-side
/// default resolution is not exercised by this helper.
fn apply_params() -> SyncParams {
    SyncParams {
        no_check: true,
        quiet: false,
        bookmark: "main".to_string(),
        remote: "origin".to_string(),
        repo: None,
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
    sync_repos(&fx.repos(), &apply_params()).expect("sync should succeed");
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
    sync_repos(&fx.repos(), &apply_params()).expect("sync should succeed");
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
    sync_repos(&fx.repos(), &apply_params()).expect("sync should succeed");
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

    let err = sync_repos(&fx.repos(), &apply_params())
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
    sync_repos(&fx.repos(), &apply_params()).expect("sync should succeed");
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

    sync_repos(&fx.repos(), &apply_params()).expect("sync should succeed");

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

    let err = sync_repos(&fx.repos(), &apply_params())
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

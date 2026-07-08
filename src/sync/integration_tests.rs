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
    sync_repos(&resolved, &default_params()).expect("sync should succeed on resolved repos");
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

/// Default sync params (the normal atomic sync — no flags).
///
/// Integration tests pass explicit repo paths through `sync_repos`
/// directly, so `repo` / `scope` stay `None` here and the CLI-side
/// default resolution is not exercised by this helper.
fn default_params() -> SyncParams {
    SyncParams {
        check: false,
        quiet: false,
        bookmark: "main".to_string(),
        remote: "origin".to_string(),
        rebase: false,
        repo: None,
        scope: None,
    }
}

/// Default params with `--rebase` set (auto-confirm the code-repo
/// non-empty `@` rebase).
fn rebase_params() -> SyncParams {
    SyncParams {
        rebase: true,
        ..default_params()
    }
}

/// True when `revset` matches at least one commit in `repo`.
fn has(repo: &Path, revset: &str) -> bool {
    !jj(
        repo,
        &[
            "log",
            "-r",
            revset,
            "--no-graph",
            "-T",
            r#"commit_id.short(12) ++ "\n""#,
        ],
    )
    .trim()
    .is_empty()
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

/// Clone `remote_url` into `<base>/<work_name>` (colocated) and
/// return the new workdir.
fn clone(base: &Path, remote_url: &Path, work_name: &str) -> PathBuf {
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
    workdir
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
    let workdir = clone(base, remote_url, work_name);
    fs::write(workdir.join(file), content).expect("write remote file");
    jj(&workdir, &["describe", "@", "-m", msg]);
    jj(&workdir, &["bookmark", "set", "main", "-r", "@"]);
    jj(&workdir, &["git", "push", "--bookmark", "main"]);
    cid(&workdir, "main")
}

/// Scenario 1: fresh fixture, nothing to do — `sync` leaves both
/// repos untouched and clears the persisted snapshots on success
/// (a stale file must not become a later revert target).
#[test]
fn sync_up_to_date() {
    let fx = Fixture::new("up-to-date");
    let work_main = cid(&fx.work, "main");
    let claude_main = cid(&fx.claude, "main");
    sync_repos(&fx.repos(), &default_params()).expect("sync should succeed");
    assert_eq!(cid(&fx.work, "main"), work_main);
    assert_eq!(cid(&fx.claude, "main"), claude_main);
    assert!(
        state::load(&fx.work).expect("load work state").is_none(),
        "state cleared on success (work)"
    );
    assert!(
        state::load(&fx.claude)
            .expect("load claude state")
            .is_none(),
        "state cleared on success (claude)"
    );
}

/// Scenario 2a: a non-empty `@` on top of main (simulates `/exit`
/// trailing session writes in `.claude`) when there's nothing new on
/// the remote. The session repo always `jj new main`s: `@` becomes a
/// fresh empty child of the unmoved main, and the trailing commit is
/// preserved as a non-empty sibling head (no longer in the working
/// copy).
#[test]
fn sync_session_jj_new_when_up_to_date() {
    let fx = Fixture::new("session-jjnew-uptodate");
    let pre_main = cid(&fx.claude, "main");
    fs::write(fx.claude.join("trailing.jsonl"), "{\"line\":1}\n").expect("write trailing file");
    sync_repos(&fx.repos(), &default_params()).expect("sync should succeed");
    // main didn't move.
    assert_eq!(cid(&fx.claude, "main"), pre_main, "main should not move");
    // @ is a fresh empty child of main.
    assert!(has(&fx.claude, "@ & empty()"), "@ should be empty");
    assert!(has(&fx.claude, "main::@"), "@ should be a child of main");
    // The trailing session commit survives as a non-empty sibling head.
    assert!(
        has(&fx.claude, "heads(all()) & ~empty()"),
        "former @ preserved as a non-empty sibling head"
    );
    // The trailing file is no longer in the working copy (@ moved off it).
    assert!(
        !fx.claude.join("trailing.jsonl").exists(),
        "@ no longer holds the trailing file"
    );
}

/// Scenario 2b: `@` has trailing writes and the remote advanced while
/// the session was offline. jj's fetch auto-ff's main; the session
/// repo then `jj new main`s onto the new tip, leaving the trailing
/// commit as a sibling head off the old tip.
#[test]
fn sync_session_jj_new_when_main_moves() {
    let fx = Fixture::new("session-jjnew-moved");
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
    sync_repos(&fx.repos(), &default_params()).expect("sync should succeed");
    assert_eq!(
        cid(&fx.claude, "main"),
        remote_head,
        "local main should match remote after auto-ff"
    );
    // @ is a fresh empty child of the new main.
    assert!(has(&fx.claude, "@ & empty()"), "@ should be empty");
    assert!(has(&fx.claude, "main::@"), "@ should be a child of main");
    // The trailing session commit survives as a non-empty sibling head.
    assert!(
        has(&fx.claude, "heads(all()) & ~empty()"),
        "former @ preserved as a non-empty sibling head"
    );
    // The trailing file is no longer in the working copy.
    assert!(
        !fx.claude.join("trailing.jsonl").exists(),
        "@ no longer holds the trailing file"
    );
}

/// Scenario 2c: the session repo refuses to reposition when `@-` is
/// not on main. A local session commit ahead of main (main left
/// behind) puts `@-` off main's line, so `jj new main` would strand
/// it — sync errors instead.
#[test]
fn sync_session_errors_when_at_parent_off_main() {
    let fx = Fixture::new("session-off-main");
    // A described commit ahead of main, with a fresh @ above it, so
    // @- is ahead of (not on) main.
    fs::write(fx.claude.join("ahead.jsonl"), "{\"line\":9}\n").expect("write ahead file");
    jj(&fx.claude, &["describe", "@", "-m", "feat: session ahead"]);
    jj(&fx.claude, &["new"]);

    let err = sync_repos(&fx.repos(), &default_params())
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("not on main"),
        "expected off-main refusal, got: {err}"
    );
}

/// Scenario 2c: `@` has trailing writes and local+remote both
/// modify the same file differently. Rebase produces conflicts;
/// sync stops with the conflicted state left in place for
/// inspection (no auto-revert) and the persisted snapshot still
/// present as the manual revert target. Trailing content stays on
/// disk — the rebase carries `@` along, it never rewrites the
/// working-copy file.
#[test]
fn sync_conflict_stops_and_keeps_state() {
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

    let pre_op = current_op_id(&fx.claude).expect("pre-sync op id");

    let err = sync_repos(&fx.repos(), &default_params())
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("conflicts"),
        "expected conflict error, got: {err}"
    );

    // Stop-on-error: the conflicted state is still there to inspect.
    assert!(
        has(&fx.claude, "conflicts()"),
        "conflicted commits left in place for inspection"
    );
    // The persisted snapshot names the pre-sync op (revert target).
    let st = state::load(&fx.claude)
        .expect("load sync state")
        .expect("state file present after failure");
    assert_eq!(st.op_id, pre_op, "state records the pre-sync op id");
    // Trailing content preserved on disk.
    assert_eq!(
        fs::read_to_string(fx.claude.join("trailing.jsonl")).unwrap(),
        "{\"line\":3}\n",
        "trailing content preserved across the stop"
    );
}

/// Scenario 3: local has commits not yet pushed; sync classifies
/// `ahead` and leaves the local bookmark alone.
#[test]
fn sync_ahead_is_noop() {
    let fx = Fixture::new("ahead");
    add_local_commit(&fx.work, "local.txt", "local\n", "feat: local only");
    let ahead_head = cid(&fx.work, "main");
    sync_repos(&fx.repos(), &default_params()).expect("sync should succeed");
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

    sync_repos(&fx.repos(), &default_params()).expect("sync should succeed");

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
/// same path differently. Rebase produces conflicts; sync stops
/// with the conflicted state in place (no auto-revert) and the
/// persisted snapshots as manual revert targets.
#[test]
fn sync_diverged_conflict_stops_and_keeps_state() {
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

    let pre_op_work = current_op_id(&fx.work).expect("work op id");
    let pre_op_claude = current_op_id(&fx.claude).expect("claude op id");

    let err = sync_repos(&fx.repos(), &default_params())
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("rebase produced conflicts"),
        "unexpected error: {err}"
    );

    // Stop-on-error: the conflicted rebase result is left in place.
    assert!(
        has(&fx.work, "conflicts()"),
        "conflicted commits left in place for inspection"
    );
    // Every repo's persisted snapshot survives as the revert target
    // — including .claude, which synced cleanly before the failure.
    let st_work = state::load(&fx.work)
        .expect("load work sync state")
        .expect("work state present after failure");
    assert_eq!(st_work.op_id, pre_op_work, "work state = pre-sync op");
    let st_claude = state::load(&fx.claude)
        .expect("load claude sync state")
        .expect("claude state present after failure");
    assert_eq!(st_claude.op_id, pre_op_claude, "claude state = pre-sync op");
    // Manual revert (what `vc-x1 revert` will drive in -4) restores
    // the pre-sync state cleanly.
    op_restore(&fx.work, &st_work.op_id).expect("manual op restore");
    assert!(
        !has(&fx.work, "conflicts()"),
        "no conflicts after manual restore"
    );
}

/// Scenario 5b: `vc-x1 revert` after a failed sync — the full
/// inspect-then-undo loop. Same conflict fixture as scenario 5;
/// `revert_repos` restores every repo from its persisted snapshot
/// (including `.claude`, which synced cleanly before the failure),
/// clears the consumed state files, and a second revert errors
/// ("nothing to revert").
#[test]
fn revert_restores_after_failed_sync() {
    let fx = Fixture::new("revert-after-conflict");
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

    sync_repos(&fx.repos(), &default_params()).expect_err("sync should fail on conflicts");
    assert!(has(&fx.work, "conflicts()"), "conflicted state to undo");

    crate::revert::revert_repos(&fx.repos()).expect("revert should succeed");

    // Pre-sync state is back: bookmark, remote-tracking, no conflicts.
    assert_eq!(cid(&fx.work, "main"), pre_main, "main restored");
    assert_eq!(
        cid(&fx.work, "main@origin"),
        pre_remote,
        "main@origin restored (pre-fetch state)"
    );
    assert!(!has(&fx.work, "conflicts()"), "no conflicts after revert");
    // Consumed snapshots are cleared in both repos.
    assert!(
        state::load(&fx.work).expect("load work state").is_none(),
        "work state cleared by revert"
    );
    assert!(
        state::load(&fx.claude)
            .expect("load claude state")
            .is_none(),
        "claude state cleared by revert"
    );
    // Nothing left to revert — explicit error, not a silent no-op.
    let err = crate::revert::revert_repos(&fx.repos())
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("nothing to revert"),
        "expected nothing-to-revert error, got: {err}"
    );
}

/// Scenario 6: code repo behind with a clean `@`. Fetch fast-forwards
/// main; reposition then `jj new`s the empty `@` onto the new tip.
#[test]
fn sync_code_jj_new_when_behind() {
    let fx = Fixture::new("code-jjnew-behind");
    let remote_code = fx.base.join("remote-code.git");
    let remote_head = push_from_clone(
        &fx.base,
        &remote_code,
        "work2",
        "remote.txt",
        "remote\n",
        "feat: remote only",
    );
    sync_repos(&fx.repos(), &default_params()).expect("sync should succeed");
    assert_eq!(
        cid(&fx.work, "main"),
        remote_head,
        "main should ff to remote"
    );
    // @ is a fresh empty child of the new main.
    assert!(has(&fx.work, "@ & empty()"), "@ should be empty");
    assert!(has(&fx.work, "main::@"), "@ should be a child of main");
    assert_eq!(
        cid(&fx.work, "@-"),
        remote_head,
        "@- should be the new main"
    );
}

/// Scenario 7: code repo behind with a non-empty `@` and no
/// `--rebase`. Without a TTY the rebase prompt defaults to no, so `@`
/// is left in place (off the new main) and its changes are preserved.
#[test]
fn sync_code_skips_rebase_without_flag() {
    let fx = Fixture::new("code-skip-rebase");
    let remote_code = fx.base.join("remote-code.git");
    let remote_head = push_from_clone(
        &fx.base,
        &remote_code,
        "work2",
        "remote.txt",
        "remote\n",
        "feat: remote only",
    );
    // Uncommitted changes make @ non-empty.
    fs::write(fx.work.join("wip.txt"), "wip\n").expect("write wip");
    sync_repos(&fx.repos(), &default_params()).expect("sync should succeed");
    assert_eq!(
        cid(&fx.work, "main"),
        remote_head,
        "main should ff to remote"
    );
    // @ left off the new main; changes preserved in place.
    assert!(
        !has(&fx.work, "main::@"),
        "@ should be left off the new main"
    );
    assert_eq!(
        fs::read_to_string(fx.work.join("wip.txt")).unwrap(),
        "wip\n",
        "WIP preserved in place"
    );
}

/// Scenario 9: two independent clones of the same remote — the
/// "two machines" shape. Clone B is made first (its `main@origin`
/// is the pre-push head), clone A then commits and pushes; sync on
/// clone B must fast-forward B's `main` to A's pushed head and
/// reposition `@` onto it.
#[test]
fn sync_clone_ffs_main_after_peer_push() {
    let fx = Fixture::new("clone-peer-push");
    let remote_code = fx.base.join("remote-code.git");
    let clone_b = clone(&fx.base, &remote_code, "clone-b");
    let pre_main = cid(&clone_b, "main");
    let pushed = push_from_clone(
        &fx.base,
        &remote_code,
        "clone-a",
        "from-a.txt",
        "from clone A\n",
        "feat: from clone A",
    );
    assert_ne!(pre_main, pushed, "A's push should advance the remote");

    sync_repos(std::slice::from_ref(&clone_b), &default_params()).expect("sync should succeed");

    assert_eq!(
        cid(&clone_b, "main"),
        pushed,
        "clone B's main should ff to A's pushed head"
    );
    assert!(has(&clone_b, "@ & empty()"), "@ should be empty");
    assert_eq!(
        cid(&clone_b, "@-"),
        pushed,
        "@ should be repositioned onto the new main"
    );
}

/// Scenario 10: `--bookmark` is code-repo-only — the session repo
/// pins `main`. Syncing a feature bookmark while the session remote
/// advances `main` must still fast-forward the session repo's `main`
/// and reposition its `@`, and must not touch a `feature` bookmark
/// there. The code repo syncs `feature` as requested.
#[test]
fn sync_feature_bookmark_pins_session_to_main() {
    let fx = Fixture::new("feature-pins-session");
    // Code repo: create + push a feature bookmark so it tracks.
    jj(&fx.work, &["bookmark", "create", "feature", "-r", "main"]);
    jj(&fx.work, &["git", "push", "--bookmark", "feature"]);
    // Session remote advances main while feature work is underway.
    let remote_claude = fx.base.join("remote-claude.git");
    let remote_head = push_from_clone(
        &fx.base,
        &remote_claude,
        "claude2",
        "remote.md",
        "remote\n",
        "feat: remote-added",
    );

    let params = SyncParams {
        bookmark: "feature".to_string(),
        ..default_params()
    };
    sync_repos(&fx.repos(), &params).expect("sync should succeed");

    // Session repo synced main and repositioned @ onto it.
    assert_eq!(
        cid(&fx.claude, "main"),
        remote_head,
        "session main should ff to remote despite --bookmark feature"
    );
    assert!(has(&fx.claude, "@ & empty()"), "@ should be empty");
    assert!(has(&fx.claude, "main::@"), "@ should be a child of main");
    // No feature bookmark appears in the session repo.
    assert!(
        !has(&fx.claude, "bookmarks(exact:feature)"),
        "session repo must not grow a 'feature' bookmark"
    );
    // Code repo's feature bookmark is in sync with its remote.
    assert_eq!(
        cid(&fx.work, "feature"),
        cid(&fx.work, "feature@origin"),
        "code repo's feature bookmark synced as requested"
    );
}

/// Scenario 8: code repo behind with a non-empty `@` and `--rebase`.
/// The flag auto-confirms, so `@` is carried onto the new main with
/// its changes intact and no conflicts.
#[test]
fn sync_code_rebases_with_flag() {
    let fx = Fixture::new("code-rebase-flag");
    let remote_code = fx.base.join("remote-code.git");
    let remote_head = push_from_clone(
        &fx.base,
        &remote_code,
        "work2",
        "remote.txt",
        "remote\n",
        "feat: remote only",
    );
    fs::write(fx.work.join("wip.txt"), "wip\n").expect("write wip");
    sync_repos(&fx.repos(), &rebase_params()).expect("sync should succeed");
    assert_eq!(
        cid(&fx.work, "main"),
        remote_head,
        "main should ff to remote"
    );
    // @ rebased onto the new main; changes preserved.
    assert!(has(&fx.work, "main::@"), "@ should be rebased onto main");
    assert_eq!(
        fs::read_to_string(fx.work.join("wip.txt")).unwrap(),
        "wip\n",
        "WIP preserved across rebase"
    );
    assert!(!has(&fx.work, "conflicts()"), "no conflicts expected");
}

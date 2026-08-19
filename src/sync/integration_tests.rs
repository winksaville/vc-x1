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
use crate::test_helpers::{Fixture, cid, jj_ok, test_ctx};
use std::fs;
use std::process::Command;

use crate::common::{default_scope, find_workspace_root_from, scope_to_repos};
use crate::jj::{current_op_id, op_restore};

/// Resolver helpers must consume what `init --repo-local` produces.
///
/// Builds a real dual-repo fixture (bare git remotes + colocated jj
/// repos + the canonical `.vc-config.toml` pair init writes), then
/// drives the resolver chain `sync()` uses when `-R` is empty:
///
/// - `find_workspace_root_from(&fx.bot)` walks up to `fx.work`
///   (proves cwd-portability against init's `path = "/.claude"` /
///   `path = "/"` config split).
/// - `default_scope(Some(&fx.work))` reads the workspace config and
///   resolves to the dual-repo default.
/// - `scope_to_repos` maps each `Scope` shape to the right
///   absolute path(s) under the fixture.
///
/// Pure check on the resolver chain: does not invoke `sync()`
/// itself, since that walks `std::env::current_dir()` and parallel
/// `cargo test` makes cwd mutation unsafe.
#[test]
fn resolver_chain_against_init_repo_local() {
    let fx = Fixture::new("resolver-chain");

    // Walk-up from the bot side lands on the work root.
    assert_eq!(
        find_workspace_root_from(&fx.bot).as_deref(),
        Some(&*fx.work),
        "find_workspace_root should resolve from .claude up to work"
    );
    // Walk-up from the work root finds itself.
    assert_eq!(
        find_workspace_root_from(&fx.work).as_deref(),
        Some(&*fx.work)
    );

    // init writes bot = "/.claude", so the
    // workspace's default scope is dual.
    assert_eq!(
        default_scope(Some(&fx.work)),
        Scope(vec![Side::Work, Side::Bot])
    );

    // Each scope shape resolves to the right absolute path(s).
    assert_eq!(
        scope_to_repos(&Scope(vec![Side::Work, Side::Bot]), Some(&fx.work)).unwrap(),
        vec![fx.work.clone(), fx.bot.clone()]
    );
    assert_eq!(
        scope_to_repos(&Scope(vec![Side::Work]), Some(&fx.work)).unwrap(),
        vec![fx.work.clone()]
    );
    assert_eq!(
        scope_to_repos(&Scope(vec![Side::Bot]), Some(&fx.work)).unwrap(),
        vec![fx.bot.clone()]
    );

    // sync_repos accepts the resolved list and reports up-to-date:
    // the resolver's output is shaped the way sync expects.
    let resolved = scope_to_repos(&Scope(vec![Side::Work, Side::Bot]), Some(&fx.work)).unwrap();
    sync_repos(&mut test_ctx(), &resolved, &default_params())
        .expect("sync should succeed on resolved repos");
}

/// Default sync params (the normal atomic sync, no flags).
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

/// Default params with `--rebase` set (auto-confirm the work-repo
/// non-empty `@` rebase).
fn rebase_params() -> SyncParams {
    SyncParams {
        rebase: true,
        ..default_params()
    }
}

/// True when `revset` matches at least one commit in `repo`.
fn has(repo: &Path, revset: &str) -> bool {
    !jj_ok(
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
/// Sequence: write file -> describe `@` -> advance `main` to `@` ->
/// create a fresh empty `@` above it.
fn add_local_commit(repo: &Path, file: &str, content: &str, msg: &str) {
    fs::write(repo.join(file), content).expect("write local file");
    jj_ok(repo, &["describe", "@", "-m", msg]);
    jj_ok(repo, &["bookmark", "set", "main", "-r", "@"]);
    jj_ok(repo, &["new"]);
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
    jj_ok(&workdir, &["describe", "@", "-m", msg]);
    jj_ok(&workdir, &["bookmark", "set", "main", "-r", "@"]);
    jj_ok(&workdir, &["git", "push", "--bookmark", "main"]);
    cid(&workdir, "main")
}

/// Scenario 1: fresh fixture, nothing to do: `sync` leaves both
/// repos untouched and persists nothing (no `.vc-x1/` state, and
/// the pre-sync snapshots live and die with the invocation).
#[test]
fn sync_up_to_date() {
    let fx = Fixture::new("up-to-date");
    let work_main = cid(&fx.work, "main");
    let bot_main = cid(&fx.bot, "main");
    sync_repos(&mut test_ctx(), &fx.repos(), &default_params()).expect("sync should succeed");
    assert_eq!(cid(&fx.work, "main"), work_main);
    assert_eq!(cid(&fx.bot, "main"), bot_main);
    assert!(
        !fx.work.join(".vc-x1").exists(),
        "no state persisted (work)"
    );
    assert!(!fx.bot.join(".vc-x1").exists(), "no state persisted (bot)");
}

/// Scenario 2a: a non-empty `@` on top of main (simulates `/exit`
/// trailing session writes in `.claude`) when there's nothing new on
/// the remote. `@-` is already the main tip, so reposition no-ops:
/// `@` keeps its chid and the trailing writes stay in the working
/// copy, no sibling head, no `jj new` churn.
#[test]
fn sync_bot_noop_when_up_to_date() {
    let fx = Fixture::new("bot-noop-uptodate");
    let pre_main = cid(&fx.bot, "main");
    fs::write(fx.bot.join("trailing.jsonl"), "{\"line\":1}\n").expect("write trailing file");
    let pre_at = jj_ok(
        &fx.bot,
        &["log", "-r", "@", "--no-graph", "-T", "change_id.short(12)"],
    );
    sync_repos(&mut test_ctx(), &fx.repos(), &default_params()).expect("sync should succeed");
    // main didn't move.
    assert_eq!(cid(&fx.bot, "main"), pre_main, "main should not move");
    // @ is the same change, no jj new, no abandoned chid.
    let post_at = jj_ok(
        &fx.bot,
        &["log", "-r", "@", "--no-graph", "-T", "change_id.short(12)"],
    );
    assert_eq!(post_at, pre_at, "@ should keep its change id");
    // The trailing writes stay in the working copy.
    assert_eq!(
        fs::read_to_string(fx.bot.join("trailing.jsonl")).unwrap(),
        "{\"line\":1}\n",
        "trailing writes stay in @"
    );
    // No sibling head was created.
    assert!(
        !has(&fx.bot, "heads(all()) & ~empty() & ~@"),
        "no non-empty sibling head should appear"
    );
}

/// Scenario 2b: `@` has trailing writes and the remote advanced while
/// the session was offline. jj's fetch auto-ff's main, and the bot
/// repo then `jj new main`s onto the new tip, leaving the trailing
/// commit as a sibling head off the old tip.
#[test]
fn sync_bot_jj_new_when_main_moves() {
    let fx = Fixture::new("bot-jjnew-moved");
    let remote_bot = fx.base.join("remote-work.claude.git");
    let remote_head = push_from_clone(
        &fx.base,
        &remote_bot,
        "claude2",
        "remote.md",
        "remote\n",
        "feat: remote-added",
    );
    // Trailing writes on @
    fs::write(fx.bot.join("trailing.jsonl"), "{\"line\":2}\n").expect("write trailing file");
    sync_repos(&mut test_ctx(), &fx.repos(), &default_params()).expect("sync should succeed");
    assert_eq!(
        cid(&fx.bot, "main"),
        remote_head,
        "local main should match remote after auto-ff"
    );
    // @ is a fresh empty child of the new main.
    assert!(has(&fx.bot, "@ & empty()"), "@ should be empty");
    assert!(has(&fx.bot, "main::@"), "@ should be a child of main");
    // The trailing bot commit survives as a non-empty sibling head.
    assert!(
        has(&fx.bot, "heads(all()) & ~empty()"),
        "former @ preserved as a non-empty sibling head"
    );
    // The trailing file is no longer in the working copy.
    assert!(
        !fx.bot.join("trailing.jsonl").exists(),
        "@ no longer holds the trailing file"
    );
}

/// Scenario 2c: the bot repo refuses to reposition when `@-` is
/// not on main. A local bot commit ahead of main (main left
/// behind) puts `@-` off main's line, so `jj new main` would strand
/// it, sync errors instead.
#[test]
fn sync_bot_errors_when_at_parent_off_main() {
    let fx = Fixture::new("bot-off-main");
    // A described commit ahead of main, with a fresh @ above it, so
    // @- is ahead of (not on) main.
    fs::write(fx.bot.join("ahead.jsonl"), "{\"line\":9}\n").expect("write ahead file");
    jj_ok(&fx.bot, &["describe", "@", "-m", "feat: bot ahead"]);
    jj_ok(&fx.bot, &["new"]);

    let err = sync_repos(&mut test_ctx(), &fx.repos(), &default_params())
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("not on main"),
        "expected off-main refusal, got: {err}"
    );
}

/// Scenario 2c: `@` has trailing writes and local+remote both
/// modify the same file differently. Rebase produces conflicts,
/// so sync stops with the conflicted state left in place for
/// inspection (no auto-revert), persists nothing, and the pre-sync
/// op captured before the run remains a valid manual
/// `jj op restore` target. Trailing content stays on disk: the
/// rebase carries `@` along, it never rewrites the working-copy
/// file.
#[test]
fn sync_conflict_stops_and_keeps_state() {
    let fx = Fixture::new("trailing-conflict");
    let remote_bot = fx.base.join("remote-work.claude.git");
    push_from_clone(
        &fx.base,
        &remote_bot,
        "claude2",
        "shared.txt",
        "remote-version\n",
        "feat: remote shared",
    );
    // Local commit on main with conflicting content
    add_local_commit(
        &fx.bot,
        "shared.txt",
        "local-version\n",
        "feat: local shared (conflicting)",
    );
    // Trailing writes on new @
    fs::write(fx.bot.join("trailing.jsonl"), "{\"line\":3}\n").expect("write trailing file");

    let err = sync_repos(&mut test_ctx(), &fx.repos(), &default_params())
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("conflicts"),
        "expected conflict error, got: {err}"
    );

    // Stop-on-error: the conflicted state is still there to inspect.
    assert!(
        has(&fx.bot, "conflicts()"),
        "conflicted commits left in place for inspection"
    );
    // Nothing persisted, even on failure: recovery is the op id the
    // failure report prints (scenario 5a exercises the restore).
    assert!(
        !fx.bot.join(".vc-x1").exists(),
        "no state persisted on failure"
    );
    // Trailing content preserved on disk.
    assert_eq!(
        fs::read_to_string(fx.bot.join("trailing.jsonl")).unwrap(),
        "{\"line\":3}\n",
        "trailing content preserved across the stop"
    );
}

/// Scenario 3: local has commits not yet pushed, so sync
/// classifies `ahead` and leaves the local bookmark alone.
#[test]
fn sync_ahead_is_noop() {
    let fx = Fixture::new("ahead");
    add_local_commit(&fx.work, "local.txt", "local\n", "feat: local only");
    let ahead_head = cid(&fx.work, "main");
    sync_repos(&mut test_ctx(), &fx.repos(), &default_params()).expect("sync should succeed");
    assert_eq!(cid(&fx.work, "main"), ahead_head);
}

/// Scenario 4: clean divergence: both sides advance main on
/// different files, so sync rebases local onto remote and the
/// result is conflict-free.
#[test]
fn sync_diverged_rebases() {
    let fx = Fixture::new("diverged");
    let remote_work = fx.base.join("remote-work.git");
    let remote_head = push_from_clone(
        &fx.base,
        &remote_work,
        "work2",
        "remote.txt",
        "remote\n",
        "feat: remote only",
    );
    add_local_commit(&fx.work, "local.txt", "local\n", "feat: local only");

    sync_repos(&mut test_ctx(), &fx.repos(), &default_params()).expect("sync should succeed");

    // Remote tracking bookmark now points at the pushed remote commit.
    assert_eq!(
        cid(&fx.work, "main@origin"),
        remote_head,
        "main@origin should match pushed remote commit"
    );
    // Local main is ahead of remote: rebased local commit sits on top.
    let main_after = cid(&fx.work, "main");
    assert_ne!(
        main_after, remote_head,
        "local main should be ahead of remote after rebase"
    );
    // Remote is an ancestor of local post-rebase.
    let anc = jj_ok(
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
    let conflicts = jj_ok(
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

/// Scenario 5: conflicting divergence: both sides modify the
/// same path differently. Rebase produces conflicts, so sync
/// stops with the conflicted state in place (no auto-revert), persists
/// nothing, and the pre-sync op ids it printed remain valid manual
/// `jj op restore` targets.
#[test]
fn sync_diverged_conflict_stops_and_keeps_state() {
    let fx = Fixture::new("conflict");
    let remote_work = fx.base.join("remote-work.git");
    push_from_clone(
        &fx.base,
        &remote_work,
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
    let pre_op_bot = current_op_id(&fx.bot).expect("bot op id");

    let err = sync_repos(&mut test_ctx(), &fx.repos(), &default_params())
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
    // Nothing persisted in either repo, .claude included (it synced
    // cleanly before the failure).
    assert!(
        !fx.work.join(".vc-x1").exists(),
        "no state persisted (work)"
    );
    assert!(!fx.bot.join(".vc-x1").exists(), "no state persisted (bot)");
    // The op ids captured before the run (what the failure report
    // prints) drive the manual restore cleanly.
    op_restore(&fx.work, &pre_op_work).expect("manual op restore (work)");
    assert!(
        !has(&fx.work, "conflicts()"),
        "no conflicts after manual restore"
    );
    op_restore(&fx.bot, &pre_op_bot).expect("manual op restore (bot)");
}

/// Scenario 5b: the manual inspect-then-undo loop after a failed
/// sync: `jj op restore` to the op ids the failure report printed
/// restores every repo (including `.claude`, which synced cleanly
/// before the failure): bookmark, remote-tracking refs, and
/// conflict state all back to pre-sync. This is the documented
/// recovery now that `revert` is disabled.
#[test]
fn manual_op_restore_recovers_after_failed_sync() {
    let fx = Fixture::new("revert-after-conflict");
    let remote_work = fx.base.join("remote-work.git");
    push_from_clone(
        &fx.base,
        &remote_work,
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
    let pre_op_work = current_op_id(&fx.work).expect("work op id");
    let pre_op_bot = current_op_id(&fx.bot).expect("bot op id");

    sync_repos(&mut test_ctx(), &fx.repos(), &default_params())
        .expect_err("sync should fail on conflicts");
    assert!(has(&fx.work, "conflicts()"), "conflicted state to undo");

    op_restore(&fx.work, &pre_op_work).expect("op restore work");
    op_restore(&fx.bot, &pre_op_bot).expect("op restore bot");

    // Pre-sync state is back: bookmark, remote-tracking, no conflicts.
    assert_eq!(cid(&fx.work, "main"), pre_main, "main restored");
    assert_eq!(
        cid(&fx.work, "main@origin"),
        pre_remote,
        "main@origin restored (pre-fetch state)"
    );
    assert!(!has(&fx.work, "conflicts()"), "no conflicts after restore");
}

/// Scenario 6: work repo behind with a clean `@`. Fetch fast-forwards
/// main, and reposition then `jj new`s the empty `@` onto the new tip.
#[test]
fn sync_work_jj_new_when_behind() {
    let fx = Fixture::new("work-jjnew-behind");
    let remote_work = fx.base.join("remote-work.git");
    let remote_head = push_from_clone(
        &fx.base,
        &remote_work,
        "work2",
        "remote.txt",
        "remote\n",
        "feat: remote only",
    );
    sync_repos(&mut test_ctx(), &fx.repos(), &default_params()).expect("sync should succeed");
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

/// Scenario 7: work repo behind with a non-empty `@` and no
/// `--rebase`. Without a TTY the rebase prompt defaults to no, so `@`
/// is left in place (off the new main) and its changes are preserved.
#[test]
fn sync_work_skips_rebase_without_flag() {
    let fx = Fixture::new("work-skip-rebase");
    let remote_work = fx.base.join("remote-work.git");
    let remote_head = push_from_clone(
        &fx.base,
        &remote_work,
        "work2",
        "remote.txt",
        "remote\n",
        "feat: remote only",
    );
    // Uncommitted changes make @ non-empty.
    fs::write(fx.work.join("wip.txt"), "wip\n").expect("write wip");
    sync_repos(&mut test_ctx(), &fx.repos(), &default_params()).expect("sync should succeed");
    assert_eq!(
        cid(&fx.work, "main"),
        remote_head,
        "main should ff to remote"
    );
    // @ left off the new main, changes preserved in place.
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

/// Scenario 9: two independent clones of the same remote: the
/// "two machines" shape. Clone B is made first (its `main@origin`
/// is the pre-push head), then clone A commits and pushes. Sync on
/// clone B must fast-forward B's `main` to A's pushed head and
/// reposition `@` onto it.
#[test]
fn sync_clone_ffs_main_after_peer_push() {
    let fx = Fixture::new("clone-peer-push");
    let remote_work = fx.base.join("remote-work.git");
    let clone_b = clone(&fx.base, &remote_work, "clone-b");
    let pre_main = cid(&clone_b, "main");
    let pushed = push_from_clone(
        &fx.base,
        &remote_work,
        "clone-a",
        "from-a.txt",
        "from clone A\n",
        "feat: from clone A",
    );
    assert_ne!(pre_main, pushed, "A's push should advance the remote");

    sync_repos(
        &mut test_ctx(),
        std::slice::from_ref(&clone_b),
        &default_params(),
    )
    .expect("sync should succeed");

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

/// Scenario 10: `--bookmark` is work-repo-only: the bot repo
/// pins `main`. Syncing a feature bookmark while the bot remote
/// advances `main` must still fast-forward the bot repo's `main`
/// and reposition its `@`, and must not touch a `feature` bookmark
/// there. The work repo syncs `feature` as requested.
#[test]
fn sync_feature_bookmark_pins_bot_to_main() {
    let fx = Fixture::new("feature-pins-bot");
    // Work repo: create + push a feature bookmark so it tracks.
    jj_ok(&fx.work, &["bookmark", "create", "feature", "-r", "main"]);
    jj_ok(&fx.work, &["git", "push", "--bookmark", "feature"]);
    // Bot remote advances main while feature work is underway.
    let remote_bot = fx.base.join("remote-work.claude.git");
    let remote_head = push_from_clone(
        &fx.base,
        &remote_bot,
        "claude2",
        "remote.md",
        "remote\n",
        "feat: remote-added",
    );

    let params = SyncParams {
        bookmark: "feature".to_string(),
        ..default_params()
    };
    sync_repos(&mut test_ctx(), &fx.repos(), &params).expect("sync should succeed");

    // Bot repo synced main and repositioned @ onto it.
    assert_eq!(
        cid(&fx.bot, "main"),
        remote_head,
        "bot main should ff to remote despite --bookmark feature"
    );
    assert!(has(&fx.bot, "@ & empty()"), "@ should be empty");
    assert!(has(&fx.bot, "main::@"), "@ should be a child of main");
    // No feature bookmark appears in the bot repo.
    assert!(
        !has(&fx.bot, "bookmarks(exact:feature)"),
        "bot repo must not grow a 'feature' bookmark"
    );
    // Work repo's feature bookmark is in sync with its remote.
    assert_eq!(
        cid(&fx.work, "feature"),
        cid(&fx.work, "feature@origin"),
        "work repo's feature bookmark synced as requested"
    );
}

/// Scenario 8: work repo behind with a non-empty `@` and `--rebase`.
/// The flag auto-confirms, so `@` is carried onto the new main with
/// its changes intact and no conflicts.
#[test]
fn sync_work_rebases_with_flag() {
    let fx = Fixture::new("work-rebase-flag");
    let remote_work = fx.base.join("remote-work.git");
    let remote_head = push_from_clone(
        &fx.base,
        &remote_work,
        "work2",
        "remote.txt",
        "remote\n",
        "feat: remote only",
    );
    fs::write(fx.work.join("wip.txt"), "wip\n").expect("write wip");
    sync_repos(&mut test_ctx(), &fx.repos(), &rebase_params()).expect("sync should succeed");
    assert_eq!(
        cid(&fx.work, "main"),
        remote_head,
        "main should ff to remote"
    );
    // @ rebased onto the new main, changes preserved.
    assert!(has(&fx.work, "main::@"), "@ should be rebased onto main");
    assert_eq!(
        fs::read_to_string(fx.work.join("wip.txt")).unwrap(),
        "wip\n",
        "WIP preserved across rebase"
    );
    assert!(!has(&fx.work, "conflicts()"), "no conflicts expected");
}

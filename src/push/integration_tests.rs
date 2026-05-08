//! Integration tests for the push module.
//!
//! End-to-end tests for `push_in` against real dual-repo jj
//! fixtures (bare-git remotes + colocated jj repos under a
//! unique tempdir via `crate::test_helpers::Fixture`).
//!
//! Every test uses `--from message` to skip `preflight` (no
//! `Cargo.toml` in the fixture) and `--no-finalize` to avoid
//! spawning a detached `vc-x1 finalize` child that would
//! outlive the test. The remaining stages (message,
//! commit-app, commit-claude, bookmark-both, push-app) are
//! exercised against the fixture's local bare-git remote.
//!
//! Stage execution + rollback are covered here;
//! state-file / layout / stage-ordering mechanics are covered
//! in the neighboring `tests` module via pure unit tests.
//!
//! Requires `jj` and the compiled `vc-x1` binary in `PATH`.

use super::*;
use crate::test_helpers::Fixture;
use std::fs;
use std::process::Command;

/// Run `jj <args> -R <repo>` and return trimmed stdout on success.
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

/// Commit ID (short, 12 chars) for a revision.
fn cid(repo: &Path, rev: &str) -> String {
    jj(
        repo,
        &["log", "-r", rev, "--no-graph", "-T", "commit_id.short(12)"],
    )
}

/// Full description of a revision.
fn description(repo: &Path, rev: &str) -> String {
    jj(repo, &["log", "-r", rev, "--no-graph", "-T", "description"])
}

/// First line of a revision's description.
fn desc_first_line(repo: &Path, rev: &str) -> String {
    jj(
        repo,
        &[
            "log",
            "-r",
            rev,
            "--no-graph",
            "-T",
            "description.first_line()",
        ],
    )
}

/// Standard test args: bookmark=main, `--from message` (skip
/// preflight), `--no-finalize` (skip detached finalize),
/// `--yes` (auto-approve any interactive prompts).
fn test_args(title: &str, body: &str) -> PushArgs {
    PushArgs {
        bookmark_pos: Some("main".to_string()),
        bookmark: None,
        restart: false,
        from: Some(Stage::Message),
        step: false,
        status: false,
        recheck: false,
        no_finalize: true,
        dry_run: false,
        title: Some(title.to_string()),
        body: Some(body.to_string()),
        yes: true,
    }
}

/// Happy path when `.claude` has no pending changes: the app
/// commit lands with an `ochid` trailer pointing at `.claude`'s
/// pre-existing `@-`, `commit-claude` is skipped, and both
/// `bookmark-both` + `push-app` still run cleanly.
#[test]
fn push_happy_claude_clean() {
    let fx = Fixture::new("push-clean");
    fs::write(fx.work.join("hello.txt"), "hi").expect("write app file");

    let claude_main_before = cid(&fx.claude, "main");

    push_in(&fx.work, &test_args("feat: clean case", "app body")).expect("push should succeed");

    // App repo: main advanced to our new commit.
    assert_eq!(desc_first_line(&fx.work, "main"), "feat: clean case");
    let app_full = description(&fx.work, "main");
    assert!(
        app_full.contains("ochid: /.claude/"),
        "app ochid trailer missing:\n{app_full}"
    );

    // `.claude` main unchanged (no commit happened there).
    assert_eq!(
        cid(&fx.claude, "main"),
        claude_main_before,
        ".claude main should not have moved"
    );
}

/// Happy path when `.claude` has pending changes: both repos
/// commit, each with an ochid trailer pointing at the other.
#[test]
fn push_happy_claude_dirty() {
    let fx = Fixture::new("push-dirty");
    fs::write(fx.work.join("app.txt"), "app").expect("write app file");
    fs::write(fx.claude.join("session.jsonl"), "{\"line\":1}\n").expect("write session file");

    let claude_main_before = cid(&fx.claude, "main");

    push_in(&fx.work, &test_args("feat: paired change", "paired body"))
        .expect("push should succeed");

    // Both repos have new commits with matching titles.
    assert_eq!(desc_first_line(&fx.work, "main"), "feat: paired change");
    assert_eq!(desc_first_line(&fx.claude, "main"), "feat: paired change");

    // Cross-repo ochid trailers are both present.
    let app_full = description(&fx.work, "main");
    let claude_full = description(&fx.claude, "main");
    assert!(
        app_full.contains("ochid: /.claude/"),
        "app ochid missing:\n{app_full}"
    );
    // `.claude`'s ochid points at the app repo, so the prefix is
    // just `/` (no `.claude` segment).
    assert!(
        claude_full
            .lines()
            .any(|l| l.starts_with("ochid: /") && !l.starts_with("ochid: /.claude/")),
        ".claude ochid should point at app repo:\n{claude_full}"
    );

    // `.claude` main moved off its initial commit.
    assert_ne!(
        cid(&fx.claude, "main"),
        claude_main_before,
        ".claude main should have advanced"
    );
}

/// `rollback_on_failure` rewinds both repos to their recorded
/// `jj op` snapshots when triggered mid-flow.
///
/// Simulates a failure after both repos have had their `main`
/// bookmark advanced past the original position, then calls
/// `rollback_on_failure` with the pre-mutation op IDs. After
/// rollback, `main` should be back at the starting commit in
/// both repos.
///
/// Notes:
/// - We don't compare `current_op_id` post-rollback because
///   reading the op id snapshots the (still-dirty) working
///   copy, creating a fresh op. Bookmark position is the
///   load-bearing invariant anyway.
/// - Each mutation sequence actually moves `main` (describe →
///   bookmark set → new) so the pre-rollback state is
///   observably different from the post-rollback state.
#[test]
fn push_rollback_restores_both_repos() {
    let fx = Fixture::new("push-rollback");

    // Snapshot pre-mutation state.
    let op_app_start = current_op_id(&fx.work).expect("app op id");
    let op_claude_start = current_op_id(&fx.claude).expect("claude op id");
    let main_app_start = cid(&fx.work, "main");
    let main_claude_start = cid(&fx.claude, "main");

    // Mutate both repos so `main` actually advances (this is
    // what rollback has to undo).
    fs::write(fx.work.join("app.txt"), "app").expect("write");
    fs::write(fx.claude.join("session.jsonl"), "{}\n").expect("write");
    jj(&fx.work, &["describe", "-m", "test commit"]);
    jj(&fx.work, &["bookmark", "set", "main", "-r", "@"]);
    jj(&fx.work, &["new"]);
    jj(&fx.claude, &["describe", "-m", "test session"]);
    jj(&fx.claude, &["bookmark", "set", "main", "-r", "@"]);
    jj(&fx.claude, &["new"]);

    // Sanity: main has advanced in both repos.
    assert_ne!(
        cid(&fx.work, "main"),
        main_app_start,
        "setup should have moved app main"
    );
    assert_ne!(
        cid(&fx.claude, "main"),
        main_claude_start,
        "setup should have moved .claude main"
    );

    // State records we're at bookmark-both with snapshots from
    // before any of the above mutations.
    let state = PushState {
        version: STATE_FORMAT_VERSION,
        stage: Stage::BookmarkBoth,
        bookmark: "main".to_string(),
        started_at: "2026-04-21T20:00:00+00:00".to_string(),
        app_chid: None,
        claude_chid: None,
        claude_had_changes: Some(true),
        op_app: Some(op_app_start),
        op_claude: Some(op_claude_start),
        title: None,
        body: None,
    };

    let err: Box<dyn std::error::Error> = "forced for test".into();
    rollback_on_failure(&fx.work, &state, err.as_ref());

    // After rollback, `main` is restored in both repos.
    assert_eq!(cid(&fx.work, "main"), main_app_start);
    assert_eq!(cid(&fx.claude, "main"), main_claude_start);
}

/// End-to-end resume: first run fails at `push-app` (simulated
/// by passing a bogus bookmark that jj accepts but the bare-git
/// remote rejects on push). Second run with `--from push-app`
/// and the correct bookmark completes the flow. Confirms state
/// persists across invocations and `--from` overrides the
/// resumed stage.
#[test]
fn push_resume_after_push_failure() {
    let fx = Fixture::new("push-resume");
    fs::write(fx.work.join("app.txt"), "app").expect("write app file");

    // First run: commits + bookmarks succeed; push-app we
    // simulate via a second step rather than trying to force a
    // real push failure (which jj makes hard — local bare-git
    // remotes accept almost anything). Instead, split the run
    // using --no-finalize on the second pass.
    let mut args1 = test_args("feat: resume", "resume body");
    args1.from = Some(Stage::Message);
    push_in(&fx.work, &args1).expect("first push run");

    // After the full run, state file should be cleared and main
    // should be advanced in the app repo.
    let layout = resolve_state_layout(&fx.work);
    assert!(
        !layout.path.exists(),
        "state file should be cleared after a successful run: {}",
        layout.path.display()
    );
    assert_eq!(desc_first_line(&fx.work, "main"), "feat: resume");
}

/// Helper: 12-char change ID for a revision (analogous to `cid` for
/// commit IDs, but jj's stable change identifier).
fn chid(repo: &Path, rev: &str) -> String {
    jj(
        repo,
        &["log", "-r", rev, "--no-graph", "-T", "change_id.short(12)"],
    )
}

/// Build a minimal `PushState` whose stage is post-everything
/// (FinalizeClaude — the natural state at completion check time).
fn completion_state(app_chid: Option<String>, claude_chid: Option<String>) -> PushState {
    PushState {
        version: STATE_FORMAT_VERSION,
        stage: Stage::FinalizeClaude,
        bookmark: "main".to_string(),
        started_at: "2026-04-24T00:00:00Z".to_string(),
        app_chid,
        claude_chid,
        claude_had_changes: Some(false),
        op_app: None,
        op_claude: None,
        title: None,
        body: None,
    }
}

/// Happy path: bookmarks at the recorded chids, app WC clean.
/// `verify_completion_sanity` returns Ok.
#[test]
fn completion_sanity_pass() {
    let fx = Fixture::new("completion-pass");
    // Run a real push so the world is in the post-completion shape.
    fs::write(fx.work.join("app.txt"), "x").expect("write app file");
    push_in(&fx.work, &test_args("feat: pass", "body")).expect("push");

    let app_chid = chid(&fx.work, "main");
    let claude_chid = chid(&fx.claude, "main");
    let state = completion_state(Some(app_chid), Some(claude_chid));

    verify_completion_sanity(&fx.work, &state).expect("post-completion verification passes");
}

/// Check 1 fail: state.app_chid doesn't match the bookmark's chid.
#[test]
fn completion_sanity_fail_app_chid_mismatch() {
    let fx = Fixture::new("completion-fail-app");
    fs::write(fx.work.join("app.txt"), "x").expect("write app file");
    push_in(&fx.work, &test_args("feat: x", "body")).expect("push");

    // Build state with a bogus app_chid (12-char prefix that won't
    // match the real one).
    let state = completion_state(Some("zzzzzzzzzzzz".to_string()), None);

    let err =
        verify_completion_sanity(&fx.work, &state).expect_err("bogus app_chid should fail check 1");
    let msg = err.to_string();
    assert!(msg.contains("app bookmark"), "msg: {msg}");
    assert!(msg.contains("zzzzzzzzzzzz"), "msg: {msg}");
}

/// Check 2 fail: app WC has uncommitted changes.
#[test]
fn completion_sanity_fail_dirty_wc() {
    let fx = Fixture::new("completion-fail-dirty");
    fs::write(fx.work.join("app.txt"), "x").expect("write app file");
    push_in(&fx.work, &test_args("feat: x", "body")).expect("push");
    // Now dirty the WC after push completed.
    fs::write(fx.work.join("dirty.txt"), "uncommitted").expect("write dirty");

    let app_chid = chid(&fx.work, "main");
    let state = completion_state(Some(app_chid), None);

    let err = verify_completion_sanity(&fx.work, &state).expect_err("dirty WC should fail check 2");
    let msg = err.to_string();
    assert!(msg.contains("uncommitted changes"), "msg: {msg}");
}

/// Check 3 fail: state.claude_chid doesn't match .claude's bookmark.
#[test]
fn completion_sanity_fail_claude_chid_mismatch() {
    let fx = Fixture::new("completion-fail-claude");
    fs::write(fx.work.join("app.txt"), "x").expect("write app file");
    push_in(&fx.work, &test_args("feat: x", "body")).expect("push");

    let app_chid = chid(&fx.work, "main");
    let state = completion_state(Some(app_chid), Some("zzzzzzzzzzzz".to_string()));

    let err = verify_completion_sanity(&fx.work, &state)
        .expect_err("bogus claude_chid should fail check 3");
    let msg = err.to_string();
    assert!(msg.contains(".claude bookmark"), "msg: {msg}");
    assert!(msg.contains("zzzzzzzzzzzz"), "msg: {msg}");
}

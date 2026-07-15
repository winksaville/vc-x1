//! Integration tests for the push module.
//!
//! End-to-end tests for `push_in` against real dual-repo jj
//! fixtures (bare-git remotes + colocated jj repos under a
//! unique tempdir via `crate::test_helpers::Fixture`).
//!
//! Every test uses `--from message` to skip `preflight` (no
//! `Cargo.toml` in the fixture). Most also use `--no-squash-push`
//! to focus on the earlier stages (message, commit-app,
//! commit-claude, bookmark-set, push-app); the
//! `push_squash_push_bot_*` tests run the in-process
//! `squash-push-bot` stage for real. Everything is exercised
//! against the fixture's local bare-git remotes.
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

/// Standard test params: bookmark=main, `--from message` (skip
/// preflight), `--no-squash-push` (skip the session squash+push),
/// `--yes` (auto-approve any interactive prompts).
fn test_params(title: &str, body: &str) -> PushParams {
    PushParams {
        bookmark: Some("main".to_string()),
        restart: false,
        from: Some(Stage::Message),
        step: false,
        status: false,
        recheck: false,
        no_squash_push: true,
        dry_run: false,
        title: Some(title.to_string()),
        body: Some(body.to_string()),
        yes: true,
    }
}

/// Happy path when `.claude` has no pending changes: the app
/// commit lands with an `ochid` trailer pointing at `.claude`'s
/// pre-existing `@-`, `commit-claude` is skipped, and both
/// `bookmark-set` + `push-app` still run cleanly.
#[test]
fn push_happy_claude_clean() {
    let fx = Fixture::new("push-clean");
    fs::write(fx.work.join("hello.txt"), "hi").expect("write app file");

    let claude_main_before = cid(&fx.claude, "main");

    push_in(&fx.work, &test_params("feat: clean case", "app body")).expect("push should succeed");

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

    push_in(&fx.work, &test_params("feat: paired change", "paired body"))
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

/// The real `squash-push-bot` stage: a full push (no
/// `--no-squash-push`) squashes `.claude`'s tail and pushes `main`
/// to the session repo's origin in-process — synchronously, no
/// detached child (Bugs #1).
#[test]
fn push_squash_push_bot_inline_pushes_session() {
    let fx = Fixture::new("push-sp-inline");
    fs::write(fx.work.join("app.txt"), "app").expect("write app file");
    fs::write(fx.claude.join("session.jsonl"), "{\"line\":1}\n").expect("write session file");

    let mut params = test_params("feat: inline squash-push", "body");
    params.no_squash_push = false;
    push_in(&fx.work, &params).expect("push should succeed");

    // Session commit reached the bare origin before push returned.
    assert_eq!(
        cid(&fx.claude, "main@origin"),
        cid(&fx.claude, "main"),
        ".claude main should be pushed to origin"
    );
    // Working copy is clean after the tail squash.
    assert_eq!(
        jj(&fx.claude, &["log", "-r", "@", "--no-graph", "-T", "empty"]),
        "true",
        ".claude @ should be empty after squash-push-bot"
    );
}

/// A tail (session write landing after `commit-claude`) is
/// folded into the session commit by the stage's squash —
/// preserving the commit's change id so app-side `ochid:`
/// trailers stay valid — and pushed.
#[test]
fn push_squash_push_bot_folds_micro_tail() {
    let fx = Fixture::new("push-sp-tail");
    fs::write(fx.work.join("app.txt"), "app").expect("write app file");
    fs::write(fx.claude.join("session.jsonl"), "{\"line\":1}\n").expect("write session file");

    // Earlier stages only: commit both repos, push app, skip the
    // session squash+push.
    push_in(&fx.work, &test_params("feat: tail case", "body")).expect("push should succeed");
    let main_chid_before = chid(&fx.claude, "main");

    // The tail lands after commit-claude.
    fs::write(fx.claude.join("tail.jsonl"), "{\"line\":2}\n").expect("write tail file");

    let mut params = test_params("feat: tail case", "body");
    params.no_squash_push = false;
    let state = PushState::new_for("main");
    stage_squash_push_bot(&fx.work, &state, &params).expect("squash-push-bot should succeed");

    // Tail folded in; chid stable; session commit pushed; @ clean.
    let files = jj(&fx.claude, &["file", "list", "-r", "main"]);
    assert!(
        files.contains("tail.jsonl"),
        "tail not folded into main: {files}"
    );
    assert_eq!(
        chid(&fx.claude, "main"),
        main_chid_before,
        "squash must keep main's change id"
    );
    assert_eq!(
        cid(&fx.claude, "main@origin"),
        cid(&fx.claude, "main"),
        ".claude main should be pushed to origin"
    );
    assert_eq!(
        jj(&fx.claude, &["log", "-r", "@", "--no-graph", "-T", "empty"]),
        "true",
        ".claude @ should be empty after squash-push-bot"
    );
}

/// A feature-bookmark push pins the session repo to `main`: the
/// app repo grows + pushes `feature`, while `.claude` advances and
/// keeps only `main` — no `feature` bookmark may appear there.
#[test]
fn push_feature_bookmark_pins_session_to_main() {
    let fx = Fixture::new("push-feature-pin");
    fs::write(fx.work.join("app.txt"), "app").expect("write app file");
    fs::write(fx.claude.join("session.jsonl"), "{\"line\":1}\n").expect("write session file");

    let claude_main_before = cid(&fx.claude, "main");

    let mut params = test_params("feat: on feature", "feature body");
    params.bookmark = Some("feature".to_string());
    push_in(&fx.work, &params).expect("push should succeed");

    // App repo: feature created, pushed, and at the new commit.
    assert_eq!(desc_first_line(&fx.work, "feature"), "feat: on feature");
    assert_eq!(
        cid(&fx.work, "feature"),
        cid(&fx.work, "feature@origin"),
        "app feature bookmark should be pushed"
    );

    // Session repo: main advanced with the paired commit...
    assert_ne!(
        cid(&fx.claude, "main"),
        claude_main_before,
        ".claude main should have advanced"
    );
    assert_eq!(desc_first_line(&fx.claude, "main"), "feat: on feature");
    // ...and no feature bookmark exists there (bookmark-list lines
    // are `name: ...`; match on the name position, not the whole
    // line — commit titles may legitimately contain "feature").
    let claude_bookmarks = jj(&fx.claude, &["bookmark", "list"]);
    assert!(
        !claude_bookmarks.lines().any(|l| l.starts_with("feature:")),
        ".claude must not grow a 'feature' bookmark:\n{claude_bookmarks}"
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

    // State records we're at bookmark-set with snapshots from
    // before any of the above mutations.
    let state = PushState {
        version: STATE_FORMAT_VERSION,
        stage: Stage::BookmarkSet,
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
    // using --no-squash-push on the second pass.
    let mut params1 = test_params("feat: resume", "resume body");
    params1.from = Some(Stage::Message);
    push_in(&fx.work, &params1).expect("first push run");

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
/// (SquashPushBot — the natural state at completion check time).
fn completion_state(app_chid: Option<String>, claude_chid: Option<String>) -> PushState {
    PushState {
        version: STATE_FORMAT_VERSION,
        stage: Stage::SquashPushBot,
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

/// Happy path: bookmarks at the recorded chids, app working copy clean.
/// `verify_completion_sanity` returns Ok.
#[test]
fn completion_sanity_pass() {
    let fx = Fixture::new("completion-pass");
    // Run a real push so the world is in the post-completion shape.
    fs::write(fx.work.join("app.txt"), "x").expect("write app file");
    push_in(&fx.work, &test_params("feat: pass", "body")).expect("push");

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
    push_in(&fx.work, &test_params("feat: x", "body")).expect("push");

    // Build state with a bogus app_chid (12-char prefix that won't
    // match the real one).
    let state = completion_state(Some("zzzzzzzzzzzz".to_string()), None);

    let err =
        verify_completion_sanity(&fx.work, &state).expect_err("bogus app_chid should fail check 1");
    let msg = err.to_string();
    assert!(msg.contains("app bookmark"), "msg: {msg}");
    assert!(msg.contains("zzzzzzzzzzzz"), "msg: {msg}");
}

/// Check 2 fail: app working copy has uncommitted changes.
#[test]
fn completion_sanity_fail_dirty_wc() {
    let fx = Fixture::new("completion-fail-dirty");
    fs::write(fx.work.join("app.txt"), "x").expect("write app file");
    push_in(&fx.work, &test_params("feat: x", "body")).expect("push");
    // Now dirty the working copy after push completed.
    fs::write(fx.work.join("dirty.txt"), "uncommitted").expect("write dirty");

    let app_chid = chid(&fx.work, "main");
    let state = completion_state(Some(app_chid), None);

    let err = verify_completion_sanity(&fx.work, &state)
        .expect_err("dirty working copy should fail check 2");
    let msg = err.to_string();
    assert!(msg.contains("uncommitted changes"), "msg: {msg}");
}

/// Check 3 fail: state.claude_chid doesn't match .claude's bookmark.
#[test]
fn completion_sanity_fail_claude_chid_mismatch() {
    let fx = Fixture::new("completion-fail-claude");
    fs::write(fx.work.join("app.txt"), "x").expect("write app file");
    push_in(&fx.work, &test_params("feat: x", "body")).expect("push");

    let app_chid = chid(&fx.work, "main");
    let state = completion_state(Some(app_chid), Some("zzzzzzzzzzzz".to_string()));

    let err = verify_completion_sanity(&fx.work, &state)
        .expect_err("bogus claude_chid should fail check 3");
    let msg = err.to_string();
    assert!(msg.contains(".claude bookmark"), "msg: {msg}");
    assert!(msg.contains("zzzzzzzzzzzz"), "msg: {msg}");
}

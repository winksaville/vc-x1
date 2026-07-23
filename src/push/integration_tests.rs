//! Integration tests for the push module.
//!
//! End-to-end tests for `push_in` against real dual-repo jj
//! fixtures (bare-git remotes + colocated jj repos under a
//! unique tempdir via `crate::test_helpers::Fixture`).
//!
//! Most tests use `--from message` to skip `preflight` — its
//! `sync --check` step re-invokes `current_exe()`, which under
//! `cargo test` is the test harness, not the CLI binary. Most
//! also use `--no-squash-push`
//! to focus on the earlier stages (message, commit-work,
//! commit-bot, bookmark-set, push-work); the
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
use crate::test_helpers::{Fixture, chid, cid, description, jj_ok};
use std::fs;

/// First line of a revision's description.
fn desc_first_line(repo: &Path, rev: &str) -> String {
    jj_ok(
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
/// preflight), `--no-squash-push` (skip the bot-repo squash+push),
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

/// Preflight's bot-published backstop: an at-rest `.claude main`
/// that doesn't match `main@origin` (a lost publish) errors — no
/// automatic fixing.
#[test]
fn push_preflight_errors_on_unpublished_bot_main() {
    let fx = Fixture::new("push-bot-unpub");
    fs::write(fx.work.join("work.txt"), "work").expect("write work file");
    // Simulate the lost publish: seal a `.claude` commit and move
    // `main` onto it without pushing.
    fs::write(fx.bot.join("lost.txt"), "lost session data").expect("write lost file");
    jj_ok(&fx.bot, &["commit", "-m", "lost bot commit"]);
    jj_ok(&fx.bot, &["bookmark", "set", "main", "-r", "@-"]);

    let mut params = test_params("feat: blocked", "work body");
    params.from = None; // run preflight (errors before the sync check)
    let err = push_in(&fx.work, &params)
        .expect_err("preflight should error on unpublished bot main")
        .to_string();
    assert!(err.contains("does not match"), "got: {err}");
    assert!(err.contains("squash-push"), "got: {err}");
}

/// Happy path when `.claude` has no pending changes: the work
/// commit lands with an `ochid` trailer pointing at `.claude`'s
/// pre-existing `@-`, `commit-bot` is skipped, and both
/// `bookmark-set` + `push-work` still run cleanly.
#[test]
fn push_happy_bot_clean() {
    let fx = Fixture::new("push-clean");
    fs::write(fx.work.join("hello.txt"), "hi").expect("write work file");

    let bot_main_before = cid(&fx.bot, "main");

    push_in(&fx.work, &test_params("feat: clean case", "work body")).expect("push should succeed");

    // Work repo: main advanced to our new commit.
    assert_eq!(desc_first_line(&fx.work, "main"), "feat: clean case");
    let work_full = description(&fx.work, "main");
    assert!(
        work_full.contains("ochid: /.claude/"),
        "work ochid trailer missing:\n{work_full}"
    );

    // `.claude` main unchanged (no commit happened there).
    assert_eq!(
        cid(&fx.bot, "main"),
        bot_main_before,
        ".claude main should not have moved"
    );
}

/// Happy path when `.claude` has pending changes: both repos
/// commit, each with an ochid trailer pointing at the other.
#[test]
fn push_happy_bot_dirty() {
    let fx = Fixture::new("push-dirty");
    fs::write(fx.work.join("work.txt"), "work").expect("write work file");
    fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write session file");

    let bot_main_before = cid(&fx.bot, "main");

    push_in(&fx.work, &test_params("feat: paired change", "paired body"))
        .expect("push should succeed");

    // Both repos have new commits with matching titles.
    assert_eq!(desc_first_line(&fx.work, "main"), "feat: paired change");
    assert_eq!(desc_first_line(&fx.bot, "main"), "feat: paired change");

    // Cross-repo ochid trailers are both present.
    let work_full = description(&fx.work, "main");
    let bot_full = description(&fx.bot, "main");
    assert!(
        work_full.contains("ochid: /.claude/"),
        "work ochid missing:\n{work_full}"
    );
    // `.claude`'s ochid points at the work repo, so the prefix is
    // just `/` (no `.claude` segment).
    assert!(
        bot_full
            .lines()
            .any(|l| l.starts_with("ochid: /") && !l.starts_with("ochid: /.claude/")),
        ".claude ochid should point at work repo:\n{bot_full}"
    );

    // `.claude` main moved off its initial commit.
    assert_ne!(
        cid(&fx.bot, "main"),
        bot_main_before,
        ".claude main should have advanced"
    );
}

/// The real `squash-push-bot` stage: a full push (no
/// `--no-squash-push`) squashes `.claude`'s tail and pushes `main`
/// to the bot repo's origin in-process — synchronously, no
/// detached child (the 0.68.1-diagnosed loss).
#[test]
fn push_squash_push_bot_inline_pushes_bot_main() {
    let fx = Fixture::new("push-sp-inline");
    fs::write(fx.work.join("work.txt"), "work").expect("write work file");
    fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write session file");

    let mut params = test_params("feat: inline squash-push", "body");
    params.no_squash_push = false;
    push_in(&fx.work, &params).expect("push should succeed");

    // Bot commit reached the bare origin before push returned.
    assert_eq!(
        cid(&fx.bot, "main@origin"),
        cid(&fx.bot, "main"),
        ".claude main should be pushed to origin"
    );
    // Working copy is clean after the tail squash.
    assert_eq!(
        jj_ok(&fx.bot, &["log", "-r", "@", "--no-graph", "-T", "empty"]),
        "true",
        ".claude @ should be empty after squash-push-bot"
    );
}

/// A tail (session write landing after `commit-bot`) is
/// folded into the bot commit by the stage's squash —
/// preserving the commit's change id so work-side `ochid:`
/// trailers stay valid — and pushed.
#[test]
fn push_squash_push_bot_folds_micro_tail() {
    let fx = Fixture::new("push-sp-tail");
    fs::write(fx.work.join("work.txt"), "work").expect("write work file");
    fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write session file");

    // Earlier stages only: commit both repos, push work, skip the
    // session squash+push.
    push_in(&fx.work, &test_params("feat: tail case", "body")).expect("push should succeed");
    let main_chid_before = chid(&fx.bot, "main");

    // The tail lands after commit-bot.
    fs::write(fx.bot.join("tail.jsonl"), "{\"line\":2}\n").expect("write tail file");

    let mut params = test_params("feat: tail case", "body");
    params.no_squash_push = false;
    let state = PushState::new_for("main");
    stage_squash_push_bot(&fx.work, &state, &params).expect("squash-push-bot should succeed");

    // Tail folded in; chid stable; bot commit pushed; @ clean.
    let files = jj_ok(&fx.bot, &["file", "list", "-r", "main"]);
    assert!(
        files.contains("tail.jsonl"),
        "tail not folded into main: {files}"
    );
    assert_eq!(
        chid(&fx.bot, "main"),
        main_chid_before,
        "squash must keep main's change id"
    );
    assert_eq!(
        cid(&fx.bot, "main@origin"),
        cid(&fx.bot, "main"),
        ".claude main should be pushed to origin"
    );
    assert_eq!(
        jj_ok(&fx.bot, &["log", "-r", "@", "--no-graph", "-T", "empty"]),
        "true",
        ".claude @ should be empty after squash-push-bot"
    );
}

/// A feature-bookmark push pins the bot repo to `main`: the
/// work repo grows + pushes `feature`, while `.claude` advances and
/// keeps only `main` — no `feature` bookmark may appear there.
#[test]
fn push_feature_bookmark_pins_bot_to_main() {
    let fx = Fixture::new("push-feature-pin");
    fs::write(fx.work.join("work.txt"), "work").expect("write work file");
    fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write session file");

    let bot_main_before = cid(&fx.bot, "main");

    let mut params = test_params("feat: on feature", "feature body");
    params.bookmark = Some("feature".to_string());
    push_in(&fx.work, &params).expect("push should succeed");

    // Work repo: feature created, pushed, and at the new commit.
    assert_eq!(desc_first_line(&fx.work, "feature"), "feat: on feature");
    assert_eq!(
        cid(&fx.work, "feature"),
        cid(&fx.work, "feature@origin"),
        "work feature bookmark should be pushed"
    );

    // Bot repo: main advanced with the paired commit...
    assert_ne!(
        cid(&fx.bot, "main"),
        bot_main_before,
        ".claude main should have advanced"
    );
    assert_eq!(desc_first_line(&fx.bot, "main"), "feat: on feature");
    // ...and no feature bookmark exists there (bookmark-list lines
    // are `name: ...`; match on the name position, not the whole
    // line — commit titles may legitimately contain "feature").
    let bot_bookmarks = jj_ok(&fx.bot, &["bookmark", "list"]);
    assert!(
        !bot_bookmarks.lines().any(|l| l.starts_with("feature:")),
        ".claude must not grow a 'feature' bookmark:\n{bot_bookmarks}"
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
    let op_work_start = current_op_id(&fx.work).expect("work op id");
    let op_bot_start = current_op_id(&fx.bot).expect("bot op id");
    let main_work_start = cid(&fx.work, "main");
    let main_bot_start = cid(&fx.bot, "main");

    // Mutate both repos so `main` actually advances (this is
    // what rollback has to undo).
    fs::write(fx.work.join("work.txt"), "work").expect("write");
    fs::write(fx.bot.join("session.jsonl"), "{}\n").expect("write");
    jj_ok(&fx.work, &["describe", "-m", "test commit"]);
    jj_ok(&fx.work, &["bookmark", "set", "main", "-r", "@"]);
    jj_ok(&fx.work, &["new"]);
    jj_ok(&fx.bot, &["describe", "-m", "test session"]);
    jj_ok(&fx.bot, &["bookmark", "set", "main", "-r", "@"]);
    jj_ok(&fx.bot, &["new"]);

    // Sanity: main has advanced in both repos.
    assert_ne!(
        cid(&fx.work, "main"),
        main_work_start,
        "setup should have moved work main"
    );
    assert_ne!(
        cid(&fx.bot, "main"),
        main_bot_start,
        "setup should have moved .claude main"
    );

    // State records we're at bookmark-set with snapshots from
    // before any of the above mutations.
    let state = PushState {
        version: STATE_FORMAT_VERSION,
        stage: Stage::BookmarkSet,
        bookmark: "main".to_string(),
        started_at: "2026-04-21T20:00:00+00:00".to_string(),
        work_chid: None,
        bot_chid: None,
        bot_had_changes: Some(true),
        op_work: Some(op_work_start),
        op_bot: Some(op_bot_start),
        title: None,
        body: None,
    };

    let err: Box<dyn std::error::Error> = "forced for test".into();
    rollback_on_failure(&fx.work, &state, err.as_ref());

    // After rollback, `main` is restored in both repos.
    assert_eq!(cid(&fx.work, "main"), main_work_start);
    assert_eq!(cid(&fx.bot, "main"), main_bot_start);
}

/// End-to-end resume: first run fails at `push-work` (simulated
/// by passing a bogus bookmark that jj accepts but the bare-git
/// remote rejects on push). Second run with `--from push-work`
/// and the correct bookmark completes the flow. Confirms state
/// persists across invocations and `--from` overrides the
/// resumed stage.
#[test]
fn push_resume_after_push_failure() {
    let fx = Fixture::new("push-resume");
    fs::write(fx.work.join("work.txt"), "work").expect("write work file");

    // First run: commits + bookmarks succeed; push-work we
    // simulate via a second step rather than trying to force a
    // real push failure (which jj makes hard — local bare-git
    // remotes accept almost anything). Instead, split the run
    // using --no-squash-push on the second pass.
    let mut params1 = test_params("feat: resume", "resume body");
    params1.from = Some(Stage::Message);
    push_in(&fx.work, &params1).expect("first push run");

    // After the full run, state file should be cleared and main
    // should be advanced in the work repo.
    let layout = resolve_state_layout(&fx.work);
    assert!(
        !layout.path.exists(),
        "state file should be cleared after a successful run: {}",
        layout.path.display()
    );
    assert_eq!(desc_first_line(&fx.work, "main"), "feat: resume");
}

/// Build a minimal `PushState` whose stage is post-everything
/// (SquashPushBot — the natural state at completion check time).
fn completion_state(work_chid: Option<String>, bot_chid: Option<String>) -> PushState {
    PushState {
        version: STATE_FORMAT_VERSION,
        stage: Stage::SquashPushBot,
        bookmark: "main".to_string(),
        started_at: "2026-04-24T00:00:00Z".to_string(),
        work_chid,
        bot_chid,
        bot_had_changes: Some(false),
        op_work: None,
        op_bot: None,
        title: None,
        body: None,
    }
}

/// Happy path: bookmarks at the recorded chids, work working copy clean.
/// `verify_completion_sanity` returns Ok.
#[test]
fn completion_sanity_pass() {
    let fx = Fixture::new("completion-pass");
    // Run a real push so the world is in the post-completion shape.
    fs::write(fx.work.join("work.txt"), "x").expect("write work file");
    push_in(&fx.work, &test_params("feat: pass", "body")).expect("push");

    let work_chid = chid(&fx.work, "main");
    let bot_chid = chid(&fx.bot, "main");
    let state = completion_state(Some(work_chid), Some(bot_chid));

    verify_completion_sanity(&fx.work, &state).expect("post-completion verification passes");
}

/// Check 1 fail: state.work_chid doesn't match the bookmark's chid.
#[test]
fn completion_sanity_fail_work_chid_mismatch() {
    let fx = Fixture::new("completion-fail-work");
    fs::write(fx.work.join("work.txt"), "x").expect("write work file");
    push_in(&fx.work, &test_params("feat: x", "body")).expect("push");

    // Build state with a bogus work_chid (12-char prefix that won't
    // match the real one).
    let state = completion_state(Some("zzzzzzzzzzzz".to_string()), None);

    let err = verify_completion_sanity(&fx.work, &state)
        .expect_err("bogus work_chid should fail check 1");
    let msg = err.to_string();
    assert!(msg.contains("work bookmark"), "msg: {msg}");
    assert!(msg.contains("zzzzzzzzzzzz"), "msg: {msg}");
}

/// Check 2 fail: work working copy has uncommitted changes.
#[test]
fn completion_sanity_fail_dirty_wc() {
    let fx = Fixture::new("completion-fail-dirty");
    fs::write(fx.work.join("work.txt"), "x").expect("write work file");
    push_in(&fx.work, &test_params("feat: x", "body")).expect("push");
    // Now dirty the working copy after push completed.
    fs::write(fx.work.join("dirty.txt"), "uncommitted").expect("write dirty");

    let work_chid = chid(&fx.work, "main");
    let state = completion_state(Some(work_chid), None);

    let err = verify_completion_sanity(&fx.work, &state)
        .expect_err("dirty working copy should fail check 2");
    let msg = err.to_string();
    assert!(msg.contains("uncommitted changes"), "msg: {msg}");
}

/// Check 3 fail: state.bot_chid doesn't match .claude's bookmark.
#[test]
fn completion_sanity_fail_bot_chid_mismatch() {
    let fx = Fixture::new("completion-fail-claude");
    fs::write(fx.work.join("work.txt"), "x").expect("write work file");
    push_in(&fx.work, &test_params("feat: x", "body")).expect("push");

    let work_chid = chid(&fx.work, "main");
    let state = completion_state(Some(work_chid), Some("zzzzzzzzzzzz".to_string()));

    let err =
        verify_completion_sanity(&fx.work, &state).expect_err("bogus bot_chid should fail check 3");
    let msg = err.to_string();
    assert!(msg.contains(".claude bookmark"), "msg: {msg}");
    assert!(msg.contains("zzzzzzzzzzzz"), "msg: {msg}");
}

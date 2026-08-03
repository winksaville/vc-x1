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
use crate::test_helpers::{Fixture, chid, cid, description, jj_ok, test_ctx};
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

/// Standard test params: bookmark=main, `--no-squash-push` (skip
/// the bot-repo squash+push), `--yes` (auto-approve any interactive
/// prompts).
fn test_params(title: &str, body: &str) -> PushParams {
    PushParams {
        bookmark: Some("main".to_string()),
        step: false,
        no_squash_push: true,
        dry_run: false,
        title: Some(title.to_string()),
        body: Some(body.to_string()),
        yes: true,
    }
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

    push_in(
        &mut test_ctx(),
        &fx.work,
        &test_params("feat: clean case", "work body"),
    )
    .expect("push should succeed");

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

/// bugs.md #4: a work repo whose `@` is empty (the rung was
/// committed by hand before invoking push) must not mint a
/// stamped empty duplicate on top of the real commit. The
/// pre-made commit is published as-is and the bot's ochid points
/// at *it*, not at a duplicate.
#[test]
fn push_empty_work_at_skips_commit_work() {
    let fx = Fixture::new("push-empty-work");
    fs::write(fx.work.join("hand.txt"), "hand-made").expect("write work file");
    jj_ok(
        &fx.work,
        &["commit", "-m", "feat: committed by hand", "-m", "body"],
    );
    fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write session file");

    let hand_chid = chid(&fx.work, "@-");

    push_in(
        &mut test_ctx(),
        &fx.work,
        &test_params("feat: would duplicate", "ignored body"),
    )
    .expect("push should succeed");

    // The hand-made commit is what the bookmark publishes — no
    // empty duplicate carrying the supplied title on top of it.
    assert_eq!(desc_first_line(&fx.work, "main"), "feat: committed by hand");
    assert_eq!(chid(&fx.work, "main"), hand_chid);

    // The bot commit's ochid names the real commit, not a duplicate.
    let bot_full = description(&fx.bot, "main");
    assert!(
        bot_full.contains(&format!("ochid: /{hand_chid}")),
        "bot ochid should point at the hand-made commit {hand_chid}:\n{bot_full}"
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

    push_in(
        &mut test_ctx(),
        &fx.work,
        &test_params("feat: paired change", "paired body"),
    )
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
    push_in(&mut test_ctx(), &fx.work, &params).expect("push should succeed");

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
    push_in(
        &mut test_ctx(),
        &fx.work,
        &test_params("feat: tail case", "body"),
    )
    .expect("push should succeed");
    let main_chid_before = chid(&fx.bot, "main");

    // The tail lands after commit-bot.
    fs::write(fx.bot.join("tail.jsonl"), "{\"line\":2}\n").expect("write tail file");

    let mut params = test_params("feat: tail case", "body");
    params.no_squash_push = false;
    stage_squash_push_bot(&mut test_ctx(), &fx.work, &params)
        .expect("squash-push-bot should succeed");

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
    push_in(&mut test_ctx(), &fx.work, &params).expect("push should succeed");

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

    // Snapshots from before any of the above mutations.
    let snapshots = Snapshots {
        work: op_work_start,
        bot: op_bot_start,
    };

    let err: Box<dyn std::error::Error> = "forced for test".into();
    rollback_on_failure(&fx.work, &snapshots, err.as_ref());

    // After rollback, `main` is restored in both repos.
    assert_eq!(cid(&fx.work, "main"), main_work_start);
    assert_eq!(cid(&fx.bot, "main"), main_bot_start);
}

/// Build a `Run` carrying the chids a completed push would have.
fn completion_run(work_chid: String, bot_chid: String) -> Run {
    Run {
        title: String::new(),
        body: String::new(),
        work_chid,
        bot_chid,
        bot_had_changes: false,
    }
}

/// Happy path: bookmarks at the run's chids, work working copy
/// clean. `verify_completion` returns Ok.
#[test]
fn completion_sanity_pass() {
    let fx = Fixture::new("completion-pass");
    // Run a real push so the world is in the post-completion shape.
    fs::write(fx.work.join("work.txt"), "x").expect("write work file");
    push_in(
        &mut test_ctx(),
        &fx.work,
        &test_params("feat: pass", "body"),
    )
    .expect("push");

    let work_chid = chid(&fx.work, "main");
    let bot_chid = chid(&fx.bot, "main");
    let run = completion_run(work_chid, bot_chid);

    verify_completion(&fx.work, "main", &run).expect("post-completion verification passes");
}

/// Check 1 fail: the run's work_chid doesn't match the bookmark.
#[test]
fn completion_sanity_fail_work_chid_mismatch() {
    let fx = Fixture::new("completion-fail-work");
    fs::write(fx.work.join("work.txt"), "x").expect("write work file");
    push_in(&mut test_ctx(), &fx.work, &test_params("feat: x", "body")).expect("push");

    // A bogus work_chid (12-char prefix that won't match the real one).
    let bot_chid = chid(&fx.bot, "main");
    let run = completion_run("zzzzzzzzzzzz".to_string(), bot_chid);

    let err =
        verify_completion(&fx.work, "main", &run).expect_err("bogus work_chid should fail check 1");
    let msg = err.to_string();
    assert!(msg.contains("work bookmark"), "msg: {msg}");
    assert!(msg.contains("zzzzzzzzzzzz"), "msg: {msg}");
}

/// Check 2 fail: work working copy has uncommitted changes.
#[test]
fn completion_sanity_fail_dirty_wc() {
    let fx = Fixture::new("completion-fail-dirty");
    fs::write(fx.work.join("work.txt"), "x").expect("write work file");
    push_in(&mut test_ctx(), &fx.work, &test_params("feat: x", "body")).expect("push");
    // Now dirty the working copy after push completed.
    fs::write(fx.work.join("dirty.txt"), "uncommitted").expect("write dirty");

    let work_chid = chid(&fx.work, "main");
    let bot_chid = chid(&fx.bot, "main");
    let run = completion_run(work_chid, bot_chid);

    let err = verify_completion(&fx.work, "main", &run)
        .expect_err("dirty working copy should fail check 2");
    let msg = err.to_string();
    assert!(msg.contains("uncommitted changes"), "msg: {msg}");
}

/// Check 3 fail: the run's bot_chid doesn't match .claude's bookmark.
#[test]
fn completion_sanity_fail_bot_chid_mismatch() {
    let fx = Fixture::new("completion-fail-claude");
    fs::write(fx.work.join("work.txt"), "x").expect("write work file");
    push_in(&mut test_ctx(), &fx.work, &test_params("feat: x", "body")).expect("push");

    let work_chid = chid(&fx.work, "main");
    let run = completion_run(work_chid, "zzzzzzzzzzzz".to_string());

    let err =
        verify_completion(&fx.work, "main", &run).expect_err("bogus bot_chid should fail check 3");
    let msg = err.to_string();
    assert!(msg.contains(".claude bookmark"), "msg: {msg}");
    assert!(msg.contains("zzzzzzzzzzzz"), "msg: {msg}");
}

/// The property that replaced resume: rerunning a completed push
/// is safe and changes nothing. Every stage no-ops — commit-work
/// skips an empty `@`, `message` doesn't demand a title when
/// neither side will commit, bookmark-set is a set, and jj's push
/// reports nothing to do.
#[test]
fn push_rerun_after_completion_is_a_noop() {
    let fx = Fixture::new("push-rerun");
    fs::write(fx.work.join("work.txt"), "work").expect("write work file");
    fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write session file");

    push_in(
        &mut test_ctx(),
        &fx.work,
        &test_params("feat: once", "body"),
    )
    .expect("first push");
    let work_after = cid(&fx.work, "main");
    let bot_after = cid(&fx.bot, "main");

    // Rerun with no message at all — nothing is pending, so none is
    // needed. Before the message guard this errored with
    // "--yes given but --title/--body missing".
    let mut params = test_params("unused", "unused");
    params.title = None;
    params.body = None;
    push_in(&mut test_ctx(), &fx.work, &params).expect("rerunning a completed push should succeed");

    assert_eq!(
        cid(&fx.work, "main"),
        work_after,
        "work main should not move"
    );
    assert_eq!(
        cid(&fx.bot, "main"),
        bot_after,
        ".claude main should not move"
    );
    assert_eq!(desc_first_line(&fx.work, "main"), "feat: once");
}

/// A publish-only run: the commits exist and only the bookmark and
/// the remote need advancing — the trapezoid recipe's final step,
/// which used to require `--from bookmark-set`. A bare push now
/// does it, with no message.
#[test]
fn push_publish_only_run_advances_bookmark() {
    let fx = Fixture::new("push-publish-only");
    fs::write(fx.work.join("work.txt"), "work").expect("write work file");
    jj_ok(&fx.work, &["commit", "-m", "feat: sealed by hand"]);

    let sealed = chid(&fx.work, "@-");

    let mut params = test_params("unused", "unused");
    params.title = None;
    params.body = None;
    push_in(&mut test_ctx(), &fx.work, &params).expect("publish-only push should succeed");

    assert_eq!(chid(&fx.work, "main"), sealed);
    assert_eq!(desc_first_line(&fx.work, "main"), "feat: sealed by hand");
}

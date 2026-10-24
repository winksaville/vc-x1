//! Integration tests for the push module.
//!
//! End-to-end tests for `push_in` against real dual-repo jj
//! fixtures (bare-git remotes + colocated jj repos under a unique
//! tempdir via `crate::test_helpers::Fixture`). The pair is made by
//! `commit::commit`, the way a rung's is, and push publishes it.
//!
//! Requires `jj` and the compiled `vc-x1` binary in `PATH`.

use super::*;
use crate::commit::{CommitParams, Target, commit};
use crate::options_flags::scope::Side;
use crate::test_helpers::{Fixture, FixturePor, chid, cid, description, jj_ok, test_ctx};
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

/// Whether a revision's working copy is empty.
fn is_empty(repo: &Path, rev: &str) -> bool {
    jj_ok(repo, &["log", "-r", rev, "--no-graph", "-T", "empty"]) == "true"
}

/// The fixture's pair, by the paths the resolver would give.
fn pair(fx: &Fixture) -> Pair {
    Pair {
        work: fx.work.clone(),
        agent: fx.bot.clone(),
    }
}

/// Both sides, work first.
fn both() -> Scope {
    Scope(vec![Side::Work, Side::Bot])
}

/// Standard test params: the line's bookmark, `-s b`, no gates.
fn test_params() -> PushParams {
    PushParams {
        bookmark: None,
        repo: None,
        scope: RepoScope::Sides(both()),
        step: false,
        dry_run: false,
        yes: true,
    }
}

/// Make a rung's pair in the fixture the way `commit` does: a work
/// file and a session line, then `commit -s b` (`agent_now`) or
/// `-s w`. Returns the work and agent `@` change ids read before
/// the commit, which the commits keep. No bookmark is advanced, so
/// the line stays as the test arranged it for push to resolve.
fn make_pair(fx: &Fixture, title: &str, agent_now: bool) -> (String, String) {
    fs::write(fx.work.join(format!("{}.txt", title.len())), title).expect("write work file");
    fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write session file");
    let work_at = chid(&fx.work, "@");
    let agent_at = chid(&fx.bot, "@");
    let params = CommitParams {
        target: Target::Pair {
            pair: pair(fx),
            agent_now,
        },
        title: Some(title.to_string()),
        body: Some("The problem.\n\n* A facet.\n  - Its solution.\n".to_string()),
        closing: None,
        opening: None,
        advance: crate::commit::Advance::Off,
        push: Some(false),
        land: None,
        skip: None,
        dry_run: false,
    };
    commit(&mut test_ctx(), &params).expect("the pair is made");
    (work_at, agent_at)
}

/// A pair made at once: push folds the journal that landed after
/// the commit into the partner, keeps its change id, and publishes
/// both bookmarks.
#[test]
fn push_publishes_a_pair_made_at_once() {
    let fx = Fixture::new("push-pair-b");
    let (work_at, agent_at) = make_pair(&fx, "feat: at once", true);
    fs::write(fx.bot.join("tail.jsonl"), "{\"line\":2}\n").expect("write tail");

    push_in(&mut test_ctx(), &pair(&fx), &both(), &test_params()).expect("push");

    assert_eq!(chid(&fx.work, "main"), work_at);
    assert_eq!(cid(&fx.work, "main@origin"), cid(&fx.work, "main"));
    assert_eq!(desc_first_line(&fx.work, "main"), "feat: at once");
    assert_eq!(chid(&fx.bot, "main"), agent_at, "the partner keeps its id");
    assert_eq!(cid(&fx.bot, "main@origin"), cid(&fx.bot, "main"));
    let files = jj_ok(&fx.bot, &["file", "list", "-r", "main"]);
    assert!(files.contains("tail.jsonl"), "tail not folded:\n{files}");
    assert!(is_empty(&fx.work, "@") && is_empty(&fx.bot, "@"));
}

/// A pair made in two steps: push commits the journal as the
/// partner, with the work commit's description and a trailer naming
/// it, at the change id the work commit already names.
#[test]
fn push_completes_a_pair_made_in_two_steps() {
    let fx = Fixture::new("push-pair-w");
    let (work_at, agent_at) = make_pair(&fx, "feat: two steps", false);
    assert!(!is_empty(&fx.bot, "@"), "the journal is still in @");

    push_in(&mut test_ctx(), &pair(&fx), &both(), &test_params()).expect("push");

    assert_eq!(chid(&fx.work, "main"), work_at);
    assert_eq!(
        chid(&fx.bot, "main"),
        agent_at,
        "the journal became the named partner"
    );
    assert_eq!(desc_first_line(&fx.bot, "main"), "feat: two steps");
    let agent_desc = description(&fx.bot, "main");
    assert!(
        agent_desc
            .trim_end()
            .ends_with(&format!("ochid: /{work_at}")),
        "{agent_desc}"
    );
    assert_eq!(agent_desc.matches("ochid:").count(), 1, "{agent_desc}");
    assert!(
        agent_desc.contains("The problem.\n\n* A facet."),
        "{agent_desc}"
    );
    let work_desc = description(&fx.work, "main");
    assert!(
        work_desc
            .trim_end()
            .ends_with(&format!("ochid: /.claude/{agent_at}")),
        "{work_desc}"
    );
    assert_eq!(cid(&fx.bot, "main@origin"), cid(&fx.bot, "main"));

    let check = crate::validate_desc::ValidateDescParams {
        pos_rev: None,
        pos_count: None,
        revision: "main".to_string(),
        limit: None,
        repos: vec![
            ("work".to_string(), fx.work.clone()),
            ("agent".to_string(), fx.bot.clone()),
        ],
        other_repo: None,
        id_len: crate::desc_helpers::DEFAULT_ID_LEN,
    };
    crate::validate_desc::validate_desc(&test_ctx(), &check).expect("both directions valid");
}

/// New work in the work `@` is the next rung's: push publishes the
/// pair at `@-` and leaves `@` as it is, its change, its file, and its
/// place above the commit, through a restamp that rewrites `@-`.
#[test]
fn push_leaves_new_work_in_at() {
    let fx = Fixture::new("push-new-work");
    let (work_at, _) = make_pair(&fx, "feat: then more", false);
    jj_ok(&fx.bot, &["commit", "-m", "an unrelated agent commit"]);
    fs::write(fx.work.join("more.txt"), "the next rung").expect("write");
    let next = chid(&fx.work, "@");

    push_in(&mut test_ctx(), &pair(&fx), &both(), &test_params()).expect("push");

    assert_eq!(chid(&fx.work, "main"), work_at);
    assert_eq!(cid(&fx.work, "main@origin"), cid(&fx.work, "main"));
    assert_eq!(chid(&fx.work, "@"), next, "@ keeps its change");
    assert_eq!(chid(&fx.work, "@-"), work_at, "@ stays above the commit");
    let files = jj_ok(&fx.work, &["diff", "--name-only", "-r", "@"]);
    assert!(
        files.lines().count() == 1 && files.ends_with("more.txt"),
        "@ keeps its work:\n{files}"
    );
    let pushed = jj_ok(&fx.work, &["file", "list", "-r", "main"]);
    assert!(!pushed.contains("more.txt"), "@ went out:\n{pushed}");
}

/// A work `@-` with no trailer is not a pair's commit, and the error
/// names the command that makes one.
#[test]
fn push_refuses_an_unpaired_commit() {
    let fx = Fixture::new("push-unpaired");
    fs::write(fx.work.join("hand.txt"), "hand-made").expect("write");
    jj_ok(&fx.work, &["commit", "-m", "feat: by hand"]);
    let err = push_in(&mut test_ctx(), &pair(&fx), &both(), &test_params())
        .expect_err("refused")
        .to_string();
    assert!(err.contains("no `ochid:` trailer"), "{err}");
    assert!(err.contains("vc-x1 commit"), "{err}");
}

/// The agent repo changed between `commit -s w` and the push, so the
/// change the work commit names is gone: push restamps the
/// unpublished work commit with the agent `@`'s id, says so, and
/// commits the journal as the partner.
#[test]
fn push_restamps_when_the_agent_at_changed() {
    let fx = Fixture::new("push-restamp");
    let (work_at, stamped) = make_pair(&fx, "feat: restamped", false);
    // A commit in the agent repo by hand: @ becomes a new change, and
    // @- is the old one under another description, so neither names
    // the work commit nor is the change it names.
    jj_ok(&fx.bot, &["commit", "-m", "an unrelated agent commit"]);
    fs::write(fx.bot.join("more.jsonl"), "{\"line\":3}\n").expect("write");
    let agent_at = chid(&fx.bot, "@");
    assert_ne!(agent_at, stamped);

    push_in(&mut test_ctx(), &pair(&fx), &both(), &test_params()).expect("push");

    let work_desc = description(&fx.work, "main");
    assert!(
        work_desc
            .trim_end()
            .ends_with(&format!("ochid: /.claude/{agent_at}")),
        "restamped:\n{work_desc}"
    );
    assert_eq!(
        chid(&fx.work, "main"),
        work_at,
        "a restamp keeps the change id"
    );
    assert_eq!(chid(&fx.bot, "main"), agent_at);
    assert!(
        description(&fx.bot, "main").contains(&format!("ochid: /{work_at}")),
        "the partner names the work commit"
    );
}

/// A feature-bookmark push pins the agent repo to `main`: the work
/// repo advances and pushes `feature`, while the agent repo advances
/// `main` and grows no `feature` bookmark.
#[test]
fn push_feature_bookmark_pins_bot_to_main() {
    let fx = Fixture::new("push-feature-pin");
    jj_ok(&fx.work, &["git", "push", "--named", "feature=@-"]);
    let (work_at, agent_at) = make_pair(&fx, "feat: on feature", true);
    let bot_main_before = cid(&fx.bot, "main");

    let mut params = test_params();
    params.bookmark = Some("feature".to_string());
    push_in(&mut test_ctx(), &pair(&fx), &both(), &params).expect("push");

    assert_eq!(chid(&fx.work, "feature"), work_at);
    assert_eq!(cid(&fx.work, "feature"), cid(&fx.work, "feature@origin"));
    assert_ne!(cid(&fx.bot, "main"), bot_main_before);
    assert_eq!(chid(&fx.bot, "main"), agent_at);
    let bot_bookmarks = jj_ok(&fx.bot, &["bookmark", "list"]);
    assert!(
        !bot_bookmarks.lines().any(|l| l.starts_with("feature:")),
        "the agent repo must not grow a 'feature' bookmark:\n{bot_bookmarks}"
    );
}

/// A named bookmark off the work `@-`'s line, on a commit already
/// made, is published where it is, a name and no pair, and one on the
/// working copy or the root is refused.
#[test]
fn push_publishes_a_named_bookmark_off_the_line_where_it_is() {
    let fx = Fixture::new("push-name");
    fs::write(fx.work.join("side.txt"), "side\n").expect("write");
    jj_ok(&fx.work, &["commit", "-m", "feat: a side commit"]);
    jj_ok(&fx.work, &["bookmark", "create", "side", "-r", "@-"]);
    let side = jj_ok(
        &fx.work,
        &["log", "-r", "side", "--no-graph", "-T", "commit_id"],
    );
    jj_ok(&fx.work, &["new", "main"]);
    let mut params = test_params();
    params.repo = Some(fx.work.clone());
    params.bookmark = Some("side".to_string());
    crate::push::push(&mut test_ctx(), &params).expect("the name is published");
    assert_eq!(
        crate::jj::remote_bookmark_targets(&fx.work, "origin")
            .expect("remote")
            .get("side"),
        Some(&side)
    );

    fs::write(fx.work.join("wip.txt"), "wip\n").expect("write");
    jj_ok(&fx.work, &["bookmark", "create", "wip", "-r", "@"]);
    params.bookmark = Some("wip".to_string());
    let err = crate::push::push(&mut test_ctx(), &params)
        .expect_err("the working copy")
        .to_string();
    assert!(err.contains("`vc-x1 commit`"), "{err}");
}

/// Two bookmarks on the line and none named is refused, and a named
/// bookmark that is not on the line is refused too.
#[test]
fn push_needs_the_bookmark_named_when_the_line_is_ambiguous() {
    let fx = Fixture::new("push-ambiguous");
    jj_ok(&fx.work, &["bookmark", "create", "second", "-r", "@-"]);
    make_pair(&fx, "feat: which line", true);
    let err = push_in(&mut test_ctx(), &pair(&fx), &both(), &test_params())
        .expect_err("ambiguous")
        .to_string();
    assert!(err.contains("several bookmarks"), "{err}");

    jj_ok(
        &fx.work,
        &["bookmark", "create", "elsewhere", "-r", "root()"],
    );
    let mut params = test_params();
    params.bookmark = Some("elsewhere".to_string());
    let err = push_in(&mut test_ctx(), &pair(&fx), &both(), &params)
        .expect_err("not on the line")
        .to_string();
    assert!(err.contains("not the nearest bookmark"), "{err}");
}

/// Build a `Run` carrying the ids a completed push would have.
fn completion_run(work_chid: String, agent_chid: String) -> Run {
    Run {
        work: Some(("main".to_string(), work_chid)),
        partner: Some(agent_chid),
        held: false,
    }
}

/// Happy path: bookmarks at the run's ids, work `@` clean.
#[test]
fn completion_sanity_pass() {
    let fx = Fixture::new("completion-pass");
    make_pair(&fx, "feat: pass", true);
    push_in(&mut test_ctx(), &pair(&fx), &both(), &test_params()).expect("push");
    let run = completion_run(chid(&fx.work, "main"), chid(&fx.bot, "main"));
    verify_completion(&pair(&fx), &run).expect("passes");
}

/// Each check fails on its own discrepancy: the work bookmark
/// elsewhere, a dirty work `@`, the agent bookmark elsewhere.
#[test]
fn completion_sanity_fails_on_each_discrepancy() {
    let fx = Fixture::new("completion-fail");
    make_pair(&fx, "feat: x", true);
    push_in(&mut test_ctx(), &pair(&fx), &both(), &test_params()).expect("push");
    let work_chid = chid(&fx.work, "main");
    let agent_chid = chid(&fx.bot, "main");

    let run = completion_run("zzzzzzzzzzzz".to_string(), agent_chid.clone());
    let msg = verify_completion(&pair(&fx), &run)
        .expect_err("work mismatch")
        .to_string();
    assert!(
        msg.contains("work bookmark") && msg.contains("zzzzzzzzzzzz"),
        "{msg}"
    );

    let run = completion_run(work_chid.clone(), "zzzzzzzzzzzz".to_string());
    let msg = verify_completion(&pair(&fx), &run)
        .expect_err("agent mismatch")
        .to_string();
    assert!(
        msg.contains("agent bookmark") && msg.contains("zzzzzzzzzzzz"),
        "{msg}"
    );

    fs::write(fx.work.join("dirty.txt"), "uncommitted").expect("write dirty");
    let run = completion_run(work_chid, agent_chid);
    let msg = verify_completion(&pair(&fx), &run)
        .expect_err("dirty")
        .to_string();
    assert!(msg.contains("uncommitted changes"), "{msg}");
}

/// Rerunning a completed push is safe and changes nothing: the plan
/// finds the pair published, the journal folds or is nothing, and
/// jj's push reports nothing to do.
#[test]
fn push_rerun_after_completion_is_a_noop() {
    let fx = Fixture::new("push-rerun");
    make_pair(&fx, "feat: once", false);
    push_in(&mut test_ctx(), &pair(&fx), &both(), &test_params()).expect("first push");
    let work_after = cid(&fx.work, "main");
    let bot_after = cid(&fx.bot, "main");

    push_in(&mut test_ctx(), &pair(&fx), &both(), &test_params()).expect("rerun");

    assert_eq!(cid(&fx.work, "main"), work_after);
    assert_eq!(cid(&fx.bot, "main"), bot_after);
    assert_eq!(desc_first_line(&fx.work, "main"), "feat: once");
}

/// The halves: `-s w` publishes the work commit and leaves the agent
/// repo alone, and `-s a` after it completes the agent half, the
/// by-hand remainder of a run that stopped after push-work.
#[test]
fn push_halves_publish_one_side_each() {
    let fx = Fixture::new("push-halves");
    let (work_at, agent_at) = make_pair(&fx, "feat: in halves", false);
    let bot_main_before = cid(&fx.bot, "main");

    push_in(
        &mut test_ctx(),
        &pair(&fx),
        &Scope(vec![Side::Work]),
        &test_params(),
    )
    .expect("work half");
    assert_eq!(chid(&fx.work, "main"), work_at);
    assert_eq!(cid(&fx.work, "main@origin"), cid(&fx.work, "main"));
    assert_eq!(
        cid(&fx.bot, "main"),
        bot_main_before,
        "the agent repo is untouched"
    );
    assert!(!is_empty(&fx.bot, "@"));

    push_in(
        &mut test_ctx(),
        &pair(&fx),
        &Scope(vec![Side::Bot]),
        &test_params(),
    )
    .expect("agent half");
    assert_eq!(chid(&fx.bot, "main"), agent_at);
    assert_eq!(cid(&fx.bot, "main@origin"), cid(&fx.bot, "main"));
}

/// The halves in the order the pair is completed: `-s a` after
/// `commit -s w` commits the journal as the partner and publishes
/// `main`, mirroring the commit before it, and `-s w` then publishes
/// the work commit, its trailer already naming the partner.
#[test]
fn push_agent_half_then_work_half() {
    let fx = Fixture::new("push-agent-first");
    let (work_at, agent_at) = make_pair(&fx, "feat: agent first", false);
    let work_main_before = cid(&fx.work, "main");
    let work_desc_before = description(&fx.work, "@-");

    push_in(
        &mut test_ctx(),
        &pair(&fx),
        &Scope(vec![Side::Bot]),
        &test_params(),
    )
    .expect("agent half");
    assert_eq!(chid(&fx.bot, "main"), agent_at);
    assert_eq!(cid(&fx.bot, "main@origin"), cid(&fx.bot, "main"));
    assert_eq!(
        cid(&fx.work, "main"),
        work_main_before,
        "the work repo is untouched"
    );

    push_in(
        &mut test_ctx(),
        &pair(&fx),
        &Scope(vec![Side::Work]),
        &test_params(),
    )
    .expect("work half");
    assert_eq!(chid(&fx.work, "main"), work_at);
    assert_eq!(cid(&fx.work, "main@origin"), cid(&fx.work, "main"));
    assert_eq!(
        description(&fx.work, "main"),
        work_desc_before,
        "no restamp: the trailer already names the partner"
    );
}

/// A dry run plans and prints, and moves nothing.
#[test]
fn push_dry_run_changes_nothing() {
    let fx = Fixture::new("push-dry-run");
    make_pair(&fx, "feat: dry", false);
    let work_main = cid(&fx.work, "main");
    let bot_main = cid(&fx.bot, "main");
    let mut params = test_params();
    params.dry_run = true;
    push_in(&mut test_ctx(), &pair(&fx), &both(), &params).expect("dry run");
    assert_eq!(cid(&fx.work, "main"), work_main);
    assert_eq!(cid(&fx.bot, "main"), bot_main);
    assert!(!is_empty(&fx.bot, "@"), "the journal is still in @");
}

/// `-s p` publishes a plain repo's `@-` on its bookmark, no trailer
/// asked for, and new work in `@` stays there.
#[test]
fn push_por_publishes_the_commit_on_its_bookmark() {
    let fx = FixturePor::new("push-por");
    fs::write(fx.work.join("plain.txt"), "plain").expect("write");
    let at = chid(&fx.work, "@");
    let params = CommitParams {
        target: Target::Por(fx.work.clone()),
        title: Some("feat: plain".to_string()),
        body: Some("A body.".to_string()),
        closing: None,
        opening: None,
        advance: crate::commit::Advance::Line,
        push: Some(false),
        land: None,
        skip: None,
        dry_run: false,
    };
    commit(&mut test_ctx(), &params).expect("plain commit");
    fs::write(fx.work.join("next.txt"), "the next rung").expect("write");

    let mut params = test_params();
    params.scope = RepoScope::Por;
    params.repo = Some(fx.work.clone());
    push(&mut test_ctx(), &params).expect("push -s p");
    assert_eq!(chid(&fx.work, "main"), at);
    assert_eq!(cid(&fx.work, "main@origin"), cid(&fx.work, "main"));
    assert!(!is_empty(&fx.work, "@"), "@ keeps its work");
}

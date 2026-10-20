//! CLI subprocess tests for `vc-x1 commit` and `vc-x1 push`.
//!
//! Each test drives the binary in a `CliFixture`, whose isolated
//! home holds the jj config every `vc-x1` and `jj` child reads, so a
//! test sets jj settings there and the user's config has no say.

mod common;

use std::fs;
use std::path::PathBuf;

use common::{CliFixture, cid, jj, run_err, run_ok};

/// A workspace under a fixture's own jj config with a 1KiB
/// `snapshot.max-new-file-size`, a work file in `@`, and the paths
/// of both repos.
fn small_limit_workspace(tag: &str) -> (CliFixture, PathBuf, PathBuf) {
    let fx = CliFixture::new(tag);
    run_ok(
        fx.cmd()
            .current_dir(&fx.base)
            .args(["init", "./tr"])
            .arg("--repo")
            .arg(format!("local={}", fx.base.display())),
    );
    // The limit, in the fixture's own jj config, after init so the
    // workspace is made under the default.
    let config = fx.home.join(".config/jj/config.toml");
    let mut text = fs::read_to_string(&config).expect("read fixture jj config");
    text.push_str("[snapshot]\nmax-new-file-size = \"1KiB\"\n");
    fs::write(&config, text).expect("write fixture jj config");
    let work = fx.path("tr");
    let agent = work.join(".agent-session");
    fs::write(work.join("hello.txt"), "hi\n").expect("write work file");
    (fx, work, agent)
}

/// The output of a failed run, both streams.
fn said(out: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// A new file over `snapshot.max-new-file-size` in either repo stops
/// `commit -s b` before either repo commits, naming the file. The
/// pair's commit once committed both repos without a refused session
/// transcript and said nothing (m-10-0 in the messages repo).
#[test]
fn cli_commit_stops_on_a_file_the_size_limit_refused() {
    for side in ["work", "agent"] {
        let (fx, work, agent) = small_limit_workspace("commit-too-large");
        let repo = if side == "work" { &work } else { &agent };
        fs::write(repo.join("transcript.jsonl"), vec![b'x'; 2048]).expect("write big file");
        let work_main = cid(&fx.home, &work, "main");
        let agent_main = cid(&fx.home, &agent, "main");

        let out = run_err(fx.cmd().current_dir(&work).args([
            "commit",
            "-s",
            "b",
            "--title",
            "feat: too large",
            "--body",
            "body",
        ]));
        let said = said(&out);
        assert!(
            said.contains("  transcript.jsonl: 2.0KiB"),
            "{side}: {said}"
        );
        assert_eq!(
            cid(&fx.home, &work, "main"),
            work_main,
            "{side}: work main moved"
        );
        assert_eq!(
            cid(&fx.home, &agent, "main"),
            agent_main,
            "{side}: agent main moved"
        );
        for r in [&work, &agent] {
            let ops = jj(
                &fx.home,
                r,
                &["op", "log", "--no-graph", "-T", "description ++ \"\\n\""],
            );
            assert!(
                !ops.contains("restore to operation"),
                "{side}: rolled back:\n{ops}"
            );
            // The newest op is init's own or a snapshot, never a
            // commit made by the refused run.
            let newest = ops.lines().next().unwrap_or(""); // OK: a log has a line
            assert!(
                !newest.starts_with("commit "),
                "{side}: a commit was made:\n{ops}"
            );
        }
        let files = jj(&fx.home, &work, &["file", "list", "-r", "@"]);
        assert!(
            files.contains("hello.txt"),
            "{side}: @ lost its change:\n{files}"
        );
    }
}

/// A refused file in the agent journal stops `push -s b` at its plan,
/// naming the file, before anything is mutated: the work half, made
/// by `commit -s w`, stays unpublished and no rollback runs.
#[test]
fn cli_push_stops_on_a_journal_file_the_size_limit_refused() {
    let (fx, work, agent) = small_limit_workspace("push-too-large");
    run_ok(fx.cmd().current_dir(&work).args([
        "commit",
        "-s",
        "w",
        "--title",
        "feat: the work half",
        "--body",
        "body",
    ]));
    fs::write(agent.join("transcript.jsonl"), vec![b'x'; 2048]).expect("write big file");
    let work_main = cid(&fx.home, &work, "main");
    let agent_main = cid(&fx.home, &agent, "main");

    let out = run_err(fx.cmd().current_dir(&work).args(["push", "-s", "b"]));
    let said = said(&out);
    assert!(said.contains("  transcript.jsonl: 2.0KiB"), "{said}");
    assert_eq!(cid(&fx.home, &work, "main"), work_main, "work main moved");
    assert_eq!(
        cid(&fx.home, &agent, "main"),
        agent_main,
        "agent main moved"
    );
    for r in [&work, &agent] {
        let ops = jj(
            &fx.home,
            r,
            &["op", "log", "--no-graph", "-T", "description ++ \"\\n\""],
        );
        assert!(!ops.contains("restore to operation"), "rolled back:\n{ops}");
    }
}

/// A bare `push` is refused: `-s` is required, and the retired
/// description flags are errors naming themselves.
#[test]
fn cli_push_requires_scope_and_refuses_the_retired_flags() {
    let fx = CliFixture::new("push-flags");
    let out = run_err(fx.cmd().current_dir(&fx.base).arg("push"));
    assert!(said(&out).contains("--scope"), "{}", said(&out));
    let out = run_err(
        fx.cmd()
            .current_dir(&fx.base)
            .args(["push", "-s", "b", "--title", "t"]),
    );
    assert!(said(&out).contains("--title"), "{}", said(&out));
}

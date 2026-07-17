//! CLI subprocess tests for `bot-session`: render a small fixture
//! transcript, exercise the reveal flags, and pin the tolerant
//! handling of a truncated final line.

mod common;

use common::{CliFixture, run_err, run_ok};

/// The fixture transcript: prompt, thinking + text + tool_use on
/// one message id, a tool result, system/meta/bookkeeping lines,
/// and a truncated final line (live-session shape).
const FIXTURE: &str = concat!(
    r#"{"type":"mode","mode":"default"}"#,
    "\n",
    r#"{"type":"user","timestamp":"2026-07-17T04:17:09.100Z","promptSource":"typed","message":{"content":"please fix the bug"}}"#,
    "\n",
    r#"{"type":"assistant","message":{"id":"m1","content":[{"type":"thinking","thinking":"secret reasoning"}]}}"#,
    "\n",
    r#"{"type":"assistant","message":{"id":"m1","content":[{"type":"text","text":"looking at it"}]}}"#,
    "\n",
    r#"{"type":"assistant","message":{"id":"m1","content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"cargo test"}}]}}"#,
    "\n",
    r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"all green"}]}}"#,
    "\n",
    r#"{"type":"system","subtype":"turn_duration","durationMs":1234}"#,
    "\n",
    r#"{"type":"user","isMeta":true,"message":{"content":"injected context"}}"#,
    "\n",
    r#"{"type":"assistant","message":{"id":"m2","content":[{"type":"te"#,
);

/// Write the fixture into the tempdir and return its path.
fn fixture_file(fx: &CliFixture) -> std::path::PathBuf {
    let path = fx.path("session.jsonl");
    std::fs::write(&path, FIXTURE).expect("write fixture");
    path
}

/// Default view: dialogue + tool one-liner shown; thinking, tool
/// result, and meta/system hidden; truncated last line warns on
/// stderr but exits 0; summary line reports the counts.
#[test]
fn cli_bot_session_default() {
    let fx = CliFixture::new("bot-session-default");
    let file = fixture_file(&fx);
    let out = run_ok(fx.cmd().arg("bot-session").arg(&file));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stdout.contains("please fix the bug"), "got: {stdout}");
    assert!(stdout.contains("looking at it"));
    assert!(stdout.contains("[tool] Bash: cargo test"));
    assert!(!stdout.contains("secret reasoning"));
    assert!(!stdout.contains("all green"));
    assert!(!stdout.contains("injected context"));
    assert!(stdout.contains("2 turns shown"), "got: {stdout}");
    assert!(stdout.contains("1 malformed lines"));
    assert!(
        stderr.contains("warn: bot-session: line 9:"),
        "got: {stderr}"
    );
}

/// Reveal flags surface the hidden content.
#[test]
fn cli_bot_session_reveal() {
    let fx = CliFixture::new("bot-session-reveal");
    let file = fixture_file(&fx);
    let out =
        run_ok(
            fx.cmd()
                .arg("bot-session")
                .arg(&file)
                .args(["--thinking", "--results", "--meta"]),
        );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("secret reasoning"), "got: {stdout}");
    assert!(stdout.contains("[result] all green"));
    assert!(stdout.contains("--- system turn_duration"));
    assert!(stdout.contains("injected context"));
}

/// --all is shorthand for every reveal flag.
#[test]
fn cli_bot_session_all() {
    let fx = CliFixture::new("bot-session-all");
    let file = fixture_file(&fx);
    let out = run_ok(fx.cmd().arg("bot-session").arg(&file).arg("--all"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("secret reasoning"), "got: {stdout}");
    assert!(stdout.contains("[result] all green"));
    assert!(stdout.contains("--- system turn_duration"));
    assert!(stdout.contains("injected context"));
}

/// --lines slices the rendered output with elision markers.
#[test]
fn cli_bot_session_lines() {
    let fx = CliFixture::new("bot-session-lines");
    let file = fixture_file(&fx);
    let out = run_ok(
        fx.cmd()
            .arg("bot-session")
            .arg(&file)
            .args(["--lines", "2"]),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("=== user 04:17:09 ==="), "got: {stdout}");
    assert!(stdout.contains("lines skipped)"), "got: {stdout}");
    assert!(!stdout.contains("[tool]"));
    assert!(
        stdout.contains("lines shown (--lines)"),
        "sliced summary, got: {stdout}"
    );
    assert!(stdout.contains("full render:"), "got: {stdout}");
    let out = run_ok(
        fx.cmd()
            .arg("bot-session")
            .arg(&file)
            .args(["--lines", "0"]),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("0 of "), "summary-only, got: {stdout}");
    assert!(!stdout.contains("==="), "no turns, got: {stdout}");
    let out = run_err(
        fx.cmd()
            .arg("bot-session")
            .arg(&file)
            .args(["--lines", "x"]),
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--lines"), "got: {stderr}");
}

/// A missing file is the one hard error.
#[test]
fn cli_bot_session_missing_file() {
    let fx = CliFixture::new("bot-session-missing");
    let out = run_err(fx.cmd().arg("bot-session").arg("no-such.jsonl"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("cannot read"), "got: {stderr}");
}

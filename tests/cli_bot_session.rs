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
    assert!(
        stdout.contains("=== user 2026-07-17 04:17:09Z ==="),
        "got: {stdout}"
    );
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

/// --none + --<item> composes a minimal view; --no-<item>
/// subtracts from the default.
#[test]
fn cli_bot_session_item_toggles() {
    let fx = CliFixture::new("bot-session-toggles");
    let file = fixture_file(&fx);
    let out = run_ok(
        fx.cmd()
            .arg("bot-session")
            .arg(&file)
            .args(["--none", "--user"]),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("please fix the bug"), "got: {stdout}");
    assert!(!stdout.contains("==="), "no headers, got: {stdout}");
    assert!(!stdout.contains("[tool]"));
    assert!(!stdout.contains("turns shown"), "no summary");
    let out = run_ok(
        fx.cmd()
            .arg("bot-session")
            .arg(&file)
            .args(["--no-tool", "--no-headers"]),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("looking at it"));
    assert!(!stdout.contains("[tool]"), "got: {stdout}");
    assert!(!stdout.contains("==="), "got: {stdout}");
    assert!(stdout.contains("turns shown"), "summary still on");
}

/// [bot-session].items in the user config replaces the built-in
/// default set; CLI flags still adjust it.
#[test]
fn cli_bot_session_config_items() {
    let fx = CliFixture::new("bot-session-config");
    let file = fixture_file(&fx);
    let cfg_dir = fx.home.join(".config").join("vc-x1");
    std::fs::create_dir_all(&cfg_dir).expect("mkdir config dir");
    std::fs::write(
        cfg_dir.join("config.toml"),
        "[bot-session]\nitems = \"user,summary\"\n",
    )
    .expect("write config");
    let out = run_ok(
        fx.cmd()
            .env("XDG_CONFIG_HOME", fx.home.join(".config"))
            .arg("bot-session")
            .arg(&file),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("please fix the bug"), "got: {stdout}");
    assert!(!stdout.contains("==="), "config drops headers");
    assert!(!stdout.contains("[tool]"));
    assert!(stdout.contains("turns shown"), "summary kept");
    let out = run_ok(
        fx.cmd()
            .env("XDG_CONFIG_HOME", fx.home.join(".config"))
            .arg("bot-session")
            .arg(&file)
            .arg("--tool"),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("[tool]"), "CLI adds over config");
}

/// Every --no-<item> flag parses and subtracts (--all minus all
/// eight = empty output); a --<item>/--no-<item> pair resolves
/// last-one-wins; the --no-all/--no-none aliases behave as
/// --none/--all.
#[test]
fn cli_bot_session_no_item_flags() {
    let fx = CliFixture::new("bot-session-no-flags");
    let file = fixture_file(&fx);
    let out = run_ok(fx.cmd().arg("bot-session").arg(&file).args([
        "--all",
        "--no-headers",
        "--no-user",
        "--no-assistant",
        "--no-tool",
        "--no-thinking",
        "--no-results",
        "--no-meta",
        "--no-summary",
    ]));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.trim().is_empty(), "got: {stdout}");

    let out = run_ok(fx.cmd().arg("bot-session").arg(&file).args([
        "--none",
        "--headers",
        "--user",
        "--assistant",
        "--tool",
        "--thinking",
        "--results",
        "--meta",
        "--summary",
    ]));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("secret reasoning"), "got: {stdout}");
    assert!(stdout.contains("[result] all green"));
    assert!(stdout.contains("--- system turn_duration"));
    assert!(stdout.contains("injected context"));
    assert!(stdout.contains("turns shown"));

    let out = run_ok(fx.cmd().arg("bot-session").arg(&file).args([
        "--no-summary",
        "--summary",
        "--none",
    ]));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("turns shown"),
        "last of the pair wins, got: {stdout}"
    );
    let out = run_ok(
        fx.cmd()
            .arg("bot-session")
            .arg(&file)
            .args(["--summary", "--no-summary"]),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains("turns shown"), "got: {stdout}");

    let out = run_ok(fx.cmd().arg("bot-session").arg(&file).arg("--no-all"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.trim().is_empty(), "--no-all = --none, got: {stdout}");
    let out = run_ok(fx.cmd().arg("bot-session").arg(&file).arg("--no-none"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("secret reasoning"),
        "--no-none = --all, got: {stdout}"
    );
}

/// The workspace .vc-config.toml [bot-session].items layer wins
/// over the user config; CLI still wins over both.
#[test]
fn cli_bot_session_workspace_items() {
    let fx = CliFixture::new("bot-session-vc-config");
    let file = fixture_file(&fx);
    let cfg_dir = fx.home.join(".config").join("vc-x1");
    std::fs::create_dir_all(&cfg_dir).expect("mkdir config dir");
    std::fs::write(
        cfg_dir.join("config.toml"),
        "[bot-session]\nitems = \"assistant\"\n",
    )
    .expect("write user config");
    std::fs::write(
        fx.base.join(".vc-config.toml"),
        "[workspace]\npath = \"/\"\n\n[bot-session]\nitems = \"user,summary\"\n",
    )
    .expect("write vc-config");
    let out = run_ok(
        fx.cmd()
            .env("XDG_CONFIG_HOME", fx.home.join(".config"))
            .current_dir(&fx.base)
            .arg("bot-session")
            .arg(&file),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("please fix the bug"), "got: {stdout}");
    assert!(
        !stdout.contains("looking at it"),
        "workspace items beat user config, got: {stdout}"
    );
    assert!(stdout.contains("turns shown"), "summary from workspace");
    let out = run_ok(
        fx.cmd()
            .env("XDG_CONFIG_HOME", fx.home.join(".config"))
            .current_dir(&fx.base)
            .arg("bot-session")
            .arg(&file)
            .arg("--assistant"),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("looking at it"), "CLI beats workspace");
}

/// --result-lines adjusts the [result] body cap; 0 = unlimited.
#[test]
fn cli_bot_session_result_lines() {
    let fx = CliFixture::new("bot-session-result-lines");
    let file = fx.path("multiline.jsonl");
    let result = (1..=6)
        .map(|i| format!("r{i}"))
        .collect::<Vec<_>>()
        .join("\\n");
    let tool_use = r#"{"type":"assistant","message":{"id":"m1","content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"x"}}]}}"#;
    let tool_result = format!(
        r#"{{"type":"user","message":{{"content":[{{"type":"tool_result","tool_use_id":"t1","content":"{result}"}}]}}}}"#
    );
    std::fs::write(&file, format!("{tool_use}\n{tool_result}\n")).expect("write fixture");
    let out =
        run_ok(
            fx.cmd()
                .arg("bot-session")
                .arg(&file)
                .args(["--results", "--result-lines", "2"]),
        );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("[result] r1"), "got: {stdout}");
    assert!(stdout.contains("r2"));
    assert!(!stdout.contains("r3"), "capped at 2, got: {stdout}");
    assert!(stdout.contains("(+4 lines)"));
    let out =
        run_ok(
            fx.cmd()
                .arg("bot-session")
                .arg(&file)
                .args(["--results", "--result-lines", "0"]),
        );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("r6"), "unlimited, got: {stdout}");
    assert!(!stdout.contains("lines)"), "no marker, got: {stdout}");
}

/// A missing file is the one hard error.
#[test]
fn cli_bot_session_missing_file() {
    let fx = CliFixture::new("bot-session-missing");
    let out = run_err(fx.cmd().arg("bot-session").arg("no-such.jsonl"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("cannot read"), "got: {stderr}");
}

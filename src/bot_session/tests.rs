//! Unit tests for the bot-session renderer — synthetic
//! transcripts built via `transcript::parse_str`, asserting
//! header collapsing, hide/reveal behavior, gists, and the
//! summary line.

use super::*;
use crate::transcript::parse_str;

/// A small conversation: prompt, thinking, text + tool_use on one
/// message id, a tool result, and a bookkeeping line.
fn sample() -> FileTranscript {
    parse_str(concat!(
        r#"{"type":"user","timestamp":"2026-07-17T04:17:09.100Z","promptSource":"typed","message":{"content":"do the thing"}}"#,
        "\n",
        r#"{"type":"assistant","timestamp":"2026-07-17T04:17:15.000Z","message":{"id":"m1","content":[{"type":"thinking","thinking":"secret plan"}]}}"#,
        "\n",
        r#"{"type":"assistant","timestamp":"2026-07-17T04:17:16.000Z","message":{"id":"m1","content":[{"type":"text","text":"on it"}]}}"#,
        "\n",
        r#"{"type":"assistant","timestamp":"2026-07-17T04:17:17.000Z","message":{"id":"m1","content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"cargo test\n# more"}}]}}"#,
        "\n",
        r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"ok: 5 passed"}]}}"#,
        "\n",
        r#"{"type":"system","subtype":"turn_duration","timestamp":"2026-07-17T04:17:20.000Z"}"#,
        "\n",
        r#"{"type":"file-history-snapshot"}"#,
    ))
}

/// Default view: one user + one assistant turn (blocks collapse
/// under one message id); thinking/result/system/bookkeeping
/// hidden and counted.
#[test]
fn default_view() {
    let (lines, stats) = render(&sample(), &RenderOptions::default());
    let out = lines.join("\n");
    assert!(out.contains("=== user 04:17:09 ==="), "got:\n{out}");
    assert!(out.contains("do the thing"));
    assert_eq!(out.matches("=== assistant").count(), 1);
    assert!(out.contains("on it"));
    assert!(out.contains("  [tool] Bash: cargo test"));
    assert!(!out.contains("secret plan"));
    assert!(!out.contains("ok: 5 passed"));
    assert!(!out.contains("system"));
    assert_eq!(stats.shown, 2);
    assert_eq!(stats.hidden_thinking, 1);
    assert_eq!(stats.hidden_tool_results, 1);
    assert_eq!(stats.hidden_meta, 1);
    assert_eq!(stats.skipped_other, 1);
}

/// Reveal flags surface thinking, results, and system lines.
#[test]
fn reveal_flags() {
    let opts = RenderOptions {
        thinking: true,
        results: true,
        meta: true,
    };
    let (lines, stats) = render(&sample(), &opts);
    let out = lines.join("\n");
    assert!(out.contains("  [thinking]"));
    assert!(out.contains("  secret plan"));
    assert!(out.contains("  [result] ok: 5 passed"));
    assert!(out.contains("--- system turn_duration 04:17:20 ---"));
    assert_eq!(stats.hidden_thinking, 0);
    assert_eq!(stats.hidden_tool_results, 0);
    assert_eq!(stats.hidden_meta, 0);
}

/// Meta user lines hide by default; sidechain entries too.
#[test]
fn meta_and_sidechain_hidden() {
    let t = parse_str(concat!(
        r#"{"type":"user","isMeta":true,"message":{"content":"injected"}}"#,
        "\n",
        r#"{"type":"assistant","isSidechain":true,"message":{"id":"m9","content":[{"type":"text","text":"sub work"}]}}"#,
    ));
    let (lines, stats) = render(&t, &RenderOptions::default());
    assert!(lines.is_empty());
    assert_eq!(stats.hidden_meta, 2);
    let (lines, _) = render(
        &t,
        &RenderOptions {
            meta: true,
            ..Default::default()
        },
    );
    let out = lines.join("\n");
    assert!(out.contains("=== user (meta)"));
    assert!(out.contains("sub work"));
}

/// A long tool result is capped with a "+N lines" tail.
#[test]
fn result_cap() {
    let body = (1..=15).map(|i| format!("l{i}")).collect::<Vec<_>>();
    let mut lines = Vec::new();
    push_result(&mut lines, &body.join("\n"), false);
    assert_eq!(lines.len(), RESULT_LINE_CAP + 1);
    assert_eq!(lines[0], "  [result] l1");
    assert!(lines[RESULT_LINE_CAP].contains("(+5 lines)"));
    let mut err_lines = Vec::new();
    push_result(&mut err_lines, "boom", true);
    assert_eq!(err_lines[0], "  [result:error] boom");
}

/// Gists: Bash first line, Read file_path, fallback pairs,
/// truncation.
#[test]
fn gists() {
    let bash = serde_json::json!({"command": "cargo test\nsecond"});
    assert_eq!(tool_use_gist("Bash", &bash), "Bash: cargo test");
    let read = serde_json::json!({"file_path": "/a/b.rs", "limit": 5});
    assert_eq!(tool_use_gist("Read", &read), "Read: /a/b.rs");
    let other = serde_json::json!({"pattern": "foo", "path": "src"});
    let gist = tool_use_gist("Grep", &other);
    assert!(gist.contains("pattern=foo") && gist.contains("path=src"));
    let long = serde_json::json!({"x": "y".repeat(200)});
    assert!(tool_use_gist("T", &long).chars().count() <= GIST_CHAR_CAP + 1);
}

/// short_time slices ISO timestamps and degrades gracefully.
#[test]
fn short_time_cases() {
    assert_eq!(short_time(Some("2026-07-17T04:17:09.100Z")), "04:17:09");
    assert_eq!(short_time(Some("short")), "short");
    assert_eq!(short_time(None), "");
}

/// --lines spec parsing: the four valid shapes and the rejects.
#[test]
fn lines_spec_parsing() {
    assert!(matches!(parse_lines_spec("5"), Ok(LinesSpec::Single(5))));
    assert!(matches!(parse_lines_spec("-5"), Ok(LinesSpec::Single(-5))));
    assert!(matches!(
        parse_lines_spec("10,3"),
        Ok(LinesSpec::Pair(10, 3))
    ));
    assert!(matches!(
        parse_lines_spec("10,-3"),
        Ok(LinesSpec::Pair(10, -3))
    ));
    assert!(matches!(parse_lines_spec("0"), Ok(LinesSpec::Single(0))));
    assert!(matches!(parse_lines_spec("5,0"), Ok(LinesSpec::Pair(5, 0))));
    assert!(parse_lines_spec("-1,5").is_err());
    assert!(parse_lines_spec("x").is_err());
    assert!(parse_lines_spec("1,2,3").is_err());
}

/// --lines slicing: head, tail, anchored ranges, clamping, and
/// elision markers.
#[test]
fn lines_slicing() {
    let ls: Vec<String> = (0..10).map(|i| format!("l{i}")).collect();
    let head = apply_lines(ls.clone(), &LinesSpec::Single(3));
    assert_eq!(head, vec!["l0", "l1", "l2", "… (7 lines skipped)"]);
    let tail = apply_lines(ls.clone(), &LinesSpec::Single(-2));
    assert_eq!(tail, vec!["… (8 lines skipped)", "l8", "l9"]);
    let mid = apply_lines(ls.clone(), &LinesSpec::Pair(4, 2));
    assert_eq!(
        mid,
        vec!["… (4 lines skipped)", "l4", "l5", "… (4 lines skipped)"]
    );
    let back = apply_lines(ls.clone(), &LinesSpec::Pair(4, -2));
    assert_eq!(
        back,
        vec!["… (2 lines skipped)", "l2", "l3", "… (6 lines skipped)"]
    );
    let clamped = apply_lines(ls.clone(), &LinesSpec::Pair(8, 5));
    assert_eq!(clamped, vec!["… (8 lines skipped)", "l8", "l9"]);
    let over = apply_lines(ls.clone(), &LinesSpec::Single(99));
    assert_eq!(over.len(), 10);
    let back_clamped = apply_lines(ls.clone(), &LinesSpec::Pair(1, -5));
    assert_eq!(back_clamped, vec!["l0", "… (9 lines skipped)"]);
    let none = apply_lines(ls.clone(), &LinesSpec::Single(0));
    assert_eq!(none, vec!["… (10 lines skipped)"]);
    let none_at = apply_lines(ls, &LinesSpec::Pair(4, 0));
    assert_eq!(none_at, vec!["… (4 lines skipped)", "… (6 lines skipped)"]);
}

/// Summary line includes only non-zero clauses.
#[test]
fn summary_lines() {
    let stats = RenderStats {
        shown: 3,
        ..Default::default()
    };
    assert_eq!(summary_line(&stats, 0, None), "bot-session: 3 turns shown");
    assert_eq!(
        summary_line(&stats, 0, Some((15, 1093))),
        "bot-session: 15 of 1093 lines shown (--lines); \
         full render: 3 turns"
    );
    let stats = RenderStats {
        shown: 2,
        hidden_thinking: 4,
        hidden_tool_results: 1,
        hidden_meta: 2,
        skipped_other: 7,
    };
    assert_eq!(
        summary_line(&stats, 1, None),
        "bot-session: 2 turns shown; hidden: 4 thinking, 1 tool results, \
         2 meta/system; skipped: 7 bookkeeping entries; 1 malformed lines"
    );
}

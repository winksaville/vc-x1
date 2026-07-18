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
    let (lines, stats) = render(
        &sample(),
        &ItemSet::BUILTIN,
        RESULT_LINE_CAP,
        0,
        usize::MAX,
        usize::MAX,
    );
    let out = lines.join("\n");
    assert!(
        out.contains("=== user 2026-07-17 04:17:09Z ==="),
        "got:\n{out}"
    );
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
    let (lines, stats) = render(
        &sample(),
        &ItemSet::ALL,
        RESULT_LINE_CAP,
        0,
        usize::MAX,
        usize::MAX,
    );
    let out = lines.join("\n");
    assert!(out.contains("  [thinking]"));
    assert!(out.contains("  secret plan"));
    assert!(out.contains("  [result] ok: 5 passed"));
    assert!(out.contains("--- system turn_duration 2026-07-17 04:17:20Z ---"));
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
    let (lines, stats) = render(
        &t,
        &ItemSet::BUILTIN,
        RESULT_LINE_CAP,
        0,
        usize::MAX,
        usize::MAX,
    );
    assert!(lines.is_empty());
    assert_eq!(stats.hidden_meta, 2);
    let (lines, _) = render(
        &t,
        &ItemSet {
            meta: true,
            ..ItemSet::BUILTIN
        },
        RESULT_LINE_CAP,
        0,
        usize::MAX,
        usize::MAX,
    );
    let out = lines.join("\n");
    assert!(out.contains("=== user (meta)"));
    assert!(out.contains("sub work"));
}

/// A long tool result is capped with a "+N lines" tail; the cap
/// is adjustable and 0 means unlimited.
#[test]
fn result_cap() {
    let body = (1..=15).map(|i| format!("l{i}")).collect::<Vec<_>>();
    let mut lines = Vec::new();
    push_result(&mut lines, &body.join("\n"), false, RESULT_LINE_CAP);
    assert_eq!(lines.len(), RESULT_LINE_CAP + 1);
    assert_eq!(lines[0], "  [result] l1");
    assert!(lines[RESULT_LINE_CAP].contains("(+5 lines)"));
    let mut err_lines = Vec::new();
    push_result(&mut err_lines, "boom", true, RESULT_LINE_CAP);
    assert_eq!(err_lines[0], "  [result:error] boom");

    let mut two = Vec::new();
    push_result(&mut two, &body.join("\n"), false, 2);
    assert_eq!(two.len(), 3, "first + 1 + tail marker");
    assert!(two[2].contains("(+13 lines)"));

    let mut unlimited = Vec::new();
    push_result(&mut unlimited, &body.join("\n"), false, 0);
    assert_eq!(unlimited.len(), 15, "cap 0 = every line, no marker");
    assert!(!unlimited.last().unwrap().contains("lines)"));
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
    assert_eq!(
        short_time(Some("2026-07-17T04:17:09.100Z")),
        "2026-07-17 04:17:09Z"
    );
    assert_eq!(short_time(Some("short")), "short");
    assert_eq!(
        short_time(Some("2026-07-17T04:17:09+02:00")),
        "2026-07-17T04:17:09+02:00",
        "non-Z passes through verbatim, never relabeled"
    );
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

/// line_bounds: head, tail, anchored ranges, clamping, zero.
#[test]
fn line_bounds_cases() {
    assert_eq!(line_bounds(&LinesSpec::Single(3), 10), (0, 3));
    assert_eq!(line_bounds(&LinesSpec::Single(-2), 10), (8, 10));
    assert_eq!(line_bounds(&LinesSpec::Pair(4, 2), 10), (4, 6));
    assert_eq!(line_bounds(&LinesSpec::Pair(4, -2), 10), (2, 4));
    assert_eq!(line_bounds(&LinesSpec::Pair(8, 5), 10), (8, 10));
    assert_eq!(line_bounds(&LinesSpec::Single(99), 10), (0, 10));
    assert_eq!(line_bounds(&LinesSpec::Pair(1, -5), 10), (0, 1));
    assert_eq!(line_bounds(&LinesSpec::Single(0), 10), (10, 10));
    assert_eq!(line_bounds(&LinesSpec::Pair(4, 0), 10), (4, 4));
}

/// A sliced render keeps only in-range entries and marks the
/// skipped source regions.
#[test]
fn render_source_slice() {
    // sample() lines: 1 prompt, 2 thinking, 3 text, 4 tool_use,
    // 5 tool_result, 6 system, 7 bookkeeping.
    let (lines, stats) = render(&sample(), &ItemSet::BUILTIN, RESULT_LINE_CAP, 2, 4, 7);
    let out = lines.join("\n");
    assert!(out.starts_with("… (2 source lines skipped)"), "got:\n{out}");
    assert!(out.ends_with("… (3 source lines skipped)"), "got:\n{out}");
    assert!(out.contains("on it"), "line 3 text in range");
    assert!(out.contains("[tool]"), "line 4 tool_use in range");
    assert!(!out.contains("do the thing"), "line 1 out of range");
    assert_eq!(stats.shown, 1, "one assistant turn in slice");
}

/// Item gating: headers off drops === lines but keeps blank
/// separators; user-only shows just the prompt; tool off drops
/// [tool] lines.
#[test]
fn item_gating() {
    let no_headers = ItemSet {
        headers: false,
        ..ItemSet::BUILTIN
    };
    let (lines, stats) = render(
        &sample(),
        &no_headers,
        RESULT_LINE_CAP,
        0,
        usize::MAX,
        usize::MAX,
    );
    let out = lines.join("\n");
    assert!(!out.contains("==="), "got:\n{out}");
    assert!(out.contains("do the thing"));
    assert_eq!(stats.shown, 2, "turns still counted");

    let user_only = ItemSet {
        user: true,
        ..ItemSet::NONE
    };
    let (lines, _) = render(
        &sample(),
        &user_only,
        RESULT_LINE_CAP,
        0,
        usize::MAX,
        usize::MAX,
    );
    assert_eq!(lines, vec!["do the thing"]);

    let no_tool = ItemSet {
        tool: false,
        ..ItemSet::BUILTIN
    };
    let (lines, _) = render(
        &sample(),
        &no_tool,
        RESULT_LINE_CAP,
        0,
        usize::MAX,
        usize::MAX,
    );
    let out = lines.join("\n");
    assert!(!out.contains("[tool]"), "got:\n{out}");
    assert!(out.contains("on it"));
}

/// resolve_items: bases (builtin / config / --all / --none) and
/// per-item overrides.
#[test]
fn resolve_items_cases() {
    let t = ItemToggles::default();
    assert_eq!(resolve_items(&t, None).unwrap(), ItemSet::BUILTIN);
    assert_eq!(
        resolve_items(&t, Some("user,summary")).unwrap(),
        ItemSet {
            user: true,
            summary: true,
            ..ItemSet::NONE
        }
    );
    let t = ItemToggles {
        all: true,
        thinking: Some(false),
        ..Default::default()
    };
    assert_eq!(
        resolve_items(&t, None).unwrap(),
        ItemSet {
            thinking: false,
            ..ItemSet::ALL
        }
    );
    let t = ItemToggles {
        none: true,
        user: Some(true),
        ..Default::default()
    };
    assert_eq!(
        resolve_items(&t, None).unwrap(),
        ItemSet {
            user: true,
            ..ItemSet::NONE
        }
    );
    // CLI toggle beats the config base.
    let t = ItemToggles {
        tool: Some(true),
        ..Default::default()
    };
    assert_eq!(
        resolve_items(&t, Some("user")).unwrap(),
        ItemSet {
            user: true,
            tool: true,
            ..ItemSet::NONE
        }
    );
    // Unknown config item errors.
    let err = resolve_items(&ItemToggles::default(), Some("user,bogus")).unwrap_err();
    assert!(err.contains("bogus"), "got: {err}");
    // Empty list is allowed (start from nothing).
    assert_eq!(
        resolve_items(&ItemToggles::default(), Some("")).unwrap(),
        ItemSet::NONE
    );
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
        "bot-session: 3 turns shown; \
         --lines selected 15 of 1093 source lines"
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

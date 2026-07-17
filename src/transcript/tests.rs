//! Unit tests for transcript parsing — inline JSON fixtures
//! covering the observed line shapes and the tolerant-parse
//! guarantees (unknown types, missing fields, truncated lines).

use super::*;

/// A typed user prompt: string content, promptSource, meta fields.
#[test]
fn user_typed_prompt() {
    let t = parse_str(
        r#"{"type":"user","uuid":"u1","parentUuid":"p0","timestamp":"2026-07-17T04:17:09.100Z","sessionId":"s1","cwd":"/w","promptSource":"typed","message":{"role":"user","content":"hello there"}}"#,
    );
    assert_eq!(t.entries.len(), 1);
    assert!(t.malformed.is_empty());
    let e = &t.entries[0];
    assert_eq!(e.line_no, 1);
    assert_eq!(e.meta.uuid.as_deref(), Some("u1"));
    assert_eq!(e.meta.parent_uuid.as_deref(), Some("p0"));
    assert_eq!(
        e.meta.timestamp.as_deref(),
        Some("2026-07-17T04:17:09.100Z")
    );
    assert_eq!(e.meta.session_id.as_deref(), Some("s1"));
    assert_eq!(e.meta.cwd.as_deref(), Some("/w"));
    assert!(!e.meta.is_meta);
    assert!(!e.meta.is_sidechain);
    match &e.kind {
        EntryKind::User {
            content,
            prompt_source,
        } => {
            assert_eq!(prompt_source.as_deref(), Some("typed"));
            match &content[..] {
                [ContentBlock::Text { text }] => assert_eq!(text, "hello there"),
                _ => panic!("expected one Text block"),
            }
        }
        _ => panic!("expected User"),
    }
}

/// Assistant lines: text, thinking, and tool_use blocks.
#[test]
fn assistant_block_kinds() {
    let t = parse_str(concat!(
        r#"{"type":"assistant","message":{"id":"m1","content":[{"type":"text","text":"hi"}]}}"#,
        "\n",
        r#"{"type":"assistant","message":{"id":"m1","content":[{"type":"thinking","thinking":"hmm"}]}}"#,
        "\n",
        r#"{"type":"assistant","message":{"id":"m1","content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"ls"}}]}}"#,
    ));
    assert_eq!(t.entries.len(), 3);
    let blocks: Vec<&ContentBlock> = t
        .entries
        .iter()
        .map(|e| match &e.kind {
            EntryKind::Assistant {
                content,
                message_id,
            } => {
                assert_eq!(message_id.as_deref(), Some("m1"));
                &content[0]
            }
            _ => panic!("expected Assistant"),
        })
        .collect();
    match blocks[0] {
        ContentBlock::Text { text } => assert_eq!(text, "hi"),
        _ => panic!("expected Text"),
    }
    match blocks[1] {
        ContentBlock::Thinking { thinking } => assert_eq!(thinking, "hmm"),
        _ => panic!("expected Thinking"),
    }
    match blocks[2] {
        ContentBlock::ToolUse { id, name, input } => {
            assert_eq!(id, "t1");
            assert_eq!(name, "Bash");
            assert_eq!(input["command"], "ls");
        }
        _ => panic!("expected ToolUse"),
    }
}

/// tool_result content: string form and array-of-text form.
#[test]
fn tool_result_shapes() {
    let t = parse_str(concat!(
        r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"plain out"}]}}"#,
        "\n",
        r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t2","is_error":true,"content":[{"type":"text","text":"a"},{"type":"text","text":"b"}]}]}}"#,
    ));
    let results: Vec<&ContentBlock> = t
        .entries
        .iter()
        .map(|e| match &e.kind {
            EntryKind::User { content, .. } => &content[0],
            _ => panic!("expected User"),
        })
        .collect();
    match results[0] {
        ContentBlock::ToolResult {
            tool_use_id,
            text,
            is_error,
        } => {
            assert_eq!(tool_use_id, "t1");
            assert_eq!(text, "plain out");
            assert!(!is_error);
        }
        _ => panic!("expected ToolResult"),
    }
    match results[1] {
        ContentBlock::ToolResult { text, is_error, .. } => {
            assert_eq!(text, "a\nb");
            assert!(*is_error);
        }
        _ => panic!("expected ToolResult"),
    }
}

/// isMeta / isSidechain flags land in EntryMeta.
#[test]
fn meta_and_sidechain_flags() {
    let t = parse_str(concat!(
        r#"{"type":"user","isMeta":true,"message":{"content":"injected"}}"#,
        "\n",
        r#"{"type":"assistant","isSidechain":true,"message":{"content":[]}}"#,
    ));
    assert!(t.entries[0].meta.is_meta);
    assert!(!t.entries[0].meta.is_sidechain);
    assert!(t.entries[1].meta.is_sidechain);
}

/// system subtype, unknown types, and a missing type field.
#[test]
fn system_unknown_and_missing_types() {
    let t = parse_str(concat!(
        r#"{"type":"system","subtype":"turn_duration"}"#,
        "\n",
        r#"{"type":"file-history-snapshot","snapshot":{}}"#,
        "\n",
        r#"{"no_type":true}"#,
    ));
    match &t.entries[0].kind {
        EntryKind::System { subtype } => assert_eq!(subtype.as_deref(), Some("turn_duration")),
        _ => panic!("expected System"),
    }
    match &t.entries[1].kind {
        EntryKind::Other { entry_type } => assert_eq!(entry_type, "file-history-snapshot"),
        _ => panic!("expected Other"),
    }
    match &t.entries[2].kind {
        EntryKind::Other { entry_type } => assert_eq!(entry_type, "<none>"),
        _ => panic!("expected Other"),
    }
}

/// Unknown content-block types are kept as Other, not dropped.
#[test]
fn unknown_block_type() {
    let t = parse_str(
        r#"{"type":"assistant","message":{"content":[{"type":"server_tool_use","x":1}]}}"#,
    );
    match &t.entries[0].kind {
        EntryKind::Assistant { content, .. } => match &content[0] {
            ContentBlock::Other { block_type } => assert_eq!(block_type, "server_tool_use"),
            _ => panic!("expected Other block"),
        },
        _ => panic!("expected Assistant"),
    }
}

/// Blank lines skip; a truncated last line records as malformed
/// with its 1-based line number; parsing continues overall.
#[test]
fn blank_and_truncated_lines() {
    let t = parse_str(concat!(
        r#"{"type":"user","message":{"content":"ok"}}"#,
        "\n\n",
        r#"{"type":"assistant","message":{"content":[{"ty"#,
    ));
    assert_eq!(t.entries.len(), 1);
    assert_eq!(t.malformed.len(), 1);
    assert_eq!(t.malformed[0].0, 3);
}

/// Non-object top level (valid JSON, wrong shape) is malformed.
#[test]
fn non_object_line() {
    let t = parse_str("[1,2,3]");
    assert!(t.entries.is_empty());
    assert_eq!(t.malformed.len(), 1);
}

/// raw retains fields the typed layer doesn't extract.
#[test]
fn raw_retains_unknown_fields() {
    let t = parse_str(r#"{"type":"user","gitBranch":"main","message":{"content":"x"}}"#);
    assert_eq!(t.entries[0].raw["gitBranch"], "main");
}

/// session_id falls back from sessionId to session_id.
#[test]
fn session_id_fallback() {
    let t = parse_str(concat!(
        r#"{"type":"user","sessionId":"a","session_id":"b","message":{"content":"x"}}"#,
        "\n",
        r#"{"type":"user","session_id":"b","message":{"content":"x"}}"#,
    ));
    assert_eq!(t.entries[0].meta.session_id.as_deref(), Some("a"));
    assert_eq!(t.entries[1].meta.session_id.as_deref(), Some("b"));
}

/// parse_file: missing file is the one hard error.
#[test]
fn parse_file_missing() {
    let err = parse_file(std::path::Path::new("/nonexistent/x.jsonl"))
        .err()
        .map(|e| e.to_string())
        .unwrap_or_default();
    assert!(err.contains("cannot read"), "got: {err}");
}

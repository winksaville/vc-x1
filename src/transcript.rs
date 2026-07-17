//! Claude Code session transcript (`.jsonl`) parsing.
//!
//! Two-layer design so schema churn in the (undocumented,
//! evolving) transcript format never breaks us:
//!
//! - serde_json is used only as a JSON-text → `Value` parser; the
//!   full parsed line is retained in `Entry::raw`, so unknown
//!   fields ride along.
//! - Hand-written extraction builds the typed layer (`EntryMeta`,
//!   `EntryKind`, `ContentBlock`) — every field degrades to
//!   `None`/`false`/empty when absent, and unrecognized entry or
//!   block types land in `Other` variants instead of erroring.
//!
//! Scope and footprint, by design:
//!
//! - `FileTranscript` is one file's parse. A *session* can span
//!   several files (sidechain `agent-*.jsonl`, compaction
//!   predecessors); assembling those is a later, separate layer.
//! - Whole-file in memory: the largest observed file (~8 MB)
//!   parses to tens of MB — fine interactively. If that stops
//!   holding, streaming via `BufRead` is a drop-in change inside
//!   `parse_file`, and `raw` retention could become optional.

use serde_json::Value;
use std::path::Path;

/// One file's parsed transcript: every parseable line, in file order.
pub struct FileTranscript {
    /// All entries, including ones a conversation view hides.
    pub entries: Vec<Entry>,
    /// Malformed lines: (1-based line number, parse error text).
    /// A live session's truncated final line lands here.
    pub malformed: Vec<(usize, String)>,
}

/// One parsed JSONL line.
pub struct Entry {
    /// 1-based line number in the source file.
    pub line_no: usize,
    /// Common metadata shared across entry types.
    pub meta: EntryMeta,
    /// Classified payload.
    pub kind: EntryKind,
    /// The full original parsed line; unknown fields ride along.
    pub raw: Value,
}

/// Common top-level fields; each degrades gracefully when absent.
pub struct EntryMeta {
    /// Unique id of this entry.
    pub uuid: Option<String>,
    /// Id of the parent entry (conversation threading).
    pub parent_uuid: Option<String>,
    /// Raw ISO-8601 timestamp string, unparsed.
    pub timestamp: Option<String>,
    /// Session id (`sessionId`, falling back to `session_id`).
    pub session_id: Option<String>,
    /// True on subagent (sidechain) lines.
    pub is_sidechain: bool,
    /// True on injected/meta lines (not typed by the user).
    pub is_meta: bool,
    /// Working directory at the time of the entry.
    pub cwd: Option<String>,
}

/// Classified payload by top-level `type`.
pub enum EntryKind {
    /// `type=user`: a typed prompt (string content) or tool
    /// results (array content) — both normalized to blocks.
    User {
        /// Normalized content blocks.
        content: Vec<ContentBlock>,
        /// `promptSource` — "typed" marks a real human prompt.
        prompt_source: Option<String>,
    },
    /// `type=assistant`: in practice one JSONL line per content
    /// block, modeled as a `Vec` to stay shape-agnostic.
    Assistant {
        /// Normalized content blocks.
        content: Vec<ContentBlock>,
        /// `message.id` — groups the blocks of one API turn.
        message_id: Option<String>,
    },
    /// `type=system` bookkeeping (e.g. subtype `turn_duration`).
    System {
        /// The `subtype` field, when present.
        subtype: Option<String>,
    },
    /// Every other `type` (mode, permission-mode, progress,
    /// file-history-*, attachment, unknown/future types, …).
    Other {
        /// The original `type` value ("<none>" when absent).
        entry_type: String,
    },
}

/// One block of `message.content`.
pub enum ContentBlock {
    /// Visible text.
    Text {
        /// The text body.
        text: String,
    },
    /// Extended-thinking text.
    Thinking {
        /// The thinking body.
        thinking: String,
    },
    /// A tool invocation.
    ToolUse {
        /// Tool-use id (pairs with a later `ToolResult`).
        id: String,
        /// Tool name.
        name: String,
        /// Tool input, kept as raw JSON.
        input: Value,
    },
    /// A tool's result, sent back on a user line.
    ToolResult {
        /// The `tool_use_id` this result answers.
        tool_use_id: String,
        /// Result text (string form, or array-of-text joined).
        text: String,
        /// True when the tool errored.
        is_error: bool,
    },
    /// Unrecognized block type — counted, never rendered.
    Other {
        /// The original block `type` value.
        block_type: String,
    },
}

/// Parse a whole transcript file.
///
/// - Errors only on I/O (unreadable file).
/// - Malformed lines are recorded in `FileTranscript::malformed`.
pub fn parse_file(path: &Path) -> Result<FileTranscript, Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    Ok(parse_str(&text))
}

/// Parse transcript text (unit-testable without files).
///
/// - Blank lines are skipped.
/// - Each remaining line becomes an `Entry` or a `malformed`
///   record; parsing never fails as a whole.
pub fn parse_str(text: &str) -> FileTranscript {
    let mut entries = Vec::new();
    let mut malformed = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line_no = i + 1;
        if line.trim().is_empty() {
            continue;
        }
        match parse_line(line_no, line) {
            Ok(entry) => entries.push(entry),
            Err(err) => malformed.push((line_no, err)),
        }
    }
    FileTranscript { entries, malformed }
}

/// Parse one JSONL line into an `Entry`.
///
/// - Err on invalid JSON or a non-object top level.
/// - Never panics; missing fields degrade per the type docs.
fn parse_line(line_no: usize, line: &str) -> Result<Entry, String> {
    let raw: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
    if !raw.is_object() {
        return Err("top level is not a JSON object".to_string());
    }
    let meta = extract_meta(&raw);
    let kind = extract_kind(&raw);
    Ok(Entry {
        line_no,
        meta,
        kind,
        raw,
    })
}

/// Extract the common top-level fields.
fn extract_meta(v: &Value) -> EntryMeta {
    EntryMeta {
        uuid: get_str(v, "uuid"),
        parent_uuid: get_str(v, "parentUuid"),
        timestamp: get_str(v, "timestamp"),
        session_id: get_str(v, "sessionId").or_else(|| get_str(v, "session_id")),
        is_sidechain: get_bool(v, "isSidechain"),
        is_meta: get_bool(v, "isMeta"),
        cwd: get_str(v, "cwd"),
    }
}

/// Classify the line by its top-level `type` field.
fn extract_kind(v: &Value) -> EntryKind {
    let entry_type = get_str(v, "type").unwrap_or_else(|| "<none>".to_string());
    match entry_type.as_str() {
        "user" => EntryKind::User {
            content: extract_content(&v["message"]),
            prompt_source: get_str(v, "promptSource"),
        },
        "assistant" => EntryKind::Assistant {
            content: extract_content(&v["message"]),
            message_id: get_str(&v["message"], "id"),
        },
        "system" => EntryKind::System {
            subtype: get_str(v, "subtype"),
        },
        _ => EntryKind::Other { entry_type },
    }
}

/// Normalize `message.content` to blocks.
///
/// - String content → a single `Text` block (typed prompts).
/// - Array content → one block per element by its `type`.
/// - Anything else → no blocks.
fn extract_content(message: &Value) -> Vec<ContentBlock> {
    match &message["content"] {
        Value::String(s) => vec![ContentBlock::Text { text: s.clone() }],
        Value::Array(items) => items.iter().map(extract_block).collect(),
        _ => Vec::new(),
    }
}

/// Extract one content block by its `type` field.
fn extract_block(b: &Value) -> ContentBlock {
    let block_type = get_str(b, "type").unwrap_or_else(|| "<none>".to_string());
    match block_type.as_str() {
        "text" => ContentBlock::Text {
            text: get_str(b, "text").unwrap_or_default(), // OK: obvious
        },
        "thinking" => ContentBlock::Thinking {
            thinking: get_str(b, "thinking").unwrap_or_default(), // OK: obvious
        },
        "tool_use" => ContentBlock::ToolUse {
            id: get_str(b, "id").unwrap_or_default(),     // OK: obvious
            name: get_str(b, "name").unwrap_or_default(), // OK: obvious
            input: b["input"].clone(),
        },
        "tool_result" => ContentBlock::ToolResult {
            tool_use_id: get_str(b, "tool_use_id").unwrap_or_default(), // OK: obvious
            text: tool_result_text(&b["content"]),
            is_error: get_bool(b, "is_error"),
        },
        _ => ContentBlock::Other { block_type },
    }
}

/// Flatten `tool_result` content to text.
///
/// - String form → as-is.
/// - Array form → the `text` of each `type=text` element,
///   newline-joined.
/// - Anything else → empty.
fn tool_result_text(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(items) => items
            .iter()
            .filter(|b| get_str(b, "type").as_deref() == Some("text"))
            .filter_map(|b| get_str(b, "text"))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

/// String field at `key`, as an owned `Option<String>`.
fn get_str(v: &Value, key: &str) -> Option<String> {
    v.get(key)?.as_str().map(str::to_string)
}

/// Bool field at `key`; false when absent or non-bool.
fn get_bool(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false) // OK: obvious
}

#[cfg(test)]
mod tests;

//! The `bot-session` subcommand: display a Claude Code session
//! transcript (`.jsonl`) as a readable conversation.
//!
//! Default view renders the dialogue — user prompts and assistant
//! text in full, tool calls as one-liners — and hides thinking,
//! tool results, meta/system/sidechain lines, and bookkeeping
//! entry types. The "Items listed" flags — `--thinking`,
//! `--results`, `--meta`, `--all` — add them back.
//! A trailing summary line always reports what was hidden or
//! skipped. Malformed lines (e.g. a live session's truncated
//! last line) warn to stderr and never fail the run.

use std::path::PathBuf;

use clap::Args;
use log::{debug, info, warn};
use serde_json::Value;

use crate::context::Context;
use crate::subcommand::SubcommandRunner;
use crate::transcript::{self, ContentBlock, EntryKind, FileTranscript};

/// Max lines of one tool result shown under `--results`.
const RESULT_LINE_CAP: usize = 10;

/// Max chars of a tool-use one-liner gist.
const GIST_CHAR_CAP: usize = 100;

/// Clap-derived args for `bot-session`.
#[derive(Args, Debug)]
pub struct BotSessionArgs {
    /// Session transcript .jsonl file to display
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Also list thinking blocks
    #[arg(long, help_heading = "Items listed")]
    pub thinking: bool,

    /// Also list tool results ([result] lines, truncated) under
    /// the [tool] calls that produced them
    #[arg(long, help_heading = "Items listed")]
    pub results: bool,

    /// Also list meta/system/sidechain entries
    #[arg(long, help_heading = "Items listed")]
    pub meta: bool,

    /// List everything: implies --thinking --results --meta
    #[arg(long, help_heading = "Items listed")]
    pub all: bool,

    /// Limit output to a slice of the rendered conversation
    /// lines (Note: Index is 0-based):
    ///   N    — first N lines (0 = summary only)
    ///   -N   — last N lines
    ///   I,C  — C lines starting at Index I
    ///   I,-C — C lines ending at Index I
    /// Cut points show an elision marker; the summary line always
    /// prints.
    #[arg(
        long,
        value_name = "SPEC",
        allow_hyphen_values = true,
        verbatim_doc_comment,
        help_heading = "Output range"
    )]
    pub lines: Option<String>,
}

/// A parsed `--lines` slice spec (see `parse_lines_spec`).
pub enum LinesSpec {
    /// `N` / `-N`: first N (positive) or last N (negative) lines.
    Single(i64),
    /// `I,C`: 0-based index I with forward (positive) or
    /// backward (negative) count C.
    Pair(i64, i64),
}

/// What the conversation view reveals beyond the default.
#[derive(Default)]
pub struct RenderOptions {
    /// Render thinking blocks.
    pub thinking: bool,
    /// Render tool results.
    pub results: bool,
    /// Render meta/system/sidechain entries.
    pub meta: bool,
}

/// Inputs to the bot-session op, flat, owned, clap-free.
pub struct BotSessionParams {
    /// Transcript file to display.
    pub file: PathBuf,
    /// Reveal options for the renderer.
    pub opts: RenderOptions,
    /// Optional `--lines` slice of the rendered output.
    pub lines: Option<LinesSpec>,
}

impl TryFrom<&BotSessionArgs> for BotSessionParams {
    type Error = String;

    /// Convert clap-derived `BotSessionArgs` into the flat
    /// `BotSessionParams`; fails on a malformed `--lines` spec.
    fn try_from(a: &BotSessionArgs) -> Result<Self, String> {
        let lines = match &a.lines {
            Some(s) => Some(parse_lines_spec(s)?),
            None => None,
        };
        Ok(Self {
            file: a.file.clone(),
            opts: RenderOptions {
                thinking: a.thinking || a.all,
                results: a.results || a.all,
                meta: a.meta || a.all,
            },
            lines,
        })
    }
}

impl SubcommandRunner for BotSessionArgs {
    type Params = BotSessionParams;

    /// Delegate to the `TryFrom<&BotSessionArgs>` impl.
    fn to_params(&self) -> Result<Self::Params, String> {
        BotSessionParams::try_from(self)
    }

    /// Run the `bot_session` op.
    fn run(ctx: &Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        bot_session(ctx, params)
    }
}

/// Run the `bot-session` subcommand: parse the transcript and
/// print the conversation view plus the trailing summary line.
///
/// `ctx` is unused — bot-session reads a plain file; it's present
/// for the uniform subcommand-layer signature.
pub fn bot_session(
    _ctx: &Context,
    params: &BotSessionParams,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("bot-session: enter");
    let t = transcript::parse_file(&params.file)?;
    for (line_no, err) in &t.malformed {
        warn!("bot-session: line {line_no}: {err}");
    }
    let (lines, stats) = render(&t, &params.opts);
    let total = lines.len();
    let (lines, sliced) = match &params.lines {
        Some(spec) => {
            let (start, end) = line_bounds(spec, total);
            (apply_lines(lines, spec), Some((end - start, total)))
        }
        None => (lines, None),
    };
    for line in &lines {
        info!("{line}");
    }
    info!("");
    info!("{}", summary_line(&stats, t.malformed.len(), sliced));
    debug!("bot-session: exit");
    Ok(())
}

/// Counters for the trailing summary line.
#[derive(Default)]
struct RenderStats {
    /// Turn headers emitted.
    shown: usize,
    /// Non-empty thinking blocks hidden (no `--thinking`).
    hidden_thinking: usize,
    /// Tool results hidden (no `--results`).
    hidden_tool_results: usize,
    /// Meta/system/sidechain entries hidden (no `--meta`).
    hidden_meta: usize,
    /// Bookkeeping (`Other`) entry types skipped.
    skipped_other: usize,
}

/// Identity of the turn a rendered line belongs to — used to
/// decide when to emit a new turn header.
type TurnKey = (String, Option<String>);

/// Render the conversation view.
///
/// - Returns the output lines and the hide/skip counters.
/// - A turn header is emitted when the (role, assistant
///   message-id) identity changes.
fn render(t: &FileTranscript, opts: &RenderOptions) -> (Vec<String>, RenderStats) {
    let mut lines: Vec<String> = Vec::new();
    let mut stats = RenderStats::default();
    let mut turn: Option<TurnKey> = None;

    for e in &t.entries {
        if e.meta.is_sidechain && !opts.meta {
            stats.hidden_meta += 1;
            continue;
        }
        let time = short_time(e.meta.timestamp.as_deref());
        match &e.kind {
            EntryKind::User {
                content,
                prompt_source: _,
            } => {
                if e.meta.is_meta && !opts.meta {
                    stats.hidden_meta += 1;
                    continue;
                }
                let label = if e.meta.is_meta {
                    "user (meta)"
                } else {
                    "user"
                };
                for b in content {
                    match b {
                        ContentBlock::Text { text } => {
                            if !text.trim().is_empty() {
                                header(&mut lines, &mut stats, &mut turn, label, None, &time);
                                lines.extend(text.lines().map(str::to_string));
                            }
                        }
                        ContentBlock::ToolResult { text, is_error, .. } => {
                            if opts.results {
                                push_result(&mut lines, text, *is_error);
                            } else {
                                stats.hidden_tool_results += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
            EntryKind::Assistant {
                content,
                message_id,
            } => {
                for b in content {
                    match b {
                        ContentBlock::Text { text } => {
                            if !text.trim().is_empty() {
                                header(
                                    &mut lines,
                                    &mut stats,
                                    &mut turn,
                                    "assistant",
                                    message_id.as_deref(),
                                    &time,
                                );
                                lines.extend(text.lines().map(str::to_string));
                            }
                        }
                        ContentBlock::ToolUse { name, input, .. } => {
                            header(
                                &mut lines,
                                &mut stats,
                                &mut turn,
                                "assistant",
                                message_id.as_deref(),
                                &time,
                            );
                            lines.push(format!("  [tool] {}", tool_use_gist(name, input)));
                        }
                        ContentBlock::Thinking { thinking } => {
                            if thinking.trim().is_empty() {
                                continue;
                            }
                            if opts.thinking {
                                header(
                                    &mut lines,
                                    &mut stats,
                                    &mut turn,
                                    "assistant",
                                    message_id.as_deref(),
                                    &time,
                                );
                                lines.push("  [thinking]".to_string());
                                lines.extend(thinking.lines().map(|l| format!("  {l}")));
                            } else {
                                stats.hidden_thinking += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
            EntryKind::System { subtype } => {
                if opts.meta {
                    if !lines.is_empty() {
                        lines.push(String::new());
                    }
                    let sub = subtype.as_deref().unwrap_or("system"); // OK: obvious
                    lines.push(format!("--- system {sub} {time} ---"));
                    turn = None;
                } else {
                    stats.hidden_meta += 1;
                }
            }
            EntryKind::Other { .. } => stats.skipped_other += 1,
        }
    }
    (lines, stats)
}

/// Emit a turn header if the (role, message-id) identity changed.
fn header(
    lines: &mut Vec<String>,
    stats: &mut RenderStats,
    turn: &mut Option<TurnKey>,
    role: &str,
    message_id: Option<&str>,
    time: &str,
) {
    let key: TurnKey = (role.to_string(), message_id.map(str::to_string));
    if turn.as_ref() == Some(&key) {
        return;
    }
    if !lines.is_empty() {
        lines.push(String::new());
    }
    lines.push(format!("=== {role} {time} ==="));
    *turn = Some(key);
    stats.shown += 1;
}

/// Append a tool result, indented and capped at
/// `RESULT_LINE_CAP` lines.
fn push_result(lines: &mut Vec<String>, text: &str, is_error: bool) {
    let tag = if is_error {
        "[result:error]"
    } else {
        "[result]"
    };
    let body: Vec<&str> = text.lines().collect();
    match body.split_first() {
        None => lines.push(format!("  {tag}")),
        Some((first, rest)) => {
            lines.push(format!("  {tag} {first}"));
            for l in rest.iter().take(RESULT_LINE_CAP - 1) {
                lines.push(format!("    {l}"));
            }
            if body.len() > RESULT_LINE_CAP {
                lines.push(format!("    … (+{} lines)", body.len() - RESULT_LINE_CAP));
            }
        }
    }
}

/// One-line gist of a tool_use: tool name plus the most
/// informative slice of its input.
///
/// - `Bash` → first line of `command`.
/// - `Read`/`Write`/`Edit` → `file_path`.
/// - Fallback → compact `key=value` pairs of string inputs.
/// - Always truncated to `GIST_CHAR_CAP` chars.
fn tool_use_gist(name: &str, input: &Value) -> String {
    let detail = match name {
        "Bash" => input["command"]
            .as_str()
            .and_then(|c| c.lines().next())
            .unwrap_or("") // OK: obvious
            .to_string(),
        "Read" | "Write" | "Edit" => input["file_path"]
            .as_str()
            .unwrap_or("") // OK: obvious
            .to_string(),
        _ => match input.as_object() {
            Some(map) => map
                .iter()
                .filter_map(|(k, v)| v.as_str().map(|s| format!("{k}={s}")))
                .collect::<Vec<_>>()
                .join(" "),
            None => String::new(),
        },
    };
    truncate_chars(&format!("{name}: {detail}"), GIST_CHAR_CAP)
}

/// Truncate to `max` chars, appending `…` when cut
/// (char-based to stay safe on multibyte text).
fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{cut}…")
    }
}

/// "HH:MM:SS" slice of an ISO-8601 timestamp; falls back to the
/// raw string, or "" when absent.
fn short_time(ts: Option<&str>) -> String {
    match ts {
        Some(t) => t.get(11..19).unwrap_or(t).to_string(), // OK: obvious
        None => String::new(),
    }
}

/// Parse a `--lines` spec string (see the flag's help).
///
/// - `N` / `-N` → `Single`; `I,C` → `Pair` (I >= 0).
/// - A zero count yields an empty slice — summary only.
/// - Anything else is an error naming the bad piece.
fn parse_lines_spec(s: &str) -> Result<LinesSpec, String> {
    let parse = |part: &str| -> Result<i64, String> {
        part.trim()
            .parse::<i64>()
            .map_err(|e| format!("--lines: bad number {part:?}: {e}"))
    };
    match s.split_once(',') {
        None => Ok(LinesSpec::Single(parse(s)?)),
        Some((o, c)) => {
            let o = parse(o)?;
            let c = parse(c)?;
            if o < 0 {
                return Err("--lines: index I must be >= 0".to_string());
            }
            Ok(LinesSpec::Pair(o, c))
        }
    }
}

/// Resolve a spec to clamped `(start, end)` bounds over `len`
/// rendered lines.
fn line_bounds(spec: &LinesSpec, len: usize) -> (usize, usize) {
    let (start, end) = match *spec {
        LinesSpec::Single(n) if n > 0 => (0, n.unsigned_abs() as usize),
        LinesSpec::Single(n) => (len.saturating_sub(n.unsigned_abs() as usize), len),
        LinesSpec::Pair(o, c) if c > 0 => {
            let o = o as usize;
            (o, o.saturating_add(c.unsigned_abs() as usize))
        }
        LinesSpec::Pair(o, c) => {
            let o = o as usize;
            (o.saturating_sub(c.unsigned_abs() as usize), o)
        }
    };
    let end = end.min(len);
    (start.min(end), end)
}

/// Slice the rendered lines per the spec, clamped to range, with
/// an elision marker at each cut point.
fn apply_lines(lines: Vec<String>, spec: &LinesSpec) -> Vec<String> {
    let len = lines.len();
    let (start, end) = line_bounds(spec, len);
    let mut out = Vec::new();
    if start > 0 {
        out.push(format!("… ({start} lines skipped)"));
    }
    out.extend(lines[start..end].iter().cloned());
    if end < len {
        out.push(format!("… ({} lines skipped)", len - end));
    }
    out
}

/// Compose the trailing summary line, omitting zero clauses.
///
/// `sliced` is `Some((displayed, total))` when `--lines` cut the
/// output — the line then leads with the slice size and re-labels
/// the file-wide stats as the full render, so it never claims
/// more was shown than was.
fn summary_line(stats: &RenderStats, malformed: usize, sliced: Option<(usize, usize)>) -> String {
    let mut parts = match sliced {
        Some((displayed, total)) => vec![
            format!("{displayed} of {total} lines shown (--lines)"),
            format!("full render: {} turns", stats.shown),
        ],
        None => vec![format!("{} turns shown", stats.shown)],
    };
    let mut hidden = Vec::new();
    if stats.hidden_thinking > 0 {
        hidden.push(format!("{} thinking", stats.hidden_thinking));
    }
    if stats.hidden_tool_results > 0 {
        hidden.push(format!("{} tool results", stats.hidden_tool_results));
    }
    if stats.hidden_meta > 0 {
        hidden.push(format!("{} meta/system", stats.hidden_meta));
    }
    if !hidden.is_empty() {
        parts.push(format!("hidden: {}", hidden.join(", ")));
    }
    if stats.skipped_other > 0 {
        parts.push(format!(
            "skipped: {} bookkeeping entries",
            stats.skipped_other
        ));
    }
    if malformed > 0 {
        parts.push(format!("{malformed} malformed lines"));
    }
    format!("bot-session: {}", parts.join("; "))
}

#[cfg(test)]
mod tests;

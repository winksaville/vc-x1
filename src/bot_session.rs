//! The `bot-session` subcommand: display a Claude Code session
//! transcript (`.jsonl`) as a readable conversation.
//!
//! Output is a set of *items* — headers, user, assistant, tool,
//! thinking, results, meta, summary — each toggled by `--<item>` /
//! `--no-<item>` flags (last one wins), with `--all` / `--none` as
//! bulk bases. The default set (headers, user, assistant, tool,
//! summary) can be replaced by the user config's
//! `[bot-session].items` list; CLI flags then adjust the resolved
//! set. Bookkeeping entry types are never rendered.
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

    /// List the trailing summary line [default: on]
    #[arg(
        long,
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "no_summary"
    )]
    pub summary: bool,
    /// Do not list the trailing summary line
    #[arg(
        long,
        hide = true,
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "summary"
    )]
    pub no_summary: bool,

    /// List turn headers (=== role time ===) [default: on]
    #[arg(
        long,
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "no_headers"
    )]
    pub headers: bool,
    /// Do not list turn headers
    #[arg(
        long,
        hide = true,
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "headers"
    )]
    pub no_headers: bool,

    /// List typed user prompts [default: on]
    #[arg(
        long,
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "no_user"
    )]
    pub user: bool,
    /// Do not list typed user prompts
    #[arg(
        long,
        hide = true,
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "user"
    )]
    pub no_user: bool,

    /// List assistant text [default: on]
    #[arg(
        long,
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "no_assistant"
    )]
    pub assistant: bool,
    /// Do not list assistant text
    #[arg(
        long,
        hide = true,
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "assistant"
    )]
    pub no_assistant: bool,

    /// List [tool] call one-liners [default: on]
    #[arg(
        long,
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "no_tool"
    )]
    pub tool: bool,
    /// Do not list [tool] call one-liners
    #[arg(
        long,
        hide = true,
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "tool"
    )]
    pub no_tool: bool,

    /// List thinking blocks [default: off]
    #[arg(
        long,
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "no_thinking"
    )]
    pub thinking: bool,
    /// Do not list thinking blocks
    #[arg(
        long,
        hide = true,
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "thinking"
    )]
    pub no_thinking: bool,

    /// List tool results ([result] lines, truncated) under the
    /// [tool] calls that produced them [default: off]
    #[arg(
        long,
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "no_results"
    )]
    pub results: bool,
    /// Do not list tool results
    #[arg(
        long,
        hide = true,
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "results"
    )]
    pub no_results: bool,

    /// List meta/system/sidechain entries [default: off]
    #[arg(
        long,
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "no_meta"
    )]
    pub meta: bool,
    /// Do not list meta/system/sidechain entries
    #[arg(
        long,
        hide = true,
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "meta"
    )]
    pub no_meta: bool,

    /// Start from every item on (then subtract with --no-<item>)
    #[arg(
        long,
        alias = "no-none",
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "none"
    )]
    pub all: bool,
    /// Start from no items (then add with --<item>)
    #[arg(
        long,
        alias = "no-all",
        help_heading = "Items listed (--no-<item> to disable)",
        overrides_with = "all"
    )]
    pub none: bool,

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

/// The set of output items the renderer emits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemSet {
    /// Turn headers (`=== role time ===`).
    pub headers: bool,
    /// Typed user prompts.
    pub user: bool,
    /// Assistant text.
    pub assistant: bool,
    /// `[tool]` call one-liners.
    pub tool: bool,
    /// Thinking blocks.
    pub thinking: bool,
    /// `[result]` lines.
    pub results: bool,
    /// Meta/system/sidechain entries.
    pub meta: bool,
    /// The trailing summary line.
    pub summary: bool,
}

impl ItemSet {
    /// Every item off.
    pub const NONE: ItemSet = ItemSet {
        headers: false,
        user: false,
        assistant: false,
        tool: false,
        thinking: false,
        results: false,
        meta: false,
        summary: false,
    };
    /// Every item on.
    pub const ALL: ItemSet = ItemSet {
        headers: true,
        user: true,
        assistant: true,
        tool: true,
        thinking: true,
        results: true,
        meta: true,
        summary: true,
    };
    /// Built-in default view: the conversation essentials.
    pub const BUILTIN: ItemSet = ItemSet {
        headers: true,
        user: true,
        assistant: true,
        tool: true,
        thinking: false,
        results: false,
        meta: false,
        summary: true,
    };
}

/// Per-item CLI toggles, pre-resolution.
///
/// - `Some(true)` from `--<item>`, `Some(false)` from
///   `--no-<item>`, `None` when neither was given.
/// - `all` / `none` pick the resolution base.
#[derive(Default)]
pub struct ItemToggles {
    /// `--all`: base = every item on.
    pub all: bool,
    /// `--none`: base = every item off.
    pub none: bool,
    /// Per-item overrides, applied after the base.
    pub headers: Option<bool>,
    /// See `headers`.
    pub user: Option<bool>,
    /// See `headers`.
    pub assistant: Option<bool>,
    /// See `headers`.
    pub tool: Option<bool>,
    /// See `headers`.
    pub thinking: Option<bool>,
    /// See `headers`.
    pub results: Option<bool>,
    /// See `headers`.
    pub meta: Option<bool>,
    /// See `headers`.
    pub summary: Option<bool>,
}

/// Inputs to the bot-session op, flat, owned, clap-free.
pub struct BotSessionParams {
    /// Transcript file to display.
    pub file: PathBuf,
    /// Per-item CLI toggles (resolved against config in the op).
    pub toggles: ItemToggles,
    /// Optional `--lines` slice of the rendered output.
    pub lines: Option<LinesSpec>,
}

/// Fold a `--<item>` / `--no-<item>` pair into an override.
fn toggle(pos: bool, neg: bool) -> Option<bool> {
    match (pos, neg) {
        (true, _) => Some(true),
        (_, true) => Some(false),
        _ => None,
    }
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
            toggles: ItemToggles {
                all: a.all,
                none: a.none,
                headers: toggle(a.headers, a.no_headers),
                user: toggle(a.user, a.no_user),
                assistant: toggle(a.assistant, a.no_assistant),
                tool: toggle(a.tool, a.no_tool),
                thinking: toggle(a.thinking, a.no_thinking),
                results: toggle(a.results, a.no_results),
                meta: toggle(a.meta, a.no_meta),
                summary: toggle(a.summary, a.no_summary),
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
/// The item set resolves git-style, most specific wins: CLI
/// toggles > workspace `.vc-config.toml` > user config >
/// built-in default (`--all`/`--none` are CLI-level bases).
pub fn bot_session(
    ctx: &Context,
    params: &BotSessionParams,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("bot-session: enter");
    let workspace_items = workspace_items()?;
    let config_items = workspace_items
        .as_deref()
        .or(ctx.user_config.bot_session_items.as_deref());
    let items = resolve_items(&params.toggles, config_items)?;
    let t = transcript::parse_file(&params.file)?;
    for (line_no, err) in &t.malformed {
        warn!("bot-session: line {line_no}: {err}");
    }
    let (lines, stats) = render(&t, &items);
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
    if items.summary {
        info!("");
        info!("{}", summary_line(&stats, t.malformed.len(), sliced));
    }
    debug!("bot-session: exit");
    Ok(())
}

/// Read `[bot-session].items` from the workspace's
/// `.vc-config.toml`, when cwd is inside a workspace.
///
/// - No workspace, no file, or no key → `Ok(None)`.
/// - Unreadable/malformed file → error (it exists but can't be
///   used; silence would mask a real config problem).
fn workspace_items() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let Some(root) = crate::common::find_workspace_root() else {
        return Ok(None);
    };
    let path = root.join(".vc-config.toml");
    if !path.exists() {
        return Ok(None);
    }
    let map = crate::toml_simple::toml_load(&path)?;
    Ok(map.get("bot-session.items").cloned())
}

/// Resolve the effective item set from toggles + config.
///
/// - Base: `--all` → `ALL`; `--none` → `NONE`; else the config
///   list when present; else `BUILTIN`.
/// - Each per-item toggle then overrides its field.
fn resolve_items(t: &ItemToggles, config: Option<&str>) -> Result<ItemSet, String> {
    let mut s = if t.all {
        ItemSet::ALL
    } else if t.none {
        ItemSet::NONE
    } else {
        match config {
            Some(c) => parse_item_list(c)?,
            None => ItemSet::BUILTIN,
        }
    };
    if let Some(v) = t.headers {
        s.headers = v;
    }
    if let Some(v) = t.user {
        s.user = v;
    }
    if let Some(v) = t.assistant {
        s.assistant = v;
    }
    if let Some(v) = t.tool {
        s.tool = v;
    }
    if let Some(v) = t.thinking {
        s.thinking = v;
    }
    if let Some(v) = t.results {
        s.results = v;
    }
    if let Some(v) = t.meta {
        s.meta = v;
    }
    if let Some(v) = t.summary {
        s.summary = v;
    }
    Ok(s)
}

/// Parse a comma-separated item list (`[bot-session].items`).
///
/// - Unknown names error, naming the valid set.
/// - An empty list is allowed (start from nothing).
fn parse_item_list(s: &str) -> Result<ItemSet, String> {
    let mut set = ItemSet::NONE;
    for name in s.split(',').map(str::trim).filter(|n| !n.is_empty()) {
        match name {
            "headers" => set.headers = true,
            "user" => set.user = true,
            "assistant" => set.assistant = true,
            "tool" => set.tool = true,
            "thinking" => set.thinking = true,
            "results" => set.results = true,
            "meta" => set.meta = true,
            "summary" => set.summary = true,
            _ => {
                return Err(format!(
                    "[bot-session].items: unknown item {name:?} (valid: headers, \
                     user, assistant, tool, thinking, results, meta, summary)"
                ));
            }
        }
    }
    Ok(set)
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
fn render(t: &FileTranscript, items: &ItemSet) -> (Vec<String>, RenderStats) {
    let mut lines: Vec<String> = Vec::new();
    let mut stats = RenderStats::default();
    let mut turn: Option<TurnKey> = None;

    for e in &t.entries {
        if e.meta.is_sidechain && !items.meta {
            stats.hidden_meta += 1;
            continue;
        }
        let time = short_time(e.meta.timestamp.as_deref());
        match &e.kind {
            EntryKind::User {
                content,
                prompt_source: _,
            } => {
                if e.meta.is_meta && !items.meta {
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
                            if items.user && !text.trim().is_empty() {
                                header(
                                    &mut lines, &mut stats, &mut turn, label, None, &time, items,
                                );
                                lines.extend(text.lines().map(str::to_string));
                            }
                        }
                        ContentBlock::ToolResult { text, is_error, .. } => {
                            if items.results {
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
                            if items.assistant && !text.trim().is_empty() {
                                header(
                                    &mut lines,
                                    &mut stats,
                                    &mut turn,
                                    "assistant",
                                    message_id.as_deref(),
                                    &time,
                                    items,
                                );
                                lines.extend(text.lines().map(str::to_string));
                            }
                        }
                        ContentBlock::ToolUse { name, input, .. } => {
                            if items.tool {
                                header(
                                    &mut lines,
                                    &mut stats,
                                    &mut turn,
                                    "assistant",
                                    message_id.as_deref(),
                                    &time,
                                    items,
                                );
                                lines.push(format!("  [tool] {}", tool_use_gist(name, input)));
                            }
                        }
                        ContentBlock::Thinking { thinking } => {
                            if thinking.trim().is_empty() {
                                continue;
                            }
                            if items.thinking {
                                header(
                                    &mut lines,
                                    &mut stats,
                                    &mut turn,
                                    "assistant",
                                    message_id.as_deref(),
                                    &time,
                                    items,
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
                if items.meta {
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
    items: &ItemSet,
) {
    let key: TurnKey = (role.to_string(), message_id.map(str::to_string));
    if turn.as_ref() == Some(&key) {
        return;
    }
    if !lines.is_empty() {
        lines.push(String::new());
    }
    if items.headers {
        lines.push(format!("=== {role} {time} ==="));
    }
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

/// "YYYY-MM-DD HH:MM:SSZ" slice of an ISO-8601 UTC timestamp;
/// "" when absent. Observed transcript timestamps are always
/// UTC (trailing Z, all 56k lines to date) and the Z is kept so
/// the display names its zone — but that's observation, not a
/// documented guarantee, so a timestamp in any other shape
/// (offset form, too short) passes through verbatim rather than
/// being sliced and mislabeled.
fn short_time(ts: Option<&str>) -> String {
    match ts {
        Some(t) if t.ends_with('Z') => match t.get(..19) {
            Some(dt) => format!("{}Z", dt.replacen('T', " ", 1)),
            None => t.to_string(),
        },
        Some(t) => t.to_string(),
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

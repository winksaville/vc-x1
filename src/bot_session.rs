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

/// Default max lines of one tool result shown under `--results`.
pub(crate) const RESULT_LINE_CAP: usize = 10;

/// Default first-column width in the `--fields` /
/// `--unknown` / `--per-line` views.
///
/// - 68 aligns the type column for ~99% of observed key paths —
///   every structural key except a long tail of
///   `snapshot.trackedFileBackups.<absolute path>.*` keys, whose
///   embedded absolute paths can be arbitrarily long and so are
///   left to overflow.
/// - Override with `--col-width`.
pub(crate) const COL_WIDTH: usize = 68;

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

    /// Limit output to a slice of the file's source JSONL lines
    /// — the same unit in every view (Note: Index is 0-based):
    ///   N    — first N lines (0 = summary only)
    ///   -N   — last N lines
    ///   I,C  — C lines starting at Index I
    ///   I,-C — C lines ending at Index I
    /// The conversation view renders the entries in the slice,
    /// with elision markers at cut points; the summary always
    /// prints.
    #[arg(
        long,
        value_name = "SPEC",
        allow_hyphen_values = true,
        verbatim_doc_comment,
        help_heading = "Output range"
    )]
    pub lines: Option<String>,

    /// Max lines shown per tool result (0 = unlimited)
    /// [default: 10; or [bot-session].result-lines]
    #[arg(long, value_name = "N", help_heading = "Output range")]
    pub result_lines: Option<usize>,

    /// First column width in the --fields / --unknown /
    /// --per-line views; longer field names overflow
    /// [default: 68; or [bot-session].col-width]
    #[arg(long, value_name = "N", help_heading = "Alternate views")]
    pub col_width: Option<usize>,

    /// Field inventory instead of the conversation: every field
    /// observed per entry type, with count, value kinds, and
    /// short samples
    #[arg(long, help_heading = "Alternate views", conflicts_with = "raw")]
    pub fields: bool,

    /// Like --fields but only fields the typed layer does not
    /// consume — the unmodeled / new surface
    #[arg(long, help_heading = "Alternate views", conflicts_with = "raw")]
    pub unknown: bool,

    /// Pretty-printed source lines instead of the conversation
    /// (unparseable lines pass through verbatim)
    #[arg(long, help_heading = "Alternate views")]
    pub raw: bool,

    /// With --fields/--unknown (implies --fields): list each
    /// selected source line's fields separately instead of
    /// aggregating across the file
    #[arg(long, help_heading = "Alternate views", conflicts_with = "raw")]
    pub per_line: bool,
}

/// Which view the invocation renders.
pub enum View {
    /// The default conversation rendering.
    Conversation,
    /// Field inventory; `unknown_only` filters to unmodeled paths,
    /// `per_line` lists each source line separately.
    Fields {
        /// Show only paths not consumed by the typed layer.
        unknown_only: bool,
        /// Group by source line instead of aggregating.
        per_line: bool,
    },
    /// Pretty-printed source lines.
    Raw,
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
    /// Max lines per tool result (0 = unlimited); `None` when
    /// `--result-lines` was not given (resolved against config in
    /// the op).
    pub result_lines: Option<usize>,
    /// First-column width in the field-inventory views; `None`
    /// when `--col-width` was not given (resolved against config
    /// in the op).
    pub col_width: Option<usize>,
    /// Which view to render.
    pub view: View,
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
            result_lines: a.result_lines,
            col_width: a.col_width,
            view: if a.raw {
                View::Raw
            } else if a.fields || a.unknown || a.per_line {
                View::Fields {
                    unknown_only: a.unknown,
                    per_line: a.per_line,
                }
            } else {
                View::Conversation
            },
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
/// The item set, `--result-lines`, and `--col-width` all resolve
/// git-style, most specific wins: CLI > workspace
/// `.vc-config.toml` > user config > built-in default
/// (`--all`/`--none` are CLI-level bases for the item set).
pub fn bot_session(
    ctx: &Context,
    params: &BotSessionParams,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("bot-session: enter");
    let ws = workspace_bot_session()?;
    let col_width = params
        .col_width
        .or(ws.col_width)
        .or(ctx.user_config.bot_session_col_width)
        .unwrap_or(COL_WIDTH);
    let result_lines = params
        .result_lines
        .or(ws.result_lines)
        .or(ctx.user_config.bot_session_result_lines)
        .unwrap_or(RESULT_LINE_CAP);
    match params.view {
        View::Raw => return raw_view(params),
        View::Fields {
            unknown_only,
            per_line,
        } => return fields_view(params, unknown_only, per_line, col_width),
        View::Conversation => {}
    }
    let config_items = ws
        .items
        .as_deref()
        .or(ctx.user_config.bot_session_items.as_deref());
    let items = resolve_items(&params.toggles, config_items)?;
    let text = std::fs::read_to_string(&params.file)
        .map_err(|e| format!("cannot read {}: {e}", params.file.display()))?;
    let total = text.lines().count();
    let range = params.lines.as_ref().map(|spec| line_bounds(spec, total));
    let (start, end) = range.unwrap_or((0, total));
    let t = transcript::parse_str(&text);
    for (line_no, err) in &t.malformed {
        if *line_no > start && *line_no <= end {
            warn!("bot-session: line {line_no}: {err}");
        }
    }
    let (lines, stats) = render(&t, &items, result_lines, start, end, total);
    for line in &lines {
        info!("{line}");
    }
    if items.summary {
        let malformed = t
            .malformed
            .iter()
            .filter(|(n, _)| *n > start && *n <= end)
            .count();
        info!("");
        info!(
            "{}",
            summary_line(&stats, malformed, range.map(|(s, e)| (e - s, total)))
        );
    }
    debug!("bot-session: exit");
    Ok(())
}

/// Render the `--raw` view: pretty-printed source lines.
///
/// - `--lines` selects *source JSONL lines* (1-based file lines
///   map to 0-based Index), matching what jq/editors see.
/// - Parseable lines pretty-print; anything else (malformed,
///   truncated) passes through verbatim.
/// - No summary, no elision markers — the output is the data.
fn raw_view(params: &BotSessionParams) -> Result<(), Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(&params.file)
        .map_err(|e| format!("cannot read {}: {e}", params.file.display()))?;
    let all: Vec<&str> = text.lines().collect();
    let (start, end) = match &params.lines {
        Some(spec) => line_bounds(spec, all.len()),
        None => (0, all.len()),
    };
    for line in &all[start..end] {
        match serde_json::from_str::<Value>(line) {
            Ok(v) => match serde_json::to_string_pretty(&v) {
                Ok(pretty) => info!("{pretty}"),
                Err(_) => info!("{line}"),
            },
            Err(_) => info!("{line}"),
        }
    }
    Ok(())
}

/// One path's aggregate in the `--fields` inventory.
#[derive(Default)]
struct FieldAgg {
    /// How many leaves were observed at this path.
    count: usize,
    /// Value kinds seen (str / num / bool / null / empty-obj /
    /// empty-arr).
    kinds: std::collections::BTreeSet<&'static str>,
    /// Up to `SAMPLE_CAP` distinct short sample values.
    samples: Vec<String>,
}

/// Max distinct samples kept per path.
const SAMPLE_CAP: usize = 3;

/// Max chars of one sample value.
const SAMPLE_CHAR_CAP: usize = 36;

/// Render the `--fields` inventory: every field per entry type
/// with count, value kinds, and samples; `unknown_only` filters
/// to fields the typed layer does not consume.
fn fields_view(
    params: &BotSessionParams,
    unknown_only: bool,
    per_line: bool,
    col_width: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(&params.file)
        .map_err(|e| format!("cannot read {}: {e}", params.file.display()))?;
    let total = text.lines().count();
    // --lines selects *source JSONL lines* here, like --raw.
    let (start, end) = match &params.lines {
        Some(spec) => line_bounds(spec, total),
        None => (0, total),
    };
    let in_range = |line_no: usize| line_no > start && line_no <= end;
    let t = transcript::parse_str(&text);
    for (line_no, err) in &t.malformed {
        if in_range(*line_no) {
            warn!("bot-session: line {line_no}: {err}");
        }
    }
    if per_line {
        return per_line_view(
            &t,
            unknown_only,
            start,
            end,
            total,
            params.lines.is_some(),
            col_width,
        );
    }
    // (entry type, path) → aggregate, sorted for stable output.
    let mut agg: std::collections::BTreeMap<(String, String), FieldAgg> =
        std::collections::BTreeMap::new();
    let mut type_counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for e in t.entries.iter().filter(|e| in_range(e.line_no)) {
        let ty = e.raw["type"].as_str().unwrap_or("<none>").to_string(); // OK: obvious
        *type_counts.entry(ty.clone()).or_default() += 1;
        let mut leaves = Vec::new();
        transcript::leaf_paths(&e.raw, "", &mut leaves);
        for (path, v) in leaves {
            if unknown_only && transcript::is_known(&path) {
                continue;
            }
            let a = agg.entry((ty.clone(), path)).or_default();
            a.count += 1;
            a.kinds.insert(kind_name(v));
            if a.samples.len() < SAMPLE_CAP
                && let Some(sample) = sample_value(v)
                && !a.samples.contains(&sample)
            {
                a.samples.push(sample);
            }
        }
    }
    let mut paths = 0usize;
    let mut current_ty = None::<String>;
    for ((ty, path), a) in &agg {
        if current_ty.as_deref() != Some(ty) {
            if current_ty.is_some() {
                info!("");
            }
            let n = type_counts.get(ty).copied().unwrap_or(0); // OK: obvious
            info!("=== {ty} ({n} lines) ===");
            current_ty = Some(ty.clone());
        }
        paths += 1;
        let kinds = a.kinds.iter().copied().collect::<Vec<_>>().join("|");
        let samples = if a.samples.is_empty() {
            String::new()
        } else {
            format!("  {}", a.samples.join(" | "))
        };
        info!(
            "  {:<width$} {:<9} x{}{}",
            path,
            kinds,
            a.count,
            samples,
            width = col_width
        );
    }
    info!("");
    let label = if unknown_only {
        "unknown paths"
    } else {
        "paths"
    };
    let selected = t.entries.iter().filter(|e| in_range(e.line_no)).count();
    let malformed = t.malformed.iter().filter(|(n, _)| in_range(*n)).count();
    let mut tail = format!("{malformed} malformed lines");
    if params.lines.is_some() {
        tail.push_str(&format!(
            " (--lines selected {} of {total} source lines)",
            end - start
        ));
    }
    info!("bot-session: {paths} {label} across {selected} entries; {tail}");
    Ok(())
}

/// List each selected source line's fields as its own section
/// (`--per-line`): `=== Index N: <type> [time] ===` then one row
/// per leaf path. Malformed lines appear in place with their
/// error; the trailing summary matches the aggregated view.
fn per_line_view(
    t: &FileTranscript,
    unknown_only: bool,
    start: usize,
    end: usize,
    total: usize,
    sliced: bool,
    col_width: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let in_range = |line_no: usize| line_no > start && line_no <= end;
    let mut sections: Vec<(usize, Option<&crate::transcript::Entry>, Option<&str>)> = t
        .entries
        .iter()
        .filter(|e| in_range(e.line_no))
        .map(|e| (e.line_no, Some(e), None))
        .collect();
    sections.extend(
        t.malformed
            .iter()
            .filter(|(n, _)| in_range(*n))
            .map(|(n, err)| (*n, None, Some(err.as_str()))),
    );
    sections.sort_by_key(|(n, _, _)| *n);
    let mut paths = 0usize;
    let mut first = true;
    for (line_no, entry, err) in &sections {
        if !first {
            info!("");
        }
        first = false;
        let index = line_no - 1;
        match (entry, err) {
            (Some(e), _) => {
                let ty = e.raw["type"].as_str().unwrap_or("<none>"); // OK: obvious
                let time = short_time(e.meta.timestamp.as_deref());
                let time_part = if time.is_empty() {
                    String::new()
                } else {
                    format!(" {time}")
                };
                info!("=== Index {index}: {ty}{time_part} ===");
                let mut leaves = Vec::new();
                transcript::leaf_paths(&e.raw, "", &mut leaves);
                for (path, v) in leaves {
                    if unknown_only && transcript::is_known(&path) {
                        continue;
                    }
                    paths += 1;
                    let value = sample_value(v).unwrap_or_else(|| kind_name(v).to_string()); // OK: obvious
                    info!(
                        "  {:<width$} {:<9} {}",
                        path,
                        kind_name(v),
                        value,
                        width = col_width
                    );
                }
            }
            (None, Some(e)) => info!("=== Index {index}: <malformed: {e}> ==="),
            (None, None) => {}
        }
    }
    info!("");
    let label = if unknown_only {
        "unknown paths"
    } else {
        "paths"
    };
    let entries = sections.iter().filter(|(_, e, _)| e.is_some()).count();
    let malformed = sections.len() - entries;
    let mut tail = format!("{malformed} malformed lines");
    if sliced {
        tail.push_str(&format!(
            " (--lines selected {} of {total} source lines)",
            end - start
        ));
    }
    info!("bot-session: {paths} {label} across {entries} entries; {tail}");
    Ok(())
}

/// Short name of a leaf value's kind.
fn kind_name(v: &Value) -> &'static str {
    match v {
        Value::String(_) => "str",
        Value::Number(_) => "num",
        Value::Bool(_) => "bool",
        Value::Null => "null",
        Value::Object(_) => "obj{}",
        Value::Array(_) => "arr[]",
    }
}

/// A short display sample of a leaf value; None for empty
/// containers (nothing informative to show).
fn sample_value(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(truncate_chars(&s.replace('\n', "\\n"), SAMPLE_CHAR_CAP)),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null => Some("null".to_string()),
        _ => None,
    }
}

/// The `[bot-session]` scalars read from the workspace's
/// `.vc-config.toml`, unresolved (CLI/user-config layering
/// happens in `bot_session`).
struct WorkspaceBotSession {
    /// `[bot-session].items`.
    items: Option<String>,
    /// `[bot-session].result-lines`.
    result_lines: Option<usize>,
    /// `[bot-session].col-width`.
    col_width: Option<usize>,
}

/// Read `[bot-session]` from the workspace's `.vc-config.toml`,
/// when cwd is inside a workspace.
///
/// - No workspace, no file, or no key → all fields `None`.
/// - Unreadable/malformed file, or a present-but-unparseable
///   scalar → error (it exists but can't be used; silence would
///   mask a real config problem).
fn workspace_bot_session() -> Result<WorkspaceBotSession, Box<dyn std::error::Error>> {
    let Some(root) = crate::common::find_workspace_root() else {
        return Ok(WorkspaceBotSession {
            items: None,
            result_lines: None,
            col_width: None,
        });
    };
    let path = root.join(".vc-config.toml");
    if !path.exists() {
        return Ok(WorkspaceBotSession {
            items: None,
            result_lines: None,
            col_width: None,
        });
    }
    let map = crate::toml_simple::toml_load(&path)?;
    let parse_usize = |key: &str| -> Result<Option<usize>, Box<dyn std::error::Error>> {
        match map.get(key) {
            None => Ok(None),
            Some(s) => s
                .parse::<usize>()
                .map(Some)
                .map_err(|e| format!("{key}: invalid usize {s:?}: {e}").into()),
        }
    };
    Ok(WorkspaceBotSession {
        items: map.get("bot-session.items").cloned(),
        result_lines: parse_usize("bot-session.result-lines")?,
        col_width: parse_usize("bot-session.col-width")?,
    })
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
fn render(
    t: &FileTranscript,
    items: &ItemSet,
    result_lines: usize,
    start: usize,
    end: usize,
    total: usize,
) -> (Vec<String>, RenderStats) {
    let mut lines: Vec<String> = Vec::new();
    let mut stats = RenderStats::default();
    let mut turn: Option<TurnKey> = None;

    if start > 0 {
        lines.push(format!("… ({start} source lines skipped)"));
    }
    for e in t
        .entries
        .iter()
        .filter(|e| e.line_no > start && e.line_no <= end)
    {
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
                                push_result(&mut lines, text, *is_error, result_lines);
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
    if end < total {
        lines.push(format!("… ({} source lines skipped)", total - end));
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

/// Append a tool result, indented and capped at `cap` lines
/// (`0` = unlimited).
fn push_result(lines: &mut Vec<String>, text: &str, is_error: bool, cap: usize) {
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
            let limit = if cap == 0 {
                rest.len()
            } else {
                cap.saturating_sub(1)
            };
            for l in rest.iter().take(limit) {
                lines.push(format!("    {l}"));
            }
            if cap != 0 && body.len() > cap {
                lines.push(format!("    … (+{} lines)", body.len() - cap));
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

/// Compose the trailing summary line, omitting zero clauses.
///
/// `sliced` is `Some((selected, total))` when `--lines` cut the
/// input — the stats then describe only the selected source
/// lines, and a trailing clause names the slice.
fn summary_line(stats: &RenderStats, malformed: usize, sliced: Option<(usize, usize)>) -> String {
    let mut parts = vec![format!("{} turns shown", stats.shown)];
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
    if let Some((selected, total)) = sliced {
        parts.push(format!(
            "--lines selected {selected} of {total} source lines"
        ));
    }
    format!("bot-session: {}", parts.join("; "))
}

#[cfg(test)]
mod tests;

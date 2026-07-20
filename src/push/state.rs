//! Push state machinery — the `Stage` enum, the persistent
//! `PushState`, and state-file layout resolution.
//!
//! Extracted from `push.rs` (the refactor program's split-push.rs
//! stage) so stage-body work reviews in a file free of the state
//! plumbing:
//!
//! - `Stage` — the named stages of the push state machine.
//! - `PushState` — the resumable run state, persisted as flat TOML
//!   via `toml_simple`.
//! - `StateLayout` / `resolve_state_layout` — where the state file
//!   lives (`.vc-config.toml` `[push]` overrides, else defaults).

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use clap::ValueEnum;
use log::debug;

use crate::toml_simple::toml_load;

/// Current state-file format version — bump when the flat key set
/// changes incompatibly so readers can detect stale state and refuse
/// to resume instead of silently misinterpreting fields.
pub(crate) const STATE_FORMAT_VERSION: u32 = 1;

/// Default directory (relative to work-repo root) for the push state
/// file when `.vc-config.toml` has no `[push]` override.
pub(crate) const DEFAULT_STATE_DIR: &str = ".vc-x1";

/// Default filename for the push state file under `state_dir`.
pub(crate) const DEFAULT_STATE_FILE: &str = "push-state.toml";

/// Named stages of the `push` state machine.
///
/// Used by `--from <stage>` to resume at a specific point and by
/// `--status` to report the current position. Ordered top-down so
/// `Stage as u8` comparisons reflect progress through the flow.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum Stage {
    /// Verify bookmark tracking, the bot-published invariant,
    /// and sync state.
    Preflight,
    /// Present diff for the first approval gate.
    Review,
    /// Compose / edit the commit message; present for second gate.
    Message,
    /// Commit the work repo.
    CommitWork,
    /// Commit the `.claude` bot repo (skipped if empty).
    CommitBot,
    /// Advance each repo's bookmark to its `@-`: work → `<bookmark>`,
    /// bot → `main`.
    BookmarkSet,
    /// `jj git push --bookmark <b> -R .`.
    PushWork,
    /// In-process squash of `.claude`'s trailing session writes
    /// + push `main`.
    SquashPushBot,
}

impl Stage {
    /// Return the stage's kebab-case string identifier (matches what
    /// the CLI accepts via `--from <stage>` and what `PushState`
    /// persists to disk).
    pub fn as_str(self) -> &'static str {
        match self {
            Stage::Preflight => "preflight",
            Stage::Review => "review",
            Stage::Message => "message",
            Stage::CommitWork => "commit-work",
            Stage::CommitBot => "commit-bot",
            Stage::BookmarkSet => "bookmark-set",
            Stage::PushWork => "push-work",
            Stage::SquashPushBot => "squash-push-bot",
        }
    }

    /// Parse a kebab-case stage name back into a `Stage`.
    ///
    /// Unknown names return `None`; callers should surface a helpful
    /// error rather than silently substituting a default.
    pub fn from_str(name: &str) -> Option<Self> {
        match name {
            "preflight" => Some(Stage::Preflight),
            "review" => Some(Stage::Review),
            "message" => Some(Stage::Message),
            "commit-work" => Some(Stage::CommitWork),
            "commit-bot" => Some(Stage::CommitBot),
            "bookmark-set" => Some(Stage::BookmarkSet),
            "push-work" => Some(Stage::PushWork),
            "squash-push-bot" => Some(Stage::SquashPushBot),
            _ => None,
        }
    }

    /// The stage that follows this one, or `None` when this is the
    /// last stage (`SquashPushBot`).
    pub fn next(self) -> Option<Self> {
        match self {
            Stage::Preflight => Some(Stage::Review),
            Stage::Review => Some(Stage::Message),
            Stage::Message => Some(Stage::CommitWork),
            Stage::CommitWork => Some(Stage::CommitBot),
            Stage::CommitBot => Some(Stage::BookmarkSet),
            Stage::BookmarkSet => Some(Stage::PushWork),
            Stage::PushWork => Some(Stage::SquashPushBot),
            Stage::SquashPushBot => None,
        }
    }

    /// The first stage in the flow — used when no saved state exists.
    pub fn first() -> Self {
        Stage::Preflight
    }
}

/// Resolved state-file layout — just the full path today, structured
/// so future additions (e.g. a `dir` exposed for `.gitignore`
/// coherence checks in 0.37.0-3) have a place to land without
/// changing callers.
#[derive(Debug, Clone)]
pub struct StateLayout {
    /// Full path `<repo>/<state-dir>/<state-file>`.
    pub path: PathBuf,
}

/// Read `state_dir` / `state_file` from `<repo_root>/.vc-config.toml`
/// under the `[push]` section, falling back to defaults (`.vc-x1`
/// and `push-state.toml`).
///
/// Missing config file is not an error — every workspace has the
/// defaults applied silently so `vc-x1 push --status` works on a
/// fresh clone without ceremony.
pub fn resolve_state_layout(repo_root: &Path) -> StateLayout {
    let config_path = repo_root.join(".vc-config.toml");
    let (dir, file) = if config_path.exists() {
        match toml_load(&config_path) {
            Ok(map) => {
                let dir = map
                    .get("push.state-dir")
                    .cloned()
                    .unwrap_or_else(|| DEFAULT_STATE_DIR.to_string());
                let file = map
                    .get("push.state-file")
                    .cloned()
                    .unwrap_or_else(|| DEFAULT_STATE_FILE.to_string());
                (dir, file)
            }
            Err(e) => {
                debug!("push: ignoring unparseable .vc-config.toml: {e}");
                (
                    DEFAULT_STATE_DIR.to_string(),
                    DEFAULT_STATE_FILE.to_string(),
                )
            }
        }
    } else {
        (
            DEFAULT_STATE_DIR.to_string(),
            DEFAULT_STATE_FILE.to_string(),
        )
    };
    let path = repo_root.join(&dir).join(&file);
    StateLayout { path }
}

/// Persistent state for an in-progress `push` run.
///
/// Serialized as flat TOML (`key = "value"`) under the dir/file
/// configured in `.vc-config.toml`'s `[push]` section. Fields added
/// across dev steps are all `Option<_>` so older states remain
/// loadable — only `version` / `stage` / `bookmark` / `started_at`
/// are required. See the `push` module docstring for which dev step
/// introduced which field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushState {
    /// State-file format version; must match `STATE_FORMAT_VERSION`
    /// to be considered readable.
    pub version: u32,
    /// The next stage to execute on resume.
    pub stage: Stage,
    /// Code-repo bookmark being advanced by this run (persisted so
    /// resume doesn't need `--bookmark` again). The bot repo's
    /// side is pinned to `BOT_BOOKMARK`, not stored here.
    pub bookmark: String,
    /// ISO-8601 UTC timestamp captured when the state was first
    /// written. Informational — helps the user spot stale state.
    pub started_at: String,
    /// Work-repo changeID captured at `message` stage (before
    /// `commit-work` runs). Stable across `jj commit` — becomes the
    /// chid of the just-committed change. Used when composing the
    /// `.claude` commit's ochid trailer. Added in 0.37.0-2.
    pub work_chid: Option<String>,
    /// `.claude` repo changeID used by the work-repo commit's ochid
    /// trailer. Either the pre-commit `@` chid (when `.claude` has
    /// pending changes — becomes `@-` after commit) or the current
    /// `@-` chid (when `.claude` is clean — stays stable). Added in
    /// 0.37.0-2.
    pub bot_chid: Option<String>,
    /// Whether `.claude`'s working copy had changes at `message`
    /// time. Decides whether `commit-bot` actually runs or
    /// skips. Added in 0.37.0-2.
    pub bot_had_changes: Option<bool>,
    /// `jj op` id of the work repo captured before `commit-work`. On
    /// failure in stages 4-6, `jj op restore` rewinds here. Added
    /// in 0.37.0-2.
    pub op_app: Option<String>,
    /// `jj op` id of `.claude` captured before `commit-work`. Same
    /// rollback target as `op_app`. Added in 0.37.0-2.
    pub op_claude: Option<String>,
    /// Composed commit title — persisted so resume doesn't need
    /// `--title` re-passed. Set during `message` stage from either
    /// `--title` or `$EDITOR`. Added in 0.37.0-4.
    pub title: Option<String>,
    /// Composed commit body (sans ochid trailer, which each commit
    /// stage appends). Persisted alongside title. Multi-line
    /// content is escaped for the flat-TOML save format (see
    /// `escape_multiline` / `unescape_multiline`). Added in
    /// 0.37.0-4.
    pub body: Option<String>,
}

impl PushState {
    /// Build a fresh state for a new run.
    pub fn new_for(bookmark: &str) -> Self {
        PushState {
            version: STATE_FORMAT_VERSION,
            stage: Stage::first(),
            bookmark: bookmark.to_string(),
            started_at: Utc::now().to_rfc3339(),
            work_chid: None,
            bot_chid: None,
            bot_had_changes: None,
            op_app: None,
            op_claude: None,
            title: None,
            body: None,
        }
    }

    /// Write the state to `path`, creating parent dirs as needed.
    ///
    /// Values are single-line and contain no `"` characters under
    /// normal operation (stage names are kebab-case, chids are
    /// ASCII, bookmark names don't contain quotes, timestamps are
    /// ASCII). A defensive escape pass replaces any stray `"` with
    /// `\"` so the file always parses with `toml_simple`. Optional
    /// fields are only emitted when set so older state files don't
    /// carry a wall of blank keys.
    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut content = String::new();
        content.push_str("# vc-x1 push state — managed file, do not edit by hand\n");
        content.push_str("[push-state]\n");
        content.push_str(&format!("version = {}\n", self.version));
        content.push_str(&format!("stage = \"{}\"\n", self.stage.as_str()));
        content.push_str(&format!("bookmark = \"{}\"\n", escape_toml(&self.bookmark)));
        content.push_str(&format!(
            "started_at = \"{}\"\n",
            escape_toml(&self.started_at)
        ));
        if let Some(v) = &self.work_chid {
            content.push_str(&format!("work_chid = \"{}\"\n", escape_toml(v)));
        }
        if let Some(v) = &self.bot_chid {
            content.push_str(&format!("bot_chid = \"{}\"\n", escape_toml(v)));
        }
        if let Some(v) = self.bot_had_changes {
            content.push_str(&format!("bot_had_changes = {v}\n"));
        }
        if let Some(v) = &self.op_app {
            content.push_str(&format!("op_app = \"{}\"\n", escape_toml(v)));
        }
        if let Some(v) = &self.op_claude {
            content.push_str(&format!("op_claude = \"{}\"\n", escape_toml(v)));
        }
        if let Some(v) = &self.title {
            content.push_str(&format!("title = \"{}\"\n", escape_multiline(v)));
        }
        if let Some(v) = &self.body {
            content.push_str(&format!("body = \"{}\"\n", escape_multiline(v)));
        }
        fs::write(path, content)?;
        debug!("push: wrote state to {}", path.display());
        Ok(())
    }

    /// Load state from `path`, returning `Ok(None)` if the file is
    /// absent (a fresh run) and `Err` if the file exists but is
    /// unusable (stale format, missing required keys, unknown stage).
    pub fn load(path: &Path) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok(None);
        }
        let map = toml_load(path)?;
        let require = |k: &str| -> Result<String, Box<dyn std::error::Error>> {
            map.get(k)
                .cloned()
                .ok_or_else(|| format!("push state {}: missing key '{k}'", path.display()).into())
        };
        let version: u32 = require("push-state.version")?
            .parse()
            .map_err(|e| format!("push state {}: invalid version: {e}", path.display()))?;
        if version != STATE_FORMAT_VERSION {
            return Err(format!(
                "push state {}: unsupported format version {version} (expected {}); \
                 re-run with --restart to start fresh",
                path.display(),
                STATE_FORMAT_VERSION
            )
            .into());
        }
        let stage_str = require("push-state.stage")?;
        let stage = Stage::from_str(&stage_str)
            .ok_or_else(|| format!("push state {}: unknown stage '{stage_str}'", path.display()))?;
        let bot_had_changes = match map.get("push-state.bot_had_changes") {
            Some(s) => Some(s.parse::<bool>().map_err(|e| {
                format!(
                    "push state {}: invalid bot_had_changes: {e}",
                    path.display()
                )
            })?),
            None => None,
        };
        Ok(Some(PushState {
            version,
            stage,
            bookmark: require("push-state.bookmark")?,
            started_at: require("push-state.started_at")?,
            work_chid: map.get("push-state.work_chid").cloned(),
            bot_chid: map.get("push-state.bot_chid").cloned(),
            bot_had_changes,
            op_app: map.get("push-state.op_app").cloned(),
            op_claude: map.get("push-state.op_claude").cloned(),
            title: map.get("push-state.title").map(|s| unescape_multiline(s)),
            body: map.get("push-state.body").map(|s| unescape_multiline(s)),
        }))
    }
}

/// Escape any `"` in a value so the single-line TOML strings we emit
/// stay parseable. `toml_simple` trims the surrounding quotes on read
/// but doesn't process escapes, so we just avoid characters that
/// would break the single-line form.
fn escape_toml(s: &str) -> String {
    s.replace('"', "\\\"")
}

/// Escape a potentially multi-line string for persistence in a
/// single-line TOML value. `toml_simple` only handles one line per
/// value so we encode newlines as `\n`, tabs as `\t`, and any stray
/// `"` / `\` as their escaped forms. The inverse is
/// `unescape_multiline`.
pub(crate) fn escape_multiline(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out
}

/// Invert `escape_multiline`. Unknown backslash-escapes pass through
/// untouched (best-effort — this is a managed state file, so it's
/// unlikely to encounter hand-edited escapes).
pub(crate) fn unescape_multiline(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

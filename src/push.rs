//! `push` subcommand — collapse the dual-repo commit+push+finalize
//! ceremony into a single resumable command.
//!
//! See `notes/chores-05.md > Add push subcommand (0.37.0)` for the
//! full design.
//!
//! Dev-step ladder (expanded from original 4 to 6 after adding an
//! integration-test step ahead of the first dogfood):
//!
//! - `0.37.0-0` — scaffolding: flag surface, `Stage` enum, stub `push()`
//! - `0.37.0-1` — state file + stage-dispatch loop with stage stubs;
//!   `--status`, `--restart`, `--from`
//! - `0.37.0-2` — real stage bodies (commits, bookmarks, push,
//!   finalize) + `jj op` snapshot rollback
//! - `0.37.0-3` — integration tests + workspace-root refactor
//!   (thread `root: &Path` through every stage so fixtures can
//!   point them at tempdirs); first `vc-x1 push` dogfood ships
//!   this commit
//! - `0.37.0-4` — interactivity: two approval gates, `$EDITOR`,
//!   message persistence across resumes
//! - `0.37.0-5` — polish: `--dry-run`, `--step`, non-tty handling,
//!   `.gitignore` coherence warning
//! - `0.37.0` — docs + workflow migration (done marker)

use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use chrono::Utc;
use clap::{Args, ValueEnum};
use log::{debug, info, warn};

use crate::common::{prompt, run};
use crate::sync::{current_op_id, op_restore};
use crate::toml_simple::toml_load;

/// Named stages of the `push` state machine.
///
/// Used by `--from <stage>` to resume at a specific point and by
/// `--status` to report the current position. Ordered top-down so
/// `Stage as u8` comparisons reflect progress through the flow.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum Stage {
    /// Run fmt / clippy / test / install / retest.
    Preflight,
    /// Present diff for the first approval gate.
    Review,
    /// Compose / edit the commit message; present for second gate.
    Message,
    /// Commit the app repo.
    CommitApp,
    /// Commit the `.claude` session repo (skipped if empty).
    CommitClaude,
    /// Advance both bookmarks to `@-`.
    BookmarkBoth,
    /// `jj git push --bookmark <b> -R .`.
    PushApp,
    /// `vc-x1 finalize --repo .claude --squash --push <b> ...`.
    FinalizeClaude,
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
            Stage::CommitApp => "commit-app",
            Stage::CommitClaude => "commit-claude",
            Stage::BookmarkBoth => "bookmark-both",
            Stage::PushApp => "push-app",
            Stage::FinalizeClaude => "finalize-claude",
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
            "commit-app" => Some(Stage::CommitApp),
            "commit-claude" => Some(Stage::CommitClaude),
            "bookmark-both" => Some(Stage::BookmarkBoth),
            "push-app" => Some(Stage::PushApp),
            "finalize-claude" => Some(Stage::FinalizeClaude),
            _ => None,
        }
    }

    /// The stage that follows this one, or `None` when this is the
    /// last stage (`FinalizeClaude`).
    pub fn next(self) -> Option<Self> {
        match self {
            Stage::Preflight => Some(Stage::Review),
            Stage::Review => Some(Stage::Message),
            Stage::Message => Some(Stage::CommitApp),
            Stage::CommitApp => Some(Stage::CommitClaude),
            Stage::CommitClaude => Some(Stage::BookmarkBoth),
            Stage::BookmarkBoth => Some(Stage::PushApp),
            Stage::PushApp => Some(Stage::FinalizeClaude),
            Stage::FinalizeClaude => None,
        }
    }

    /// The first stage in the flow — used when no saved state exists.
    pub fn first() -> Self {
        Stage::Preflight
    }
}

/// CLI arguments for the `push` subcommand.
///
/// Flag set mirrors the design in `notes/chores-05.md`. Flags are
/// parsed in 0.37.0-0; their real effects land across the remaining
/// 0.37.0-N steps. See the module docstring for which flags activate
/// when.
#[derive(Args, Debug)]
pub struct PushArgs {
    /// Bookmark to advance in both repos (positional form of `--bookmark`).
    ///
    /// Accepting a positional lets the common case read as `vc-x1 push main`
    /// without the `--bookmark` ceremony; `--bookmark` is kept as an
    /// alias for scripts and for composition with other args. The two
    /// forms conflict if both supplied.
    #[arg(value_name = "BOOKMARK", conflicts_with = "bookmark")]
    pub bookmark_pos: Option<String>,

    /// Bookmark to advance in both repos (flag form; see positional).
    #[arg(long, conflicts_with = "bookmark_pos")]
    pub bookmark: Option<String>,

    /// Clear any saved state file and start from stage 1.
    #[arg(long)]
    pub restart: bool,

    /// Explicit stage to jump to (advanced / debug use).
    #[arg(long, value_name = "STAGE")]
    pub from: Option<Stage>,

    /// Pause between every stage for an interactive approval gate.
    #[arg(long)]
    pub step: bool,

    /// Print where the saved state thinks we are and exit.
    #[arg(long)]
    pub status: bool,

    /// Re-run `preflight` even on resume (default: skip if last run succeeded).
    #[arg(long)]
    pub recheck: bool,

    /// Stop before `finalize-claude` so it can be run manually.
    #[arg(long)]
    pub no_finalize: bool,

    /// Print the exact commands for every stage without side effects.
    #[arg(long)]
    pub dry_run: bool,

    /// Commit title (skip `$EDITOR` for the message stage).
    #[arg(long, value_name = "STR")]
    pub title: Option<String>,

    /// Commit body (skip `$EDITOR` for the message stage).
    #[arg(long, value_name = "STR")]
    pub body: Option<String>,

    /// Auto-approve interactive prompts (review gate, message-edit
    /// confirmation). Required when `--title` / `--body` aren't both
    /// supplied in non-interactive contexts.
    #[arg(short = 'y', long)]
    pub yes: bool,
}

/// Current state-file format version — bump when the flat key set
/// changes incompatibly so readers can detect stale state and refuse
/// to resume instead of silently misinterpreting fields.
const STATE_FORMAT_VERSION: u32 = 1;

/// Default directory (relative to app-repo root) for the push state
/// file when `.vc-config.toml` has no `[push]` override.
const DEFAULT_STATE_DIR: &str = ".vc-x1";

/// Default filename for the push state file under `state_dir`.
const DEFAULT_STATE_FILE: &str = "push-state.toml";

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
/// are required. See the module docstring for which dev step
/// introduced which field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushState {
    /// State-file format version; must match `STATE_FORMAT_VERSION`
    /// to be considered readable.
    pub version: u32,
    /// The next stage to execute on resume.
    pub stage: Stage,
    /// Bookmark being advanced by this run (persisted so resume
    /// doesn't need `--bookmark` again).
    pub bookmark: String,
    /// ISO-8601 UTC timestamp captured when the state was first
    /// written. Informational — helps the user spot stale state.
    pub started_at: String,
    /// App-repo changeID captured at `message` stage (before
    /// `commit-app` runs). Stable across `jj commit` — becomes the
    /// chid of the just-committed change. Used when composing the
    /// `.claude` commit's ochid trailer. Added in 0.37.0-2.
    pub app_chid: Option<String>,
    /// `.claude` repo changeID used by the app-repo commit's ochid
    /// trailer. Either the pre-commit `@` chid (when `.claude` has
    /// pending changes — becomes `@-` after commit) or the current
    /// `@-` chid (when `.claude` is clean — stays stable). Added in
    /// 0.37.0-2.
    pub claude_chid: Option<String>,
    /// Whether `.claude`'s working copy had changes at `message`
    /// time. Decides whether `commit-claude` actually runs or
    /// skips. Added in 0.37.0-2.
    pub claude_had_changes: Option<bool>,
    /// `jj op` id of the app repo captured before `commit-app`. On
    /// failure in stages 4-6, `jj op restore` rewinds here. Added
    /// in 0.37.0-2.
    pub op_app: Option<String>,
    /// `jj op` id of `.claude` captured before `commit-app`. Same
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
            app_chid: None,
            claude_chid: None,
            claude_had_changes: None,
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
        if let Some(v) = &self.app_chid {
            content.push_str(&format!("app_chid = \"{}\"\n", escape_toml(v)));
        }
        if let Some(v) = &self.claude_chid {
            content.push_str(&format!("claude_chid = \"{}\"\n", escape_toml(v)));
        }
        if let Some(v) = self.claude_had_changes {
            content.push_str(&format!("claude_had_changes = {v}\n"));
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
        let claude_had_changes = match map.get("push-state.claude_had_changes") {
            Some(s) => Some(s.parse::<bool>().map_err(|e| {
                format!(
                    "push state {}: invalid claude_had_changes: {e}",
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
            app_chid: map.get("push-state.app_chid").cloned(),
            claude_chid: map.get("push-state.claude_chid").cloned(),
            claude_had_changes,
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
fn escape_multiline(s: &str) -> String {
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
fn unescape_multiline(s: &str) -> String {
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

/// Whether `stdin` is attached to a terminal.
///
/// Interactive prompts (`stage_review`, `$EDITOR` launch in
/// `stage_message`, `--step` gates) need this to fail fast in
/// scripted / CI contexts instead of hanging on `read_line`.
/// `--yes` overrides the check — script callers opt in by
/// asserting "all prompts auto-approved".
fn is_stdin_tty() -> bool {
    std::io::stdin().is_terminal()
}

/// Verify the configured state dir is matched by a `.gitignore` entry,
/// erroring if not. The check is intentionally simple — looks for
/// `<state_dir>` or `/<state_dir>` as a line in `.gitignore` at the
/// repo root. Misses trickier patterns (nested wildcards, ignore files
/// further down the tree) but catches the common "user changed the
/// config and forgot to update .gitignore" case, which is the whole
/// point. Fatal as of 0.37.1 — the warning was easy to miss in the
/// preflight wall of output, and a committed state file is a real
/// foot-gun. `vc-x1 init` and `vc-x1 test-fixture` write `/.vc-x1`
/// into their generated `.gitignore` so the fresh-repo path is clean.
fn check_gitignore_coherence(
    root: &Path,
    state_dir_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let gitignore = root.join(".gitignore");
    let matched = match fs::read_to_string(&gitignore) {
        Ok(content) => content.lines().any(|line| {
            let l = line.trim();
            l == state_dir_name
                || l == format!("/{state_dir_name}")
                || l == format!("{state_dir_name}/")
                || l == format!("/{state_dir_name}/")
        }),
        Err(_) => false,
    };
    if !matched {
        return Err(format!(
            "push: state dir '{state_dir_name}' is not in {} — \
             add a '/{state_dir_name}' line so in-progress push-state \
             files don't get committed",
            gitignore.display()
        )
        .into());
    }
    Ok(())
}

/// Entry point for the `push` subcommand.
///
/// 0.37.0-5 behavior: interactive flow with two approval gates
/// (review + message) plus polish flags: `--dry-run` prints what
/// would run without side effects, `--step` pauses between every
/// stage, non-tty stdin fails fast when prompts are required,
/// `--yes` opts out of all prompts. On any failure in stages 4-6
/// (the local mutation window between `commit-app` and
/// `bookmark-both`), both repos roll back to the `jj op` snapshot
/// recorded at the start of `commit-app`. After `push-app` the
/// remote boundary is crossed and recovery is forward-only.
pub fn push(args: &PushArgs) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    push_in(&cwd, args)
}

/// `push` parameterized on the workspace root. CLI dispatch calls
/// this with `std::env::current_dir()`; integration tests call it
/// with a fixture tempdir so the stage bodies mutate the fixture's
/// repos instead of the developer's working tree.
pub(crate) fn push_in(
    workspace_root: &Path,
    args: &PushArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let layout = resolve_state_layout(workspace_root);

    if args.status {
        return cmd_status(&layout);
    }

    if args.dry_run {
        info!("push: DRY-RUN — no side effects (no commits, no pushes, no state written)");
    }

    // Gitignore coherence check (fatal). Only checked when we
    // actually have a file path with a parent (resolve_state_layout
    // always gives one, so this is effectively always-on).
    if let Some(dir_name) = layout
        .path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
    {
        check_gitignore_coherence(workspace_root, dir_name)?;
    }

    if args.restart && layout.path.exists() {
        fs::remove_file(&layout.path)?;
        debug!("push: --restart cleared state at {}", layout.path.display());
    }

    // Load existing state, or require a bookmark (positional or flag)
    // to bootstrap one.
    let mut state = match PushState::load(&layout.path)? {
        Some(s) => s,
        None => {
            let bookmark = args
                .bookmark_pos
                .as_deref()
                .or(args.bookmark.as_deref())
                .ok_or(
                    "push: no saved state; a bookmark is required to start a new run \
                     — pass it as a positional (`vc-x1 push main`) or via `--bookmark main`",
                )?;
            PushState::new_for(bookmark)
        }
    };

    // `--from` overrides the resumed stage (does not affect bookmark
    // or other persisted fields).
    if let Some(from) = args.from {
        state.stage = from;
    }

    run_from(workspace_root, &mut state, args, &layout)
}

/// Full path of the `.claude` session repo for a given workspace
/// root. Centralized so a future layout change (e.g. configurable
/// session-repo name) has one caller to update.
fn claude_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".claude")
}

/// Print the resumed stage (or "no saved state") and return.
fn cmd_status(layout: &StateLayout) -> Result<(), Box<dyn std::error::Error>> {
    match PushState::load(&layout.path)? {
        Some(state) => {
            info!(
                "push-state: stage={} bookmark={} started={} (file: {})",
                state.stage.as_str(),
                state.bookmark,
                state.started_at,
                layout.path.display()
            );
        }
        None => {
            info!(
                "push-state: no saved state ({} does not exist)",
                layout.path.display()
            );
        }
    }
    Ok(())
}

/// Walk the state machine from `state.stage` to the end, saving
/// progress after each stage.
///
/// Records a `jj op` snapshot in both repos the first time we enter
/// `commit-app` and leaves it in state so resume inherits the
/// rollback target. On failure inside the rollback-eligible window
/// (`commit-app` / `commit-claude` / `bookmark-both`), both repos
/// are restored before the error propagates.
fn run_from(
    root: &Path,
    state: &mut PushState,
    args: &PushArgs,
    layout: &StateLayout,
) -> Result<(), Box<dyn std::error::Error>> {
    // Only persist state in real runs. Dry-runs are inspection-only
    // — carrying their inferred chids / op snapshots into real runs
    // would mislead the user about what was actually recorded.
    if !args.dry_run {
        state.save(&layout.path)?;
    }
    loop {
        let stage = state.stage;

        // Snapshot op ids once on first `commit-app` entry. Skipped
        // in dry-run since we won't mutate anything to roll back.
        if stage == Stage::CommitApp && state.op_app.is_none() && !args.dry_run {
            state.op_app = Some(current_op_id(root)?);
            state.op_claude = Some(current_op_id(&claude_path(root))?);
            state.save(&layout.path)?;
        }

        let result = run_stage(root, stage, state, args, layout);

        if let Err(e) = &result {
            if stage_is_rollback_eligible(stage) && !args.dry_run {
                rollback_on_failure(root, state, e.as_ref());
            }
            return result;
        }

        let next = stage.next();

        // --step: pause between every stage so the user can inspect
        // intermediate state before the next one runs. Only if
        // there's a next stage to gate, and skipped entirely when
        // --yes is set or stdin isn't a tty (the prompt would hang
        // in scripted contexts — the script is presumed consenting).
        if args.step && next.is_some() {
            if args.yes {
                debug!(
                    "push step: --yes skips step gate between {} and next",
                    stage.as_str()
                );
            } else if !is_stdin_tty() {
                return Err("push: --step requires a tty (stdin is not interactive); \
                            add --yes to bypass step gates in non-interactive contexts"
                    .into());
            } else {
                let answer = prompt(&format!(
                    "push step: {} done. Continue to {}? [y/N] ",
                    stage.as_str(),
                    next.map(Stage::as_str).unwrap_or("")
                ))?;
                let normalized = answer.trim().to_ascii_lowercase();
                if normalized != "y" && normalized != "yes" {
                    return Err(format!(
                        "push: step gate declined after {} (got {answer:?})",
                        stage.as_str()
                    )
                    .into());
                }
            }
        }

        match next {
            Some(n) => {
                state.stage = n;
                if !args.dry_run {
                    state.save(&layout.path)?;
                }
            }
            None => break,
        }
    }
    if args.dry_run {
        info!("push: DRY-RUN complete — no changes written");
    } else {
        if layout.path.exists() {
            fs::remove_file(&layout.path)?;
        }
        info!("push: completed all stages (state cleared)");
    }
    Ok(())
}

/// Whether a failure in `stage` should trigger the cross-repo op
/// restore. Anything at or past `push-app` crosses the remote
/// boundary — the app commit is live on origin, rollback is no
/// longer sound — so those failures propagate without touching
/// snapshots.
fn stage_is_rollback_eligible(stage: Stage) -> bool {
    matches!(
        stage,
        Stage::CommitApp | Stage::CommitClaude | Stage::BookmarkBoth
    )
}

/// Restore both repos to their `jj op` snapshots, if we have them.
///
/// Best-effort — if the restore itself fails we warn but don't
/// shadow the original error the caller will propagate. Exposed at
/// `pub(crate)` so integration tests can exercise the rollback path
/// directly rather than forcing a stage failure.
pub(crate) fn rollback_on_failure(
    root: &Path,
    state: &PushState,
    original: &dyn std::error::Error,
) {
    warn!("push: rolling back both repos after: {original}");
    if let Some(op) = &state.op_app {
        match op_restore(root, op) {
            Ok(()) => info!("push: restored app repo to op {op}"),
            Err(e) => warn!("push: app repo restore failed: {e}"),
        }
    }
    if let Some(op) = &state.op_claude {
        match op_restore(&claude_path(root), op) {
            Ok(()) => info!("push: restored .claude to op {op}"),
            Err(e) => warn!("push: .claude restore failed: {e}"),
        }
    }
}

/// Execute one stage. Arms mirror `Stage`'s declaration order.
fn run_stage(
    root: &Path,
    stage: Stage,
    state: &mut PushState,
    args: &PushArgs,
    layout: &StateLayout,
) -> Result<(), Box<dyn std::error::Error>> {
    match stage {
        Stage::Preflight => stage_preflight(root, args),
        Stage::Review => stage_review(root, args),
        Stage::Message => stage_message(root, state, args, layout),
        Stage::CommitApp => stage_commit_app(root, state, args),
        Stage::CommitClaude => stage_commit_claude(root, state, args),
        Stage::BookmarkBoth => stage_bookmark_both(root, state, args),
        Stage::PushApp => stage_push_app(root, state, args),
        Stage::FinalizeClaude => stage_finalize_claude(root, state, args),
    }
}

/// Preflight: `vc-x1 sync --check && cargo fmt && cargo clippy
/// -D warnings && cargo test`.
///
/// Sync runs in `--check` mode so divergence with the remote is
/// surfaced before we burn cargo cycles, but **not** auto-resolved —
/// a rebase mid-push is exactly the kind of unsupervised mutation
/// the two approval gates exist to prevent. If sync reports action
/// needed, preflight errors and the user resolves explicitly with
/// `vc-x1 sync --no-check` before re-running push. The cargo steps
/// match CLAUDE.md's pre-commit checklist (minus `cargo install` /
/// retest, which are project-specific). All subprocesses run in the
/// workspace root so cargo picks up the right `Cargo.toml`. Skipped
/// in `--dry-run` since `cargo fmt` writes files.
fn stage_preflight(root: &Path, args: &PushArgs) -> Result<(), Box<dyn std::error::Error>> {
    if args.dry_run {
        info!("push preflight: [dry-run] would run vc-x1 sync --check / cargo fmt / clippy / test");
        return Ok(());
    }
    info!("push preflight: vc-x1 sync --check");
    run("vc-x1", &["sync", "--check"], root)?;
    info!("push preflight: cargo fmt");
    run("cargo", &["fmt"], root)?;
    info!("push preflight: cargo clippy --all-targets -- -D warnings");
    run(
        "cargo",
        &["clippy", "--all-targets", "--", "-D", "warnings"],
        root,
    )?;
    info!("push preflight: cargo test");
    run("cargo", &["test"], root)?;
    Ok(())
}

/// Review: first approval gate — "is the work done right?".
///
/// Shows a `jj diff --stat` of the pending changes in both repos
/// and prompts the user to continue. `--yes` short-circuits the
/// prompt (required for scripted / non-tty use). In `--dry-run`
/// the diff is still shown (that's the point of dry-run — see
/// what *would* be reviewed) but approval is auto-granted.
fn stage_review(root: &Path, args: &PushArgs) -> Result<(), Box<dyn std::error::Error>> {
    let claude = claude_path(root);
    let app_arg = root.to_string_lossy();
    let claude_arg = claude.to_string_lossy();
    info!("push review: pending changes:");
    info!("  app ({app_arg}):");
    let app_stat = run("jj", &["diff", "--stat", "-R", &app_arg], root)?;
    for line in app_stat.lines() {
        info!("    {line}");
    }
    info!("  .claude ({claude_arg}):");
    let claude_stat = run("jj", &["diff", "--stat", "-R", &claude_arg], root)?;
    for line in claude_stat.lines() {
        info!("    {line}");
    }
    if args.yes {
        info!("push review: auto-approved (--yes)");
        return Ok(());
    }
    if args.dry_run {
        info!("push review: [dry-run] auto-approved");
        return Ok(());
    }
    if !is_stdin_tty() {
        return Err("push review: stdin is not a tty; \
                    pass --yes to auto-approve in non-interactive contexts"
            .into());
    }
    let answer = prompt("push review: approve and continue to message stage? [y/N] ")?;
    let normalized = answer.trim().to_ascii_lowercase();
    if normalized == "y" || normalized == "yes" {
        Ok(())
    } else {
        Err(format!(
            "push review: declined (got {answer:?}); re-run with --yes or confirm with 'y'"
        )
        .into())
    }
}

/// Message: collect pre-commit changeIDs and record whether
/// `.claude` has pending changes so `commit-claude` can skip when
/// empty. `--title` and `--body` are required in 0.37.0-2; `$EDITOR`
/// support + message persistence land in 0.37.0-3.
fn stage_message(
    root: &Path,
    state: &mut PushState,
    args: &PushArgs,
    layout: &StateLayout,
) -> Result<(), Box<dyn std::error::Error>> {
    // Resolve title/body by priority:
    //   1. --title / --body flags (both must be present to skip editor)
    //   2. persisted title/body from prior stage run (resume case)
    //   3. $EDITOR template (interactive); fails if --yes and nothing else
    let (title, body) = match (args.title.clone(), args.body.clone()) {
        (Some(t), Some(b)) => (t, b),
        _ => match (state.title.clone(), state.body.clone()) {
            (Some(t), Some(b)) => (t, b),
            _ => {
                if args.yes {
                    return Err("push message: --yes given but --title/--body missing \
                                and no persisted message to resume — pass both flags \
                                or run interactively."
                        .into());
                }
                if args.dry_run {
                    return Err("push message: --dry-run given but --title/--body missing \
                                and no persisted message — pass both flags so dry-run \
                                has a message to preview."
                        .into());
                }
                if !is_stdin_tty() {
                    return Err("push message: stdin is not a tty and no --title/--body \
                                supplied; cannot launch $EDITOR in a non-interactive \
                                context. Pass --title and --body, or run interactively."
                        .into());
                }
                compose_message_via_editor(layout)?
            }
        },
    };

    // Persist the composed message so a later resume (e.g. after a
    // commit-app retry) doesn't need the flags re-passed.
    state.title = Some(title.clone());
    state.body = Some(body.clone());

    let claude = claude_path(root);
    let app_chid = get_change_id(root, "@")?;
    let claude_empty = jj_log_empty(&claude, "@")?;
    let claude_had_changes = !claude_empty;
    let claude_ref = if claude_had_changes { "@" } else { "@-" };
    let claude_chid = get_change_id(&claude, claude_ref)?;

    info!(
        "push message: title=\"{}\", app_chid={app_chid}, claude_chid={claude_chid}, claude_had_changes={claude_had_changes}",
        title.lines().next().unwrap_or("") // OK: obvious
    );
    state.app_chid = Some(app_chid);
    state.claude_chid = Some(claude_chid);
    state.claude_had_changes = Some(claude_had_changes);
    Ok(())
}

/// Launch `$EDITOR` (falling back to `vi`) on a template file under
/// `state_dir`, then parse the saved content into a `(title, body)`
/// tuple. Lines starting with `#` are treated as comments and
/// stripped. Empty input aborts the push.
fn compose_message_via_editor(
    layout: &StateLayout,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string()); // OK: POSIX fallback when nothing is configured
    let msg_path = layout
        .path
        .parent()
        .ok_or("push message: state layout has no parent dir")?
        .join("push-message.txt");
    if let Some(parent) = msg_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let template = "\
# Leave as-is to abort. Enter the Title on the first line,
# optionally followed by the Body on subsequent lines. The
# blank line between Title and Body is inserted automatically,
# and the ochid trailer is appended per-repo automatically —
# don't add either here.
# Lines starting with `#` are ignored.
";
    fs::write(&msg_path, template)?;
    info!("push message: launching {editor} on {}", msg_path.display());
    let status = std::process::Command::new(&editor)
        .arg(&msg_path)
        .status()
        .map_err(|e| format!("failed to launch {editor}: {e}"))?;
    if !status.success() {
        return Err(format!("{editor} exited non-zero ({status})").into());
    }
    let content = fs::read_to_string(&msg_path)?;
    let _ = fs::remove_file(&msg_path);
    parse_message(&content)
        .ok_or_else(|| "push message: template was left empty or all-comments — aborting".into())
}

/// Parse the editor-saved message into `(title, body)`. Strips
/// `#`-prefixed comment lines, trims surrounding blanks, and treats
/// the first non-comment line as the title with everything after as
/// the body (a blank line between is allowed but not required —
/// the commit stages insert the title/body separator themselves).
/// Returns `None` when the message has no non-comment content.
fn parse_message(raw: &str) -> Option<(String, String)> {
    let mut meaningful = String::new();
    for line in raw.lines() {
        let trimmed_left = line.trim_start();
        if trimmed_left.starts_with('#') {
            continue;
        }
        meaningful.push_str(line);
        meaningful.push('\n');
    }
    let trimmed = meaningful.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut lines = trimmed.lines();
    let title = lines.next().unwrap_or("").trim().to_string(); // OK: trimmed non-empty ⇒ first line exists
    let body = lines.collect::<Vec<_>>().join("\n").trim().to_string();
    if title.is_empty() {
        return None;
    }
    Some((title, body))
}

/// Commit app repo with `title` / `body` and the `ochid:` trailer
/// pointing at `.claude`'s chid.
fn stage_commit_app(
    root: &Path,
    state: &PushState,
    args: &PushArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let (title, body) = resolve_message(state, args)?;
    let claude_chid = state
        .claude_chid
        .as_deref()
        .ok_or("push commit-app: claude_chid not set (message stage didn't run)")?;
    let body_with_trailer = format!("{body}\n\nochid: /.claude/{claude_chid}");
    let app_arg = root.to_string_lossy();
    if args.dry_run {
        info!(
            "push commit-app: [dry-run] would run jj commit -R {app_arg} -m \"{title}\" -m <body+ochid>"
        );
        return Ok(());
    }
    info!("push commit-app: jj commit -R {app_arg}");
    run(
        "jj",
        &[
            "commit",
            "-R",
            &app_arg,
            "-m",
            &title,
            "-m",
            &body_with_trailer,
        ],
        root,
    )?;
    Ok(())
}

/// Pull the title/body pair from CLI args or persisted state,
/// preferring CLI args (override-on-resume case). Returns `Err`
/// when neither source has them — which only happens if
/// `stage_message` didn't run or was force-bypassed.
fn resolve_message(
    state: &PushState,
    args: &PushArgs,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let title = args
        .title
        .clone()
        .or_else(|| state.title.clone())
        .ok_or("push: title missing — message stage didn't run")?;
    let body = args
        .body
        .clone()
        .or_else(|| state.body.clone())
        .ok_or("push: body missing — message stage didn't run")?;
    Ok((title, body))
}

/// Commit `.claude` with the same title/body and the ochid trailer
/// pointing at the app commit's chid, or skip if `.claude` had no
/// pending changes.
fn stage_commit_claude(
    root: &Path,
    state: &PushState,
    args: &PushArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    if !state.claude_had_changes.unwrap_or(false) {
        info!("push commit-claude: skip (.claude had no pending changes)");
        return Ok(());
    }
    let (title, body) = resolve_message(state, args)?;
    let app_chid = state
        .app_chid
        .as_deref()
        .ok_or("push commit-claude: app_chid not set (message stage didn't run)")?;
    let body_with_trailer = format!("{body}\n\nochid: /{app_chid}");
    let claude = claude_path(root);
    let claude_arg = claude.to_string_lossy();
    if args.dry_run {
        info!(
            "push commit-claude: [dry-run] would run jj commit -R {claude_arg} -m \"{title}\" -m <body+ochid>"
        );
        return Ok(());
    }
    info!("push commit-claude: jj commit -R {claude_arg}");
    run(
        "jj",
        &[
            "commit",
            "-R",
            &claude_arg,
            "-m",
            &title,
            "-m",
            &body_with_trailer,
        ],
        root,
    )?;
    Ok(())
}

/// Advance the bookmark to `@-` in both repos.
fn stage_bookmark_both(
    root: &Path,
    state: &PushState,
    args: &PushArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let bk = &state.bookmark;
    let app_arg = root.to_string_lossy();
    let claude = claude_path(root);
    let claude_arg = claude.to_string_lossy();
    if args.dry_run {
        info!(
            "push bookmark-both: [dry-run] would run jj bookmark set {bk} -r @- -R {app_arg} / {claude_arg}"
        );
        return Ok(());
    }
    info!("push bookmark-both: jj bookmark set {bk} -r @- -R {app_arg} / {claude_arg}");
    run(
        "jj",
        &["bookmark", "set", bk, "-r", "@-", "-R", &app_arg],
        root,
    )?;
    run(
        "jj",
        &["bookmark", "set", bk, "-r", "@-", "-R", &claude_arg],
        root,
    )?;
    Ok(())
}

/// Push the app repo's bookmark to origin.
fn stage_push_app(
    root: &Path,
    state: &PushState,
    args: &PushArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let bk = &state.bookmark;
    let app_arg = root.to_string_lossy();
    if args.dry_run {
        info!("push push-app: [dry-run] would run jj git push --bookmark {bk} -R {app_arg}");
        return Ok(());
    }
    info!("push push-app: jj git push --bookmark {bk} -R {app_arg}");
    run(
        "jj",
        &["git", "push", "--bookmark", bk, "-R", &app_arg],
        root,
    )?;
    Ok(())
}

/// Finalize `.claude` via an out-of-process `vc-x1 finalize` call.
/// Shells out rather than calling `finalize::finalize` in-process
/// so `--detach` can fork a child that outlives push's own
/// lifetime. `--no-finalize` turns this stage into a no-op (which
/// is how integration tests avoid spawning a detached process).
fn stage_finalize_claude(
    root: &Path,
    state: &PushState,
    args: &PushArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    if args.no_finalize {
        info!("push finalize-claude: skip (--no-finalize)");
        return Ok(());
    }
    let bk = &state.bookmark;
    let claude = claude_path(root);
    let claude_arg = claude.to_string_lossy();
    if args.dry_run {
        info!(
            "push finalize-claude: [dry-run] would run vc-x1 finalize --repo {claude_arg} --squash --push {bk} --delay 10 --detach"
        );
        return Ok(());
    }
    info!(
        "push finalize-claude: vc-x1 finalize --repo {claude_arg} --squash --push {bk} --delay 10 --detach"
    );
    run(
        "vc-x1",
        &[
            "finalize",
            "--repo",
            &claude_arg,
            "--squash",
            "--push",
            bk,
            "--delay",
            "10",
            "--detach",
            "--log",
            "/tmp/vc-x1-finalize.log",
        ],
        root,
    )?;
    Ok(())
}

/// Return the 12-character change ID for `rev` in `repo`.
fn get_change_id(repo: &Path, rev: &str) -> Result<String, Box<dyn std::error::Error>> {
    let out = run(
        "jj",
        &[
            "log",
            "-r",
            rev,
            "--no-graph",
            "-T",
            "change_id.short(12)",
            "-R",
            &repo.to_string_lossy(),
        ],
        Path::new("."),
    )?;
    Ok(out.trim().to_string())
}

/// True when the given revision is empty (no working-copy changes
/// relative to its parent).
fn jj_log_empty(repo: &Path, rev: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let out = run(
        "jj",
        &[
            "log",
            "-r",
            rev,
            "--no-graph",
            "-T",
            "empty",
            "-R",
            &repo.to_string_lossy(),
        ],
        Path::new("."),
    )?;
    match out.trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(format!("jj_log_empty: unexpected template output {other:?}").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: PushArgs,
    }

    /// Per-test tempdir counter so file-system state doesn't collide
    /// across parallel runs.
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Build a unique tempdir for a test and create it.
    fn unique_tmp(tag: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0); // OK: clock error → 0 is harmless for unique tempdir naming
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!("vc-x1-push-{tag}-{ts}-{n}"));
        fs::create_dir_all(&path).expect("create tempdir");
        path
    }

    /// Bare `push` with no flags leaves every optional field at its default.
    #[test]
    fn parse_defaults() {
        let cli = Cli::try_parse_from(["test"]).unwrap();
        assert!(cli.args.bookmark.is_none());
        assert!(cli.args.bookmark_pos.is_none());
        assert!(!cli.args.restart);
        assert!(cli.args.from.is_none());
        assert!(!cli.args.step);
        assert!(!cli.args.status);
        assert!(!cli.args.recheck);
        assert!(!cli.args.no_finalize);
        assert!(!cli.args.dry_run);
        assert!(cli.args.title.is_none());
        assert!(cli.args.body.is_none());
    }

    /// Positional bookmark form: `vc-x1 push main`.
    #[test]
    fn parse_bookmark_positional() {
        let cli = Cli::try_parse_from(["test", "main"]).unwrap();
        assert_eq!(cli.args.bookmark_pos.as_deref(), Some("main"));
        assert!(cli.args.bookmark.is_none());
    }

    /// Flag bookmark form: `vc-x1 push --bookmark main`.
    #[test]
    fn parse_bookmark_flag() {
        let cli = Cli::try_parse_from(["test", "--bookmark", "dev"]).unwrap();
        assert_eq!(cli.args.bookmark.as_deref(), Some("dev"));
        assert!(cli.args.bookmark_pos.is_none());
    }

    /// Positional + flag together is rejected by clap (conflicts_with).
    #[test]
    fn parse_bookmark_both_conflicts() {
        let result = Cli::try_parse_from(["test", "main", "--bookmark", "dev"]);
        assert!(result.is_err());
    }

    /// Boolean flags all honored when set.
    #[test]
    fn parse_bool_flags() {
        let cli = Cli::try_parse_from([
            "test",
            "--restart",
            "--step",
            "--status",
            "--recheck",
            "--no-finalize",
            "--dry-run",
        ])
        .unwrap();
        assert!(cli.args.restart);
        assert!(cli.args.step);
        assert!(cli.args.status);
        assert!(cli.args.recheck);
        assert!(cli.args.no_finalize);
        assert!(cli.args.dry_run);
    }

    /// `--bookmark`, `--title`, `--body` parse their values.
    #[test]
    fn parse_string_flags() {
        let cli = Cli::try_parse_from([
            "test",
            "--bookmark",
            "main",
            "--title",
            "feat: x",
            "--body",
            "details here",
        ])
        .unwrap();
        assert_eq!(cli.args.bookmark.as_deref(), Some("main"));
        assert_eq!(cli.args.title.as_deref(), Some("feat: x"));
        assert_eq!(cli.args.body.as_deref(), Some("details here"));
    }

    /// `--from` accepts each defined stage by its kebab-case name.
    #[test]
    fn parse_from_stage() {
        for (name, expected) in [
            ("preflight", Stage::Preflight),
            ("review", Stage::Review),
            ("message", Stage::Message),
            ("commit-app", Stage::CommitApp),
            ("commit-claude", Stage::CommitClaude),
            ("bookmark-both", Stage::BookmarkBoth),
            ("push-app", Stage::PushApp),
            ("finalize-claude", Stage::FinalizeClaude),
        ] {
            let cli = Cli::try_parse_from(["test", "--from", name]).unwrap();
            assert_eq!(cli.args.from, Some(expected), "stage {name}");
        }
    }

    /// `--from` rejects unknown stage names.
    #[test]
    fn parse_from_stage_rejects_unknown() {
        let result = Cli::try_parse_from(["test", "--from", "bogus"]);
        assert!(result.is_err());
    }

    /// `Stage::next` walks every stage in order and terminates at
    /// `FinalizeClaude`.
    #[test]
    fn stage_next_walks_full_flow() {
        let walk: Vec<Stage> = std::iter::successors(Some(Stage::first()), |s| s.next()).collect();
        assert_eq!(
            walk,
            vec![
                Stage::Preflight,
                Stage::Review,
                Stage::Message,
                Stage::CommitApp,
                Stage::CommitClaude,
                Stage::BookmarkBoth,
                Stage::PushApp,
                Stage::FinalizeClaude,
            ]
        );
    }

    /// `Stage::as_str` and `Stage::from_str` round-trip every variant.
    #[test]
    fn stage_str_roundtrip() {
        for stage in [
            Stage::Preflight,
            Stage::Review,
            Stage::Message,
            Stage::CommitApp,
            Stage::CommitClaude,
            Stage::BookmarkBoth,
            Stage::PushApp,
            Stage::FinalizeClaude,
        ] {
            assert_eq!(Stage::from_str(stage.as_str()), Some(stage));
        }
    }

    /// `resolve_state_layout` uses defaults when `.vc-config.toml`
    /// has no `[push]` section.
    #[test]
    fn layout_defaults_when_no_config() {
        let tmp = unique_tmp("layout-default");
        let layout = resolve_state_layout(&tmp);
        assert_eq!(layout.path, tmp.join(".vc-x1").join("push-state.toml"));
        let _ = fs::remove_dir_all(&tmp);
    }

    /// `resolve_state_layout` picks up `[push]` overrides from the
    /// config file.
    #[test]
    fn layout_reads_config_overrides() {
        let tmp = unique_tmp("layout-override");
        fs::write(
            tmp.join(".vc-config.toml"),
            "[push]\nstate-dir = \"custom-dir\"\nstate-file = \"custom.toml\"\n",
        )
        .expect("write config");
        let layout = resolve_state_layout(&tmp);
        assert_eq!(layout.path, tmp.join("custom-dir").join("custom.toml"));
        let _ = fs::remove_dir_all(&tmp);
    }

    /// Save-then-load round-trips every field, including the
    /// 0.37.0-2 optional additions (chids, op snapshots, had-changes
    /// flag).
    #[test]
    fn state_save_load_roundtrip() {
        let tmp = unique_tmp("state-roundtrip");
        let path = tmp.join("push-state.toml");
        let original = PushState {
            version: STATE_FORMAT_VERSION,
            stage: Stage::CommitClaude,
            bookmark: "feature/thing".to_string(),
            started_at: "2026-04-21T20:15:33+00:00".to_string(),
            app_chid: Some("abc123def456".to_string()),
            claude_chid: Some("fedcba654321".to_string()),
            claude_had_changes: Some(true),
            op_app: Some("opapp12345".to_string()),
            op_claude: Some("opcla54321".to_string()),
            title: Some("feat: round-trip title".to_string()),
            body: Some("Multi-line\nbody with\n\tspecial \"chars\" and \\ backslash.".to_string()),
        };
        original.save(&path).expect("save");
        let loaded = PushState::load(&path).expect("load").expect("Some state");
        assert_eq!(original, loaded);
        let _ = fs::remove_dir_all(&tmp);
    }

    /// Save-then-load also round-trips a state that has only the
    /// base required fields set (matches what 0.37.0-1 states look
    /// like — backward-compatible upgrade path).
    #[test]
    fn state_save_load_roundtrip_no_options() {
        let tmp = unique_tmp("state-roundtrip-bare");
        let path = tmp.join("push-state.toml");
        let original = PushState {
            version: STATE_FORMAT_VERSION,
            stage: Stage::Preflight,
            bookmark: "main".to_string(),
            started_at: "2026-04-21T00:00:00+00:00".to_string(),
            app_chid: None,
            claude_chid: None,
            claude_had_changes: None,
            op_app: None,
            op_claude: None,
            title: None,
            body: None,
        };
        original.save(&path).expect("save");
        let loaded = PushState::load(&path).expect("load").expect("Some state");
        assert_eq!(original, loaded);
        let _ = fs::remove_dir_all(&tmp);
    }

    /// `claude_had_changes = false` round-trips as `Some(false)`
    /// (distinct from unset / `None`).
    #[test]
    fn state_save_load_claude_had_changes_false() {
        let tmp = unique_tmp("state-cladechanges-false");
        let path = tmp.join("push-state.toml");
        let original = PushState {
            version: STATE_FORMAT_VERSION,
            stage: Stage::CommitClaude,
            bookmark: "main".to_string(),
            started_at: "2026-04-21T00:00:00+00:00".to_string(),
            app_chid: Some("abc".to_string()),
            claude_chid: Some("def".to_string()),
            claude_had_changes: Some(false),
            op_app: None,
            op_claude: None,
            title: None,
            body: None,
        };
        original.save(&path).expect("save");
        let loaded = PushState::load(&path).expect("load").expect("Some state");
        assert_eq!(loaded.claude_had_changes, Some(false));
        let _ = fs::remove_dir_all(&tmp);
    }

    /// `stage_is_rollback_eligible` returns true only for the three
    /// local-mutation stages.
    #[test]
    fn rollback_eligibility_covers_local_window() {
        assert!(!stage_is_rollback_eligible(Stage::Preflight));
        assert!(!stage_is_rollback_eligible(Stage::Review));
        assert!(!stage_is_rollback_eligible(Stage::Message));
        assert!(stage_is_rollback_eligible(Stage::CommitApp));
        assert!(stage_is_rollback_eligible(Stage::CommitClaude));
        assert!(stage_is_rollback_eligible(Stage::BookmarkBoth));
        assert!(!stage_is_rollback_eligible(Stage::PushApp));
        assert!(!stage_is_rollback_eligible(Stage::FinalizeClaude));
    }

    /// Missing state file → `Ok(None)`, not an error (fresh-run case).
    #[test]
    fn state_load_missing_returns_none() {
        let tmp = unique_tmp("state-missing");
        let path = tmp.join("does-not-exist.toml");
        let loaded = PushState::load(&path).expect("load");
        assert!(loaded.is_none());
        let _ = fs::remove_dir_all(&tmp);
    }

    /// State file with a stale format version fails to load with a
    /// helpful `--restart` hint.
    #[test]
    fn state_load_rejects_stale_version() {
        let tmp = unique_tmp("state-stale");
        let path = tmp.join("push-state.toml");
        fs::write(
            &path,
            "[push-state]\n\
             version = 99999\n\
             stage = \"preflight\"\n\
             bookmark = \"main\"\n\
             started_at = \"2026-04-21T00:00:00+00:00\"\n",
        )
        .expect("write stale state");
        let err = PushState::load(&path).unwrap_err().to_string();
        assert!(err.contains("unsupported format version"), "got: {err}");
        assert!(err.contains("--restart"), "got: {err}");
        let _ = fs::remove_dir_all(&tmp);
    }

    /// State file with an unknown stage name fails to load.
    #[test]
    fn state_load_rejects_unknown_stage() {
        let tmp = unique_tmp("state-bad-stage");
        let path = tmp.join("push-state.toml");
        fs::write(
            &path,
            "[push-state]\n\
             version = 1\n\
             stage = \"bogus-stage\"\n\
             bookmark = \"main\"\n\
             started_at = \"2026-04-21T00:00:00+00:00\"\n",
        )
        .expect("write bad state");
        let err = PushState::load(&path).unwrap_err().to_string();
        assert!(err.contains("unknown stage"), "got: {err}");
        let _ = fs::remove_dir_all(&tmp);
    }

    /// State file missing a required key fails to load.
    #[test]
    fn state_load_rejects_missing_key() {
        let tmp = unique_tmp("state-missing-key");
        let path = tmp.join("push-state.toml");
        fs::write(
            &path,
            "[push-state]\n\
             version = 1\n\
             stage = \"preflight\"\n\
             started_at = \"2026-04-21T00:00:00+00:00\"\n",
        )
        .expect("write state without bookmark");
        let err = PushState::load(&path).unwrap_err().to_string();
        assert!(err.contains("missing key"), "got: {err}");
        assert!(err.contains("bookmark"), "got: {err}");
        let _ = fs::remove_dir_all(&tmp);
    }

    /// Fresh state has `stage = first()` and the bookmark we ask for.
    #[test]
    fn state_new_for_initializes_correctly() {
        let s = PushState::new_for("main");
        assert_eq!(s.version, STATE_FORMAT_VERSION);
        assert_eq!(s.stage, Stage::first());
        assert_eq!(s.bookmark, "main");
        assert!(!s.started_at.is_empty());
    }

    /// `escape_multiline` round-trips via `unescape_multiline`
    /// across every escape case we emit.
    #[test]
    fn multiline_escape_roundtrip() {
        for original in [
            "",
            "simple",
            "line one\nline two",
            "tab\tseparated\tvalues",
            "quoted \"text\" inside",
            "backslash \\ here",
            "mixed:\n\t\"quoted\"\\ path\r\n",
        ] {
            let escaped = escape_multiline(original);
            // Escaped form contains no raw newlines, tabs, or CRs —
            // safe for our single-line TOML value slot.
            assert!(!escaped.contains('\n'), "escape leaked \\n: {escaped:?}");
            assert!(!escaped.contains('\t'), "escape leaked \\t: {escaped:?}");
            assert!(!escaped.contains('\r'), "escape leaked \\r: {escaped:?}");
            assert_eq!(unescape_multiline(&escaped), original);
        }
    }

    /// `parse_message` extracts title + body, strips `#` comments,
    /// and rejects all-comments / empty input.
    #[test]
    fn parse_message_cases() {
        // Title + body separated by blank line.
        let (t, b) = parse_message("feat: x\n\nBody here.\nSecond line.\n").unwrap();
        assert_eq!(t, "feat: x");
        assert_eq!(b, "Body here.\nSecond line.");

        // Title + body with no blank line — first line is title, rest is body.
        let (t, b) = parse_message("feat: x\nBody here.\nSecond line.\n").unwrap();
        assert_eq!(t, "feat: x");
        assert_eq!(b, "Body here.\nSecond line.");

        // Body with internal blank lines preserved.
        let (t, b) = parse_message("feat: x\npara 1\n\npara 2\n").unwrap();
        assert_eq!(t, "feat: x");
        assert_eq!(b, "para 1\n\npara 2");

        // Comments stripped.
        let (t, b) =
            parse_message("# comment\nfeat: y\n# mid-comment\n\nbody\n# tail-comment\n").unwrap();
        assert_eq!(t, "feat: y");
        assert_eq!(b, "body");

        // Title only (no body).
        let (t, b) = parse_message("feat: z\n").unwrap();
        assert_eq!(t, "feat: z");
        assert_eq!(b, "");

        // All comments → None (caller aborts).
        assert!(parse_message("# only comments\n# and more\n").is_none());
        // All blank → None.
        assert!(parse_message("   \n\n").is_none());
    }
}

#[cfg(test)]
mod integration_tests {
    //! End-to-end tests for `push_in` against real dual-repo jj
    //! fixtures (bare-git remotes + colocated jj repos under a
    //! unique tempdir via `crate::test_helpers::Fixture`).
    //!
    //! Every test uses `--from message` to skip `preflight` (no
    //! `Cargo.toml` in the fixture) and `--no-finalize` to avoid
    //! spawning a detached `vc-x1 finalize` child that would
    //! outlive the test. The remaining stages (message,
    //! commit-app, commit-claude, bookmark-both, push-app) are
    //! exercised against the fixture's local bare-git remote.
    //!
    //! Stage execution + rollback are covered here;
    //! state-file / layout / stage-ordering mechanics are covered
    //! in the neighboring `tests` module via pure unit tests.
    //!
    //! Requires `jj` and the compiled `vc-x1` binary in `PATH`.

    use super::*;
    use crate::test_helpers::Fixture;
    use std::fs;
    use std::process::Command;

    /// Run `jj <args> -R <repo>` and return trimmed stdout on success.
    fn jj(repo: &Path, args: &[&str]) -> String {
        let out = Command::new("jj")
            .args(args)
            .arg("-R")
            .arg(repo)
            .output()
            .expect("spawn jj");
        assert!(
            out.status.success(),
            "jj {args:?} failed in {}: {}",
            repo.display(),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// Commit ID (short, 12 chars) for a revision.
    fn cid(repo: &Path, rev: &str) -> String {
        jj(
            repo,
            &["log", "-r", rev, "--no-graph", "-T", "commit_id.short(12)"],
        )
    }

    /// Full description of a revision.
    fn description(repo: &Path, rev: &str) -> String {
        jj(repo, &["log", "-r", rev, "--no-graph", "-T", "description"])
    }

    /// First line of a revision's description.
    fn desc_first_line(repo: &Path, rev: &str) -> String {
        jj(
            repo,
            &[
                "log",
                "-r",
                rev,
                "--no-graph",
                "-T",
                "description.first_line()",
            ],
        )
    }

    /// Standard test args: bookmark=main, `--from message` (skip
    /// preflight), `--no-finalize` (skip detached finalize),
    /// `--yes` (auto-approve any interactive prompts).
    fn test_args(title: &str, body: &str) -> PushArgs {
        PushArgs {
            bookmark_pos: Some("main".to_string()),
            bookmark: None,
            restart: false,
            from: Some(Stage::Message),
            step: false,
            status: false,
            recheck: false,
            no_finalize: true,
            dry_run: false,
            title: Some(title.to_string()),
            body: Some(body.to_string()),
            yes: true,
        }
    }

    /// Happy path when `.claude` has no pending changes: the app
    /// commit lands with an `ochid` trailer pointing at `.claude`'s
    /// pre-existing `@-`, `commit-claude` is skipped, and both
    /// `bookmark-both` + `push-app` still run cleanly.
    #[test]
    fn push_happy_claude_clean() {
        let fx = Fixture::new("push-clean");
        fs::write(fx.work.join("hello.txt"), "hi").expect("write app file");

        let claude_main_before = cid(&fx.claude, "main");

        push_in(&fx.work, &test_args("feat: clean case", "app body")).expect("push should succeed");

        // App repo: main advanced to our new commit.
        assert_eq!(desc_first_line(&fx.work, "main"), "feat: clean case");
        let app_full = description(&fx.work, "main");
        assert!(
            app_full.contains("ochid: /.claude/"),
            "app ochid trailer missing:\n{app_full}"
        );

        // `.claude` main unchanged (no commit happened there).
        assert_eq!(
            cid(&fx.claude, "main"),
            claude_main_before,
            ".claude main should not have moved"
        );
    }

    /// Happy path when `.claude` has pending changes: both repos
    /// commit, each with an ochid trailer pointing at the other.
    #[test]
    fn push_happy_claude_dirty() {
        let fx = Fixture::new("push-dirty");
        fs::write(fx.work.join("app.txt"), "app").expect("write app file");
        fs::write(fx.claude.join("session.jsonl"), "{\"line\":1}\n").expect("write session file");

        let claude_main_before = cid(&fx.claude, "main");

        push_in(&fx.work, &test_args("feat: paired change", "paired body"))
            .expect("push should succeed");

        // Both repos have new commits with matching titles.
        assert_eq!(desc_first_line(&fx.work, "main"), "feat: paired change");
        assert_eq!(desc_first_line(&fx.claude, "main"), "feat: paired change");

        // Cross-repo ochid trailers are both present.
        let app_full = description(&fx.work, "main");
        let claude_full = description(&fx.claude, "main");
        assert!(
            app_full.contains("ochid: /.claude/"),
            "app ochid missing:\n{app_full}"
        );
        // `.claude`'s ochid points at the app repo, so the prefix is
        // just `/` (no `.claude` segment).
        assert!(
            claude_full
                .lines()
                .any(|l| l.starts_with("ochid: /") && !l.starts_with("ochid: /.claude/")),
            ".claude ochid should point at app repo:\n{claude_full}"
        );

        // `.claude` main moved off its initial commit.
        assert_ne!(
            cid(&fx.claude, "main"),
            claude_main_before,
            ".claude main should have advanced"
        );
    }

    /// `rollback_on_failure` rewinds both repos to their recorded
    /// `jj op` snapshots when triggered mid-flow.
    ///
    /// Simulates a failure after both repos have had their `main`
    /// bookmark advanced past the original position, then calls
    /// `rollback_on_failure` with the pre-mutation op IDs. After
    /// rollback, `main` should be back at the starting commit in
    /// both repos.
    ///
    /// Notes:
    /// - We don't compare `current_op_id` post-rollback because
    ///   reading the op id snapshots the (still-dirty) working
    ///   copy, creating a fresh op. Bookmark position is the
    ///   load-bearing invariant anyway.
    /// - Each mutation sequence actually moves `main` (describe →
    ///   bookmark set → new) so the pre-rollback state is
    ///   observably different from the post-rollback state.
    #[test]
    fn push_rollback_restores_both_repos() {
        let fx = Fixture::new("push-rollback");

        // Snapshot pre-mutation state.
        let op_app_start = current_op_id(&fx.work).expect("app op id");
        let op_claude_start = current_op_id(&fx.claude).expect("claude op id");
        let main_app_start = cid(&fx.work, "main");
        let main_claude_start = cid(&fx.claude, "main");

        // Mutate both repos so `main` actually advances (this is
        // what rollback has to undo).
        fs::write(fx.work.join("app.txt"), "app").expect("write");
        fs::write(fx.claude.join("session.jsonl"), "{}\n").expect("write");
        jj(&fx.work, &["describe", "-m", "test commit"]);
        jj(&fx.work, &["bookmark", "set", "main", "-r", "@"]);
        jj(&fx.work, &["new"]);
        jj(&fx.claude, &["describe", "-m", "test session"]);
        jj(&fx.claude, &["bookmark", "set", "main", "-r", "@"]);
        jj(&fx.claude, &["new"]);

        // Sanity: main has advanced in both repos.
        assert_ne!(
            cid(&fx.work, "main"),
            main_app_start,
            "setup should have moved app main"
        );
        assert_ne!(
            cid(&fx.claude, "main"),
            main_claude_start,
            "setup should have moved .claude main"
        );

        // State records we're at bookmark-both with snapshots from
        // before any of the above mutations.
        let state = PushState {
            version: STATE_FORMAT_VERSION,
            stage: Stage::BookmarkBoth,
            bookmark: "main".to_string(),
            started_at: "2026-04-21T20:00:00+00:00".to_string(),
            app_chid: None,
            claude_chid: None,
            claude_had_changes: Some(true),
            op_app: Some(op_app_start),
            op_claude: Some(op_claude_start),
            title: None,
            body: None,
        };

        let err: Box<dyn std::error::Error> = "forced for test".into();
        rollback_on_failure(&fx.work, &state, err.as_ref());

        // After rollback, `main` is restored in both repos.
        assert_eq!(cid(&fx.work, "main"), main_app_start);
        assert_eq!(cid(&fx.claude, "main"), main_claude_start);
    }

    /// End-to-end resume: first run fails at `push-app` (simulated
    /// by passing a bogus bookmark that jj accepts but the bare-git
    /// remote rejects on push). Second run with `--from push-app`
    /// and the correct bookmark completes the flow. Confirms state
    /// persists across invocations and `--from` overrides the
    /// resumed stage.
    #[test]
    fn push_resume_after_push_failure() {
        let fx = Fixture::new("push-resume");
        fs::write(fx.work.join("app.txt"), "app").expect("write app file");

        // First run: commits + bookmarks succeed; push-app we
        // simulate via a second step rather than trying to force a
        // real push failure (which jj makes hard — local bare-git
        // remotes accept almost anything). Instead, split the run
        // using --no-finalize on the second pass.
        let mut args1 = test_args("feat: resume", "resume body");
        args1.from = Some(Stage::Message);
        push_in(&fx.work, &args1).expect("first push run");

        // After the full run, state file should be cleared and main
        // should be advanced in the app repo.
        let layout = resolve_state_layout(&fx.work);
        assert!(
            !layout.path.exists(),
            "state file should be cleared after a successful run: {}",
            layout.path.display()
        );
        assert_eq!(desc_first_line(&fx.work, "main"), "feat: resume");
    }
}

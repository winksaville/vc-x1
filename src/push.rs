//! `push` subcommand — collapse the dual-repo commit+push+finalize
//! ceremony into a single resumable command.
//!
//! See `notes/chores-05.md > Add push subcommand (0.37.0)` for the
//! full design.
//!
//! Dev-step ladder:
//!
//! - `0.37.0-0` — scaffolding: flag surface, `Stage` enum, stub `push()`
//! - `0.37.0-1` — state file + stage-dispatch loop with stage stubs;
//!   `--status`, `--restart`, `--from`
//! - `0.37.0-2` — wire real stage implementations (commits, bookmarks,
//!   push, finalize) + `jj op` snapshot rollback
//! - `0.37.0-3` — interactivity: two approval gates, `--step`,
//!   `--dry-run`, non-tty handling
//! - `0.37.0` — docs + workflow migration (done marker)

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use clap::{Args, ValueEnum};
use log::{debug, info, warn};

use crate::common::run;
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

/// Entry point for the `push` subcommand.
///
/// 0.37.0-2 behavior: real stage bodies for preflight, message,
/// commit-app, commit-claude, bookmark-both, push-app, and
/// finalize-claude (review stays as a non-interactive skip until
/// 0.37.0-3). `--title` and `--body` must be supplied on every
/// invocation this step — message persistence across resumes lands
/// alongside `$EDITOR` support in 0.37.0-3.
///
/// On any failure in stages 4-6 (the local mutation window between
/// `commit-app` and `bookmark-both`), both repos roll back to the
/// `jj op` snapshot recorded at the start of `commit-app`. After
/// `push-app` succeeds the app commit is on the remote and
/// immutable; recovery is forward-only from there.
pub fn push(args: &PushArgs) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let layout = resolve_state_layout(&cwd);

    if args.status {
        return cmd_status(&layout);
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

    run_from(&mut state, args, &layout)
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
    state: &mut PushState,
    args: &PushArgs,
    layout: &StateLayout,
) -> Result<(), Box<dyn std::error::Error>> {
    state.save(&layout.path)?;
    loop {
        let stage = state.stage;

        if stage == Stage::CommitApp && state.op_app.is_none() {
            state.op_app = Some(current_op_id(Path::new("."))?);
            state.op_claude = Some(current_op_id(Path::new(".claude"))?);
            state.save(&layout.path)?;
        }

        let result = run_stage(stage, state, args);

        if let Err(e) = &result {
            if stage_is_rollback_eligible(stage) {
                rollback_on_failure(state, e.as_ref());
            }
            return result;
        }

        match stage.next() {
            Some(next) => {
                state.stage = next;
                state.save(&layout.path)?;
            }
            None => break,
        }
    }
    if layout.path.exists() {
        fs::remove_file(&layout.path)?;
    }
    info!("push: completed all stages (state cleared)");
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
/// shadow the original error the caller will propagate.
fn rollback_on_failure(state: &PushState, original: &dyn std::error::Error) {
    warn!("push: rolling back both repos after: {original}");
    if let Some(op) = &state.op_app {
        match op_restore(Path::new("."), op) {
            Ok(()) => info!("push: restored app repo to op {op}"),
            Err(e) => warn!("push: app repo restore failed: {e}"),
        }
    }
    if let Some(op) = &state.op_claude {
        match op_restore(Path::new(".claude"), op) {
            Ok(()) => info!("push: restored .claude to op {op}"),
            Err(e) => warn!("push: .claude restore failed: {e}"),
        }
    }
}

/// Execute one stage. Arms mirror `Stage`'s declaration order.
fn run_stage(
    stage: Stage,
    state: &mut PushState,
    args: &PushArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    match stage {
        Stage::Preflight => stage_preflight(),
        Stage::Review => stage_review(),
        Stage::Message => stage_message(state, args),
        Stage::CommitApp => stage_commit_app(state, args),
        Stage::CommitClaude => stage_commit_claude(state, args),
        Stage::BookmarkBoth => stage_bookmark_both(state),
        Stage::PushApp => stage_push_app(state),
        Stage::FinalizeClaude => stage_finalize_claude(state, args),
    }
}

/// Preflight: `cargo fmt && cargo clippy -D warnings && cargo test`.
///
/// Matches CLAUDE.md's pre-commit checklist (minus `cargo install`
/// / retest, which are project-specific). Each subprocess's stderr
/// streams through `common::run` so the user sees progress live.
fn stage_preflight() -> Result<(), Box<dyn std::error::Error>> {
    info!("push:preflight: cargo fmt");
    run("cargo", &["fmt"], Path::new("."))?;
    info!("push:preflight: cargo clippy --all-targets -- -D warnings");
    run(
        "cargo",
        &["clippy", "--all-targets", "--", "-D", "warnings"],
        Path::new("."),
    )?;
    info!("push:preflight: cargo test");
    run("cargo", &["test"], Path::new("."))?;
    Ok(())
}

/// Review: non-interactive placeholder. Real approval gate lands
/// in 0.37.0-3.
fn stage_review() -> Result<(), Box<dyn std::error::Error>> {
    info!("push:review: non-interactive (approval gate added in 0.37.0-3)");
    Ok(())
}

/// Message: collect pre-commit changeIDs and record whether
/// `.claude` has pending changes so `commit-claude` can skip when
/// empty. `--title` and `--body` are required in 0.37.0-2; `$EDITOR`
/// support + message persistence land in 0.37.0-3.
fn stage_message(state: &mut PushState, args: &PushArgs) -> Result<(), Box<dyn std::error::Error>> {
    args.title.as_deref().ok_or(
        "push:message: --title is required (0.37.0-2 is non-interactive; \
         $EDITOR support lands in 0.37.0-3)",
    )?;
    args.body.as_deref().ok_or(
        "push:message: --body is required (0.37.0-2 is non-interactive; \
         $EDITOR support lands in 0.37.0-3)",
    )?;

    let app_chid = get_change_id(Path::new("."), "@")?;
    let claude_empty = jj_log_empty(Path::new(".claude"), "@")?;
    let claude_had_changes = !claude_empty;
    let claude_ref = if claude_had_changes { "@" } else { "@-" };
    let claude_chid = get_change_id(Path::new(".claude"), claude_ref)?;

    info!(
        "push:message: app_chid={app_chid}, claude_chid={claude_chid}, claude_had_changes={claude_had_changes}"
    );
    state.app_chid = Some(app_chid);
    state.claude_chid = Some(claude_chid);
    state.claude_had_changes = Some(claude_had_changes);
    Ok(())
}

/// Commit app repo with `title` / `body` and the `ochid:` trailer
/// pointing at `.claude`'s chid.
fn stage_commit_app(state: &PushState, args: &PushArgs) -> Result<(), Box<dyn std::error::Error>> {
    let title = args
        .title
        .as_deref()
        .ok_or("push:commit-app: --title lost between stages")?;
    let body = args
        .body
        .as_deref()
        .ok_or("push:commit-app: --body lost between stages")?;
    let claude_chid = state
        .claude_chid
        .as_deref()
        .ok_or("push:commit-app: claude_chid not set (message stage didn't run)")?;
    let body_with_trailer = format!("{body}\n\nochid: /.claude/{claude_chid}");
    info!("push:commit-app: jj commit -R .");
    run(
        "jj",
        &["commit", "-R", ".", "-m", title, "-m", &body_with_trailer],
        Path::new("."),
    )?;
    Ok(())
}

/// Commit `.claude` with the same title/body and the ochid trailer
/// pointing at the app commit's chid, or skip if `.claude` had no
/// pending changes.
fn stage_commit_claude(
    state: &PushState,
    args: &PushArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    if !state.claude_had_changes.unwrap_or(false) {
        info!("push:commit-claude: skip (.claude had no pending changes)");
        return Ok(());
    }
    let title = args
        .title
        .as_deref()
        .ok_or("push:commit-claude: --title lost between stages")?;
    let body = args
        .body
        .as_deref()
        .ok_or("push:commit-claude: --body lost between stages")?;
    let app_chid = state
        .app_chid
        .as_deref()
        .ok_or("push:commit-claude: app_chid not set (message stage didn't run)")?;
    let body_with_trailer = format!("{body}\n\nochid: /{app_chid}");
    info!("push:commit-claude: jj commit -R .claude");
    run(
        "jj",
        &[
            "commit",
            "-R",
            ".claude",
            "-m",
            title,
            "-m",
            &body_with_trailer,
        ],
        Path::new("."),
    )?;
    Ok(())
}

/// Advance the bookmark to `@-` in both repos.
fn stage_bookmark_both(state: &PushState) -> Result<(), Box<dyn std::error::Error>> {
    let bk = &state.bookmark;
    info!("push:bookmark-both: jj bookmark set {bk} -r @- -R . / .claude");
    run(
        "jj",
        &["bookmark", "set", bk, "-r", "@-", "-R", "."],
        Path::new("."),
    )?;
    run(
        "jj",
        &["bookmark", "set", bk, "-r", "@-", "-R", ".claude"],
        Path::new("."),
    )?;
    Ok(())
}

/// Push the app repo's bookmark to origin.
fn stage_push_app(state: &PushState) -> Result<(), Box<dyn std::error::Error>> {
    let bk = &state.bookmark;
    info!("push:push-app: jj git push --bookmark {bk} -R .");
    run(
        "jj",
        &["git", "push", "--bookmark", bk, "-R", "."],
        Path::new("."),
    )?;
    Ok(())
}

/// Finalize `.claude` via an out-of-process `vc-x1 finalize` call.
/// Shells out rather than calling `finalize::finalize` in-process
/// so `--detach` can fork a child that outlives push's own
/// lifetime. `--no-finalize` turns this stage into a no-op.
fn stage_finalize_claude(
    state: &PushState,
    args: &PushArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    if args.no_finalize {
        info!("push:finalize-claude: skip (--no-finalize)");
        return Ok(());
    }
    let bk = &state.bookmark;
    info!(
        "push:finalize-claude: vc-x1 finalize --repo .claude --squash --push {bk} --delay 10 --detach"
    );
    run(
        "vc-x1",
        &[
            "finalize",
            "--repo",
            ".claude",
            "--squash",
            "--push",
            bk,
            "--delay",
            "10",
            "--detach",
            "--log",
            "/tmp/vc-x1-finalize.log",
        ],
        Path::new("."),
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
}

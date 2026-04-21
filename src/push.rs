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
use log::{debug, info};

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
/// configured in `.vc-config.toml`'s `[push]` section. The struct is
/// intentionally small in 0.37.0-1 — real stage implementations in
/// later dev steps will add fields (op-snapshot ids, ochid decisions,
/// composed message, etc.).
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
}

impl PushState {
    /// Build a fresh state for a new run.
    pub fn new_for(bookmark: &str) -> Self {
        PushState {
            version: STATE_FORMAT_VERSION,
            stage: Stage::first(),
            bookmark: bookmark.to_string(),
            started_at: Utc::now().to_rfc3339(),
        }
    }

    /// Write the state to `path`, creating parent dirs as needed.
    ///
    /// Values are single-line and contain no `"` characters under
    /// normal operation (stage names are kebab-case, bookmark names
    /// don't contain quotes, timestamps are ASCII). A defensive
    /// escape pass replaces any stray `"` with `\"` so the file
    /// always parses with `toml_simple`.
    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = format!(
            "# vc-x1 push state — managed file, do not edit by hand\n\
             [push-state]\n\
             version = {version}\n\
             stage = \"{stage}\"\n\
             bookmark = \"{bookmark}\"\n\
             started_at = \"{started_at}\"\n",
            version = self.version,
            stage = self.stage.as_str(),
            bookmark = escape_toml(&self.bookmark),
            started_at = escape_toml(&self.started_at),
        );
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
        Ok(Some(PushState {
            version,
            stage,
            bookmark: require("push-state.bookmark")?,
            started_at: require("push-state.started_at")?,
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
/// 0.37.0-1 behavior: infrastructure only. `--status` reports
/// persisted state; `--restart` clears it; a bare `vc-x1 push` walks
/// the state machine from the resumed stage to completion, with each
/// stage logging what it *would* do. Actual stage side-effects land
/// in 0.37.0-2 onward.
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
/// 0.37.0-1 stage bodies are stubs that log `"stage X: not
/// implemented yet"` and return `Ok(())`; real bodies land across
/// subsequent dev steps. The loop itself — advance, persist, repeat —
/// is production shape.
fn run_from(
    state: &mut PushState,
    args: &PushArgs,
    layout: &StateLayout,
) -> Result<(), Box<dyn std::error::Error>> {
    state.save(&layout.path)?;
    loop {
        run_stage(state.stage, state, args)?;
        match state.stage.next() {
            Some(next) => {
                state.stage = next;
                state.save(&layout.path)?;
            }
            None => break,
        }
    }
    // Completed — clear the state file.
    if layout.path.exists() {
        fs::remove_file(&layout.path)?;
    }
    info!("push: completed all stages (state cleared)");
    Ok(())
}

/// Execute one stage.
///
/// In 0.37.0-1 every arm is a stub. The ordering of arms matches the
/// declaration order of `Stage` so future implementations slot in
/// place without reshuffling.
fn run_stage(
    stage: Stage,
    _state: &mut PushState,
    _args: &PushArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "push: stage {} (stub — not implemented yet)",
        stage.as_str()
    );
    Ok(())
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

    /// Save-then-load round-trips every field, including special
    /// characters that need escaping.
    #[test]
    fn state_save_load_roundtrip() {
        let tmp = unique_tmp("state-roundtrip");
        let path = tmp.join("push-state.toml");
        let original = PushState {
            version: STATE_FORMAT_VERSION,
            stage: Stage::CommitClaude,
            bookmark: "feature/thing".to_string(),
            started_at: "2026-04-21T20:15:33+00:00".to_string(),
        };
        original.save(&path).expect("save");
        let loaded = PushState::load(&path).expect("load").expect("Some state");
        assert_eq!(original, loaded);
        let _ = fs::remove_dir_all(&tmp);
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

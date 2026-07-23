//! Unit tests for the push module.

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
    assert!(!cli.args.no_squash_push);
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
        "--no-squash-push",
        "--dry-run",
    ])
    .unwrap();
    assert!(cli.args.restart);
    assert!(cli.args.step);
    assert!(cli.args.status);
    assert!(cli.args.recheck);
    assert!(cli.args.no_squash_push);
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
        ("commit-work", Stage::CommitWork),
        ("commit-bot", Stage::CommitBot),
        ("bookmark-set", Stage::BookmarkSet),
        ("push-work", Stage::PushWork),
        ("squash-push-bot", Stage::SquashPushBot),
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
/// `SquashPushBot`.
#[test]
fn stage_next_walks_full_flow() {
    let walk: Vec<Stage> = std::iter::successors(Some(Stage::first()), |s| s.next()).collect();
    assert_eq!(
        walk,
        vec![
            Stage::Preflight,
            Stage::Review,
            Stage::Message,
            Stage::CommitWork,
            Stage::CommitBot,
            Stage::BookmarkSet,
            Stage::PushWork,
            Stage::SquashPushBot,
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
        Stage::CommitWork,
        Stage::CommitBot,
        Stage::BookmarkSet,
        Stage::PushWork,
        Stage::SquashPushBot,
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
        stage: Stage::CommitBot,
        bookmark: "feature/thing".to_string(),
        started_at: "2026-04-21T20:15:33+00:00".to_string(),
        work_chid: Some("abc123def456".to_string()),
        bot_chid: Some("fedcba654321".to_string()),
        bot_had_changes: Some(true),
        op_work: Some("opapp12345".to_string()),
        op_bot: Some("opcla54321".to_string()),
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
        work_chid: None,
        bot_chid: None,
        bot_had_changes: None,
        op_work: None,
        op_bot: None,
        title: None,
        body: None,
    };
    original.save(&path).expect("save");
    let loaded = PushState::load(&path).expect("load").expect("Some state");
    assert_eq!(original, loaded);
    let _ = fs::remove_dir_all(&tmp);
}

/// `bot_had_changes = false` round-trips as `Some(false)`
/// (distinct from unset / `None`).
#[test]
fn state_save_load_bot_had_changes_false() {
    let tmp = unique_tmp("state-cladechanges-false");
    let path = tmp.join("push-state.toml");
    let original = PushState {
        version: STATE_FORMAT_VERSION,
        stage: Stage::CommitBot,
        bookmark: "main".to_string(),
        started_at: "2026-04-21T00:00:00+00:00".to_string(),
        work_chid: Some("abc".to_string()),
        bot_chid: Some("def".to_string()),
        bot_had_changes: Some(false),
        op_work: None,
        op_bot: None,
        title: None,
        body: None,
    };
    original.save(&path).expect("save");
    let loaded = PushState::load(&path).expect("load").expect("Some state");
    assert_eq!(loaded.bot_had_changes, Some(false));
    let _ = fs::remove_dir_all(&tmp);
}

/// `stage_is_rollback_eligible` returns true only for the three
/// local-mutation stages.
#[test]
fn rollback_eligibility_covers_local_window() {
    assert!(!stage_is_rollback_eligible(Stage::Preflight));
    assert!(!stage_is_rollback_eligible(Stage::Review));
    assert!(!stage_is_rollback_eligible(Stage::Message));
    assert!(stage_is_rollback_eligible(Stage::CommitWork));
    assert!(stage_is_rollback_eligible(Stage::CommitBot));
    assert!(stage_is_rollback_eligible(Stage::BookmarkSet));
    assert!(!stage_is_rollback_eligible(Stage::PushWork));
    assert!(!stage_is_rollback_eligible(Stage::SquashPushBot));
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
        format!(
            "[push-state]\n\
             version = {STATE_FORMAT_VERSION}\n\
             stage = \"bogus-stage\"\n\
             bookmark = \"main\"\n\
             started_at = \"2026-04-21T00:00:00+00:00\"\n"
        ),
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
        format!(
            "[push-state]\n\
             version = {STATE_FORMAT_VERSION}\n\
             stage = \"preflight\"\n\
             started_at = \"2026-04-21T00:00:00+00:00\"\n"
        ),
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

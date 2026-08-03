//! Unit tests for the push module.
//!
//! Flag parsing and message parsing only: the state-file and
//! stage-ordering unit tests went with the state machine at
//! 0.77.0-3. What replaced them is behavioral and lives in
//! `integration_tests`: each stage no-ops when its work is already
//! done, so a rerun is safe.

use super::*;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    args: PushArgs,
}

/// Bare `push` with no flags leaves every optional field at its default.
#[test]
fn parse_defaults() {
    let cli = Cli::try_parse_from(["test"]).unwrap();
    assert!(cli.args.bookmark.is_none());
    assert!(cli.args.bookmark_pos.is_none());
    assert!(!cli.args.step);
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
    let cli =
        Cli::try_parse_from(["test", "--step", "--no-squash-push", "--dry-run", "--yes"]).unwrap();
    assert!(cli.args.step);
    assert!(cli.args.no_squash_push);
    assert!(cli.args.dry_run);
    assert!(cli.args.yes);
}

/// The retired resume flags are gone from the CLI surface.
#[test]
fn parse_rejects_retired_resume_flags() {
    for flag in ["--restart", "--status", "--recheck"] {
        assert!(
            Cli::try_parse_from(["test", flag]).is_err(),
            "{flag} should no longer parse"
        );
    }
    assert!(
        Cli::try_parse_from(["test", "--from", "message"]).is_err(),
        "--from should no longer parse"
    );
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

/// `parse_message` extracts title + body, strips `#` comments,
/// and rejects all-comments / empty input.
#[test]
fn parse_message_cases() {
    // Title + body separated by blank line.
    let (t, b) = parse_message("feat: x\n\nBody here.\nSecond line.\n").unwrap();
    assert_eq!(t, "feat: x");
    assert_eq!(b, "Body here.\nSecond line.");

    // Title + body with no blank line: first line is title, rest is body.
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

    // All comments -> None (caller aborts).
    assert!(parse_message("# only comments\n# and more\n").is_none());
    // All blank -> None.
    assert!(parse_message("   \n\n").is_none());
}

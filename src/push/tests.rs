//! Unit tests for the push module: flag parsing. The behavior is in
//! `integration_tests`, against real fixtures.

use super::*;
use crate::options_flags::scope::Side;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    args: PushArgs,
}

fn parse(args: &[&str]) -> Result<PushArgs, String> {
    match Cli::try_parse_from(args) {
        Ok(cli) => Ok(cli.args),
        Err(e) => Err(e.to_string()),
    }
}

/// `-s` is required and the other flags default off.
#[test]
fn parse_defaults_and_scope_required() {
    let a = parse(&["test", "-s", "b"]).unwrap();
    assert!(a.bookmark.is_none());
    assert!(a.bookmark_pos.is_none());
    assert!(a.repo.is_none());
    assert_eq!(
        a.scope,
        RepoScope::Sides(Scope(vec![Side::Work, Side::Bot]))
    );
    assert!(!a.step);
    assert!(!a.dry_run);
    assert!(!a.yes);
    let err = parse(&["test"]).unwrap_err();
    assert!(err.contains("--scope"), "{err}");
}

/// The bookmark's two spellings, exclusive, collapse into one param.
#[test]
fn parse_bookmark_forms() {
    let a = parse(&["test", "main", "-s", "w"]).unwrap();
    assert_eq!(PushParams::from(&a).bookmark.as_deref(), Some("main"));
    let a = parse(&["test", "--bookmark", "dev", "-s", "a"]).unwrap();
    assert_eq!(PushParams::from(&a).bookmark.as_deref(), Some("dev"));
    assert!(parse(&["test", "main", "--bookmark", "dev", "-s", "b"]).is_err());
}

/// The description flags and the stage switch retired with the
/// pair are refused, as are the older resume flags.
#[test]
fn parse_rejects_retired_flags() {
    for flag in [
        "--title",
        "--body",
        "--no-squash-push",
        "--restart",
        "--status",
        "--recheck",
        "--from",
    ] {
        let err = parse(&["test", "-s", "b", flag, "x"]).unwrap_err();
        assert!(err.contains(flag), "{flag}: {err}");
    }
}

/// `--step`, `--dry-run`, `--yes`, and `-R` parse.
#[test]
fn parse_switches_and_repo() {
    let a = parse(&[
        "test",
        "-s",
        "p",
        "-R",
        "../other",
        "--step",
        "--dry-run",
        "--yes",
    ])
    .unwrap();
    assert_eq!(a.scope, RepoScope::Por);
    assert_eq!(a.repo, Some(PathBuf::from("../other")));
    assert!(a.step && a.dry_run && a.yes);
}

/// The partner's text is the work description without its trailer.
#[test]
fn without_trailers_strips_the_ochid_lines() {
    let desc = "feat: x\n\nBody.\n\nochid: /.claude/abc\n";
    assert_eq!(without_trailers(desc), "feat: x\n\nBody.");
    assert_eq!(without_trailers("feat: y\n"), "feat: y");
}

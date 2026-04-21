//! `push` subcommand — collapse the dual-repo commit+push+finalize
//! ceremony into a single resumable command.
//!
//! See `notes/chores-05.md > Add push subcommand (0.37.0)` for the
//! full design. This module is introduced in 0.37.0-0 as a skeleton:
//! full flag surface, argument parsing, and a stub `push()` that
//! errors with `not yet implemented`. Subsequent `0.37.0-N` commits
//! layer the implementation onto this scaffolding:
//!
//! - `0.37.0-1` — stages + state file, non-interactive
//! - `0.37.0-2` — two-approval interactive flow
//! - `0.37.0-3` — polish (`--dry-run`, `--step`, `--restart`, non-tty)
//! - `0.37.0`   — docs + workflow migration (done marker)

use clap::{Args, ValueEnum};

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

/// CLI arguments for the `push` subcommand.
///
/// Flag set mirrors the design in `notes/chores-05.md`. Flags are
/// parsed in 0.37.0-0; most of them do nothing until the matching
/// implementation dev step lands.
#[derive(Args, Debug)]
pub struct PushArgs {
    /// Bookmark to advance in both repos (required for real runs).
    #[arg(long)]
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

/// Stub entry point — errors until the dev-step implementations land.
///
/// Kept as a real callable so the CLI wiring, argument parsing, and
/// completion surface can be exercised and tested from 0.37.0-0
/// onward without waiting for the state machine. Each `0.37.0-N`
/// step replaces this stub with progressively more real behavior.
pub fn push(_args: &PushArgs) -> Result<(), Box<dyn std::error::Error>> {
    Err("push: not yet implemented (scaffolding only in 0.37.0-0)".into())
}

#[cfg(test)]
mod tests {
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

    /// The 0.37.0-0 stub reports "not yet implemented" and returns `Err`.
    #[test]
    fn stub_returns_unimplemented_error() {
        let args = PushArgs {
            bookmark: None,
            restart: false,
            from: None,
            step: false,
            status: false,
            recheck: false,
            no_finalize: false,
            dry_run: false,
            title: None,
            body: None,
        };
        let err = push(&args).unwrap_err().to_string();
        assert!(err.contains("not yet implemented"), "got: {err}");
    }
}

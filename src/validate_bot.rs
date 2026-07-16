//! The `validate-bot` subcommand: read-only check that the bot
//! repo is in its expected at-rest state.
//!
//! - `main` must match `main@origin`: the bookmark only moves
//!   inside a push / squash-push run, which publishes it in the
//!   same invocation, so an at-rest mismatch means an earlier
//!   publish was lost (the 0.68.1-diagnosed silent session-push loss went
//!   unnoticed for 8 cycles).
//! - Also verifies `main`'s remote refs are tracked.
//! - Exits non-zero on any finding and fixes nothing (decided
//!   2026-07-15: no automatic fixing) — resolve with
//!   `vc-x1 squash-push -R <bot-repo>`.
//! - Cheap: two `jj` lookups, no cargo steps — cheap enough for
//!   routine use (reacquaint, timers, scripts).

use std::path::PathBuf;

use clap::Args;
use log::{debug, info};

use crate::context::Context;
use crate::subcommand::SubcommandRunner;

/// The bot repo's pinned bookmark — all session work publishes to
/// `main` (mirrors push's `BOT_BOOKMARK`).
const BOT_BOOKMARK: &str = "main";

/// Check the bot repo's at-rest invariant: `main` published at
/// origin (matches `main@origin`) and its remote refs tracked.
///
/// Read-only; exits non-zero on a mismatch. Fixes nothing — resolve
/// with `vc-x1 squash-push -R <bot-repo>`.
#[derive(Args, Debug)]
pub struct ValidateBotArgs {
    /// Path to the bot repo
    #[arg(short = 'R', long, default_value = ".claude")]
    pub repo: PathBuf,
}

/// Per-invocation validate-bot inputs — the clap-free shape the op
/// works against.
#[derive(Debug)]
pub struct ValidateBotParams {
    pub repo: PathBuf,
}

impl TryFrom<&ValidateBotArgs> for ValidateBotParams {
    type Error = String;

    /// Canonicalize `--repo` (early, visible failure on a bad path).
    fn try_from(a: &ValidateBotArgs) -> Result<Self, String> {
        let repo = std::fs::canonicalize(&a.repo)
            .map_err(|e| format!("cannot resolve repo path '{}': {e}", a.repo.display()))?;
        Ok(ValidateBotParams { repo })
    }
}

impl SubcommandRunner for ValidateBotArgs {
    type Params = ValidateBotParams;

    /// Delegate to the `TryFrom<&ValidateBotArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        ValidateBotParams::try_from(self)
    }

    /// Run the `validate-bot` op (`ctx` unused — the op is fully
    /// parameterized by `Params`).
    fn run(_ctx: &Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        validate_bot(params)
    }
}

/// Run the `validate-bot` op: tracking check, then the published
/// invariant; report the in-sync state on success.
pub fn validate_bot(params: &ValidateBotParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("validate-bot: entry params={params:?}");
    let repo_str = params.repo.to_string_lossy();
    crate::common::verify_tracking(&params.repo, BOT_BOOKMARK)?;
    crate::common::verify_bot_published(&params.repo, BOT_BOOKMARK)?;
    info!("validate-bot: '{repo_str}' {BOT_BOOKMARK} is published at origin");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::common::{PublishState, bookmark_publish_state, verify_bot_published};
    use crate::test_helpers::{Fixture, jj_ok};
    use crate::{Cli, Commands};
    use clap::Parser;

    fn parse(args: &[&str]) -> ValidateBotArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::ValidateBot(a)) => a,
            _ => panic!("expected ValidateBot"),
        }
    }

    /// Simulate a lost publish in `repo`: seal a commit and move
    /// `main` onto it without pushing.
    fn move_main_unpushed(repo: &Path) {
        std::fs::write(repo.join("lost.txt"), "lost session data\n").expect("write lost file");
        jj_ok(repo, &["commit", "-m", "lost session commit"]);
        jj_ok(repo, &["bookmark", "set", "main", "-r", "@-"]);
    }

    #[test]
    fn no_args_defaults() {
        let args = parse(&["vc-x1", "validate-bot"]);
        assert_eq!(args.repo, PathBuf::from(".claude"));
    }

    #[test]
    fn long_repo_flag() {
        let args = parse(&["vc-x1", "validate-bot", "--repo", "some/dir"]);
        assert_eq!(args.repo, PathBuf::from("some/dir"));
    }

    #[test]
    fn unknown_opt() {
        let err = Cli::try_parse_from(["vc-x1", "validate-bot", "--bogus"])
            .unwrap_err()
            .to_string();
        assert!(err.contains("--bogus"));
    }

    #[test]
    fn try_from_bad_path_errors() {
        let args = ValidateBotArgs {
            repo: PathBuf::from("/nonexistent/vc-x1-validate-bot"),
        };
        let err = ValidateBotParams::try_from(&args).unwrap_err();
        assert!(err.contains("cannot resolve repo path"), "got: {err}");
    }

    #[test]
    fn publish_state_in_sync_after_init() {
        let fx = Fixture::new("vb-insync");
        assert_eq!(
            bookmark_publish_state(&fx.claude, "main").unwrap(),
            PublishState::InSync
        );
    }

    #[test]
    fn publish_state_mismatch_after_unpushed_move() {
        let fx = Fixture::new("vb-mismatch");
        move_main_unpushed(&fx.claude);
        match bookmark_publish_state(&fx.claude, "main").unwrap() {
            PublishState::Mismatch { local, remote } => assert_ne!(local, remote),
            other => panic!("expected Mismatch, got {other:?}"),
        }
    }

    #[test]
    fn publish_state_never_pushed_bookmark() {
        let fx = Fixture::new("vb-never");
        jj_ok(&fx.work, &["bookmark", "create", "feature", "-r", "@-"]);
        assert_eq!(
            bookmark_publish_state(&fx.work, "feature").unwrap(),
            PublishState::NeverPushed
        );
    }

    #[test]
    fn publish_state_missing_bookmark_errors() {
        let fx = Fixture::new("vb-missing");
        let err = bookmark_publish_state(&fx.work, "no-such-bookmark")
            .unwrap_err()
            .to_string();
        assert!(err.contains("does not resolve"), "got: {err}");
    }

    #[test]
    fn validate_bot_ok_on_fresh_fixture() {
        let fx = Fixture::new("vb-ok");
        let params = ValidateBotParams {
            repo: fx.claude.clone(),
        };
        validate_bot(&params).expect("fresh fixture should validate");
    }

    #[test]
    fn validate_bot_errors_on_unpushed_move() {
        let fx = Fixture::new("vb-err");
        move_main_unpushed(&fx.claude);
        let params = ValidateBotParams {
            repo: fx.claude.clone(),
        };
        let err = validate_bot(&params).unwrap_err().to_string();
        assert!(err.contains("does not match"), "got: {err}");
        assert!(err.contains("squash-push"), "got: {err}");
    }

    #[test]
    fn verify_bot_published_never_pushed_message() {
        let fx = Fixture::new("vb-nevermsg");
        jj_ok(&fx.work, &["bookmark", "create", "feature", "-r", "@-"]);
        let err = verify_bot_published(&fx.work, "feature")
            .unwrap_err()
            .to_string();
        assert!(err.contains("never been pushed"), "got: {err}");
    }
}

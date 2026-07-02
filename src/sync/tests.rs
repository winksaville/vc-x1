//! Unit tests for the sync module.

use super::*;
use crate::options_flags::scope::Side;

/// Default flags: no `--check` (the default is the normal atomic
/// sync), bookmark "main", remote "origin", no `-R` and no `--scope`
/// (caller will resolve via the workspace-default scope), `--quiet`
/// off.
#[test]
fn parse_defaults() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test"]).unwrap();
    assert!(!cli.args.check);
    assert!(!cli.args.quiet);
    assert!(!cli.args.rebase);
    assert_eq!(cli.args.bookmark, "main");
    assert_eq!(cli.args.remote, "origin");
    assert!(cli.args.scope.is_none());
}

/// `--rebase` sets the flag; it flows through to `SyncParams`.
#[test]
fn parse_rebase_flag() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "--rebase"]).unwrap();
    assert!(cli.args.rebase);
    assert!(SyncParams::from(&cli.args).rebase);
}

/// `-q` / `--quiet` CLI form is honored.
#[test]
fn parse_quiet_flag() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "--quiet"]).unwrap();
    assert!(cli.args.quiet);
    let cli_short = Cli::try_parse_from(["test", "-q"]).unwrap();
    assert!(cli_short.args.quiet);
}

/// Overrides: `--bookmark`, `--remote` honored.
#[test]
fn parse_overrides() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "--bookmark", "dev", "--remote", "upstream"]).unwrap();
    assert!(!cli.args.check);
    assert_eq!(cli.args.bookmark, "dev");
    assert_eq!(cli.args.remote, "upstream");
    assert!(cli.args.scope.is_none());
}

/// Hidden deprecated `--check` still parses (push preflight relies
/// on it until rewired in-process) and flows through to params.
#[test]
fn parse_check_flag() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "--check"]).unwrap();
    assert!(cli.args.check);
    assert!(SyncParams::from(&cli.args).check);
}

/// `--no-check` is gone — a stale script invocation must fail
/// loudly rather than silently flip semantics.
#[test]
fn parse_no_check_rejected() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    assert!(Cli::try_parse_from(["test", "--no-check"]).is_err());
}

/// `--scope=code` parses into `Scope([Code])`.
#[test]
fn parse_scope_code() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "--scope", "code"]).unwrap();
    assert_eq!(cli.args.scope, Some(Scope(vec![Side::Code])));
}

/// `--scope=code,bot` parses into `Scope([Code, Bot])`.
#[test]
fn parse_scope_code_bot() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "--scope", "code,bot"]).unwrap();
    assert_eq!(cli.args.scope, Some(Scope(vec![Side::Code, Side::Bot])));
}

/// `-R PATH` parses into the `repo` field; `--scope` stays None.
#[test]
fn parse_repo_flag() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "-R", "./solo"]).unwrap();
    assert_eq!(cli.args.repo, Some(PathBuf::from("./solo")));
    assert!(cli.args.scope.is_none());
}

/// `-s` is the short form of `--scope`.
#[test]
fn parse_scope_short_form() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "-s", "code"]).unwrap();
    assert_eq!(cli.args.scope, Some(Scope(vec![Side::Code])));
}

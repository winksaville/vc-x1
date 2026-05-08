//! Unit tests for the sync module.

use super::*;
use crate::scope::Side;

/// Default flags: neither `--check` nor `--no-check` set (the
/// implicit default is check mode), bookmark "main", remote "origin",
/// no `-R` and no `--scope` (caller will resolve via the
/// workspace-default scope), `--quiet` off.
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
    assert!(!cli.args.no_check);
    assert!(!cli.args.quiet);
    assert_eq!(cli.args.bookmark, "main");
    assert_eq!(cli.args.remote, "origin");
    assert!(cli.args.scope.is_none());
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

/// Overrides: `--no-check`, `--bookmark`, `--remote` all honored.
#[test]
fn parse_overrides() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from([
        "test",
        "--no-check",
        "--bookmark",
        "dev",
        "--remote",
        "upstream",
    ])
    .unwrap();
    assert!(cli.args.no_check);
    assert!(!cli.args.check);
    assert_eq!(cli.args.bookmark, "dev");
    assert_eq!(cli.args.remote, "upstream");
    assert!(cli.args.scope.is_none());
}

/// `--check` flag parses as the explicit form of the default.
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
    assert!(!cli.args.no_check);
}

/// `--check` and `--no-check` together are rejected by clap.
#[test]
fn parse_check_no_check_conflict() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    assert!(Cli::try_parse_from(["test", "--check", "--no-check"]).is_err());
}

/// `--scope=code` parses into `Scope::Roles([Code])`.
#[test]
fn parse_scope_code() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "--scope", "code"]).unwrap();
    assert_eq!(cli.args.scope, Some(Scope::Roles(vec![Side::Code])));
}

/// `--scope=code,bot` parses into `Scope::Roles([Code, Bot])`.
#[test]
fn parse_scope_code_bot() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "--scope", "code,bot"]).unwrap();
    assert_eq!(
        cli.args.scope,
        Some(Scope::Roles(vec![Side::Code, Side::Bot]))
    );
}

/// `--scope=./path` parses into `Scope::Single(_)` — single-repo mode.
#[test]
fn parse_scope_path_form() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "--scope", "./solo"]).unwrap();
    assert_eq!(cli.args.scope, Some(Scope::Single(PathBuf::from("./solo"))));
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
    assert_eq!(cli.args.scope, Some(Scope::Roles(vec![Side::Code])));
}

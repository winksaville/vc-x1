//! Unit tests for the sync module.

use super::*;
use crate::options_flags::scope::{RepoScope, Scope, Side};

/// Default flags: bookmark "main", remote "origin", no `-R`,
/// `--quiet` off. `-s` has no default: it is required.
#[test]
fn parse_defaults() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "-s", "w"]).unwrap();
    assert!(!cli.args.quiet);
    assert!(!cli.args.force);
    assert!(!cli.args.dry_run);
    assert_eq!(cli.args.bookmark, "main");
    assert_eq!(cli.args.remote, "origin");
    assert_eq!(cli.args.scope, RepoScope::Sides(Scope(vec![Side::Work])));
}

/// Without `-s` the parse fails: sync never guesses which repos it
/// acts on, and a bad value lists the four scopes.
#[test]
fn parse_without_scope_errors() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let err = match Cli::try_parse_from(["test"]) {
        Ok(_) => panic!("a bare sync parsed"),
        Err(e) => e.to_string(),
    };
    assert!(err.contains("--scope"), "{err}");
    let err = match Cli::try_parse_from(["test", "-s", "x"]) {
        Ok(_) => panic!("-s x parsed"),
        Err(e) => e.to_string(),
    };
    assert!(
        err.contains("[possible values: work, agent, both, por]"),
        "{err}"
    );
}

/// `--force` sets the flag; it flows through to `SyncParams`.
#[test]
fn parse_force_flag() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "--force", "-s", "w"]).unwrap();
    assert!(cli.args.force);
    assert!(SyncParams::from(&cli.args).force);
}

/// `--rebase`, the flag's earlier name, is an alias for `--force`.
#[test]
fn parse_rebase_flag() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "--rebase", "-s", "w"]).unwrap();
    assert!(cli.args.force);
    assert!(SyncParams::from(&cli.args).force);
}

/// `--dry-run` sets the flag; it flows through to `SyncParams`.
#[test]
fn parse_dry_run_flag() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "--dry-run", "-s", "w"]).unwrap();
    assert!(cli.args.dry_run);
    assert!(SyncParams::from(&cli.args).dry_run);
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
    let cli = Cli::try_parse_from(["test", "--quiet", "-s", "w"]).unwrap();
    assert!(cli.args.quiet);
    let cli_short = Cli::try_parse_from(["test", "-q", "-s", "w"]).unwrap();
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
    let cli = Cli::try_parse_from([
        "test",
        "--bookmark",
        "dev",
        "--remote",
        "upstream",
        "-s",
        "w",
    ])
    .unwrap();
    assert_eq!(cli.args.bookmark, "dev");
    assert_eq!(cli.args.remote, "upstream");
    assert_eq!(cli.args.scope, RepoScope::Sides(Scope(vec![Side::Work])));
}

/// `--check` and `--no-check` are gone: a stale script invocation
/// must fail loudly rather than silently flip semantics.
#[test]
fn parse_check_flags_rejected() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    assert!(Cli::try_parse_from(["test", "--check", "-s", "w"]).is_err());
    assert!(Cli::try_parse_from(["test", "--no-check", "-s", "w"]).is_err());
}

/// `--scope=work` parses into the work side.
#[test]
fn parse_scope_work() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "--scope", "work"]).unwrap();
    assert_eq!(cli.args.scope, RepoScope::Sides(Scope(vec![Side::Work])));
}

/// `-s b` parses into both sides, work first.
#[test]
fn parse_scope_both() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "-s", "b"]).unwrap();
    assert_eq!(
        cli.args.scope,
        RepoScope::Sides(Scope(vec![Side::Work, Side::Bot]))
    );
}

/// `-R PATH` parses into the `repo` field, beside `-s p`.
#[test]
fn parse_repo_flag() {
    use clap::Parser;
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: SyncArgs,
    }
    let cli = Cli::try_parse_from(["test", "-R", "./solo", "-s", "p"]).unwrap();
    assert_eq!(cli.args.repo, Some(PathBuf::from("./solo")));
    assert_eq!(cli.args.scope, RepoScope::Por);
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
    let cli = Cli::try_parse_from(["test", "-s", "work"]).unwrap();
    assert_eq!(cli.args.scope, RepoScope::Sides(Scope(vec![Side::Work])));
}

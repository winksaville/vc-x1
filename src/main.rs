mod chid;
mod common;
mod desc;
mod finalize;
mod list;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about = "vc-x1: jj workspace tooling")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Print the changeID for a revision
    Chid(chid::ChidArgs),

    /// Show full description of a commit
    Desc(desc::DescArgs),

    /// List commits in a jj repo
    List(list::ListArgs),

    /// Squash working copy into target commit (daemonizes by default)
    Finalize(finalize::FinalizeArgs),
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Chid(args) => {
            if let Err(e) = chid::chid(&args) {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Commands::Desc(args) => {
            if let Err(e) = desc::desc(&args) {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Commands::List(args) => {
            if let Err(e) = list::list(&args) {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Commands::Finalize(args) => {
            let opts = args.into_opts();
            finalize::log_msg(&opts.log, "main: finalize entry");
            match finalize::finalize(&opts) {
                Ok(()) => {
                    finalize::log_msg(&opts.log, "main: finalize exit ok");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    finalize::log_msg(&opts.log, &format!("main: finalize exit err={e}"));
                    eprintln!("error: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(args).unwrap()
    }

    fn parse_err(args: &[&str]) -> String {
        Cli::try_parse_from(args).unwrap_err().to_string()
    }

    #[test]
    fn chid_defaults() {
        let cli = parse(&["vc-x1", "chid"]);
        if let Commands::Chid(args) = cli.command {
            assert_eq!(args.revision, "@");
            assert_eq!(args.repo, PathBuf::from("."));
            assert_eq!(args.limit, None);
        } else {
            panic!("expected Chid");
        }
    }

    #[test]
    fn chid_with_revision() {
        let cli = parse(&["vc-x1", "chid", "-r", "@-"]);
        if let Commands::Chid(args) = cli.command {
            assert_eq!(args.revision, "@-");
        } else {
            panic!("expected Chid");
        }
    }

    #[test]
    fn chid_with_repo() {
        let cli = parse(&["vc-x1", "chid", "-R", ".claude"]);
        if let Commands::Chid(args) = cli.command {
            assert_eq!(args.repo, PathBuf::from(".claude"));
        } else {
            panic!("expected Chid");
        }
    }

    #[test]
    fn chid_with_limit() {
        let cli = parse(&["vc-x1", "chid", "-l", "5"]);
        if let Commands::Chid(args) = cli.command {
            assert_eq!(args.limit, Some(5));
        } else {
            panic!("expected Chid");
        }
    }

    #[test]
    fn chid_all_opts() {
        let cli = parse(&["vc-x1", "chid", "-r", "@--", "-R", ".claude", "-l", "3"]);
        if let Commands::Chid(args) = cli.command {
            assert_eq!(args.revision, "@--");
            assert_eq!(args.repo, PathBuf::from(".claude"));
            assert_eq!(args.limit, Some(3));
        } else {
            panic!("expected Chid");
        }
    }

    #[test]
    fn desc_defaults() {
        let cli = parse(&["vc-x1", "desc"]);
        if let Commands::Desc(args) = cli.command {
            assert_eq!(args.revision, "@");
            assert_eq!(args.repo, PathBuf::from("."));
        } else {
            panic!("expected Desc");
        }
    }

    #[test]
    fn desc_with_revision() {
        let cli = parse(&["vc-x1", "desc", "-r", "wmuxkqwu"]);
        if let Commands::Desc(args) = cli.command {
            assert_eq!(args.revision, "wmuxkqwu");
        } else {
            panic!("expected Desc");
        }
    }

    #[test]
    fn desc_with_repo() {
        let cli = parse(&["vc-x1", "desc", "-R", "/tmp"]);
        if let Commands::Desc(args) = cli.command {
            assert_eq!(args.repo, PathBuf::from("/tmp"));
        } else {
            panic!("expected Desc");
        }
    }

    #[test]
    fn desc_with_limit() {
        let cli = parse(&["vc-x1", "desc", "-l", "3"]);
        if let Commands::Desc(args) = cli.command {
            assert_eq!(args.limit, Some(3));
        } else {
            panic!("expected Desc");
        }
    }

    #[test]
    fn desc_all_opts() {
        let cli = parse(&["vc-x1", "desc", "-r", "@-", "-R", ".claude", "-l", "5"]);
        if let Commands::Desc(args) = cli.command {
            assert_eq!(args.revision, "@-");
            assert_eq!(args.repo, PathBuf::from(".claude"));
            assert_eq!(args.limit, Some(5));
        } else {
            panic!("expected Desc");
        }
    }

    #[test]
    fn list_defaults() {
        let cli = parse(&["vc-x1", "list"]);
        if let Commands::List(args) = cli.command {
            assert_eq!(args.revision, "@");
            assert_eq!(args.repo, PathBuf::from("."));
            assert!(args.limit.is_none());
        } else {
            panic!("expected List");
        }
    }

    #[test]
    fn list_with_revision() {
        let cli = parse(&["vc-x1", "list", "-r", "@-"]);
        if let Commands::List(args) = cli.command {
            assert_eq!(args.revision, "@-");
        } else {
            panic!("expected List");
        }
    }

    #[test]
    fn list_with_repo() {
        let cli = parse(&["vc-x1", "list", "-R", "/some/path"]);
        if let Commands::List(args) = cli.command {
            assert_eq!(args.repo, PathBuf::from("/some/path"));
        } else {
            panic!("expected List");
        }
    }

    #[test]
    fn list_with_limit() {
        let cli = parse(&["vc-x1", "list", "-l", "5"]);
        if let Commands::List(args) = cli.command {
            assert_eq!(args.limit, Some(5));
        } else {
            panic!("expected List");
        }
    }

    #[test]
    fn list_all_opts() {
        let cli = parse(&["vc-x1", "list", "-r", "all()", "-R", ".claude", "-l", "10"]);
        if let Commands::List(args) = cli.command {
            assert_eq!(args.revision, "all()");
            assert_eq!(args.repo, PathBuf::from(".claude"));
            assert_eq!(args.limit, Some(10));
        } else {
            panic!("expected List");
        }
    }

    #[test]
    fn finalize_defaults() {
        let cli = parse(&["vc-x1", "finalize"]);
        if let Commands::Finalize(args) = cli.command {
            assert_eq!(args.repo, PathBuf::from("."));
            assert_eq!(args.source, "@");
            assert_eq!(args.target, "@-");
            assert_eq!(args.delay, 1.0);
            assert!(!args.push);
            assert!(!args.foreground);
        } else {
            panic!("expected Finalize");
        }
    }

    #[test]
    fn finalize_all_opts() {
        let cli = parse(&[
            "vc-x1",
            "finalize",
            "--repo",
            ".claude",
            "--source",
            "@",
            "--target",
            "@-",
            "--delay",
            "2.5",
            "--push",
            "--log",
            "/tmp/test.log",
            "--foreground",
        ]);
        if let Commands::Finalize(args) = cli.command {
            assert_eq!(args.repo, PathBuf::from(".claude"));
            assert_eq!(args.source, "@");
            assert_eq!(args.target, "@-");
            assert_eq!(args.delay, 2.5);
            assert!(args.push);
            assert_eq!(args.log, Some(PathBuf::from("/tmp/test.log")));
            assert!(args.foreground);
        } else {
            panic!("expected Finalize");
        }
    }

    #[test]
    fn finalize_partial_opts() {
        let cli = parse(&["vc-x1", "finalize", "--repo", ".claude", "--push"]);
        if let Commands::Finalize(args) = cli.command {
            assert_eq!(args.repo, PathBuf::from(".claude"));
            assert!(args.push);
        } else {
            panic!("expected Finalize");
        }
    }

    #[test]
    fn finalize_bad_delay() {
        let err = parse_err(&["vc-x1", "finalize", "--delay", "abc"]);
        assert!(err.contains("invalid value"));
    }

    #[test]
    fn finalize_unknown_opt() {
        let err = parse_err(&["vc-x1", "finalize", "--bogus"]);
        assert!(err.contains("--bogus"));
    }

    #[test]
    fn unknown_command() {
        let err = parse_err(&["vc-x1", "bogus"]);
        assert!(err.contains("bogus"));
    }
}

mod common;
mod desc;
mod finalize;
mod list;

use std::path::PathBuf;
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
    /// Show full description of a commit by changeID
    Desc {
        /// The jj changeID (or prefix) to look up
        chid: String,

        /// Path to jj repo (default: current directory)
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },

    /// List commits in a jj repo
    List {
        /// Path to jj repo (defaults to current directory)
        path: Option<PathBuf>,
    },

    /// Squash working copy into target commit (daemonizes by default)
    Finalize(finalize::FinalizeArgs),
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Desc { chid, repo } => {
            if let Err(e) = desc::desc(&chid, &repo) {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Commands::List { path } => {
            let path = path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            if let Err(e) = list::list(&path) {
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
    use super::*;

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(args).unwrap()
    }

    fn parse_err(args: &[&str]) -> String {
        Cli::try_parse_from(args).unwrap_err().to_string()
    }

    #[test]
    fn desc_with_chid() {
        let cli = parse(&["vc-x1", "desc", "wmuxkqwu"]);
        if let Commands::Desc { chid, repo } = cli.command {
            assert_eq!(chid, "wmuxkqwu");
            assert_eq!(repo, PathBuf::from("."));
        } else {
            panic!("expected Desc");
        }
    }

    #[test]
    fn desc_with_repo() {
        let cli = parse(&["vc-x1", "desc", "wmuxkqwu", "--repo", "/tmp"]);
        if let Commands::Desc { chid, repo } = cli.command {
            assert_eq!(chid, "wmuxkqwu");
            assert_eq!(repo, PathBuf::from("/tmp"));
        } else {
            panic!("expected Desc");
        }
    }

    #[test]
    fn desc_missing_chid() {
        let err = parse_err(&["vc-x1", "desc"]);
        assert!(err.contains("CHID"));
    }

    #[test]
    fn list_with_path() {
        let cli = parse(&["vc-x1", "list", "/some/path"]);
        if let Commands::List { path } = cli.command {
            assert_eq!(path, Some(PathBuf::from("/some/path")));
        } else {
            panic!("expected List");
        }
    }

    #[test]
    fn list_no_path() {
        let cli = parse(&["vc-x1", "list"]);
        if let Commands::List { path } = cli.command {
            assert!(path.is_none());
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

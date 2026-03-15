mod common;
mod finalize;
mod list;

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
Usage: vc-x1 [OPTIONS] <COMMAND>

Commands:
  list [<path>]       List commits in a jj repo (defaults to current directory)
  finalize [OPTIONS]  Squash working copy into target commit (daemonizes by default)

Options:
  -V, --version  Print version
  -h, --help     Print this help message";

#[derive(Debug, PartialEq)]
enum Command {
    Version,
    Help,
    FinalizeHelp,
    List { path: PathBuf },
    Finalize(finalize::FinalizeOpts),
}

fn parse_args<I>(args: I) -> Result<Command, String>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let _program = args.next();

    match args.next().as_deref() {
        Some("--version" | "-V") => Ok(Command::Version),
        Some("--help" | "-h" | "help") => Ok(Command::Help),
        Some("list") => {
            let path = match args.next() {
                Some(p) => PathBuf::from(p),
                None => std::env::current_dir().map_err(|e| e.to_string())?,
            };
            Ok(Command::List { path })
        }
        Some("finalize") => match finalize::parse_args(&mut args)? {
            Some(opts) => Ok(Command::Finalize(opts)),
            None => Ok(Command::FinalizeHelp),
        },
        Some(other) => Err(format!("unknown command: {other}\n\n{USAGE}")),
        None => Ok(Command::Help),
    }
}

fn main() -> ExitCode {
    let cmd = match parse_args(std::env::args()) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    match cmd {
        Command::Version => {
            println!("vc-x1 {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Command::Help => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Command::FinalizeHelp => {
            println!("{}", finalize::USAGE);
            ExitCode::SUCCESS
        }
        Command::List { path } => {
            if let Err(e) = list::list(&path) {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Command::Finalize(opts) => {
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

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_version_long() {
        assert_eq!(
            parse_args(args(&["vc-x1", "--version"])),
            Ok(Command::Version)
        );
    }

    #[test]
    fn parse_version_short() {
        assert_eq!(parse_args(args(&["vc-x1", "-V"])), Ok(Command::Version));
    }

    #[test]
    fn parse_help_flag() {
        assert_eq!(parse_args(args(&["vc-x1", "--help"])), Ok(Command::Help));
    }

    #[test]
    fn parse_help_short() {
        assert_eq!(parse_args(args(&["vc-x1", "-h"])), Ok(Command::Help));
    }

    #[test]
    fn parse_help_subcommand() {
        assert_eq!(parse_args(args(&["vc-x1", "help"])), Ok(Command::Help));
    }

    #[test]
    fn parse_no_args_shows_help() {
        assert_eq!(parse_args(args(&["vc-x1"])), Ok(Command::Help));
    }

    #[test]
    fn parse_list_with_path() {
        assert_eq!(
            parse_args(args(&["vc-x1", "list", "/some/path"])),
            Ok(Command::List {
                path: PathBuf::from("/some/path")
            })
        );
    }

    #[test]
    fn parse_list_no_path_uses_cwd() {
        let result = parse_args(args(&["vc-x1", "list"]));
        assert!(result.is_ok());
        if let Ok(Command::List { path }) = result {
            assert!(path.is_absolute());
        } else {
            panic!("expected List");
        }
    }

    #[test]
    fn parse_unknown_command() {
        let result = parse_args(args(&["vc-x1", "bogus"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown command: bogus"));
    }

    #[test]
    fn parse_finalize_defaults() {
        assert_eq!(
            parse_args(args(&["vc-x1", "finalize"])),
            Ok(Command::Finalize(finalize::FinalizeOpts::default()))
        );
    }

    #[test]
    fn parse_finalize_help() {
        assert_eq!(
            parse_args(args(&["vc-x1", "finalize", "--help"])),
            Ok(Command::FinalizeHelp)
        );
    }

    #[test]
    fn parse_finalize_via_main() {
        assert_eq!(
            parse_args(args(&["vc-x1", "finalize", "--repo", ".claude", "--push"])),
            Ok(Command::Finalize(finalize::FinalizeOpts {
                repo: PathBuf::from(".claude"),
                push: true,
                ..finalize::FinalizeOpts::default()
            }))
        );
    }
}

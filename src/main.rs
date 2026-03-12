use std::path::{Path, PathBuf};
use std::process::ExitCode;

use jj_lib::config::StackedConfig;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::{Repo, StoreFactories};
use jj_lib::revset::{RevsetExpression, SymbolResolver};
use jj_lib::settings::UserSettings;
use jj_lib::workspace::{Workspace, default_working_copy_factories};
use pollster::FutureExt;

const USAGE: &str = "\
Usage: vc-x1 [OPTIONS] <COMMAND>

Commands:
  list [<path>]  List commits in a jj repo (defaults to current directory)

Options:
  -V, --version  Print version
  -h, --help     Print this help message";

#[derive(Debug, PartialEq)]
enum Command {
    Version,
    Help,
    List { path: PathBuf },
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
        Some(other) => Err(format!("unknown command: {other}\n\n{USAGE}")),
        None => Ok(Command::Help),
    }
}

fn list(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let config = StackedConfig::with_defaults();
    let settings = UserSettings::from_config(config)?;
    let store_factories = StoreFactories::default();
    let working_copy_factories = default_working_copy_factories();

    let workspace = Workspace::load(&settings, path, &store_factories, &working_copy_factories)?;
    let repo = workspace.repo_loader().load_at_head().block_on()?;

    let expression = RevsetExpression::all();
    let no_extensions: &[Box<dyn jj_lib::revset::SymbolResolverExtension>] = &[];
    let symbol_resolver = SymbolResolver::new(repo.as_ref(), no_extensions);
    let resolved = expression.resolve_user_expression(repo.as_ref(), &symbol_resolver)?;
    let revset = resolved.evaluate(repo.as_ref())?;

    let root_commit_id = repo.store().root_commit_id().clone();

    for result in revset.iter() {
        let commit_id = result?;
        if commit_id == root_commit_id {
            continue;
        }
        let commit = repo.store().get_commit(&commit_id)?;

        let change_hex = encode_reverse_hex(commit.change_id().as_bytes());
        let change_short = &change_hex[..change_hex.len().min(12)];

        let commit_hex = commit.id().hex();
        let commit_short = &commit_hex[..commit_hex.len().min(12)];

        let first_line = commit.description().lines().next().unwrap_or("");

        println!("{} {} {}", change_short, commit_short, first_line);
    }

    Ok(())
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
        Command::List { path } => {
            if let Err(e) = list(&path) {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
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
}

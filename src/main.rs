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
  list [<path>]       List commits in a jj repo (defaults to current directory)
  finalize [OPTIONS]  Squash working copy into target commit (daemonizes by default)

Options:
  -V, --version  Print version
  -h, --help     Print this help message";

const FINALIZE_USAGE: &str = "\
Usage: vc-x1 finalize [OPTIONS]

Squash source revision into target revision, optionally push afterward.
By default, daemonizes and returns immediately.

Options:
  --repo <path>      Path to jj repo (default: current directory)
  --source <revset>  Source revision to squash (default: @)
  --target <revset>  Target revision to squash into (default: @-)
  --delay <seconds>  Seconds to wait before squashing (default: 1)
  --push             Push after squashing
  --log <path>       Log file path (default: /tmp/vc-x1-finalize.log)
  --foreground       Run in foreground instead of daemonizing
  -h, --help         Print this help message";

#[derive(Debug, PartialEq)]
enum Command {
    Version,
    Help,
    List { path: PathBuf },
    Finalize(FinalizeOpts),
}

#[derive(Debug, PartialEq)]
struct FinalizeOpts {
    repo: PathBuf,
    source: String,
    target: String,
    delay_secs: f64,
    push: bool,
    log: PathBuf,
    foreground: bool,
}

impl Default for FinalizeOpts {
    fn default() -> Self {
        Self {
            repo: PathBuf::from("."),
            source: "@".to_string(),
            target: "@-".to_string(),
            delay_secs: 1.0,
            push: false,
            log: PathBuf::from("/tmp/vc-x1-finalize.log"),
            foreground: false,
        }
    }
}

fn parse_finalize_args<I>(args: &mut I) -> Result<FinalizeOpts, String>
where
    I: Iterator<Item = String>,
{
    let mut opts = FinalizeOpts::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Err(FINALIZE_USAGE.to_string()),
            "--repo" => {
                opts.repo = PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--repo requires a value".to_string())?,
                );
            }
            "--source" => {
                opts.source = args
                    .next()
                    .ok_or_else(|| "--source requires a value".to_string())?;
            }
            "--target" => {
                opts.target = args
                    .next()
                    .ok_or_else(|| "--target requires a value".to_string())?;
            }
            "--delay" => {
                let val = args
                    .next()
                    .ok_or_else(|| "--delay requires a value".to_string())?;
                opts.delay_secs = val
                    .parse()
                    .map_err(|_| format!("invalid delay value: {val}"))?;
            }
            "--push" => opts.push = true,
            "--log" => {
                opts.log = PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--log requires a value".to_string())?,
                );
            }
            "--foreground" => opts.foreground = true,
            other => {
                return Err(format!(
                    "unknown finalize option: {other}\n\n{FINALIZE_USAGE}"
                ));
            }
        }
    }

    Ok(opts)
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
        Some("finalize") => {
            let opts = parse_finalize_args(&mut args)?;
            Ok(Command::Finalize(opts))
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

fn finalize(_opts: &FinalizeOpts) -> Result<(), Box<dyn std::error::Error>> {
    // TODO dev2: daemonize
    // TODO dev3: implement squash + push logic
    eprintln!("finalize: not yet implemented");
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
        Command::Finalize(opts) => {
            if let Err(e) = finalize(&opts) {
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

    #[test]
    fn parse_finalize_defaults() {
        assert_eq!(
            parse_args(args(&["vc-x1", "finalize"])),
            Ok(Command::Finalize(FinalizeOpts::default()))
        );
    }

    #[test]
    fn parse_finalize_all_opts() {
        assert_eq!(
            parse_args(args(&[
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
            ])),
            Ok(Command::Finalize(FinalizeOpts {
                repo: PathBuf::from(".claude"),
                source: "@".to_string(),
                target: "@-".to_string(),
                delay_secs: 2.5,
                push: true,
                log: PathBuf::from("/tmp/test.log"),
                foreground: true,
            }))
        );
    }

    #[test]
    fn parse_finalize_partial_opts() {
        assert_eq!(
            parse_args(args(&["vc-x1", "finalize", "--repo", ".claude", "--push"])),
            Ok(Command::Finalize(FinalizeOpts {
                repo: PathBuf::from(".claude"),
                push: true,
                ..FinalizeOpts::default()
            }))
        );
    }

    #[test]
    fn parse_finalize_missing_value() {
        let result = parse_args(args(&["vc-x1", "finalize", "--repo"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--repo requires a value"));
    }

    #[test]
    fn parse_finalize_bad_delay() {
        let result = parse_args(args(&["vc-x1", "finalize", "--delay", "abc"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid delay value"));
    }

    #[test]
    fn parse_finalize_unknown_opt() {
        let result = parse_args(args(&["vc-x1", "finalize", "--bogus"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown finalize option"));
    }
}

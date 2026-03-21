mod chid;
mod common;
mod desc;
mod desc_helpers;
mod finalize;
mod fix_desc;
mod list;
mod show;
mod toml_simple;
mod validate_desc;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about = "vc-x1: jj workspace tooling")]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Print the changeID for a revision
    Chid(chid::ChidArgs),

    /// Show full description of a commit
    Desc(desc::DescArgs),

    /// List commits in a jj repo
    List(list::ListArgs),

    /// Show commit details and diff summary
    Show(show::ShowArgs),

    /// Validate commit descriptions against the other repo
    #[command(
        long_about = "Validate commit descriptions against the other repo.\n\n\
        Output columns: STATUS  CHANGEID  TITLE  [DETAILS]\n\n\
        Status labels:\n  \
          ok   — ochid trailer is valid\n  \
          err  — ochid has issues (wrong prefix, wrong length, ID not found)\n  \
          miss — no ochid trailer; shows match from other repo if found"
    )]
    ValidateDesc(validate_desc::ValidateDescArgs),

    /// Fix commit descriptions against the other repo (dry-run by default)
    #[command(long_about = "Fix commit descriptions against the other repo.\n\n\
        Default is dry-run; use --no-dry-run to write changes.\n\n\
        Output columns: STATUS  CHANGEID  TITLE  [DETAILS]\n\n\
        Status labels:\n  \
          ok    — ochid trailer is valid (no change)\n  \
          fix   — ochid has issues, shows proposed fix (dry-run)\n  \
          fixed — ochid was rewritten (--no-dry-run)\n  \
          add   — missing ochid, match found, shows proposed addition (dry-run)\n  \
          added — missing ochid was added (--no-dry-run)\n  \
          skip  — skipped (no ochid, no match, or max-fixes reached)\n  \
          err   — ID not found and no --fallback provided")]
    FixDesc(fix_desc::FixDescArgs),

    /// Squash working copy into target commit
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
        Commands::Show(args) => {
            if let Err(e) = show::show(&args) {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Commands::ValidateDesc(args) => {
            if let Err(e) = validate_desc::validate_desc(&args) {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Commands::FixDesc(args) => {
            if let Err(e) = fix_desc::fix_desc(&args) {
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

    #[test]
    fn unknown_command() {
        let err = Cli::try_parse_from(["vc-x1", "bogus"])
            .unwrap_err()
            .to_string();
        assert!(err.contains("bogus"));
    }
}

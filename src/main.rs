mod chid;
mod clone;
mod common;
mod desc;
mod desc_helpers;
mod finalize;
mod fix_desc;
mod init;
mod list;
mod logging;
mod show;
mod symlink;
mod toml_simple;
mod validate_desc;

use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::CompleteEnv;
use log::error;

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

    /// Clone a dual-repo project
    Clone(clone::CloneArgs),

    /// Create a new dual-repo project
    Init(init::InitArgs),

    /// Create Claude Code project symlink
    Symlink(symlink::SymlinkArgs),

    /// Squash working copy into target commit
    Finalize(finalize::FinalizeArgs),
}

impl Commands {
    fn verbose(&self) -> bool {
        match self {
            Commands::Chid(chid_args) => chid_args.common.verbose,
            Commands::Desc(desc_args) => desc_args.common.verbose,
            Commands::List(list_args) => list_args.common.verbose,
            Commands::Show(show_args) => show_args.common.verbose,
            Commands::ValidateDesc(validate_desc_args) => validate_desc_args.verbose,
            Commands::FixDesc(fix_desc_args) => fix_desc_args.verbose,
            Commands::Clone(clone_args) => clone_args.verbose,
            Commands::Init(init_args) => init_args.verbose,
            Commands::Symlink(symlink_args) => symlink_args.verbose,
            Commands::Finalize(_finalize_args) => false, // finalize uses --log
        }
    }
}

fn run_command(result: Result<(), Box<dyn std::error::Error>>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    CompleteEnv::with_factory(Cli::command).complete();
    let cli = Cli::parse();

    logging::CliLogger::init(cli.command.verbose(), None);

    match cli.command {
        Commands::Chid(chid_args) => run_command(chid::chid(&chid_args)),
        Commands::Desc(desc_args) => run_command(desc::desc(&desc_args)),
        Commands::List(list_args) => run_command(list::list(&list_args)),
        Commands::Show(show_args) => run_command(show::show(&show_args)),
        Commands::ValidateDesc(validate_desc_args) => {
            run_command(validate_desc::validate_desc(&validate_desc_args))
        }
        Commands::FixDesc(fix_desc_args) => run_command(fix_desc::fix_desc(&fix_desc_args)),
        Commands::Clone(clone_args) => run_command(clone::clone_repo(&clone_args)),
        Commands::Init(init_args) => run_command(init::init(&init_args)),
        Commands::Symlink(symlink_args) => run_command(symlink::symlink(&symlink_args)),
        Commands::Finalize(finalize_args) => {
            let opts = finalize_args.into_opts();
            finalize::log_msg(&opts.log, "main: finalize entry");
            match finalize::finalize(&opts) {
                Ok(()) => {
                    finalize::log_msg(&opts.log, "main: finalize exit ok");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    finalize::log_msg(&opts.log, &format!("main: finalize exit err={e}"));
                    error!("{e}");
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

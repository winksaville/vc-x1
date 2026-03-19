mod chid;
mod common;
mod desc;
mod finalize;
mod list;
mod show;

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

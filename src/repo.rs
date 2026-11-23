//! `vc-x1 repo`: what is done to a workspace's repos as a whole, a
//! noun and its verbs as `cycle` and `rung` are. `delete` removes a
//! workspace from this machine, and `rename` and `move` are to join it.

pub mod delete;

use std::process::ExitCode;

use clap::{Args, Subcommand};

use crate::context::Context;
use crate::subcommand::SubcommandRunner;

/// The `repo` command and its subcommand.
#[derive(Args, Debug)]
pub struct RepoArgs {
    #[command(subcommand)]
    pub command: RepoCommand,
}

/// The `repo` subcommands.
#[derive(Subcommand, Debug)]
pub enum RepoCommand {
    /// Remove a workspace from this machine
    #[command(
        long_about = "Remove a workspace from this machine: the work repo's directory, \
        the agent repo's when it is outside, and the Claude Code symlink when it points at the \
        agent repo. --local-repos also removes the bare repos the origins are when they are \
        paths here, and a remote reached by a URL is never touched. Work that is nowhere else, \
        uncommitted or unpushed, refuses unless --local-repos, which removes it all. The plan is \
        printed first, and --dry-run prints it alone."
    )]
    Delete(delete::DeleteArgs),
}

impl RepoArgs {
    /// Run the subcommand asked for.
    pub fn dispatch(&self, ctx: &mut Context) -> ExitCode {
        match &self.command {
            RepoCommand::Delete(args) => args.dispatch(ctx),
        }
    }
}

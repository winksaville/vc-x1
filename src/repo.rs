//! `vc-x1 repo`: what is done to a workspace's repos as a whole, a
//! noun and its verbs as `cycle` and `rung` are. `delete` removes a
//! workspace from this machine, `rename` gives it a new name, and
//! `move` puts it in another directory.

pub mod delete;
pub mod relocate;
pub mod rename;

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
    /// Give a workspace a new name, its remote included
    #[command(
        long_about = "Give a workspace a new name, its remote included: the work repo's \
        directory, its origin's last segment, the agent remote when its name is the old one plus \
        a suffix, an agent directory outside the work repo the same way, both origin URLs, the \
        config's names for them, and the Claude Code symlink. What keeps the old name, the \
        family's names for it among them, is listed. Uncommitted or unpushed work, a new name \
        taken, a name GitHub would change, and an origin neither GitHub nor a bare repo here \
        refuse it before anything changes. The plan is printed first, and --dry-run prints it \
        alone."
    )]
    Rename(rename::RenameArgs),
    /// Move a workspace to another directory, its name kept
    #[command(
        long_about = "Move a workspace into DIR, its name kept: the work repo's directory \
        with the agent repo inside it, an origin that is a relative path rewritten for the new \
        place, and the Claude Code symlink made for the new root. No remote is touched. DIR \
        missing, the workspace there already, another filesystem, and an agent repo outside the \
        work repo refuse it before anything moves. The plan is printed first, and --dry-run \
        prints it alone."
    )]
    Move(relocate::MoveArgs),
}

impl RepoArgs {
    /// Run the subcommand asked for.
    pub fn dispatch(&self, ctx: &mut Context) -> ExitCode {
        match &self.command {
            RepoCommand::Delete(args) => args.dispatch(ctx),
            RepoCommand::Rename(args) => args.dispatch(ctx),
            RepoCommand::Move(args) => args.dispatch(ctx),
        }
    }
}

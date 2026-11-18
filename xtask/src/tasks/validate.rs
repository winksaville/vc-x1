//! `cargo xtask validate [--fast]`: check the source, each command of
//! the list one invocation, stopping at the first failure.
//!
//! - The lists are the workspace's, `[workspace.metadata.xtask]
//!   validate` and `validate-fast`, else the defaults in
//!   [`crate::workspace`]: `cargo fmt`, `cargo clippy --all-targets --
//!   -D warnings`, and `cargo test`, or `cargo test --bins` alone.
//! - The commands run with the build's provenance, as `build`'s do, so
//!   the test build compiles with the commit and state the dev build
//!   that follows it has, which then reuses it rather than recompiling.

use std::process::ExitCode;

use crate::provenance::Provenance;
use crate::util::{command_at, root, status};
use crate::workspace::Workspace;

/// The task's arguments.
#[derive(clap::Args, Debug)]
#[command(
    about = "Check the source: fmt, clippy, and the tests, by default",
    long_about = "\
        Check the source, stopping at the first failure, by the workspace's\n\
        [workspace.metadata.xtask] validate list, or validate-fast with\n\
        --fast, each command an array of words, `cargo` the cargo running\n\
        the task. Without them, `cargo fmt`, `cargo clippy --all-targets --\n\
        -D warnings`, and `cargo test`, or `cargo test --bins` alone with\n\
        --fast. Each runs with the build's provenance, as build's does, so\n\
        the test build is the one a dev build then reuses."
)]
pub struct Args {
    /// Run the fast list, by default the binaries' unit tests alone
    #[arg(long)]
    pub fast: bool,
}

/// Run the list in order, stopping at the first failure.
pub fn run(args: &Args) -> Result<ExitCode, String> {
    let name = match args.fast {
        true => "fast",
        false => "full",
    };
    let root = root();
    let lists = Workspace::read(&root)?.validation(args.fast);
    let provenance = Provenance::read(&root);
    for (i, words) in lists.iter().enumerate() {
        let what = format!("`{}`", words.join(" "));
        eprintln!("validate: {what}");
        let cmd = command_at(&root, provenance.as_ref(), words);
        status(cmd, &what).map_err(|e| {
            format!(
                "validate: {e}, command {} of {} ({name})",
                i + 1,
                lists.len()
            )
        })?;
    }
    println!("validate: {} command(s) passed ({name})", lists.len());
    Ok(ExitCode::SUCCESS)
}

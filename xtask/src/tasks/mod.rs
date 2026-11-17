//! The tasks, one module each, and the [`Task`] enum that names them.
//!
//! A task's module holds its `Args`, clap's, with its help, and its
//! `run`, which prints what the task says and returns its exit code.
//! Adding a task is three edits, all in this directory:
//!
//! 1. A module, `tasks/<task>.rs`, with `pub struct Args` deriving
//!    `clap::Args`, its `about` and `long_about` in `#[command(...)]`,
//!    and `pub fn run(args: &Args) -> Result<ExitCode, String>`.
//! 2. Its `mod` line below and a [`Task`] variant holding its `Args`.
//! 3. The variant's arm in [`dispatch`].
//!
//! What the tasks share is outside this directory, in `crate::util` and
//! `crate::provenance`.
//!
//! - `version` and `bump`: the version-of-record in `Cargo.toml`.
//! - `completions`: the bash function completing `cargo xtask`.
//! - `validate`, `build`, `run`, and `install`: the work's, each run
//!   through cargo with the build's provenance.

use std::process::ExitCode;

use clap::Subcommand;

pub mod build;
pub mod bump;
pub mod completions;
pub mod install;
pub mod run;
pub mod validate;
pub mod version;

/// A task, its help on its `Args`.
#[derive(Subcommand, Debug)]
pub enum Task {
    Version(version::Args),
    Bump(bump::Args),
    Completions(completions::Args),
    Validate(validate::Args),
    Build(build::Args),
    Run(run::Args),
    Install(install::Args),
}

/// Run a task.
pub fn dispatch(task: &Task) -> Result<ExitCode, String> {
    match task {
        Task::Version(args) => version::run(args),
        Task::Bump(args) => bump::run(args),
        Task::Completions(args) => completions::run(args),
        Task::Validate(args) => validate::run(args),
        Task::Build(args) => build::run(args),
        Task::Run(args) => run::run(args),
        Task::Install(args) => install::run(args),
    }
}

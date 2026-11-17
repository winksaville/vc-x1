//! `cargo xtask validate [--fast]`: check the source, each element of
//! the list one invocation of cargo, stopping at the first failure.
//!
//! The commands run with the build's provenance, as `build`'s do, so
//! the test build compiles with the commit and state the dev build
//! that follows it has, which then reuses it rather than recompiling.

use std::process::ExitCode;

use crate::provenance::Provenance;
use crate::util::{cargo_at, root, status};

/// The full validation.
const FULL: &[&[&str]] = &[
    &["fmt"],
    &["clippy", "--all-targets", "--", "-D", "warnings"],
    &["test", "--", "--include-ignored"],
];

/// The fast validation, the binaries' unit tests alone.
const FAST: &[&[&str]] = &[&["test", "--bins"]];

/// The task's arguments.
#[derive(clap::Args, Debug)]
#[command(
    about = "Check the source: fmt, clippy, and every test",
    long_about = "\
        Check the source, stopping at the first failure: `cargo fmt`, `cargo\n\
        clippy --all-targets -- -D warnings`, and `cargo test --\n\
        --include-ignored`, or `cargo test --bins` alone with --fast. Each\n\
        runs with the build's provenance, as build's does, so the test build\n\
        is the one a dev build then reuses."
)]
pub struct Args {
    /// Run the binaries' unit tests alone
    #[arg(long)]
    pub fast: bool,
}

/// Run the lists in order, stopping at the first failure.
pub fn run(args: &Args) -> Result<ExitCode, String> {
    let (lists, name) = match args.fast {
        true => (FAST, "fast"),
        false => (FULL, "full"),
    };
    let root = root();
    let provenance = Provenance::read(&root);
    for (i, words) in lists.iter().enumerate() {
        let what = format!("`cargo {}`", words.join(" "));
        eprintln!("validate: {what}");
        let mut cmd = cargo_at(&root, provenance.as_ref());
        cmd.args(*words);
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

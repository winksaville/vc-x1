//! `cargo xtask build [--release | --profile NAME]`: build a Cargo
//! profile with the build's provenance, dev by default, as `cargo
//! build`'s. `run` and `install` build through [`build`] first.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::provenance::Provenance;
use crate::util::{ProfileArgs, binary, cargo_at, root, status};

/// The task's arguments.
#[derive(clap::Args, Debug)]
#[command(
    about = "Build a profile, dev by default",
    long_about = "\
        Build a Cargo profile, `cargo build --profile NAME --locked`, with the\n\
        build's provenance, `@-`'s commit and whether `@` is empty, read\n\
        through jj, so the binary's banner says what it was built from. Dev\n\
        by default, as `cargo build`'s."
)]
pub struct Args {
    #[command(flatten)]
    pub profile: ProfileArgs,
}

/// Build, saying where the binary is.
pub fn run(args: &Args) -> Result<ExitCode, String> {
    let bin = build(&root(), &args.profile.name("dev"))?;
    println!("build: {}", bin.display());
    Ok(ExitCode::SUCCESS)
}

/// The build's arguments to cargo.
pub fn build_args(profile: &str) -> Vec<String> {
    ["build", "--profile", profile, "--locked"]
        .map(str::to_string)
        .to_vec()
}

/// Build `profile` with the provenance, returning the binary, checked
/// to be there.
pub fn build(root: &Path, profile: &str) -> Result<PathBuf, String> {
    let provenance = Provenance::read(root);
    let mut cmd = cargo_at(root, provenance.as_ref());
    cmd.args(build_args(profile));
    status(cmd, &format!("`cargo {}`", build_args(profile).join(" ")))?;
    let bin = binary(root, profile)?;
    match bin.is_file() {
        true => Ok(bin),
        false => Err(format!(
            "the build passed and left no binary at '{}'",
            bin.display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The build is the profile's, locked.
    #[test]
    fn the_build_is_the_profiles() {
        assert_eq!(build_args("dev"), ["build", "--profile", "dev", "--locked"]);
    }
}

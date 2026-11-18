//! `cargo xtask build [--release | --profile NAME] [--bin NAME]`: build
//! a Cargo profile with the build's provenance, dev by default, as
//! `cargo build`'s. `run` and `install` build through [`build`] first.
//!
//! - The build is the workspace's default members, as a plain `cargo
//!   build`'s and `validate`'s are, so a switch between them resolves
//!   the same features and recompiles nothing, and the binary is picked
//!   after. A binary outside the default members adds `--bin`.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::provenance::Provenance;
use crate::util::{BinArgs, ProfileArgs, cargo_at, root, status};
use crate::workspace::{Bin, Workspace};

/// The task's arguments.
#[derive(clap::Args, Debug)]
#[command(
    about = "Build a profile, dev by default",
    long_about = "\
        Build a Cargo profile, `cargo build --profile NAME --locked`, with the\n\
        build's provenance, `@-`'s commit and whether `@` is empty, read\n\
        through jj, for a build.rs that compiles it in. Dev by default, as\n\
        `cargo build`'s. The binary is --bin's, else the workspace's default:\n\
        [workspace.metadata.xtask] default-bin, else the root package's\n\
        default-run, else its one binary."
)]
pub struct Args {
    #[command(flatten)]
    pub profile: ProfileArgs,

    #[command(flatten)]
    pub bin: BinArgs,
}

/// Build, saying where the binary is.
pub fn run(args: &Args) -> Result<ExitCode, String> {
    let root = root();
    let ws = Workspace::read(&root)?;
    let bin = ws.bin(args.bin.bin.as_deref())?;
    let path = build(&root, &ws, &args.profile.name("dev"), bin)?;
    println!("build: {}", path.display());
    Ok(ExitCode::SUCCESS)
}

/// The build's arguments to cargo, `--package` and `--bin` for a
/// binary a plain build leaves out, since `--bin` alone looks only in
/// the default members.
pub fn build_args(profile: &str, bin: &Bin) -> Vec<String> {
    let mut args: Vec<String> = ["build", "--profile", profile, "--locked"]
        .map(str::to_string)
        .to_vec();
    if !bin.default_member {
        args.extend([
            "--package".to_string(),
            bin.package.clone(),
            "--bin".to_string(),
            bin.name.clone(),
        ]);
    }
    args
}

/// Build `profile` with the provenance, returning `bin`'s binary,
/// checked to be there.
pub fn build(root: &Path, ws: &Workspace, profile: &str, bin: &Bin) -> Result<PathBuf, String> {
    let provenance = Provenance::read(root);
    let mut cmd = cargo_at(root, provenance.as_ref());
    cmd.args(build_args(profile, bin));
    status(
        cmd,
        &format!("`cargo {}`", build_args(profile, bin).join(" ")),
    )?;
    let path = ws.binary(profile, bin);
    match path.is_file() {
        true => Ok(path),
        false => Err(format!(
            "the build passed and left no binary at '{}'",
            path.display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The build is the profile's, locked, and names a binary only
    /// when a plain build leaves it out.
    #[test]
    fn the_build_is_the_profiles() {
        let member = Bin {
            name: "app".to_string(),
            package: "app".to_string(),
            default_member: true,
        };
        assert_eq!(
            build_args("dev", &member),
            ["build", "--profile", "dev", "--locked"]
        );
        let other = Bin {
            name: "tool".to_string(),
            package: "tools".to_string(),
            default_member: false,
        };
        assert_eq!(
            build_args("release", &other),
            [
                "build",
                "--profile",
                "release",
                "--locked",
                "--package",
                "tools",
                "--bin",
                "tool"
            ]
        );
    }
}

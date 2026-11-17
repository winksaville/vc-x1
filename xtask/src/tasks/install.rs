//! `cargo xtask install [--release | --profile NAME] [--dir DIR]
//! [--dry-run]`: build a profile, release by default, then copy the
//! binary into cargo's bin directory.
//!
//! - The copy is named by the profile: `vc-x1` for `release` and
//!   `vc-x1-<name>` for any other, so a dev build sits beside the
//!   release and never replaces it, and other sessions keep the plain
//!   name.
//! - The copy is a temporary file beside the destination renamed into
//!   place, so a running copy of the old binary is undisturbed and a
//!   failed copy leaves it, and the installed binary's `-V` is printed.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use super::build::{build, build_args};
use crate::util::{BIN, ProfileArgs, binary, command, root};

/// The task's arguments.
#[derive(clap::Args, Debug)]
#[command(
    about = "Build a profile and copy vc-x1 onto the PATH, release by default",
    long_about = "\
        Build a Cargo profile, release by default, as build does, then copy\n\
        the binary into cargo's bin directory, `$CARGO_HOME/bin` or\n\
        `~/.cargo/bin`: `vc-x1` for release and `vc-x1-<name>` for any other\n\
        profile, so `--profile dev` installs `vc-x1-dev` beside the release\n\
        and never replaces it. The copy is a temporary file renamed into\n\
        place, so a running copy of the old binary is undisturbed, and the\n\
        installed binary's -V is printed."
)]
pub struct Args {
    #[command(flatten)]
    pub profile: ProfileArgs,

    /// Copy into DIR instead of cargo's bin directory
    #[arg(long, value_name = "DIR")]
    pub dir: Option<PathBuf>,

    /// Say what would be built and copied, and do nothing
    #[arg(long)]
    pub dry_run: bool,
}

/// Build and copy, printing the copy's `-V`, or with `--dry-run` what
/// would be done.
pub fn run(args: &Args) -> Result<ExitCode, String> {
    let said = install(
        &root(),
        &args.profile.name("release"),
        args.dir.as_deref(),
        args.dry_run,
    )?;
    println!("{said}");
    Ok(ExitCode::SUCCESS)
}

/// The name the binary of `profile` is installed under.
pub fn installed_name(profile: &str) -> String {
    match profile {
        "release" => BIN.to_string(),
        name => format!("{BIN}-{name}"),
    }
}

/// Cargo's bin directory, where `install` copies by default:
/// `$CARGO_HOME/bin`, else `~/.cargo/bin`.
fn install_dir() -> Result<PathBuf, String> {
    if let Some(home) = std::env::var_os("CARGO_HOME") {
        return Ok(PathBuf::from(home).join("bin"));
    }
    match std::env::var_os("HOME") {
        Some(home) => Ok(PathBuf::from(home).join(".cargo").join("bin")),
        None => Err("neither CARGO_HOME nor HOME is set, so `--dir` is needed".to_string()),
    }
}

/// Build `profile` and copy its binary into `dir`, else cargo's bin
/// directory, under [`installed_name`], returning what to print.
fn install(
    root: &Path,
    profile: &str,
    dir: Option<&Path>,
    dry_run: bool,
) -> Result<String, String> {
    let dir = match dir {
        Some(dir) => dir.to_path_buf(),
        None => install_dir()?,
    };
    let name = format!(
        "{}{}",
        installed_name(profile),
        std::env::consts::EXE_SUFFIX
    );
    let dest = dir.join(&name);
    if dry_run {
        return Ok(format!(
            "install: would run `cargo {}`, then copy '{}' to '{}'",
            build_args(profile).join(" "),
            binary(root, profile)?.display(),
            dest.display()
        ));
    }
    if !dir.is_dir() {
        return Err(format!("no directory '{}' to install into", dir.display()));
    }
    let bin = build(root, profile)?;
    let tmp = dir.join(format!("{name}.new"));
    std::fs::copy(&bin, &tmp)
        .map_err(|e| format!("copying '{}' to '{}': {e}", bin.display(), tmp.display()))?;
    std::fs::rename(&tmp, &dest)
        .map_err(|e| format!("moving '{}' to '{}': {e}", tmp.display(), dest.display()))?;
    let version = command(&dest)
        .arg("-V")
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .map_err(|e| format!("'{}' -V did not run: {e}", dest.display()))?;
    Ok(format!("install: {}\n{version}", dest.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The installed name is the binary's with the profile appended,
    /// `release`'s plain.
    #[test]
    fn the_installed_name_is_the_profiles() {
        assert_eq!(installed_name("release"), "vc-x1");
        assert_eq!(installed_name("dev"), "vc-x1-dev");
        assert_eq!(installed_name("soak"), "vc-x1-soak");
    }
}

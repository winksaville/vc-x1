//! `cargo xtask install [--release | --profile NAME] [--bin NAME]
//! [--dir DIR] [--dry-run]`: build a profile, dev by default, then copy
//! the binary into cargo's bin directory.
//!
//! - Dev is the default, as `build`'s and `run`'s, where `cargo
//!   install`'s is release, so the plain name, the binary every session
//!   on the machine uses, is replaced only by a deliberate `--release`.
//! - The copy is named by the profile: `<bin>` for `release` and
//!   `<bin>-<profile>` for any other, so a dev build sits beside the
//!   release and never replaces it.
//! - The copy is a temporary file beside the destination renamed into
//!   place, so a running copy of the old binary is undisturbed and a
//!   failed copy leaves it, and the installed binary's `-V` is printed.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use super::build::{build, build_args};
use crate::util::{BinArgs, ProfileArgs, command, root};
use crate::workspace::{Bin, Workspace};

/// The task's arguments.
#[derive(clap::Args, Debug)]
#[command(
    about = "Build a profile and copy the binary onto the PATH, dev by default",
    long_about = "\
        Build a Cargo profile, dev by default, as build does, then copy the\n\
        binary, --bin's or the workspace's default, into cargo's bin\n\
        directory, `$CARGO_HOME/bin` or `~/.cargo/bin`: `<bin>` for release\n\
        and `<bin>-<profile>` for any other, so `--profile dev` installs\n\
        `<bin>-dev` beside the release and never replaces it.\n\n\
        Dev is the default, where `cargo install`'s is release, so the plain\n\
        name, which every session on the machine uses, is replaced only by\n\
        --release. The copy is a temporary file renamed into place, so a\n\
        running copy of the old binary is undisturbed, and the installed\n\
        binary's -V is printed."
)]
pub struct Args {
    #[command(flatten)]
    pub profile: ProfileArgs,

    #[command(flatten)]
    pub bin: BinArgs,

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
    let root = root();
    let ws = Workspace::read(&root)?;
    let bin = ws.bin(args.bin.bin.as_deref())?;
    let said = install(
        &root,
        &ws,
        bin,
        &args.profile.name("dev"),
        args.dir.as_deref(),
        args.dry_run,
    )?;
    println!("{said}");
    Ok(ExitCode::SUCCESS)
}

/// The name `bin`'s build of `profile` is installed under.
pub fn installed_name(bin: &str, profile: &str) -> String {
    match profile {
        "release" => bin.to_string(),
        name => format!("{bin}-{name}"),
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

/// Build `bin`'s `profile` and copy it into `dir`, else cargo's bin
/// directory, under [`installed_name`], returning what to print.
fn install(
    root: &Path,
    ws: &Workspace,
    bin: &Bin,
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
        installed_name(&bin.name, profile),
        std::env::consts::EXE_SUFFIX
    );
    let dest = dir.join(&name);
    if dry_run {
        return Ok(format!(
            "install: would run `cargo {}`, then copy '{}' to '{}'",
            build_args(profile, bin).join(" "),
            ws.binary(profile, bin).display(),
            dest.display()
        ));
    }
    if !dir.is_dir() {
        return Err(format!("no directory '{}' to install into", dir.display()));
    }
    let path = build(root, ws, profile, bin)?;
    let tmp = dir.join(format!("{name}.new"));
    std::fs::copy(&path, &tmp)
        .map_err(|e| format!("copying '{}' to '{}': {e}", path.display(), tmp.display()))?;
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
    /// release's plain.
    #[test]
    fn the_installed_name_is_the_binarys_and_the_profiles() {
        assert_eq!(installed_name("app", "release"), "app");
        assert_eq!(installed_name("app", "dev"), "app-dev");
        assert_eq!(installed_name("tool", "soak"), "tool-soak");
    }
}

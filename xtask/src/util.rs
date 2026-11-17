//! What the tasks share: spawning a process, the workspace root,
//! cargo, and where a profile's build leaves the binary.
//!
//! - A profile is Cargo's, `[profile.<name>]` in `Cargo.toml`, `dev`
//!   and `release` built in. Its binary is in cargo's target directory,
//!   `debug` for `dev` and `test`, `release` for `bench`, and the
//!   profile's own name for any other, so no config names it.
//! - [`ProfileArgs`] is the `--release` and `--profile NAME` pair a task
//!   flattens into its own `Args`, each task giving its default.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;

use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};
use toml_edit::DocumentMut;

use crate::provenance::Provenance;

/// The binary the tasks build, run, and install.
pub const BIN: &str = "vc-x1";

/// A process to spawn.
///
/// The root's clippy.toml bans spawning for `vc-x1`, which goes through
/// jj-lib. A project task running cargo, jj, or the binary it built is
/// the exception, and this is the one site.
#[allow(clippy::disallowed_methods)]
pub fn command(program: impl AsRef<OsStr>) -> Command {
    Command::new(program)
}

/// The workspace root, the directory above this package's.
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// The cargo that runs this task, `cargo` when the binary is run by
/// hand.
pub fn cargo() -> OsString {
    match std::env::var_os("CARGO") {
        Some(cargo) => cargo,
        None => "cargo".into(),
    }
}

/// A cargo command from `root` with the provenance in its environment.
pub fn cargo_at(root: &Path, provenance: Option<&Provenance>) -> Command {
    let mut cmd = command(cargo());
    cmd.current_dir(root);
    if let Some(p) = provenance {
        p.set_env(&mut cmd);
    }
    cmd
}

/// Run `cmd`, an error naming `what` when it fails.
pub fn status(mut cmd: Command, what: &str) -> Result<(), String> {
    let status = cmd
        .status()
        .map_err(|e| format!("{what} did not run: {e}"))?;
    match status.success() {
        true => Ok(()),
        false => Err(format!("{what} failed: {status}")),
    }
}

/// The profile flags of the tasks that build.
#[derive(clap::Args, Debug)]
pub struct ProfileArgs {
    /// The release profile
    #[arg(long, conflicts_with = "profile")]
    pub release: bool,

    /// A Cargo profile by name, `dev`, `release`, or one Cargo.toml declares
    #[arg(long, value_name = "NAME", add = ArgValueCandidates::new(profile_candidates))]
    pub profile: Option<String>,
}

impl ProfileArgs {
    /// The profile named, else `default`.
    pub fn name(&self, default: &str) -> String {
        profile_of(self.release, self.profile.as_deref(), default)
    }
}

/// The profile `--release` and `--profile` name, else `default`.
pub fn profile_of(release: bool, profile: Option<&str>, default: &str) -> String {
    match (release, profile) {
        (true, _) => "release".to_string(),
        (false, Some(name)) => name.to_string(),
        (false, None) => default.to_string(),
    }
}

/// What `--profile` completes to: `dev`, `release`, and every
/// `[profile.<name>]` the workspace's Cargo.toml declares.
fn profile_candidates() -> Vec<CompletionCandidate> {
    let mut names = vec!["dev".to_string(), "release".to_string()];
    if let Ok(text) = std::fs::read_to_string(root().join("Cargo.toml"))
        && let Ok(doc) = text.parse::<DocumentMut>()
        && let Some(table) = doc.get("profile").and_then(|p| p.as_table_like())
    {
        for (name, _) in table.iter() {
            if !names.iter().any(|n| n == name) {
                names.push(name.to_string());
            }
        }
    }
    names.into_iter().map(CompletionCandidate::new).collect()
}

/// The directory under cargo's target directory a profile's binaries
/// are in.
pub fn profile_dir(profile: &str) -> &str {
    match profile {
        "dev" | "test" => "debug",
        "bench" => "release",
        name => name,
    }
}

/// Cargo's target directory for the workspace at `root`, as `cargo
/// metadata` says, so a `target-dir` configured anywhere is followed.
fn target_dir(root: &Path) -> Result<PathBuf, String> {
    let out = command(cargo())
        .current_dir(root)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .output()
        .map_err(|e| format!("cargo metadata did not run: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let meta: serde_json::Value = serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("cargo metadata printed no JSON: {e}"))?;
    match meta.get("target_directory").and_then(|d| d.as_str()) {
        Some(dir) => Ok(PathBuf::from(dir)),
        None => Err("cargo metadata named no target_directory".to_string()),
    }
}

/// Where the build of `profile` leaves the binary.
pub fn binary(root: &Path, profile: &str) -> Result<PathBuf, String> {
    Ok(target_dir(root)?
        .join(profile_dir(profile))
        .join(format!("{BIN}{}", std::env::consts::EXE_SUFFIX)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A profile's directory is cargo's, and the flags name the
    /// profile, else the task's default.
    #[test]
    fn profiles_are_cargos() {
        assert_eq!(profile_dir("dev"), "debug");
        assert_eq!(profile_dir("test"), "debug");
        assert_eq!(profile_dir("release"), "release");
        assert_eq!(profile_dir("bench"), "release");
        assert_eq!(profile_dir("soak"), "soak");
        assert_eq!(profile_of(true, None, "dev"), "release");
        assert_eq!(profile_of(false, Some("soak"), "dev"), "soak");
        assert_eq!(profile_of(false, None, "dev"), "dev");
    }
}

//! `cargo xtask version`: print the version in `Cargo.toml`, the
//! version-of-record.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use toml_edit::DocumentMut;

use crate::util::root;

/// The task's arguments, none.
#[derive(clap::Args, Debug)]
#[command(about = "Print the version in Cargo.toml")]
pub struct Args {}

/// Print the version.
pub fn run(_args: &Args) -> Result<ExitCode, String> {
    println!("{}", version_in(&read(&manifest())?)?);
    Ok(ExitCode::SUCCESS)
}

/// The workspace's manifest, the root package's `Cargo.toml`.
pub fn manifest() -> PathBuf {
    root().join("Cargo.toml")
}

/// The text of the manifest at `path`.
pub fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("Cargo.toml: {e}"))
}

/// The package version in a manifest's text.
pub fn version_in(manifest: &str) -> Result<String, String> {
    let doc: DocumentMut = manifest
        .parse()
        .map_err(|e| format!("Cargo.toml does not parse: {e}"))?;
    doc.get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| "Cargo.toml has no `[package] version`".to_string())
}

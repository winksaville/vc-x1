//! `cargo xtask version`: print the version in `Cargo.toml`, the
//! version-of-record, the root package's or, in a virtual workspace,
//! `[workspace.package]`'s.

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

/// Where the version-of-record is in a manifest: `[package] version`
/// when it is written there, else `[workspace.package] version`, a
/// virtual workspace's, or a package's that says `version.workspace =
/// true`.
pub fn version_path(doc: &DocumentMut) -> Result<&'static [&'static str], String> {
    let written = |path: &[&str]| {
        let mut item = doc.as_item();
        for key in path {
            item = match item.get(key) {
                Some(next) => next,
                None => return false,
            };
        }
        item.get("version").is_some_and(|v| v.as_str().is_some())
    };
    if written(&["package"]) {
        return Ok(&["package"]);
    }
    if written(&["workspace", "package"]) {
        return Ok(&["workspace", "package"]);
    }
    Err("Cargo.toml has neither `[package] version` nor `[workspace.package] version`".to_string())
}

/// The version-of-record in a manifest's text, by [`version_path`].
pub fn version_in(manifest: &str) -> Result<String, String> {
    let doc: DocumentMut = manifest
        .parse()
        .map_err(|e| format!("Cargo.toml does not parse: {e}"))?;
    let mut item = doc.as_item();
    for key in version_path(&doc)? {
        item = &item[key];
    }
    // OK: version_path found a string there, so the default is never taken
    Ok(item["version"].as_str().unwrap_or_default().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The version is the package's when written there, else the
    /// workspace's, and a manifest with neither is refused.
    #[test]
    fn the_version_is_the_packages_or_the_workspaces() {
        assert_eq!(
            version_in("[package]\nversion = \"1.2.3\"\n").unwrap(),
            "1.2.3"
        );
        let virtual_ws =
            "[workspace]\nmembers = [\"a\"]\n\n[workspace.package]\nversion = \"0.4.0\"\n";
        assert_eq!(version_in(virtual_ws).unwrap(), "0.4.0");
        let inherits =
            "[package]\nversion.workspace = true\n\n[workspace.package]\nversion = \"2.0.0\"\n";
        assert_eq!(version_in(inherits).unwrap(), "2.0.0");
        assert!(version_in("[workspace]\n").unwrap_err().contains("neither"));
    }
}

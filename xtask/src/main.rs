//! `cargo xtask`: the project's own tasks, outside `vc-x1`.
//!
//! - `cargo xtask version` prints the version in `Cargo.toml`.
//! - `cargo xtask bump <next|bare|VERSION>` sets it: `next` adds one to
//!   the suffix's last number, `0.85.22-8` to `0.85.22-9`, and gives a
//!   bare version its first, `-0`, `bare` drops the suffix, a
//!   close-out's version, and anything else is the version as written.
//!   `Cargo.lock` is refreshed in the same step, since the build is
//!   `--locked`.
//!
//! The manifest is edited by `toml_edit`, which keeps its comments,
//! layout, and key order and changes the one value.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use toml_edit::{DocumentMut, value};

/// What `bump` is asked for.
#[derive(Debug, PartialEq, Eq)]
enum Bump {
    /// One more than the suffix's last number, or `-0` on a bare version.
    Next,
    /// The version without its suffix.
    Bare,
    /// The version as written.
    To(String),
}

/// The version `bump` gives `current`.
///
/// The version is `X.Y.Z` or `X.Y.Z-<suffix>`, the suffix dot-separated
/// identifiers with exactly one `-` in the whole version, as the
/// project's suffix scheme has it. Refused: `next` on a suffix whose
/// last identifier is not a number, and a version written with a `+`
/// or a second `-`.
fn bumped(current: &str, bump: &Bump) -> Result<String, String> {
    let (public, suffix) = match current.split_once('-') {
        Some((public, suffix)) => (public, Some(suffix)),
        None => (current, None),
    };
    match bump {
        Bump::Bare => Ok(public.to_string()),
        Bump::To(version) => {
            let well_formed = version.matches('-').count() <= 1
                && !version.contains('+')
                && !version.is_empty()
                && version
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || ".-".contains(c));
            match well_formed {
                true => Ok(version.clone()),
                false => Err(format!(
                    "'{version}' is not a version, `X.Y.Z` or `X.Y.Z-<suffix>` with one `-` and \
                     no `+`"
                )),
            }
        }
        Bump::Next => match suffix {
            None => Ok(format!("{public}-0")),
            Some(suffix) => {
                let (head, last) = match suffix.rsplit_once('.') {
                    Some((head, last)) => (Some(head), last),
                    None => (None, suffix),
                };
                let n: u64 = last.parse().map_err(|_| {
                    format!(
                        "the suffix of '{current}' ends '{last}', not a number, so `next` has \
                         nothing to add one to: name the version"
                    )
                })?;
                Ok(match head {
                    Some(head) => format!("{public}-{head}.{}", n + 1),
                    None => format!("{public}-{}", n + 1),
                })
            }
        },
    }
}

/// The package version in a manifest's text.
fn version_in(manifest: &str) -> Result<String, String> {
    let doc: DocumentMut = manifest
        .parse()
        .map_err(|e| format!("Cargo.toml does not parse: {e}"))?;
    doc.get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| "Cargo.toml has no `[package] version`".to_string())
}

/// A manifest's text with the package version set to `version`, the
/// rest as it was, comments and layout included.
fn with_version(manifest: &str, version: &str) -> Result<String, String> {
    let mut doc: DocumentMut = manifest
        .parse()
        .map_err(|e| format!("Cargo.toml does not parse: {e}"))?;
    let package = doc
        .get_mut("package")
        .ok_or_else(|| "Cargo.toml has no `[package]`".to_string())?;
    let slot = package
        .get_mut("version")
        .ok_or_else(|| "Cargo.toml has no `[package] version`".to_string())?;
    // The decor, a trailing comment on the line among it, stays.
    let decor = slot.as_value().map(|v| v.decor().clone());
    *slot = value(version);
    if let (Some(decor), Some(v)) = (decor, slot.as_value_mut()) {
        *v.decor_mut() = decor;
    }
    Ok(doc.to_string())
}

/// The workspace root, the directory above this package's.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// Refresh `Cargo.lock` for the workspace's own packages, offline, so
/// a `--locked` build finds the version the manifest now has.
fn refresh_lock(root: &Path) -> Result<(), String> {
    // `CARGO` is the cargo that runs this task, absent only when the
    // binary is run by hand.
    let cargo = match std::env::var_os("CARGO") {
        Some(cargo) => cargo,
        None => "cargo".into(),
    };
    // The root's clippy.toml bans spawning for `vc-x1`, which goes
    // through jj-lib. A project task running cargo is the exception.
    #[allow(clippy::disallowed_methods)]
    let status = std::process::Command::new(cargo)
        .args(["update", "--workspace", "--offline", "--quiet"])
        .current_dir(root)
        .status()
        .map_err(|e| format!("cargo update did not run: {e}"))?;
    match status.success() {
        true => Ok(()),
        false => Err(format!(
            "cargo update --workspace --offline failed: {status}"
        )),
    }
}

/// Run a task, returning what it prints.
fn run(args: &[String]) -> Result<String, String> {
    let manifest = root().join("Cargo.toml");
    let read = || std::fs::read_to_string(&manifest).map_err(|e| format!("Cargo.toml: {e}"));
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["version"] => version_in(&read()?),
        ["bump", to] => {
            let bump = match *to {
                "next" => Bump::Next,
                "bare" => Bump::Bare,
                version => Bump::To(version.to_string()),
            };
            let text = read()?;
            let current = version_in(&text)?;
            let new = bumped(&current, &bump)?;
            if new == current {
                return Ok(format!("{current}, as it was"));
            }
            std::fs::write(&manifest, with_version(&text, &new)?)
                .map_err(|e| format!("Cargo.toml: {e}"))?;
            refresh_lock(&root())?;
            Ok(format!("{current} -> {new}, Cargo.lock refreshed"))
        }
        _ => Err("usage: cargo xtask version | cargo xtask bump <next|bare|VERSION>".to_string()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(said) => {
            println!("{said}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("xtask: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `next` walks the suffix, `bare` drops it, and a version named
    /// is taken as written when it is one.
    #[test]
    fn the_bump_follows_the_suffix_scheme() {
        let next = |v: &str| bumped(v, &Bump::Next);
        assert_eq!(next("0.85.22").unwrap(), "0.85.22-0");
        assert_eq!(next("0.85.22-0").unwrap(), "0.85.22-1");
        assert_eq!(next("0.85.22-9").unwrap(), "0.85.22-10");
        assert_eq!(next("0.85.22-3.1").unwrap(), "0.85.22-3.2");
        assert!(next("0.85.22-rc").unwrap_err().contains("not a number"));
        assert_eq!(bumped("0.85.22-12", &Bump::Bare).unwrap(), "0.85.22");
        assert_eq!(bumped("0.85.22", &Bump::Bare).unwrap(), "0.85.22");
        let to = |v: &str| bumped("0.85.22-1", &Bump::To(v.to_string()));
        assert_eq!(to("0.86.0-0").unwrap(), "0.86.0-0");
        for bad in ["", "1.0.0-1-2", "1.0.0+build", "1.0.0 "] {
            assert!(to(bad).is_err(), "{bad}");
        }
    }

    /// The version's value changes and nothing else does: the comments,
    /// the blank lines, the key order, and a dependency's own version.
    #[test]
    fn the_manifest_keeps_its_comments_and_layout() {
        let manifest = "\
[package]
name = \"vc-x1\"
version = \"0.85.22-8\" # the version-of-record
edition = \"2024\"

# A comment between tables.
[dependencies]
clap = { version = \"4.6\", features = [\"derive\"] }
# The rng jj-lib takes.
rand = \"0.10\"
";
        assert_eq!(version_in(manifest).unwrap(), "0.85.22-8");
        let new = with_version(manifest, "0.85.22-9").unwrap();
        assert_eq!(new, manifest.replace("0.85.22-8", "0.85.22-9"));
        assert!(with_version("[workspace]\n", "1.0.0").is_err());
    }
}

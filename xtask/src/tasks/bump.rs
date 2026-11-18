//! `cargo xtask bump <next|bare|VERSION>`: set the version in
//! `Cargo.toml` and refresh `Cargo.lock`.
//!
//! The manifest is edited by `toml_edit`, which keeps its comments,
//! layout, and key order and changes the one value.

use std::path::Path;
use std::process::ExitCode;

use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};
use toml_edit::{DocumentMut, value};

use super::version::{manifest, read, version_in, version_path};
use crate::util::{cargo_at, root, status};

/// The task's arguments.
#[derive(clap::Args, Debug)]
#[command(
    about = "Set the version in Cargo.toml and refresh Cargo.lock",
    long_about = "\
        Set the package version in Cargo.toml, its comments and layout kept,\n\
        and refresh Cargo.lock in the same step, the build being --locked.\n\n\
        `next` adds one to the suffix's last number, 0.85.22-8 to 0.85.22-9,\n\
        and gives a bare version its first, -0. `bare` drops the suffix, a\n\
        close-out's version. Anything else is the version as written, X.Y.Z\n\
        or X.Y.Z-<suffix>, with one `-` at most and no `+`."
)]
pub struct Args {
    /// `next`, `bare`, or the version as written
    #[arg(value_name = "next|bare|VERSION", add = ArgValueCandidates::new(candidates))]
    pub to: String,
}

/// Set the version and refresh the lock, saying what changed.
pub fn run(args: &Args) -> Result<ExitCode, String> {
    let bump = match args.to.as_str() {
        "next" => Bump::Next,
        "bare" => Bump::Bare,
        version => Bump::To(version.to_string()),
    };
    let path = manifest();
    let text = read(&path)?;
    let current = version_in(&text)?;
    let new = bumped(&current, &bump)?;
    if new == current {
        println!("{current}, as it was");
        return Ok(ExitCode::SUCCESS);
    }
    std::fs::write(&path, with_version(&text, &new)?).map_err(|e| format!("Cargo.toml: {e}"))?;
    refresh_lock(&root())?;
    println!("{current} -> {new}, Cargo.lock refreshed");
    Ok(ExitCode::SUCCESS)
}

/// What the argument completes to, beside a version as written.
fn candidates() -> Vec<CompletionCandidate> {
    vec![
        CompletionCandidate::new("next")
            .help(Some("one more than the suffix's last number".into())),
        CompletionCandidate::new("bare").help(Some("the version without its suffix".into())),
    ]
}

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

/// A manifest's text with the version-of-record, by [`version_path`],
/// set to `version`, the rest as it was, comments and layout included.
fn with_version(manifest: &str, version: &str) -> Result<String, String> {
    let mut doc: DocumentMut = manifest
        .parse()
        .map_err(|e| format!("Cargo.toml does not parse: {e}"))?;
    let path = version_path(&doc)?;
    let mut table = doc.as_item_mut();
    for key in path {
        table = &mut table[key];
    }
    let slot = &mut table["version"];
    // The decor, a trailing comment on the line among it, stays.
    let decor = slot.as_value().map(|v| v.decor().clone());
    *slot = value(version);
    if let (Some(decor), Some(v)) = (decor, slot.as_value_mut()) {
        *v.decor_mut() = decor;
    }
    Ok(doc.to_string())
}

/// Refresh `Cargo.lock` for the workspace's own packages, offline, so
/// a `--locked` build finds the version the manifest now has.
fn refresh_lock(root: &Path) -> Result<(), String> {
    let mut cmd = cargo_at(root, None);
    cmd.args(["update", "--workspace", "--offline", "--quiet"]);
    status(cmd, "cargo update --workspace --offline")
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
name = \"app\"
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
        let virtual_ws =
            "[workspace]\nmembers = [\"a\"]\n\n[workspace.package]\nversion = \"0.4.0\"\n";
        assert_eq!(
            with_version(virtual_ws, "0.4.1-0").unwrap(),
            virtual_ws.replace("0.4.0", "0.4.1-0")
        );
    }
}

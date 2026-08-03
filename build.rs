//! Export the resolved `jj-lib` version as `JJ_LIB_VERSION`.
//!
//! jj-lib exports no version constant of its own, so the only
//! statement of what this binary links against is the resolved
//! version in `Cargo.lock`:
//!
//! - the lock is read from `$CARGO_MANIFEST_DIR` directly, not by
//!   walking ancestors: we are not a workspace, and a walk can
//!   bind a sibling project's lock, which is worse than failing.
//! - a missing or jj-lib-less lock fails the build rather than
//!   emitting an "unknown" that would later be reported as fact.

use std::path::Path;

/// Extract `jj-lib`'s resolved version from `Cargo.lock` text.
///
/// Scans for the `[[package]]` block whose `name` is `jj-lib` and
/// returns that block's `version`. Line-based rather than a TOML
/// parse: build scripts get no dev-dependencies, and the lock's
/// shape is fixed by cargo.
fn jj_lib_version(lock: &str) -> Option<String> {
    let mut in_jj_lib = false;
    for line in lock.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            in_jj_lib = false;
        } else if let Some(rest) = line.strip_prefix("name = ") {
            in_jj_lib = rest.trim_matches('"') == "jj-lib";
        } else if let Some(rest) = line.strip_prefix("version = ")
            && in_jj_lib
        {
            return Some(rest.trim_matches('"').to_string());
        }
    }
    None
}

/// Read the lock beside `Cargo.toml` and emit the version env var.
fn main() {
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => dir,
        Err(e) => panic!("CARGO_MANIFEST_DIR unset: {e}"),
    };
    let lock_path = Path::new(&manifest_dir).join("Cargo.lock");
    println!("cargo::rerun-if-changed={}", lock_path.display());

    let lock = match std::fs::read_to_string(&lock_path) {
        Ok(text) => text,
        Err(e) => panic!("cannot read {}: {e}", lock_path.display()),
    };
    match jj_lib_version(&lock) {
        Some(version) => println!("cargo::rustc-env=JJ_LIB_VERSION={version}"),
        None => panic!("no jj-lib package in {}", lock_path.display()),
    }
}

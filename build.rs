//! Export the resolved `jj-lib` version as `JJ_LIB_VERSION`, and
//! guard the single-name convention.
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
//!
//! The name guard (chores-16, 0.78.3): a suffixed version marks a
//! dev rung, and a dev rung must never carry the stable package
//! name, or installing it replaces the stable `vc-x1` binary with
//! unpushed code. It lives here rather than in a `#[test]` because
//! `cargo install` never runs tests: a build script runs on every
//! cargo verb, so the forbidden combination fails to compile at
//! all, install included.

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

/// The single-name convention's guard: refuse to build a suffixed
/// (dev-rung) version under the stable package name.
///
/// Reported via the `cargo::error` directive rather than `panic!`:
/// cargo then prints one clean error line and fails the build,
/// instead of wrapping a build-script panic in exit-status and
/// backtrace noise.
fn guard_single_name() {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    if version.contains('-') && name == "vc-x1" {
        // The version is not echoed in the text: cargo's own
        // `error: <name>@<version>:` prefix already carries it.
        println!("cargo::error=");
        println!("cargo::error=A suffixed version is used with the stable package name, `vc-x1`");
        println!("cargo::error=so refusing to build `vc-x1` as it is the stable binary name.");
        println!("cargo::error=");
    }
}

/// Read the lock beside `Cargo.toml` and emit the version env var;
/// enforce the single-name guard first.
fn main() {
    guard_single_name();

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

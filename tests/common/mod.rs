//! Shared harness for CLI subprocess integration tests.
//!
//! Lives under `tests/common/` so Cargo treats it as a shared module
//! (a top-level `tests/common.rs` would be compiled as its own test
//! crate). Each `tests/<name>.rs` declares `mod common;` to opt in.
//!
//! Cargo compiles each `tests/*.rs` as its own crate; helpers used
//! by some test crates but not others get dead-code warnings in the
//! crates that don't reach them. The crate-level allow below mutes
//! that — it's the standard idiom for shared `tests/common/`.
//!
//! - `vc_x1()` returns a `Command` for the binary Cargo built for
//!   the current test crate and is the value returned by
//!   `env!("CARGO_BIN_EXE_vc-x1")`.
//! - `run_ok` / `run_err` wrap `Command::output` with a panic on
//!   the unexpected exit status, embedding stdout/stderr in the
//!   message so failures are debuggable.
//! - `CliFixture` owns a per-test tempdir under `$TMPDIR` and
//!   removes it on drop. Subprocesses get `HOME` overridden to a
//!   tempdir-scoped path so user config can't leak into or out of
//!   the test.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// Single source of truth for tempdir-root resolution, shared with
// the binary's unit-test harness via `#[path]` include. See
// `src/test_tmp_root.rs` for the rationale and dependency-free
// constraint.
#[path = "../../src/test_tmp_root.rs"]
mod test_tmp_root;
use test_tmp_root::{resolve_tmp_root, should_keep_tempdir};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Build a unique tempdir path for a CLI test.
///
/// Mirrors `src/test_helpers::unique_base`'s shape (timestamp +
/// per-process counter) but uses a `vc-x1-cli-test-` prefix to
/// distinguish from the in-process fixtures' `vc-x1-test-` paths.
pub fn unique_base(tag: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    resolve_tmp_root().join(format!("vc-x1-cli-test-{tag}-{ts}-{n}"))
}

/// Build a `Command` invoking the `vc-x1` binary that Cargo built
/// for this test crate. Cargo sets `CARGO_BIN_EXE_vc-x1` to the
/// full path of the freshly-built binary; this `Command` runs it.
pub fn vc_x1() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vc-x1"))
}

/// Run `cmd`, panicking with stdout/stderr if it does not exit 0.
pub fn run_ok(cmd: &mut Command) -> Output {
    let out = cmd.output().expect("spawn vc-x1");
    if !out.status.success() {
        panic!(
            "vc-x1 unexpectedly failed: status={:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
    }
    out
}

/// Run `cmd`, panicking with stdout/stderr if it exits 0.
pub fn run_err(cmd: &mut Command) -> Output {
    let out = cmd.output().expect("spawn vc-x1");
    if out.status.success() {
        panic!(
            "vc-x1 unexpectedly succeeded\n--- stdout ---\n{}\n--- stderr ---\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
    }
    out
}

/// RAII tempdir owner for CLI tests.
///
/// - `base` is a freshly-allocated unique tempdir; it does *not*
///   exist on disk until the caller (or the subprocess) creates it.
/// - `home` is a sibling under `base` reserved for `HOME` override
///   on subprocesses, so config reads don't escape the fixture.
/// - `Drop` removes the whole `base` tree (best-effort).
pub struct CliFixture {
    pub base: PathBuf,
    pub home: PathBuf,
}

impl CliFixture {
    /// Allocate a fresh fixture; create `home/` so `HOME=…` points
    /// at a real directory.
    pub fn new(tag: &str) -> Self {
        let base = unique_base(tag);
        let home = base.join("home");
        std::fs::create_dir_all(&home).expect("mkdir cli fixture home");
        CliFixture { base, home }
    }

    /// Build a `vc-x1` `Command` with `HOME` pointed at the
    /// fixture's isolated home dir, so user config can't leak.
    pub fn cmd(&self) -> Command {
        let mut c = vc_x1();
        c.env("HOME", &self.home);
        c
    }

    /// Convenience: join a relative path under `base`.
    pub fn path(&self, rel: impl AsRef<Path>) -> PathBuf {
        self.base.join(rel)
    }
}

impl Drop for CliFixture {
    /// Remove the fixture tree on drop. Suppressed when
    /// `$VC_X1_TEST_KEEP` is set — see
    /// `test_tmp_root::should_keep_tempdir`.
    fn drop(&mut self) {
        if should_keep_tempdir() {
            eprintln!("VC_X1_TEST_KEEP set; preserving {}", self.base.display());
        } else {
            let _ = std::fs::remove_dir_all(&self.base);
        }
    }
}

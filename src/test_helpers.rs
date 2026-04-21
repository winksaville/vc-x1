//! Shared test helpers for dual-repo integration tests.
//!
//! Provides a `Fixture` built on top of `crate::test_fixture::test_fixture`
//! plus a per-process unique-tempdir helper so parallel tests don't
//! collide. Lifted out of the `sync` test module (originally inline at
//! `sync.rs:521–560`) so `push`'s tests (0.37.0) and any future
//! subcommand's tests can sit on the same harness without
//! copy-paste drift.
//!
//! Test-only — the whole module is gated at its declaration site via
//! `#[cfg(test)] mod test_helpers;`.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::test_fixture::{TestFixtureArgs, test_fixture};

/// Per-process counter so same-nanosecond tempdir collisions yield
/// distinct paths when tests run in parallel.
static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Build a unique tempdir path for a test fixture.
///
/// Combines a nanosecond timestamp with a per-process atomic counter
/// so parallel tests and same-nanosecond collisions both yield
/// distinct paths. Prefix is `vc-x1-test-<tag>-<ts>-<n>` so callers
/// from different subcommand tests stay discoverable in `$TMPDIR`.
pub fn unique_base(tag: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0); // OK: clock error → 0 is harmless for unique tempdir naming
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("vc-x1-test-{tag}-{ts}-{n}"))
}

/// Owned dual-repo fixture with RAII cleanup.
///
/// Builds a fresh throwaway workspace under a unique tempdir via
/// `test_fixture`, exposing the two repo paths (`work` and
/// `work/.claude`). The tempdir tree is removed when the value is
/// dropped, so a panicking test still cleans up after itself.
pub struct Fixture {
    /// Root tempdir that owns both repos and their bare-git remotes.
    pub base: PathBuf,
    /// Code repo path (`<base>/work`).
    pub work: PathBuf,
    /// Bot session repo path (`<base>/work/.claude`).
    pub claude: PathBuf,
}

impl Fixture {
    /// Build a fresh fixture in a unique tempdir.
    pub fn new(tag: &str) -> Self {
        let base = unique_base(tag);
        let args = TestFixtureArgs {
            path: Some(base.clone()),
            with_pending: false,
            use_template: None,
        };
        test_fixture(&args).expect("build test fixture");
        let work = base.join("work");
        let claude = work.join(".claude");
        Fixture { base, work, claude }
    }

    /// Convenience: return both repo paths as a `Vec<PathBuf>`
    /// suitable for `sync_repos` (or any other `&[PathBuf]` API).
    pub fn repos(&self) -> Vec<PathBuf> {
        vec![self.work.clone(), self.claude.clone()]
    }
}

impl Drop for Fixture {
    /// Remove the fixture tree on drop. Best-effort; a failure here
    /// doesn't fail the test.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

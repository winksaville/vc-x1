//! Shared test helpers for dual-repo integration tests.
//!
//! Provides a `Fixture` that wraps `crate::init::init_with_symlink`
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

use crate::args::ScopeKind;
use crate::common::write_file;
use crate::config::RepoSelector;
use crate::init::{InitArgs, init_with_symlink};

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
/// Builds a fresh throwaway dual-repo set under a unique tempdir
/// by driving `init::init_with_symlink` with a path TARGET and
/// `--repo local=<base>` (so config lookup is skipped via the
/// resolve_repo short-circuit). `create_symlink=false` suppresses
/// the `~/.claude/projects/` side effect. Exposes the two repo
/// paths (`work` and `work/.claude`). The tempdir tree is removed
/// when the value is dropped, so a panicking test still cleans up
/// after itself.
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
        Self::new_opts(tag, false, None)
    }

    /// Build a fresh fixture with optional pending changes and a
    /// template seed.
    ///
    /// - `with_pending` — after init, write a small file into each
    ///   repo so `@` carries uncommitted changes (useful for
    ///   finalize / push tests).
    /// - `use_template` — `CODE[,BOT]` forwarded to init.
    pub fn new_opts(tag: &str, with_pending: bool, use_template: Option<String>) -> Self {
        let base = unique_base(tag);
        // init refuses to reuse an existing project_dir, so `base`
        // must not exist yet. `unique_base` already guarantees that.
        //
        // Path TARGET = `<base>/work` (workspace destination); the
        // basename ("work") becomes the repo name. `--repo local=<base>`
        // sets the bare-repo parent so the layout mirrors the old
        // `--repo-local <base>` + NAME=`work` shape:
        //   <base>/work/                  (code repo)
        //   <base>/work/.claude/          (bot session repo)
        //   <base>/remote-code.git        (code bare origin)
        //   <base>/remote-claude.git      (bot bare origin)
        let work_path = base.join("work");
        let args = InitArgs {
            target: work_path.to_string_lossy().into_owned(),
            name: None,
            account: None,
            repo: Some(RepoSelector {
                category: "local".to_string(),
                value: Some(base.to_string_lossy().into_owned()),
            }),
            scope: ScopeKind::CodeBot,
            private: false,
            dry_run: false,
            push_retries: 5,
            push_retry_delay: 3,
            use_template,
        };
        init_with_symlink(&args, false).expect("build test fixture via init");

        let work = base.join("work");
        let claude = work.join(".claude");

        if with_pending {
            write_file(&work.join("TODO.md"), "# TODO\n- first feature\n")
                .expect("write pending TODO.md");
            write_file(
                &claude.join("session-notes.md"),
                "# Session notes\n- simulated pending work\n",
            )
            .expect("write pending session-notes.md");
        }

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

/// Owned single-repo (POR) fixture with RAII cleanup.
///
/// Sibling of `Fixture` for `--scope=por` flows. Drives
/// `init::init_with_symlink` with `ScopeKind::Por` and a path
/// TARGET; `--repo local=<base>` steers the bare origin to
/// `<base>/remote.git` (vs. dual's `remote-code.git` /
/// `remote-claude.git`). No `.claude/` peer, no symlink.
///
/// Field shape differs from `Fixture` — there is no `claude` peer
/// path — so it's a distinct type rather than an `Option<PathBuf>`
/// on `Fixture` (the latter would force every dual-using caller to
/// `.unwrap()` or pattern-match).
pub struct FixturePor {
    /// Root tempdir that owns the repo and its bare-git remote.
    pub base: PathBuf,
    /// POR repo path (`<base>/work`).
    pub work: PathBuf,
}

impl FixturePor {
    /// Build a fresh POR fixture in a unique tempdir.
    pub fn new(tag: &str) -> Self {
        let base = unique_base(tag);
        // Path TARGET = `<base>/work`; basename ("work") becomes
        // the repo name. `--repo local=<base>` sets the bare-repo
        // parent, producing the layout:
        //   <base>/work/        (POR repo)
        //   <base>/remote.git   (bare origin)
        let work_path = base.join("work");
        let args = InitArgs {
            target: work_path.to_string_lossy().into_owned(),
            name: None,
            account: None,
            repo: Some(RepoSelector {
                category: "local".to_string(),
                value: Some(base.to_string_lossy().into_owned()),
            }),
            scope: ScopeKind::Por,
            private: false,
            dry_run: false,
            push_retries: 5,
            push_retry_delay: 3,
            use_template: None,
        };
        init_with_symlink(&args, false).expect("build test fixture via init (POR)");

        let work = base.join("work");
        FixturePor { base, work }
    }
}

impl Drop for FixturePor {
    /// Remove the fixture tree on drop. Best-effort; a failure here
    /// doesn't fail the test.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

//! CLI subprocess tests for `vc-x1 sync`: the "two machines" flow.
//!
//! Drives the whole scenario through the binary: `vc-x1 init` a
//! project with local bare remotes, `vc-x1 clone` it twice (trA /
//! trB), change + `vc-x1 push` on trA, then `vc-x1 sync` on trB and
//! verify trB's `main` moved to the pushed head. Raw `jj` is used
//! only to inspect results, never to mutate.
//!
//! The two init/clone incompatibilities this test used to work
//! around inline (bugs.md #1 and #2) were fixed at 0.76.0-5:
//! init's local bares are created with `--initial-branch=main`,
//! and its bot bare is `derive_bot_url` of the work bare, so
//! init's naming and clone's derivation agree. A relative
//! local-path TARGET now resolves on both sides too, so the
//! absolute-path requirement is gone.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::{CliFixture, cid, run_err, run_ok};

/// Directory holding the binary under test, for prepending to
/// `PATH` so subcommands that re-invoke the binary (push
/// preflight) find the same build. The env-var name is
/// built from `CARGO_PKG_NAME` so a Cargo.toml rename needs no
/// edit here.
fn bin_dir() -> PathBuf {
    Path::new(env!(concat!("CARGO_BIN_EXE_", env!("CARGO_PKG_NAME"))))
        .parent()
        .expect("binary has a parent dir")
        .to_path_buf()
}

/// `PATH` value with the test binary's dir prepended.
fn test_path() -> String {
    let orig = std::env::var("PATH").unwrap_or_default();
    format!("{}:{orig}", bin_dir().display())
}

/// Write a minimal jj user config under the fixture's isolated HOME
/// so jj invocations spawned inside `vc-x1` have an identity.
/// Fixture state after the shared peer-push setup: two clones of
/// one init'ed project, with trA's pushed head recorded.
struct PeerPush {
    fx: CliFixture,
    tr_b: PathBuf,
    /// Commit id of the change trA pushed to the shared remote.
    pushed: String,
}

/// Shared setup: `vc-x1 init` with local bare remotes, `vc-x1
/// clone` trB then trA, change + `vc-x1 push` on trA.
///
/// Push notes: `--no-squash-push` stops before squash-push-bot
/// (out of scope); PATH carries the binary under test. Preflight
/// is gone as of 0.77.0-3, so no stage-skipping flag is needed,
/// which is what removing the `sync --check` self-spawn bought.
fn setup_peer_push(tag: &str) -> PeerPush {
    let fx = CliFixture::new(tag);

    // vc-x1 init ./tr --repo local=<base>
    run_ok(
        fx.cmd()
            .current_dir(&fx.base)
            .args(["init", "./tr"])
            .arg("--repo")
            .arg(format!("local={}", fx.base.display())),
    );

    // vc-x1 clone <work-remote> trB / trA (B first, so its
    // main@origin is the pre-push head). A relative source
    // exercises the bugs.md #2 fix: both sides resolve it
    // against the invoking cwd.
    let work_remote = PathBuf::from("./remote-work.git");
    for name in ["trB", "trA"] {
        run_ok(
            fx.cmd()
                .current_dir(&fx.base)
                .arg("clone")
                .arg(&work_remote)
                .arg(name),
        );
    }
    let tr_a = fx.path("trA");
    let tr_b = fx.path("trB");
    let pre_main = cid(&fx.home, &tr_b, "main");

    fs::write(tr_a.join("from-a.txt"), "from clone A\n").expect("write trA change");
    run_ok(fx.cmd().current_dir(&tr_a).env("PATH", test_path()).args([
        "push",
        "main",
        "--title",
        "feat: from clone A",
        "--body",
        "add from-a.txt",
        "--yes",
        "--no-squash-push",
    ]));
    let pushed = cid(&fx.home, &tr_a, "main");
    assert_ne!(pre_main, pushed, "trA's push should advance the remote");

    PeerPush { fx, tr_b, pushed }
}

/// Assert trB ended in the fully-synced state: work-repo `main` at
/// the pushed head with `@` repositioned onto it, and the bot
/// repo's `@` repositioned onto its `main`.
fn assert_trb_synced(p: &PeerPush) {
    let home = &p.fx.home;
    assert_eq!(
        cid(home, &p.tr_b, "main"),
        p.pushed,
        "trB's main should move to trA's pushed head"
    );
    assert_eq!(
        cid(home, &p.tr_b, "@-"),
        p.pushed,
        "trB's @ should be repositioned onto the new main"
    );
    let bot = p.tr_b.join(".claude");
    assert_eq!(
        cid(home, &bot, "@-"),
        cid(home, &bot, "main"),
        "trB/.claude's @ should be repositioned onto its main"
    );
}

/// Plain `vc-x1 sync` (the invocation a user reaches for): trB
/// must end fully synced: main at trA's pushed head, `@`
/// repositioned in both repos. Encodes the t1A/t1B transcript
/// (2026-07-02) where the pre-0.67.0 check-mode default left `@-`
/// on the pre-fetch tip; red until 0.67.0-2 made sync single-mode.
#[test]
fn cli_sync_default_moves_main_and_at_after_peer_push() {
    let p = setup_peer_push("sync-default-peer-push");
    run_ok(
        p.fx.cmd()
            .current_dir(&p.tr_b)
            .env("PATH", test_path())
            .arg("sync"),
    );
    assert_trb_synced(&p);
}

/// The hidden deprecated `--check` alias (push preflight's
/// verify-only shell-out) still parses and still skips the
/// reposition step: after it runs, trB's `main` has moved (jj's
/// fetch auto-ffs the tracked bookmark: the mode was never fully
/// read-only) but `@-` stays on the pre-fetch tip.
#[test]
fn cli_sync_check_alias_verifies_only() {
    let p = setup_peer_push("sync-check-peer-push");
    let pre_at_parent = cid(&p.fx.home, &p.tr_b, "@-");
    run_ok(
        p.fx.cmd()
            .current_dir(&p.tr_b)
            .env("PATH", test_path())
            .args(["sync", "--check"]),
    );
    assert_eq!(
        cid(&p.fx.home, &p.tr_b, "main"),
        p.pushed,
        "fetch auto-ff still moves trB's main under --check"
    );
    assert_eq!(
        cid(&p.fx.home, &p.tr_b, "@-"),
        pre_at_parent,
        "--check must not reposition @"
    );
}

/// `--no-check` is gone: a stale script invocation must fail
/// loudly rather than silently flip semantics.
#[test]
fn cli_sync_no_check_rejected() {
    let fx = CliFixture::new("sync-no-check-rejected");
    run_err(fx.cmd().current_dir(&fx.base).args(["sync", "--no-check"]));
}

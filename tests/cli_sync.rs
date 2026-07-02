//! CLI subprocess tests for `vc-x1 sync` — the "two machines" flow.
//!
//! Drives the whole scenario through the binary: `vc-x1 init` a
//! project with local bare remotes, `vc-x1 clone` it twice (trA /
//! trB), change + `vc-x1 push` on trA, then `vc-x1 sync` on trB and
//! verify trB's `main` moved to the pushed head. Raw `jj` is used
//! only to inspect results, never to mutate.
//!
//! Two init/clone incompatibilities are worked around inline (both
//! recorded in `notes/bugs.md`):
//!
//! - init's local bare remotes leave HEAD at `refs/heads/master`
//!   while the pushed branch is `main`, so a later `jj git clone`
//!   has no default branch to auto-track and `vc-x1 clone` errors
//!   → point HEAD at `main` with `git symbolic-ref`.
//! - init names the session remote `remote-claude.git`, but clone
//!   derives `<code-source-stem>.claude.git` → symlink
//!   `remote-code.claude.git` → `remote-claude.git`.

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use common::{CliFixture, run_ok};

/// Directory holding the `vc-x1` binary under test, for prepending
/// to `PATH` so subcommands that re-invoke `vc-x1` (push preflight,
/// finalize) find the same binary.
fn bin_dir() -> PathBuf {
    Path::new(env!("CARGO_BIN_EXE_vc-x1"))
        .parent()
        .expect("binary has a parent dir")
        .to_path_buf()
}

/// `PATH` value with the test binary's dir prepended.
fn test_path() -> String {
    let orig = std::env::var("PATH").unwrap_or_default();
    format!("{}:{orig}", bin_dir().display())
}

/// Run `jj <args>` in `dir` (inspection only) with `HOME` pointed
/// at the fixture's isolated home; assert success and return
/// trimmed stdout.
fn jj(home: &Path, dir: &Path, args: &[&str]) -> String {
    let out = Command::new("jj")
        .args(args)
        .env("HOME", home)
        .current_dir(dir)
        .output()
        .expect("spawn jj");
    assert!(
        out.status.success(),
        "jj {args:?} failed in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Resolve `rev` to its short commit id in `repo`.
fn cid(home: &Path, repo: &Path, rev: &str) -> String {
    jj(
        home,
        repo,
        &["log", "-r", rev, "--no-graph", "-T", "commit_id.short(12)"],
    )
}

/// Write a minimal jj user config under the fixture's isolated HOME
/// so jj invocations spawned inside `vc-x1` have an identity.
fn write_jj_config(home: &Path) {
    let dir = home.join(".config/jj");
    fs::create_dir_all(&dir).expect("mkdir jj config dir");
    fs::write(
        dir.join("config.toml"),
        "[user]\nname = \"cli-test\"\nemail = \"cli-test@example.com\"\n",
    )
    .expect("write jj config");
}

/// Point a bare remote's HEAD at `main` (init leaves it at
/// `refs/heads/master` — see module docs).
fn set_head_main(bare: &Path) {
    let out = Command::new("git")
        .args(["symbolic-ref", "HEAD", "refs/heads/main"])
        .current_dir(bare)
        .output()
        .expect("spawn git symbolic-ref");
    assert!(
        out.status.success(),
        "git symbolic-ref failed in {}: {}",
        bare.display(),
        String::from_utf8_lossy(&out.stderr)
    );
}

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
/// Push notes: `--from review` skips the cargo preflight (the
/// fixture is not a Rust project); `--no-finalize` stops before
/// finalize-claude (out of scope, and its detached form logs to a
/// fixed /tmp path); PATH carries the binary under test for push's
/// internal `vc-x1` calls.
fn setup_peer_push(tag: &str) -> PeerPush {
    let fx = CliFixture::new(tag);
    write_jj_config(&fx.home);

    // vc-x1 init ./tr --repo local=<base>
    run_ok(
        fx.cmd()
            .current_dir(&fx.base)
            .args(["init", "./tr"])
            .arg("--repo")
            .arg(format!("local={}", fx.base.display())),
    );

    // Workarounds — see module docs.
    set_head_main(&fx.path("remote-code.git"));
    set_head_main(&fx.path("remote-claude.git"));
    std::os::unix::fs::symlink(
        fx.path("remote-claude.git"),
        fx.path("remote-code.claude.git"),
    )
    .expect("symlink session remote");

    // vc-x1 clone <code-remote> trB / trA (B first, so its
    // main@origin is the pre-push head). Absolute source path —
    // the derived session source is resolved from the clone's
    // target dir, so a relative path would not resolve.
    let code_remote = fx.path("remote-code.git");
    for name in ["trB", "trA"] {
        run_ok(
            fx.cmd()
                .current_dir(&fx.base)
                .arg("clone")
                .arg(&code_remote)
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
        "--from",
        "review",
        "--title",
        "feat: from clone A",
        "--body",
        "add from-a.txt",
        "--yes",
        "--no-finalize",
    ]));
    let pushed = cid(&fx.home, &tr_a, "main");
    assert_ne!(pre_main, pushed, "trA's push should advance the remote");

    PeerPush { fx, tr_b, pushed }
}

/// Assert trB ended in the fully-synced state: code-repo `main` at
/// the pushed head with `@` repositioned onto it, and the session
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
    let claude = p.tr_b.join(".claude");
    assert_eq!(
        cid(home, &claude, "@-"),
        cid(home, &claude, "main"),
        "trB/.claude's @ should be repositioned onto its main"
    );
}

/// Apply mode (`sync --no-check`): trB converges — passes today.
#[test]
fn cli_sync_no_check_moves_main_after_peer_push() {
    let p = setup_peer_push("sync-nocheck-peer-push");
    run_ok(
        p.fx.cmd()
            .current_dir(&p.tr_b)
            .env("PATH", test_path())
            .args(["sync", "--no-check"]),
    );
    assert_trb_synced(&p);
}

/// Default mode (plain `vc-x1 sync`, the invocation a user
/// reaches for): trB must end fully synced, exactly as in the
/// t1A/t1B transcript (2026-07-02) where it did not.
///
/// Ignored until the 0.67.0 single-mode sync lands: today the
/// default is `--check`, whose fetch auto-ffs `main` but skips the
/// reposition step, leaving `@-` on the pre-fetch tip. Run
/// explicitly with `cargo test -- --ignored` to see the failure.
#[test]
#[ignore = "fails until 0.67.0-2 single-mode sync (default must converge + reposition)"]
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

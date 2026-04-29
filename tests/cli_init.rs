//! CLI subprocess integration tests for `vc-x1 init`.
//!
//! Counterparts to the in-process fixture tests in
//! `src/init.rs::tests` (`por_fixture_*`, `dual_fixture_*`). Those
//! call `init_with_symlink` directly; these spawn the `vc-x1`
//! binary so argument parsing, exit codes, and the actual binary
//! Cargo built are exercised end-to-end.
//!
//! Each test uses `CliFixture` for HOME isolation — the symlink
//! `init` installs at `$HOME/.claude/projects/...` lands inside the
//! fixture's owned home dir and gets dropped with it.

mod common;

use common::{CliFixture, run_ok};

/// `vc-x1 init <base>/work --scope=por --repo=local=<base>` lays
/// down the POR layout: code repo at `<base>/work/`, no `.claude/`
/// peer, bare origin at `<base>/remote.git`.
#[test]
fn cli_init_por_creates_layout() {
    let fx = CliFixture::new("init-por-layout");
    let work = fx.path("work");
    let base_str = fx.base.to_string_lossy().into_owned();
    let work_str = work.to_string_lossy().into_owned();

    run_ok(
        fx.cmd()
            .arg("init")
            .arg(&work_str)
            .arg("--scope=por")
            .arg(format!("--repo=local={base_str}")),
    );

    assert!(work.exists() && work.is_dir(), "work dir present");
    assert!(
        !work.join(".claude").exists(),
        "POR layout must not have a .claude/ peer"
    );
    assert!(
        fx.path("remote.git").exists(),
        "POR uses <base>/remote.git as the bare origin"
    );
    assert!(
        !fx.path("remote-code.git").exists(),
        "dual-shape bares should be absent in POR"
    );
    assert!(
        !fx.path("remote-claude.git").exists(),
        "dual-shape bares should be absent in POR"
    );
}

/// `vc-x1 init <base>/work --scope=code,bot --repo=local=<base>`
/// lays down the dual layout: both repos and both bare origins.
#[test]
fn cli_init_dual_creates_layout() {
    let fx = CliFixture::new("init-dual-layout");
    let work = fx.path("work");
    let claude = work.join(".claude");
    let base_str = fx.base.to_string_lossy().into_owned();
    let work_str = work.to_string_lossy().into_owned();

    run_ok(
        fx.cmd()
            .arg("init")
            .arg(&work_str)
            .arg("--scope=code,bot")
            .arg(format!("--repo=local={base_str}")),
    );

    assert!(work.exists() && work.is_dir(), "work dir present");
    assert!(
        claude.exists() && claude.is_dir(),
        "nested .claude dir present"
    );
    assert!(
        fx.path("remote-code.git").exists(),
        "code-side bare origin present"
    );
    assert!(
        fx.path("remote-claude.git").exists(),
        "session-side bare origin present"
    );
    assert!(
        !fx.path("remote.git").exists(),
        "POR-shape bare must not appear in dual layout"
    );
}

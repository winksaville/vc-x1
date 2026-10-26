//! CLI subprocess test: the dr-1 fixture builder runs against the
//! vc-x1 this test build makes.
//!
//! The `dr1_*` tests read a built fixture and never run the builder,
//! so a change to a command the builder drives went unnoticed until
//! a workspace with no fixture needed one. This test builds the
//! fixture into its own tempdir, so the builder fails here the day
//! a command it drives changes.

mod common;

use std::process::Command;

use common::{CliFixture, isolate_config};

/// `build-dr-1.py --local` builds the fixture with this build's
/// vc-x1, and the fixture has both repos.
#[test]
#[ignore = "runs the dr-1 fixture builder"]
fn dr1_builder_runs_against_this_vc_x1() {
    let fx = CliFixture::new("dr1-builder");
    let fixtures = fx.path("fixtures");
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/support/fixtures/build-dr-1.py"
    );
    let vc = env!(concat!("CARGO_BIN_EXE_", env!("CARGO_PKG_NAME")));

    // Register entry 4 (clippy.toml): the fixture builder is test
    // fixture setup, run with the built binary as its vc-x1.
    #[allow(clippy::disallowed_methods)]
    let mut cmd = Command::new("python3");
    cmd.arg(script)
        .arg(&fixtures)
        .arg("--local")
        .env("VC_X1", vc)
        .current_dir(&fx.base);
    isolate_config(&mut cmd, &fx.home);
    let out = cmd.output().expect("spawn python3");
    assert!(
        out.status.success(),
        "build-dr-1.py failed: status={:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    let dr1 = fixtures.join("dr-1");
    for repo in [dr1.clone(), dr1.join(".claude")] {
        assert!(
            repo.join(".jj").is_dir() && repo.join(".git").exists(),
            "{} is not a colocated repo",
            repo.display()
        );
    }
}

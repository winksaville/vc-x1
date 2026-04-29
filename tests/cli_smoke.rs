//! CLI subprocess smoke tests.
//!
//! Pre-flight for the rest of the `tests/` crate: spawn `vc-x1`
//! with no-op-ish arguments and verify the harness wires up
//! correctly. If these fail, the more state-mutating CLI tests
//! aren't worth running yet.

mod common;

use common::{CliFixture, run_ok};

/// `vc-x1 --version` exits 0 and prints a line containing the
/// crate name. Pins that `env!("CARGO_BIN_EXE_vc-x1")` resolves and
/// the binary actually runs.
#[test]
fn cli_version_runs() {
    let fx = CliFixture::new("smoke-version");
    let out = run_ok(fx.cmd().arg("--version"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("vc-x1"),
        "expected --version output to mention 'vc-x1', got: {stdout:?}"
    );
}

/// `vc-x1 --help` exits 0 and lists at least one subcommand we know
/// to exist (`init`). Pins clap's help renderer + the subcommand
/// surface compiled into the test binary.
#[test]
fn cli_help_lists_init() {
    let fx = CliFixture::new("smoke-help");
    let out = run_ok(fx.cmd().arg("--help"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("init"),
        "expected --help to list 'init' subcommand, got: {stdout:?}"
    );
}

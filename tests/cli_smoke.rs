//! CLI subprocess smoke tests.
//!
//! Pre-flight for the rest of the `tests/` crate: spawn `vc-x1`
//! with no-op-ish arguments and verify the harness wires up
//! correctly. If these fail, the more state-mutating CLI tests
//! aren't worth running yet.

mod common;

use common::{CliFixture, run_err, run_ok};

/// `vc-x1 --version` exits 0 and prints a line containing the
/// crate name. Pins that the `CARGO_BIN_EXE_<bin-name>` macro
/// resolves and the binary actually runs.
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

/// `vc-x1 version` exits 0 and reports all three versions on
/// stdout. Outside a workspace the jj-data line says so rather
/// than being dropped, which is what makes the report usable when
/// the workspace is the thing that's broken.
#[test]
fn cli_version_subcommand_reports_all_three() {
    let fx = CliFixture::new("smoke-version-sub");
    let out = run_ok(fx.cmd().arg("version"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    for expected in ["vc-x1", "jj-lib ", "jj-data "] {
        assert!(
            stdout.contains(expected),
            "expected `version` output to contain {expected:?}, got: {stdout:?}"
        );
    }
}

/// A mismatched `jj` stops every subcommand, is overridable for
/// one run, and leaves `version` answering.
///
/// Driven with a fake `jj` first on `PATH`, since the real pair
/// matches here and the refusal path is the half that matters.
#[test]
fn cli_version_gate_stops_every_subcommand() {
    let fx = CliFixture::new("smoke-gate");
    let bin = fx.path("fakebin");
    std::fs::create_dir_all(&bin).expect("mkdir fakebin");
    let jj = bin.join("jj");
    std::fs::write(&jj, "#!/bin/sh\necho \"jj 0.99.0-deadbeefcafe\"\n").expect("write fake jj");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&jj, std::fs::Permissions::from_mode(0o755))
            .expect("chmod fake jj");
    }
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    // Opens a repo: refuses, and says how to proceed.
    let out = run_err(fx.cmd().env("PATH", &path).arg("chid").arg("@"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("0.99.0") && stderr.contains("--allow-jj-mismatch"),
        "expected a mismatch refusal naming the override, got: {stderr:?}"
    );

    // Same run, overridden.
    let ok = run_ok(
        fx.cmd()
            .env("PATH", &path)
            .arg("--allow-jj-mismatch")
            .arg("chid")
            .arg("@"),
    );
    assert!(
        !String::from_utf8_lossy(&ok.stdout).is_empty(),
        "override should let the command run"
    );

    // No carve-out: even a markdown linter refuses, because a
    // list of exempt commands only enforces its own completeness.
    // The override is what keeps it usable.
    run_err(
        fx.cmd()
            .env("PATH", &path)
            .arg("validate-todo")
            .arg("TODO.md"),
    );
    run_ok(
        fx.cmd()
            .env("PATH", &path)
            .arg("--allow-jj-mismatch")
            .arg("validate-todo")
            .arg("TODO.md"),
    );

    // The diagnostic still answers, withholding only what it may
    // not read.
    let report = run_ok(fx.cmd().env("PATH", &path).arg("version"));
    let stdout = String::from_utf8_lossy(&report.stdout);
    assert!(
        stdout.contains("jj 0.99.0") && stdout.contains("jj-data withheld"),
        "expected a degraded report, got: {stdout:?}"
    );
}

/// `-VV` prints the full report on stdout and then runs the
/// subcommand anyway. The ride-along is the whole reason `-VV`
/// exists alongside the `version` subcommand, which can only
/// stand alone.
#[test]
fn cli_vv_reports_then_runs_the_subcommand() {
    let fx = CliFixture::new("smoke-vv-ride");
    let out = run_ok(fx.cmd().arg("-VV").arg("chid").arg("@"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("jj-lib ") && stdout.contains("jj-data "),
        "expected the full report on stdout, got: {stdout:?}"
    );
    assert!(
        stdout.lines().count() > 5,
        "expected the subcommand's own output after the report, got: {stdout:?}"
    );
    assert!(
        out.stderr.is_empty(),
        "-VV should not also print the ambient banner: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The ambient banner goes to stderr, never stdout, and
/// `--no-banner` silences it. Pins the property the split exists
/// for: a command that emits data on stdout stays pipeable, so
/// `--no-banner` is a convenience rather than a requirement.
///
/// Driven with `chid`, a command whose whole output is one
/// datum. `--help` would not exercise this: clap exits before
/// `main` reaches the banner, and help carries its own via
/// `before_help`.
#[test]
fn cli_ambient_banner_is_stderr_only() {
    let fx = CliFixture::new("smoke-banner-stream");
    let out = run_ok(fx.cmd().arg("chid").arg("@"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.lines().next().is_some_and(|l| l.contains("vc-x1")),
        "expected banner on stderr, got: {stderr:?}"
    );
    // The banner is `<invoked name> <version>`, and the version string
    // is its unique marker, robust to which bin name ran the test.
    let version = env!("CARGO_PKG_VERSION");
    assert!(
        !stdout.contains(version),
        "banner leaked onto stdout: {stdout:?}"
    );

    let quiet = run_ok(fx.cmd().arg("--no-banner").arg("chid").arg("@"));
    let quiet_err = String::from_utf8_lossy(&quiet.stderr);
    assert!(
        !quiet_err.contains(version),
        "--no-banner did not suppress the banner: {quiet_err:?}"
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

/// The pre-0.80.0 `validate-bot` name is rejected with the fix-it,
/// for any flags, rather than aliased.
#[test]
fn cli_validate_bot_old_name_rejected() {
    let fx = CliFixture::new("validate-bot-old-name");
    let out = run_err(fx.cmd().arg("validate-bot").arg("-R").arg("x"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("pre-0.80.0"), "got: {stderr}");
    assert!(stderr.contains("`validate-agent`"), "got: {stderr}");
}

/// The banner and the report carry the workspace's agent-files set
/// version, read from the name of `agent-data/agent-files-v*`, and
/// `agent-files version` prints it bare. In a workspace without the
/// file the banner stays plain and the subcommand says why.
#[test]
fn cli_reports_the_agent_files_set_version() {
    let fx = CliFixture::new("smoke-set-version");
    std::fs::write(fx.base.join(".vc-config.toml"), "[repos]\nwork = \".\"\n")
        .expect("write vc-config");
    let data = fx.base.join("agent-data");
    std::fs::create_dir_all(&data).expect("mkdir agent-data");
    std::fs::write(data.join("agent-files-v0.1.0"), "").expect("write version file");

    let out = run_ok(fx.cmd().current_dir(&fx.base).arg("-V"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("(agent-files v0.1.0)"),
        "expected the set version in the banner, got: {stdout:?}"
    );

    let out = run_ok(fx.cmd().current_dir(&fx.base).arg("version"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.lines().any(|l| l == "agent-files v0.1.0"),
        "expected an agent-files line in the report, got: {stdout:?}"
    );

    let out = run_ok(
        fx.cmd()
            .current_dir(&fx.base)
            .arg("agent-files")
            .arg("version"),
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "v0.1.0\n");

    // A second workspace with no `agent-data`, its own config so the
    // root finder stops there rather than walking up into the first.
    let outside = fx.path("outside");
    std::fs::create_dir_all(&outside).expect("mkdir outside");
    std::fs::write(outside.join(".vc-config.toml"), "[repos]\nwork = \".\"\n")
        .expect("write outside vc-config");
    let out = run_ok(fx.cmd().current_dir(&outside).arg("-V"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("agent-files"),
        "expected a plain banner without a version file, got: {stdout:?}"
    );
    let out = run_err(
        fx.cmd()
            .current_dir(&outside)
            .arg("agent-files")
            .arg("version"),
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("agent-files none: no agent-data/agent-files-v* file"),
        "expected the none reason, got: {stderr:?}"
    );
}

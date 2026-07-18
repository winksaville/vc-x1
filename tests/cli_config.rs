//! CLI subprocess tests for `config`: pin the annotated-schema
//! print, the `--home` filter, and `--validate`'s unknown-key
//! check.

mod common;

use common::{CliFixture, run_err, run_ok};

/// `vc-x1 config` (default `--home all`) prints both home groups:
/// bot-session keys (present in both), the user-only `[default]` /
/// `[repo]` sections, and the workspace `[workspace]` / `[push]`
/// sections.
#[test]
fn cli_config_default() {
    let fx = CliFixture::new("config-default");
    let out = run_ok(fx.cmd().arg("config"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("col-width"), "got: {stdout}");
    assert!(stdout.contains("68"), "got: {stdout}");
    assert!(
        stdout.contains("used by: bot-session --col-width"),
        "got: {stdout}"
    );
    assert!(stdout.contains("[workspace]"), "got: {stdout}");
    assert!(stdout.contains("path"), "got: {stdout}");
    assert!(stdout.contains("push-state.toml"), "got: {stdout}");
    assert!(stdout.contains("[default]"), "got: {stdout}");
    assert!(stdout.contains("[repo]"), "got: {stdout}");
}

/// `--home user` prints only the User group: `[default]` shows up,
/// `[workspace]` / `[push]` (workspace-only sections) do not.
#[test]
fn cli_config_home_user() {
    let fx = CliFixture::new("config-home-user");
    let out = run_ok(fx.cmd().arg("config").arg("--home").arg("user"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("[default]"), "got: {stdout}");
    assert!(!stdout.contains("[workspace]"), "got: {stdout}");
    assert!(!stdout.contains("[push]"), "got: {stdout}");
}

/// `--home workspace` prints only the Workspace group:
/// `[workspace]` and the push-state default show up; the
/// user-only `[default]` section does not.
#[test]
fn cli_config_home_workspace() {
    let fx = CliFixture::new("config-home-workspace");
    let out = run_ok(fx.cmd().arg("config").arg("--home").arg("workspace"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("[workspace]"), "got: {stdout}");
    assert!(stdout.contains("push-state.toml"), "got: {stdout}");
    assert!(
        stdout.contains("used by: push / squash-push (state-file name)"),
        "got: {stdout}"
    );
    assert!(!stdout.contains("[default]"), "got: {stdout}");
}

/// `--validate --home workspace` against a clean workspace config
/// (a valid `[bot-session].col-width`) exits 0 and reports all
/// keys recognized.
#[test]
fn cli_config_validate_clean() {
    let fx = CliFixture::new("config-validate-clean");
    std::fs::write(
        fx.base.join(".vc-config.toml"),
        "[workspace]\npath = \"/\"\n\n[bot-session]\ncol-width = 40\n",
    )
    .expect("write vc-config");
    let out = run_ok(fx.cmd().current_dir(&fx.base).arg("config").args([
        "--validate",
        "--home",
        "workspace",
    ]));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("all keys recognized"), "got: {stdout}");
}

/// `--validate --home workspace` against a workspace config with a
/// typo'd key exits non-zero and names the unknown key.
#[test]
fn cli_config_validate_unknown() {
    let fx = CliFixture::new("config-validate-unknown");
    std::fs::write(
        fx.base.join(".vc-config.toml"),
        "[workspace]\npath = \"/\"\n\n[bot-session]\ncol-widht = 40\n\n[push]\nstate-fil = \"x\"\n",
    )
    .expect("write vc-config");
    let out = run_err(fx.cmd().current_dir(&fx.base).arg("config").args([
        "--validate",
        "--home",
        "workspace",
    ]));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("bot-session.col-widht"), "got: {stderr}");
    assert!(stderr.contains("2 unknown key(s) found"), "got: {stderr}");
}

/// `--validate --home user` against a user config using the
/// dynamic `repo.category.<cat>` family exits 0 — the placeholder
/// matches the concrete `remote` segment.
#[test]
fn cli_config_validate_dynamic_ok() {
    let fx = CliFixture::new("config-validate-dynamic");
    let cfg_dir = fx.home.join(".config").join("vc-x1");
    std::fs::create_dir_all(&cfg_dir).expect("mkdir config dir");
    std::fs::write(
        cfg_dir.join("config.toml"),
        "[repo.category]\nremote = \"foo\"\n\n[default]\naccount = \"x\"\n",
    )
    .expect("write user config");
    let out = run_ok(
        fx.cmd()
            .env("XDG_CONFIG_HOME", fx.home.join(".config"))
            .arg("config")
            .args(["--validate", "--home", "user"]),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("all keys recognized"), "got: {stdout}");
}

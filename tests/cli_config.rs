//! CLI subprocess tests for `config`: pin the annotated-schema
//! print and the `--home` filter.

mod common;

use common::{CliFixture, run_ok};

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
    assert!(!stdout.contains("[default]"), "got: {stdout}");
}

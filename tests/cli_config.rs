//! CLI subprocess tests for `config`: pin the annotated-schema
//! print, the positional target (side keywords / explicit path),
//! and `--validate`'s unknown-key check.

mod common;

use common::{CliFixture, run_err, run_ok};

/// `vc-x1 config` (default target `work,bot`) prints both side
/// groups of workspace keys — bot-session keys (settable on both
/// sides), `[workspace]`, `[push]` — and no longer prints the
/// user-only `[default]` / `[repo]` sections (the user config is
/// reached only by path).
#[test]
fn cli_config_default() {
    let fx = CliFixture::new("config-default");
    let out = run_ok(fx.cmd().current_dir(&fx.base).arg("config"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("# ── work: "), "got: {stdout}");
    assert!(stdout.contains("# ── bot: "), "got: {stdout}");
    assert!(stdout.contains("col-width"), "got: {stdout}");
    assert!(
        stdout.contains("used by: bot-session --col-width"),
        "got: {stdout}"
    );
    assert!(stdout.contains("[workspace]"), "got: {stdout}");
    assert!(stdout.contains("push-state.toml"), "got: {stdout}");
    assert!(!stdout.contains("[default]"), "got: {stdout}");
    assert!(!stdout.contains("[repo]"), "got: {stdout}");
    assert!(
        stdout.contains("# col-width = 68") && !stdout.contains("68   # example"),
        "got: {stdout}"
    );
}

/// `config work` prints only the Work group: `[workspace]` and the
/// push-state default show up once; no Bot group, no user-only
/// `[default]` section.
#[test]
fn cli_config_work_target() {
    let fx = CliFixture::new("config-work-target");
    let out = run_ok(fx.cmd().current_dir(&fx.base).arg("config").arg("work"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("# ── work: "), "got: {stdout}");
    assert!(!stdout.contains("# ── bot: "), "got: {stdout}");
    assert!(stdout.contains("[workspace]"), "got: {stdout}");
    assert!(stdout.contains("push-state.toml"), "got: {stdout}");
    assert!(
        stdout.contains("used by: push / squash-push (state-file name)"),
        "got: {stdout}"
    );
    assert!(!stdout.contains("[default]"), "got: {stdout}");
}

/// `config <path>` prints the whole schema — a path carries no
/// side information, so every home's keys show under a divider
/// that is just the path: the user-only `[default]` section and
/// the workspace `[workspace]` / `[push]` sections all appear.
/// The user config has no keyword — the path is the way in.
#[test]
fn cli_config_user_path() {
    let fx = CliFixture::new("config-user-path");
    let user_cfg = fx.home.join(".config").join("vc-x1").join("config.toml");
    let out = run_ok(fx.cmd().current_dir(&fx.base).arg("config").arg(&user_cfg));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&format!("# ── {} ──", user_cfg.display())),
        "got: {stdout}"
    );
    assert!(stdout.contains("[default]"), "got: {stdout}");
    assert!(
        stdout.contains("account = \"work\"   # example"),
        "got: {stdout}"
    );
    assert!(stdout.contains("[workspace]"), "got: {stdout}");
    assert!(stdout.contains("[push]"), "got: {stdout}");
}

/// `config --validate` against a clean single-repo workspace (a
/// valid `[bot-session].col-width`) exits 0: the work side checks
/// out and the absent bot side is skipped with a note.
#[test]
fn cli_config_validate_clean() {
    let fx = CliFixture::new("config-validate-clean");
    std::fs::write(
        fx.base.join(".vc-config.toml"),
        "[workspace]\nwork = \"/\"\n\n[bot-session]\ncol-width = 40\n",
    )
    .expect("write vc-config");
    let out = run_ok(
        fx.cmd()
            .current_dir(&fx.base)
            .arg("config")
            .arg("--validate"),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("all checks passed"), "got: {stdout}");
    assert!(stdout.contains("no bot repo configured"), "got: {stdout}");
}

/// `config --validate` against a workspace config with typo'd keys
/// exits non-zero and names each unknown key.
#[test]
fn cli_config_validate_unknown() {
    let fx = CliFixture::new("config-validate-unknown");
    std::fs::write(
        fx.base.join(".vc-config.toml"),
        "[workspace]\nwork = \"/\"\n\n[bot-session]\ncol-widht = 40\n\n[push]\nstate-fil = \"x\"\n",
    )
    .expect("write vc-config");
    let out = run_err(
        fx.cmd()
            .current_dir(&fx.base)
            .arg("config")
            .arg("--validate"),
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("bot-session.col-widht"), "got: {stderr}");
    assert!(stderr.contains("2 problem(s) found"), "got: {stderr}");
}

/// `config <path> --validate` against a user config using the
/// dynamic `repo.category.<cat>` family exits 0 — a path target
/// validates against the whole schema, and the placeholder matches
/// the concrete `remote` segment.
#[test]
fn cli_config_validate_dynamic_ok() {
    let fx = CliFixture::new("config-validate-dynamic");
    let cfg_dir = fx.home.join(".config").join("vc-x1");
    std::fs::create_dir_all(&cfg_dir).expect("mkdir config dir");
    let user_cfg = cfg_dir.join("config.toml");
    std::fs::write(
        &user_cfg,
        "[repo.category]\nremote = \"foo\"\n\n[default]\naccount = \"x\"\n",
    )
    .expect("write user config");
    let out = run_ok(
        fx.cmd()
            .current_dir(&fx.base)
            .arg("config")
            .arg(&user_cfg)
            .arg("--validate"),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("all checks passed"), "got: {stdout}");
}

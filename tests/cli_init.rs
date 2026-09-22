//! CLI subprocess integration tests for `vc-x1 init`.
//!
//! Counterparts to the fixture tests in `src/init.rs::tests`
//! (`por_fixture_*`, `dual_fixture_*`). Those call the functions
//! directly; these spawn the `vc-x1` binary so argument parsing,
//! exit codes, and the actual binary Cargo built are exercised
//! end-to-end.
//!
//! Each test uses `CliFixture` for HOME isolation: the symlink
//! `init` installs at `$HOME/.claude/projects/...` lands inside the
//! fixture's owned home dir and gets dropped with it.

mod common;

use common::{CliFixture, run_err, run_ok};

/// `vc-x1 init <base>/work --por --repo=local=<base>` lays down
/// the POR layout: work repo at `<base>/work/`, no `.agent-session/`
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
            .arg("--por")
            .arg(format!("--repo=local={base_str}")),
    );

    assert!(work.exists() && work.is_dir(), "work dir present");
    assert!(
        !work.join(".agent-session").exists(),
        "POR layout must not have a .agent-session/ peer"
    );
    assert!(
        fx.path("remote.git").exists(),
        "POR uses <base>/remote.git as the bare origin"
    );
    assert!(
        !fx.path("remote-work.git").exists(),
        "dual-shape bares should be absent in POR"
    );
    assert!(
        !fx.path("remote-work.agent-session.git").exists(),
        "dual-shape bares should be absent in POR"
    );
}

/// `vc-x1 init <base>/work --repo=local=<base>` (no `--por`)
/// lays down the dual layout: both repos and both bare origins.
#[test]
fn cli_init_dual_creates_layout() {
    let fx = CliFixture::new("init-dual-layout");
    let work = fx.path("work");
    let bot = work.join(".agent-session");
    let base_str = fx.base.to_string_lossy().into_owned();
    let work_str = work.to_string_lossy().into_owned();

    run_ok(
        fx.cmd()
            .arg("init")
            .arg(&work_str)
            .arg(format!("--repo=local={base_str}")),
    );

    assert!(work.exists() && work.is_dir(), "work dir present");
    assert!(
        bot.exists() && bot.is_dir(),
        "nested .agent-session dir present"
    );
    assert!(
        fx.path("remote-work.git").exists(),
        "work-side bare origin present"
    );
    assert!(
        fx.path("remote-work.agent-session.git").exists(),
        "bot-side bare origin present"
    );
    assert!(
        !fx.path("remote.git").exists(),
        "POR-shape bare must not appear in dual layout"
    );
}

/// The output a run printed, both streams, since the log goes to
/// stderr.
fn output_text(out: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// The commit `rev` names in the bare repo at `bare`, read with the
/// real git: an inspection, never a mutation.
fn git_rev(bare: &std::path::Path, rev: &str) -> String {
    // Register entry 4 (clippy.toml): test inspection with the real git.
    #[allow(clippy::disallowed_methods)]
    let out = std::process::Command::new("git")
        .arg("--git-dir")
        .arg(bare)
        .args(["rev-parse", rev])
        .output()
        .expect("spawn git");
    assert!(
        out.status.success(),
        "git rev-parse {rev} in {}",
        bare.display()
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// The one symlink `init` installed under the fixture's
/// `~/.claude/projects/`, and where it points.
fn installed_symlink(fx: &CliFixture) -> std::path::PathBuf {
    let projects = fx.home.join(".claude").join("projects");
    let entries: Vec<_> = std::fs::read_dir(&projects)
        .expect("the symlink's directory exists")
        .map(|e| e.expect("read entry").path())
        .collect();
    assert_eq!(entries.len(), 1, "one symlink: {entries:?}");
    std::fs::read_link(&entries[0]).expect("the entry is a symlink")
}

/// `vc-x1 init <work> --adopt` on a plain directory runs all ten
/// steps through the binary, the symlink included: both repos and
/// both bare origins appear, a file over jj's new-file limit is named
/// and committed, and the symlink points at the agent repo.
#[test]
fn cli_init_adopt_plain_directory() {
    let fx = CliFixture::new("init-adopt-plain");
    let work = fx.path("work");
    std::fs::create_dir_all(&work).expect("mkdir work");
    std::fs::write(work.join("notes.txt"), "a record\n").expect("write notes");
    std::fs::write(work.join("big.jsonl"), vec![b'x'; 2 * 1024 * 1024]).expect("write big");

    let out = run_ok(
        fx.cmd()
            .arg("init")
            .arg(&work)
            .arg("--adopt")
            .arg(format!("--repo=local={}", fx.base.display())),
    );
    let text = output_text(&out);

    for n in 1..=10 {
        assert!(text.contains(&format!("Step {n}: ")), "step {n}: {text}");
    }
    assert!(!text.contains("Step 11:"), "ten steps: {text}");
    assert!(
        text.contains("big.jsonl"),
        "the large file is named: {text}"
    );
    assert!(work.join(".jj").exists() && work.join(".agent-session/.jj").exists());
    assert!(fx.path("remote-work.git").exists());
    assert!(fx.path("remote-work.agent-session.git").exists());
    assert_eq!(installed_symlink(&fx), work.join(".agent-session"));
}

/// `vc-x1 init <work> --adopt` on the single-repo workspace
/// `init --por` makes edits its config in place, pushes nothing on
/// the work side, publishes the agent repo beside the origin, and
/// installs the symlink.
#[test]
fn cli_init_adopt_single_repo_workspace() {
    let fx = CliFixture::new("init-adopt-single");
    let work = fx.path("work");
    run_ok(
        fx.cmd()
            .arg("init")
            .arg(&work)
            .arg("--por")
            .arg(format!("--repo=local={}", fx.base.display())),
    );

    let origin_before = git_rev(&fx.path("remote.git"), "main");

    let out = run_ok(fx.cmd().arg("init").arg(&work).arg("--adopt"));
    let text = output_text(&out);

    assert!(
        text.contains("Step 1: Add the agent repo to the work repo's config, in place"),
        "{text}"
    );
    assert!(
        !text.contains("Set main on the work repo"),
        "no work publish step: {text}"
    );
    assert_eq!(
        git_rev(&fx.path("remote.git"), "main"),
        origin_before,
        "the work side's origin is untouched"
    );
    let config = std::fs::read_to_string(work.join(".vc-config.md")).expect("read config");
    assert!(config.contains("agent = \".agent-session\""), "{config}");
    assert!(
        config.contains("agent-repo = \"remote.agent-session\""),
        "{config}"
    );
    assert!(
        fx.path("remote.agent-session.git").exists(),
        "agent bare beside the origin"
    );
    assert_eq!(installed_symlink(&fx), work.join(".agent-session"));
}

/// An existing target without `--adopt`, and `--adopt` on a missing
/// one, fail with the message that says what would work.
#[test]
fn cli_init_existing_target_refusals() {
    let fx = CliFixture::new("init-adopt-refusals");
    let work = fx.path("work");
    std::fs::create_dir_all(&work).expect("mkdir work");
    let local = format!("--repo=local={}", fx.base.display());

    let text = output_text(&run_err(fx.cmd().arg("init").arg(&work).arg(&local)));
    assert!(text.contains("pass --adopt"), "{text}");

    let missing = fx.path("missing");
    let text = output_text(&run_err(
        fx.cmd()
            .arg("init")
            .arg(&missing)
            .arg("--adopt")
            .arg(&local),
    ));
    assert!(text.contains("does not exist"), "{text}");
    assert!(!work.join(".jj").exists(), "nothing written");
}

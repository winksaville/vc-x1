//! CLI subprocess tests for `vc-x1 clone`'s destination.
//!
//! The destination names the Claude Code symlink, and Claude Code
//! looks for the one named from the real working directory, which
//! has no `.` or `..` in it. A destination given as `./name` must
//! come out the same as `name`.

mod common;

use common::{CliFixture, jj, run_ok};

/// `clone <source> ./name` prints and links the normalized path: no
/// `/./` in the output, and the one symlink named as Claude Code will
/// derive it from `<base>/name`.
#[test]
fn clone_into_dot_slash_name_links_the_normalized_path() {
    let fx = CliFixture::new("clone-dot-slash");
    run_ok(
        fx.cmd()
            .current_dir(&fx.base)
            .args(["init", "./tr"])
            .arg("--repo")
            .arg(format!("local={}", fx.base.display())),
    );
    // init installed a symlink for `tr`; clear it so the clone's is
    // the only one.
    let projects = fx.home.join(".claude").join("projects");
    std::fs::remove_dir_all(&projects).expect("clear init's symlink");

    let out = run_ok(
        fx.cmd()
            .current_dir(&fx.base)
            .args(["clone", "./remote-work.git", "./cl"]),
    );
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!text.contains("/./"), "no `/./` in the paths: {text}");

    let cl = fx.path("cl");
    let expected: String = cl
        .to_string_lossy()
        .chars()
        .map(|c| if c == '/' || c == '.' { '-' } else { c })
        .collect();
    let entries: Vec<_> = std::fs::read_dir(&projects)
        .expect("the symlink's directory exists")
        .map(|e| {
            e.expect("read entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(entries, vec![expected], "named from the normalized path");
    assert_eq!(
        std::fs::read_link(projects.join(&entries[0])).expect("a symlink"),
        cl.join(".agent-session")
    );
}

/// A config with no `[remote] agent-repo` names its agent-repo from
/// `repos.agent`: a `.claude` dir clones `remote-work.claude`, not the
/// `.agent-session` default, which here does not exist.
#[test]
fn clone_without_remote_agent_repo_names_it_from_the_agent_dir() {
    let fx = CliFixture::new("clone-agent-dir-name");
    run_ok(
        fx.cmd()
            .current_dir(&fx.base)
            .args([
                "init",
                "./tr",
                "--agent-dir",
                ".claude",
                "--agent-suffix",
                ".claude",
            ])
            .arg("--repo")
            .arg(format!("local={}", fx.base.display())),
    );
    assert!(fx.path("remote-work.claude.git").exists());
    assert!(!fx.path("remote-work.agent-session.git").exists());

    // Drop the key, as a workspace created before it reads, and
    // publish that to the work remote the clone reads.
    let tr = fx.path("tr");
    let cfg = tr.join(".vc-config.md");
    let text = std::fs::read_to_string(&cfg).expect("read config");
    let key = "[remote]\nagent-repo = \"remote-work.claude\"\n";
    assert!(text.contains(key), "init recorded the key: {text}");
    std::fs::write(&cfg, text.replace(key, "")).expect("write config");
    jj(&fx.home, &tr, &["commit", "-m", "drop [remote] agent-repo"]);
    jj(&fx.home, &tr, &["bookmark", "set", "main", "-r", "@-"]);
    jj(&fx.home, &tr, &["git", "push", "--bookmark", "main"]);

    let out = run_ok(
        fx.cmd()
            .current_dir(&fx.base)
            .args(["clone", "./remote-work.git", "./cl"]),
    );
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(text.contains("remote-work.claude.git"), "{text}");
    assert!(fx.path("cl/.claude/.jj").is_dir(), "{text}");
}

//! CLI subprocess integration tests for `vc-x1 delete`.
//!
//! Each test makes a workspace with `vc-x1 init --test-repo` under a
//! `CliFixture`, whose isolated `HOME` holds the Claude Code symlink
//! `init` installs, so nothing reaches the real `~/.claude/projects/`.

mod common;

use std::path::{Path, PathBuf};
use std::process::Output;

use common::{CliFixture, run_err, run_ok};

/// Both streams, since the log goes to stderr.
fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// A dual workspace `ws` with its bare remotes in `remotes`, made by
/// `init --test-repo`, and the symlink `init` installed.
struct Ws {
    root: PathBuf,
    remotes: PathBuf,
    link: PathBuf,
}

fn make_ws(fx: &CliFixture) -> Ws {
    let root = fx.path("ws");
    let remotes = fx.path("remotes");
    run_ok(
        fx.cmd()
            .arg("init")
            .arg("--target")
            .arg(&root)
            .arg("--test-repo")
            .arg(&remotes),
    );
    let projects = fx.home.join(".claude").join("projects");
    let link = std::fs::read_dir(&projects)
        .expect("the symlink's directory")
        .map(|e| e.expect("entry").path())
        .next()
        .expect("init installed a symlink");
    Ws {
        root: root.canonicalize().expect("root"),
        remotes,
        link,
    }
}

/// `vc-x1 delete <args>` from `dir`.
fn delete(fx: &CliFixture, dir: &Path, args: &[&str]) -> std::process::Command {
    let mut c = fx.cmd();
    c.arg("delete").args(args).current_dir(dir);
    c
}

/// Everything `init` made is still there.
fn all_there(ws: &Ws) {
    assert!(ws.root.join(".agent-session").is_dir(), "agent repo");
    assert!(ws.root.is_dir(), "work repo");
    assert!(ws.link.symlink_metadata().is_ok(), "symlink");
    assert!(ws.remotes.join("ws.git").is_dir(), "work origin");
    assert!(
        ws.remotes.join("ws.agent-session.git").is_dir(),
        "agent origin"
    );
}

/// A dry run names every part, the origins kept without
/// `--local-repos` and removed with it, from the root or from inside,
/// and removes nothing.
#[test]
fn a_dry_run_names_every_part_and_removes_nothing() {
    let fx = CliFixture::new("delete-dry-run");
    let ws = make_ws(&fx);
    let root = ws.root.to_str().expect("utf-8");

    let out = text(&run_ok(&mut delete(
        &fx,
        &fx.base,
        &["-s", "b", "--dry-run", root],
    )));
    assert!(
        out.contains(&format!("remove the symlink {}", ws.link.display())),
        "{out}"
    );
    assert!(
        out.contains(&format!("remove the work repo {root}")),
        "{out}"
    );
    assert!(out.contains("the agent repo inside it"), "{out}");
    assert!(out.contains("keep work's origin"), "{out}");
    assert!(out.contains("--local-repos removes it"), "{out}");
    assert!(out.contains("nothing is removed"), "{out}");
    all_there(&ws);

    let inside = ws.root.join(".agent-session");
    let out = text(&run_ok(&mut delete(
        &fx,
        &inside,
        &["-s", "b", "--dry-run", "--local-repos"],
    )));
    assert!(out.contains("remove work's origin"), "{out}");
    assert!(out.contains("remove .agent-session's origin"), "{out}");
    all_there(&ws);
}

/// The scope must match the workspace, a path inside one is no root,
/// and a directory with no config is no workspace.
#[test]
fn the_wrong_scope_and_a_non_root_are_refused() {
    let fx = CliFixture::new("delete-refusals");
    let ws = make_ws(&fx);
    let root = ws.root.to_str().expect("utf-8");

    let out = text(&run_err(&mut delete(
        &fx,
        &fx.base,
        &["-s", "p", "--dry-run", root],
    )));
    assert!(
        out.contains("is a dual workspace: `-s b` deletes it"),
        "{out}"
    );

    let inside = ws.root.join(".agent-session");
    let inside = inside.to_str().expect("utf-8");
    let out = text(&run_err(&mut delete(
        &fx,
        &fx.base,
        &["-s", "b", "--dry-run", inside],
    )));
    assert!(out.contains("not its root"), "{out}");

    let out = text(&run_err(&mut delete(
        &fx,
        &fx.base,
        &["-s", "b", "--dry-run"],
    )));
    assert!(out.contains("is in no workspace"), "{out}");
    all_there(&ws);
}

/// Work that is nowhere else refuses without `--local-repos`, naming
/// it, and with it is listed as lost.
#[test]
fn work_nowhere_else_refuses_without_local_repos() {
    let fx = CliFixture::new("delete-nowhere-else");
    let ws = make_ws(&fx);
    let root = ws.root.to_str().expect("utf-8");
    std::fs::write(ws.root.join("new.txt"), "new\n").expect("write");

    let out = text(&run_err(&mut delete(
        &fx,
        &fx.base,
        &["-s", "b", "--dry-run", root],
    )));
    assert!(out.contains("work: @ has uncommitted changes"), "{out}");
    assert!(out.contains("--local-repos to remove it all"), "{out}");

    let out = text(&run_ok(&mut delete(
        &fx,
        &fx.base,
        &["-s", "b", "--dry-run", "--local-repos", root],
    )));
    assert!(
        out.contains("lost: work: @ has uncommitted changes"),
        "{out}"
    );
    all_there(&ws);
}

/// `--yes` removes the workspace and the symlink and keeps the bare
/// repos, and `--local-repos --yes` removes them too. A stdin that is
/// no terminal, without `--yes`, removes nothing.
#[test]
fn yes_removes_the_workspace_and_local_repos_its_origins() {
    let fx = CliFixture::new("delete-removes");
    let ws = make_ws(&fx);
    let root = ws.root.to_str().expect("utf-8").to_string();

    let out = text(&run_err(
        delete(&fx, &fx.base, &["-s", "b", &root]).stdin(std::process::Stdio::null()),
    ));
    assert!(out.contains("asking requires a terminal"), "{out}");
    all_there(&ws);

    let out = text(&run_ok(&mut delete(
        &fx,
        &fx.base,
        &["-s", "b", "--yes", &root],
    )));
    assert!(out.contains("delete: done"), "{out}");
    assert!(!ws.root.exists(), "work repo removed");
    assert!(ws.link.symlink_metadata().is_err(), "symlink removed");
    assert!(ws.remotes.join("ws.git").is_dir(), "work origin kept");
    assert!(
        ws.remotes.join("ws.agent-session.git").is_dir(),
        "agent origin kept"
    );

    let fx = CliFixture::new("delete-removes-local");
    let ws = make_ws(&fx);
    let root = ws.root.to_str().expect("utf-8").to_string();
    run_ok(&mut delete(
        &fx,
        &fx.base,
        &["-s", "b", "--yes", "--local-repos", &root],
    ));
    assert!(!ws.root.exists(), "work repo removed");
    assert!(ws.link.symlink_metadata().is_err(), "symlink removed");
    assert!(!ws.remotes.join("ws.git").exists(), "work origin removed");
    assert!(
        !ws.remotes.join("ws.agent-session.git").exists(),
        "agent origin removed"
    );
    assert!(ws.remotes.is_dir(), "the remotes' directory is kept");
}

/// A run from inside the workspace removes it and says the shell's
/// directory went with it.
#[test]
fn a_run_from_inside_removes_its_own_workspace() {
    let fx = CliFixture::new("delete-inside");
    let ws = make_ws(&fx);
    let out = text(&run_ok(&mut delete(&fx, &ws.root, &["-s", "b", "--yes"])));
    assert!(out.contains("this shell's directory is gone"), "{out}");
    assert!(!ws.root.exists(), "work repo removed");
}

/// A URL origin is never touched: `--local-repos` over one is refused,
/// and without it the workspace goes and the URL is only named.
#[test]
fn a_url_origin_refuses_local_repos() {
    let fx = CliFixture::new("delete-url");
    let ws = make_ws(&fx);
    let root = ws.root.to_str().expect("utf-8").to_string();
    common::jj(
        &fx.home,
        &ws.root,
        &[
            "git",
            "remote",
            "set-url",
            "origin",
            "https://example.invalid/o/ws.git",
        ],
    );
    let out = text(&run_err(&mut delete(
        &fx,
        &fx.base,
        &["-s", "b", "--yes", "--local-repos", &root],
    )));
    assert!(
        out.contains("is a URL, which delete never touches"),
        "{out}"
    );
    all_there(&ws);

    let out = text(&run_ok(&mut delete(
        &fx,
        &fx.base,
        &["-s", "b", "--yes", &root],
    )));
    assert!(
        out.contains("keep work's origin https://example.invalid/o/ws.git, a URL"),
        "{out}"
    );
    assert!(!ws.root.exists(), "work repo removed");
}

/// An agent-repo outside the work-repo is removed with it, and a
/// symlink that points elsewhere is left and named.
#[test]
fn an_agent_repo_outside_goes_and_a_foreign_symlink_stays() {
    let fx = CliFixture::new("delete-outside");
    let ws = make_ws(&fx);
    let outside = fx.path("ws-agent");
    std::fs::rename(ws.root.join(".agent-session"), &outside).expect("move the agent repo");
    let cfg = ws.root.join(".vc-config.md");
    let text_cfg = std::fs::read_to_string(&cfg).expect("read config");
    let moved = text_cfg.replace(
        "agent = \".agent-session\"",
        &format!("agent = \"{}\"", outside.display()),
    );
    assert_ne!(text_cfg, moved, "the config names the agent repo");
    std::fs::write(&cfg, moved).expect("write config");
    let agent_cfg = outside.join(".vc-config.md");
    let a = std::fs::read_to_string(&agent_cfg).expect("read agent config");
    std::fs::write(
        &agent_cfg,
        a.replace(
            "work = \"..\"",
            &format!("work = \"{}\"", ws.root.display()),
        ),
    )
    .expect("write agent config");
    std::fs::remove_file(&ws.link).expect("remove the symlink");
    #[cfg(unix)]
    std::os::unix::fs::symlink(fx.path("elsewhere"), &ws.link).expect("a foreign symlink");

    // The config edit is uncommitted work, so `--local-repos` takes it.
    let root = ws.root.to_str().expect("utf-8").to_string();
    let out = text(&run_ok(&mut delete(
        &fx,
        &fx.base,
        &["-s", "b", "--yes", "--local-repos", &root],
    )));
    assert!(out.contains("not this workspace"), "{out}");
    assert!(!ws.root.exists(), "work repo removed");
    assert!(!outside.exists(), "agent repo outside removed");
    assert!(
        ws.link.symlink_metadata().is_ok(),
        "the foreign symlink stays"
    );
}

/// A run that stopped part way is finished by a second: a part already
/// gone, here the work origin and the symlink, is named as gone and the
/// rest is removed.
#[test]
fn a_second_run_finishes_what_a_first_left() {
    let fx = CliFixture::new("delete-second-run");
    let ws = make_ws(&fx);
    std::fs::remove_file(&ws.link).expect("the symlink, as a first run removes it");
    std::fs::remove_dir_all(ws.remotes.join("ws.git")).expect("the work origin");
    let root = ws.root.to_str().expect("utf-8").to_string();
    let out = text(&run_ok(&mut delete(
        &fx,
        &fx.base,
        &["-s", "b", "--yes", "--local-repos", &root],
    )));
    assert!(out.contains("no Claude Code symlink"), "{out}");
    assert!(out.contains("is gone already"), "{out}");
    assert!(!ws.root.exists(), "work repo removed");
    assert!(
        !ws.remotes.join("ws.agent-session.git").exists(),
        "agent origin removed"
    );
}

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

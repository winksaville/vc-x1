//! CLI subprocess integration tests for `vc-x1 repo delete`.
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

/// `vc-x1 repo delete <args>` from `dir`.
fn delete(fx: &CliFixture, dir: &Path, args: &[&str]) -> std::process::Command {
    let mut c = fx.cmd();
    c.args(["repo", "delete"]).args(args).current_dir(dir);
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
        out.contains("is a dual workspace: `-s b` takes it"),
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

/// `vc-x1 repo rename <args>` from `dir`.
fn rename(fx: &CliFixture, dir: &Path, args: &[&str]) -> std::process::Command {
    let mut c = fx.cmd();
    c.args(["repo", "rename"]).args(args).current_dir(dir);
    c
}

/// A dry run names every part and its new name, the bare origins, both
/// URLs, the config's agent-repo, the work repo, and the symlink, and
/// renames nothing.
#[test]
fn a_rename_dry_run_names_every_part_and_renames_nothing() {
    let fx = CliFixture::new("rename-dry-run");
    let ws = make_ws(&fx);
    let root = ws.root.to_str().expect("utf-8");

    let out = text(&run_ok(&mut rename(
        &fx,
        &fx.base,
        &["-s", "b", "--dry-run", root, "ws2"],
    )));
    let remotes = ws.remotes.canonicalize().expect("remotes");
    assert!(
        out.contains(&format!(
            "move work's origin {} to {}",
            remotes.join("ws.git").display(),
            remotes.join("ws2.git").display()
        )),
        "{out}"
    );
    assert!(
        out.contains("ws2.agent-session.git"),
        "the agent origin follows: {out}"
    );
    assert!(out.contains("set work's origin URL"), "{out}");
    assert!(
        out.contains("set remote.agent-repo in") && out.contains("to \"ws2.agent-session\""),
        "{out}"
    );
    let root_to = ws.root.with_file_name("ws2");
    assert!(
        out.contains(&format!(
            "move the work repo {root} to {}",
            root_to.display()
        )),
        "{out}"
    );
    assert!(
        out.contains(&format!("remove the symlink {}", ws.link.display())),
        "{out}"
    );
    assert!(out.contains("make the symlink"), "{out}");
    assert!(out.contains("nothing is renamed"), "{out}");
    all_there(&ws);
    assert!(!root_to.exists(), "nothing made under the new name");
}

/// What would stop a rename refuses it before anything changes: work
/// nowhere else, a new name taken, a name that is no directory name,
/// and the name it has already.
#[test]
fn a_rename_is_refused_before_anything_changes() {
    let fx = CliFixture::new("rename-refusals");
    let ws = make_ws(&fx);
    let root = ws.root.to_str().expect("utf-8");

    let out = text(&run_err(&mut rename(
        &fx,
        &fx.base,
        &["-s", "b", "--dry-run", root, "a/b"],
    )));
    assert!(out.contains("`repo move` changes where"), "{out}");

    let out = text(&run_err(&mut rename(
        &fx,
        &fx.base,
        &["-s", "b", "--dry-run", root, "ws"],
    )));
    assert!(out.contains("named 'ws' already"), "{out}");

    std::fs::create_dir(ws.root.with_file_name("taken")).expect("mkdir");
    let out = text(&run_err(&mut rename(
        &fx,
        &fx.base,
        &["-s", "b", "--dry-run", root, "taken"],
    )));
    assert!(out.contains("is there already"), "{out}");

    std::fs::write(ws.root.join("new.txt"), "new\n").expect("write");
    let out = text(&run_err(&mut rename(
        &fx,
        &fx.base,
        &["-s", "b", "--dry-run", root, "ws2"],
    )));
    assert!(out.contains("work: @ has uncommitted changes"), "{out}");
    assert!(out.contains("nothing is renamed"), "{out}");
    all_there(&ws);
}

/// A rename with `--yes` moves the bare origins, sets both URLs and
/// the config's agent-repo, moves the work repo with its agent repo,
/// and remakes the symlink, after which the workspace syncs from its
/// new place and nothing is left under the old names. Without `--yes`
/// and a terminal it renames nothing.
#[test]
fn a_rename_moves_every_part_on_this_machine() {
    let fx = CliFixture::new("rename-moves");
    let ws = make_ws(&fx);
    let root = ws.root.to_str().expect("utf-8").to_string();

    let out = text(&run_err(
        rename(&fx, &fx.base, &["-s", "b", &root, "ws2"]).stdin(std::process::Stdio::null()),
    ));
    assert!(out.contains("asking requires a terminal"), "{out}");
    all_there(&ws);

    let out = text(&run_ok(&mut rename(
        &fx,
        &fx.base,
        &["-s", "b", "--yes", &root, "ws2"],
    )));
    assert!(out.contains("rename: done"), "{out}");
    let root_to = ws.root.with_file_name("ws2");
    assert!(!ws.root.exists(), "the old work repo is gone");
    assert!(
        root_to.join(".agent-session").is_dir(),
        "agent repo moved with it"
    );
    assert!(!ws.remotes.join("ws.git").exists(), "old work origin gone");
    assert!(ws.remotes.join("ws2.git").is_dir(), "work origin moved");
    assert!(
        ws.remotes.join("ws2.agent-session.git").is_dir(),
        "agent origin moved"
    );
    assert!(ws.link.symlink_metadata().is_err(), "old symlink removed");
    let links: Vec<PathBuf> = std::fs::read_dir(ws.link.parent().expect("projects"))
        .expect("projects")
        .map(|e| e.expect("entry").path())
        .collect();
    assert_eq!(links.len(), 1, "{links:?}");
    assert_eq!(
        std::fs::read_link(&links[0]).expect("a symlink"),
        root_to.join(".agent-session"),
    );
    let cfg = std::fs::read_to_string(root_to.join(".vc-config.md")).expect("config");
    assert!(cfg.contains("agent-repo = \"ws2.agent-session\""), "{cfg}");
    let out = text(&run_ok(
        fx.cmd().args(["status", "-s", "b"]).current_dir(&root_to),
    ));
    assert!(out.contains("status: clean"), "committed: {out}");
    assert!(
        out.contains("main | chore: rename the workspace to ws2"),
        "{out}"
    );

    let out = text(&run_ok(
        fx.cmd()
            .args(["sync", "-s", "b", "--dry-run"])
            .current_dir(&root_to),
    ));
    assert!(out.contains("up to date"), "{out}");
}

/// A one-side scope is refused by delete, which would otherwise take
/// it as `-s b` and delete both repos, while rename takes `-s w` as
/// `-s b`.
#[test]
fn one_side_scopes_mean_what_they_say() {
    let fx = CliFixture::new("repo-scopes");
    let ws = make_ws(&fx);
    let root = ws.root.to_str().expect("utf-8");

    for s in ["w", "a"] {
        let out = text(&run_err(&mut delete(
            &fx,
            &fx.base,
            &["-s", s, "--dry-run", root],
        )));
        assert!(out.contains("deleted whole, not one side"), "{out}");
    }
    let out = text(&run_ok(&mut rename(
        &fx,
        &fx.base,
        &["-s", "w", "--dry-run", root, "ws2"],
    )));
    assert!(
        out.contains("ws2.agent-session"),
        "the agent side follows: {out}"
    );
    all_there(&ws);
}

/// `-s a` renames the agent directory in place, named by the root or
/// by the agent directory itself: the work config's `repos.agent` and
/// `remote.agent-repo`, the work `.gitignore` line, the bare origin,
/// its URL, and the symlink all follow, and the workspace syncs after.
#[test]
fn a_rename_of_the_agent_side_alone() {
    let fx = CliFixture::new("rename-agent");
    let ws = make_ws(&fx);
    let agent = ws.root.join(".agent-session");
    let agent_s = agent.to_str().expect("utf-8").to_string();

    let out = text(&run_ok(&mut rename(
        &fx,
        &fx.base,
        &["-s", "a", "--dry-run", &agent_s, ".agentsession"],
    )));
    assert!(out.contains("set the line /.agent-session in"), "{out}");
    assert!(!out.contains("move the work repo"), "{out}");
    all_there(&ws);

    let out = text(&run_ok(&mut rename(
        &fx,
        &ws.root,
        &["-s", "a", "--yes", ".", ".agentsession"],
    )));
    assert!(out.contains("rename: done"), "{out}");
    let agent_to = ws.root.join(".agentsession");
    assert!(agent_to.is_dir() && !agent.exists(), "agent dir renamed");
    assert!(
        ws.remotes.join("ws.agentsession.git").is_dir(),
        "agent origin moved"
    );
    assert!(ws.remotes.join("ws.git").is_dir(), "work origin kept");
    let cfg = std::fs::read_to_string(ws.root.join(".vc-config.md")).expect("config");
    assert!(cfg.contains("agent = \".agentsession\""), "{cfg}");
    assert!(cfg.contains("agent-repo = \"ws.agentsession\""), "{cfg}");
    let ignore = std::fs::read_to_string(ws.root.join(".gitignore")).expect("gitignore");
    assert!(ignore.lines().any(|l| l == "/.agentsession"), "{ignore}");
    assert_eq!(std::fs::read_link(&ws.link).expect("symlink"), agent_to);
    let out = text(&run_ok(
        fx.cmd().args(["status", "-s", "b"]).current_dir(&ws.root),
    ));
    assert!(out.contains("status: clean"), "committed and pushed: {out}");

    let out = text(&run_ok(&mut rename(
        &fx,
        &ws.root,
        &["-s", "a", "--yes", "--local", ".", ".agent-session"],
    )));
    assert!(out.contains("kept on this machine"), "{out}");
    let out = text(&run_ok(
        fx.cmd().args(["status", "-s", "b"]).current_dir(&ws.root),
    ));
    assert!(out.contains("status: clean"), "--local commits: {out}");
    let out = text(&run_ok(
        fx.cmd()
            .args(["sync", "-s", "b", "--dry-run"])
            .current_dir(&ws.root),
    ));
    assert!(out.contains("ahead"), "--local pushes nothing: {out}");

    let out = text(&run_ok(
        fx.cmd()
            .args(["sync", "-s", "b", "--dry-run"])
            .current_dir(&ws.root),
    ));
    assert!(out.contains("up to date"), "{out}");
}

/// `vc-x1 repo move <args>` from `dir`.
fn relocate(fx: &CliFixture, dir: &Path, args: &[&str]) -> std::process::Command {
    let mut c = fx.cmd();
    c.args(["repo", "move"]).args(args).current_dir(dir);
    c
}

/// Point `repo`'s origin at `url` in its git config, as a hand edit
/// would.
fn set_origin(repo: &Path, url: &str) {
    let cfg = repo.join(".git").join("config");
    let text = std::fs::read_to_string(&cfg).expect("git config");
    let mut out = String::new();
    let mut in_origin = false;
    for line in text.lines() {
        if line.trim_start().starts_with('[') {
            in_origin = line.trim() == "[remote \"origin\"]";
        }
        if in_origin && line.trim_start().starts_with("url = ") {
            out.push_str(&format!("\turl = {url}\n"));
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    std::fs::write(&cfg, out).expect("write git config");
}

/// A move puts the workspace under DIR with its name, rewrites a
/// relative origin for the new place and keeps an absolute one, makes
/// the symlink for the new root, and the workspace syncs from there.
/// DIR missing and DIR inside the workspace are refused first.
#[test]
fn a_move_puts_the_workspace_in_another_directory() {
    let fx = CliFixture::new("move");
    let ws = make_ws(&fx);
    let root = ws.root.to_str().expect("utf-8").to_string();
    set_origin(&ws.root, "../remotes/ws.git");

    let out = text(&run_err(&mut relocate(
        &fx,
        &fx.base,
        &["-s", "b", "--dry-run", &root, "nowhere"],
    )));
    assert!(out.contains("make it first"), "{out}");
    let inside = ws.root.join("sub");
    std::fs::create_dir(&inside).expect("mkdir");
    let out = text(&run_err(&mut relocate(
        &fx,
        &fx.base,
        &[
            "-s",
            "b",
            "--dry-run",
            &root,
            inside.to_str().expect("utf-8"),
        ],
    )));
    assert!(out.contains("inside the workspace"), "{out}");
    std::fs::remove_dir(&inside).expect("rmdir");

    let dest = fx.path("deeper").join("place");
    std::fs::create_dir_all(&dest).expect("mkdir");
    let dest_s = dest.to_str().expect("utf-8").to_string();
    let out = text(&run_ok(&mut relocate(
        &fx,
        &fx.base,
        &["-s", "b", "--yes", &root, &dest_s],
    )));
    assert!(out.contains("move: done"), "{out}");
    let root_to = dest.canonicalize().expect("dest").join("ws");
    assert!(!ws.root.exists(), "the old place is empty");
    assert!(
        root_to.join(".agent-session").is_dir(),
        "agent repo moved with it"
    );
    let git_cfg = std::fs::read_to_string(root_to.join(".git").join("config")).expect("config");
    assert!(
        git_cfg.contains("url = ../../../remotes/ws.git"),
        "{git_cfg}"
    );
    let agent_cfg =
        std::fs::read_to_string(root_to.join(".agent-session").join(".git").join("config"))
            .expect("agent config");
    assert!(
        agent_cfg.contains("ws.agent-session.git"),
        "absolute kept: {agent_cfg}"
    );
    assert!(ws.link.symlink_metadata().is_err(), "old symlink removed");
    let links: Vec<PathBuf> = std::fs::read_dir(ws.link.parent().expect("projects"))
        .expect("projects")
        .map(|e| e.expect("entry").path())
        .collect();
    assert_eq!(links.len(), 1, "{links:?}");
    assert_eq!(
        std::fs::read_link(&links[0]).expect("a symlink"),
        root_to.join(".agent-session")
    );

    let out = text(&run_ok(
        fx.cmd()
            .args(["sync", "-s", "b", "--dry-run"])
            .current_dir(&root_to),
    ));
    assert!(out.contains("up to date"), "{out}");
}

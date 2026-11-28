//! `vc-x1 repo delete`: remove a workspace from this machine.
//!
//! - The workspace is found by its `.vc-config.md`, as every command
//!   finds it: PATH is its root, or the current directory is inside
//!   it. A PATH that is not a root is refused, so a path given by
//!   hand never reaches a workspace above it.
//! - Its parts: the work-repo's directory, the agent-repo's when the
//!   config puts it outside the work-repo, and the Claude Code symlink
//!   `~/.claude/projects/<encoded root>` when it points at the
//!   agent-repo. A symlink pointing elsewhere, or a directory Claude
//!   Code made there, is left and named.
//! - `--local-repos` adds the bare repos the two `origin`s name when
//!   they are paths on this machine. A remote reached by a URL is
//!   never touched, and `--local-repos` over one is refused.
//! - What is nowhere else stops it: without `--local-repos` the remote
//!   keeps what was pushed, so uncommitted work in the work `@` and
//!   commits no remote bookmark reaches refuse. With it nothing
//!   survives anyway, so they are listed and confirmed. The agent
//!   `@` holds the session's data and is listed, never refused.
//! - `-s b` names a dual workspace and `-s p` a plain repo, and a scope
//!   that does not match the workspace is refused.

use std::path::{Path, PathBuf};

use clap::{Args, ValueHint};
use log::{debug, info};

use crate::context::Context;
use crate::jj;
use crate::options_flags::dry_run::DryRunFlag;
use crate::options_flags::scope::{RepoScope, repo_scope_parser};
use crate::subcommand::SubcommandRunner;

/// Remove a workspace from this machine.
#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// The workspace's root [default: the workspace the current
    /// directory is in]
    #[arg(value_name = "PATH", value_hint = ValueHint::DirPath)]
    pub path: Option<PathBuf>,

    /// Also remove the bare repos the `origin`s are, when they are
    /// paths on this machine, never a remote reached by a URL
    #[arg(long)]
    pub local_repos: bool,

    /// Remove without asking, for a run with no terminal
    #[arg(long)]
    pub yes: bool,

    /// The workspace's shape, required: `b|both` a dual workspace,
    /// `p|por` a plain repo
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = repo_scope_parser(&["both", "por"]),
        required = true
    )]
    pub scope: RepoScope,

    /// Print what it would remove, and remove nothing
    #[command(flatten)]
    pub dry_run: DryRunFlag,
}

/// Inputs to the delete op, flat, owned, clap-free.
#[derive(Debug)]
pub struct DeleteParams {
    pub path: Option<PathBuf>,
    pub local_repos: bool,
    pub yes: bool,
    pub dual: bool,
    pub dry_run: bool,
}

impl From<&DeleteArgs> for DeleteParams {
    fn from(a: &DeleteArgs) -> Self {
        DeleteParams {
            path: a.path.clone(),
            local_repos: a.local_repos,
            yes: a.yes,
            dual: matches!(a.scope, RepoScope::Sides(_)),
            dry_run: a.dry_run.value,
        }
    }
}

impl SubcommandRunner for DeleteArgs {
    type Params = DeleteParams;

    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(DeleteParams::from(self))
    }

    /// Run the `delete` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        debug!("repo delete: enter {params:?}");
        let cwd = std::env::current_dir()?;
        let projects = crate::symlink::default_symlink_dir()?;
        delete(ctx, params, &cwd, &projects)
    }
}

/// One repo's `origin`: a bare repo on this machine, a URL, or none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// No `origin`, as an `init` that failed before publishing leaves.
    None,
    /// A path on this machine that is a bare git repo.
    Local(PathBuf),
    /// A path on this machine that is not a bare git repo, never
    /// removed, so a config pointing at a working tree is safe.
    NotBare(PathBuf),
    /// A path on this machine where nothing is, as a run that removed
    /// it and stopped leaves.
    Gone(PathBuf),
    /// A URL, reached over the network, never touched.
    Url(String),
}

/// What is at the Claude Code symlink's path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Link {
    /// Nothing.
    Absent,
    /// A symlink to the workspace's agent-repo, removed.
    Ours(PathBuf),
    /// A symlink pointing elsewhere, left.
    Elsewhere(PathBuf, PathBuf),
    /// Something that is not a symlink, Claude Code's own, left.
    NotSymlink(PathBuf),
}

/// One repo of the workspace and what it holds that is nowhere else.
#[derive(Debug, Clone)]
pub struct Repo {
    /// `work` or the agent-repo's directory name.
    pub label: String,
    /// The agent-repo, whose `@` is the session's data.
    pub agent: bool,
    pub path: PathBuf,
    /// Removed with its own directory, outside the work-repo.
    pub outside: bool,
    pub origin: Origin,
    /// The working copy holds changes.
    pub dirty_at: bool,
    /// Commits below `@` no remote bookmark reaches, short ids.
    pub unpushed: Vec<String>,
}

/// Everything `delete` found, before it removes anything.
#[derive(Debug, Clone)]
pub struct Plan {
    pub root: PathBuf,
    /// The work-repo first, then the agent-repo of a dual workspace.
    pub repos: Vec<Repo>,
    pub link: Link,
    pub local_repos: bool,
}

/// The workspace's root: PATH when it is one, else the workspace the
/// current directory is in. Refused when there is none, when PATH is
/// inside a workspace but not its root, and when the root holds the
/// home directory, since removing it would remove the home.
pub fn find_root(verb: &str, path: Option<&Path>, cwd: &Path) -> Result<PathBuf, String> {
    let root = match path {
        Some(p) => {
            let p = if p.is_absolute() {
                p.to_path_buf()
            } else {
                cwd.join(p)
            };
            let p = p
                .canonicalize()
                .map_err(|e| format!("repo {verb}: '{}' cannot be read: {e}", p.display()))?;
            match crate::common::find_workspace_root_from(&p) {
                Some(r) if r == p => r,
                Some(r) => {
                    return Err(format!(
                        "repo {verb}: '{}' is inside the workspace at '{}', not its root: name the \
                         root",
                        p.display(),
                        r.display()
                    ));
                }
                None => {
                    return Err(format!(
                        "repo {verb}: '{}' is no workspace, having no `.vc-config.md`",
                        p.display()
                    ));
                }
            }
        }
        None => crate::common::find_workspace_root_from(cwd).ok_or_else(|| {
            format!(
                "repo {verb}: '{}' is in no workspace, having no `.vc-config.md` above it: name \
                 the workspace's root",
                cwd.display()
            )
        })?,
    };
    if let Some(home) = crate::jj::session::home_dir()
        && let Ok(home) = home.canonicalize()
        && home.starts_with(&root)
    {
        return Err(format!(
            "repo {verb}: the workspace at '{}' holds the home directory '{}', and is left alone",
            root.display(),
            home.display()
        ));
    }
    Ok(root)
}

/// The local path an `origin` URL names, relative to `repo` when it
/// is relative, or `None` when it is a URL reached over the network.
pub fn local_path(url: &str, repo: &Path) -> Option<PathBuf> {
    let path = url.strip_prefix("file://").unwrap_or(url); // OK: no scheme, the URL is the path
    if path.starts_with('/') {
        Some(PathBuf::from(path))
    } else if path.starts_with("./") || path.starts_with("../") {
        Some(repo.join(path))
    } else {
        None
    }
}

/// Classify `repo`'s `origin`.
fn origin_of(repo: &Path) -> Result<Origin, Box<dyn std::error::Error>> {
    let Some(url) = jj::remote_url(repo, "origin")? else {
        return Ok(Origin::None);
    };
    let Some(path) = local_path(&url, repo) else {
        return Ok(Origin::Url(url));
    };
    if !path.exists() {
        return Ok(Origin::Gone(path));
    }
    let path = path.canonicalize().unwrap_or(path); // OK: unresolvable, it is named as written
    let bare = gix::open(&path).map(|r| r.is_bare()).unwrap_or(false); // OK: unreadable is not bare
    Ok(if bare {
        Origin::Local(path)
    } else {
        Origin::NotBare(path)
    })
}

/// Read one repo: its `origin`, its working copy, and the commits
/// below `@` no remote bookmark reaches. A directory already gone,
/// from a run that stopped, is planned as gone.
fn read_repo(
    label: &str,
    agent: bool,
    path: &Path,
    outside: bool,
) -> Result<Option<Repo>, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(None);
    }
    let origin = origin_of(path)?;
    let dirty_at = !jj::matches(path, "@ & empty()")?;
    let unpushed = jj::cids_short_of(
        path,
        "::visible_heads() ~ ::remote_bookmarks() ~ root() ~ @",
    )?;
    Ok(Some(Repo {
        label: label.to_string(),
        agent,
        path: path.to_path_buf(),
        outside,
        origin,
        dirty_at,
        unpushed,
    }))
}

/// What is at the symlink's path for the workspace at `root`, in the
/// directory `projects`, against the agent-repo `agent`.
pub fn link_of(root: &Path, agent: Option<&Path>, projects: &Path) -> Result<Link, String> {
    let name = crate::symlink::encode_path(
        root.to_str()
            .ok_or_else(|| format!("repo: '{}' is not valid UTF-8", root.display()))?,
    );
    let path = projects.join(name);
    let Ok(meta) = path.symlink_metadata() else {
        return Ok(Link::Absent);
    };
    if !meta.is_symlink() {
        return Ok(Link::NotSymlink(path));
    }
    let target = std::fs::read_link(&path)
        .map_err(|e| format!("repo: cannot read the symlink '{}': {e}", path.display()))?;
    let resolved = target.canonicalize().unwrap_or_else(|_| target.clone()); // OK: a dangling link compares as written
    let ours = agent.is_some_and(|a| a.canonicalize().map(|a| a == resolved).unwrap_or(false)); // OK: an agent-repo gone is not this link's
    Ok(if ours {
        Link::Ours(path)
    } else {
        Link::Elsewhere(path, target)
    })
}

/// A workspace's parts as found, before any verb acts on them.
#[derive(Debug, Clone)]
pub struct Survey {
    pub root: PathBuf,
    /// The work-repo first, then the agent-repo of a dual workspace.
    pub repos: Vec<Repo>,
    pub link: Link,
}

/// Find the workspace at `path`, or the one `cwd` is in, and read its
/// parts, changing nothing: the shared first step of `repo delete`,
/// `rename`, and `move`, `verb` naming the command in its messages.
/// `dual` is the `-s` the caller was given, refused when the workspace
/// is the other shape.
pub fn survey(
    verb: &str,
    path: Option<&Path>,
    dual: bool,
    cwd: &Path,
    projects: &Path,
) -> Result<Survey, Box<dyn std::error::Error>> {
    let root = find_root(verb, path, cwd)?;
    let agent = crate::common::configured_bot_dir(&root)?;
    match (dual, agent.is_some()) {
        (true, false) => {
            return Err(format!(
                "repo {verb} -s b: the workspace at '{}' is a plain repo: `-s p` takes it",
                root.display()
            )
            .into());
        }
        (false, true) => {
            return Err(format!(
                "repo {verb} -s p: the workspace at '{}' is a dual workspace: `-s b` takes it",
                root.display()
            )
            .into());
        }
        _ => {}
    }
    let mut repos: Vec<Repo> = read_repo("work", false, &root, false)?
        .into_iter()
        .collect();
    if let Some(agent) = &agent {
        let agent = agent.canonicalize().unwrap_or_else(|_| agent.clone()); // OK: a missing agent-repo is planned as gone
        if agent.starts_with(&root) && agent != root {
            repos.extend(read_repo(&label_of(&agent), true, &agent, false)?);
        } else if agent.starts_with(&root) || root.starts_with(&agent) {
            return Err(format!(
                "repo {verb}: the agent-repo '{}' and the work-repo '{}' hold each other, which no \
                 workspace vc-x1 makes does: nothing is changed",
                agent.display(),
                root.display()
            )
            .into());
        } else {
            repos.extend(read_repo(&label_of(&agent), true, &agent, true)?);
        }
    }
    let link = link_of(&root, agent.as_deref(), projects)?;
    Ok(Survey { root, repos, link })
}

/// Find the workspace and plan what `delete` removes, removing
/// nothing.
pub fn plan(
    params: &DeleteParams,
    cwd: &Path,
    projects: &Path,
) -> Result<Plan, Box<dyn std::error::Error>> {
    let Survey { root, repos, link } =
        survey("delete", params.path.as_deref(), params.dual, cwd, projects)?;
    Ok(Plan {
        root,
        repos,
        link,
        local_repos: params.local_repos,
    })
}

/// An agent-repo's label, its directory's name.
fn label_of(agent: &Path) -> String {
    agent
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "agent".to_string()) // OK: a root path has no name
}

impl Plan {
    /// The URL remotes `--local-repos` would have to leave, which
    /// refuse it.
    pub fn url_remotes(&self) -> Vec<String> {
        self.repos
            .iter()
            .filter_map(|r| match &r.origin {
                Origin::Url(u) => Some(format!("{} {u}", r.label)),
                _ => None,
            })
            .collect()
    }

    /// What is nowhere else, one line each: work in a working copy and
    /// commits no remote bookmark reaches. The agent `@` is the
    /// session's data and is a note of its own.
    pub fn nowhere_else(&self) -> Vec<String> {
        let mut out = Vec::new();
        for r in &self.repos {
            if r.dirty_at && !r.agent {
                out.push(format!("{}: @ has uncommitted changes", r.label));
            }
            if !r.unpushed.is_empty() {
                out.push(format!(
                    "{}: {} commit(s) no remote bookmark reaches: {}",
                    r.label,
                    r.unpushed.len(),
                    r.unpushed.join(" ")
                ));
            }
        }
        out
    }

    /// Whether the agent `@` holds session data, which goes with it.
    pub fn agent_session(&self) -> bool {
        self.repos.iter().any(|r| r.agent && r.dirty_at)
    }

    /// The refusals, each a reason nothing is removed.
    pub fn refusals(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.local_repos {
            for u in self.url_remotes() {
                out.push(format!(
                    "--local-repos: the origin of {u} is a URL, which delete never touches: \
                     run without --local-repos"
                ));
            }
        } else {
            for w in self.nowhere_else() {
                out.push(format!(
                    "{w}, and is nowhere else: push it, or --local-repos to remove it all"
                ));
            }
        }
        out
    }

    /// The lines that say what goes and what stays.
    pub fn lines(&self) -> Vec<String> {
        let mut out = vec![format!(
            "repo delete: the workspace at '{}'",
            self.root.display()
        )];
        match &self.link {
            Link::Ours(p) => out.push(format!("  remove the symlink {}", p.display())),
            Link::Absent => out.push("  no Claude Code symlink".to_string()),
            Link::Elsewhere(p, t) => out.push(format!(
                "  leave the symlink {}, pointing at {}, not this workspace",
                p.display(),
                t.display()
            )),
            Link::NotSymlink(p) => out.push(format!(
                "  leave {}, not a symlink: Claude Code's own",
                p.display()
            )),
        }
        for r in &self.repos {
            let line = match (&r.origin, self.local_repos) {
                (Origin::Local(p), true) => {
                    format!("  remove {}'s origin {}", r.label, p.display())
                }
                (Origin::Local(p), false) => {
                    format!(
                        "  keep {}'s origin {}, --local-repos removes it",
                        r.label,
                        p.display()
                    )
                }
                (Origin::NotBare(p), _) => format!(
                    "  keep {}'s origin {}, not a bare repo",
                    r.label,
                    p.display()
                ),
                (Origin::Url(u), _) => format!("  keep {}'s origin {u}, a URL", r.label),
                (Origin::Gone(p), _) => {
                    format!("  {}'s origin {} is gone already", r.label, p.display())
                }
                (Origin::None, _) => format!("  {} has no origin", r.label),
            };
            out.push(line);
        }
        for r in self.repos.iter().filter(|r| r.outside) {
            out.push(format!(
                "  remove the {} repo {}",
                r.label,
                r.path.display()
            ));
        }
        out.push(format!(
            "  remove the work repo {}{}",
            self.root.display(),
            if self.repos.iter().any(|r| r.agent && !r.outside) {
                ", the agent repo inside it"
            } else {
                ""
            }
        ));
        if self.agent_session() {
            out.push("  the agent @ holds session data, which goes with it".to_string());
        }
        if self.local_repos {
            for w in self.nowhere_else() {
                out.push(format!("  lost: {w}"));
            }
        }
        out
    }
}

/// Find the workspace, plan, print the plan, and, confirmed, remove
/// what it names.
pub fn delete(
    ctx: &mut Context,
    params: &DeleteParams,
    cwd: &Path,
    projects: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let plan = plan(params, cwd, projects)?;
    for line in plan.lines() {
        info!("{line}");
    }
    let refusals = plan.refusals();
    if !refusals.is_empty() {
        return Err(format!(
            "repo delete: nothing is removed:\n  {}",
            refusals.join("\n  ")
        )
        .into());
    }
    if params.dry_run {
        info!("repo delete: [dry-run] nothing is removed");
        return Ok(());
    }
    confirm(params.yes, ctx.stdin_is_tty)?;
    remove(&plan)?;
    if cwd.starts_with(&plan.root) {
        info!("repo delete: done, and this shell's directory is gone with it: `cd` out of it");
    } else {
        info!("repo delete: done");
    }
    Ok(())
}

/// Ask before removing, unless `--yes`. With no terminal to ask, the
/// run is refused rather than hung, and a declined answer is an error,
/// so a script learns nothing was removed.
fn confirm(yes: bool, stdin_is_tty: bool) -> Result<(), Box<dyn std::error::Error>> {
    if yes {
        debug!("repo delete: --yes, not asked");
        return Ok(());
    }
    if !stdin_is_tty {
        return Err(
            "repo delete: asking requires a terminal (stdin is not interactive): --yes removes \
             without asking, nothing is removed"
                .into(),
        );
    }
    let answer = crate::common::prompt("repo delete: remove these? [y/N] ")?;
    let answer = answer.trim().to_ascii_lowercase();
    if answer != "y" && answer != "yes" {
        return Err(format!("repo delete: declined (got {answer:?}), nothing is removed").into());
    }
    Ok(())
}

/// Remove the plan's parts in its order: the symlink, so no Claude
/// Code session writes into a workspace half gone, the origins under
/// `--local-repos`, an agent-repo outside the work-repo, and the
/// work-repo last, so a run that stops leaves the workspace findable
/// and a second run removes the rest. A failure names what is left.
pub fn remove(plan: &Plan) -> Result<(), Box<dyn std::error::Error>> {
    // Each step: what it is called, its path, and whether it is the
    // symlink, a file, rather than a directory.
    let mut steps: Vec<(String, PathBuf, bool)> = Vec::new();
    if let Link::Ours(p) = &plan.link {
        steps.push((format!("the symlink {}", p.display()), p.clone(), true));
    }
    if plan.local_repos {
        for r in &plan.repos {
            if let Origin::Local(p) = &r.origin {
                steps.push((
                    format!("{}'s origin {}", r.label, p.display()),
                    p.clone(),
                    false,
                ));
            }
        }
    }
    for r in plan.repos.iter().filter(|r| r.outside) {
        steps.push((
            format!("the {} repo {}", r.label, r.path.display()),
            r.path.clone(),
            false,
        ));
    }
    if plan.root.exists() {
        steps.push((
            format!("the work repo {}", plan.root.display()),
            plan.root.clone(),
            false,
        ));
    }
    for (i, (name, path, file)) in steps.iter().enumerate() {
        let done = if *file {
            std::fs::remove_file(path)
        } else {
            std::fs::remove_dir_all(path)
        };
        done.map_err(|e| {
            let left: Vec<&str> = steps[i..].iter().map(|(n, _, _)| n.as_str()).collect();
            format!(
                "repo delete: cannot remove {name}: {e}. Left: {}. A second run removes the rest",
                left.join(", ")
            )
        })?;
        info!("repo delete: removed {name}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A path is local in its three spellings, and a URL is not.
    #[test]
    fn an_origin_is_local_when_it_is_a_path() {
        let repo = Path::new("/w/ws");
        assert_eq!(
            local_path("/r/ws.git", repo),
            Some(PathBuf::from("/r/ws.git"))
        );
        assert_eq!(
            local_path("file:///r/ws.git", repo),
            Some(PathBuf::from("/r/ws.git"))
        );
        assert_eq!(
            local_path("../r/ws.git", repo),
            Some(PathBuf::from("/w/ws/../r/ws.git"))
        );
        assert_eq!(local_path("https://github.com/o/ws.git", repo), None);
        assert_eq!(local_path("git@github.com:o/ws.git", repo), None);
    }
}

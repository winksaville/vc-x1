//! `vc-x1 repo move`: move a workspace to another directory, its name
//! kept.
//!
//! - The workspace is found as `repo delete` and `rename` find it, by
//!   [`delete::survey`], and moved whole into DIR, its agent-repo
//!   inside it moving with it.
//! - No remote is touched and no config changes. An `origin` that is a
//!   relative path is rewritten for the new place, and one that is
//!   absolute or a URL stays as written.
//! - The Claude Code symlink is made for the new root.
//! - Refused before anything moves: DIR missing or no directory, the
//!   workspace there already, DIR inside the workspace, another
//!   filesystem, and an agent-repo outside the work-repo.
//! - The steps are `rename`'s, [`rename::apply`].

use std::path::{Component, Path, PathBuf};

use clap::{Args, ValueHint};
use log::{debug, info};

use super::delete::{self, Link, Survey};
use super::rename::{self, Plan, Remote, RepoPlan};
use crate::context::Context;
use crate::options_flags::dry_run::DryRunFlag;
use crate::options_flags::scope::{RepoScope, Side, repo_scope_parser};
use crate::subcommand::SubcommandRunner;

/// Move a workspace to another directory.
#[derive(Args, Debug)]
pub struct MoveArgs {
    /// The workspace's root, `.` the one the current directory is
    #[arg(value_name = "CURRENT", value_hint = ValueHint::DirPath)]
    pub current: PathBuf,

    /// The directory to move it into, its name kept
    #[arg(value_name = "DIR", value_hint = ValueHint::DirPath)]
    pub dir: PathBuf,

    /// Move without asking, for a run with no terminal
    #[arg(long)]
    pub yes: bool,

    /// The workspace's shape, required: `b|both` a dual workspace,
    /// `w|work` the same, `p|por` a plain repo
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = repo_scope_parser(&["both", "work", "por"]),
        required = true
    )]
    pub scope: RepoScope,

    /// Print what it would move, and move nothing
    #[command(flatten)]
    pub dry_run: DryRunFlag,
}

/// Inputs to the move op, flat, owned, clap-free.
#[derive(Debug)]
pub struct MoveParams {
    pub current: PathBuf,
    pub dir: PathBuf,
    pub yes: bool,
    pub dual: bool,
    pub dry_run: bool,
}

impl SubcommandRunner for MoveArgs {
    type Params = MoveParams;

    /// `-s w` is `-s b`, and `-s a` is refused, an agent-repo inside
    /// the work-repo moving with it.
    fn to_params(&self) -> Result<Self::Params, String> {
        if let RepoScope::Sides(s) = &self.scope
            && s.0 == [Side::Bot]
        {
            return Err(
                "repo move -s a: a workspace moves whole, its agent-repo with it: `-s b`".into(),
            );
        }
        Ok(MoveParams {
            current: self.current.clone(),
            dir: self.dir.clone(),
            yes: self.yes,
            dual: matches!(self.scope, RepoScope::Sides(_)),
            dry_run: self.dry_run.value,
        })
    }

    /// Run the `move` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        debug!("repo move: enter {params:?}");
        let cwd = std::env::current_dir()?;
        let projects = crate::symlink::default_symlink_dir()?;
        relocate(ctx, params, &cwd, &projects)
    }
}

/// The path from the directory `from` to `to`, both absolute and
/// normalized, as `..` steps up to their common part and then down.
pub fn relative_path(from: &Path, to: &Path) -> PathBuf {
    let f: Vec<Component> = from.components().collect();
    let t: Vec<Component> = to.components().collect();
    let common = f.iter().zip(&t).take_while(|(a, b)| a == b).count();
    let mut out = PathBuf::new();
    for _ in common..f.len() {
        out.push("..");
    }
    for c in &t[common..] {
        out.push(c.as_os_str());
    }
    out
}

/// Whether `url` is a path relative to the repo, `./` or `../`, the
/// one form a move breaks.
fn is_relative_path(url: &str) -> bool {
    url.starts_with("./") || url.starts_with("../")
}

/// Find the workspace and plan its move, changing nothing.
pub fn plan(
    params: &MoveParams,
    cwd: &Path,
    projects: &Path,
) -> Result<Plan, Box<dyn std::error::Error>> {
    let Survey { root, repos, link } =
        delete::survey("move", Some(&params.current), params.dual, cwd, projects)?;
    let dir = if params.dir.is_absolute() {
        params.dir.clone()
    } else {
        cwd.join(&params.dir)
    };
    let dest = dir.canonicalize().map_err(|e| {
        format!(
            "repo move: DIR '{}' cannot be read: {e}: make it first",
            dir.display()
        )
    })?;
    if !dest.is_dir() {
        return Err(format!("repo move: DIR '{}' is no directory", dest.display()).into());
    }
    let name = root
        .file_name()
        .ok_or_else(|| format!("repo move: '{}' has no name", root.display()))?;
    let root_to = dest.join(name);
    let mut refusals: Vec<String> = Vec::new();

    if dest.starts_with(&root) {
        refusals.push(format!(
            "DIR '{}' is inside the workspace, which cannot hold itself",
            dest.display()
        ));
    } else if root_to == root {
        refusals.push(format!(
            "the workspace is in '{}' already: nothing to move",
            dest.display()
        ));
    } else if root_to.symlink_metadata().is_ok() {
        refusals.push(format!("{} is there already", root_to.display()));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let from_dev = std::fs::metadata(&root)?.dev();
        let to_dev = std::fs::metadata(&dest)?.dev();
        if from_dev != to_dev {
            refusals.push(format!(
                "DIR '{}' is on another filesystem, which a move cannot cross: copy the workspace \
                 there instead",
                dest.display()
            ));
        }
    }

    let mut plans: Vec<RepoPlan> = Vec::new();
    let mut agent_to = None;
    for r in &repos {
        if r.outside {
            refusals.push(format!(
                "the agent-repo {} is outside the work-repo, which move does not take: move it by \
                 hand",
                r.path.display()
            ));
            continue;
        }
        let rel = r.path.strip_prefix(&root).unwrap_or(Path::new("")); // OK: a repo not outside is under the root
        let path_to = root_to.join(rel);
        if r.agent {
            agent_to = Some(path_to.clone());
        }
        let url = crate::jj::remote_url(&r.path, "origin")?;
        let url_to = match &url {
            Some(u) if is_relative_path(u) => {
                let target = crate::common::normalize_path(&r.path.join(u));
                Some(
                    relative_path(&path_to, &target)
                        .to_string_lossy()
                        .into_owned(),
                )
            }
            other => other.clone(),
        };
        plans.push(RepoPlan {
            label: r.label.clone(),
            url_from: url,
            url_to,
            remote: Remote::Kept,
            dir: r.path.clone(),
            dir_to: None,
        });
    }

    let link_to = if params.dual {
        let path = projects.join(crate::symlink::encode_path(root_to.to_str().ok_or_else(
            || format!("repo move: '{}' is not valid UTF-8", root_to.display()),
        )?));
        if path.symlink_metadata().is_ok() {
            refusals.push(format!(
                "the Claude Code symlink for the new root, {}, is there already",
                path.display()
            ));
        }
        Some(path)
    } else {
        None
    };

    Ok(Plan {
        root,
        root_to,
        repos: plans,
        link,
        link_to,
        agent_to,
        configs: Vec::new(),
        ignores: Vec::new(),
        kept: Vec::new(),
        refusals,
    })
}

/// The lines that say what moves and what stays.
pub fn lines(plan: &Plan) -> Vec<String> {
    let mut out = vec![format!(
        "repo move: the workspace at '{}', to '{}'",
        plan.root.display(),
        plan.root_to.display()
    )];
    for p in &plan.repos {
        match (&p.url_from, &p.url_to) {
            (Some(from), Some(to)) if from != to => {
                out.push(format!("  set {}'s origin URL {from} to {to}", p.label));
            }
            (Some(u), _) => out.push(format!("  keep {}'s origin {u}", p.label)),
            (None, _) => out.push(format!("  {} has no origin", p.label)),
        }
    }
    out.push(format!(
        "  move the work repo {} to {}",
        plan.root.display(),
        plan.root_to.display()
    ));
    match &plan.link {
        Link::Ours(p) => out.push(format!("  remove the symlink {}", p.display())),
        Link::Absent => {}
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
    if let Some(to) = &plan.link_to {
        out.push(format!("  make the symlink {}", to.display()));
    }
    out
}

/// Find the workspace, plan, print the plan, and, confirmed, move it.
pub fn relocate(
    ctx: &mut Context,
    params: &MoveParams,
    cwd: &Path,
    projects: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let plan = plan(params, cwd, projects)?;
    for line in lines(&plan) {
        info!("{line}");
    }
    if !plan.refusals.is_empty() {
        return Err(format!(
            "repo move: nothing is moved:\n  {}",
            plan.refusals.join("\n  ")
        )
        .into());
    }
    if params.dry_run {
        info!("repo move: [dry-run] nothing is moved");
        return Ok(());
    }
    rename::confirm("move", params.yes, ctx.stdin_is_tty)?;
    rename::apply("move", &plan, &rename::Gh)?;
    if cwd.starts_with(&plan.root) {
        info!(
            "repo move: done, the workspace is at {}, and this shell's directory moved with it: \
             `cd` there",
            plan.root_to.display()
        );
    } else {
        info!(
            "repo move: done, the workspace is at {}",
            plan.root_to.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A relative path climbs to the common part and comes down.
    #[test]
    fn a_relative_path_climbs_and_descends() {
        assert_eq!(
            relative_path(Path::new("/a/b/ws"), Path::new("/a/remotes/ws.git")),
            PathBuf::from("../../remotes/ws.git")
        );
        assert_eq!(
            relative_path(Path::new("/a/c/d/ws"), Path::new("/a/remotes/ws.git")),
            PathBuf::from("../../../remotes/ws.git")
        );
        assert_eq!(
            relative_path(Path::new("/a"), Path::new("/a/x")),
            PathBuf::from("x")
        );
    }

    /// Only a `./` or `../` origin is rewritten by a move.
    #[test]
    fn only_a_relative_origin_is_rewritten() {
        assert!(is_relative_path("../remotes/ws.git"));
        assert!(!is_relative_path("/r/ws.git"));
        assert!(!is_relative_path("https://github.com/o/ws.git"));
    }
}

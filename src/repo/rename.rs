//! `vc-x1 repo rename`: give a workspace a new name, its remote
//! included.
//!
//! - The workspace is found as `repo delete` finds it, by
//!   [`delete::survey`], and taken whole: `-s b` a dual workspace,
//!   `-s p` a plain repo.
//! - The new names: the work directory `<parent>/NEW`, the work
//!   remote's last segment `NEW`, the agent remote `NEW<suffix>` when
//!   `[remote] agent-repo` is the old remote name plus a suffix, an
//!   agent directory outside the work-repo `NEW<suffix>` when it is the
//!   old directory's name plus one, and the Claude Code symlink made
//!   for the new root. A name that is not the old one's is kept and
//!   listed, as are the family's names for the workspace.
//! - Everything that would stop it is found before anything changes,
//!   and the plan is printed first, `--dry-run` stopping there.

use std::path::{Path, PathBuf};

use clap::{Args, ValueHint};
use log::{debug, info};

use super::delete::{self, Link, Origin, Survey};
use crate::context::Context;
use crate::options_flags::dry_run::DryRunFlag;
use crate::options_flags::scope::{RepoScope, repo_scope_parser};
use crate::subcommand::SubcommandRunner;

/// Give a workspace a new name.
#[derive(Args, Debug)]
pub struct RenameArgs {
    /// The workspace's root [default: the workspace the current
    /// directory is in]
    #[arg(value_name = "PATH", value_hint = ValueHint::DirPath)]
    pub path: Option<PathBuf>,

    /// The new name, one directory name, and the remote's under it
    #[arg(long, value_name = "NEW")]
    pub to: String,

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

    /// Print what it would rename, and rename nothing
    #[command(flatten)]
    pub dry_run: DryRunFlag,
}

/// Inputs to the rename op, flat, owned, clap-free.
#[derive(Debug)]
pub struct RenameParams {
    pub path: Option<PathBuf>,
    pub to: String,
    pub dual: bool,
    pub dry_run: bool,
}

impl From<&RenameArgs> for RenameParams {
    fn from(a: &RenameArgs) -> Self {
        RenameParams {
            path: a.path.clone(),
            to: a.to.clone(),
            dual: matches!(a.scope, RepoScope::Sides(_)),
            dry_run: a.dry_run.value,
        }
    }
}

impl SubcommandRunner for RenameArgs {
    type Params = RenameParams;

    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(RenameParams::from(self))
    }

    /// Run the `rename` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        debug!("repo rename: enter {params:?}");
        let cwd = std::env::current_dir()?;
        let projects = crate::symlink::default_symlink_dir()?;
        rename(ctx, params, &cwd, &projects)
    }
}

/// What becomes of one repo's remote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Remote {
    /// No `origin`: nothing on a remote to rename.
    None,
    /// A bare repo on this machine, moved to its new path.
    LocalBare { from: PathBuf, to: PathBuf },
    /// A GitHub repo, renamed there, `owner/name` each.
    GitHub { from: String, to: String },
    /// The name is kept, not being the old one's.
    Kept,
}

/// One repo of the workspace, before and after.
#[derive(Debug, Clone)]
pub struct RepoPlan {
    /// `work` or the agent-repo's directory name.
    pub label: String,
    /// `origin`'s URL before and after, `None` with no `origin`.
    pub url_from: Option<String>,
    pub url_to: Option<String>,
    pub remote: Remote,
    /// The repo's directory and, when it is outside the work-repo and
    /// renamed, where it goes.
    pub dir: PathBuf,
    pub dir_to: Option<PathBuf>,
}

/// One edit to a config: in the `.vc-config.md` of `dir`, the key
/// `table.key` from one quoted value to another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEdit {
    pub dir: PathBuf,
    pub key: String,
    pub from: String,
    pub to: String,
}

/// Everything `rename` found and decided, before it changes anything.
#[derive(Debug, Clone)]
pub struct Plan {
    pub root: PathBuf,
    pub root_to: PathBuf,
    /// The work-repo first, then the agent-repo of a dual workspace.
    pub repos: Vec<RepoPlan>,
    pub link: Link,
    /// The symlink's path for the new root, `None` for a plain repo.
    pub link_to: Option<PathBuf>,
    pub configs: Vec<ConfigEdit>,
    /// What keeps the old name, said and not changed.
    pub kept: Vec<String>,
    /// Each a reason nothing is renamed.
    pub refusals: Vec<String>,
}

/// Why `name` cannot be a directory name, or `None` when it can.
pub fn dir_name_problem(name: &str) -> Option<String> {
    if name.is_empty() {
        return Some("the new name is empty".to_string());
    }
    if name == "." || name == ".." {
        return Some(format!("'{name}' names no directory of its own"));
    }
    if name.contains('/') || name.contains('\0') {
        return Some(format!(
            "'{name}' is more than one directory name: `repo move` changes where a workspace is"
        ));
    }
    if name.starts_with('-') {
        return Some(format!(
            "'{name}' begins with '-', which reads as an option"
        ));
    }
    None
}

/// Why GitHub would not take `name` as it is, or `None` when it
/// would: GitHub keeps ASCII letters, digits, `.`, `-`, and `_`,
/// replaces any other character with `-`, and drops a trailing `.git`,
/// so the URL written after a rename would name a repo that is not
/// there.
pub fn github_name_problem(name: &str) -> Option<String> {
    if name.len() > 100 {
        return Some(format!(
            "'{name}' is {} characters, past GitHub's 100",
            name.len()
        ));
    }
    if let Some(c) = name
        .chars()
        .find(|c| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_')))
    {
        let made: String = name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        return Some(format!(
            "'{name}' holds '{c}', which GitHub would change, making the repo '{made}'"
        ));
    }
    if name.ends_with(".git") {
        return Some(format!(
            "'{name}' ends with '.git', which GitHub drops, making the repo '{}'",
            name.trim_end_matches(".git")
        ));
    }
    None
}

/// `path` with its last component replaced by `name`.
fn sibling(path: &Path, name: &str) -> PathBuf {
    match path.parent() {
        Some(p) => p.join(name),
        None => PathBuf::from(name),
    }
}

/// A path's last component as text, empty when it has none.
fn last_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default() // OK: a root path has no name, compared as empty
}

/// The new name for `old` when it is `base` plus a suffix, as
/// `ws.agent-session` is `ws` plus `.agent-session`: `new` plus that
/// suffix, or `None` when `old` is not `base` plus a suffix.
fn renamed_with_suffix(old: &str, base: &str, new: &str) -> Option<String> {
    let suffix = old.strip_prefix(base)?;
    if suffix.is_empty() {
        return None;
    }
    Some(format!("{new}{suffix}"))
}

/// `value`, a path as a config writes it, with its last component
/// `to` when that component is `from`, or `None` when it is not.
fn path_value_renamed(value: &str, from: &str, to: &str) -> Option<String> {
    let trimmed = value.trim_end_matches('/');
    let cut = trimmed.rfind('/').map_or(0, |i| i + 1);
    (&trimmed[cut..] == from).then(|| format!("{}{to}", &trimmed[..cut]))
}

/// One repo's remote as planned.
struct RemotePlan {
    url_from: Option<String>,
    url_to: Option<String>,
    /// The remote's name before, its URL's last segment.
    name_from: String,
    remote: Remote,
}

/// Plan one repo's remote: its URL before and after, its name before,
/// and what becomes of the repo on the remote, a refusal pushed for an
/// `origin` rename cannot handle.
fn plan_remote(
    repo: &delete::Repo,
    name_to: Option<&str>,
    refusals: &mut Vec<String>,
) -> Result<RemotePlan, Box<dyn std::error::Error>> {
    let url = crate::jj::remote_url(&repo.path, "origin")?;
    let Some(url) = url else {
        return Ok(RemotePlan {
            url_from: None,
            url_to: None,
            name_from: String::new(),
            remote: Remote::None,
        });
    };
    let name_from = crate::url::derive_name(&url)?;
    let Some(name_to) = name_to else {
        return Ok(RemotePlan {
            url_from: Some(url.clone()),
            url_to: Some(url),
            name_from,
            remote: Remote::Kept,
        });
    };
    let url_to = crate::url::agent_url(&url, name_to);
    let remote = match &repo.origin {
        Origin::None => Remote::None,
        Origin::Local(from) => {
            let to =
                delete::local_path(&url_to, &repo.path).unwrap_or_else(|| sibling(from, name_to)); // OK: a local origin's new URL is a path too
            Remote::LocalBare {
                from: from.clone(),
                to: crate::common::normalize_path(&to),
            }
        }
        Origin::Url(u) if crate::init::is_github_url(u) => Remote::GitHub {
            from: crate::init::github_slug_from_url(u)?,
            to: crate::init::github_slug_from_url(&url_to)?,
        },
        Origin::Url(u) => {
            refusals.push(format!(
                "{}'s origin {u} is neither GitHub nor a bare repo on this machine, which rename \
                 cannot rename",
                repo.label
            ));
            Remote::Kept
        }
        Origin::NotBare(p) => {
            refusals.push(format!(
                "{}'s origin {} is not a bare repo, which rename does not move",
                repo.label,
                p.display()
            ));
            Remote::Kept
        }
        Origin::Gone(p) => {
            refusals.push(format!(
                "{}'s origin {} is gone: set the origin, or remove it, first",
                repo.label,
                p.display()
            ));
            Remote::Kept
        }
    };
    Ok(RemotePlan {
        url_from: Some(url),
        url_to: Some(url_to),
        name_from,
        remote,
    })
}

/// Find the workspace and plan its rename, changing nothing.
pub fn plan(
    params: &RenameParams,
    cwd: &Path,
    projects: &Path,
) -> Result<Plan, Box<dyn std::error::Error>> {
    let new = params.to.as_str();
    if let Some(problem) = dir_name_problem(new) {
        return Err(format!("repo rename --to: {problem}").into());
    }
    let Survey { root, repos, link } =
        delete::survey("rename", params.path.as_deref(), params.dual, cwd, projects)?;
    let old_dir = last_name(&root);
    let root_to = sibling(&root, new);
    let mut refusals: Vec<String> = Vec::new();
    let mut kept: Vec<String> = Vec::new();
    let mut configs: Vec<ConfigEdit> = Vec::new();

    for r in &repos {
        if r.dirty_at && !r.agent {
            refusals.push(format!(
                "{}: @ has uncommitted changes: commit them first",
                r.label
            ));
        }
        if !r.unpushed.is_empty() {
            refusals.push(format!(
                "{}: {} commit(s) no remote bookmark reaches, {}: push them first",
                r.label,
                r.unpushed.len(),
                r.unpushed.join(" ")
            ));
        }
    }

    let mut plans: Vec<RepoPlan> = Vec::new();
    let work = repos.iter().find(|r| !r.agent);
    let mut work_name_from = old_dir.clone();
    if let Some(w) = work {
        let RemotePlan {
            url_from,
            url_to,
            name_from,
            remote,
        } = plan_remote(w, Some(new), &mut refusals)?;
        if url_from.is_some() {
            work_name_from = name_from;
        }
        plans.push(RepoPlan {
            label: w.label.clone(),
            url_from,
            url_to,
            remote,
            dir: w.path.clone(),
            dir_to: None,
        });
    }
    if let Some(a) = repos.iter().find(|r| r.agent) {
        let configured = crate::common::configured_agent_repo(&root)?;
        let agent_name_from = match (&configured, crate::jj::remote_url(&a.path, "origin")?) {
            (Some(n), _) => Some(n.clone()),
            (None, Some(u)) => Some(crate::url::derive_name(&u)?),
            (None, None) => None,
        };
        let agent_name_to = agent_name_from
            .as_deref()
            .and_then(|n| renamed_with_suffix(n, &work_name_from, new));
        if let (Some(from), None) = (&agent_name_from, &agent_name_to) {
            kept.push(format!(
                "the agent remote '{from}', not '{work_name_from}' plus a suffix"
            ));
        }
        let RemotePlan {
            url_from,
            url_to,
            remote,
            ..
        } = plan_remote(a, agent_name_to.as_deref(), &mut refusals)?;
        if let (Some(from), Some(to)) = (&configured, &agent_name_to) {
            configs.push(ConfigEdit {
                dir: root.clone(),
                key: "remote.agent-repo".to_string(),
                from: from.clone(),
                to: to.clone(),
            });
        }
        let dir_to = if a.outside {
            let name = last_name(&a.path);
            match renamed_with_suffix(&name, &old_dir, new) {
                Some(n) => {
                    let to = sibling(&a.path, &n);
                    edit_path_key(&root, "repos.agent", &name, &n, &mut configs)?;
                    edit_path_key(&a.path, "repos.work", &old_dir, new, &mut configs)?;
                    Some(to)
                }
                None => {
                    kept.push(format!(
                        "the agent directory {}, not '{old_dir}' plus a suffix",
                        a.path.display()
                    ));
                    edit_path_key(&a.path, "repos.work", &old_dir, new, &mut configs)?;
                    None
                }
            }
        } else {
            None
        };
        plans.push(RepoPlan {
            label: a.label.clone(),
            url_from,
            url_to,
            remote,
            dir: a.path.clone(),
            dir_to,
        });
    }

    if plans
        .iter()
        .any(|p| matches!(p.remote, Remote::GitHub { .. }))
        && let Some(problem) = github_name_problem(new)
    {
        refusals.push(format!("--to: {problem}"));
    }
    if root_to == root {
        refusals.push(format!(
            "the workspace is named '{new}' already: nothing to rename"
        ));
    }
    if root_to.symlink_metadata().is_ok() && root_to != root {
        refusals.push(format!("{} is there already", root_to.display()));
    }
    for p in &plans {
        if let Some(to) = &p.dir_to
            && to.symlink_metadata().is_ok()
        {
            refusals.push(format!("{} is there already", to.display()));
        }
        if let Remote::LocalBare { from, to } = &p.remote
            && from != to
            && to.symlink_metadata().is_ok()
        {
            refusals.push(format!(
                "{}'s new origin {} is there already",
                p.label,
                to.display()
            ));
        }
    }

    let link_to = if params.dual {
        let name =
            crate::symlink::encode_path(root_to.to_str().ok_or_else(|| {
                format!("repo rename: '{}' is not valid UTF-8", root_to.display())
            })?);
        let path = projects.join(name);
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

    if let Some(cfg) = crate::config_md::load(&root)?
        && let Some(member) = crate::toml_simple::toml_get(&cfg.map, "family.member")
    {
        kept.push(format!(
            "[family] member '{member}', the family's name for it, and the other members' \
             configs naming it"
        ));
    }

    Ok(Plan {
        root,
        root_to,
        repos: plans,
        link,
        link_to,
        configs,
        kept,
        refusals,
    })
}

/// Push the edit of `key` in `dir`'s config, a path value whose last
/// component is `from`, to end in `to`. Nothing is pushed when the key
/// is absent or ends otherwise.
fn edit_path_key(
    dir: &Path,
    key: &str,
    from: &str,
    to: &str,
    configs: &mut Vec<ConfigEdit>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(cfg) = crate::config_md::load(dir)? else {
        return Ok(());
    };
    if let Some(value) = crate::toml_simple::toml_get(&cfg.map, key)
        && let Some(new) = path_value_renamed(value, from, to)
    {
        configs.push(ConfigEdit {
            dir: dir.to_path_buf(),
            key: key.to_string(),
            from: value.clone(),
            to: new,
        });
    }
    Ok(())
}

impl Plan {
    /// The lines that say what is renamed and what is kept.
    pub fn lines(&self) -> Vec<String> {
        let mut out = vec![format!(
            "repo rename: the workspace at '{}', to '{}'",
            self.root.display(),
            last_name(&self.root_to)
        )];
        for p in &self.repos {
            match &p.remote {
                Remote::None => {
                    out.push(format!("  {} has no origin, no remote to rename", p.label))
                }
                Remote::Kept => {}
                Remote::LocalBare { from, to } => out.push(format!(
                    "  move {}'s origin {} to {}",
                    p.label,
                    from.display(),
                    to.display()
                )),
                Remote::GitHub { from, to } => {
                    out.push(format!("  rename {}'s GitHub repo {from} to {to}", p.label))
                }
            }
        }
        for p in &self.repos {
            if let (Some(from), Some(to)) = (&p.url_from, &p.url_to)
                && from != to
            {
                out.push(format!("  set {}'s origin URL {from} to {to}", p.label));
            }
        }
        for c in &self.configs {
            out.push(format!(
                "  set {} in {}/.vc-config.md from \"{}\" to \"{}\"",
                c.key,
                c.dir.display(),
                c.from,
                c.to
            ));
        }
        for p in &self.repos {
            if let Some(to) = &p.dir_to {
                out.push(format!(
                    "  move the {} repo {} to {}",
                    p.label,
                    p.dir.display(),
                    to.display()
                ));
            }
        }
        out.push(format!(
            "  move the work repo {} to {}",
            self.root.display(),
            self.root_to.display()
        ));
        match &self.link {
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
        if let Some(to) = &self.link_to {
            out.push(format!("  make the symlink {}", to.display()));
        }
        for k in &self.kept {
            out.push(format!("  keep {k}"));
        }
        out
    }
}

/// Find the workspace, plan, and print the plan, refusing what would
/// stop it.
pub fn rename(
    _ctx: &mut Context,
    params: &RenameParams,
    cwd: &Path,
    projects: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let plan = plan(params, cwd, projects)?;
    for line in plan.lines() {
        info!("{line}");
    }
    if !plan.refusals.is_empty() {
        return Err(format!(
            "repo rename: nothing is renamed:\n  {}",
            plan.refusals.join("\n  ")
        )
        .into());
    }
    if params.dry_run {
        info!("repo rename: [dry-run] nothing is renamed");
        return Ok(());
    }
    Err("repo rename: only the plan is built so far: --dry-run prints it".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A name with a slash, a dot pair, or a leading dash is refused
    /// as a directory name, and an ordinary one is not.
    #[test]
    fn a_new_name_is_one_directory_name() {
        assert_eq!(dir_name_problem("ws2"), None);
        assert!(dir_name_problem("").is_some());
        assert!(dir_name_problem("..").is_some());
        assert!(dir_name_problem("a/b").unwrap().contains("repo move"));
        assert!(dir_name_problem("-x").is_some());
    }

    /// GitHub keeps letters, digits, `.`, `-`, and `_`, and says what
    /// it would make of anything else.
    #[test]
    fn a_name_github_would_change_is_named() {
        assert_eq!(github_name_problem("my_repo-2.x"), None);
        let p = github_name_problem("a+b=c").unwrap();
        assert!(p.contains("'a-b-c'"), "{p}");
        assert!(github_name_problem("x.git").unwrap().contains("'x'"));
        assert!(github_name_problem(&"a".repeat(101)).is_some());
    }

    /// The agent name follows the work name when it is the work name
    /// plus a suffix, and is kept otherwise.
    #[test]
    fn a_suffixed_name_follows_the_base() {
        assert_eq!(
            renamed_with_suffix("ws.agent-session", "ws", "ws2"),
            Some("ws2.agent-session".to_string())
        );
        assert_eq!(renamed_with_suffix("ws", "ws", "ws2"), None);
        assert_eq!(renamed_with_suffix("other.claude", "ws", "ws2"), None);
    }

    /// A config path keeps everything but its last component.
    #[test]
    fn a_path_value_changes_its_last_component() {
        assert_eq!(
            path_value_renamed(
                "../ws.agent-session",
                "ws.agent-session",
                "ws2.agent-session"
            ),
            Some("../ws2.agent-session".to_string())
        );
        assert_eq!(path_value_renamed("..", "ws", "ws2"), None);
        assert_eq!(
            path_value_renamed("../ws", "ws", "ws2"),
            Some("../ws2".to_string())
        );
    }
}

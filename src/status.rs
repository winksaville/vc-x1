//! `status` subcommand, alias `st`: the scoped repos' working-copy
//! state in one call, and the verdict At rest's "clean" asks for.
//!
//! - `StatusArgs`: clap surface, a `SCOPE` positional or `-s`, and
//!   a `-R` workspace root.
//! - `StatusParams`: clap-free, the labeled repos to report.
//! - `status(&Context, &StatusParams)`: the op. Prints each repo
//!   under its label, `work` and the agent dir's name, with the
//!   changed paths and the `@` and `@-` lines as `jj st` prints
//!   them, then one verdict line: `clean` when every scoped `@` is
//!   empty and undescribed, `dirty` naming the repos that are not.
//!   `work` needs no config, so a plain jj repo answers for it.

use std::path::{Path, PathBuf};

use clap::Args;
use log::info;

use crate::common;
use crate::context::Context;
use crate::jj;
use crate::options_flags::scope::{Scope, Side, scope_keywords};
use crate::subcommand::SubcommandRunner;

/// CLI args for `status`.
#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Side(s) to report [default: work]
    #[arg(value_name = "SCOPE", value_parser = scope_keywords(), conflicts_with = "scope")]
    pub pos_scope: Option<Scope>,

    /// Side(s) to report, the positional's flag form
    #[arg(short = 's', long = "scope", value_name = "SCOPE", value_parser = scope_keywords())]
    pub scope: Option<Scope>,

    /// Workspace root or single jj repo path [default: the workspace
    /// around the current directory, else .]
    #[arg(short = 'R', long = "repo", value_name = "PATH")]
    pub repo: Option<PathBuf>,
}

/// Clap-free params for `status`: the labeled repos to report, in
/// scope order.
#[derive(Debug)]
pub struct StatusParams {
    pub repos: Vec<(String, PathBuf)>,
}

impl TryFrom<&StatusArgs> for StatusParams {
    type Error = String;

    /// The scope from the positional or `-s`, `work` when neither;
    /// the root from `-R`, else the workspace around the current
    /// directory, else none, which is the plain-repo case where
    /// `work` is `.` and the other scopes have nothing to resolve.
    fn try_from(a: &StatusArgs) -> Result<Self, String> {
        let scope = a
            .pos_scope
            .clone()
            .or_else(|| a.scope.clone())
            .unwrap_or(Scope(vec![Side::Work]));
        let root = match &a.repo {
            Some(p) => Some(
                p.canonicalize()
                    .map_err(|e| format!("cannot resolve repo path '{}': {e}", p.display()))?,
            ),
            None => {
                let cwd = std::env::current_dir().map_err(|e| format!("current dir: {e}"))?;
                resolve_root(&cwd).map_err(|e| e.to_string())?
            }
        };
        let repos = labeled_repos(&scope, root.as_deref()).map_err(|e| e.to_string())?;
        Ok(StatusParams { repos })
    }
}

/// The nearest ancestor of `start`, itself included, holding a
/// `.jj` directory: the repo `jj st` would report from there.
pub(crate) fn nearest_jj(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|d| d.join(".jj").is_dir())
        .map(Path::to_path_buf)
}

/// The root to resolve scopes against from `start`: the workspace
/// around it, unless a nearer jj repo is neither of the
/// workspace's sides, in which case that repo is a plain repo
/// nested in the tree and answers as itself. `None` outside any
/// workspace, the plain case where `work` is `.`.
fn resolve_root(start: &Path) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let Some(root) = common::find_workspace_root_from(start) else {
        return Ok(None);
    };
    let Some(nearest) = nearest_jj(start) else {
        return Ok(Some(root));
    };
    let nearest = nearest.canonicalize()?;
    let root = root.canonicalize()?;
    if nearest == root {
        return Ok(Some(root));
    }
    let bot = common::configured_bot_dir(&root)?
        .map(|b| b.canonicalize())
        .transpose()?;
    if bot.as_deref() == Some(nearest.as_path()) {
        return Ok(Some(root));
    }
    Ok(Some(nearest))
}

impl SubcommandRunner for StatusArgs {
    type Params = StatusParams;

    fn to_params(&self) -> Result<Self::Params, String> {
        StatusParams::try_from(self)
    }

    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        status(ctx, params)
    }
}

/// One repo's verdict: `None` when clean, else why it is not.
fn dirt(st: &jj::WcStatus) -> Option<&'static str> {
    match (st.empty, st.described) {
        (true, false) => None,
        (false, _) => Some("@ has changes"),
        (true, true) => Some("@ is described"),
    }
}

/// Render one repo's block, `jj st`'s shape under a label line.
fn render(label: &str, path: &Path, st: &jj::WcStatus) -> String {
    let mut out = format!("{label} ({}):\n", path.display());
    if st.changes.is_empty() {
        out.push_str("The working copy has no changes.\n");
    } else {
        out.push_str("Working copy changes:\n");
        for (letter, path) in &st.changes {
            out.push_str(&format!("{letter} {path}\n"));
        }
    }
    out.push_str(&format!("Working copy  (@) : {}\n", st.wc_line));
    for p in &st.parent_lines {
        out.push_str(&format!("Parent commit (@-): {p}\n"));
    }
    out
}

/// The scoped repos of the workspace at `root`, labeled: the work
/// side is `work`, the agent side its directory's name. Paths come
/// from `scope_to_repos`, so `agent` outside a dual workspace is
/// its error, and `work` with no root is `.`, canonicalized.
fn labeled_repos(
    scope: &Scope,
    root: Option<&Path>,
) -> Result<Vec<(String, PathBuf)>, Box<dyn std::error::Error>> {
    let paths = common::scope_to_repos(scope, root)?;
    scope
        .0
        .iter()
        .zip(paths)
        .map(|(side, path)| {
            let path = path
                .canonicalize()
                .map_err(|e| format!("cannot resolve repo path '{}': {e}", path.display()))?;
            let label = match side {
                Side::Work => "work".to_string(),
                Side::Bot => path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "agent".to_string()),
            };
            Ok((label, path))
        })
        .collect()
}

/// The verdict line over the labeled statuses.
fn verdict(reports: &[(String, jj::WcStatus)]) -> String {
    let dirty: Vec<String> = reports
        .iter()
        .filter_map(|(label, st)| dirt(st).map(|why| format!("{label} {why}")))
        .collect();
    if dirty.is_empty() {
        "status: clean".to_string()
    } else {
        format!("status: dirty: {}", dirty.join(", "))
    }
}

/// Print every scoped repo's status and the verdict.
pub fn status(_ctx: &Context, params: &StatusParams) -> Result<(), Box<dyn std::error::Error>> {
    let mut reports = Vec::new();
    for (label, path) in &params.repos {
        let st = jj::wc_status(path)?;
        info!("{}", render(label, path, &st));
        reports.push((label.clone(), st));
    }
    info!("{}", verdict(&reports));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{Fixture, FixturePor};
    use clap::Parser;

    #[derive(Parser)]
    struct T {
        #[command(flatten)]
        a: StatusArgs,
    }

    fn parse(args: &[&str]) -> StatusArgs {
        T::try_parse_from(args).expect("parse").a
    }

    /// The positional and `-s` name the same scope, `both` reads
    /// as `work,agent`, neither means `work`, and the two forms
    /// together are refused.
    #[test]
    fn scope_positional_and_flag() {
        let both = Scope(vec![Side::Work, Side::Bot]);
        assert_eq!(parse(&["t", "both"]).pos_scope, Some(both.clone()));
        assert_eq!(parse(&["t", "-s", "both"]).scope, Some(both));
        assert_eq!(
            parse(&["t", "agent"]).pos_scope,
            Some(Scope(vec![Side::Bot]))
        );
        let none = parse(&["t"]);
        assert!(none.pos_scope.is_none() && none.scope.is_none());
        assert!(T::try_parse_from(["t", "work", "-s", "agent"]).is_err());
        assert!(T::try_parse_from(["t", "everything"]).is_err());
    }

    /// A plain repo answers for `work` as itself and has no agent
    /// side to resolve.
    #[test]
    fn por_answers_for_work_only() {
        let fx = FixturePor::new("status-por");
        let work = Scope(vec![Side::Work]);
        let repos = labeled_repos(&work, Some(&fx.work)).unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].0, "work");
        assert_eq!(repos[0].1, fx.work.canonicalize().unwrap());
        let st = jj::wc_status(&repos[0].1).unwrap();
        assert!(st.changes.is_empty(), "{st:?}");
        let err = labeled_repos(&Scope(vec![Side::Bot]), Some(&fx.work)).unwrap_err();
        assert!(err.to_string().contains("--scope=agent"), "{err}");
    }

    /// From the work root, the agent dir, and a subdirectory the
    /// root is the workspace; from a plain repo nested in the tree
    /// the root is that repo; outside any workspace there is none.
    #[test]
    fn resolve_root_stops_at_a_nested_plain_repo() {
        let fx = Fixture::new("status-nested");
        let root = fx.work.canonicalize().unwrap();
        assert_eq!(resolve_root(&fx.work).unwrap(), Some(root.clone()));
        assert_eq!(resolve_root(&fx.bot).unwrap(), Some(root.clone()));
        let sub = fx.work.join("src");
        std::fs::create_dir_all(&sub).unwrap();
        assert_eq!(resolve_root(&sub).unwrap(), Some(root.clone()));
        let nested = fx.work.join("tmp").join("plain");
        std::fs::create_dir_all(&nested).unwrap();
        jj::git_init_colocated(&nested).unwrap();
        let deeper = nested.join("deep");
        std::fs::create_dir_all(&deeper).unwrap();
        let plain = nested.canonicalize().unwrap();
        assert_eq!(resolve_root(&nested).unwrap(), Some(plain.clone()));
        assert_eq!(resolve_root(&deeper).unwrap(), Some(plain.clone()));
        let repos = labeled_repos(&Scope(vec![Side::Work]), Some(&plain)).unwrap();
        assert_eq!(repos, vec![("work".to_string(), plain.clone())]);
        assert!(labeled_repos(&Scope(vec![Side::Bot]), Some(&plain)).is_err());
        assert_eq!(resolve_root(&fx.base).unwrap(), None);
    }

    /// A fresh dual workspace is clean under `both`: both `@`
    /// empty and undescribed, no changed paths, and the verdict
    /// says so.
    #[test]
    fn fresh_workspace_is_clean() {
        let fx = Fixture::new("status-clean");
        let both = Scope(vec![Side::Work, Side::Bot]);
        let repos = labeled_repos(&both, Some(&fx.work)).unwrap();
        assert_eq!(repos[0].0, "work");
        assert_eq!(repos[1].0, ".claude");
        let reports: Vec<(String, jj::WcStatus)> = repos
            .into_iter()
            .map(|(l, p)| (l, jj::wc_status(&p).unwrap()))
            .collect();
        for (_, st) in &reports {
            assert!(st.changes.is_empty(), "{st:?}");
            assert!(
                st.wc_line.contains("(empty) (no description set)"),
                "{st:?}"
            );
            assert_eq!(st.parent_lines.len(), 1, "{st:?}");
        }
        assert_eq!(verdict(&reports), "status: clean");
    }

    /// A new file on the work side shows as `A` under its label,
    /// and the verdict names the repo and why.
    #[test]
    fn a_new_file_makes_the_work_side_dirty() {
        let fx = Fixture::new("status-dirty");
        std::fs::write(fx.work.join("a.txt"), "one\n").unwrap();
        let st = jj::wc_status(&fx.work).unwrap();
        assert_eq!(st.changes, vec![('A', "a.txt".to_string())]);
        assert!(!st.empty);
        let block = render("work", &fx.work, &st);
        assert!(block.starts_with("work ("), "{block}");
        assert!(
            block.contains("Working copy changes:\nA a.txt\n"),
            "{block}"
        );
        assert!(block.contains("Working copy  (@) : "), "{block}");
        assert!(block.contains("Parent commit (@-): "), "{block}");
        let bot = jj::wc_status(&fx.bot).unwrap();
        let reports = vec![("work".to_string(), st), (".claude".to_string(), bot)];
        assert_eq!(verdict(&reports), "status: dirty: work @ has changes");
    }

    /// An edit and a delete carry their letters, and a described
    /// empty `@` is dirty for its description.
    #[test]
    fn letters_and_a_described_empty_wc() {
        let fx = Fixture::new("status-letters");
        std::fs::write(fx.work.join("a.txt"), "one\n").unwrap();
        std::fs::write(fx.work.join("b.txt"), "one\n").unwrap();
        jj::commit(&fx.work, "test: seed a.txt and b.txt").unwrap();
        std::fs::write(fx.work.join("a.txt"), "two\n").unwrap();
        std::fs::remove_file(fx.work.join("b.txt")).unwrap();
        let st = jj::wc_status(&fx.work).unwrap();
        assert_eq!(
            st.changes,
            vec![('M', "a.txt".to_string()), ('D', "b.txt".to_string())]
        );
        jj::commit(&fx.work, "test: edit and delete").unwrap();
        jj::describe(&fx.work, "@", "wip: an intent").unwrap();
        let st = jj::wc_status(&fx.work).unwrap();
        assert!(st.empty && st.described, "{st:?}");
        assert_eq!(dirt(&st), Some("@ is described"));
        assert!(st.wc_line.ends_with("(empty) wip: an intent"), "{st:?}");
    }
}

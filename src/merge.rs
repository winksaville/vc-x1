//! The `merge` subcommand: add the sources named to one target
//! commit's parents, after the ones it has, keeping the target's own
//! change, description, and change id, so its `ochid:` trailer stays
//! right.
//!
//! One target and N sources, with no assumption about where either
//! lives: the target is any commit, empty or not, and each source is
//! a revset naming one or more commits, a bookmark, `main`, a change
//! id, a merge made earlier, a root, a set of commits merged in
//! one at a time or all at once.
//!
//! - The meaning is `jj rebase -r <target> -o <p1> ... -o <s1> ...`,
//!   the target's parents then the sources: the target's tree becomes
//!   the merge of them all with its own change applied, so it carries
//!   every source's changes. Merging the sources one at a time and
//!   all at once give the same tree, and all at once is one jj op.
//! - A source that is already a parent is skipped, and a merge that
//!   adds nothing changes nothing. A revset's commits are added oldest
//!   first. Removing a parent is not a merge: `jj undo` takes back the
//!   last one, and `jj rebase -r <target> -o ...` sets any parents.
//! - A cycle's closing is `main` and the last rung as `@`'s parents,
//!   in that order, which `commit --closing` sets before it commits.
//! - Refused: a source that is the target or below it, a cycle, and
//!   a target in jj's `immutable()`, as `jj` refuses it, unless
//!   `--ignore-immutable`. A pushed rung is mutable to jj, and the
//!   push that follows its rewrite takes approval like any other.
//!   Reported and allowed: a conflicted result, left as jj leaves it.
//!   A source that is an ancestor of another is the no-fast-forward
//!   shape, a closing's `main` below its last rung, so it is neither.
//! - `-s w` merges in the work repo, `-s p` in a plain repo. The
//!   agent repo is a linear journal that never holds a merge, so
//!   `-s a` and `-s b` are refused.
//! - A failure restores the repo from a `jj op` snapshot.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info, warn};

use crate::context::Context;
use crate::jj;
use crate::options_flags::dry_run::DryRunFlag;
use crate::options_flags::scope::{REPO_SCOPE_FORMS, RepoScope, parse_repo_scope};
use crate::pair::{Snapshot, refuse_conflicts, resolve_pair};
use crate::subcommand::SubcommandRunner;

/// Add the sources to a target commit's parents, keeping its change
/// and change id.
#[derive(Args, Debug)]
pub struct MergeArgs {
    /// The commit whose parents grow, any commit, typically @
    #[arg(long, value_name = "REV", required = true)]
    pub target: String,

    /// A source, a revset of one or more commits added to the
    /// target's parents in the order given; repeat for more
    #[arg(long = "from", value_name = "REVSET", required = true)]
    pub from: Vec<String>,

    /// Merge into a target in jj's `immutable()`, as `jj` allows it
    #[arg(long)]
    pub ignore_immutable: bool,

    /// Where: a path in the workspace for `-s w`, or the repo itself
    /// for `-s p` [default: the current directory]
    #[arg(short = 'R', long, value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Which repo, required: `w|work` or `p|por`; the agent repo
    /// never holds a merge
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = parse_repo_scope,
        required = true,
        help = format!("Which repo, required: {REPO_SCOPE_FORMS}, `a` and `b` refused")
    )]
    pub scope: RepoScope,

    /// Print the parents it would give, and change nothing
    #[command(flatten)]
    pub dry_run: DryRunFlag,
}

/// Inputs to the merge op, flat, owned, clap-free.
#[derive(Debug)]
pub struct MergeParams {
    pub repo: PathBuf,
    pub target: String,
    pub from: Vec<String>,
    pub ignore_immutable: bool,
    pub dry_run: bool,
}
impl TryFrom<&MergeArgs> for MergeParams {
    type Error = String;

    /// Resolve the scope to the one repo, refusing the agent side.
    fn try_from(a: &MergeArgs) -> Result<Self, String> {
        let repo = match &a.scope {
            RepoScope::Por => {
                let repos = crate::common::resolve_scoped_repos(&a.scope, a.repo.as_deref())
                    .map_err(|e| e.to_string())?;
                match repos.as_slice() {
                    [repo] => repo.clone(),
                    other => {
                        return Err(format!(
                            "merge -s p: expected one repo, got {}",
                            other.len()
                        ));
                    }
                }
            }
            RepoScope::Sides(sides) if sides.is_work_only() => {
                resolve_pair(a.repo.as_deref())
                    .map_err(|e| e.to_string())?
                    .work
            }
            RepoScope::Sides(_) => {
                return Err(
                    "merge: the agent repo is a linear journal that never holds a \
                            merge, so the scope is `-s w`, or `-s p` for a plain repo"
                        .into(),
                );
            }
        };
        Ok(MergeParams {
            repo,
            target: a.target.clone(),
            from: a.from.clone(),
            ignore_immutable: a.ignore_immutable,
            dry_run: a.dry_run.value,
        })
    }
}

impl SubcommandRunner for MergeArgs {
    type Params = MergeParams;

    /// Delegate to the `TryFrom<&MergeArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        MergeParams::try_from(self)
    }

    /// Run the `merge` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        merge(ctx, params)
    }
}

/// The sources resolved to commit ids, each revset's oldest first,
/// refusing a revset that names no commit and one with a commit at
/// or below the target. A source that is an ancestor of another is
/// allowed and unremarked: it is the no-fast-forward shape, a
/// closing's `main` below its last rung.
pub fn resolve_sources(
    repo: &Path,
    target: &str,
    from: &[String],
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut ids = Vec::with_capacity(from.len());
    for source in from {
        if jj::matches(repo, &format!("({source}) & ({target})::"))? {
            return Err(format!(
                "merge: the source '{source}' is the target '{target}' or below it, so it \
                 cannot be its parent"
            )
            .into());
        }
        let found = jj::cids_of(repo, source)?;
        if found.is_empty() {
            return Err(format!("merge: the source '{source}' names no commit").into());
        }
        ids.extend(found);
    }
    Ok(ids)
}

/// The target's `parents` with the `sources` not among them added
/// after, in order, and the sources that were added.
fn appended(parents: &[String], sources: &[String]) -> (Vec<String>, Vec<String>) {
    let mut all = parents.to_vec();
    let mut added = Vec::new();
    for s in sources {
        if !all.contains(s) {
            all.push(s.clone());
            added.push(s.clone());
        }
    }
    (all, added)
}

/// The 12-character forms of `ids`, joined for a log line.
fn shown(ids: &[String]) -> String {
    ids.iter()
        .map(|p| &p[..p.len().min(12)])
        .collect::<Vec<_>>()
        .join(", ")
}

/// Give the target the sources' commit ids as its parents, all of
/// them, in order, in `repo` under `who`, returning its new commit id
/// and warning when the result is conflicted. The caller has checked
/// and snapshotted.
pub fn set_parents(
    ctx: &mut Context,
    repo: &Path,
    target: &str,
    sources: &[String],
    who: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let refs: Vec<&str> = sources.iter().map(String::as_str).collect();
    let new_id = ctx.session(repo)?.reparent(target, &refs)?;
    if jj::matches(repo, &format!("{new_id} & conflicts()"))? {
        warn!(
            "{who}: the merge is conflicted, its sources changing the same lines: `jj resolve \
             --list -r {}` names the paths, and resolving them, or another merge, is yours",
            &new_id[..new_id.len().min(12)]
        );
    }
    Ok(new_id)
}

/// Add the sources to the target's parents, refusing before any
/// change a conflicted repo, an immutable target unless
/// `--ignore-immutable`, and a source at or below the target.
pub fn merge(ctx: &mut Context, params: &MergeParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("merge: enter {params:?}");
    let repo = &params.repo;
    let target = &params.target;
    refuse_conflicts(repo, "merge", "target's")?;
    let parents =
        jj::parent_cids_of(repo, target).map_err(|e| format!("merge: the target: {e}"))?;
    if !params.ignore_immutable && jj::is_immutable(repo, target)? {
        return Err(format!(
            "merge: the target '{target}' is immutable, in jj's `immutable()`, so `jj` \
             would refuse to rewrite it. Pass --ignore-immutable to merge into it anyway"
        )
        .into());
    }
    let chid = jj::chid_of(repo, target)?;
    let sources = resolve_sources(repo, target, &params.from)?;
    let (all, added) = appended(&parents, &sources);
    if added.is_empty() {
        info!(
            "merge: every source is already a parent of {chid}, so nothing changes, in {}",
            repo.display()
        );
        return Ok(());
    }
    if params.dry_run {
        info!(
            "merge: [dry-run] would add {} to {chid}'s parents, making them {}, in {}",
            shown(&added),
            shown(&all),
            repo.display()
        );
        return Ok(());
    }
    let snapshot = Snapshot::take(repo)?;
    match set_parents(ctx, repo, target, &all, "merge") {
        Ok(new_id) => {
            info!(
                "merge: {chid} has the parents {}, now commit {}",
                shown(&all),
                &new_id[..new_id.len().min(12)]
            );
            Ok(())
        }
        Err(e) => {
            snapshot.restore("merge", e.as_ref());
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{Fixture, chid, jj_ok, test_ctx};
    use crate::{Cli, Commands};
    use clap::Parser;
    use std::fs;

    fn parse(args: &[&str]) -> Result<MergeArgs, String> {
        match Cli::try_parse_from(args) {
            Ok(Cli {
                command: Some(Commands::Merge(a)),
                ..
            }) => Ok(a),
            Ok(_) => panic!("expected Merge"),
            Err(e) => Err(e.to_string()),
        }
    }

    fn commit_id(repo: &Path, rev: &str) -> String {
        jj_ok(repo, &["log", "-r", rev, "--no-graph", "-T", "commit_id"])
    }

    fn parents(repo: &Path, rev: &str) -> Vec<String> {
        jj_ok(
            repo,
            &[
                "log",
                "-r",
                rev,
                "--no-graph",
                "-T",
                "parents.map(|p| p.commit_id()).join(\" \")",
            ],
        )
        .split_whitespace()
        .map(str::to_string)
        .collect()
    }

    fn files(repo: &Path, rev: &str) -> String {
        jj_ok(repo, &["file", "list", "-r", rev])
    }

    fn params(fx: &Fixture, target: &str, from: &[&str]) -> MergeParams {
        MergeParams {
            repo: fx.work.clone(),
            target: target.to_string(),
            from: from.iter().map(|s| s.to_string()).collect(),
            ignore_immutable: false,
            dry_run: false,
        }
    }

    /// Write `file` with `text` and commit it with `title` on `@`.
    fn commit_file(repo: &Path, file: &str, text: &str, title: &str) {
        fs::write(repo.join(file), text).expect("write");
        jj_ok(repo, &["commit", "-m", title]);
    }

    /// A commit on `base` adding `file`, returning its commit id, with
    /// `@` left a new empty commit on `base`.
    fn side(repo: &Path, base: &str, file: &str) -> String {
        jj_ok(repo, &["new", base]);
        commit_file(repo, file, file, &format!("feat: {file}"));
        let id = commit_id(repo, "@-");
        jj_ok(repo, &["new", base]);
        id
    }

    /// `-s`, `--target`, and `--from` are required, and the agent side
    /// is refused.
    #[test]
    fn scope_target_and_from_are_required_and_the_agent_side_is_refused() {
        let err = parse(&["vc-x1", "merge", "--target", "@", "--from", "main"]).unwrap_err();
        assert!(err.contains("--scope"), "{err}");
        let err = parse(&["vc-x1", "merge", "-s", "w", "--from", "main"]).unwrap_err();
        assert!(err.contains("--target"), "{err}");
        let err = parse(&["vc-x1", "merge", "-s", "w", "--target", "@"]).unwrap_err();
        assert!(err.contains("--from"), "{err}");
        let fx = Fixture::new("merge-agent");
        for scope in ["a", "b"] {
            let mut a = parse(&[
                "vc-x1", "merge", "-s", scope, "--target", "@", "--from", "main",
            ])
            .unwrap();
            a.repo = Some(fx.work.clone());
            let err = MergeParams::try_from(&a).unwrap_err();
            assert!(err.contains("linear journal"), "{scope}: {err}");
        }
    }

    /// Sources are added after the target's parents, two at once and
    /// then one more, its tree carrying every source and its change
    /// id kept, and adding a parent again changes nothing.
    #[test]
    fn sources_are_added_after_the_parents_at_once_or_one_at_a_time() {
        let fx = Fixture::new("merge-add");
        let base = commit_id(&fx.work, "main");
        let a = side(&fx.work, &base, "a.txt");
        let b = side(&fx.work, &base, "b.txt");
        let c = side(&fx.work, &base, "c.txt");
        fs::write(fx.work.join("own.txt"), "own").expect("write");
        let id = chid(&fx.work, "@");

        merge(&mut test_ctx(), &params(&fx, "@", &[&a, &b])).expect("two at once");
        assert_eq!(parents(&fx.work, "@"), [base.clone(), a.clone(), b.clone()]);
        merge(&mut test_ctx(), &params(&fx, "@", &[&c])).expect("one more");
        assert_eq!(parents(&fx.work, "@"), [base, a.clone(), b, c]);
        let tree = files(&fx.work, "@");
        for f in ["a.txt", "b.txt", "c.txt", "own.txt"] {
            assert!(tree.contains(f), "{f}: {tree}");
        }
        assert_eq!(chid(&fx.work, "@"), id);

        let before = commit_id(&fx.work, "@");
        merge(&mut test_ctx(), &params(&fx, "@", &[&a])).expect("already a parent");
        assert_eq!(commit_id(&fx.work, "@"), before);
    }

    /// A revset naming several commits adds them all, oldest first.
    #[test]
    fn a_revset_of_several_commits_adds_them_oldest_first() {
        let fx = Fixture::new("merge-revset");
        let base = commit_id(&fx.work, "main");
        let a = side(&fx.work, &base, "a.txt");
        let b = side(&fx.work, &base, "b.txt");

        merge(&mut test_ctx(), &params(&fx, "@", &[&format!("{b} | {a}")])).expect("merge");
        assert_eq!(parents(&fx.work, "@"), [base, a, b]);
    }

    /// A source with no history of its own, a root's child from an
    /// unrelated line, merges like any other.
    #[test]
    fn a_rootless_source_merges() {
        let fx = Fixture::new("merge-root");
        let main = commit_id(&fx.work, "main");
        let other = side(&fx.work, "root()", "other.txt");
        jj_ok(&fx.work, &["new", &main]);

        merge(&mut test_ctx(), &params(&fx, "@", &[&other])).expect("merge");
        assert_eq!(parents(&fx.work, "@"), [main, other]);
        assert!(files(&fx.work, "@").contains("other.txt"));
    }

    /// Sources changing the same lines leave the target conflicted, as
    /// jj leaves it: reported, not refused.
    #[test]
    fn a_conflicted_result_is_reported_and_left() {
        let fx = Fixture::new("merge-conflict");
        let base = commit_id(&fx.work, "main");
        commit_file(&fx.work, "same.txt", "one\n", "feat: one side");
        let a = commit_id(&fx.work, "@-");
        jj_ok(&fx.work, &["new", &base]);
        commit_file(&fx.work, "same.txt", "other\n", "feat: other side");
        let b = commit_id(&fx.work, "@-");
        jj_ok(&fx.work, &["new", &base]);

        merge(&mut test_ctx(), &params(&fx, "@", &[&a, &b])).expect("a conflict is allowed");
        assert!(crate::jj::matches(&fx.work, "@ & conflicts()").expect("query"));
    }

    /// A source at or below the target, a source naming no commit, and
    /// an immutable target are refused before anything moves, and a
    /// dry run moves nothing.
    #[test]
    fn refusals_and_a_dry_run_change_nothing() {
        let fx = Fixture::new("merge-refused");
        let other = side(&fx.work, "root()", "other.txt");
        jj_ok(&fx.work, &["new", "main"]);
        commit_file(&fx.work, "one.txt", "one", "feat: rung one");
        commit_file(&fx.work, "two.txt", "two", "feat: rung two");
        let before = parents(&fx.work, "@-");

        let err = merge(&mut test_ctx(), &params(&fx, "@-", &["@"])).unwrap_err();
        assert!(err.to_string().contains("below it"), "{err}");

        let err = merge(&mut test_ctx(), &params(&fx, "@-", &["none()"])).unwrap_err();
        assert!(err.to_string().contains("names no commit"), "{err}");

        let mut dry = params(&fx, "@-", &[&other]);
        dry.dry_run = true;
        merge(&mut test_ctx(), &dry).expect("dry run");
        assert_eq!(parents(&fx.work, "@-"), before);

        let err = merge(&mut test_ctx(), &params(&fx, "main", &[&other])).unwrap_err();
        assert!(err.to_string().contains("immutable"), "{err}");
    }

    /// A rung pushed to a tracked bookmark is mutable to jj, so it
    /// takes a merge, and an immutable target takes one with
    /// `--ignore-immutable`.
    #[test]
    fn a_pushed_rung_merges_and_an_immutable_target_takes_the_flag() {
        let fx = Fixture::new("merge-mutable");
        let main = commit_id(&fx.work, "main");
        let other = side(&fx.work, "root()", "other.txt");
        jj_ok(&fx.work, &["new", &main]);
        commit_file(&fx.work, "rung.txt", "rung", "feat: a rung");
        let rung = commit_id(&fx.work, "@-");
        jj_ok(&fx.work, &["bookmark", "create", "cycle", "-r", &rung]);
        jj_ok(&fx.work, &["git", "push", "--bookmark", "cycle"]);
        assert!(crate::pair::is_published(&fx.work, "@-").expect("query"));

        merge(&mut test_ctx(), &params(&fx, "@-", &[&other])).expect("a pushed rung");
        assert_eq!(parents(&fx.work, "@-"), [main.clone(), other.clone()]);

        // A trunk commit with a parent, since git holds no merge with
        // the root as a parent.
        jj_ok(&fx.work, &["new", &main]);
        commit_file(&fx.work, "trunk.txt", "trunk", "feat: on trunk");
        jj_ok(&fx.work, &["bookmark", "set", "main", "-r", "@-"]);
        jj_ok(&fx.work, &["git", "push", "--bookmark", "main"]);
        let side_b = side(&fx.work, &main, "b.txt");
        let err = merge(&mut test_ctx(), &params(&fx, "main", &[&side_b])).unwrap_err();
        assert!(err.to_string().contains("--ignore-immutable"), "{err}");
        let mut forced = params(&fx, "main", &[&side_b]);
        forced.ignore_immutable = true;
        merge(&mut test_ctx(), &forced).expect("immutable, with the flag");
        assert_eq!(parents(&fx.work, "main"), [main, side_b]);
    }
}

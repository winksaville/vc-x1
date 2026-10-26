//! Land: make a finished cycle permanent on its trunk. The trunk is
//! fast-forwarded to the cycle's last commit and pushed with the agent
//! half as its partner, and the cycle's bookmark is deleted locally and
//! on `origin`, every check made before anything moves.
//!
//! - [`plan`] refuses before anything is made: the trunk is a local
//!   bookmark at its `origin` position, so the sync a Land begins with
//!   has nothing to bring, and the branch descends from it, so the tree
//!   landed is the reviewed one. The branch's other bookmarks are named
//!   as merged and left as they are.
//! - [`run`] lands what the plan names: a conflicted target is refused,
//!   the trunk is fast-forwarded and pushed, and the cycle's bookmark is
//!   deleted, `--skip` keeping a side. A dry run makes every check and
//!   says each step, and does none: each step is its line and then the
//!   step behind `if !dry`, so a step added brings its dry-run line.
//!
//! `vc-x1 land -s b|p --source <BM|REV> --onto <BOOKMARK>` lands a cycle
//! whose last commit is already made, and `commit --closing ... --land`
//! plans its Land before the closing is made and runs it after, both
//! through these two.
//!
//! - `--source` names the cycle's last commit, a bookmark or a revision,
//!   and the cycle's bookmark by the rule [`cycle_bookmark`] states,
//!   `--closing`'s.
//! - `--onto` is any local bookmark `--source` descends from, so a
//!   bushy tree lands anywhere. Land only fast-forwards: a merge is
//!   `commit --closing`'s to make.
//! - `--delete-intermediate-bookmarks` deletes the bookmarks on the
//!   commits strictly between `--onto` and `--source`, the earlier
//!   cycles' of a stack landing together. Without it they are named as
//!   merged and kept, and a bookmark at the tip besides the cycle's is
//!   kept either way.
//! - The publish follows where the target is: the work `@-`, whose pair
//!   may be unfinished, goes through `push`, which finishes it, and a
//!   commit below it, whose pair is finished, by its trunk pushed alone
//!   and the agent `main` published.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info};

use crate::commit::Skip;
use crate::context::Context;
use crate::jj;
use crate::options_flags::dry_run::DryRunFlag;
use crate::options_flags::scope::{RepoScope, repo_scope_parser};
use crate::options_flags::squash::SquashSpec;
use crate::subcommand::SubcommandRunner;

/// Land a cycle whose last commit is made: `--onto` fast-forwarded to
/// `--source` and pushed, and the cycle's bookmark deleted.
#[derive(Args, Debug)]
pub struct LandArgs {
    /// The cycle's last commit: a bookmark, the cycle's, or a revision,
    /// whose current bookmark, else its nearest parent's above `--onto`,
    /// is the cycle's
    #[arg(long, value_name = "BM|REV", required = true)]
    pub source: String,

    /// The bookmark landed on, fast-forwarded to `--source`: a local
    /// bookmark at its `origin` position that `--source` descends from
    #[arg(long, value_name = "BOOKMARK", required = true)]
    pub onto: String,

    /// Which side of the cycle's bookmark is kept: `local`, `remote`,
    /// or `both`, none deleted
    #[arg(long, value_enum, value_name = "SIDE", hide_possible_values = true)]
    pub skip: Option<Skip>,

    /// Also delete the bookmarks on the commits strictly between `--onto`
    /// and `--source`, locally and on origin, as `--skip` allows: a stack
    /// of cycles landing together [default: named as merged and kept]
    #[arg(long)]
    pub delete_intermediate_bookmarks: bool,

    /// Where: a path in the workspace for `-s b`, or the repo itself for
    /// `-s p` [default: the current directory]
    #[arg(short = 'R', long, value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Which repos, required: `b` a dual workspace's, `p` a plain repo
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = repo_scope_parser(&["both", "por"]),
        required = true,
        help = "Which repos, required: `b|both` or `p|por`",
        long_help = "Which repos, required: `b|both` a dual workspace, or `p|por` a plain repo. \
                     `w` alone is refused, since it could publish trailers naming agent commits \
                     not yet published, and `a` alone, which has no cycle to land"
    )]
    pub scope: RepoScope,

    /// Make every check and say each step, and change nothing
    #[command(flatten)]
    pub dry_run: DryRunFlag,
}

/// Inputs to the land op, flat, owned, clap-free.
#[derive(Debug)]
pub struct LandParams {
    /// The work repo of `-s b`, or the plain repo of `-s p`.
    pub repo: PathBuf,
    pub scope: RepoScope,
    pub source: String,
    pub onto: String,
    pub skip: Option<Skip>,
    pub delete_intermediate: bool,
    pub dry_run: bool,
}

impl TryFrom<&LandArgs> for LandParams {
    type Error = String;

    /// Resolve the scope to the repo the Land moves bookmarks in.
    fn try_from(a: &LandArgs) -> Result<Self, String> {
        let repo = match &a.scope {
            RepoScope::Por => {
                let repos = crate::common::resolve_scoped_repos(&a.scope, a.repo.as_deref())
                    .map_err(|e| e.to_string())?;
                match repos.as_slice() {
                    [repo] => repo.clone(),
                    other => {
                        return Err(format!("land -s p: expected one repo, got {}", other.len()));
                    }
                }
            }
            RepoScope::Sides(sides) if sides.is_both() => {
                crate::pair::resolve_pair(a.repo.as_deref())
                    .map_err(|e| e.to_string())?
                    .work
            }
            RepoScope::Sides(_) => {
                return Err(
                    "land: a work-only Land would publish trailers naming agent commits its \
                     agent half may not have published, and the agent repo alone has no cycle to \
                     land, so the scope is `-s b`, or `-s p` for a plain repo"
                        .into(),
                );
            }
        };
        Ok(LandParams {
            repo,
            scope: a.scope.clone(),
            source: a.source.clone(),
            onto: a.onto.clone(),
            skip: a.skip,
            delete_intermediate: a.delete_intermediate_bookmarks,
            dry_run: a.dry_run.value,
        })
    }
}

impl SubcommandRunner for LandArgs {
    type Params = LandParams;

    /// Delegate to the `TryFrom<&LandArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        LandParams::try_from(self)
    }

    /// Run the `land` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        land(ctx, params)
    }
}

/// Entry point: resolve `--source` and the cycle's bookmark, plan the
/// Land, refusing before anything moves, and run it.
pub fn land(ctx: &mut Context, params: &LandParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("land: enter {params:?}");
    let repo = params.repo.as_path();
    let tip = resolve_tip(repo, "land", &params.source)?;
    if !jj::local_bookmark_exists(repo, &params.onto)? {
        return Err(format!(
            "land: '{}' is not a local bookmark, and Land moves one",
            params.onto
        )
        .into());
    }
    if jj::cid_of(repo, &params.onto)? == tip {
        return Err(format!(
            "land: '{}' is at '{}' already, so there is nothing to land",
            params.onto, params.source
        )
        .into());
    }
    let bookmark = cycle_bookmark(repo, "land", "--source", &params.source, &params.onto, &tip)?;
    if bookmark.is_none() {
        info!(
            "land: no bookmark on the branch above '{}', so none is deleted",
            params.onto
        );
    }
    let plan = plan(
        repo,
        "land",
        params.onto.clone(),
        &tip,
        bookmark.as_deref(),
        params.skip,
        params.delete_intermediate,
    )?;
    run(
        ctx,
        repo,
        params.scope.clone(),
        &plan,
        &tip,
        &format!("'{}'", params.source),
        params.dry_run,
    )
}

/// The one commit `arg` names, the cycle's tip, refused when it names
/// none or several.
pub(crate) fn resolve_tip(
    repo: &Path,
    label: &str,
    arg: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let cids = jj::cids_of(repo, arg).map_err(|e| {
        format!(
            "{label}: '{arg}' does not resolve in {}: {e}",
            repo.display()
        )
    })?;
    match cids.as_slice() {
        [one] => Ok(one.clone()),
        other => Err(format!(
            "{label}: '{arg}' names {} commits in {}, and a cycle's tip is exactly one",
            other.len(),
            repo.display()
        )
        .into()),
    }
}

/// The cycle's bookmark, the one a Land deletes, decided by `arg`, the
/// argument of `flag` that names the tip: a bookmark named is that one;
/// a revision means the current bookmark on the tip, else the nearest
/// parent's, above `trunk`, several at that height refused with the
/// list, since naming one settles it, and none being none. The trunk
/// itself is never the cycle's.
pub(crate) fn cycle_bookmark(
    repo: &Path,
    label: &str,
    flag: &str,
    arg: &str,
    trunk: &str,
    tip: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if arg != trunk && jj::local_bookmark_exists(repo, arg)? {
        return Ok(Some(arg.to_string()));
    }
    let base = jj::cid_of(repo, trunk)?;
    let mut found =
        jj::local_bookmarks_at(repo, &format!("heads(({base}..({tip})) & bookmarks())"))?;
    found.retain(|b| *b != trunk);
    match found.len() {
        0 => Ok(None),
        1 => Ok(found.pop()),
        _ => Err(format!(
            "{label}: '{arg}' has several bookmarks at its nearest, {}, so which is the \
             cycle's is not known: name it, {flag} <BOOKMARK>",
            found.join(", ")
        )
        .into()),
    }
}

/// A Land planned before anything is made: the trunk and where it is,
/// and the cycle's own bookmark, deleted after, `skip` keeping a side.
#[derive(Debug)]
pub(crate) struct LandPlan {
    /// How messages name the command, e.g. `commit --land`.
    pub label: &'static str,
    pub trunk: String,
    pub trunk_cid: String,
    pub topic: Option<String>,
    pub skip: Option<Skip>,
    /// The bookmarks strictly between the trunk and the tip, deleted
    /// after the cycle's when asked, `skip` keeping the same side.
    pub intermediate: Vec<String>,
    /// The branch's other bookmarks, merged by the Land and left as
    /// they are, named after it.
    pub merged: Vec<String>,
}

/// Plan a Land of the branch read up to `tip` onto `trunk`, refusing
/// before anything is made. `bookmark` is the cycle's, deleted after,
/// and with `delete_intermediate` the bookmarks strictly between too.
pub(crate) fn plan(
    repo: &Path,
    label: &'static str,
    trunk: String,
    tip: &str,
    bookmark: Option<&str>,
    skip: Option<Skip>,
    delete_intermediate: bool,
) -> Result<LandPlan, Box<dyn std::error::Error>> {
    if !jj::local_bookmark_exists(repo, &trunk)? {
        return Err(
            format!("{label}: '{trunk}' is not a local bookmark, and Land moves one").into(),
        );
    }
    let trunk_cid = jj::cid_of(repo, &trunk)?;
    let remote = jj::remote_bookmark_targets(repo, "origin")?
        .get(&trunk)
        .cloned();
    if remote.as_deref() != Some(trunk_cid.as_str()) {
        return Err(format!(
            "{label}: '{trunk}' is at {trunk_cid} locally and {} on origin: sync first, \
             since a trunk that moved stops a Land",
            remote.as_deref().unwrap_or("absent")
        )
        .into());
    }
    if !jj::matches(repo, &format!("{trunk_cid} & ::({tip})"))? {
        return Err(format!(
            "{label}: the branch does not descend from '{trunk}' at {trunk_cid}, so the \
             trunk moved since the fork and the Land would carry a tree no validation saw: rebase \
             the branch onto it first"
        )
        .into());
    }
    let topic = bookmark.map(str::to_string);
    let others = |revset: &str| -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut found = jj::local_bookmarks_at(repo, revset)?;
        found.retain(|b| *b != trunk && Some(b) != topic.as_ref());
        Ok(found)
    };
    let between = others(&format!("(({trunk_cid}..({tip})) ~ ({tip})) & bookmarks()"))?;
    let mut merged = others(&format!("({tip}) & bookmarks()"))?;
    let intermediate = if delete_intermediate {
        between
    } else {
        merged.splice(0..0, between);
        Vec::new()
    };
    info!(
        "{label}: land onto '{trunk}' at {trunk_cid}, the cycle's bookmark {}",
        topic
            .as_deref()
            .map(|t| format!("'{t}'"))
            .unwrap_or_else(|| "none".into())
    );
    Ok(LandPlan {
        label,
        trunk,
        trunk_cid,
        topic,
        skip,
        intermediate,
        merged,
    })
}

/// Land what `land` plans onto `target`, named `what` in messages:
/// refuse a conflicted target, fast-forward the trunk to it, publish
/// it, `scope` saying which repos, and delete the cycle's bookmark,
/// `--skip` keeping a side. A dry run makes every check and says each
/// step, and does none.
pub(crate) fn run(
    ctx: &mut Context,
    repo: &Path,
    scope: RepoScope,
    land: &LandPlan,
    target: &str,
    what: &str,
    dry: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let LandPlan {
        label,
        trunk,
        trunk_cid,
        topic,
        skip,
        intermediate,
        merged,
    } = land;
    let keep = match skip {
        Some(Skip::Both) => None,
        Some(Skip::Local) => Some(Some(crate::delete_bookmark::Keep::Local)),
        Some(Skip::Remote) => Some(Some(crate::delete_bookmark::Keep::Remote)),
        None => Some(None),
    };
    // Checks run in both modes, so a dry run refuses what a Land would.
    if jj::matches(repo, &format!("({target}) & conflicts()"))? {
        return Err(format!(
            "{label}: {what} is conflicted, so it is not landed: resolve it, then \
             land it with `vc-x1 move-bookmark --bookmark {trunk} --target {target}` and \
             `vc-x1 push`"
        )
        .into());
    }
    // Each step's line, then the step, which a dry run leaves undone.
    let tag = if dry { " [dry-run]" } else { "" };
    // The work @-'s pair may be unfinished, which push finishes. A
    // commit below it has a finished pair, so its trunk goes alone.
    let at_work_head = jj::matches(repo, &format!("({target}) & @-"))?;
    info!("{label}:{tag} fast-forward '{trunk}' from {trunk_cid} to {what}");
    if !dry {
        ctx.session(repo)?.bookmark_set(trunk, target)?;
    }
    if at_work_head {
        info!("{label}:{tag} push '{trunk}', the work @-, with its pair");
        if !dry {
            crate::push::push(
                ctx,
                &crate::push::PushParams {
                    bookmark: Some(trunk.clone()),
                    repo: Some(repo.to_path_buf()),
                    scope,
                    step: false,
                    dry_run: false,
                    yes: true,
                },
            )?;
        }
    } else {
        info!("{label}:{tag} push '{trunk}'");
        if !dry {
            crate::common::verify_tracking(repo, trunk)?;
            ctx.session(repo)?.git_push_bookmark(trunk)?;
        }
        if scope != RepoScope::Por {
            let agent = crate::pair::resolve_pair(Some(repo))?.agent;
            info!("{label}:{tag} publish the agent main");
            if !dry {
                crate::squash_push::squash_push(
                    ctx,
                    &crate::squash_push::SquashPushParams {
                        repo: agent,
                        squash: SquashSpec {
                            source: "@".to_string(),
                            target: "@-".to_string(),
                        },
                        bookmark: "main".to_string(),
                        at_rest: false,
                        yes: true,
                        dry_run: false,
                    },
                )?;
            }
        }
    }
    if let (Some(t), Some(keep)) = (topic, keep) {
        info!("{label}:{tag} delete '{t}'");
        if !dry {
            crate::delete_bookmark::delete_bookmark(ctx, repo, t, keep, false)?;
        }
    }
    if let Some(keep) = keep {
        for b in intermediate {
            info!("{label}:{tag} delete '{b}', between '{trunk}' and {what}");
            if !dry {
                crate::delete_bookmark::delete_bookmark(ctx, repo, b, keep, false)?;
            }
        }
    }
    if !merged.is_empty() {
        info!(
            "{label}:{tag} merged into '{trunk}' and left as they are, yours to keep or \
             delete: {}",
            merged.join(", ")
        );
    }
    if !dry {
        info!(
            "{label}: landed on '{trunk}', the undo being a revert, the trunk having moved \
             from {trunk_cid}. `vc-x1 install` installs the artifact"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commit::{CommitParams, commit};
    use crate::options_flags::scope::{Scope, Side};
    use crate::test_helpers::{Fixture, chid, jj_ok, test_ctx};
    use crate::{Cli, Commands};
    use clap::Parser;
    use std::fs;

    fn commit_id(repo: &Path, rev: &str) -> String {
        jj_ok(repo, &["log", "-r", rev, "--no-graph", "-T", "commit_id"])
    }

    fn on_origin(repo: &Path, name: &str) -> bool {
        jj::remote_bookmark_targets(repo, "origin")
            .expect("remote")
            .contains_key(name)
    }

    /// Run `vc-x1 commit <args>` in the fixture's work repo, pushed.
    fn vc_commit(fx: &Fixture, args: &[&str]) {
        let mut argv = vec!["vc-x1", "commit"];
        argv.extend_from_slice(args);
        let Ok(Cli {
            command: Some(Commands::Commit(mut a)),
            ..
        }) = Cli::try_parse_from(&argv)
        else {
            panic!("expected Commit: {argv:?}");
        };
        a.repo = Some(fx.work.clone());
        let params = CommitParams::try_from(&a).expect("commit params");
        commit(&mut test_ctx(), &params).expect("the commit is made");
    }

    /// `vc-x1 land <args>` in the fixture's work repo, parsed.
    fn land_params(fx: &Fixture, args: &[&str]) -> LandParams {
        let mut argv = vec!["vc-x1", "land"];
        argv.extend_from_slice(args);
        let Ok(Cli {
            command: Some(Commands::Land(mut a)),
            ..
        }) = Cli::try_parse_from(&argv)
        else {
            panic!("expected Land: {argv:?}");
        };
        a.repo = Some(fx.work.clone());
        LandParams::try_from(&a).expect("land params")
    }

    /// A cycle pushed on `feat-a-cycle`: its opening and one rung.
    /// Returns the rung's commit id.
    fn pushed_cycle(fx: &Fixture) -> String {
        fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
        vc_commit(
            fx,
            &[
                "-s",
                "b",
                "--opening",
                "--title",
                "feat: a cycle",
                "--body",
                "Why.",
            ],
        );
        fs::write(fx.work.join("rung.txt"), "rung\n").expect("write");
        vc_commit(
            fx,
            &["-s", "b", "--title", "feat: a rung", "--body", "The rung."],
        );
        commit_id(&fx.work, "feat-a-cycle")
    }

    /// The cycle at the work `@-` lands: `main` at its tip locally and on
    /// origin in both repos, and its bookmark gone from both.
    #[test]
    fn a_cycle_at_the_work_head_lands() {
        let fx = Fixture::new("land-head");
        let tip = pushed_cycle(&fx);
        let p = land_params(
            &fx,
            &["-s", "b", "--source", "feat-a-cycle", "--onto", "main"],
        );
        land(&mut test_ctx(), &p).expect("landed");
        assert_eq!(commit_id(&fx.work, "main"), tip);
        assert_eq!(commit_id(&fx.work, "main@origin"), tip);
        assert!(!jj::local_bookmark_exists(&fx.work, "feat-a-cycle").expect("local"));
        assert!(!on_origin(&fx.work, "feat-a-cycle"), "deleted on origin");
        assert_eq!(
            commit_id(&fx.bot, "main@origin"),
            commit_id(&fx.bot, "main"),
            "the agent main is published"
        );
    }

    /// A source below the work `@-` lands by its trunk alone: `main` at
    /// the source, the commit above it left where it is.
    #[test]
    fn a_source_below_the_work_head_lands() {
        let fx = Fixture::new("land-below");
        let tip = pushed_cycle(&fx);
        fs::write(fx.work.join("next.txt"), "next\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "next"]);
        let next = commit_id(&fx.work, "@-");
        let p = land_params(
            &fx,
            &["-s", "b", "--source", "feat-a-cycle", "--onto", "main"],
        );
        land(&mut test_ctx(), &p).expect("landed");
        assert_eq!(commit_id(&fx.work, "main"), tip);
        assert_eq!(commit_id(&fx.work, "main@origin"), tip);
        assert_eq!(commit_id(&fx.work, "@-"), next, "the commit above stays");
        assert!(!on_origin(&fx.work, "feat-a-cycle"), "deleted on origin");
        assert_eq!(
            commit_id(&fx.bot, "main@origin"),
            commit_id(&fx.bot, "main"),
            "the agent main is published"
        );
    }

    /// `-s p` lands a plain repo: `main` at the source locally and on
    /// origin, and the bookmark gone.
    #[test]
    fn a_plain_repo_lands() {
        let fx = crate::test_helpers::FixturePor::new("land-por");
        fs::write(fx.work.join("rung.txt"), "rung\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: a rung"]);
        jj_ok(&fx.work, &["git", "push", "--named", "feat-a-cycle=@-"]);
        let tip = commit_id(&fx.work, "feat-a-cycle");
        let argv = [
            "vc-x1",
            "land",
            "-s",
            "p",
            "--source",
            "feat-a-cycle",
            "--onto",
            "main",
        ];
        let Ok(Cli {
            command: Some(Commands::Land(mut a)),
            ..
        }) = Cli::try_parse_from(argv)
        else {
            panic!("expected Land");
        };
        a.repo = Some(fx.work.clone());
        let p = LandParams::try_from(&a).expect("land params");
        land(&mut test_ctx(), &p).expect("landed");
        assert_eq!(commit_id(&fx.work, "main"), tip);
        assert_eq!(commit_id(&fx.work, "main@origin"), tip);
        assert!(!on_origin(&fx.work, "feat-a-cycle"), "deleted on origin");
    }

    /// `-s w` and `-s a` are refused before anything is read: a dual
    /// workspace lands both repos together.
    #[test]
    fn one_side_alone_is_refused() {
        let fx = Fixture::new("land-one-side");
        for scope in ["w", "a"] {
            let argv = [
                "vc-x1", "land", "-s", scope, "--source", "main", "--onto", "main",
            ];
            let Ok(Cli {
                command: Some(Commands::Land(mut a)),
                ..
            }) = Cli::try_parse_from(argv)
            else {
                panic!("expected Land");
            };
            a.repo = Some(fx.work.clone());
            let err = LandParams::try_from(&a).expect_err("refused");
            assert!(err.contains("the scope is `-s b`"), "{scope}: {err}");
        }
    }

    /// A stack: `earlier` on the opening, between `main` and the tip,
    /// and `mine` beside `feat-a-cycle` at the tip, both published.
    fn stacked_cycle(fx: &Fixture) -> String {
        let tip = pushed_cycle(fx);
        jj_ok(
            &fx.work,
            &["git", "push", "--named", "earlier=feat-a-cycle-"],
        );
        jj_ok(&fx.work, &["git", "push", "--named", "mine=feat-a-cycle"]);
        tip
    }

    /// `--delete-intermediate-bookmarks` deletes the bookmarks strictly
    /// between, locally and on origin, its dry run naming them first,
    /// and keeps one beside the cycle's at the tip.
    #[test]
    fn the_intermediate_bookmarks_are_deleted_when_asked() {
        let fx = Fixture::new("land-intermediate");
        let tip = stacked_cycle(&fx);
        let args = [
            "-s",
            "b",
            "--source",
            "feat-a-cycle",
            "--onto",
            "main",
            "--delete-intermediate-bookmarks",
        ];
        let mut dry = args.to_vec();
        dry.push("--dry-run");
        land(&mut test_ctx(), &land_params(&fx, &dry)).expect("planned");
        assert!(on_origin(&fx.work, "earlier"), "a dry run deletes nothing");

        land(&mut test_ctx(), &land_params(&fx, &args)).expect("landed");
        assert_eq!(commit_id(&fx.work, "main@origin"), tip);
        for gone in ["feat-a-cycle", "earlier"] {
            assert!(
                !jj::local_bookmark_exists(&fx.work, gone).expect("local"),
                "{gone}"
            );
            assert!(!on_origin(&fx.work, gone), "{gone} deleted on origin");
        }
        assert_eq!(commit_id(&fx.work, "mine"), tip, "the one at the tip stays");
        assert!(on_origin(&fx.work, "mine"));
    }

    /// Without the flag the bookmarks between are named as merged and
    /// kept, and `--skip remote` keeps the remote side of each deleted.
    #[test]
    fn the_intermediate_bookmarks_are_kept_unless_asked() {
        let fx = Fixture::new("land-intermediate-kept");
        stacked_cycle(&fx);
        let p = land_params(
            &fx,
            &["-s", "b", "--source", "feat-a-cycle", "--onto", "main"],
        );
        land(&mut test_ctx(), &p).expect("landed");
        assert!(jj::local_bookmark_exists(&fx.work, "earlier").expect("local"));
        assert!(on_origin(&fx.work, "earlier"));

        let fx = Fixture::new("land-intermediate-skip");
        stacked_cycle(&fx);
        let p = land_params(
            &fx,
            &[
                "-s",
                "b",
                "--source",
                "feat-a-cycle",
                "--onto",
                "main",
                "--delete-intermediate-bookmarks",
                "--skip",
                "remote",
            ],
        );
        land(&mut test_ctx(), &p).expect("landed");
        assert!(!jj::local_bookmark_exists(&fx.work, "earlier").expect("local"));
        assert!(
            on_origin(&fx.work, "earlier"),
            "--skip remote keeps origin's"
        );
    }

    /// A revision takes its nearest bookmark, and a dry run makes the
    /// checks and moves nothing.
    #[test]
    fn a_dry_run_moves_nothing() {
        let fx = Fixture::new("land-dry");
        pushed_cycle(&fx);
        let main = commit_id(&fx.work, "main");
        let p = land_params(
            &fx,
            &["-s", "b", "--source", "@-", "--onto", "main", "--dry-run"],
        );
        land(&mut test_ctx(), &p).expect("planned");
        assert_eq!(commit_id(&fx.work, "main"), main);
        assert!(jj::local_bookmark_exists(&fx.work, "feat-a-cycle").expect("local"));
        assert!(on_origin(&fx.work, "feat-a-cycle"));
    }

    /// Every refusal comes before anything moves: an `--onto` that is
    /// not an ancestor, one already at the source, and one that moved
    /// from origin.
    #[test]
    fn a_land_is_refused_before_anything_moves() {
        let fx = Fixture::new("land-refused");
        pushed_cycle(&fx);
        let main = commit_id(&fx.work, "main");
        let refused = |args: &[&str], want: &str| {
            let p = land_params(&fx, args);
            let err = land(&mut test_ctx(), &p).expect_err("refused").to_string();
            assert!(err.contains(want), "{err}");
            assert_eq!(commit_id(&fx.work, "main"), main, "main stays");
            assert!(on_origin(&fx.work, "feat-a-cycle"), "the bookmark stays");
        };
        refused(
            &["-s", "b", "--source", "main", "--onto", "main"],
            "nothing to land",
        );
        refused(
            &["-s", "b", "--source", "feat-a-cycle", "--onto", "no-such"],
            "is not a local bookmark",
        );
        jj_ok(&fx.work, &["git", "push", "--named", "side=feat-a-cycle"]);
        refused(
            &["-s", "b", "--source", "main", "--onto", "side"],
            "does not descend",
        );
        jj_ok(
            &fx.work,
            &[
                "bookmark",
                "set",
                "main",
                "-r",
                "feat-a-cycle-",
                "--allow-backwards",
            ],
        );
        let moved = commit_id(&fx.work, "main");
        let p = land_params(
            &fx,
            &["-s", "b", "--source", "feat-a-cycle", "--onto", "main"],
        );
        let err = land(&mut test_ctx(), &p).expect_err("refused").to_string();
        assert!(err.contains("sync first"), "{err}");
        assert_eq!(commit_id(&fx.work, "main"), moved, "main stays");
    }

    /// A fixture whose `@-` is a conflicted merge, the plan of a Land
    /// of it onto `main`, and `main`'s commit id.
    fn conflicted(tag: &str) -> (Fixture, LandPlan, String) {
        let fx = Fixture::new(tag);
        fs::write(fx.work.join("a.txt"), "base\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "base"]);
        let base = chid(&fx.work, "@-");
        fs::write(fx.work.join("a.txt"), "left\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "left"]);
        let left = chid(&fx.work, "@-");
        jj_ok(&fx.work, &["new", &base]);
        fs::write(fx.work.join("a.txt"), "right\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "right"]);
        let right = chid(&fx.work, "@-");
        jj_ok(&fx.work, &["new", &left, &right]);
        jj_ok(&fx.work, &["new"]);
        assert!(jj::matches(&fx.work, "@- & conflicts()").expect("matches"));
        let main = commit_id(&fx.work, "main");
        let plan = LandPlan {
            label: "commit --land",
            trunk: "main".to_string(),
            trunk_cid: main.clone(),
            topic: None,
            skip: None,
            intermediate: Vec::new(),
            merged: Vec::new(),
        };
        (fx, plan, main)
    }

    /// A conflicted closing is not landed: the trunk stays where it is
    /// and the error names the commands that land it after a resolve.
    /// `commit` refuses a conflicted repo up front and a closing onto a
    /// trunk the branch descends from cannot conflict, so the guard is
    /// reached here directly, on a conflicted `@-` made by hand.
    #[test]
    fn a_conflicted_closing_is_not_landed() {
        let (fx, plan, main) = conflicted("land-conflicted");
        let pair = RepoScope::Sides(Scope(vec![Side::Work, Side::Bot]));
        let err = run(
            &mut test_ctx(),
            &fx.work,
            pair,
            &plan,
            "@-",
            "the closing",
            false,
        )
        .expect_err("refused")
        .to_string();
        assert!(err.contains("the closing is conflicted"), "{err}");
        assert!(
            err.contains("vc-x1 move-bookmark --bookmark main --target @-"),
            "{err}"
        );
        assert_eq!(commit_id(&fx.work, "main"), main, "main stays");
        assert_eq!(
            commit_id(&fx.work, "main@origin"),
            main,
            "nothing was pushed"
        );
    }

    /// A dry run makes the checks a Land makes, so it refuses a
    /// conflicted closing rather than report a Land that would fail.
    #[test]
    fn a_dry_run_refuses_a_conflicted_closing() {
        let (fx, plan, main) = conflicted("land-conflicted-dry");
        let pair = RepoScope::Sides(Scope(vec![Side::Work, Side::Bot]));
        let err = run(
            &mut test_ctx(),
            &fx.work,
            pair,
            &plan,
            "@-",
            "the closing",
            true,
        )
        .expect_err("refused")
        .to_string();
        assert!(err.contains("the closing is conflicted"), "{err}");
        assert_eq!(commit_id(&fx.work, "main"), main, "main stays");
    }
}

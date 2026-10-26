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
//! `commit --closing ... --land` plans its Land before the closing is
//! made and runs it after, through these two.

use std::path::Path;

use log::info;

use crate::commit::Skip;
use crate::context::Context;
use crate::jj;
use crate::options_flags::scope::RepoScope;

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
    /// The branch's other bookmarks, merged by the Land and left as
    /// they are, named after it.
    pub merged: Vec<String>,
}

/// Plan a Land of the branch read up to `tip` onto `trunk`, refusing
/// before anything is made. `bookmark` is the cycle's, deleted after.
pub(crate) fn plan(
    repo: &Path,
    label: &'static str,
    trunk: String,
    tip: &str,
    bookmark: Option<&str>,
    skip: Option<Skip>,
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
    let mut merged =
        jj::local_bookmarks_at(repo, &format!("({trunk_cid}..({tip})) & bookmarks()"))?;
    merged.retain(|b| *b != trunk && Some(b) != topic.as_ref());
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
        merged,
    })
}

/// Land what `land` plans onto `target`, the closing: refuse a
/// conflicted target, fast-forward the trunk to it, push the trunk with
/// the agent half as its partner, `scope` saying which push, and delete
/// the cycle's bookmark, `--skip` keeping a side. A dry run makes every
/// check and says each step, and does none.
pub(crate) fn run(
    ctx: &mut Context,
    repo: &Path,
    scope: RepoScope,
    land: &LandPlan,
    target: &str,
    dry: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let LandPlan {
        label,
        trunk,
        trunk_cid,
        topic,
        skip,
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
            "{label}: the closing is conflicted, so it is not landed: resolve it, then \
             land it with `vc-x1 move-bookmark --bookmark {trunk} --target {target}` and \
             `vc-x1 push`"
        )
        .into());
    }
    // Each step's line, then the step, which a dry run leaves undone.
    let tag = if dry { " [dry-run]" } else { "" };
    info!("{label}:{tag} fast-forward '{trunk}' from {trunk_cid} to the closing");
    if !dry {
        ctx.session(repo)?.bookmark_set(trunk, target)?;
    }
    info!("{label}:{tag} push '{trunk}'");
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
    if let (Some(t), Some(keep)) = (topic, keep) {
        info!("{label}:{tag} delete '{t}'");
        if !dry {
            crate::delete_bookmark::delete_bookmark(ctx, repo, t, keep, false)?;
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
    use crate::options_flags::scope::{Scope, Side};
    use crate::test_helpers::{Fixture, chid, jj_ok, test_ctx};
    use std::fs;

    fn commit_id(repo: &Path, rev: &str) -> String {
        jj_ok(repo, &["log", "-r", rev, "--no-graph", "-T", "commit_id"])
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
        let err = run(&mut test_ctx(), &fx.work, pair, &plan, "@-", false)
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
        let err = run(&mut test_ctx(), &fx.work, pair, &plan, "@-", true)
            .expect_err("refused")
            .to_string();
        assert!(err.contains("the closing is conflicted"), "{err}");
        assert_eq!(commit_id(&fx.work, "main"), main, "main stays");
    }
}

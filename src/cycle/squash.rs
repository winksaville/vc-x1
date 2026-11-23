//! `cycle close --squash`: a cycle made one commit under its bare title.
//!
//! - With no commit on the branch, or the opening alone, it is the
//!   single-step commit [`close`] makes, `@` committed now or folded
//!   into the opening.
//! - With work rungs, the agent side is re-pointed first: each agent
//!   commit naming a rung is rewritten to name the opening, whose change
//!   id the squash keeps, so only descriptions change there and the
//!   agent-repo keeps its per-rung history. Then the opening is given
//!   every agent commit's `ochid:`, each rung is squashed into it, and
//!   the single-step fold finishes the cycle.
//! - A cycle closed and not landed is converted: its closing, a
//!   trapezoid's merge or a linear closing, is squashed in with the
//!   rungs, and the record `## Closed` holds is made a single-step's,
//!   its `#### Rungs` removed and the ladder one rung. A trapezoid whose
//!   target moved since is refused, the one commit then a tree no rung
//!   validated.
//! - Every check is made before anything is rewritten, and a failure
//!   restores both repos.

use std::path::{Path, PathBuf};

use log::info;

use super::{
    CloseCycle, PushCycle, SquashCycle, branch_revset, close, closing_body, cycle_bookmark,
    land_checks, land_closed, push, record,
};
use crate::context::Context;
use crate::desc_helpers::{Trailers, extract_ochids, with_trailers};
use crate::jj;

/// `cycle close --squash`: make the cycle one commit.
pub(super) fn squash(ctx: &mut Context, s: &SquashCycle) -> Result<(), Box<dyn std::error::Error>> {
    let label = "cycle close --squash";
    let c = &s.one;
    let r = &c.rung.running;
    let work = r.work.as_path();
    let path = work.join(crate::commit::RECORD);
    let todo =
        std::fs::read_to_string(&path).map_err(|e| format!("{label}: {}: {e}", path.display()))?;
    if record::cycle_title(&todo).is_err()
        && let Some(cycle) = record::closed_title(&todo)
    {
        return convert(ctx, s, &cycle, &todo);
    }
    let bookmark = cycle_bookmark(r, label)?;
    let branch = branch_revset(&bookmark);
    let cids = jj::cids_of(work, &branch)?;
    if cids.len() <= 1 {
        return close(ctx, c, label);
    }
    let cycle = record::cycle_title(&todo).map_err(|e| format!("{label}: {e}"))?;
    refuse_not_opening(work, &cids, &cycle, label)?;
    if !jj::local_bookmarks_at(work, "@ | @-")?.contains(&bookmark) {
        return Err(format!(
            "{label}: the cycle's bookmark '{bookmark}' is not on the work @ or @-, and the squash \
             is made from the cycle's tip: `jj new {bookmark}` moves @ onto it"
        )
        .into());
    }
    refuse_published(work, &branch, &bookmark, label)?;
    let commits: Vec<String> = cids
        .iter()
        .map(|cid| jj::desc_of(work, cid).map(|d| d.lines().next().unwrap_or_default().to_string()))
        .collect::<Result<_, _>>()?;
    if let Some(current) = record::current_rung(work).map_err(|e| format!("{label}: {e}"))?
        && !commits.contains(&current)
        && current != format!("{cycle} closing")
    {
        return Err(format!(
            "{label}: the rung '{current}' is started and not committed: `rung commit` commits \
             it, or `rung stop` leaves it, its edits folded in by the squash"
        )
        .into());
    }
    crate::pair::refuse_conflicts(work, label, "work")?;
    crate::commit::check_version(work, "@", c.rung.same_version, label)?;
    if let Some(l) = &c.land {
        land_checks(work, &l.dest, Some("@-"), l.agent.is_none())?;
    }
    let plan = plan_fold(work, c, &cids, s.yes, label)?;
    plan.say(label, r.dry_run);
    if r.dry_run {
        info!("{label}: [dry-run] then fold the work @ into it as '{cycle}', the record closed");
        return Ok(());
    }
    let one = CloseCycle {
        land: None,
        rung: super::CommitRung {
            push: false,
            ..c.rung.clone()
        },
        ..c.clone()
    };
    plan.make(ctx, work, label, |ctx| close(ctx, &one, label))?;
    after(ctx, c, bookmark, plan.agent)
}

/// Convert the cycle `## Closed` holds, closed and not landed, to one
/// commit: its closing squashed in with the rungs, then the record made
/// a single-step's and the commit finalized under the bare title.
fn convert(
    ctx: &mut Context,
    s: &SquashCycle,
    cycle: &str,
    todo: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let label = "cycle close --squash";
    let c = &s.one;
    let r = &c.rung.running;
    let work = r.work.as_path();
    let bookmark = match &r.bookmark {
        Some(b) => b.clone(),
        None => crate::check::anchors::anchor_slug(cycle),
    };
    if !jj::local_bookmark_exists(work, &bookmark)? {
        return Err(format!(
            "{label}: the cycle '{cycle}' `## Closed` holds has no bookmark '{bookmark}' here, \
             so it is landed or gone, and a landed cycle is never rewritten: `cycle plan start` \
             opens the next one"
        )
        .into());
    }
    if !jj::local_bookmarks_at(work, "@-")?.contains(&bookmark) || !jj::is_empty(work, "@")? {
        return Err(format!(
            "{label}: the closed cycle '{cycle}' is converted from an empty work @ on its \
             closing, which its bookmark '{bookmark}' is on: `jj new {bookmark}`"
        )
        .into());
    }
    let closing = format!("{cycle} closing");
    let title = jj::desc_of(work, "@-")?
        .lines()
        .next()
        .unwrap_or_default() // OK: a commit with no description has no title
        .trim_end()
        .to_string();
    if title != closing {
        return Err(format!(
            "{label}: the commit '{bookmark}' is on is '{title}', not the closing '{closing}', \
             and only a closing is converted"
        )
        .into());
    }
    let own = format!("bookmarks(exact:\"{bookmark}\")");
    if jj::matches(work, &format!("@- & ::(bookmarks() ~ {own})"))? {
        return Err(format!(
            "{label}: the closing of '{cycle}' is reached by another bookmark, landed here, and a \
             landed cycle is never rewritten"
        )
        .into());
    }
    let branch = branch_revset(&bookmark);
    let cids = jj::cids_of(work, &branch)?;
    refuse_not_opening(work, &cids, cycle, label)?;
    refuse_published(work, &branch, &bookmark, label)?;
    let parents = jj::parent_cids_of(work, "@-")?;
    if parents.len() > 2 {
        return Err(format!(
            "{label}: the closing of '{cycle}' joins {} heads, and only a trapezoid's or a linear \
             closing is converted",
            parents.len()
        )
        .into());
    }
    if parents.len() == 2 && !jj::matches(work, &format!("{} & ::({}-)", parents[0], cids[0]))? {
        return Err(format!(
            "{label}: the trapezoid's target, its first parent {}, moved after the cycle opened, \
             and the one commit would be a tree no rung validated: land it as it is, or `jj \
             rebase -b {bookmark} -d <target>` and close again",
            super::short(&parents[0])
        )
        .into());
    }
    crate::pair::refuse_conflicts(work, label, "work")?;
    if let Some(l) = &c.land {
        land_checks(work, &l.dest, Some(cids[0].as_str()), l.agent.is_none())?;
    }
    let body = match (&c.rung.body, c.default_body) {
        (Some(b), _) => b.clone(),
        (None, true) => closing_body(work, cycle)?,
        (None, false) => {
            return Err(format!(
                "{label}: the one commit holds the work, so it gives --body, its problem and \
                 solution, or --default-body for the pointer at the record"
            )
            .into());
        }
    };
    let new_todo = record::without_closed_rungs(todo);
    let ladder_path = work.join(record::LADDER_FILE);
    let old_ladder = std::fs::read_to_string(&ladder_path)
        .map_err(|e| format!("{label}: {}: {e}", ladder_path.display()))?;
    let new_ladder = record::written_ladder(&old_ladder, &[format!("- 0 (done) {cycle}")])
        .map_err(|e| format!("{label}: {e}"))?;
    let plan = plan_fold(work, c, &cids, s.yes, label)?;
    info!(
        "{label}: the closed cycle '{cycle}' converted to one commit, its closing among the rungs"
    );
    plan.say(label, r.dry_run);
    if r.dry_run {
        info!(
            "{label}: [dry-run] then the record made a single-step's, without `#### Rungs`, the \
             ladder one rung, and the opening finalized as '{cycle}'"
        );
        return Ok(());
    }
    let opening = plan.opening.clone();
    let pair = c.rung.agent.as_ref().map(|(agent, _)| crate::pair::Pair {
        work: r.work.clone(),
        agent: agent.clone(),
    });
    plan.make(ctx, work, label, |ctx| {
        // The closing's children, `@` among them, sit on its parents once
        // it is squashed in, so `@` is made again on the one commit.
        ctx.session(work)?.new_on(&opening)?;
        ctx.session(work)?.bookmark_set(&bookmark, "@-")?;
        std::fs::write(work.join(crate::commit::RECORD), &new_todo)?;
        std::fs::write(&ladder_path, &new_ladder)?;
        info!("{label}: the record is a single-step's, and the ladder one rung");
        crate::commit::finalize(ctx, work, pair.as_ref(), cycle, &body, false)
    })?;
    after(ctx, c, bookmark, plan.agent)
}

/// Refuse a branch whose first commit is not the cycle's opening.
fn refuse_not_opening(
    work: &Path,
    cids: &[String],
    cycle: &str,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let opening = format!("{cycle} opening");
    let first = match cids.first() {
        Some(cid) => jj::desc_of(work, cid)?,
        None => String::new(),
    };
    if first.lines().next().map(str::trim_end) != Some(opening.as_str()) {
        return Err(format!(
            "{label}: the branch's first commit is not the cycle's opening, '{opening}', so there \
             is no commit to squash the rungs into"
        )
        .into());
    }
    Ok(())
}

/// Refuse a branch a remote bookmark other than the cycle's reaches, its
/// history published.
fn refuse_published(
    work: &Path,
    branch: &str,
    bookmark: &str,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let published =
        format!("({branch}) & ::(remote_bookmarks() ~ remote_bookmarks(exact:\"{bookmark}\"))");
    if jj::matches(work, &published)? {
        return Err(format!(
            "{label}: a commit of the branch is reached by a remote bookmark other than \
             '{bookmark}', its history published, and is never rewritten: `cycle close <TARGET>` \
             makes the ladder's closing"
        )
        .into());
    }
    Ok(())
}

/// The fold planned: the opening every commit after it is squashed into,
/// with its new description, and the agent commits re-pointed to it.
struct Fold {
    opening: String,
    opening_desc: String,
    rungs: Vec<String>,
    repoints: Vec<(String, String)>,
    agent: Option<PathBuf>,
}

/// Plan the fold of the commits `cids`, oldest first, into the first,
/// the agent commits they name re-pointed to it, refusing an agent
/// commit `main@origin` reaches without `yes`.
fn plan_fold(
    work: &Path,
    c: &CloseCycle,
    cids: &[String],
    yes: bool,
    label: &str,
) -> Result<Fold, Box<dyn std::error::Error>> {
    let chids = cids
        .iter()
        .map(|cid| jj::chid_of(work, cid))
        .collect::<Result<Vec<_>, _>>()?;
    // The agent commits the cycle's work commits name, in order, once.
    let mut named: Vec<String> = Vec::new();
    for cid in cids {
        for id in extract_ochids(&jj::desc_of(work, cid)?) {
            if !named.contains(&id) {
                named.push(id);
            }
        }
    }
    let opening = chids[0].clone();
    let rungs = chids[1..].to_vec();
    let is_rung = |id: &str| rungs.iter().any(|r| r.starts_with(id) || id.starts_with(r));
    let agent = c.rung.agent.as_ref().map(|(a, _)| a.clone());
    let mut repoints: Vec<(String, String)> = Vec::new();
    if let Some(agent) = &agent {
        for id in &named {
            let Some(achid) = jj::one_chid(agent, id)? else {
                return Err(format!(
                    "{label}: the agent commit '{id}' a work commit names is not in {}",
                    agent.display()
                )
                .into());
            };
            let desc = jj::desc_of(agent, &achid)?;
            // The agent @, its trailers the push's to write.
            if desc.trim().is_empty() {
                continue;
            }
            let t = Trailers::of(&desc);
            let mut ochids: Vec<String> = Vec::new();
            for o in &t.ochids {
                let o = if is_rung(o) {
                    opening.clone()
                } else {
                    o.clone()
                };
                if !ochids.contains(&o) {
                    ochids.push(o);
                }
            }
            if ochids == t.ochids {
                continue;
            }
            if !yes && jj::matches(agent, &format!("({achid}) & ::main@origin"))? {
                return Err(format!(
                    "{label}: the agent commit {achid} is on the agent repo's main@origin, and \
                     its rewrite makes the next agent push a forced one: --yes rewrites it"
                )
                .into());
            }
            repoints.push((achid, with_trailers(&desc, &Trailers { ochids, ..t })));
        }
    }
    let opening_desc = jj::desc_of(work, &opening)?;
    let ot = Trailers::of(&opening_desc);
    let opening_desc = match &agent {
        Some(_) => with_trailers(
            &opening_desc,
            &Trailers {
                ochids: named,
                ..ot
            },
        ),
        None => opening_desc,
    };
    Ok(Fold {
        opening,
        opening_desc,
        rungs,
        repoints,
        agent,
    })
}

impl Fold {
    /// Say what the fold does, each step under a dry run.
    fn say(&self, label: &str, dry_run: bool) {
        info!(
            "{label}: {} commit{} squashed into the opening {}, {} agent commit{} re-pointed to it",
            self.rungs.len(),
            if self.rungs.len() == 1 { "" } else { "s" },
            self.opening,
            self.repoints.len(),
            if self.repoints.len() == 1 { "" } else { "s" },
        );
        if dry_run {
            for (achid, _) in &self.repoints {
                info!(
                    "{label}: [dry-run] re-point the agent commit {achid} to {}",
                    self.opening
                );
            }
            for rung in &self.rungs {
                info!(
                    "{label}: [dry-run] squash {rung} into the opening {}",
                    self.opening
                );
            }
        }
    }

    /// Make the fold, the agent side first, then `finish`, restoring both
    /// repos on a failure.
    fn make(
        &self,
        ctx: &mut Context,
        work: &Path,
        label: &str,
        finish: impl FnOnce(&mut Context) -> Result<(), Box<dyn std::error::Error>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let agent_snapshot = match &self.agent {
            Some(a) => Some(crate::pair::Snapshot::take(a)?),
            None => None,
        };
        let work_snapshot = crate::pair::Snapshot::take(work)?;
        let made = (|| -> Result<(), Box<dyn std::error::Error>> {
            if let Some(agent) = &self.agent {
                for (achid, desc) in &self.repoints {
                    info!(
                        "{label}: re-point the agent commit {achid} to {}",
                        self.opening
                    );
                    ctx.session(agent)?.describe(achid, desc)?;
                }
            }
            ctx.session(work)?
                .describe(&self.opening, &self.opening_desc)?;
            for rung in &self.rungs {
                info!("{label}: squash {rung} into the opening {}", self.opening);
                ctx.session(work)?.squash_into(rung, &self.opening)?;
            }
            finish(ctx)
        })();
        if let Err(e) = &made {
            work_snapshot.restore(label, e.as_ref());
            if let Some(snapshot) = &agent_snapshot {
                snapshot.restore(label, e.as_ref());
            }
        }
        made
    }
}

/// The Land or the draft push the squash chains, once it is made.
fn after(
    ctx: &mut Context,
    c: &CloseCycle,
    bookmark: String,
    agent: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    match &c.land {
        Some(l) => land_closed(ctx, l, Some(bookmark)),
        None if c.rung.push => push(
            ctx,
            &PushCycle {
                running: c.rung.running.clone(),
                agent,
            },
        ),
        None => Ok(()),
    }
}

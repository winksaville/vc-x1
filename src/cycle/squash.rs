//! `cycle squash`: a cycle made one commit under its bare title.
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
//! - Every check is made before anything is rewritten, and a failure
//!   restores both repos.

use log::info;

use super::{
    CloseCycle, PushCycle, SquashCycle, branch_revset, close, cycle_bookmark, land_checks,
    land_closed, push, record,
};
use crate::context::Context;
use crate::desc_helpers::{Trailers, extract_ochids, with_trailers};
use crate::jj;

/// `cycle squash`: make the cycle one commit.
pub(super) fn squash(ctx: &mut Context, s: &SquashCycle) -> Result<(), Box<dyn std::error::Error>> {
    let label = "cycle squash";
    let c = &s.one;
    let r = &c.rung.running;
    let work = r.work.as_path();
    let bookmark = cycle_bookmark(r, label)?;
    let branch = branch_revset(&bookmark);
    let cids = jj::cids_of(work, &branch)?;
    if cids.len() <= 1 {
        return close(ctx, c, label);
    }
    let chids = cids
        .iter()
        .map(|cid| jj::chid_of(work, cid))
        .collect::<Result<Vec<_>, _>>()?;
    let path = work.join(crate::commit::RECORD);
    let todo =
        std::fs::read_to_string(&path).map_err(|e| format!("{label}: {}: {e}", path.display()))?;
    let cycle = record::cycle_title(&todo).map_err(|e| format!("{label}: {e}"))?;
    let opening = format!("{cycle} opening");
    let first = jj::desc_of(work, &cids[0])?;
    if first.lines().next().map(str::trim_end) != Some(opening.as_str()) {
        return Err(format!(
            "{label}: the branch's first commit is not the cycle's opening, '{opening}', so there \
             is no commit to squash the rungs into"
        )
        .into());
    }
    if !jj::local_bookmarks_at(work, "@ | @-")?.contains(&bookmark) {
        return Err(format!(
            "{label}: the cycle's bookmark '{bookmark}' is not on the work @ or @-, and the squash \
             is made from the cycle's tip: `jj new {bookmark}` moves @ onto it"
        )
        .into());
    }
    let published =
        format!("({branch}) & ::(remote_bookmarks() ~ remote_bookmarks(exact:\"{bookmark}\"))");
    if jj::matches(work, &published)? {
        return Err(format!(
            "{label}: a commit of the branch is reached by a remote bookmark other than \
             '{bookmark}', its history published, and is never rewritten: `cycle close` makes \
             the ladder's closing"
        )
        .into());
    }
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

    // The agent commits the cycle's work commits name, in order, once.
    let mut named: Vec<String> = Vec::new();
    for cid in &cids {
        for id in extract_ochids(&jj::desc_of(work, cid)?) {
            if !named.contains(&id) {
                named.push(id);
            }
        }
    }
    let opening_chid = chids[0].clone();
    let rungs = &chids[1..];
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
                    opening_chid.clone()
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
            if !s.yes && jj::matches(agent, &format!("({achid}) & ::main@origin"))? {
                return Err(format!(
                    "{label}: the agent commit {achid} is on the agent repo's main@origin, and \
                     its rewrite makes the next agent push a forced one: --yes rewrites it"
                )
                .into());
            }
            repoints.push((achid, with_trailers(&desc, &Trailers { ochids, ..t })));
        }
    }
    let opening_desc = jj::desc_of(work, &opening_chid)?;
    let ot = Trailers::of(&opening_desc);
    let opening_desc = match &agent {
        Some(_) => with_trailers(
            &opening_desc,
            &Trailers {
                ochids: named.clone(),
                ..ot
            },
        ),
        None => opening_desc,
    };

    info!(
        "{label}: {} work rung{} squashed into the opening {opening_chid}, {} agent commit{} \
         re-pointed to it",
        rungs.len(),
        if rungs.len() == 1 { "" } else { "s" },
        repoints.len(),
        if repoints.len() == 1 { "" } else { "s" },
    );
    if r.dry_run {
        for (achid, _) in &repoints {
            info!("{label}: [dry-run] re-point the agent commit {achid} to {opening_chid}");
        }
        for rung in rungs {
            info!("{label}: [dry-run] squash {rung} into the opening {opening_chid}");
        }
        info!("{label}: [dry-run] then fold the work @ into it as '{cycle}', the record closed");
        return Ok(());
    }

    let agent_snapshot = match &agent {
        Some(a) => Some(crate::pair::Snapshot::take(a)?),
        None => None,
    };
    let work_snapshot = crate::pair::Snapshot::take(work)?;
    let one = CloseCycle {
        land: None,
        rung: super::CommitRung {
            push: false,
            ..c.rung.clone()
        },
        ..c.clone()
    };
    let made = (|| -> Result<(), Box<dyn std::error::Error>> {
        if let Some(agent) = &agent {
            for (achid, desc) in &repoints {
                info!("{label}: re-point the agent commit {achid} to {opening_chid}");
                ctx.session(agent)?.describe(achid, desc)?;
            }
        }
        ctx.session(work)?.describe(&opening_chid, &opening_desc)?;
        for rung in rungs {
            info!("{label}: squash {rung} into the opening {opening_chid}");
            ctx.session(work)?.squash_into(rung, &opening_chid)?;
        }
        close(ctx, &one, label)
    })();
    if let Err(e) = &made {
        work_snapshot.restore(label, e.as_ref());
        if let Some(snapshot) = &agent_snapshot {
            snapshot.restore(label, e.as_ref());
        }
    }
    made?;
    match &c.land {
        Some(l) => land_closed(ctx, l, Some(bookmark)),
        None if c.rung.push => push(
            ctx,
            &PushCycle {
                running: r.clone(),
                agent,
            },
        ),
        None => Ok(()),
    }
}

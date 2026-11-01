//! The `trailers` subcommand: rewrite a range of work commits' trailers
//! to what `vc-x1` stamps today, only their descriptions changing.
//!
//! - `ochid:` becomes the partner's bare change id, so a label or URL
//!   form goes.
//! - `ocid:` links the agent commit named: its SHA, or the placeholder
//!   while it is still the uncommitted agent `@`.
//! - `pcid:` links each parent, by its SHA now, so the range goes oldest
//!   first, a commit's rewrite having changed the SHA its child links.
//! - A trailer naming nothing that resolves, a `lost` or an old short
//!   id, is left as it is, with no guessing and no `ocid:`.
//!
//! A Land runs [`update`] over the cycle too, with `add` off, so it only
//! refreshes the links a commit has, `--no-url-trailer`'s having none.
//! This changes the work repo alone, since an agent commit carries only
//! change ids, which a rewrite keeps.

use std::path::PathBuf;

use clap::Args;
use log::{debug, info, warn};

use crate::context::Context;
use crate::desc_helpers::{
    DEFAULT_ID_LEN, Trailers, commit_url, extract_bare_id, ocid_placeholder_for, pcids_for,
    repo_url_for, with_trailers,
};
use crate::jj;
use crate::options_flags::dry_run::DryRunFlag;
use crate::pair::Pair;
use crate::subcommand::SubcommandRunner;

/// Update the trailers of a range of work commits to what is stamped
/// today, only their descriptions changing.
#[derive(Args, Debug)]
pub struct TrailersArgs {
    /// Rewrite the trailers, the one thing this does so far
    #[arg(long, required = true)]
    pub update: bool,

    /// The commits: a bookmark or revision, those above the trunk up to
    /// it, or a revset with `..` or `::`, as written [default: those
    /// above the trunk up to `@-`]
    #[arg(long, value_name = "BM|REV")]
    pub range: Option<String>,

    /// Rewrite a commit already on the trunk's remote, which is refused
    /// without it
    #[arg(long)]
    pub yes: bool,

    /// Where: a path in the workspace, its root or either repo [default:
    /// the current directory]
    #[arg(short = 'R', long, value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Print each description it would write, and change nothing
    #[command(flatten)]
    pub dry_run: DryRunFlag,
}

/// Inputs to the trailers op, flat, owned, clap-free.
#[derive(Debug)]
pub struct TrailersParams {
    pub pair: Pair,
    pub range: Option<String>,
    pub yes: bool,
    pub dry_run: bool,
}

impl SubcommandRunner for TrailersArgs {
    type Params = TrailersParams;

    /// Resolve the workspace's pair.
    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(TrailersParams {
            pair: crate::pair::resolve_pair(self.repo.as_deref()).map_err(|e| e.to_string())?,
            range: self.range.clone(),
            yes: self.yes,
            dry_run: self.dry_run.value,
        })
    }

    /// Run the `trailers` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        trailers(ctx, params)
    }
}

/// Entry point: resolve the range, refuse a rewrite of the trunk's
/// remote unless `--yes`, and update.
pub fn trailers(
    ctx: &mut Context,
    params: &TrailersParams,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("trailers: enter {params:?}");
    let work = params.pair.work.as_path();
    let trunk = crate::commit::configured_trunk(work)?;
    let revset = match &params.range {
        Some(r) if r.contains("..") || r.contains("::") => r.clone(),
        Some(r) => format!("{trunk}..({r})"),
        None => format!("{trunk}..@-"),
    };
    let remote_trunk = format!("{trunk}@origin");
    if !params.yes
        && jj::rev_exists(work, &remote_trunk)?
        && jj::matches(work, &format!("({revset}) & ::{remote_trunk}"))?
    {
        return Err(format!(
            "trailers: '{revset}' reaches commits on {remote_trunk}, so this would rewrite the \
             trunk's published history: --yes does it anyway"
        )
        .into());
    }
    if repo_url_for(&params.pair.agent)?.is_none() {
        info!("trailers: the agent repo has no `origin` to link, so no `ocid:`");
    }
    if repo_url_for(work)?.is_none() {
        info!("trailers: the work repo has no `origin` to link, so no `pcid:`");
    }
    let changed = update(
        ctx,
        &params.pair,
        &Update {
            label: "trailers",
            revset: &revset,
            add: true,
            agent_at_committed: false,
            dry: params.dry_run,
        },
    )?;
    let verb = if params.dry_run {
        "would change"
    } else {
        "changed"
    };
    info!("trailers: {verb} {changed} commit(s) in '{revset}'");
    Ok(())
}

/// What [`update`] does, and to which commits.
pub(crate) struct Update<'a> {
    /// How messages name the command.
    pub label: &'a str,
    /// The work commits.
    pub revset: &'a str,
    /// Stamp `ocid:` and `pcid:` on a commit lacking them, which a Land
    /// does not, a commit with no links having asked for none.
    pub add: bool,
    /// A dry run of a Land: the agent `@` counts as committed, so what
    /// names it reads as set, though no SHA exists to show.
    pub agent_at_committed: bool,
    pub dry: bool,
}

/// Rewrite the trailers of each commit in the range, oldest first,
/// its tree checked unchanged, and return how many changed. Rerunning
/// finds nothing to do. A failure restores the work repo.
pub(crate) fn update(
    ctx: &mut Context,
    pair: &Pair,
    u: &Update,
) -> Result<usize, Box<dyn std::error::Error>> {
    let (work, agent) = (pair.work.as_path(), pair.agent.as_path());
    let label = u.label;
    // By change id, since a rewrite changes the SHAs of what is above it.
    let chids = jj::cids_of(work, u.revset)?
        .iter()
        .map(|rev| jj::chid_of(work, rev))
        .collect::<Result<Vec<_>, _>>()?;
    let agent_url = repo_url_for(agent)?;
    let agent_at = jj::chid_of(agent, "@")?;
    let placeholder = ocid_placeholder_for(agent)?;
    let snapshot = crate::pair::Snapshot::take(work)?;
    let mut changed = 0;
    for chid in &chids {
        let desc = jj::desc_of(work, chid)?;
        let old = Trailers::of(&desc);
        if old.ochids.is_empty() {
            continue;
        }
        let resolves = |id: &str| -> Result<bool, Box<dyn std::error::Error>> {
            Ok(id.len() == DEFAULT_ID_LEN && jj::rev_exists(agent, id)?)
        };
        let mut ochids = Vec::new();
        for value in &old.ochids {
            let id = extract_bare_id(value);
            ochids.push(match resolves(id)? {
                true => id.to_string(),
                false => value.clone(),
            });
        }
        let partner = extract_bare_id(&ochids[ochids.len() - 1]).to_string();
        let ocid = if !u.add && old.ocid.is_none() {
            None
        } else if !resolves(&partner)? {
            old.ocid.clone()
        } else if partner == agent_at && u.agent_at_committed {
            agent_url
                .as_deref()
                .map(|url| commit_url(url, "<the agent commit's SHA>"))
        } else if partner == agent_at {
            placeholder.clone()
        } else {
            match &agent_url {
                Some(url) => Some(commit_url(url, &jj::cid_of(agent, &partner)?)),
                None => old.ocid.clone(),
            }
        };
        let pcids = if u.add || !old.pcids.is_empty() {
            pcids_for(work, &jj::parent_cids_of(work, chid)?)?
        } else {
            Vec::new()
        };
        let new = with_trailers(
            &desc,
            &Trailers {
                ochids,
                ocid,
                pcids,
            },
        );
        if new == desc {
            continue;
        }
        changed += 1;
        if u.dry {
            info!("{label}: [dry-run] {chid}:\n{new}");
            continue;
        }
        if crate::pair::is_published(work, chid)? {
            warn!("{label}: {chid} is published, so its copy on the remote goes stale");
        }
        info!("{label}: update the trailers of {chid}");
        let tree = jj::tree_of(work, chid)?;
        let stamped = ctx.session(work)?.describe(chid, &new).and_then(|_| {
            match jj::tree_of(work, chid)? == tree {
                true => Ok(()),
                false => {
                    Err(format!("{label}: updating {chid}'s trailers changed its tree").into())
                }
            }
        });
        if let Err(e) = stamped {
            snapshot.restore(label, e.as_ref());
            return Err(e);
        }
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{Fixture, chid, description, git_ok, jj_ok, test_ctx};
    use std::fs;

    const AGENT_URL: &str = "https://github.com/o/r.agent-session";
    const WORK_URL: &str = "https://github.com/o/r";

    fn hosted(fx: &Fixture) {
        git_ok(&fx.work, &["remote", "set-url", "origin", WORK_URL]);
        git_ok(&fx.bot, &["remote", "set-url", "origin", AGENT_URL]);
    }

    fn cid(repo: &std::path::Path, rev: &str) -> String {
        jj::cid_of(repo, rev).expect("cid")
    }

    /// A rung by hand, carrying `trailer`, with a custom one below it.
    fn rung(fx: &Fixture, name: &str, trailer: &str) -> String {
        fs::write(fx.work.join(format!("{name}.txt")), "x\n").expect("write");
        let desc = format!("feat: {name}\n\nBody.\n\nochid: {trailer}\nreviewed-by: wink\n");
        jj_ok(&fx.work, &["commit", "-m", &desc]);
        chid(&fx.work, "@-")
    }

    fn params(fx: &Fixture, range: &str, dry_run: bool) -> TrailersParams {
        TrailersParams {
            pair: Pair {
                work: fx.work.clone(),
                agent: fx.bot.clone(),
            },
            range: Some(range.to_string()),
            yes: false,
            dry_run,
        }
    }

    /// Old forms become the bare change id, `ocid:` the placeholder for
    /// the agent `@` and a SHA for a commit made, `pcid:` each parent's
    /// SHA as rewritten, the custom trailer kept, change ids and trees
    /// unchanged, a dry run doing none of it, and a rerun nothing.
    #[test]
    fn a_range_takes_todays_trailers() {
        let fx = Fixture::new("trailers-update");
        hosted(&fx);
        let made = chid(&fx.bot, "@-");
        let made_sha = cid(&fx.bot, "@-");
        let at = chid(&fx.bot, "@");
        let first = rung(&fx, "first", &format!("/.claude/{made}"));
        let second = rung(&fx, "second", &format!("{AGENT_URL}#{at}"));
        let lost = rung(&fx, "third", "/.claude/lost");
        let tree = jj::tree_of(&fx.work, &second).expect("tree");

        let before = description(&fx.work, &first);
        let n = trailers(&mut test_ctx(), &params(&fx, "@-", true)).ok();
        assert!(n.is_some(), "a dry run");
        assert_eq!(
            description(&fx.work, &first),
            before,
            "a dry run writes none"
        );

        trailers(&mut test_ctx(), &params(&fx, "@-", false)).expect("updated");
        let parent = cid(&fx.work, "main");
        assert!(
            description(&fx.work, &first).trim_end().ends_with(&format!(
                "\n\nochid: {made}\nocid: {AGENT_URL}/commit/{made_sha}\npcid: {WORK_URL}/commit/{parent}\nreviewed-by: wink"
            )),
            "{}",
            description(&fx.work, &first)
        );
        let parent = cid(&fx.work, &first);
        assert!(
            description(&fx.work, &second).trim_end().ends_with(&format!(
                "\n\nochid: {at}\nocid: {AGENT_URL}/commit/xxxx\npcid: {WORK_URL}/commit/{parent}\nreviewed-by: wink"
            )),
            "{}",
            description(&fx.work, &second)
        );
        let third = description(&fx.work, &lost);
        assert!(third.contains("ochid: /.claude/lost\n"), "{third}");
        assert!(!third.contains("ocid:"), "{third}");
        assert_eq!(jj::tree_of(&fx.work, &second).expect("tree"), tree);
        assert!(jj::is_empty(&fx.work, "@").expect("empty"));

        let sha = cid(&fx.work, &second);
        trailers(&mut test_ctx(), &params(&fx, "@-", false)).expect("rerun");
        assert_eq!(cid(&fx.work, &second), sha, "a rerun rewrites nothing");
    }

    /// With no `--range` it reaches `@-`, past a bookmark below it, and a
    /// local `origin` links by `file:///`.
    #[test]
    fn the_default_range_reaches_the_work_head() {
        let fx = Fixture::new("trailers-default");
        let made = chid(&fx.bot, "@-");
        let first = rung(&fx, "first", &format!("/.claude/{made}"));
        jj_ok(&fx.work, &["bookmark", "set", "topic", "-r", "@-"]);
        let second = rung(&fx, "second", &format!("/.claude/{made}"));

        let mut p = params(&fx, "unused", false);
        p.range = None;
        trailers(&mut test_ctx(), &p).expect("updated");

        for c in [&first, &second] {
            let desc = description(&fx.work, c);
            assert!(
                desc.contains(&format!("\n\nochid: {made}\nocid: file:///")),
                "{desc}"
            );
            assert!(desc.contains("\npcid: file:///"), "{desc}");
            assert!(desc.trim_end().ends_with("\nreviewed-by: wink"), "{desc}");
        }
    }

    /// A commit with three parents gets three `pcid:` lines, in parent
    /// order, whichever way the parents were made.
    #[test]
    fn every_parent_gets_a_pcid_in_order() {
        let fx = Fixture::new("trailers-three-parents");
        let made = chid(&fx.bot, "@-");
        let mut sides = Vec::new();
        for name in ["c", "a", "b"] {
            jj_ok(&fx.work, &["new", "main"]);
            rung(&fx, name, &made);
            sides.push(cid(&fx.work, "@-"));
        }
        jj_ok(&fx.work, &["new", &sides[0], &sides[1], &sides[2]]);
        let merge = rung(&fx, "merge", &made);
        assert_eq!(
            jj::parent_cids_of(&fx.work, &merge).expect("parents"),
            sides
        );

        trailers(&mut test_ctx(), &params(&fx, &merge, false)).expect("updated");

        let parents = jj::parent_cids_of(&fx.work, &merge).expect("parents");
        let pcids = Trailers::of(&description(&fx.work, &merge)).pcids;
        assert_eq!(pcids.len(), 3, "{pcids:?}");
        for (pcid, parent) in pcids.iter().zip(&parents) {
            assert!(pcid.ends_with(&format!("/commit/{parent}")), "{pcid}");
        }
    }

    /// A range reaching the trunk's remote is refused unless `--yes`.
    #[test]
    fn the_trunks_remote_is_refused_unless_asked() {
        let fx = Fixture::new("trailers-trunk");
        hosted(&fx);
        let main = cid(&fx.work, "main");
        let err = trailers(&mut test_ctx(), &params(&fx, "::main", false))
            .expect_err("refused")
            .to_string();
        assert!(err.contains("main@origin"), "{err}");
        assert_eq!(cid(&fx.work, "main"), main);
    }

    /// With `add` off, as a Land runs it, a commit with no links gets none.
    #[test]
    fn a_commit_with_no_links_stays_so_without_add() {
        let fx = Fixture::new("trailers-no-add");
        hosted(&fx);
        let made = chid(&fx.bot, "@-");
        let first = rung(&fx, "first", &made);
        let before = description(&fx.work, &first);
        let n = update(
            &mut test_ctx(),
            &Pair {
                work: fx.work.clone(),
                agent: fx.bot.clone(),
            },
            &Update {
                label: "t",
                revset: "main..@-",
                add: false,
                agent_at_committed: false,
                dry: false,
            },
        )
        .expect("updated");
        assert_eq!(n, 0);
        assert_eq!(description(&fx.work, &first), before);
    }
}

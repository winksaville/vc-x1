//! The `trailers` subcommand: rewrite a range of work commits' trailers
//! to what `vc-x1` stamps today, only their descriptions changing.
//!
//! - `ochid:` becomes the partner's bare change id, so a label or URL
//!   form goes, and an older 8 or 32 character id, or a SHA, naming one
//!   agent commit becomes its 12 character change id.
//! - `ocid:` links the agent commit named: its SHA, or the placeholder
//!   while it is still the uncommitted agent `@`.
//! - `pcid:` links each parent, by its SHA now, so the range goes oldest
//!   first, a commit's rewrite having changed the SHA its child links. A
//!   commit with no `ochid:` gets its `pcid:` lines alone.
//! - A trailer naming nothing that resolves, a `lost` or a partner gone,
//!   is left as it is, with no guessing and no `ocid:`, and the summary
//!   names it.
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
    Trailers, commit_url, extract_bare_id, is_change_id_form, is_commit_id_form,
    ocid_placeholder_for, pcids_for, repo_url_for, with_trailers,
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
    /// without it, a dry run excepted
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
/// remote unless `--yes` or a dry run, update, and sort every commit in
/// the summary.
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
        let reaches = format!(
            "'{revset}' reaches commits on {remote_trunk}, so this would rewrite the trunk's \
             published history"
        );
        if !params.dry_run {
            return Err(format!("trailers: {reaches}: --yes does it anyway").into());
        }
        info!("trailers: [dry-run] {reaches}, which the run takes --yes for");
    }
    if repo_url_for(&params.pair.agent)?.is_none() {
        info!("trailers: the agent repo has no `origin` to link, so no `ocid:`");
    }
    if repo_url_for(work)?.is_none() {
        info!("trailers: the work repo has no `origin` to link, so no `pcid:`");
    }
    let outcome = update(
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
    info!(
        "trailers: {verb} {} of {} commit(s) in '{revset}', {} already current",
        outcome.changed,
        outcome.changed + outcome.current,
        outcome.current
    );
    for (chid, value) in &outcome.unresolved {
        let why = match extract_bare_id(value) {
            "lost" | "none" => "a marker naming no partner",
            _ => "which names no one agent commit",
        };
        info!("trailers: {chid} keeps `ochid: {value}`, {why}, so its `ocid:` is left as it is");
    }
    for chid in &outcome.no_ochid {
        info!("trailers: {chid} has no `ochid:`, so it links its parents alone");
    }
    Ok(())
}

/// What [`update`] did with each commit, for the summary.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Outcome {
    /// Commits whose trailers changed, or would in a dry run.
    pub changed: usize,
    /// Commits already as they would be stamped.
    pub current: usize,
    /// Each `ochid:` kept as it is, naming no one agent commit, by the
    /// change id of the commit carrying it.
    pub unresolved: Vec<(String, String)>,
    /// Commits with no `ochid:`, their `pcid:` lines alone stamped.
    pub no_ochid: Vec<String>,
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
/// its tree checked unchanged, and return what became of each. Rerunning
/// finds nothing to do. A failure restores the work repo.
pub(crate) fn update(
    ctx: &mut Context,
    pair: &Pair,
    u: &Update,
) -> Result<Outcome, Box<dyn std::error::Error>> {
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
    let mut outcome = Outcome::default();
    for chid in &chids {
        let desc = jj::desc_of(work, chid)?;
        let old = Trailers::of(&desc);
        if old.ochids.is_empty() {
            if !u.add {
                outcome.current += 1;
                continue;
            }
            outcome.no_ochid.push(chid.clone());
        }
        // Each `ochid:` as the 12 character change id of the one agent
        // commit it names, and `partner` the last one's, as `ocid:` links.
        let mut ochids = Vec::new();
        let mut partner = None;
        for value in &old.ochids {
            let id = extract_bare_id(value);
            partner = match is_change_id_form(id) || is_commit_id_form(id) {
                true => jj::one_chid(agent, id)?,
                false => None,
            };
            match &partner {
                Some(full) => ochids.push(full.clone()),
                None => {
                    ochids.push(value.clone());
                    outcome.unresolved.push((chid.clone(), value.clone()));
                }
            }
        }
        let ocid = match &partner {
            _ if !u.add && old.ocid.is_none() => None,
            None => old.ocid.clone(),
            Some(p) if *p == agent_at && u.agent_at_committed => agent_url
                .as_deref()
                .map(|url| commit_url(url, "<the agent commit's SHA>")),
            Some(p) if *p == agent_at => placeholder.clone(),
            Some(p) => match &agent_url {
                Some(url) => Some(commit_url(url, &jj::cid_of(agent, p)?)),
                None => old.ocid.clone(),
            },
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
            outcome.current += 1;
            continue;
        }
        outcome.changed += 1;
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
    Ok(outcome)
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
        assert_eq!(n.changed, 0);
        assert_eq!(description(&fx.work, &first), before);
    }

    /// An older 8 or 32 character id, or a SHA, naming one agent commit
    /// becomes its 12 character change id, linked by `ocid:`, and a `lost`
    /// stays as it is, the outcome naming it.
    #[test]
    fn an_older_id_becomes_the_twelve() {
        let fx = Fixture::new("trailers-older-id");
        hosted(&fx);
        let made = chid(&fx.bot, "@-");
        let made_sha = cid(&fx.bot, "@-");
        let full = jj_ok(
            &fx.bot,
            &["log", "-r", "@-", "--no-graph", "-T", "change_id"],
        );
        let full = full.trim();
        assert_eq!(full.len(), 32, "{full}");
        let short = rung(&fx, "short", &format!("/.claude/{}", &made[..8]));
        let long = rung(&fx, "long", full);
        let sha = rung(&fx, "sha", &format!("/.claude/{}", &made_sha[..12]));
        let lost = rung(&fx, "lost", "/.claude/lost");

        let outcome = update(
            &mut test_ctx(),
            &Pair {
                work: fx.work.clone(),
                agent: fx.bot.clone(),
            },
            &Update {
                label: "t",
                revset: "main..@-",
                add: true,
                agent_at_committed: false,
                dry: false,
            },
        )
        .expect("updated");

        for c in [&short, &long, &sha] {
            let desc = description(&fx.work, c);
            assert!(
                desc.contains(&format!(
                    "\n\nochid: {made}\nocid: {AGENT_URL}/commit/{made_sha}\npcid: "
                )),
                "{desc}"
            );
        }
        assert!(description(&fx.work, &lost).contains("ochid: /.claude/lost\n"));
        assert_eq!(
            outcome.unresolved,
            vec![(lost.clone(), "/.claude/lost".to_string())]
        );
        assert_eq!((outcome.changed, outcome.current), (4, 0));
    }

    /// A commit with no `ochid:` gets its `pcid:` lines alone, named in the
    /// outcome, and with `add` off, as a Land runs it, nothing.
    #[test]
    fn a_partnerless_commit_links_its_parents() {
        let fx = Fixture::new("trailers-partnerless");
        hosted(&fx);
        fs::write(fx.work.join("bare.txt"), "x\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: bare\n\nBody.\n"]);
        let bare = chid(&fx.work, "@-");
        let parent = cid(&fx.work, "main");
        let pair = Pair {
            work: fx.work.clone(),
            agent: fx.bot.clone(),
        };
        let run = |add: bool| {
            update(
                &mut test_ctx(),
                &pair,
                &Update {
                    label: "t",
                    revset: "main..@-",
                    add,
                    agent_at_committed: false,
                    dry: false,
                },
            )
            .expect("updated")
        };

        let before = description(&fx.work, &bare);
        assert_eq!(run(false).changed, 0);
        assert_eq!(description(&fx.work, &bare), before);

        let outcome = run(true);
        assert_eq!(outcome.no_ochid, vec![bare.clone()]);
        assert_eq!(
            description(&fx.work, &bare).trim_end(),
            format!("feat: bare\n\nBody.\n\npcid: {WORK_URL}/commit/{parent}")
        );
        assert_eq!(run(true).changed, 0, "a rerun rewrites nothing");
    }

    /// A dry run reaching the trunk's remote needs no `--yes`, and changes
    /// nothing.
    #[test]
    fn a_dry_run_of_the_trunks_remote_needs_no_yes() {
        let fx = Fixture::new("trailers-trunk-dry");
        hosted(&fx);
        let main = cid(&fx.work, "main");
        trailers(&mut test_ctx(), &params(&fx, "::main", true)).expect("a dry run");
        assert_eq!(cid(&fx.work, "main"), main);
    }
}

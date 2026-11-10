//! The `trailers` subcommand: rewrite a range of work commits' trailers
//! to what `vc-x1` stamps today, only their descriptions changing.
//!
//! - `ochid:` becomes the partner's bare change id, so a label or URL
//!   form goes, and an older 8 or 32 character id, or a SHA, naming one
//!   agent commit becomes its 12 character change id.
//! - `orepo:` above it names the agent repo, and an older commit's
//!   `ocid:` and `pcid:`, which named commits by SHA, go. No trailer then
//!   holds a SHA, so a commit's rewrite leaves its children's valid.
//! - A trailer naming nothing that resolves, a `lost` or a partner gone,
//!   is left as it is, with no guessing, and the summary names it. A
//!   commit with no `ochid:` has no partner to link, and only loses any
//!   older links it carries.
//!
//! A Land runs [`update`] over the cycle too, with `add` off, so it gives
//! `orepo:` only to a commit that links already, `--no-url-trailer`'s
//! having none. This changes the work repo alone, the agent repo's
//! trailers being change ids, which a rewrite keeps.

use std::path::PathBuf;

use clap::Args;
use log::{debug, info, warn};

use crate::context::Context;
use crate::desc_helpers::{
    Trailers, extract_bare_id, is_change_id_form, is_commit_id_form, repo_url_for, with_trailers,
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

    /// The commits, a revset taken as written, `main..@-` a cycle's
    /// and `..@-` a whole history. Required: no base is assumed
    #[arg(long, value_name = "REVSET", required = true)]
    pub range: String,

    /// Rewrite a commit in jj's `immutable()`, `main@origin` among
    /// them, which is refused without it, a dry run excepted
    #[arg(long)]
    pub ignore_immutable: bool,

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
    pub range: String,
    pub ignore_immutable: bool,
    pub dry_run: bool,
}

impl SubcommandRunner for TrailersArgs {
    type Params = TrailersParams;

    /// Resolve the workspace's pair.
    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(TrailersParams {
            pair: crate::pair::resolve_pair(self.repo.as_deref()).map_err(|e| e.to_string())?,
            range: self.range.clone(),
            ignore_immutable: self.ignore_immutable,
            dry_run: self.dry_run.value,
        })
    }

    /// Run the `trailers` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        trailers(ctx, params)
    }
}

/// Entry point: refuse a range reaching jj's `immutable()` unless
/// `--ignore-immutable` or a dry run, update, and sort every commit in
/// the summary.
pub fn trailers(
    ctx: &mut Context,
    params: &TrailersParams,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("trailers: enter {params:?}");
    let work = params.pair.work.as_path();
    let revset = params.range.clone();
    if !params.ignore_immutable && jj::is_immutable(work, &revset)? {
        let reaches = format!(
            "'{revset}' reaches commits in jj's `immutable()`, so this would rewrite history \
             others have"
        );
        if !params.dry_run {
            return Err(format!("trailers: {reaches}: --ignore-immutable does it anyway").into());
        }
        info!("trailers: [dry-run] {reaches}, which the run takes --ignore-immutable for");
    }
    if repo_url_for(&params.pair.agent)?.is_none() {
        info!("trailers: the agent repo has no `origin` to link, so no `orepo:`");
    }
    let outcome = update(
        ctx,
        &params.pair,
        &Update {
            label: "trailers",
            revset: &revset,
            add: true,
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
        info!("trailers: {chid} keeps `ochid: {value}`, {why}");
    }
    for chid in &outcome.no_ochid {
        info!("trailers: {chid} has no `ochid:`, so it has no partner to link");
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
    /// Commits with no `ochid:`, so no partner to link.
    pub no_ochid: Vec<String>,
}

/// What [`update`] does, and to which commits.
pub(crate) struct Update<'a> {
    /// How messages name the command.
    pub label: &'a str,
    /// The work commits.
    pub revset: &'a str,
    /// Stamp `orepo:` on a commit with no links, which a Land does not,
    /// a commit with no links having asked for none.
    pub add: bool,
    pub dry: bool,
}

/// Whether `desc` links by an older trailer, `ocid:` or `pcid:`, which a
/// restamp turns into `orepo:`.
fn has_older_links(desc: &str) -> bool {
    desc.lines()
        .any(|line| line.starts_with("ocid: ") || line.starts_with("pcid: "))
}

/// Rewrite the trailers of each commit in the range, its tree checked
/// unchanged, and return what became of each. Rerunning finds nothing
/// to do. A failure restores the work repo.
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
    let snapshot = crate::pair::Snapshot::take(work)?;
    let mut outcome = Outcome::default();
    for chid in &chids {
        let desc = jj::desc_of(work, chid)?;
        let old = Trailers::of(&desc);
        if old.ochids.is_empty() {
            if !u.add && !has_older_links(&desc) {
                outcome.current += 1;
                continue;
            }
            outcome.no_ochid.push(chid.clone());
        }
        // Each `ochid:` as the 12 character change id of the one agent
        // commit it names.
        let mut ochids = Vec::new();
        for value in &old.ochids {
            let id = extract_bare_id(value);
            let partner = match is_change_id_form(id) || is_commit_id_form(id) {
                true => jj::one_chid(agent, id)?,
                false => None,
            };
            match partner {
                Some(full) => ochids.push(full),
                None => {
                    ochids.push(value.clone());
                    outcome.unresolved.push((chid.clone(), value.clone()));
                }
            }
        }
        // `orepo:` says where the partners are, so it goes with them.
        let linked = u.add || old.orepo.is_some() || has_older_links(&desc);
        let orepo = match (ochids.is_empty() || !linked, &agent_url) {
            (true, _) => None,
            (false, Some(url)) => Some(url.clone()),
            (false, None) => old.orepo.clone(),
        };
        let new = with_trailers(&desc, &Trailers::new(orepo, ochids));
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
            range: range.to_string(),
            ignore_immutable: false,
            dry_run,
        }
    }

    /// Old forms become the bare change id below `orepo:`, the agent `@`
    /// and a commit made alike, the custom trailer kept, change ids and
    /// trees unchanged, a dry run doing none of it, and a rerun nothing.
    #[test]
    fn a_range_takes_todays_trailers() {
        let fx = Fixture::new("trailers-update");
        hosted(&fx);
        let made = chid(&fx.bot, "@-");
        let at = chid(&fx.bot, "@");
        let first = rung(&fx, "first", &format!("/.claude/{made}"));
        let second = rung(&fx, "second", &format!("{AGENT_URL}#{at}"));
        let lost = rung(&fx, "third", "/.claude/lost");
        let tree = jj::tree_of(&fx.work, &second).expect("tree");

        let before = description(&fx.work, &first);
        let n = trailers(&mut test_ctx(), &params(&fx, "main..@-", true)).ok();
        assert!(n.is_some(), "a dry run");
        assert_eq!(
            description(&fx.work, &first),
            before,
            "a dry run writes none"
        );

        trailers(&mut test_ctx(), &params(&fx, "main..@-", false)).expect("updated");
        for (c, partner) in [(&first, &made), (&second, &at)] {
            let desc = description(&fx.work, c);
            assert!(
                desc.trim_end().ends_with(&format!(
                    "\n\norepo: {AGENT_URL}\nochid: {partner}\nreviewed-by: wink"
                )),
                "{desc}"
            );
        }
        let third = description(&fx.work, &lost);
        assert!(third.contains("ochid: /.claude/lost\n"), "{third}");
        assert_eq!(jj::tree_of(&fx.work, &second).expect("tree"), tree);
        assert!(jj::is_empty(&fx.work, "@").expect("empty"));

        let sha = cid(&fx.work, &second);
        trailers(&mut test_ctx(), &params(&fx, "main..@-", false)).expect("rerun");
        assert_eq!(cid(&fx.work, &second), sha, "a rerun rewrites nothing");
    }

    /// The range is taken as written, `main..@-` reaching past a bookmark
    /// below `@-`, and a local `origin` is named by `file:///`.
    #[test]
    fn the_range_is_taken_as_written() {
        let fx = Fixture::new("trailers-as-written");
        let made = chid(&fx.bot, "@-");
        let first = rung(&fx, "first", &format!("/.claude/{made}"));
        jj_ok(&fx.work, &["bookmark", "set", "topic", "-r", "@-"]);
        let second = rung(&fx, "second", &format!("/.claude/{made}"));

        trailers(&mut test_ctx(), &params(&fx, "main..@-", false)).expect("updated");

        for c in [&first, &second] {
            let desc = description(&fx.work, c);
            assert!(desc.contains("\n\norepo: file:///"), "{desc}");
            assert!(desc.contains(&format!("\nochid: {made}\n")), "{desc}");
            assert!(desc.trim_end().ends_with("\nreviewed-by: wink"), "{desc}");
        }
    }

    /// An older commit's `ocid:` and `pcid:` give way to `orepo:`, a merge's
    /// several `pcid:` lines included, and so does a Land's run, which adds
    /// no link to a commit with none.
    #[test]
    fn older_links_become_orepo() {
        let fx = Fixture::new("trailers-older-links");
        hosted(&fx);
        let made = chid(&fx.bot, "@-");
        let mut sides = Vec::new();
        for name in ["a", "b"] {
            jj_ok(&fx.work, &["new", "main"]);
            rung(&fx, name, &made);
            sides.push(cid(&fx.work, "@-"));
        }
        jj_ok(&fx.work, &["new", &sides[0], &sides[1]]);
        fs::write(fx.work.join("merge.txt"), "x\n").expect("write");
        let old = format!(
            "feat: merge\n\nBody.\n\nochid: {made}\nocid: {AGENT_URL}/commit/1234\n\
             pcid: {WORK_URL}/commit/{}\npcid: {WORK_URL}/commit/{}\n",
            sides[0], sides[1]
        );
        jj_ok(&fx.work, &["commit", "-m", &old]);
        let merge = chid(&fx.work, "@-");

        let outcome = update(
            &mut test_ctx(),
            &Pair {
                work: fx.work.clone(),
                agent: fx.bot.clone(),
            },
            &Update {
                label: "t",
                revset: &merge,
                add: false,
                dry: false,
            },
        )
        .expect("updated");
        assert_eq!(outcome.changed, 1);
        assert_eq!(
            description(&fx.work, &merge).trim_end(),
            format!("feat: merge\n\nBody.\n\norepo: {AGENT_URL}\nochid: {made}")
        );
        assert_eq!(
            jj::parent_cids_of(&fx.work, &merge).expect("parents"),
            sides,
            "the parents are git's"
        );
    }

    /// A range reaching jj's `immutable()`, `main@origin`, is refused
    /// unless `--ignore-immutable`.
    #[test]
    fn immutable_is_refused_unless_asked() {
        let fx = Fixture::new("trailers-immutable");
        hosted(&fx);
        let main = cid(&fx.work, "main");
        let err = trailers(&mut test_ctx(), &params(&fx, "::main", false))
            .expect_err("refused")
            .to_string();
        assert!(
            err.contains("immutable()") && err.contains("--ignore-immutable"),
            "{err}"
        );
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
                dry: false,
            },
        )
        .expect("updated");
        assert_eq!(n.changed, 0);
        assert_eq!(description(&fx.work, &first), before);
    }

    /// An older 8 or 32 character id, or a SHA, naming one agent commit
    /// becomes its 12 character change id, below `orepo:`, and a `lost`
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
                dry: false,
            },
        )
        .expect("updated");

        for c in [&short, &long, &sha] {
            let desc = description(&fx.work, c);
            assert!(
                desc.contains(&format!("\n\norepo: {AGENT_URL}\nochid: {made}\n")),
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

    /// A commit with no `ochid:` has no partner to link: it is named in the
    /// outcome and left as it is, and with `add` off, as a Land runs it,
    /// not even named.
    #[test]
    fn a_partnerless_commit_is_named_and_left() {
        let fx = Fixture::new("trailers-partnerless");
        hosted(&fx);
        fs::write(fx.work.join("bare.txt"), "x\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: bare\n\nBody.\n"]);
        let bare = chid(&fx.work, "@-");
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
                    dry: false,
                },
            )
            .expect("updated")
        };

        let before = description(&fx.work, &bare);
        let outcome = run(false);
        assert_eq!((outcome.changed, outcome.no_ochid.len()), (0, 0));
        let outcome = run(true);
        assert_eq!(outcome.changed, 0);
        assert_eq!(outcome.no_ochid, vec![bare.clone()]);
        assert_eq!(description(&fx.work, &bare), before);
    }

    /// A dry run reaching jj's `immutable()` needs no
    /// `--ignore-immutable`, and changes nothing.
    #[test]
    fn a_dry_run_of_immutable_needs_no_override() {
        let fx = Fixture::new("trailers-immutable-dry");
        hosted(&fx);
        let main = cid(&fx.work, "main");
        trailers(&mut test_ctx(), &params(&fx, "::main", true)).expect("a dry run");
        assert_eq!(cid(&fx.work, "main"), main);
    }
}

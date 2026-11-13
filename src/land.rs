//! Land: make a finished cycle permanent on its `--onto` bookmark. The agent `@` is
//! committed when the cycle's commits name it, the bookmark is
//! fast-forwarded to the cycle's last commit, and the cycle's bookmark
//! is deleted, all on the machine, every check made before anything
//! moves. `--push` also publishes, the work repo first and then the
//! agent repo, and deletes the bookmark on `origin`, and without it
//! `vc-x1 push` publishes.
//!
//! - [`plan`] refuses before anything is made: `--onto` is a local
//!   bookmark at its `origin` position, so the sync a Land begins with
//!   has nothing to bring, and the branch descends from it, so the tree
//!   landed is the reviewed one. The branch's other bookmarks are named
//!   as merged and left as they are.
//! - [`run`] lands what the plan names: a conflicted target is refused,
//!   the agent `@` is committed with one `ochid:` per work commit naming
//!   it, oldest first, the trailers the cycle's commits carry are updated
//!   by [`crate::trailers::update`], an older commit's `ocid:` and
//!   `pcid:` giving way to `orepo:`, `--onto` is fast-forwarded, and the cycle's bookmark is deleted, `--skip`
//!   keeping a side. A dry run makes every
//!   check and says each step, and does none: each step is its line and
//!   then the step behind `if !dry`, so a step added brings its dry-run
//!   line.
//!
//! `vc-x1 land -s b|p --source <BM|REV> --onto <BOOKMARK>` lands a cycle
//! whose last commit is already made, and `commit --closing ... --land`
//! plans its Land before the closing is made and runs it after, both
//! through these two.
//!
//! - `--source` names the cycle's last commit, a bookmark or a revision,
//!   and the cycle's bookmark by the rule [`cycle_bookmark`] states,
//!   `--closing`'s. Left out, it is the one local bookmark besides
//!   `--onto`, [`default_source`].
//! - `--onto` is any local bookmark `--source` descends from, so a
//!   bushy tree lands anywhere. Land only fast-forwards: a merge is
//!   `commit --closing`'s to make.
//! - `--delete-intermediate-bookmarks` deletes the bookmarks on the
//!   commits strictly between `--onto` and `--source`, the earlier
//!   cycles' of a stack landing together. Without it they are named as
//!   merged and kept, and a bookmark at the tip besides the cycle's is
//!   kept either way.
//! - The publish follows where the landed commit is: the work `@-`, whose pair
//!   may be unfinished, goes through `push`, which finishes it, and a
//!   commit below it, whose pair is finished, by its `--onto` bookmark pushed alone
//!   and the agent `main` published.

use std::path::{Path, PathBuf};

use log::{debug, info};

use crate::context::Context;
use crate::desc_helpers::{
    extract_bare_id, extract_ochid_from_desc, repo_url_for, with_ochids, without_trailer_lines,
};
use crate::jj;
use crate::options_flags::scope::RepoScope;
use crate::options_flags::squash::SquashSpec;

/// Which side of the cycle's bookmark a Land keeps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Skip {
    /// The local bookmark is kept, only `origin`'s deleted.
    Local,
    /// The remote bookmark is kept, only the local one deleted.
    Remote,
    /// Both are kept, none deleted.
    Both,
}

/// How the Land's messages name the command: `vc-x1 land` is retired,
/// and `cycle --land` runs this code.
const LABEL: &str = "cycle --land";

/// Inputs to the land op, flat, owned, clap-free.
#[derive(Debug)]
pub struct LandParams {
    /// The work repo of `-s b`, or the plain repo of `-s p`.
    pub repo: PathBuf,
    pub scope: RepoScope,
    /// `None` is the one local bookmark besides `onto`.
    pub source: Option<String>,
    pub onto: String,
    pub skip: Option<Skip>,
    pub delete_intermediate: bool,
    pub push: bool,
    pub dry_run: bool,
}

/// Entry point: resolve `--source` and the cycle's bookmark, plan the
/// Land, refusing before anything moves, and run it.
pub fn land(ctx: &mut Context, params: &LandParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("{LABEL}: enter {params:?}");
    let repo = params.repo.as_path();
    if !jj::local_bookmark_exists(repo, &params.onto)? {
        return Err(format!(
            "{LABEL}: '{}' is not a local bookmark, and Land moves one",
            params.onto
        )
        .into());
    }
    let source = match &params.source {
        Some(s) => s.clone(),
        None => default_source(repo, &params.onto)?,
    };
    let tip = resolve_tip(repo, LABEL, &source)?;
    if jj::cid_of(repo, &params.onto)? == tip {
        return Err(format!(
            "{LABEL}: '{}' is at '{source}' already, so there is nothing to land",
            params.onto
        )
        .into());
    }
    let bookmark = cycle_bookmark(repo, LABEL, "--bookmark", &source, &params.onto, &tip)?;
    if bookmark.is_none() {
        info!(
            "{LABEL}: no bookmark on the branch above '{}', so none is deleted",
            params.onto
        );
    }
    let plan = plan(
        repo,
        params.onto.clone(),
        &tip,
        bookmark.as_deref(),
        params.skip,
        params.delete_intermediate,
        matches!(params.scope, RepoScope::Por),
    )?;
    run(
        ctx,
        repo,
        params.scope.clone(),
        &LandPlan {
            push: params.push,
            ..plan
        },
        &tip,
        &format!("'{source}'"),
        params.dry_run,
    )
}

/// The cycle's bookmark left unnamed: the one local bookmark besides
/// `onto`, refused with the list when there are several, since naming
/// one, `--bookmark`, settles it.
pub(crate) fn default_source(
    repo: &Path,
    onto: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut found = jj::local_bookmarks_at(repo, "bookmarks()")?;
    found.retain(|b| b != onto);
    match found.as_slice() {
        [one] => {
            info!(
                "{LABEL}: the cycle's bookmark is '{one}', the one local bookmark besides '{onto}'"
            );
            Ok(one.clone())
        }
        [] => Err(format!(
            "{LABEL}: there is no local bookmark besides '{onto}', so there is nothing to land"
        )
        .into()),
        several => Err(format!(
            "{LABEL}: there are several local bookmarks besides '{onto}', {}, so which is the \
             cycle's is not known: name it, --bookmark <NAME>",
            several.join(", ")
        )
        .into()),
    }
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
/// parent's, above `onto`, several at that height refused with the
/// list, since naming one settles it, and none being none. The `onto`
/// bookmark itself is never the cycle's.
pub(crate) fn cycle_bookmark(
    repo: &Path,
    label: &str,
    flag: &str,
    arg: &str,
    onto: &str,
    tip: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if arg != onto && jj::local_bookmark_exists(repo, arg)? {
        return Ok(Some(arg.to_string()));
    }
    let base = jj::cid_of(repo, onto)?;
    let mut found =
        jj::local_bookmarks_at(repo, &format!("heads(({base}..({tip})) & bookmarks())"))?;
    found.retain(|b| *b != onto);
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

/// A commit id cut to the length the ids are shown at.
fn short(id: &str) -> String {
    id.chars()
        .take(crate::desc_helpers::DEFAULT_ID_LEN)
        .collect()
}

/// The install the Land's last line names, when the workspace's config
/// has one: the artifact is the project's, so a workspace with no
/// `[install]` is told nothing.
fn install_hint(repo: &Path) -> String {
    let configured = crate::config_md::load(repo)
        .ok()
        .flatten()
        .is_some_and(|cfg| {
            cfg.map
                .keys()
                .any(|k| k.starts_with("install.") || k.starts_with("build."))
        });
    match configured {
        true => " `vc-x1 install` installs the artifact".to_string(),
        false => String::new(),
    }
}

/// The refusal of a target that is not where its `origin` is, naming
/// the way on by how the two stand: one only behind is brought up by
/// a sync, one only ahead, or absent there, is published by a push,
/// and two that diverged are the user's to reconcile first.
pub(crate) fn not_at_origin(
    repo: &Path,
    label: &str,
    onto: &str,
    local: &str,
    remote: Option<&str>,
    plain: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let short = |id: &str| -> String {
        id.chars()
            .take(crate::desc_helpers::DEFAULT_ID_LEN)
            .collect()
    };
    let scope = if plain { "p" } else { "b" };
    // `-r` publishes the bookmark where it is, which `--bookmark` does
    // only when it sits at the work `@-`.
    let push = format!("`vc-x1 push -s {scope} -r {onto}` publishes it first");
    let Some(remote) = remote else {
        return Ok(format!(
            "{label}: '{onto}' is at {} locally and is not on origin, so a Land has nothing \
             published to fast-forward from: {push}",
            short(local)
        ));
    };
    let at = format!(
        "'{onto}' is at {} locally and {} on origin",
        short(local),
        short(remote)
    );
    Ok(if jj::matches(repo, &format!("{remote} & ::{local}"))? {
        format!("{label}: {at}, ahead with commits not yet published: {push}")
    } else if jj::matches(repo, &format!("{local} & ::{remote}"))? {
        format!(
            "{label}: {at}, behind with commits not yet fetched: `vc-x1 sync -s {scope}` brings it \
             up to date"
        )
    } else {
        format!(
            "{label}: {at}, and the two have diverged, each with commits the other lacks: \
             reconcile them first, a sync refusing until then"
        )
    })
}

/// A Land planned before anything is made: the `--onto` bookmark and where it is,
/// and the cycle's own bookmark, deleted after, `skip` keeping a side.
#[derive(Debug)]
pub(crate) struct LandPlan {
    /// How messages name the command, e.g. `commit --land`.
    pub label: &'static str,
    pub onto: String,
    pub onto_cid: String,
    pub topic: Option<String>,
    pub skip: Option<Skip>,
    /// The bookmarks strictly between the `--onto` bookmark and the tip, deleted
    /// after the cycle's when asked, `skip` keeping the same side.
    pub intermediate: Vec<String>,
    /// The branch's other bookmarks, merged by the Land and left as
    /// they are, named after it.
    pub merged: Vec<String>,
    /// Publish the Land and delete on `origin` too. [`plan`] leaves it
    /// off, for the caller to set.
    pub push: bool,
}

/// Plan a Land of the branch read up to `tip` onto `onto`, refusing
/// before anything is made. `bookmark` is the cycle's, deleted after,
/// and with `delete_intermediate` the bookmarks strictly between too.
pub(crate) fn plan(
    repo: &Path,
    onto: String,
    tip: &str,
    bookmark: Option<&str>,
    skip: Option<Skip>,
    delete_intermediate: bool,
    plain: bool,
) -> Result<LandPlan, Box<dyn std::error::Error>> {
    if !jj::local_bookmark_exists(repo, &onto)? {
        return Err(
            format!("{LABEL}: '{onto}' is not a local bookmark, and Land moves one").into(),
        );
    }
    let onto_cid = jj::cid_of(repo, &onto)?;
    let remote = jj::remote_bookmark_targets(repo, "origin")?
        .get(&onto)
        .cloned();
    if remote.as_deref() != Some(onto_cid.as_str()) {
        return Err(not_at_origin(repo, LABEL, &onto, &onto_cid, remote.as_deref(), plain)?.into());
    }
    if !jj::matches(repo, &format!("{onto_cid} & ::({tip})"))? {
        return Err(format!(
            "{LABEL}: the branch does not descend from '{onto}' at {onto_cid}, so \
             '{onto}' moved since the fork and the Land would carry a tree no validation saw: rebase \
             the branch onto it first"
        )
        .into());
    }
    let topic = bookmark.map(str::to_string);
    let others = |revset: &str| -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut found = jj::local_bookmarks_at(repo, revset)?;
        found.retain(|b| *b != onto && Some(b) != topic.as_ref());
        Ok(found)
    };
    let between = others(&format!("(({onto_cid}..({tip})) ~ ({tip})) & bookmarks()"))?;
    let mut merged = others(&format!("({tip}) & bookmarks()"))?;
    let intermediate = if delete_intermediate {
        between
    } else {
        merged.splice(0..0, between);
        Vec::new()
    };
    info!(
        "{LABEL}: land onto '{onto}' at {}, the cycle's bookmark {}",
        short(&onto_cid),
        topic
            .as_deref()
            .map(|t| format!("'{t}'"))
            .unwrap_or_else(|| "none".into())
    );
    Ok(LandPlan {
        label: LABEL,
        onto,
        onto_cid,
        topic,
        skip,
        intermediate,
        merged,
        push: false,
    })
}

/// The agent commit a Land makes: its description, the target's text
/// with one `ochid:` per commit from `onto_cid` to `target` that names
/// the agent `@`, oldest first, in the form each took. `None` when none
/// does, the cycle's pairs being made already. A commit above `target`
/// naming it is refused, since it would be left behind by the commit.
fn land_agent_desc(
    pair: &crate::pair::Pair,
    label: &str,
    onto_cid: &str,
    target: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let work = pair.work.as_path();
    let agent_at = jj::chid_of(&pair.agent, "@")?;
    let names_agent_at = |rev: &str| -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(extract_ochid_from_desc(&jj::desc_of(work, rev)?)
            .filter(|ochid| extract_bare_id(ochid) == agent_at))
    };
    for rev in jj::cids_of(work, &format!("({target})..@-"))? {
        if names_agent_at(&rev)?.is_some() {
            return Err(format!(
                "{label}: {rev}, above what is landed, also names the agent @ {agent_at}, which a \
                 Land commits, so land the line up to it"
            )
            .into());
        }
    }
    let mut values = Vec::new();
    for rev in jj::cids_of(work, &format!("{onto_cid}..({target})"))? {
        if names_agent_at(&rev)?.is_some() {
            values.push(jj::chid_of(work, &rev)?);
        }
    }
    if values.is_empty() {
        return Ok(None);
    }
    let desc = jj::desc_of(work, target)?;
    let text = without_trailer_lines(&desc).collect::<Vec<_>>().join("\n");
    Ok(Some(with_ochids(&text, repo_url_for(work)?, &values)))
}

/// Land what `land` plans onto `target`, named `what` in messages:
/// refuse a conflicted target, commit the agent `@` when the cycle's
/// commits name it, fast-forward `onto` to it, delete the cycle's
/// bookmark, `--skip` keeping a side, and with `land.push` publish it,
/// `scope` saying which repos, and delete on `origin` too. A dry run
/// makes every check and says each step, and does none.
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
        onto,
        onto_cid,
        topic,
        skip,
        intermediate,
        merged,
        push,
    } = land;
    let push = *push;
    // Local, the remote side is left whatever is asked, so `--skip
    // local`, which would delete only the remote, deletes nothing.
    let keep = match (skip, push) {
        (Some(Skip::Both), _) | (Some(Skip::Local), false) => None,
        (Some(Skip::Local), true) => Some(Some(crate::delete_bookmark::Keep::Local)),
        (Some(Skip::Remote), _) | (None, false) => Some(Some(crate::delete_bookmark::Keep::Remote)),
        (None, true) => Some(None),
    };
    // Checks run in both modes, so a dry run refuses what a Land would.
    if jj::matches(repo, &format!("({target}) & conflicts()"))? {
        return Err(format!(
            "{label}: {what} is conflicted, so it is not landed: resolve it, then \
             land it with `vc-x1 move-bookmark --bookmark {onto} --target {target}` and \
             `vc-x1 push`"
        )
        .into());
    }
    // Each step's line, then the step, which a dry run leaves undone.
    let tag = if dry { " [dry-run]" } else { "" };
    // The work @-'s pair may be unfinished, which push finishes. A
    // commit below it has a finished pair, so its `--onto` bookmark goes alone.
    let at_work_head = jj::matches(repo, &format!("({target}) & @-"))?;
    let pair = match scope {
        RepoScope::Por => None,
        RepoScope::Sides(_) => Some(crate::pair::resolve_pair(Some(repo))?),
    };
    let agent_desc = match &pair {
        Some(pair) => land_agent_desc(pair, label, onto_cid, target)?,
        None => None,
    };
    if let (Some(pair), Some(desc)) = (&pair, &agent_desc) {
        info!(
            "{label}:{tag} commit the agent @ {} as the partner, in {}:\n{desc}",
            jj::chid_of(&pair.agent, "@")?,
            pair.agent.display()
        );
        if !dry {
            let snapshot = crate::pair::Snapshot::take(&pair.agent)?;
            let made = ctx
                .session(&pair.agent)?
                .commit(desc)
                .and_then(|_| ctx.session(&pair.agent)?.bookmark_set("main", "@-"));
            if let Err(e) = made {
                snapshot.restore(label, e.as_ref());
                return Err(e);
            }
        }
    }
    // By change id, since restamping a commit's trailers changes its SHA.
    let target_chid = jj::chid_of(repo, target)?;
    if let Some(pair) = &pair {
        crate::trailers::update(
            ctx,
            pair,
            &crate::trailers::Update {
                label,
                revset: &format!("{onto_cid}..({target})"),
                add: false,
                dry,
            },
        )?;
    }
    info!(
        "{label}:{tag} fast-forward '{onto}' from {} to {what}",
        short(onto_cid)
    );
    if !dry {
        ctx.session(repo)?.bookmark_set(onto, &target_chid)?;
    }
    if push && at_work_head {
        info!("{label}:{tag} push '{onto}', the work @-, with its pair");
        if !dry {
            crate::push::push(
                ctx,
                &crate::push::PushParams {
                    revision: None,
                    bookmark: Some(onto.clone()),
                    repo: Some(repo.to_path_buf()),
                    scope: scope.clone(),
                    step: false,
                    dry_run: false,
                    yes: true,
                    forced: crate::common::ForcedPush::Refuse,
                },
            )?;
        }
    } else if push {
        info!("{label}:{tag} push '{onto}'");
        if !dry {
            crate::common::verify_tracking(repo, onto)?;
            ctx.session(repo)?
                .git_push_bookmark(onto, crate::jj::Force::Refuse)?;
        }
        if let Some(pair) = &pair {
            info!("{label}:{tag} publish the agent main");
            // A published agent `@-` takes no fold, which would force the
            // push, so the journal rides the next agent commit.
            if !dry && (agent_desc.is_some() || crate::pair::is_published(&pair.agent, "@-")?) {
                crate::push::publish_agent_main(ctx, &pair.agent, crate::jj::Force::Refuse)?;
            } else if !dry {
                crate::squash_push::squash_push(
                    ctx,
                    &crate::squash_push::SquashPushParams {
                        repo: pair.agent.clone(),
                        squash: SquashSpec {
                            source: "@".to_string(),
                            target: "@-".to_string(),
                        },
                        bookmark: "main".to_string(),
                        at_rest: false,
                        yes: true,
                        dry_run: false,
                        forced: crate::common::ForcedPush::Refuse,
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
            info!("{label}:{tag} delete '{b}', between '{onto}' and {what}");
            if !dry {
                crate::delete_bookmark::delete_bookmark(ctx, repo, b, keep, false)?;
            }
        }
    }
    if !merged.is_empty() {
        info!(
            "{label}:{tag} merged into '{onto}' and left as they are, yours to keep or \
             delete: {}",
            merged.join(", ")
        );
    }
    if !dry && push {
        info!(
            "{label}: landed on '{onto}', the undo being a revert, '{onto}' having moved \
             from {}.{}",
            short(onto_cid),
            install_hint(repo)
        );
    } else if !dry {
        let flag = if pair.is_some() { "b" } else { "p" };
        let publish = if at_work_head {
            format!("`vc-x1 push -s {flag} --bookmark {onto}`")
        } else if pair.is_some() {
            format!("`vc-x1 push -s w -r {onto}` and `vc-x1 push -s a`")
        } else {
            format!("`vc-x1 push -s p -r {onto}`")
        };
        info!(
            "{label}: landed on '{onto}' on this machine, '{onto}' having moved from \
             {}, and {publish} publishes it.{}",
            short(onto_cid),
            install_hint(repo)
        );
        for t in topic.iter().chain(intermediate) {
            if jj::remote_bookmark_targets(repo, "origin")?.contains_key(t) && keep.is_some() {
                info!(
                    "{label}: '{t}' is still on origin, and `vc-x1 delete-bookmark -s {} \
                     --bookmark {t} --skip local` deletes it",
                    if pair.is_some() { "w" } else { "p" }
                );
            }
        }
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

    /// `vc-x1 land <args> --push` in the fixture's work repo, parsed.
    fn land_params(fx: &Fixture, args: &[&str]) -> LandParams {
        let mut args = args.to_vec();
        args.push("--push");
        local_land_params(fx, &args)
    }

    /// The params of a Land in the fixture's work repo, from the flags
    /// `cycle --land` passes on, written as `vc-x1 land`'s were.
    fn local_land_params(fx: &Fixture, args: &[&str]) -> LandParams {
        let mut p = LandParams {
            repo: fx.work.clone(),
            scope: RepoScope::Sides(Scope(vec![Side::Work, Side::Bot])),
            source: None,
            onto: String::new(),
            skip: None,
            delete_intermediate: false,
            push: false,
            dry_run: false,
        };
        let mut args = args.iter();
        while let Some(flag) = args.next() {
            let mut value = || args.next().expect("a value").to_string();
            match *flag {
                "-s" => {
                    if value() == "p" {
                        p.scope = RepoScope::Por;
                    }
                }
                "--source" => p.source = Some(value()),
                "--onto" => p.onto = value(),
                "--skip" => {
                    p.skip = Some(match value().as_str() {
                        "local" => Skip::Local,
                        "remote" => Skip::Remote,
                        _ => Skip::Both,
                    })
                }
                "--delete-intermediate-bookmarks" => p.delete_intermediate = true,
                "--push" => p.push = true,
                "--dry-run" => p.dry_run = true,
                other => panic!("unknown flag {other}"),
            }
        }
        p
    }

    /// A cycle pushed on `feat-a-cycle`: its opening and one rung.
    /// Returns the rung's commit id.
    fn pushed_cycle(fx: &Fixture) -> String {
        fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
        // The bookmark `vc-x1 open` makes first, on the work `@`.
        jj_ok(&fx.work, &["bookmark", "create", "feat-a-cycle", "-r", "@"]);
        vc_commit(
            fx,
            &[
                "-s",
                "b",
                "--push",
                "--title",
                "feat: a cycle opening",
                "--body",
                "Why.",
            ],
        );
        fs::write(fx.work.join("rung.txt"), "rung\n").expect("write");
        vc_commit(
            fx,
            &[
                "-s",
                "b",
                "--push",
                "--title",
                "feat: a rung",
                "--body",
                "The rung.",
            ],
        );
        commit_id(&fx.work, "feat-a-cycle")
    }

    /// A cycle held local: an opening and a rung, each the work half
    /// alone, so both name the agent `@`. Their change ids, oldest first.
    fn local_cycle(fx: &Fixture) -> Vec<String> {
        let mut made = Vec::new();
        fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
        // The bookmark `vc-x1 open` makes first, on the work `@`.
        jj_ok(&fx.work, &["bookmark", "create", "feat-a-cycle", "-r", "@"]);
        vc_commit(
            fx,
            &[
                "-s",
                "w",
                "--title",
                "feat: a cycle opening",
                "--body",
                "Why.",
            ],
        );
        made.push(chid(&fx.work, "@-"));
        fs::write(fx.work.join("rung.txt"), "rung\n").expect("write");
        vc_commit(
            fx,
            &["-s", "w", "--title", "feat: a rung", "--body", "The rung."],
        );
        made.push(chid(&fx.work, "@-"));
        made
    }

    /// A local cycle lands on the machine: the agent `@` committed with
    /// one `ochid:` per work commit, `main` moved, the bookmark gone,
    /// nothing on origin, and its dry run doing none of it. `push` then
    /// publishes both, and the journal stays out of the agent commit.
    #[test]
    fn a_local_cycle_lands_locally_and_push_publishes_it() {
        let fx = Fixture::new("land-local");
        let rungs = local_cycle(&fx);
        let main_origin = commit_id(&fx.work, "main@origin");
        let agent_origin = commit_id(&fx.bot, "main@origin");
        let agent_at = chid(&fx.bot, "@");

        let args = ["-s", "b", "--onto", "main"];
        let mut dry = args.to_vec();
        dry.push("--dry-run");
        land(&mut test_ctx(), &local_land_params(&fx, &dry)).expect("planned");
        assert_eq!(chid(&fx.bot, "@"), agent_at, "a dry run commits nothing");
        assert_eq!(commit_id(&fx.work, "main"), main_origin);

        land(&mut test_ctx(), &local_land_params(&fx, &args)).expect("landed");
        assert_eq!(chid(&fx.work, "main"), rungs[1]);
        let tip = commit_id(&fx.work, "main");
        assert_eq!(commit_id(&fx.work, "main@origin"), main_origin);
        assert!(!jj::local_bookmark_exists(&fx.work, "feat-a-cycle").expect("local"));
        assert_eq!(
            chid(&fx.bot, "@-"),
            agent_at,
            "the change the trailers name"
        );
        let desc = crate::test_helpers::description(&fx.bot, "@-");
        assert!(desc.starts_with("feat: a rung\n\nThe rung.\n"), "{desc}");
        assert_eq!(crate::desc_helpers::extract_ochids(&desc), rungs, "{desc}");
        assert_eq!(commit_id(&fx.bot, "main"), commit_id(&fx.bot, "@-"));
        assert_eq!(commit_id(&fx.bot, "main@origin"), agent_origin);

        fs::write(fx.bot.join("journal.jsonl"), "{\"later\":1}\n").expect("write");
        crate::push::push(
            &mut test_ctx(),
            &crate::push::PushParams {
                revision: None,
                bookmark: Some("main".to_string()),
                repo: Some(fx.work.clone()),
                scope: RepoScope::Sides(Scope(vec![Side::Work, Side::Bot])),
                step: false,
                dry_run: false,
                yes: true,
                forced: crate::common::ForcedPush::Ask,
            },
        )
        .expect("published");
        assert_eq!(commit_id(&fx.work, "main@origin"), tip);
        // The Land's agent commit was not yet published, so the journal
        // folds into it, the change id the trailers name kept, and both
        // repos are left clean.
        assert_eq!(chid(&fx.bot, "main@origin"), agent_at);
        assert!(
            jj::is_empty(&fx.bot, "@").expect("empty"),
            "the journal folded"
        );
    }

    /// With hosted origins a Land leaves the rungs' trailers as made,
    /// `orepo:` and `ochid:` naming no SHA, so no rung is rewritten, and
    /// its agent commit names the work repo and each rung. Its dry run
    /// changes nothing.
    #[test]
    fn a_land_rewrites_no_rung() {
        let fx = Fixture::new("land-orepo");
        let (work_url, agent_url) = (
            "https://github.com/o/r",
            "https://github.com/o/r.agent-session",
        );
        crate::test_helpers::git_ok(&fx.work, &["remote", "set-url", "origin", work_url]);
        crate::test_helpers::git_ok(&fx.bot, &["remote", "set-url", "origin", agent_url]);
        let rungs = local_cycle(&fx);
        let agent_at = chid(&fx.bot, "@");
        let shas: Vec<String> = rungs.iter().map(|c| commit_id(&fx.work, c)).collect();

        let args = ["-s", "b", "--onto", "main"];
        let mut dry = args.to_vec();
        dry.push("--dry-run");
        land(&mut test_ctx(), &local_land_params(&fx, &dry)).expect("planned");
        land(&mut test_ctx(), &local_land_params(&fx, &args)).expect("landed");
        for (c, sha) in rungs.iter().zip(&shas) {
            let desc = jj::desc_of(&fx.work, c).expect("desc");
            assert!(
                desc.ends_with(&format!("\n\norepo: {agent_url}\nochid: {agent_at}")),
                "{desc}"
            );
            assert_eq!(&commit_id(&fx.work, c), sha, "the rung is not rewritten");
        }
        let agent_desc = jj::desc_of(&fx.bot, "@-").expect("desc");
        assert!(
            agent_desc.ends_with(&format!(
                "\n\norepo: {work_url}\nochid: {}\nochid: {}",
                rungs[0], rungs[1]
            )),
            "{agent_desc}"
        );
        assert_eq!(chid(&fx.work, "main"), rungs[1]);
        assert!(!jj::local_bookmark_exists(&fx.work, "feat-a-cycle").expect("local"));
        assert!(jj::is_empty(&fx.work, "@").expect("empty"));
    }

    /// An older rung's `ocid:` gives way to `orepo:` at the Land, a pushed
    /// one's included, the remote's copy going stale.
    #[test]
    fn an_older_rungs_links_become_orepo() {
        let fx = Fixture::new("land-older-links");
        local_cycle(&fx);
        let placeholder = "https://h/o/r.agent-session/commit/xxxx";
        for rev in ["@--", "@-"] {
            let text = crate::desc_helpers::without_trailer_lines(
                &crate::test_helpers::description(&fx.work, rev),
            )
            .collect::<Vec<_>>()
            .join("\n");
            let ochid = crate::desc_helpers::extract_ochid_from_desc(
                &crate::test_helpers::description(&fx.work, rev),
            )
            .expect("an ochid");
            let desc = format!(
                "{}\n\nochid: {ochid}\nocid: {placeholder}\n",
                text.trim_end()
            );
            jj_ok(&fx.work, &["describe", "-r", rev, "-m", &desc]);
        }
        jj_ok(&fx.work, &["git", "push", "--named", "shared=@--"]);
        crate::test_helpers::git_ok(
            &fx.bot,
            &["remote", "set-url", "origin", "https://h/o/r.agent-session"],
        );
        land(
            &mut test_ctx(),
            &local_land_params(
                &fx,
                &["-s", "b", "--source", "feat-a-cycle", "--onto", "main"],
            ),
        )
        .expect("landed");
        for rev in ["main-", "main"] {
            let desc = crate::test_helpers::description(&fx.work, rev);
            assert!(
                desc.contains("\n\norepo: https://h/o/r.agent-session\nochid: "),
                "{rev}: {desc}"
            );
            assert!(!desc.contains("ocid:"), "{rev}: {desc}");
        }
    }

    /// Landing below a commit that also names the agent `@` is refused
    /// before anything moves, since the commit would leave it behind.
    #[test]
    fn a_land_below_a_commit_naming_the_agent_at_is_refused() {
        let fx = Fixture::new("land-local-below");
        local_cycle(&fx);
        let main = commit_id(&fx.work, "main");
        let agent_at = chid(&fx.bot, "@");
        let err = land(
            &mut test_ctx(),
            &local_land_params(&fx, &["-s", "b", "--source", "@--", "--onto", "main"]),
        )
        .expect_err("refused")
        .to_string();
        assert!(err.contains("also names the agent @"), "{err}");
        assert_eq!(commit_id(&fx.work, "main"), main);
        assert_eq!(chid(&fx.bot, "@"), agent_at);
    }

    /// `--skip local` deletes only the remote, so a local Land, which
    /// touches no remote, deletes nothing, and a published cycle's
    /// bookmark stays on origin until it is deleted there.
    #[test]
    fn a_local_land_leaves_the_remote_bookmark() {
        let fx = Fixture::new("land-local-remote");
        pushed_cycle(&fx);
        land(
            &mut test_ctx(),
            &local_land_params(&fx, &["-s", "b", "--onto", "main", "--skip", "local"]),
        )
        .expect("landed");
        assert!(jj::local_bookmark_exists(&fx.work, "feat-a-cycle").expect("local"));
        assert!(on_origin(&fx.work, "feat-a-cycle"));

        let fx = Fixture::new("land-local-remote-kept");
        pushed_cycle(&fx);
        land(
            &mut test_ctx(),
            &local_land_params(&fx, &["-s", "b", "--onto", "main"]),
        )
        .expect("landed");
        assert!(!jj::local_bookmark_exists(&fx.work, "feat-a-cycle").expect("local"));
        assert!(on_origin(&fx.work, "feat-a-cycle"));
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

    /// A source below the work `@-` lands by its `--onto` bookmark alone: `main` at
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
        let p = LandParams {
            repo: fx.work.clone(),
            scope: RepoScope::Por,
            source: Some("feat-a-cycle".to_string()),
            onto: "main".to_string(),
            skip: None,
            delete_intermediate: false,
            push: true,
            dry_run: false,
        };
        land(&mut test_ctx(), &p).expect("landed");
        assert_eq!(commit_id(&fx.work, "main"), tip);
        assert_eq!(commit_id(&fx.work, "main@origin"), tip);
        assert!(!on_origin(&fx.work, "feat-a-cycle"), "deleted on origin");
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

    /// `--source` left out is the one local bookmark besides `--onto`,
    /// and with several it is refused with their names.
    #[test]
    fn the_source_defaults_to_the_one_other_bookmark() {
        let fx = Fixture::new("land-default-source");
        let tip = pushed_cycle(&fx);
        land(
            &mut test_ctx(),
            &land_params(&fx, &["-s", "b", "--onto", "main"]),
        )
        .expect("landed");
        assert_eq!(commit_id(&fx.work, "main@origin"), tip);
        assert!(!on_origin(&fx.work, "feat-a-cycle"), "deleted on origin");
        let err = land(
            &mut test_ctx(),
            &land_params(&fx, &["-s", "b", "--onto", "main"]),
        )
        .expect_err("refused")
        .to_string();
        assert!(err.contains("no local bookmark besides 'main'"), "{err}");

        let fx = Fixture::new("land-default-source-several");
        stacked_cycle(&fx);
        let main = commit_id(&fx.work, "main");
        let err = land(
            &mut test_ctx(),
            &land_params(&fx, &["-s", "b", "--onto", "main"]),
        )
        .expect_err("refused")
        .to_string();
        assert!(
            err.contains("several local bookmarks besides 'main', earlier, feat-a-cycle, mine"),
            "{err}"
        );
        assert_eq!(commit_id(&fx.work, "main"), main, "main stays");
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
        assert!(
            err.contains("ahead") && err.contains("vc-x1 push -s b -r main"),
            "{err}"
        );
        assert_eq!(commit_id(&fx.work, "main"), moved, "main stays");
    }

    /// The Land names `vc-x1 install` only where the workspace's config
    /// has an install or a build, the artifact being the project's.
    #[test]
    fn the_install_is_named_only_when_configured() {
        let fx = Fixture::new("land-install-hint");
        assert_eq!(install_hint(&fx.work), "", "a fresh workspace has none");
        let path = fx.work.join(".vc-config.md");
        let mut cfg = fs::read_to_string(&path).expect("read the config");
        cfg.push_str("\n```toml\n[install]\ndir = \"~/.cargo/bin\"\n```\n");
        fs::write(&path, cfg).expect("write the config");
        assert!(install_hint(&fx.work).contains("`vc-x1 install`"));
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
            onto: "main".to_string(),
            onto_cid: main.clone(),
            topic: None,
            skip: None,
            intermediate: Vec::new(),
            merged: Vec::new(),
            push: true,
        };
        (fx, plan, main)
    }

    /// A conflicted closing is not landed: `main` stays where it is
    /// and the error names the commands that land it after a resolve.
    /// `commit` refuses a conflicted repo up front and a closing onto a
    /// bookmark the branch descends from cannot conflict, so the guard is
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

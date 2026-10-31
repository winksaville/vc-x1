//! Land: make a finished cycle permanent on its trunk. The agent `@` is
//! committed when the cycle's commits name it, the trunk is
//! fast-forwarded to the cycle's last commit, and the cycle's bookmark
//! is deleted, all on the machine, every check made before anything
//! moves. `--push` also publishes, the agent repo first, and deletes
//! the bookmark on `origin`, and without it `vc-x1 push` publishes.
//!
//! - [`plan`] refuses before anything is made: the trunk is a local
//!   bookmark at its `origin` position, so the sync a Land begins with
//!   has nothing to bring, and the branch descends from it, so the tree
//!   landed is the reviewed one. The branch's other bookmarks are named
//!   as merged and left as they are.
//! - [`run`] lands what the plan names: a conflicted target is refused,
//!   the agent `@` is committed with one `ochid:` per work commit naming
//!   it, oldest first, each placeholder `ocid:` gets its SHA, the trunk
//!   is fast-forwarded, and the cycle's bookmark is deleted, `--skip`
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
//! - The publish follows where the target is: the work `@-`, whose pair
//!   may be unfinished, goes through `push`, which finishes it, and a
//!   commit below it, whose pair is finished, by its trunk pushed alone
//!   and the agent `main` published.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info, warn};

use crate::commit::Skip;
use crate::context::Context;
use crate::desc_helpers::{
    extract_bare_id, extract_ochid_from_desc, extract_ocid_from_desc, is_url_form,
    ochid_value_in_form, ocid_with_sha, with_ochids, with_ocid_value, without_trailer_lines,
};
use crate::jj;
use crate::options_flags::dry_run::DryRunFlag;
use crate::options_flags::scope::{RepoScope, repo_scope_parser};
use crate::options_flags::squash::SquashSpec;
use crate::subcommand::SubcommandRunner;

/// Land a cycle whose last commit is made: `--onto` fast-forwarded to
/// `--source` and the cycle's bookmark deleted, locally unless `--push`.
#[derive(Args, Debug)]
pub struct LandArgs {
    /// The cycle's last commit: a bookmark, the cycle's, or a revision,
    /// whose current bookmark, else its nearest parent's above `--onto`,
    /// is the cycle's [default: the one local bookmark besides `--onto`]
    #[arg(long, value_name = "BM|REV")]
    pub source: Option<String>,

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

    /// Publish the Land, the agent repo first, and delete the cycle's
    /// bookmark on `origin` as well, a Land otherwise staying on the
    /// machine
    #[arg(long)]
    pub push: bool,

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
    /// `None` is the one local bookmark besides `onto`.
    pub source: Option<String>,
    pub onto: String,
    pub skip: Option<Skip>,
    pub delete_intermediate: bool,
    pub push: bool,
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
            push: a.push,
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
    if !jj::local_bookmark_exists(repo, &params.onto)? {
        return Err(format!(
            "land: '{}' is not a local bookmark, and Land moves one",
            params.onto
        )
        .into());
    }
    let source = match &params.source {
        Some(s) => s.clone(),
        None => default_source(repo, &params.onto)?,
    };
    let tip = resolve_tip(repo, "land", &source)?;
    if jj::cid_of(repo, &params.onto)? == tip {
        return Err(format!(
            "land: '{}' is at '{source}' already, so there is nothing to land",
            params.onto
        )
        .into());
    }
    let bookmark = cycle_bookmark(repo, "land", "--source", &source, &params.onto, &tip)?;
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
        &LandPlan {
            push: params.push,
            ..plan
        },
        &tip,
        &format!("'{source}'"),
        params.dry_run,
    )
}

/// `--source` left out: the one local bookmark besides `onto`, refused
/// with the list when there are several, since naming one settles it.
pub(crate) fn default_source(
    repo: &Path,
    onto: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut found = jj::local_bookmarks_at(repo, "bookmarks()")?;
    found.retain(|b| b != onto);
    match found.as_slice() {
        [one] => {
            info!("land: --source is '{one}', the one local bookmark besides '{onto}'");
            Ok(one.clone())
        }
        [] => Err(format!(
            "land: there is no local bookmark besides '{onto}', so there is nothing to land"
        )
        .into()),
        several => Err(format!(
            "land: there are several local bookmarks besides '{onto}', {}, so which is the \
             source is not known: name it, --source <BM|REV>",
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
    /// Publish the Land and delete on `origin` too. [`plan`] leaves it
    /// off, for the caller to set.
    pub push: bool,
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
        push: false,
    })
}

/// The agent commit a Land makes: its description, the target's text
/// with one `ochid:` per commit from the trunk to `target` that names
/// the agent `@`, oldest first, in the form each took. `None` when none
/// does, the cycle's pairs being made already. A commit above `target`
/// naming it is refused, since it would be left behind by the commit.
fn land_agent_desc(
    pair: &crate::pair::Pair,
    label: &str,
    trunk_cid: &str,
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
    for rev in jj::cids_of(work, &format!("{trunk_cid}..({target})"))? {
        if let Some(ochid) = names_agent_at(&rev)? {
            values.push(ochid_value_in_form(
                is_url_form(&ochid),
                work,
                &jj::chid_of(work, &rev)?,
            )?);
        }
    }
    if values.is_empty() {
        return Ok(None);
    }
    let desc = jj::desc_of(work, target)?;
    let text = without_trailer_lines(&desc).collect::<Vec<_>>().join("\n");
    Ok(Some(with_ochids(&text, &values)))
}

/// What [`set_ocids`] reads: the commits from the trunk to `target`, and
/// `partner_rev`, the agent commit they name, `@` in a dry run that has
/// not committed it.
struct LandOcids<'a> {
    label: &'a str,
    trunk_cid: &'a str,
    target: &'a str,
    partner_rev: &'a str,
    dry: bool,
}

/// Set the placeholder `ocid:` of each commit from the trunk to `target`
/// that names the agent commit to that commit's SHA, oldest first, the
/// tree checked unchanged. A published commit is left with a warning,
/// since this would rewrite it. Finding nothing is no error, so a Land
/// rerun after a failure finishes what it left.
fn set_ocids(
    ctx: &mut Context,
    pair: &crate::pair::Pair,
    o: &LandOcids,
) -> Result<(), Box<dyn std::error::Error>> {
    let LandOcids { label, dry, .. } = *o;
    let work = pair.work.as_path();
    let partner = jj::chid_of(&pair.agent, o.partner_rev)?;
    let sha = jj::cid_of(&pair.agent, o.partner_rev)?;
    let mut todo = Vec::new();
    for rev in jj::cids_of(work, &format!("{}..({})", o.trunk_cid, o.target))? {
        let desc = jj::desc_of(work, &rev)?;
        let names = extract_ochid_from_desc(&desc).is_some_and(|c| extract_bare_id(&c) == partner);
        let value = extract_ocid_from_desc(&desc).and_then(|v| ocid_with_sha(&v, &sha));
        let (true, Some(value)) = (names, value) else {
            continue;
        };
        let chid = jj::chid_of(work, &rev)?;
        if crate::pair::is_published(work, &rev)? {
            warn!("{label}: {chid} is published, so its `ocid:` keeps its placeholder");
            continue;
        }
        todo.push((
            chid,
            with_ocid_value(&desc, &value),
            jj::tree_of(work, &rev)?,
        ));
    }
    if dry {
        for (chid, _, _) in &todo {
            info!("{label}: [dry-run] set the `ocid:` of {chid} to the agent commit's SHA");
        }
        return Ok(());
    }
    let snapshot = crate::pair::Snapshot::take(work)?;
    for (chid, desc, tree) in &todo {
        info!("{label}: set the `ocid:` of {chid} to {sha}");
        let stamped = ctx.session(work)?.describe(chid, desc).and_then(|_| {
            match jj::tree_of(work, chid)? == *tree {
                true => Ok(()),
                false => Err(format!("{label}: setting the `ocid:` changed {chid}'s tree").into()),
            }
        });
        if let Err(e) = stamped {
            snapshot.restore(label, e.as_ref());
            return Err(e);
        }
    }
    Ok(())
}

/// Land what `land` plans onto `target`, named `what` in messages:
/// refuse a conflicted target, commit the agent `@` when the cycle's
/// commits name it, fast-forward the trunk to it, delete the cycle's
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
        trunk,
        trunk_cid,
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
    let pair = match scope {
        RepoScope::Por => None,
        RepoScope::Sides(_) => Some(crate::pair::resolve_pair(Some(repo))?),
    };
    let agent_desc = match &pair {
        Some(pair) => land_agent_desc(pair, label, trunk_cid, target)?,
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
    // By change id, since setting a commit's `ocid:` changes its SHA.
    let target_chid = jj::chid_of(repo, target)?;
    if let Some(pair) = &pair {
        set_ocids(
            ctx,
            pair,
            &LandOcids {
                label,
                trunk_cid,
                target,
                partner_rev: if dry && agent_desc.is_some() {
                    "@"
                } else {
                    "@-"
                },
                dry,
            },
        )?;
    }
    info!("{label}:{tag} fast-forward '{trunk}' from {trunk_cid} to {what}");
    if !dry {
        ctx.session(repo)?.bookmark_set(trunk, &target_chid)?;
    }
    if push && at_work_head {
        info!("{label}:{tag} push '{trunk}', the work @-, with its pair");
        if !dry {
            crate::push::push(
                ctx,
                &crate::push::PushParams {
                    revision: None,
                    bookmark: Some(trunk.clone()),
                    repo: Some(repo.to_path_buf()),
                    scope: scope.clone(),
                    step: false,
                    dry_run: false,
                    yes: true,
                },
            )?;
        }
    } else if push {
        info!("{label}:{tag} push '{trunk}'");
        if !dry {
            crate::common::verify_tracking(repo, trunk)?;
            ctx.session(repo)?.git_push_bookmark(trunk)?;
        }
        if let Some(pair) = &pair {
            info!("{label}:{tag} publish the agent main");
            if !dry && agent_desc.is_some() {
                crate::push::publish_agent_main(ctx, &pair.agent)?;
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
    if !dry && push {
        info!(
            "{label}: landed on '{trunk}', the undo being a revert, the trunk having moved \
             from {trunk_cid}. `vc-x1 install` installs the artifact"
        );
    } else if !dry {
        let flag = if pair.is_some() { "b" } else { "p" };
        let publish = if at_work_head {
            format!("`vc-x1 push -s {flag} --bookmark {trunk}`")
        } else if pair.is_some() {
            format!("`vc-x1 push -s w -r {trunk}` and `vc-x1 push -s a`")
        } else {
            format!("`vc-x1 push -s p -r {trunk}`")
        };
        info!(
            "{label}: landed on '{trunk}' on this machine, the trunk having moved from \
             {trunk_cid}, and {publish} publishes it. `vc-x1 install` installs the artifact"
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

    /// `vc-x1 land <args>` in the fixture's work repo, parsed.
    fn local_land_params(fx: &Fixture, args: &[&str]) -> LandParams {
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
                "--push",
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
        vc_commit(
            fx,
            &[
                "-s",
                "w",
                "--opening",
                "--title",
                "feat: a cycle",
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
    /// one `ochid:` per work commit, the trunk moved, the bookmark gone,
    /// nothing on origin, and its dry run doing none of it. `push` then
    /// publishes both, and the journal stays out of the agent commit.
    #[test]
    fn a_local_cycle_lands_locally_and_push_publishes_it() {
        let fx = Fixture::new("land-local");
        let rungs = local_cycle(&fx);
        let tip = commit_id(&fx.work, "feat-a-cycle");
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
        assert_eq!(commit_id(&fx.work, "main"), tip);
        assert_eq!(commit_id(&fx.work, "main@origin"), main_origin);
        assert!(!jj::local_bookmark_exists(&fx.work, "feat-a-cycle").expect("local"));
        assert_eq!(
            chid(&fx.bot, "@-"),
            agent_at,
            "the change the trailers name"
        );
        let desc = crate::test_helpers::description(&fx.bot, "@-");
        assert!(desc.starts_with("feat: a rung\n\nThe rung.\n"), "{desc}");
        assert_eq!(
            crate::desc_helpers::extract_ochids(&desc),
            rungs.iter().map(|c| format!("/{c}")).collect::<Vec<_>>(),
            "{desc}"
        );
        assert_eq!(commit_id(&fx.bot, "main"), commit_id(&fx.bot, "@-"));
        assert_eq!(commit_id(&fx.bot, "main@origin"), agent_origin);

        let stamped = commit_id(&fx.bot, "@-");
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
            },
        )
        .expect("published");
        assert_eq!(commit_id(&fx.work, "main@origin"), tip);
        assert_eq!(commit_id(&fx.bot, "main@origin"), stamped, "nothing folded");
        assert!(
            !jj::is_empty(&fx.bot, "@").expect("empty"),
            "the journal stays in @"
        );
    }

    /// With hosted origins a Land sets each `ocid:` to the agent commit's
    /// SHA: the change ids, trees, and `ochid:` unchanged, the trunk at the
    /// rewritten tip, and the cycle's bookmark gone. Its dry run sets none.
    #[test]
    fn a_land_sets_each_ocid_to_the_agent_commits_sha() {
        let fx = Fixture::new("land-ocid");
        let agent_url = "https://github.com/o/r.agent-session";
        crate::test_helpers::git_ok(
            &fx.work,
            &["remote", "set-url", "origin", "https://github.com/o/r"],
        );
        crate::test_helpers::git_ok(&fx.bot, &["remote", "set-url", "origin", agent_url]);
        let rungs = local_cycle(&fx);
        let agent_at = chid(&fx.bot, "@");
        let trees: Vec<String> = rungs
            .iter()
            .map(|c| jj::tree_of(&fx.work, c).expect("tree"))
            .collect();

        let args = ["-s", "b", "--onto", "main"];
        let mut dry = args.to_vec();
        dry.push("--dry-run");
        land(&mut test_ctx(), &local_land_params(&fx, &dry)).expect("planned");
        let placeholder = format!("ocid: {agent_url}/commit/xxxx");
        for c in &rungs {
            assert!(
                jj::desc_of(&fx.work, c)
                    .expect("desc")
                    .contains(&placeholder)
            );
        }

        land(&mut test_ctx(), &local_land_params(&fx, &args)).expect("landed");
        let sha = commit_id(&fx.bot, "@-");
        for (c, tree) in rungs.iter().zip(&trees) {
            let desc = jj::desc_of(&fx.work, c).expect("desc");
            assert!(
                desc.ends_with(&format!(
                    "\n\nochid: {agent_url}#{agent_at}\nocid: {agent_url}/commit/{sha}"
                )),
                "{desc}"
            );
            assert_eq!(&jj::tree_of(&fx.work, c).expect("tree"), tree);
        }
        assert_eq!(chid(&fx.work, "main"), rungs[1]);
        assert!(!jj::local_bookmark_exists(&fx.work, "feat-a-cycle").expect("local"));
        assert_eq!(chid(&fx.work, "@-"), rungs[1]);
        assert!(jj::is_empty(&fx.work, "@").expect("empty"));
        assert_eq!(commit_id(&fx.bot, "@-"), sha, "no fold");
    }

    /// A published commit keeps its placeholder, and the rung above it
    /// is still set.
    #[test]
    fn a_published_commits_ocid_is_left() {
        let fx = Fixture::new("land-ocid-published");
        local_cycle(&fx);
        let placeholder = "ocid: https://h/o/r.agent-session/commit/xxxx";
        for rev in ["@--", "@-"] {
            let desc = format!(
                "{}\n{placeholder}\n",
                crate::test_helpers::description(&fx.work, rev).trim_end()
            );
            jj_ok(&fx.work, &["describe", "-r", rev, "-m", &desc]);
        }
        jj_ok(&fx.work, &["git", "push", "--named", "shared=@--"]);
        land(
            &mut test_ctx(),
            &local_land_params(
                &fx,
                &["-s", "b", "--source", "feat-a-cycle", "--onto", "main"],
            ),
        )
        .expect("landed");
        let sha = commit_id(&fx.bot, "@-");
        assert!(
            crate::test_helpers::description(&fx.work, "main-").contains(placeholder),
            "the published one"
        );
        assert!(
            crate::test_helpers::description(&fx.work, "main")
                .contains(&format!("ocid: https://h/o/r.agent-session/commit/{sha}")),
            "the one above it"
        );
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
            "--push",
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
            push: true,
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

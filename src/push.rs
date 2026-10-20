//! `push` subcommand: publish a rung's pair. It carries no
//! description and writes none, so what was reviewed is what goes
//! out.
//!
//! `vc-x1 commit` makes the pair, the work commit and the agent
//! commit, each naming the other, or the work half alone with its
//! trailer naming the agent `@`. Push then:
//!
//! - refuses a work `@` that is not empty: unsquashed work is a fix
//!   to fold or the next rung, the user's call, never push's;
//! - refuses a work `@-` with no `ochid:` trailer: not a pair's
//!   commit, so it is made with `commit` first;
//! - completes the agent half, by the state it finds: the agent
//!   `@-` already names the work commit (`commit -s b`), so the
//!   journal in `@` is folded into it, else the agent `@` is the
//!   change the work commit names (`commit -s w`), so the journal
//!   is committed as the partner with the work commit's own
//!   description, else the agent repo changed since the commit,
//!   and the unpublished work commit is restamped with the agent
//!   `@`'s id, said out loud, and the journal committed;
//! - advances the work bookmark, the one `@-` sits on or the one
//!   named, and the agent `main`, pushes the bookmark, and
//!   squash-pushes `main`.
//!
//! The run is a straight line with no saved state, and rerunning is
//! always safe: every stage checks its precondition and no-ops when
//! its work is done. Failures before the remote is reached restore
//! both repos from `jj op` snapshots taken moments earlier. From
//! `push-work` on the remote boundary is crossed, and putting the
//! repos right is the user's call.
//!
//! `-s` is required: `b` publishes the pair, `w` the work half only,
//! `a` the agent half only, the by-hand remainder of a run that
//! stopped after `push-work`, and `p` a plain repo's `@-` on its
//! bookmark, no trailer asked for.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info, warn};

use crate::common::prompt;
use crate::context::Context;
use crate::desc_helpers::{extract_bare_id, extract_ochid_from_desc, with_trailer};
use crate::jj;
use crate::options_flags::scope::{REPO_SCOPE_FORMS, RepoScope, Scope, parse_repo_scope};
use crate::options_flags::squash::SquashSpec;
use crate::pair::{Pair, Partner, Snapshots, resolve_pair};
use crate::subcommand::SubcommandRunner;

/// Bookmark the agent repo always advances and pushes. The agent
/// repo is a linear journal on `main` by design: `<bookmark>` names
/// a work-repo bookmark only.
const BOT_BOOKMARK: &str = "main";

/// CLI arguments for the `push` subcommand.
#[derive(Args, Debug)]
pub struct PushArgs {
    /// Work-repo bookmark to advance, a check against the line `@-`
    /// is on [default: the bookmark of that line]
    #[arg(value_name = "BOOKMARK", conflicts_with = "bookmark")]
    pub bookmark_pos: Option<String>,

    /// Bookmark to advance in the work repo (flag form, see positional).
    #[arg(long, conflicts_with = "bookmark_pos")]
    pub bookmark: Option<String>,

    /// Where: a path in the workspace for `-s b|w|a`, its root or
    /// either repo, or the repo itself for `-s p` [default: the
    /// current directory]
    #[arg(short = 'R', long, value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Which repos, required: `b|both` publishes the pair, `w|work`
    /// the work half, `a|agent` the agent half, `p|por` a plain repo
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = parse_repo_scope,
        required = true,
        help = format!("Which repos, required: {REPO_SCOPE_FORMS}")
    )]
    pub scope: RepoScope,

    /// Pause between every stage for an interactive approval gate.
    #[arg(long)]
    pub step: bool,

    /// Print the exact commands for every stage without side effects.
    #[arg(long)]
    pub dry_run: bool,

    /// Pass the `--step` gates without asking.
    #[arg(short = 'y', long)]
    pub yes: bool,
}

/// Inputs to the push op, flat, owned, clap-free: the two clap
/// bookmark spellings collapsed into `bookmark`, the rest straight
/// over.
pub struct PushParams {
    pub bookmark: Option<String>,
    pub repo: Option<PathBuf>,
    pub scope: RepoScope,
    pub step: bool,
    pub dry_run: bool,
    pub yes: bool,
}

/// What the agent half has to do, decided from the state the pair
/// is found in before anything is touched.
#[derive(Debug, PartialEq, Eq)]
enum AgentPlan {
    /// The agent `@-` names the work commit: fold the journal into it.
    Fold,
    /// The agent `@` is the change the work commit names, or the work
    /// commit is restamped to name it: commit the journal as the
    /// partner with the work commit's description.
    Commit,
}

/// What the stages consume: the bookmark, the work commit's id,
/// and the agent half's plan with the id the agent `main` ends at.
/// Lives for one run in one process.
struct Run {
    bookmark: String,
    /// The work `@-`'s change id, the commit being published.
    work_chid: String,
    /// The agent side: its plan and the change id `main` ends at,
    /// `None` when the scope leaves the agent half out.
    agent: Option<(AgentPlan, String)>,
}

impl From<&PushArgs> for PushParams {
    /// Collapse `bookmark_pos` / `bookmark` (clap enforces they don't
    /// both appear) into the single `bookmark` field.
    fn from(a: &PushArgs) -> Self {
        Self {
            bookmark: a.bookmark_pos.clone().or_else(|| a.bookmark.clone()),
            repo: a.repo.clone(),
            scope: a.scope.clone(),
            step: a.step,
            dry_run: a.dry_run,
            yes: a.yes,
        }
    }
}

impl SubcommandRunner for PushArgs {
    type Params = PushParams;

    /// Delegate to the `From<&PushArgs>` impl above (total).
    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(PushParams::from(self))
    }

    /// Run the `push` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        push(ctx, params)
    }
}

/// Whether `stdin` is attached to a terminal, for the `--step` gates.
fn is_stdin_tty() -> bool {
    crate::common::is_stdin_tty()
}

/// Entry point for the `push` subcommand: resolve `-s` and `-R` and
/// publish.
pub fn push(ctx: &mut Context, params: &PushParams) -> Result<(), Box<dyn std::error::Error>> {
    match &params.scope {
        RepoScope::Por => {
            let repos = crate::common::resolve_scoped_repos(&params.scope, params.repo.as_deref())?;
            let [repo] = repos.as_slice() else {
                return Err(format!("push -s p: expected one repo, got {}", repos.len()).into());
            };
            push_por(ctx, repo, params)
        }
        RepoScope::Sides(sides) => {
            // Both paths whatever the side: a half still needs the
            // other repo to find or name its partner.
            let pair = resolve_pair(params.repo.as_deref())?;
            push_in(ctx, &pair, sides, params)
        }
    }
}

/// `push` on a dual workspace, parameterized on the pair and the
/// sides to publish. CLI dispatch resolves these from `-s` and `-R`,
/// and integration tests pass a fixture's pair.
///
/// The stages run in a straight line. The mutation window, the
/// agent commit or restamp and both bookmark sets, is wrapped so a
/// failure inside it restores both repos from snapshots taken just
/// before it. After `push-work` the remote boundary is crossed and a
/// failure simply propagates.
pub(crate) fn push_in(
    ctx: &mut Context,
    pair: &Pair,
    sides: &Scope,
    params: &PushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = pair.work.as_path();
    if params.dry_run {
        info!("push: DRY-RUN: no side effects (no commits, no pushes)");
    }
    let run = stage_plan(workspace_root, sides, params)?;
    step_gate(params, "plan", "commit-agent")?;

    // Snapshot both repos before the first mutation. Dry-runs
    // mutate nothing, so they need no rollback target.
    let snapshots = if params.dry_run {
        None
    } else {
        Some(Snapshots::take(pair)?)
    };

    if let Err(e) = mutate(ctx, workspace_root, sides, &run, params) {
        if let Some(snapshots) = &snapshots {
            snapshots.restore(pair, "push", e.as_ref());
        }
        return Err(e);
    }

    if sides.has_work() {
        stage_push_work(ctx, workspace_root, &run.bookmark, params)?;
    }
    if sides.has_bot() {
        step_gate(params, "push-work", "squash-push-agent")?;
        stage_squash_push_bot(ctx, workspace_root, params)?;
    }

    if params.dry_run {
        info!("push: DRY-RUN complete: no changes written");
        return Ok(());
    }

    // Honest completion: confirm the world matches what the stages
    // claim to have done. Warning-only: work has crossed the remote
    // boundary by now, so rollback isn't sound and the user needs to
    // investigate.
    match verify_completion(workspace_root, sides, &run) {
        Ok(()) => info!("push: completed all stages (verified)"),
        Err(e) => {
            warn!("push: completed stages but post-completion verification failed:");
            warn!("  {e}");
            warn!("  investigate the discrepancy before the next push");
        }
    }
    Ok(())
}

/// Plan: read the pair's state and decide every stage's input before
/// anything is touched. The refusals live here, so a run that cannot
/// complete stops with both repos as they were.
fn stage_plan(
    root: &Path,
    sides: &Scope,
    params: &PushParams,
) -> Result<Run, Box<dyn std::error::Error>> {
    let bot = bot_path(root)?;
    // The work `@` is a decision: a fix to fold, or the next rung.
    if !jj::is_empty(root, "@")? {
        return Err(
            "push: the work repo's @ holds unsquashed work. Fold it into the pair with \
                    `vc-x1 squash -s b`, or leave it as the next rung and push after \
                    `vc-x1 commit`"
                .into(),
        );
    }
    let pair = Pair {
        work: root.to_path_buf(),
        agent: bot.clone(),
    };
    let Some(work) = crate::pair::work_commit(&pair)? else {
        return Err(
            "push: the work repo's @- carries no `ochid:` trailer, so it is not a pair's \
             commit. Make the pair with `vc-x1 commit -s b` or `-s w` first"
                .into(),
        );
    };
    let work_chid = work.chid.clone();

    let bookmark = match &params.bookmark {
        Some(b) => {
            if !jj::local_bookmark_exists(root, b)? {
                return Err(format!("push: bookmark '{b}' does not exist in the work repo").into());
            }
            let nearest = jj::local_bookmarks_at(root, "heads(::@- & bookmarks())")?;
            if !nearest.contains(b) {
                return Err(format!(
                    "push: bookmark '{b}' is not the nearest bookmark below the work @-, the \
                     line's own ({}). Name one of them, or none to use the line's",
                    nearest.join(", ")
                )
                .into());
            }
            b.clone()
        }
        None => crate::squash_push::current_bookmark(root, "@-")?,
    };

    // One plan line: what is published, and what the agent half does
    // and why, the restamp a warning.
    let publish = format!("push plan: publish {work_chid} on {bookmark}");
    let agent = if sides.has_bot() {
        match crate::pair::partner(&pair, &work)? {
            Partner::Committed(partner) => {
                info!(
                    "{publish}, and fold the journal into the partner at @- {partner}, which names \
                     the work commit"
                );
                Some((AgentPlan::Fold, partner))
            }
            Partner::Pending(agent_at) => {
                info!(
                    "{publish}, and commit the agent @ {agent_at} as the partner, the change the \
                     work commit names"
                );
                Some((AgentPlan::Commit, agent_at))
            }
            Partner::Changed { stamped, agent_at } => {
                if jj::matches(root, &format!("@- & ::{bookmark}@origin"))? {
                    return Err("push: the work @- is already published, so it cannot be \
                                restamped. Put the agent repo back at the change it names, or \
                                make a new pair"
                        .into());
                }
                warn!(
                    "{publish}, and restamp the work commit to name the agent @ {agent_at}: it \
                     names {stamped}, which is neither the agent @ nor the partner at @-, so the \
                     agent repo changed since the commit"
                );
                Some((AgentPlan::Commit, agent_at))
            }
        }
    } else {
        info!("{publish}, the agent half left out by the scope");
        None
    };

    Ok(Run {
        bookmark,
        work_chid,
        agent,
    })
}

/// The rollback-eligible window: everything that mutates a repo
/// before anything reaches a remote.
fn mutate(
    ctx: &mut Context,
    root: &Path,
    sides: &Scope,
    run: &Run,
    params: &PushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some((plan, agent_chid)) = &run.agent {
        stage_commit_agent(ctx, root, plan, agent_chid, run, params)?;
        step_gate(params, "commit-agent", "bookmark-set")?;
    }
    stage_bookmark_set(ctx, root, sides, &run.bookmark, params)?;
    step_gate(params, "bookmark-set", "push-work")?;
    Ok(())
}

/// Commit the agent journal as the partner when the plan says so,
/// restamping the work commit first when its trailer names another
/// change. A `Fold` plan has nothing to do here: the squash-push
/// stage folds the journal.
fn stage_commit_agent(
    ctx: &mut Context,
    root: &Path,
    plan: &AgentPlan,
    agent_chid: &str,
    run: &Run,
    params: &PushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    if *plan == AgentPlan::Fold {
        info!("push commit-agent: skip (the journal folds into the partner at squash-push)");
        return Ok(());
    }
    let bot = bot_path(root)?;
    let bot_arg = bot.to_string_lossy();
    let work_desc = jj::desc_of(root, "@-")?;
    let text = without_trailers(&work_desc);
    let agent_desc = with_trailer(&text, &format!("/{}", run.work_chid));
    let stamped = extract_ochid_from_desc(&work_desc)
        .map(|o| extract_bare_id(&o).to_string())
        .unwrap_or_default();
    let restamp = stamped != agent_chid;
    if params.dry_run {
        if restamp {
            info!("push commit-agent: [dry-run] would restamp the work @- to name {agent_chid}");
        }
        info!("push commit-agent: [dry-run] would run jj commit -R {bot_arg} -m <work desc+ochid>");
        return Ok(());
    }
    if restamp {
        let prefix = crate::desc_helpers::ochid_prefix_for(&bot)?;
        let restamped = with_trailer(&text, &format!("{prefix}{agent_chid}"));
        info!("push commit-agent: restamp work @- to name {agent_chid}");
        ctx.session(root)?.describe("@-", &restamped)?;
    }
    info!("push commit-agent: commit -R {bot_arg}");
    ctx.session(&bot)?.commit(&agent_desc)?;
    Ok(())
}

/// A description without its `ochid:` lines, trimmed, the text a
/// partner commit shares.
fn without_trailers(desc: &str) -> String {
    desc.lines()
        .filter(|l| !l.starts_with("ochid:"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

/// Advance each scoped repo's bookmark to its `@-`: the work repo
/// advances `bookmark`, the agent repo always `main`.
fn stage_bookmark_set(
    ctx: &mut Context,
    root: &Path,
    sides: &Scope,
    bookmark: &str,
    params: &PushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let work_arg = root.to_string_lossy();
    let bot = bot_path(root)?;
    let bot_arg = bot.to_string_lossy();
    if params.dry_run {
        info!(
            "push bookmark-set: [dry-run] would run jj bookmark set {bookmark} -r @- -R {work_arg} \
             / {BOT_BOOKMARK} -r @- -R {bot_arg}"
        );
        return Ok(());
    }
    if sides.has_work() {
        info!("push bookmark-set: bookmark set {bookmark} -r @- -R {work_arg}");
        ctx.session(root)?.bookmark_set(bookmark, "@-")?;
    }
    if sides.has_bot() {
        info!("push bookmark-set: bookmark set {BOT_BOOKMARK} -r @- -R {bot_arg}");
        ctx.session(&bot)?.bookmark_set(BOT_BOOKMARK, "@-")?;
    }
    Ok(())
}

/// Push the work repo's bookmark to origin, after checking every
/// remote ref for it is tracked. Rerunning is safe: `jj git push` on
/// an already-published bookmark reports nothing to do.
fn stage_push_work(
    ctx: &mut Context,
    root: &Path,
    bookmark: &str,
    params: &PushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let work_arg = root.to_string_lossy();
    if params.dry_run {
        info!(
            "push push-work: [dry-run] would run jj git push --bookmark {bookmark} -R {work_arg}"
        );
        return Ok(());
    }
    crate::common::verify_tracking(root, bookmark)?;
    info!("push push-work: git push --bookmark {bookmark} -R {work_arg}");
    ctx.session(root)?.git_push_bookmark(bookmark)?;
    Ok(())
}

/// Squash-push the agent repo, always `main`: the journal in `@`
/// folds into `@-`, the partner, and `main` is pushed, via
/// `squash_push::squash_push` so the ochid-drop guard applies.
fn stage_squash_push_bot(
    ctx: &mut Context,
    root: &Path,
    params: &PushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let bot = bot_path(root)?;
    let bot_arg = bot.to_string_lossy();
    if params.dry_run {
        info!(
            "push squash-push-agent: [dry-run] would squash @ -> @- and push {BOT_BOOKMARK} in {bot_arg}"
        );
        return Ok(());
    }
    info!("push squash-push-agent: squash @ -> @- + push {BOT_BOOKMARK} -R {bot_arg}");
    let sp = crate::squash_push::SquashPushParams {
        repo: bot.clone(),
        squash: SquashSpec {
            source: "@".to_string(),
            target: "@-".to_string(),
        },
        bookmark: BOT_BOOKMARK.to_string(),
        // Mid-push the mismatch is the normal state (bookmark-set
        // just moved main and this stage publishes it): don't report
        // a lost publish.
        at_rest: false,
        yes: true,
    };
    crate::squash_push::squash_push(ctx, &sp)
}

/// `--step`: pause between stages so the user can inspect the repos
/// before the next one runs. `--yes` passes the gates, and a non-tty
/// without it is an error rather than a hang.
fn step_gate(
    params: &PushParams,
    done: &str,
    next: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !params.step {
        return Ok(());
    }
    if params.yes {
        debug!("push step: --yes skips step gate between {done} and {next}");
        return Ok(());
    }
    if !is_stdin_tty() {
        return Err("push: --step requires a tty (stdin is not interactive); \
                    add --yes to bypass step gates in non-interactive contexts"
            .into());
    }
    let answer = prompt(&format!(
        "push step: {done} done. Continue to {next}? [y/N] "
    ))?;
    let normalized = answer.trim().to_ascii_lowercase();
    if normalized != "y" && normalized != "yes" {
        return Err(format!("push: step gate declined after {done} (got {answer:?})").into());
    }
    Ok(())
}

/// Full path of the agent repo for a workspace root, resolved from
/// the config's `repos.agent`. Also the dual-mode entry preflight:
/// resolution verifies both repos' recorded topology agrees.
fn bot_path(workspace_root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    crate::common::require_bot_dir(workspace_root)
}

/// Verify the world matches what the stages claim: the work bookmark
/// at the run's work commit, the work `@` empty, and the agent `main`
/// at the partner. The agent `@` may hold new session writes that
/// landed after the squash, so it is not checked.
fn verify_completion(
    root: &Path,
    sides: &Scope,
    run: &Run,
) -> Result<(), Box<dyn std::error::Error>> {
    if sides.has_work() {
        let actual = jj::chid_of(root, &run.bookmark)?;
        if actual != run.work_chid {
            return Err(format!(
                "completion sanity: work bookmark '{}' is at chid '{actual}' but this run \
                 published '{}'.",
                run.bookmark, run.work_chid
            )
            .into());
        }
        if !jj::is_empty(root, "@")? {
            let wc_diff = jj::diff_stat(root)?;
            return Err(format!(
                "completion sanity: work working copy has uncommitted changes. Diff stat:\n{wc_diff}"
            )
            .into());
        }
    }
    if let Some((_, agent_chid)) = &run.agent {
        let actual = jj::chid_of(&bot_path(root)?, BOT_BOOKMARK)?;
        if actual != *agent_chid {
            return Err(format!(
                "completion sanity: agent bookmark '{BOT_BOOKMARK}' is at chid '{actual}' but \
                 this run published '{agent_chid}'."
            )
            .into());
        }
    }
    Ok(())
}

/// `push -s p`: a plain repo's `@-` goes out on its bookmark, no
/// trailer asked for, the work half's checks otherwise.
fn push_por(
    ctx: &mut Context,
    repo: &Path,
    params: &PushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    if !jj::is_empty(repo, "@")? {
        return Err(
            "push -s p: @ holds unsquashed work. Fold it into @- or commit it first".into(),
        );
    }
    if jj::desc_of(repo, "@-")?.trim().is_empty() {
        return Err("push -s p: @- has no description, so it is not a commit to publish".into());
    }
    let bookmark = match &params.bookmark {
        Some(b) => b.clone(),
        None => crate::squash_push::current_bookmark(repo, "@-")?,
    };
    let arg = repo.to_string_lossy();
    if params.dry_run {
        info!(
            "push: [dry-run] would run jj bookmark set {bookmark} -r @- -R {arg}, then git push --bookmark {bookmark}"
        );
        return Ok(());
    }
    info!("push bookmark-set: bookmark set {bookmark} -r @- -R {arg}");
    ctx.session(repo)?.bookmark_set(&bookmark, "@-")?;
    crate::common::verify_tracking(repo, &bookmark)?;
    info!("push push-work: git push --bookmark {bookmark} -R {arg}");
    ctx.session(repo)?.git_push_bookmark(&bookmark)?;
    info!("push: completed");
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

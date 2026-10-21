//! `push` subcommand: publish a rung's pair. It carries no
//! description and writes none, so what was reviewed is what goes
//! out.
//!
//! `vc-x1 commit` makes the pair, the work commit and the agent
//! commit, each naming the other, or the work half alone with its
//! trailer naming the agent `@`. Each trailer is fixed before push
//! runs, so the two halves are independent, and `push -s b` is
//! `push -s a` then `push -s w`, the order the pair is completed in:
//!
//! - the agent half completes the agent commit by the state it
//!   finds, folding the journal into a committed partner (`commit
//!   -s b`), or committing it as the partner with the work commit's
//!   own text when the agent `@` is the change the work commit names
//!   (`commit -s w`) or the agent repo changed since, then sets and
//!   pushes `main`;
//! - the work half checks its trailer names the partner, restamping
//!   the unpublished work commit when it does not, said out loud,
//!   then sets and pushes the bookmark `@-` sits on, or the one
//!   named.
//!
//! Before either: a work `@` that is not empty is refused, being a
//! fix to fold or the next rung, the user's call, and so is a work
//! `@-` with no `ochid:` trailer, which is not a pair's commit. Each
//! half restores the repo it touched from a `jj op` snapshot when a
//! stage fails before its own push, and from a push on the remote
//! boundary is crossed. Rerunning is always safe: a half whose work
//! is done finds nothing to do.
//!
//! `-s` is required: `b` both halves, `a` the agent half, `w` the
//! work half, and `p` a plain repo's `@-` on its bookmark, no
//! trailer asked for.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info, warn};

use crate::common::prompt;
use crate::context::Context;
use crate::desc_helpers::with_trailer;
use crate::jj;
use crate::options_flags::scope::{
    ALL_REPO_SCOPES, REPO_SCOPE_FORMS, RepoScope, Scope, repo_scope_parser,
};
use crate::options_flags::squash::SquashSpec;
use crate::pair::{Pair, Partner, Snapshot, WorkCommit, is_published, resolve_pair};
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
        value_parser = repo_scope_parser(ALL_REPO_SCOPES),
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

/// What a run published, for the completion check: the work
/// bookmark with the work commit's change id, and the partner the
/// agent `main` ends at, each `None` when the scope left its half
/// out.
struct Run {
    work: Option<(String, String)>,
    partner: Option<String>,
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
/// The checks every half needs run first, then the agent half, then
/// the work half, each restoring its own repo on a local failure.
pub(crate) fn push_in(
    ctx: &mut Context,
    pair: &Pair,
    sides: &Scope,
    params: &PushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    if params.dry_run {
        info!("push: DRY-RUN: no side effects (no commits, no pushes)");
    }
    // The dual-mode preflight: both repos' recorded topology agrees.
    crate::common::require_bot_dir(&pair.work)?;
    if sides.has_work() && !jj::is_empty(&pair.work, "@")? {
        return Err(
            "push: the work repo's @ holds unsquashed work. Fold it into the pair with \
                    `vc-x1 squash -s b`, or leave it as the next rung and push after \
                    `vc-x1 commit`"
                .into(),
        );
    }
    let Some(work) = crate::pair::work_commit(pair)? else {
        return Err(
            "push: the work repo's @- carries no `ochid:` trailer, so it is not a \
                    pair's commit. Make the pair with `vc-x1 commit -s b` or `-s w` first"
                .into(),
        );
    };
    // The bookmark first, so a run whose work half cannot publish
    // stops before the agent half publishes anything.
    let bookmark = if sides.has_work() {
        Some(resolve_bookmark(&pair.work, params)?)
    } else {
        None
    };

    let partner = if sides.has_bot() {
        Some(agent_half(ctx, pair, &work, params)?)
    } else {
        None
    };
    if let Some(bookmark) = &bookmark {
        if sides.has_bot() {
            step_gate(params, "the agent half", "the work half")?;
        }
        work_half(ctx, pair, &work, bookmark, params)?;
    }

    if params.dry_run {
        info!("push: DRY-RUN complete: no changes written");
        return Ok(());
    }
    // Honest completion, warning-only: the remote boundary is crossed
    // by now, so the user investigates rather than a rollback.
    let run = Run {
        work: bookmark.map(|b| (b, work.chid.clone())),
        partner,
    };
    match verify_completion(pair, &run) {
        Ok(()) => info!("push: completed all stages (verified)"),
        Err(e) => {
            warn!("push: completed stages but post-completion verification failed:");
            warn!("  {e}");
            warn!("  investigate the discrepancy before the next push");
        }
    }
    Ok(())
}

/// The work bookmark: the one named, which must be among the nearest
/// bookmarks below the work `@-`, else the line's own.
fn resolve_bookmark(
    root: &Path,
    params: &PushParams,
) -> Result<String, Box<dyn std::error::Error>> {
    match &params.bookmark {
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
            Ok(b.clone())
        }
        None => crate::squash_push::current_bookmark(root, "@-"),
    }
}

/// The agent half: complete the partner by the state found, then set
/// and push `main`. Returns the partner's change id, where `main`
/// ends.
fn agent_half(
    ctx: &mut Context,
    pair: &Pair,
    work: &WorkCommit,
    params: &PushParams,
) -> Result<String, Box<dyn std::error::Error>> {
    // The intent lines carry the dry-run tag too, or they read as done.
    let dry = if params.dry_run { "[dry-run] " } else { "" };
    let (partner, commit_journal) = match crate::pair::partner(pair, work)? {
        Partner::Committed(partner) => {
            info!(
                "push agent: {dry}fold the journal into the partner at @- {partner}, which names the \
                 work commit, and publish {BOT_BOOKMARK}"
            );
            (partner, false)
        }
        Partner::Pending(agent_at) => {
            info!(
                "push agent: {dry}commit the agent @ {agent_at} as the partner, the change the work \
                 commit names, and publish {BOT_BOOKMARK}"
            );
            (agent_at, true)
        }
        Partner::Changed { stamped, agent_at } => {
            // The work half will restamp, which a published work
            // commit cannot take, so stop before anything moves.
            if is_published(&pair.work, "@-")? {
                return Err(
                    "push: the work @- is published and names an agent change that is \
                            no longer the partner, so it cannot be restamped. Put the agent repo \
                            back at the change it names, or make a new pair"
                        .into(),
                );
            }
            warn!(
                "push agent: {dry}the work commit names {stamped}, which is neither the agent @ nor \
                 the partner at @-, so the agent repo changed since the commit. Commit the agent \
                 @ {agent_at} as the partner and publish {BOT_BOOKMARK}, and the work half \
                 restamps the work commit to name it"
            );
            (agent_at, true)
        }
    };
    let agent_arg = pair.agent.to_string_lossy();
    if params.dry_run {
        if commit_journal {
            info!("push agent: [dry-run] would run jj commit -R {agent_arg} -m <work text+ochid>");
        }
        info!("push agent: [dry-run] would squash @ -> @- and push {BOT_BOOKMARK} -R {agent_arg}");
        return Ok(partner);
    }
    if commit_journal {
        let text = without_trailers(&jj::desc_of(&pair.work, "@-")?);
        let desc = with_trailer(&text, &format!("/{}", work.chid));
        let snapshot = Snapshot::take(&pair.agent)?;
        info!("push agent: commit -R {agent_arg}");
        if let Err(e) = ctx.session(&pair.agent)?.commit(&desc) {
            snapshot.restore("push agent", e.as_ref());
            return Err(e);
        }
        step_gate(params, "the agent commit", "the agent publish")?;
    }
    info!("push agent: squash @ -> @- + push {BOT_BOOKMARK} -R {agent_arg}");
    let sp = crate::squash_push::SquashPushParams {
        repo: pair.agent.clone(),
        squash: SquashSpec {
            source: "@".to_string(),
            target: "@-".to_string(),
        },
        bookmark: BOT_BOOKMARK.to_string(),
        // Mid-push the mismatch is the normal state, since this stage
        // is what publishes main: don't report a lost publish.
        at_rest: false,
        yes: true,
        dry_run: false,
    };
    crate::squash_push::squash_push(ctx, &sp)?;
    Ok(partner)
}

/// The work half: restamp the unpublished work commit when its
/// trailer does not name the partner, then set and push the bookmark.
fn work_half(
    ctx: &mut Context,
    pair: &Pair,
    work: &WorkCommit,
    bookmark: &str,
    params: &PushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let target = match crate::pair::partner(pair, work)? {
        Partner::Committed(partner) => partner,
        Partner::Pending(agent_at) | Partner::Changed { agent_at, .. } => agent_at,
    };
    let restamp = work.stamped != target;
    let dry = if params.dry_run { "[dry-run] " } else { "" };
    if restamp {
        if is_published(&pair.work, "@-")? {
            return Err(format!(
                "push: the work @- is published and names {}, not the partner {target}, so it \
                 cannot be restamped",
                work.stamped
            )
            .into());
        }
        warn!(
            "push work: {dry}publish {} on {bookmark}, restamping its trailer to name the partner {target}",
            work.chid
        );
    } else {
        info!("push work: {dry}publish {} on {bookmark}", work.chid);
    }
    let work_arg = pair.work.to_string_lossy();
    if params.dry_run {
        info!("push work: [dry-run] would run jj bookmark set {bookmark} -r @- -R {work_arg}");
        info!("push work: [dry-run] would run jj git push --bookmark {bookmark} -R {work_arg}");
        return Ok(());
    }
    let snapshot = Snapshot::take(&pair.work)?;
    let local = (|| -> Result<(), Box<dyn std::error::Error>> {
        if restamp {
            let text = without_trailers(&jj::desc_of(&pair.work, "@-")?);
            let prefix = crate::desc_helpers::ochid_prefix_for(&pair.agent)?;
            info!("push work: restamp @- to name {target}");
            ctx.session(&pair.work)?
                .describe("@-", &with_trailer(&text, &format!("{prefix}{target}")))?;
        }
        info!("push work: bookmark set {bookmark} -r @- -R {work_arg}");
        ctx.session(&pair.work)?.bookmark_set(bookmark, "@-")?;
        Ok(())
    })();
    if let Err(e) = local {
        snapshot.restore("push work", e.as_ref());
        return Err(e);
    }
    step_gate(params, "the work bookmark", "the work push")?;
    crate::common::verify_tracking(&pair.work, bookmark)?;
    info!("push work: git push --bookmark {bookmark} -R {work_arg}");
    ctx.session(&pair.work)?.git_push_bookmark(bookmark)?;
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

/// Verify the world matches what the halves claim: the work
/// bookmark at the work commit and the work `@` empty, and the agent
/// `main` at the partner. The agent `@` may hold new session writes
/// that landed after the squash, so it is not checked.
fn verify_completion(pair: &Pair, run: &Run) -> Result<(), Box<dyn std::error::Error>> {
    if let Some((bookmark, work_chid)) = &run.work {
        let actual = jj::chid_of(&pair.work, bookmark)?;
        if actual != *work_chid {
            return Err(format!(
                "completion sanity: work bookmark '{bookmark}' is at chid '{actual}' but this \
                 run published '{work_chid}'."
            )
            .into());
        }
        if !jj::is_empty(&pair.work, "@")? {
            let wc_diff = jj::diff_stat(&pair.work)?;
            return Err(format!(
                "completion sanity: work working copy has uncommitted changes. Diff stat:\n{wc_diff}"
            )
            .into());
        }
    }
    if let Some(partner) = &run.partner {
        let actual = jj::chid_of(&pair.agent, BOT_BOOKMARK)?;
        if actual != *partner {
            return Err(format!(
                "completion sanity: agent bookmark '{BOT_BOOKMARK}' is at chid '{actual}' but \
                 this run published '{partner}'."
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

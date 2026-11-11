//! `push` subcommand: publish a rung's pair. It carries no
//! description and writes none, so what was reviewed is what goes
//! out.
//!
//! `vc-x1 commit` makes the pair, the work commit and the agent
//! commit, each naming the other, or the work half alone with its
//! trailer naming the agent `@`. Each side names the other by change
//! id, which no push moves, so the two halves are independent. `push
//! -s b` makes every local step first, then publishes the work repo
//! and then the agent repo, the order `squash-push -s b` publishes in:
//!
//! - the agent half completes the agent commit by the state it
//!   finds, committing the journal as the partner with the work
//!   commit's own text when the agent `@` is the change the work
//!   commit names (`commit -s w`) or the agent repo changed since;
//! - the work half checks its trailer names the partner, restamping
//!   the unpublished work commit when it does not, said out loud,
//!   then sets and pushes the bookmark `@-` sits on, or the one
//!   named, a new name created there;
//! - the agent half then publishes `main`, folding the journal into a
//!   committed partner (`commit -s b`, or a Land's) unless the partner
//!   is already published, since a fold would rewrite it.
//!
//! A push that would be forced, a published commit rewritten, is
//! warned of and answered before anything moves, by `--yes` or a
//! prompt, and without a tty and `--yes` it is an error.
//!
//! Before either: a work `@` that is not empty is refused, being a
//! fix to fold or the next rung, the user's call, and so is a work
//! `@-` with no `ochid:` trailer, which is not a pair's commit. Each
//! half restores the repo it touched from a `jj op` snapshot when a
//! stage fails before its own push, and from a push on the remote
//! boundary is crossed. Rerunning is always safe: a half whose work
//! is done finds nothing to do.
//!
//! `-r` names another commit, already made, and only a name goes out
//! for it, the bookmark at it or a new one: the pair's checks are the
//! work `@-`'s.
//!
//! `-s` is required: `b` both halves, `a` the agent half, `w` the
//! work half, and `p` a plain repo's `@-` on its bookmark, no
//! trailer asked for.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info, warn};

use crate::common::{ForcedPush, prompt};
use crate::context::Context;
use crate::desc_helpers::{Trailers, repo_url_for, with_trailer, with_trailers};
use crate::jj::{self, Force};
use crate::options_flags::retired_positional::refusal;
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

/// The commit push publishes when `-r` is left out: the work commit of
/// the pair `commit` made.
const WORK_COMMIT: &str = "@-";

/// CLI arguments for the `push` subcommand.
#[derive(Args, Debug)]
pub struct PushArgs {
    /// Retired: the bookmark, now `--bookmark`, or a revision, now
    /// `-r`, refused naming the one it was.
    #[arg(value_name = "RETIRED", hide = true)]
    pub retired: Option<String>,

    /// The commit to publish, one: `@-` publishes the pair, any other
    /// a name alone [default: `@-`]
    #[arg(short = 'r', long, value_name = "REV")]
    pub revision: Option<String>,

    /// The remote's name for it: a bookmark on the line, moved there,
    /// one already at REV, or a new name, created there [default: the
    /// line's bookmark, or the one at REV]
    #[arg(long, value_name = "NAME")]
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

    /// Pass the `--step` gates, and a forced push, without asking
    #[arg(short = 'y', long)]
    pub yes: bool,
}

/// Inputs to the push op, flat, owned, clap-free, straight over from
/// the flags.
pub struct PushParams {
    pub revision: Option<String>,
    pub bookmark: Option<String>,
    pub repo: Option<PathBuf>,
    pub scope: RepoScope,
    pub step: bool,
    pub dry_run: bool,
    pub yes: bool,
    /// Who answers a push that would be forced: `--yes`, else asked.
    /// `commit --push` and `cycle --land --push` refuse one.
    pub forced: ForcedPush,
}

/// What a run published, for the completion check: the work
/// bookmark with the work commit's change id, and the partner the
/// agent `main` ends at, each `None` when the scope left its half
/// out. `held` is a work `@` holding new work when the run began,
/// which push leaves as it is.
struct Run {
    work: Option<(String, String)>,
    partner: Option<String>,
    held: bool,
}

impl From<&PushArgs> for PushParams {
    /// The flags straight over; the retired positional is
    /// `to_params`'s to refuse.
    fn from(a: &PushArgs) -> Self {
        Self {
            revision: a.revision.clone(),
            bookmark: a.bookmark.clone(),
            repo: a.repo.clone(),
            scope: a.scope.clone(),
            step: a.step,
            dry_run: a.dry_run,
            yes: a.yes,
            forced: if a.yes {
                ForcedPush::Yes
            } else {
                ForcedPush::Ask
            },
        }
    }
}

impl SubcommandRunner for PushArgs {
    type Params = PushParams;

    /// Refuse the retired positional, else delegate to the
    /// `From<&PushArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        if let Some(value) = &self.retired {
            return Err(retired_refusal(self.repo.as_deref(), value));
        }
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

/// The refusal for the retired positional: `--bookmark` when `value`
/// is a bookmark in the repo push runs in, which is what it meant, else
/// `-r`, a revision being the other thing it could be.
fn retired_refusal(repo: Option<&Path>, value: &str) -> String {
    let repo = repo.unwrap_or(Path::new(".")); // OK: no -R is the current directory
    // OK: a failed lookup only loses the hint, so it reads as no bookmark
    let is_bookmark = jj::local_bookmark_exists(repo, value).unwrap_or(false);
    refusal("push", value, if is_bookmark { "--bookmark" } else { "-r" })
}

/// The commit push publishes: `-r`'s, else the work `@-`.
fn revision(params: &PushParams) -> &str {
    params.revision.as_deref().unwrap_or(WORK_COMMIT) // OK: `-r` left out is the `@-`
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
            refuse_a_revision(repo, params)?;
            if publish_elsewhere(ctx, repo, params)? {
                return Ok(());
            }
            push_por(ctx, repo, params)
        }
        RepoScope::Sides(sides) => {
            // Both paths whatever the side: a half still needs the
            // other repo to find or name its partner.
            let pair = resolve_pair(params.repo.as_deref())?;
            refuse_a_revision(&pair.work, params)?;
            if !sides.has_work() {
                if let Some(rev) = &params.revision {
                    return Err(format!(
                        "push -s a: the agent half publishes the agent `main`, so `-r {rev}` names \
                         nothing. `-s b` or `-s w` publishes a work commit"
                    )
                    .into());
                }
            } else if publish_elsewhere(ctx, &pair.work, params)? {
                return Ok(());
            }
            push_in(ctx, &pair, sides, params)
        }
    }
}

/// Refuse a `--bookmark` that is no bookmark but a revision, such as
/// `@-`: the flag names the remote's name, and `-r` the commit.
fn refuse_a_revision(repo: &Path, params: &PushParams) -> Result<(), Box<dyn std::error::Error>> {
    let Some(name) = &params.bookmark else {
        return Ok(());
    };
    if jj::local_bookmark_exists(repo, name)? || !jj::rev_exists(repo, name)? {
        return Ok(());
    }
    Err(format!(
        "push: '{name}' is a revision, not a bookmark name: `-r {name}` names the commit to \
         publish, and `--bookmark` its name on the remote"
    )
    .into())
}

/// A REV other than the work `@-`, a commit already made: publish a
/// name for it and no pair, since the pair's checks, its trailer, its
/// partner, and the restamp, are the `@-`'s. True when it was this
/// case. The name is the one bookmark at REV, or `--bookmark`'s, that
/// bookmark or a new one created there. REV must be one commit, and
/// the working copy, which holds changes not yet committed, and the
/// root, which has nothing to publish, are refused, `vc-x1 commit`
/// being what turns the working copy into a commit.
fn publish_elsewhere(
    ctx: &mut Context,
    repo: &Path,
    params: &PushParams,
) -> Result<bool, Box<dyn std::error::Error>> {
    let rev = revision(params);
    let cids = jj::cids_of(repo, rev)?;
    let at = match cids.as_slice() {
        [one] => one.clone(),
        [] => return Err(format!("push: -r '{rev}' resolves to no commit").into()),
        many => {
            return Err(format!(
                "push: -r '{rev}' resolves to {} commits, and push publishes one",
                many.len()
            )
            .into());
        }
    };
    if at == jj::cid_of(repo, WORK_COMMIT)? {
        return Ok(false);
    }
    if at == jj::cid_of(repo, "@")? {
        return Err(format!(
            "push: -r '{rev}' is the working copy, which is not committed: `vc-x1 commit` \
             commits it and pushes it"
        )
        .into());
    }
    if at == jj::cid_of(repo, "root()")? {
        return Err(
            format!("push: -r '{rev}' is the root commit, which has nothing to publish").into(),
        );
    }
    let at_rev = jj::local_bookmarks_at(repo, &at)?;
    let (name, new) = match &params.bookmark {
        Some(name) if at_rev.contains(name) => (name.clone(), false),
        Some(name) if jj::local_bookmark_exists(repo, name)? => {
            return Err(format!(
                "push: bookmark '{name}' is not at -r '{rev}': push publishes a name where it \
                 is, or a new one created there, and `vc-x1 move-bookmark` moves one"
            )
            .into());
        }
        Some(name) => (name.clone(), true),
        None => match at_rev.as_slice() {
            [one] => (one.clone(), false),
            [] => {
                return Err(format!(
                    "push: -r '{rev}' carries no bookmark, so name one with --bookmark, which \
                     is created there"
                )
                .into());
            }
            many => {
                return Err(format!(
                    "push: -r '{rev}' carries several bookmarks: {}, so name the one with \
                     --bookmark",
                    many.join(", ")
                )
                .into());
            }
        },
    };
    let short = jj::cid_short_of(repo, &at)?;
    let dry = if params.dry_run { "[dry-run] " } else { "" };
    if new {
        info!("push: {dry}create '{name}' at {short}, a commit already made, and push it");
    } else {
        info!("push: {dry}publish '{name}' where it is, {short}, a commit already made");
    }
    let force = answer_forced(ctx, repo, &name, &at, params)?;
    if params.dry_run {
        return Ok(true);
    }
    if new {
        ctx.session(repo)?.bookmark_set(&name, &at)?;
    } else {
        crate::common::verify_tracking(repo, &name)?;
    }
    ctx.session(repo)?.git_push_bookmark(&name, force)?;
    Ok(true)
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
    // New work in the work `@` is the next rung's, and push publishes
    // `@-`, so it is left as it is.
    let held = sides.has_work() && !jj::is_empty(&pair.work, "@")?;
    if held {
        info!("push: the work @ holds new work, left as it is, and @- is what goes out");
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
    // A push that would be forced, a published commit rewritten, is
    // answered here, before anything moves, and each half then pushes
    // knowing the answer.
    let work_force = match &bookmark {
        Some(b) => answer_forced(ctx, &pair.work, b, WORK_COMMIT, params)?,
        None => Force::Refuse,
    };
    let agent_force = if sides.has_bot() {
        let target = agent_target(pair, &work)?;
        answer_forced(ctx, &pair.agent, BOT_BOOKMARK, target, params)?
    } else {
        Force::Refuse
    };

    // Every local step before any push: the agent partner completed,
    // then the work half restamps and publishes, then the agent `main`
    // is published. Either order is correct, each side naming the other
    // by change id, which neither push moves. Work first, as `squash-push
    // -s b` publishes: the work-repo is the shared one, so its push is the
    // one likely refused, and refused first it leaves nothing published.
    let agent = if sides.has_bot() {
        Some(agent_half(ctx, pair, &work, params)?)
    } else {
        None
    };
    if let Some(bookmark) = &bookmark {
        if sides.has_bot() {
            step_gate(params, "the agent commit", "the work half")?;
        }
        work_half(ctx, pair, &work, bookmark, params, work_force)?;
    }
    let partner = match agent {
        Some((partner, how)) => {
            if sides.has_work() {
                step_gate(params, "the work push", "the agent publish")?;
            } else {
                step_gate(params, "the agent commit", "the agent publish")?;
            }
            if !params.dry_run {
                agent_publish(ctx, pair, how, agent_force)?;
            }
            Some(partner)
        }
        None => None,
    };

    if params.dry_run {
        info!("push: DRY-RUN complete: no changes written");
        return Ok(());
    }
    // Honest completion, warning-only: the remote boundary is crossed
    // by now, so the user investigates rather than a rollback.
    let run = Run {
        work: bookmark.map(|b| (b, work.chid.clone())),
        partner,
        held,
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

/// The bookmark `@-` goes out on: the one named, which must be among
/// the nearest bookmarks below `@-` or a new name, set there by the
/// push, else the line's own.
fn resolve_bookmark(
    root: &Path,
    params: &PushParams,
) -> Result<String, Box<dyn std::error::Error>> {
    match &params.bookmark {
        Some(b) => {
            if !jj::local_bookmark_exists(root, b)? {
                let dry = if params.dry_run { "[dry-run] " } else { "" };
                info!("push: {dry}'{b}' is a new name, created at @- and pushed tracked");
                return Ok(b.clone());
            }
            let nearest = jj::local_bookmarks_at(root, "heads(::@- & bookmarks())")?;
            if !nearest.contains(b) {
                return Err(format!(
                    "push: bookmark '{b}' is not the nearest bookmark below @-, the line's own \
                     ({}). Name one of them, a new name to create at @-, or none to use the \
                     line's, and `-r {b}` publishes it where it is",
                    nearest.join(", ")
                )
                .into());
            }
            Ok(b.clone())
        }
        None => crate::squash_push::current_bookmark(root, WORK_COMMIT),
    }
}

/// How the agent half publishes `main`, once the work half has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentPublish {
    /// Set `main` at `@-` and push, folding nothing: the partner is
    /// already published.
    Main,
    /// Fold the journal into `@-` and push, by `squash-push`.
    SquashPush,
}

/// The agent half's local steps: complete the partner by the state
/// found, committing the journal as the partner when the work commit
/// names the agent `@`. Pushes nothing, [`agent_publish`] does, after
/// the work half. Returns the partner's change id, where `main` ends,
/// and how `main` is published.
fn agent_half(
    ctx: &mut Context,
    pair: &Pair,
    work: &WorkCommit,
    params: &PushParams,
) -> Result<(String, AgentPublish), Box<dyn std::error::Error>> {
    // The intent lines carry the dry-run tag too, or they read as done.
    let dry = if params.dry_run { "[dry-run] " } else { "" };
    let (partner, commit_journal) = match crate::pair::partner(pair, work)? {
        Partner::Published(partner) => {
            info!(
                "push agent: {dry}publish {BOT_BOOKMARK} at the partner @- {partner}, which is \
                 already published, leaving the journal in @ for the next agent commit"
            );
            return Ok((partner, AgentPublish::Main));
        }
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
        info!(
            "push agent: [dry-run] would squash @ -> @- and push {BOT_BOOKMARK} -R {agent_arg}, \
             after the work half"
        );
        return Ok((partner, AgentPublish::SquashPush));
    }
    if commit_journal {
        let text = without_trailers(&jj::desc_of(&pair.work, "@-")?);
        let desc = with_trailer(&text, repo_url_for(&pair.work)?, &work.chid);
        let snapshot = Snapshot::take(&pair.agent)?;
        info!("push agent: commit -R {agent_arg}");
        if let Err(e) = ctx.session(&pair.agent)?.commit(&desc) {
            snapshot.restore("push agent", e.as_ref());
            return Err(e);
        }
    }
    Ok((partner, AgentPublish::SquashPush))
}

/// The agent half's publish, after the work half: `main` set at the
/// partner and pushed, the journal folded into it first unless the
/// partner is already published.
fn agent_publish(
    ctx: &mut Context,
    pair: &Pair,
    how: AgentPublish,
    force: Force,
) -> Result<(), Box<dyn std::error::Error>> {
    let agent_arg = pair.agent.to_string_lossy();
    match how {
        AgentPublish::Main => {
            info!("push agent: push {BOT_BOOKMARK} -R {agent_arg}");
            publish_agent_main(ctx, &pair.agent, force)
        }
        AgentPublish::SquashPush => {
            info!("push agent: squash @ -> @- + push {BOT_BOOKMARK} -R {agent_arg}");
            let sp = crate::squash_push::SquashPushParams {
                repo: pair.agent.clone(),
                squash: SquashSpec {
                    source: "@".to_string(),
                    target: "@-".to_string(),
                },
                bookmark: BOT_BOOKMARK.to_string(),
                // Mid-push, the bookmark off its origin is the normal
                // state, since this stage is what publishes main.
                at_rest: false,
                yes: true,
                dry_run: false,
                // Answered by push before anything moved.
                forced: match force {
                    Force::Allow => ForcedPush::Yes,
                    Force::Refuse => ForcedPush::Refuse,
                },
            };
            crate::squash_push::squash_push(ctx, &sp)
        }
    }
}

/// Set the agent `main` at `@-` and push it, folding nothing, so a
/// partner already published is never rewritten.
pub(crate) fn publish_agent_main(
    ctx: &mut Context,
    agent: &Path,
    force: Force,
) -> Result<(), Box<dyn std::error::Error>> {
    ctx.session(agent)?.bookmark_set(BOT_BOOKMARK, "@-")?;
    crate::common::verify_tracking(agent, BOT_BOOKMARK)?;
    ctx.session(agent)?.git_push_bookmark(BOT_BOOKMARK, force)
}

/// Answer, before anything moves, a push of `rev` as `bookmark` that
/// would be forced: warned of, then answered as `params.forced` says.
/// `Allow` when the push is forced and answered yes, `Refuse` when it
/// is a fast-forward, so a push that turns forced after is refused. A
/// dry run says so and asks nothing.
fn answer_forced(
    ctx: &Context,
    repo: &Path,
    bookmark: &str,
    rev: &str,
    params: &PushParams,
) -> Result<Force, Box<dyn std::error::Error>> {
    let Some(why) = crate::common::forced_bookmark_push(repo, bookmark, rev)? else {
        return Ok(Force::Refuse);
    };
    warn!("push: the push of '{bookmark}' is forced, since {why}");
    if params.dry_run {
        info!("push: [dry-run] would ask before the forced push of '{bookmark}', unless --yes");
        return Ok(Force::Allow);
    }
    crate::common::confirm_forced_push("push", bookmark, &why, params.forced, ctx.stdin_is_tty)?;
    Ok(Force::Allow)
}

/// The agent commit `main` ends at, read before the agent half moves
/// anything: the agent `@` when the journal is committed as the
/// partner, else `@-`.
fn agent_target(
    pair: &Pair,
    work: &WorkCommit,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    Ok(match crate::pair::partner(pair, work)? {
        Partner::Published(_) | Partner::Committed(_) => "@-",
        Partner::Pending(_) | Partner::Changed { .. } => "@",
    })
}

/// The work half: restamp the unpublished work commit when its
/// trailer does not name the partner, then set and push the bookmark.
fn work_half(
    ctx: &mut Context,
    pair: &Pair,
    work: &WorkCommit,
    bookmark: &str,
    params: &PushParams,
    force: Force,
) -> Result<(), Box<dyn std::error::Error>> {
    let target = match crate::pair::partner(pair, work)? {
        Partner::Committed(partner) | Partner::Published(partner) => partner,
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
            let desc = jj::desc_of(&pair.work, "@-")?;
            let text = without_trailers(&desc);
            let mut trailers = Trailers::of(&desc);
            info!("push work: restamp @- to name {target}");
            trailers.ochids = vec![target.clone()];
            ctx.session(&pair.work)?
                .describe("@-", &with_trailers(&text, &trailers))?;
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
    ctx.session(&pair.work)?.git_push_bookmark(bookmark, force)
}

/// A description without its stamped trailer lines, trimmed, the text
/// a partner commit shares.
fn without_trailers(desc: &str) -> String {
    crate::desc_helpers::without_trailer_lines(desc)
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
/// bookmark at the work commit and the work `@` empty unless it held
/// new work when the run began, and the agent `main` at the partner. The agent `@` may hold new session writes
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
        if !run.held && !jj::is_empty(&pair.work, "@")? {
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
        info!("push -s p: @ holds new work, left as it is, and @- is what goes out");
    }
    if jj::desc_of(repo, "@-")?.trim().is_empty() {
        return Err("push -s p: @- has no description, so it is not a commit to publish".into());
    }
    let bookmark = resolve_bookmark(repo, params)?;
    let force = answer_forced(ctx, repo, &bookmark, WORK_COMMIT, params)?;
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
    ctx.session(repo)?.git_push_bookmark(&bookmark, force)?;
    info!("push: completed");
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

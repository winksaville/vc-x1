//! The `commit` subcommand: make a rung's commit pair, one commit
//! per repo of a dual workspace, each stamped with the other's
//! change id, before anything is pushed.
//!
//! The pair is what the per-rung flow reviews: the description and
//! the diff together, as a commit, by `vc-x1 show -s both @-`.
//! Nothing here reaches a remote, so a fix after the review is a
//! fold into the pair and never a remote rewrite, and `vc-x1 push`
//! has nothing left to write: it publishes what was reviewed.
//!
//! - `-s b` makes both commits now. Both change ids are known
//!   before either is made, since a jj commit keeps the working
//!   copy's change id, so each trailer is written once and never
//!   predicted or restamped.
//! - `-s w` makes the work commit now, its trailer naming the agent
//!   `@` all the same, and leaves the agent half to `push`, which
//!   commits the journal as the partner. The pair is then made in
//!   two steps, and the review's own transcript lands in the rung's
//!   agent commit.
//! - `-s a` commits the agent repo alone, an unpaired commit with the
//!   given title and body and no trailer, since it belongs to no
//!   rung. An agent `@` that is a work commit's pending partner keeps
//!   its change id when committed, so the work commit's trailer still
//!   resolves, and a warning says the back-trailer is missing.
//! - The commit is pushed in the same invocation, as `vc-x1 push`
//!   would after it: a pair by `push -s b`, its agent half completed
//!   as the partner and its bookmark published, a plain repo by
//!   `push -s p`, and an agent commit by publishing the agent `main`.
//!   `--no-push` keeps it local, and `--push` pushes it over
//!   `[commit] push = false`, true when absent. A commit with
//!   `--no-advance` pushes only with `--push`.
//! - `-s p` commits a plain repo with the title and body and no
//!   trailer, the same description discipline on a repo with no
//!   partner.
//! - `--body` is the text, or `@PATH` to read it from a file, since
//!   a body in commit-body form has markers and blank lines a shell
//!   quote mangles.
//! - The work side's `@` must hold a change, since a rung changes
//!   the work repo. The agent side's `@` is the session's journal
//!   and is committed as it stands.
//! - `--closing [REV]` closes the cycle, nothing assumed about the
//!   branch:
//!   - Bare: a single-step cycle, which has no parent to name. `@-`,
//!     its opening, is finalized in place, no new commit: `@` folds
//!     into it and it takes the bare cycle title, its body and
//!     trailer kept, and the agent `@` is left for push, since the
//!     commit's partner is the opening's.
//!   - With `REV`, any one commit, a bookmark, a change id, a commit
//!     id, or an `@` form: a multi-step cycle's closing commit, `@`
//!     given `REV` and the last rung as its parents, in that order,
//!     by the primitive `vc-x1 merge` uses, then committed. A bookmark
//!     is read where it is now, so one that moved during the cycle is
//!     joined by the merge. The opening alone above `REV` is refused,
//!     pointing at the bare form, unless `--dead-end`.
//!   - `--title` is the bare cycle title, read from the opening, the
//!     first commit above `REV`, when left out, and " closing" is
//!     appended, " dead-end closing" with `--dead-end`, which always
//!     makes a closing commit. `--body` defaults to `[closing] body`,
//!     its `{title}` the cycle title, and a finalized commit keeps
//!     the opening's.
//! - `--opening <BOOKMARK>` makes the work commit the cycle's
//!   opening: `--title` is the bare cycle title and " opening" is
//!   appended. The bookmark is created at the work `@-`, where the
//!   cycle's branch starts, and pushed to `origin`, a name for a
//!   commit already public, so nothing unreviewed goes out, and the
//!   opening itself stays local until the push after its review.
//!   `--skip local` takes a bookmark already at `@-` and pushes it,
//!   `--skip remote` creates it and pushes nothing, and `--skip both`
//!   only checks it is there. Every check runs before the commit.
//! - The line's bookmark advances to the work commit, locally: the
//!   nearest bookmark below the work `@-`, as `push` resolves it, or
//!   the one `--bookmark` names, or `--opening`'s. jj moves a bookmark
//!   with its commit through a later squash or amend, so the advance
//!   holds through the review's fixes, and the remote moves only at
//!   `push`. `--no-advance` leaves it, and none or several candidates
//!   are refused before anything is made. The agent repo's `main` is
//!   push's to set.
//! - A failure after the first commit restores both repos from the
//!   `jj op` snapshots taken just before it, as push does inside its
//!   mutation window.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info, warn};

use crate::context::Context;
use crate::desc_helpers::{ochid_prefix_for, with_trailer};
use crate::jj;
use crate::options_flags::dry_run::DryRunFlag;
use crate::options_flags::scope::{REPO_SCOPE_FORMS, RepoScope, Scope, Side, repo_scope_parser};
use crate::pair::{Pair, Snapshots, resolve_pair};
use crate::subcommand::SubcommandRunner;

/// Commit the working copies as a rung's pair, or a plain repo's
/// commit, with the given title and body.
#[derive(Args, Debug)]
pub struct CommitArgs {
    /// Where: a path in the workspace for `-s b|w`, its root or
    /// either repo, or the repo itself for `-s p` [default: the
    /// current directory]
    #[arg(short = 'R', long, value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Which repos, required: `b|both` makes both commits now,
    /// `w|work` the work half with the agent half left to push,
    /// `a|agent` the agent repo alone, unpaired, and `p|por` a plain
    /// repo's commit
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = repo_scope_parser(&["work", "agent", "both", "por"]),
        required = true,
        help = format!("Which repos, required: {REPO_SCOPE_FORMS}")
    )]
    pub scope: RepoScope,

    /// Commit title, the Conventional Commit line; with `--opening`
    /// or `--closing` the bare cycle title [default with `--closing`:
    /// read from the opening]
    #[arg(long, value_name = "STR", required_unless_present = "closing")]
    pub title: Option<String>,

    /// Commit body, the problem and its solutions, or @PATH to read
    /// it from a file [default with `--closing`: `[closing] body`, or
    /// the opening's for a single-step cycle]
    #[arg(long, value_name = "STR|@PATH", required_unless_present = "closing")]
    pub body: Option<String>,

    /// Close the cycle: bare, a single-step cycle's opening at `@-`
    /// finalized in place; with REV, any one commit, the closing
    /// commit, the merge of REV and the last rung
    #[arg(long, value_name = "REV", conflicts_with = "opening")]
    pub closing: Option<Option<String>>,

    /// With `--closing`, close the cycle as a dead end: the title ends
    /// " dead-end closing", always on a closing commit
    #[arg(long, requires = "closing")]
    pub dead_end: bool,

    /// Make the work commit the cycle's opening: " opening" is
    /// appended to the bare cycle title, and BOOKMARK is created at
    /// the work `@-` and pushed to `origin` [default: the bare
    /// title's slug]
    #[arg(long, value_name = "BOOKMARK")]
    pub opening: Option<Option<String>>,

    /// What `--opening` leaves out: `local`, the bookmark is already
    /// at the work `@-` and is pushed; `remote`, it is created and
    /// not pushed; `both`, it is only checked
    #[arg(long, value_enum, value_name = "SIDE", requires = "opening")]
    pub skip: Option<Skip>,

    /// The bookmark to advance to the work commit, locally, one of the
    /// nearest below the work `@-` [default: the line's, or
    /// `--opening`'s]
    #[arg(long, value_name = "NAME", conflicts_with_all = ["opening", "no_advance"])]
    pub bookmark: Option<String>,

    /// Leave the line's bookmark where it is, rather than advancing it
    /// to the work commit
    #[arg(long)]
    pub no_advance: bool,

    /// Push what was committed, over `[commit] push = false`
    #[arg(long, conflicts_with = "no_push")]
    pub push: bool,

    /// Keep the commit local, over `[commit] push`, true when absent
    #[arg(long)]
    pub no_push: bool,

    /// Print the descriptions it would write, trailers included, and
    /// commit nothing
    #[command(flatten)]
    pub dry_run: DryRunFlag,
}

/// What `--skip` leaves out of `--opening`'s bookmark.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Skip {
    /// The bookmark exists at the work `@-` already, so it is not
    /// created, only pushed.
    Local,
    /// The bookmark is created and not pushed.
    Remote,
    /// The bookmark exists and is neither created nor pushed.
    Both,
}

/// Which local bookmark the commit advances to the work commit.
#[derive(Debug, PartialEq, Eq)]
pub enum Advance {
    /// The line's: `--opening`'s, else the nearest below the work `@-`.
    Line,
    /// The one `--bookmark` names.
    Named(String),
    /// None, `--no-advance`.
    Off,
}

/// `--opening`'s bookmark and what `--skip` leaves out of it.
#[derive(Debug)]
pub struct Opening {
    pub bookmark: String,
    pub skip: Option<Skip>,
}

impl Opening {
    /// Whether the bookmark is created here, rather than found.
    fn creates(&self) -> bool {
        !matches!(self.skip, Some(Skip::Local | Skip::Both))
    }

    /// Whether the bookmark is pushed here.
    fn pushes(&self) -> bool {
        !matches!(self.skip, Some(Skip::Remote | Skip::Both))
    }
}

/// The opening's title: the bare cycle title with " opening"
/// appended, refused when it carries the suffix already, so it is
/// never doubled.
fn opening_title(title: &str) -> Result<String, String> {
    refuse_suffixed(title, "--opening")?;
    Ok(format!("{} opening", title.trim_end()))
}

/// The opening's bookmark: the one named, else the bare cycle title's
/// slug, the rules' name for a cycle's bookmark, the anchor algorithm
/// a heading of the title gets, printed since it is a default.
fn opening_bookmark(named: Option<&str>, title: &str) -> Result<String, String> {
    if let Some(name) = named {
        return Ok(name.to_string());
    }
    let slug = crate::validate_anchors::anchor_slug(title.trim_end());
    if slug.trim_matches(['-', '_']).is_empty() {
        return Err(format!(
            "commit --opening: the title '{title}' has no letters or digits to name the \
             bookmark: name it, --opening <BOOKMARK>"
        ));
    }
    info!("commit --opening: the bookmark is '{slug}', the title's slug");
    Ok(slug)
}

/// `--closing`'s revision, none for a single-step cycle, and how the
/// cycle closes.
#[derive(Debug)]
pub struct Closing {
    pub rev: Option<String>,
    pub dead_end: bool,
}

/// Refuse a bare cycle title that carries a bookend's suffix, since
/// `--opening` and `--closing` append it and it is never doubled.
fn refuse_suffixed(title: &str, flag: &str) -> Result<(), String> {
    for suffix in [" opening", " closing"] {
        if title.trim_end().ends_with(suffix) {
            return Err(format!(
                "commit {flag}: the title '{title}' ends \"{suffix}\" already: pass the bare \
                 cycle title, and {flag} appends the bookend's"
            ));
        }
    }
    Ok(())
}

/// Inputs to the commit op, flat, owned, clap-free: the repos `-s`
/// and `-R` resolved to, and the description.
#[derive(Debug)]
pub struct CommitParams {
    /// The pair, work then agent, or the one plain repo.
    pub target: Target,
    /// The title, the opening's with " opening" appended, and the
    /// body. Either is None only with `--closing`, which derives it.
    pub title: Option<String>,
    pub body: Option<String>,
    /// Close the cycle, finalizing its one commit or making the
    /// closing commit.
    pub closing: Option<Closing>,
    /// Make the work commit the cycle's opening, with its bookmark.
    pub opening: Option<Opening>,
    /// The local bookmark advanced to the work commit.
    pub advance: Advance,
    /// Push what was committed: the flag given, else `[commit] push`.
    pub push: Option<bool>,
    /// Print the descriptions and commit nothing.
    pub dry_run: bool,
}

/// What `-s` resolved to.
#[derive(Debug)]
pub enum Target {
    /// A dual workspace's two repos, work then agent. With
    /// `agent_now` the agent commit is made too, else it is left to
    /// push, the work commit's trailer naming the agent `@` either
    /// way.
    Pair { pair: Pair, agent_now: bool },
    /// A plain repo with no partner.
    Por(PathBuf),
    /// The agent repo alone, an unpaired commit, the pair kept to
    /// tell whether its `@` is a work commit's pending partner.
    Agent(Pair),
}

impl TryFrom<&CommitArgs> for CommitParams {
    type Error = String;

    /// Resolve the scope into its repos, refusing the agent half
    /// alone, read the body when it names a file, and check the
    /// description is not blank.
    fn try_from(a: &CommitArgs) -> Result<Self, String> {
        let body = match &a.body {
            Some(b) => Some(crate::desc_helpers::read_body(b, "commit")?),
            None => None,
        };
        if a.title.as_deref().is_some_and(|t| t.trim().is_empty())
            || body.as_deref().is_some_and(|b| b.trim().is_empty())
        {
            return Err("commit: --title and --body must both be non-empty".into());
        }
        let target = match &a.scope {
            RepoScope::Por => {
                let repos = crate::common::resolve_scoped_repos(&a.scope, a.repo.as_deref())
                    .map_err(|e| e.to_string())?;
                match repos.as_slice() {
                    [repo] => Target::Por(repo.clone()),
                    other => {
                        return Err(format!(
                            "commit -s p: expected one repo, got {}",
                            other.len()
                        ));
                    }
                }
            }
            RepoScope::Sides(sides) => {
                // Both paths whatever the side, since `w` needs the
                // agent @'s change id for its trailer.
                let pair = resolve_pair(a.repo.as_deref()).map_err(|e| e.to_string())?;
                if sides.is_bot_only() {
                    if a.opening.is_some() || a.closing.is_some() {
                        return Err("commit -s a: a bookend is the work repo's, so `--opening` \
                                    and `--closing` take `-s b` or `-s w`"
                            .into());
                    }
                    Target::Agent(pair)
                } else {
                    Target::Pair {
                        pair,
                        agent_now: sides.is_both(),
                    }
                }
            }
        };
        let (title, opening) = match (&a.opening, &a.title) {
            (Some(named), Some(t)) => (
                Some(opening_title(t)?),
                Some(Opening {
                    bookmark: opening_bookmark(named.as_deref(), t)?,
                    skip: a.skip,
                }),
            ),
            (Some(_), None) => return Err("commit --opening: --title is required".into()),
            (None, t) => (t.clone(), None),
        };
        if a.closing.is_some()
            && let Some(t) = &title
        {
            refuse_suffixed(t, "--closing")?;
        }
        Ok(CommitParams {
            target,
            title,
            body,
            closing: a.closing.as_ref().map(|rev| Closing {
                rev: rev.clone(),
                dead_end: a.dead_end,
            }),
            opening,
            advance: match (&a.bookmark, a.no_advance) {
                (_, true) => Advance::Off,
                (Some(b), false) => Advance::Named(b.clone()),
                (None, false) => Advance::Line,
            },
            push: match (a.push, a.no_push) {
                (true, _) => Some(true),
                (_, true) => Some(false),
                _ => None,
            },
            dry_run: a.dry_run.value,
        })
    }
}

impl SubcommandRunner for CommitArgs {
    type Params = CommitParams;

    /// Delegate to the `TryFrom<&CommitArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        CommitParams::try_from(self)
    }

    /// Run the `commit` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        commit(ctx, params)
    }
}

/// Entry point: the pair, or the plain commit.
pub fn commit(ctx: &mut Context, params: &CommitParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("commit: enter {params:?}");
    match &params.target {
        Target::Pair { pair, agent_now } => commit_pair(ctx, pair, *agent_now, params),
        Target::Por(repo) => commit_por(ctx, repo, params),
        Target::Agent(pair) => commit_agent(ctx, pair, params),
    }
}

/// Refuse a repo that cannot take the commit: a conflicted one, or
/// for the work side an empty `@`, since a rung changes the work
/// repo and an empty commit would carry a trailer naming nothing
/// worth reviewing.
fn preflight(
    repo: &Path,
    label: &str,
    must_change: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::pair::refuse_conflicts(repo, "commit", label)?;
    if must_change && jj::is_empty(repo, "@")? {
        return Err(format!(
            "commit: nothing to commit, the {label} repo's @ is empty ('{}')",
            repo.display()
        )
        .into());
    }
    Ok(())
}

/// Make the pair: both change ids read from the working copies
/// first, then the work commit with the agent's id in its trailer
/// and, with `agent_now`, the agent commit with the work's id in
/// its own. Without it the agent `@` is left for push to commit as
/// the partner, its id already in the work commit's trailer.
fn commit_pair(
    ctx: &mut Context,
    pair: &Pair,
    agent_now: bool,
    params: &CommitParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let (work, agent) = (pair.work.as_path(), pair.agent.as_path());
    let closing = plan_closing(work, params)?;
    let record = match &closing {
        Some(_) => RecordMove::plan(work)?,
        None => None,
    };
    if let Some(ClosingPlan::Finalize { title, body }) = &closing {
        let body = finalized_body(
            work,
            body.as_deref(),
            record.as_ref().map(|r| r.name.as_str()),
        )?;
        if let Some(r) = &record {
            r.write(params.dry_run)?;
        }
        finalize(ctx, work, Some(pair), title, Some(&body), params.dry_run)?;
        return push_after(ctx, work, Publish::Pair, None, params, params.dry_run);
    }
    preflight(work, "work", record.is_none())?;
    preflight(agent, "agent", false)?;
    warn_unflagged_closing(params);
    let (text, merge) = description(params, closing)?;
    let plan = plan_bookmarks(work, params)?;
    let work_chid = jj::chid_of(work, "@")?;
    let agent_chid = jj::chid_of(agent, "@")?;
    let agent_prefix = ochid_prefix_for(agent)?;
    let work_desc = with_trailer(&text, &format!("{agent_prefix}{agent_chid}"));
    let agent_desc = with_trailer(&text, &format!("/{work_chid}"));

    if params.dry_run {
        if let Some((base, tip)) = &merge {
            info!(
                "commit: [dry-run] the work commit is the merge of {base} and the last rung {tip}"
            );
        }
        dry_run_bookmarks(params, &plan);
        if let Some(r) = &record {
            r.write(true)?;
        }
        info!(
            "commit: [dry-run] the work commit {work_chid}, in {}:\n{work_desc}",
            work.display()
        );
        if agent_now {
            info!(
                "commit: [dry-run] the agent commit {agent_chid}, in {}:\n{agent_desc}",
                agent.display()
            );
        } else {
            info!(
                "commit: [dry-run] the agent @ {agent_chid} is left for push to commit as the partner"
            );
        }
        return push_after(
            ctx,
            work,
            Publish::Pair,
            plan.advance.as_deref(),
            params,
            true,
        );
    }
    if let Some(r) = &record {
        r.write(false)?;
    }
    let snapshots = Snapshots::take(pair)?;
    if let Some((base, tip)) = &merge
        && let Err(e) = make_closing_merge(ctx, work, base, tip)
    {
        snapshots.restore(pair, "commit", e.as_ref());
        return Err(e);
    }
    info!("commit work: {} -> @- {work_chid}", work.display());
    if let Err(e) = ctx.session(work)?.commit(&work_desc) {
        snapshots.restore(pair, "commit", e.as_ref());
        return Err(e);
    }
    if !agent_now {
        finish_bookmarks(ctx, work, params, &plan)?;
        info!(
            "commit: the work half is made, its trailer naming the agent @ {agent_chid}, which \
             push commits as the partner. Review it with `vc-x1 show -s work @- -R {}`",
            work.display()
        );
        return push_after(
            ctx,
            work,
            Publish::Pair,
            plan.advance.as_deref(),
            params,
            false,
        );
    }
    info!("commit agent: {} -> @- {agent_chid}", agent.display());
    if let Err(e) = ctx.session(agent)?.commit(&agent_desc) {
        snapshots.restore(pair, "commit", e.as_ref());
        return Err(e);
    }
    finish_bookmarks(ctx, work, params, &plan)?;
    info!(
        "commit: the pair is made, review it with `vc-x1 show -s both @- -R {}`",
        work.display()
    );
    push_after(
        ctx,
        work,
        Publish::Pair,
        plan.advance.as_deref(),
        params,
        false,
    )
}

/// The doing file, the closed file, and the closed file's name as
/// written.
type RecordFiles = (PathBuf, PathBuf, String);

/// The record's two files, `[record] doing` and `closed`, joined to
/// the config's directory, when both are set, with the closed file's
/// name as written for the pointer.
fn record_files(repo: &Path) -> Result<Option<RecordFiles>, Box<dyn std::error::Error>> {
    let Some(cfg) = crate::config_md::load(repo)? else {
        return Ok(None);
    };
    let get = |k: &str| crate::toml_simple::toml_get(&cfg.map, k).cloned();
    let (Some(doing), Some(closed)) = (get("record.doing"), get("record.closed")) else {
        return Ok(None);
    };
    let dir = cfg.path.parent().unwrap_or(repo); // OK: a config file always has a parent dir
    Ok(Some((dir.join(&doing), dir.join(&closed), closed)))
}

/// The record moved, as text: the doing file's `## In Progress` block
/// and its `# References` into the closed file's `## Closed` and
/// `# References`, replacing what they held, and the doing file's
/// `## In Progress` reset, its references gone. Refused when a
/// heading is missing or nothing is in progress.
fn moved_record(doing: &str, closed: &str) -> Result<(String, String), String> {
    const EMPTY: &str = "_No cycle currently in progress._";
    let split = |text: &str, heading: &str, file: &str| -> Result<(usize, usize, usize), String> {
        let at = text
            .find(&format!("\n{heading}\n"))
            .map(|i| i + 1)
            .ok_or_else(|| format!("commit --closing: the {file} file has no `{heading}`"))?;
        let body = at + heading.len() + 1;
        let refs = text[body..].find("\n# References\n").map(|i| body + i + 1);
        Ok((at, body, refs.unwrap_or(text.len())))
    };
    let (_, d_body, d_refs) = split(doing, "## In Progress", "doing")?;
    let block = doing[d_body..d_refs].trim();
    if block.is_empty() || block == EMPTY {
        return Err("commit --closing: nothing is in progress in the doing file to move".into());
    }
    let refs = doing[d_refs..]
        .strip_prefix("# References\n")
        .unwrap_or("")
        .trim();
    let new_doing = format!("{}\n{EMPTY}\n", &doing[..d_body]);
    let (_, c_body, _) = split(closed, "## Closed", "closed")?;
    let mut new_closed = format!("{}\n{block}\n", &closed[..c_body]);
    if !refs.is_empty() {
        new_closed.push_str(&format!("\n# References\n\n{refs}\n"));
    }
    Ok((new_doing, new_closed))
}

/// A record move planned before anything is made: the two files and
/// their new text, and the closed file's name for a finalized body's
/// pointer.
#[derive(Debug)]
struct RecordMove {
    doing: PathBuf,
    closed: PathBuf,
    new_doing: String,
    new_closed: String,
    name: String,
}

impl RecordMove {
    /// Plan the move when `[record]` names both files, reading and
    /// checking them, so a refusal comes before anything is written.
    fn plan(repo: &Path) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let Some((doing, closed, name)) = record_files(repo)? else {
            return Ok(None);
        };
        let (new_doing, new_closed) = moved_record(
            &std::fs::read_to_string(&doing)?,
            &std::fs::read_to_string(&closed)?,
        )?;
        Ok(Some(Self {
            doing,
            closed,
            new_doing,
            new_closed,
            name,
        }))
    }

    /// Write the move into the working copy, so the commit carries it,
    /// or with `dry_run` say what would move.
    fn write(&self, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
        let (doing, closed) = (self.doing.display(), self.closed.display());
        if dry_run {
            info!("commit: [dry-run] move the In Progress block of {doing} into {closed}");
            return Ok(());
        }
        info!("commit --closing: move the In Progress block of {doing} into {closed}");
        std::fs::write(&self.doing, &self.new_doing)?;
        std::fs::write(&self.closed, &self.new_closed)?;
        Ok(())
    }
}

/// Which push publishes a commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Publish {
    /// A pair, `push -s b`: the agent half completed as the partner,
    /// the work bookmark published.
    Pair,
    /// A plain repo, `push -s p`.
    Por,
}

/// Whether to push what was committed: the flag given, else the
/// repo's `[commit] push`, else true. A commit that left its bookmark
/// where it was, `--no-advance`, pushes only when `--push` asks, since
/// a push would move that bookmark.
fn wants_push(repo: &Path, params: &CommitParams) -> Result<bool, Box<dyn std::error::Error>> {
    if let Some(push) = params.push {
        return Ok(push);
    }
    if params.advance == Advance::Off {
        return Ok(false);
    }
    let Some(cfg) = crate::config_md::load(repo)? else {
        return Ok(crate::config_schema::COMMIT_PUSH_DEFAULT);
    };
    match crate::toml_simple::toml_get(&cfg.map, "commit.push").map(String::as_str) {
        None => Ok(crate::config_schema::COMMIT_PUSH_DEFAULT),
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        Some(other) => Err(format!(
            "commit.push: invalid bool {other:?}: expected true or false, unquoted"
        )
        .into()),
    }
}

/// Push what was committed, as `vc-x1 push` would after it, when the
/// commit asks: the bookmark advanced, or the line's when none was,
/// and with `dry` the push's dry run, so a dry run shows it too.
fn push_after(
    ctx: &mut Context,
    repo: &Path,
    publish: Publish,
    bookmark: Option<&str>,
    params: &CommitParams,
    dry: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !wants_push(repo, params)? {
        info!("commit: kept local, `vc-x1 push` publishes it");
        return Ok(());
    }
    let scope = match publish {
        Publish::Pair => RepoScope::Sides(Scope(vec![Side::Work, Side::Bot])),
        Publish::Por => RepoScope::Por,
    };
    info!("commit: push what was committed");
    crate::push::push(
        ctx,
        &crate::push::PushParams {
            bookmark: bookmark.map(str::to_string),
            repo: Some(repo.to_path_buf()),
            scope,
            step: false,
            dry_run: dry,
            yes: true,
        },
    )
}

/// Commit the agent repo alone, unpaired: its `@` with the title and
/// body and no trailer, then its `main` set to it and pushed when the
/// commit pushes. An `@` that is a work commit's pending partner keeps
/// its change id, so the work commit's trailer still resolves, and a
/// warning says the back-trailer is missing.
fn commit_agent(
    ctx: &mut Context,
    pair: &Pair,
    params: &CommitParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let agent = pair.agent.as_path();
    preflight(agent, "agent", true)?;
    let (text, _) = description(params, None)?;
    let chid = jj::chid_of(agent, "@")?;
    if let Some(work) = crate::pair::work_commit(pair)?
        && let crate::pair::Partner::Pending(at) = crate::pair::partner(pair, &work)?
    {
        warn!(
            "commit -s a: the agent @ {at} is the pending partner of the work commit {}, which \
             names it, and this commit carries no trailer back to it",
            work.chid
        );
    }
    let push = wants_push(&pair.work, params)?;
    if params.dry_run {
        info!(
            "commit: [dry-run] the agent commit {chid}, in {}:\n{text}",
            agent.display()
        );
        if push {
            info!("commit: [dry-run] then set the agent main to it and push it");
        }
        return Ok(());
    }
    let snapshot = crate::pair::Snapshot::take(agent)?;
    info!("commit agent: {} -> @- {chid}, unpaired", agent.display());
    if let Err(e) = ctx.session(agent)?.commit(&text) {
        snapshot.restore("commit", e.as_ref());
        return Err(e);
    }
    if !push {
        info!("commit: kept local, `vc-x1 push -s a` publishes it");
        return Ok(());
    }
    info!("commit agent: set main to @- and push it");
    let session = ctx.session(agent)?;
    session.bookmark_set("main", "@-")?;
    session.git_push_bookmark("main")
}

/// What `--closing` does, decided before anything is made.
#[derive(Debug)]
enum ClosingPlan {
    /// A branch of one commit above `REV`, the opening: a single-step
    /// cycle, its one commit finalized in place under the bare cycle
    /// title, with the body given or else the opening's own.
    Finalize { title: String, body: Option<String> },
    /// A longer branch: the closing commit, the merge of `base` and
    /// the last rung `tip`.
    Merge {
        base: String,
        tip: String,
        title: String,
        body: String,
    },
}

/// Plan `--closing`, refusing before anything is made. A bare
/// `--closing` is a single-step cycle, which has no parent to name:
/// `@-` is finalized in place, and it must be an opening, unless
/// `--title` is given. `--closing <REV>` is a multi-step cycle's
/// closing commit, `REV` naming exactly one commit, its first parent,
/// with more above it than the opening alone, unless `--dead-end`,
/// whose closing keeps the opening as it was. The count checks what
/// was typed rather than choosing the shape.
fn plan_closing(
    repo: &Path,
    params: &CommitParams,
) -> Result<Option<ClosingPlan>, Box<dyn std::error::Error>> {
    let Some(c) = &params.closing else {
        return Ok(None);
    };
    let Some(rev) = &c.rev else {
        if c.dead_end {
            return Err(
                "commit --closing --dead-end: a dead end gets a closing commit, so name \
                        the revision it merges onto, --closing <REV>"
                    .into(),
            );
        }
        let title = match &params.title {
            Some(t) => t.trim_end().to_string(),
            None => {
                let desc = jj::desc_of(repo, "@-")?;
                let first = desc.lines().next().unwrap_or_default().trim_end();
                match first.strip_suffix(" opening") {
                    Some(cycle) => cycle.to_string(),
                    None => {
                        return Err(format!(
                            "commit --closing: a bare --closing finalizes a single-step cycle's \
                             opening, and the work @- is '{first}': a multi-step closing names \
                             its parent, --closing <REV>, or pass --title"
                        )
                        .into());
                    }
                }
            }
        };
        info!(
            "commit --closing: a single-step cycle, its one commit finalized in place as '{title}'"
        );
        return Ok(Some(ClosingPlan::Finalize {
            title,
            body: params.body.clone(),
        }));
    };
    let base = resolve_closing(repo, rev)?;
    info!("commit --closing: '{rev}' is {base}");
    let nodes = jj::cids_of(repo, &format!("{base}..@-"))?;
    if nodes.is_empty() {
        return Err(format!(
            "commit --closing: nothing above '{rev}' to close, the work @- being in ::{rev}: a \
             cycle starts with `commit --opening`"
        )
        .into());
    }
    let cycle = match &params.title {
        Some(t) => t.trim_end().to_string(),
        None => cycle_title(repo, &base, rev)?,
    };
    if nodes.len() == 1 && !c.dead_end {
        return Err(format!(
            "commit --closing: only the opening is above '{rev}', a single-step cycle, which has \
             no parent to name: a bare --closing finalizes it"
        )
        .into());
    }
    let suffix = if c.dead_end {
        " dead-end closing"
    } else {
        " closing"
    };
    let body = match &params.body {
        Some(b) => b.clone(),
        None => default_body(repo, &cycle)?,
    };
    info!(
        "commit --closing: {} commits above '{rev}', so the closing commit merges it and the \
         last rung",
        nodes.len()
    );
    Ok(Some(ClosingPlan::Merge {
        base,
        tip: jj::cid_of(repo, "@-")?,
        title: format!("{cycle}{suffix}"),
        body,
    }))
}

/// `--closing`'s revision as one commit id, refused unless it names
/// exactly one commit.
fn resolve_closing(repo: &Path, rev: &str) -> Result<String, Box<dyn std::error::Error>> {
    let cids = jj::cids_of(repo, rev).map_err(|e| {
        format!(
            "commit --closing: '{rev}' does not resolve in {}: {e}",
            repo.display()
        )
    })?;
    match cids.as_slice() {
        [one] => Ok(one.clone()),
        other => Err(format!(
            "commit --closing: '{rev}' names {} commits in {}, and a cycle closes onto exactly \
             one",
            other.len(),
            repo.display()
        )
        .into()),
    }
}

/// The cycle title read from the opening, the first commit above
/// `base`, its " opening" removed. Refused when the branch has no one
/// first commit or it is not an opening, asking for `--title`.
fn cycle_title(repo: &Path, base: &str, rev: &str) -> Result<String, Box<dyn std::error::Error>> {
    let roots = jj::cids_of(repo, &format!("roots({base}..@-)"))?;
    let [root] = roots.as_slice() else {
        return Err(format!(
            "commit --closing: the branch above '{rev}' has {} first commits, so there is no \
             one opening to read the cycle title from: pass --title",
            roots.len()
        )
        .into());
    };
    let desc = jj::desc_of(repo, root)?;
    let first = desc.lines().next().unwrap_or_default().trim_end();
    match first.strip_suffix(" opening") {
        Some(cycle) => {
            info!("commit --closing: the cycle is '{cycle}', read from the opening");
            Ok(cycle.to_string())
        }
        None => Err(format!(
            "commit --closing: the first commit above '{rev}', '{first}', is not an opening, \
             so the cycle title cannot be read from it: pass --title"
        )
        .into()),
    }
}

/// The closing commit's default body, `[closing] body` with its
/// `{title}` the cycle title, refused when the key is absent.
fn default_body(repo: &Path, cycle: &str) -> Result<String, Box<dyn std::error::Error>> {
    let body = crate::config_md::load(repo)?
        .and_then(|cfg| crate::toml_simple::toml_get(&cfg.map, "closing.body").cloned());
    match body {
        Some(b) => Ok(b.replace("{title}", cycle)),
        None => Err(
            "commit --closing: no --body, and no `[closing] body` in the config to default it from"
                .into(),
        ),
    }
}

/// The closing merge's two sources, `REV` then the last rung.
type MergeSources = (String, String);

/// The description of a commit that is not finalized in place, with
/// the closing merge's sources when it is the closing commit.
fn description(
    params: &CommitParams,
    closing: Option<ClosingPlan>,
) -> Result<(String, Option<MergeSources>), Box<dyn std::error::Error>> {
    match closing {
        Some(ClosingPlan::Merge {
            base,
            tip,
            title,
            body,
        }) => Ok((format!("{title}\n\n{body}"), Some((base, tip)))),
        _ => match (&params.title, &params.body) {
            (Some(title), Some(body)) => Ok((format!("{title}\n\n{body}"), None)),
            _ => Err("commit: --title and --body are required without --closing".into()),
        },
    }
}

/// Without `--closing` a title ending " closing" may be a closing
/// meant and not asked for, so it is warned about, never refused, a
/// title like "fix: the door stops closing" being just a title.
fn warn_unflagged_closing(params: &CommitParams) {
    if params.closing.is_none()
        && let Some(t) = &params.title
        && t.trim_end().ends_with(" closing")
    {
        warn!(
            "commit: the title '{t}' ends \" closing\": if it closes the cycle, pass \
             --closing <REV> with the bare cycle title"
        );
    }
}

/// A finalized commit's body: the one given, else the opening's own,
/// with the record's pointer, "See details in <closed>.", added when
/// the record moved and the kept body does not say it already.
fn finalized_body(
    repo: &Path,
    given: Option<&str>,
    pointer: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(b) = given {
        return Ok(b.to_string());
    }
    let body = opening_body(repo)?;
    Ok(match pointer {
        Some(closed) if !body.contains(&format!("See details in {closed}")) => {
            format!("{body}\n\nSee details in {closed}.")
        }
        _ => body,
    })
}

/// The opening's body: its description less the title line and the
/// `ochid:` trailers.
fn opening_body(repo: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let desc = jj::desc_of(repo, "@-")?;
    Ok(desc
        .lines()
        .skip(1)
        .filter(|l| !l.starts_with("ochid:"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string())
}

/// Finalize a single-step cycle's one commit in place: fold the work
/// `@` into it and give it the bare cycle title, its body the one
/// given or else its own, its trailer kept, so its change id and its
/// partner stand. The agent `@` is left for push, which commits or
/// folds it as the partner. The line's bookmark follows the rewrite.
fn finalize(
    ctx: &mut Context,
    repo: &Path,
    pair: Option<&Pair>,
    title: &str,
    body: Option<&str>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    preflight(repo, "work", false)?;
    let trailer = match pair {
        Some(pair) => match crate::pair::work_commit(pair)? {
            Some(wc) => Some(wc.trailer),
            None => {
                return Err(
                    "commit --closing: the work @- carries no `ochid:` trailer, so it \
                            is not a pair's commit to finalize"
                        .into(),
                );
            }
        },
        None => None,
    };
    let body = match body {
        Some(b) => b.to_string(),
        None => opening_body(repo)?,
    };
    let text = format!("{title}\n\n{body}");
    let desc = match &trailer {
        Some(t) => with_trailer(&text, t),
        None => text,
    };
    let fold = !jj::is_empty(repo, "@")?;
    let chid = jj::chid_of(repo, "@-")?;
    if dry_run {
        if fold {
            info!("commit: [dry-run] fold the work @ into @- {chid}");
        }
        info!(
            "commit: [dry-run] finalize @- {chid}, in {}:\n{desc}",
            repo.display()
        );
        return Ok(());
    }
    let snapshot = crate::pair::Snapshot::take(repo)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        if fold {
            info!("commit work: @ -> @- {chid}");
            ctx.session(repo)?.squash_into("@", "@-")?;
        }
        info!("commit work: finalize @- {chid} as '{title}', its trailer kept");
        ctx.session(repo)?.describe("@-", &desc)
    })();
    if let Err(e) = result {
        snapshot.restore("commit", e.as_ref());
        return Err(e);
    }
    info!(
        "commit: the single-step cycle's one commit is finalized, the agent @ left for push. \
         Review it with `vc-x1 show -s work @- -R {}`",
        repo.display()
    );
    Ok(())
}

/// What the commit does to bookmarks, resolved before anything is
/// made: where `--opening`'s bookmark starts, and the bookmark
/// advanced to the work commit.
#[derive(Debug)]
struct BookmarkPlan {
    start: Option<String>,
    advance: Option<String>,
}

/// Resolve the bookmark plan, refusing before anything is made: the
/// opening's checks, then the bookmark to advance, `--opening`'s, the
/// one named, which must be among the nearest below the work `@-` as
/// `push` requires, or the line's, refused when there are none or
/// several, since a wrong guess moves the wrong branch.
fn plan_bookmarks(
    repo: &Path,
    params: &CommitParams,
) -> Result<BookmarkPlan, Box<dyn std::error::Error>> {
    let start = match &params.opening {
        Some(o) => Some(check_opening(repo, o)?),
        None => None,
    };
    let advance = match (&params.advance, &params.opening) {
        (Advance::Off, _) => None,
        (_, Some(o)) => Some(o.bookmark.clone()),
        (Advance::Named(b), None) => {
            let nearest = jj::local_bookmarks_at(repo, "heads(::@- & bookmarks())")?;
            if !nearest.contains(b) {
                return Err(format!(
                    "commit --bookmark: '{b}' is not the nearest bookmark below the work @- in {}, \
                     the line's own ({})",
                    repo.display(),
                    nearest.join(", ")
                )
                .into());
            }
            Some(b.clone())
        }
        (Advance::Line, None) => Some(crate::squash_push::current_bookmark(repo, "@-").map_err(
            |e| format!("commit: {e} with --bookmark <NAME>, or pass --no-advance to move none"),
        )?),
    };
    Ok(BookmarkPlan { start, advance })
}

/// What the bookmark plan would do, for a dry run.
fn dry_run_bookmarks(params: &CommitParams, plan: &BookmarkPlan) {
    if let (Some(o), Some(start)) = (&params.opening, &plan.start) {
        dry_run_opening(o, start);
    }
    if let Some(name) = &plan.advance {
        info!("commit: [dry-run] advance '{name}' to the work commit, locally");
    }
}

/// Carry out the bookmark plan once the commits are made, so a failed
/// commit moves no bookmark: the opening's bookmark created at its
/// start and pushed, then the advance to the work commit, the work
/// `@-` now.
fn finish_bookmarks(
    ctx: &mut Context,
    repo: &Path,
    params: &CommitParams,
    plan: &BookmarkPlan,
) -> Result<(), Box<dyn std::error::Error>> {
    if let (Some(o), Some(start)) = (&params.opening, &plan.start) {
        make_opening_bookmark(ctx, repo, o, start)?;
    }
    if let Some(name) = &plan.advance {
        info!("commit: advance '{name}' to the work commit, locally");
        ctx.session(repo)?.bookmark_set(name, "@-")?;
    }
    Ok(())
}

/// `--opening`'s bookmark checked before anything is made, returning
/// the commit id of the work `@-`, where the cycle's branch starts and
/// the bookmark goes. A bookmark created here must not exist, and one
/// taken by `--skip` must be at `@-` already. A push must not move a
/// remote bookmark elsewhere, and `@-` must already be on a remote, so
/// the push publishes a name and nothing unreviewed.
fn check_opening(repo: &Path, o: &Opening) -> Result<String, Box<dyn std::error::Error>> {
    let name = &o.bookmark;
    let start = jj::cid_of(repo, "@-")?;
    let exists = jj::local_bookmark_exists(repo, name)?;
    if o.creates() && exists {
        return Err(format!(
            "commit --opening: the bookmark '{name}' exists in {} already: --skip local takes \
             one at the work @- and pushes it, --skip both only checks it",
            repo.display()
        )
        .into());
    }
    if !o.creates() && !jj::local_bookmarks_at(repo, "@-")?.contains(name) {
        return Err(format!(
            "commit --opening --skip: the bookmark '{name}' is not at the work @- {start} in {}, \
             where the cycle's branch starts",
            repo.display()
        )
        .into());
    }
    if o.pushes() {
        if let Some(at) = jj::remote_bookmark_targets(repo, "origin")?.get(name.as_str())
            && *at != start
        {
            return Err(format!(
                "commit --opening: the bookmark '{name}' is on origin at {at}, not the work @- \
                 {start}, and the opening moves no published bookmark"
            )
            .into());
        }
        if !jj::matches(repo, "@- & ::remote_bookmarks()")? {
            return Err(format!(
                "commit --opening: the work @- {start} is on no remote, so pushing '{name}' at it \
                 would publish unreviewed commits: --skip remote creates it and pushes nothing"
            )
            .into());
        }
    }
    Ok(start)
}

/// What `--opening` would do to its bookmark, for a dry run.
fn dry_run_opening(o: &Opening, start: &str) {
    let name = &o.bookmark;
    match (o.creates(), o.pushes()) {
        (true, true) => info!("commit: [dry-run] create '{name}' at {start} and push it to origin"),
        (true, false) => info!("commit: [dry-run] create '{name}' at {start}, pushing nothing"),
        (false, true) => info!("commit: [dry-run] push '{name}', at {start} already, to origin"),
        (false, false) => info!("commit: [dry-run] '{name}' is at {start} already, nothing to do"),
    }
}

/// Create `--opening`'s bookmark at `start` and push it, as `--skip`
/// says, after the commit, so a failed commit leaves no bookmark. A
/// failed push leaves the opening and the local bookmark, which the
/// push after the review publishes.
fn make_opening_bookmark(
    ctx: &mut Context,
    repo: &Path,
    o: &Opening,
    start: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = &o.bookmark;
    if o.creates() {
        info!("commit --opening: create '{name}' at {start}");
        ctx.session(repo)?.bookmark_set(name, start)?;
    }
    if o.pushes() {
        info!("commit --opening: push '{name}' to origin");
        ctx.session(repo)?.git_push_bookmark(name).map_err(|e| {
            format!(
                "commit --opening: the opening is made and '{name}' is at {start} locally, but \
                 its push failed: {e}. The push after the review publishes both"
            )
        })?;
    }
    Ok(())
}

/// Merge the closing's revision and the last rung into `@`, the
/// closing's edits, setting them as `@`'s parents in that order,
/// which `vc-x1 merge`'s append would not, before the commit, which
/// keeps `@`'s parents.
fn make_closing_merge(
    ctx: &mut Context,
    repo: &Path,
    base: &str,
    tip: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("commit work: the closing merges {base} and {tip} into @");
    crate::merge::set_parents(
        ctx,
        repo,
        "@",
        &[base.to_string(), tip.to_string()],
        "commit",
    )?;
    Ok(())
}

/// Commit a plain repo with the description and no trailer.
fn commit_por(
    ctx: &mut Context,
    repo: &Path,
    params: &CommitParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let closing = plan_closing(repo, params)?;
    let record = match &closing {
        Some(_) => RecordMove::plan(repo)?,
        None => None,
    };
    if let Some(ClosingPlan::Finalize { title, body }) = &closing {
        let body = finalized_body(
            repo,
            body.as_deref(),
            record.as_ref().map(|r| r.name.as_str()),
        )?;
        if let Some(r) = &record {
            r.write(params.dry_run)?;
        }
        finalize(ctx, repo, None, title, Some(&body), params.dry_run)?;
        return push_after(ctx, repo, Publish::Por, None, params, params.dry_run);
    }
    preflight(repo, "plain", record.is_none())?;
    warn_unflagged_closing(params);
    let (desc, merge) = description(params, closing)?;
    let plan = plan_bookmarks(repo, params)?;
    let chid = jj::chid_of(repo, "@")?;
    if params.dry_run {
        if let Some((base, tip)) = &merge {
            info!("commit: [dry-run] the merge of {base} and the last rung {tip}");
        }
        dry_run_bookmarks(params, &plan);
        if let Some(r) = &record {
            r.write(true)?;
        }
        info!(
            "commit: [dry-run] the commit {chid}, in {}:\n{desc}",
            repo.display()
        );
        return push_after(
            ctx,
            repo,
            Publish::Por,
            plan.advance.as_deref(),
            params,
            true,
        );
    }
    if let Some(r) = &record {
        r.write(false)?;
    }
    let snapshot = crate::pair::Snapshot::take(repo)?;
    if let Some((base, tip)) = &merge
        && let Err(e) = make_closing_merge(ctx, repo, base, tip)
    {
        snapshot.restore("commit", e.as_ref());
        return Err(e);
    }
    info!("commit: {} -> @- {chid}", repo.display());
    if let Err(e) = ctx.session(repo)?.commit(&desc) {
        snapshot.restore("commit", e.as_ref());
        return Err(e);
    }
    finish_bookmarks(ctx, repo, params, &plan)?;
    push_after(
        ctx,
        repo,
        Publish::Por,
        plan.advance.as_deref(),
        params,
        false,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{Fixture, FixturePor, chid, description, jj_ok, test_ctx};
    use crate::{Cli, Commands};
    use clap::Parser;
    use std::fs;

    fn parse(args: &[&str]) -> Result<CommitArgs, String> {
        match Cli::try_parse_from(args) {
            Ok(Cli {
                command: Some(Commands::Commit(a)),
                ..
            }) => Ok(a),
            Ok(_) => panic!("expected Commit"),
            Err(e) => Err(e.to_string()),
        }
    }

    fn args(scope: &str, repo: &Path) -> CommitArgs {
        let mut a = parse(&[
            "vc-x1",
            "commit",
            "-s",
            scope,
            "--title",
            "feat: a rung",
            "--body",
            "The problem.\n\n* A facet.\n  - Its solution.",
        ])
        .expect("parses");
        a.repo = Some(repo.to_path_buf());
        // Local unless a test asks, so a test's assertions see the
        // commit and not a push it did not ask for.
        a.no_push = true;
        a
    }

    /// `-s`, `--title`, and `--body` are required, and a blank
    /// description is refused at params time.
    #[test]
    fn scope_title_and_body_are_required() {
        let err = parse(&["vc-x1", "commit", "--title", "t", "--body", "b"]).unwrap_err();
        assert!(err.contains("--scope"), "{err}");
        let err = parse(&["vc-x1", "commit", "-s", "b", "--body", "b"]).unwrap_err();
        assert!(err.contains("--title"), "{err}");
        let fx = Fixture::new("commit-blank");
        let mut a = args("b", &fx.work);
        a.body = Some("  \n".to_string());
        let err = CommitParams::try_from(&a).unwrap_err();
        assert!(err.contains("non-empty"), "{err}");
    }

    /// `-s a` commits the agent repo alone, unpaired, with no trailer,
    /// and pushes its `main` when the commit pushes, the work repo
    /// untouched. A bookend is the work repo's, so `-s a` refuses one,
    /// and `-s p` on a side is still the resolver's refusal.
    #[test]
    fn the_agent_repo_is_committed_alone() {
        let fx = Fixture::new("commit-agent");
        let err = CommitParams::try_from(&args("p", &fx.bot)).unwrap_err();
        assert!(err.contains("the agent side"), "{err}");
        let mut a = args("a", &fx.work);
        a.closing = Some(None);
        let err = CommitParams::try_from(&a).unwrap_err();
        assert!(err.contains("a bookend is the work repo's"), "{err}");

        fs::write(fx.bot.join("notes.md"), "a fix-up\n").expect("write");
        let agent_at = chid(&fx.bot, "@");
        let work_at = chid(&fx.work, "@");
        let mut a = args("a", &fx.work);
        a.no_push = false;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the agent commit is made");

        assert_eq!(chid(&fx.bot, "@-"), agent_at);
        let desc = description(&fx.bot, "@-");
        assert!(
            desc.starts_with("feat: a rung\n\n") && !desc.contains("ochid:"),
            "{desc}"
        );
        assert_eq!(
            chid(&fx.bot, "main@origin"),
            agent_at,
            "the agent main is pushed"
        );
        assert_eq!(chid(&fx.work, "@"), work_at, "the work repo is untouched");
    }

    /// A commit pushes what it committed by default: the pair, its
    /// bookmark advanced and published and the agent `main` with it.
    /// `--no-push` keeps it local, and `[commit] push = false` too,
    /// which `--push` overrides.
    #[test]
    fn a_commit_pushes_unless_kept_local() {
        let fx = Fixture::new("commit-push");
        jj_ok(&fx.work, &["bookmark", "create", "topic", "-r", "@-"]);
        jj_ok(&fx.work, &["bookmark", "delete", "main"]);
        let remote = |name: &str| -> Option<String> {
            jj::remote_bookmark_targets(&fx.work, "origin")
                .expect("remote")
                .get(name)
                .cloned()
        };
        let mut a = args("b", &fx.work);
        a.bookmark = Some("topic".to_string());

        fs::write(fx.work.join("one.txt"), "one\n").expect("write");
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("kept local");
        assert_eq!(remote("topic"), None, "--no-push publishes nothing");

        fs::write(fx.work.join("two.txt"), "two\n").expect("write");
        fs::write(fx.bot.join("session.jsonl"), "{\"line\":2}\n").expect("write");
        a.no_push = false;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("pushed");
        assert_eq!(
            remote("topic"),
            Some(commit_id(&fx.work, "@-")),
            "the default pushes"
        );
        assert_eq!(
            chid(&fx.bot, "main@origin"),
            chid(&fx.bot, "@-"),
            "with the agent main"
        );

        let path = fx.work.join(".vc-config.md");
        let mut cfg = fs::read_to_string(&path).expect("read the config");
        cfg.push_str("\n```toml\n[commit]\npush = false\n```\n");
        fs::write(&path, cfg).expect("write the config");
        let pushed = remote("topic");
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("kept local by the key");
        assert_eq!(remote("topic"), pushed, "the key keeps it local");
        fs::write(fx.work.join("three.txt"), "three\n").expect("write");
        a.push = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("pushed by the flag");
        assert_eq!(
            remote("topic"),
            Some(commit_id(&fx.work, "@-")),
            "--push overrides"
        );
    }

    /// `-s b` makes the pair: each commit keeps its working copy's
    /// change id, carries the title and body, and names the other
    /// in its trailer, so the pair passes validate-desc in both
    /// directions with nothing pushed.
    #[test]
    fn both_makes_the_pair_with_trailers_naming_each_other() {
        let fx = Fixture::new("commit-pair");
        fs::write(fx.work.join("rung.txt"), "work\n").expect("write");
        fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write");
        let work_at = chid(&fx.work, "@");
        let agent_at = chid(&fx.bot, "@");

        let params = CommitParams::try_from(&args("b", &fx.bot)).expect("params");
        commit(&mut test_ctx(), &params).expect("the pair is made");

        assert_eq!(
            chid(&fx.work, "@-"),
            work_at,
            "the work commit keeps @'s change id"
        );
        assert_eq!(
            chid(&fx.bot, "@-"),
            agent_at,
            "the agent commit keeps @'s change id"
        );
        let work_desc = description(&fx.work, "@-");
        let agent_desc = description(&fx.bot, "@-");
        assert!(
            work_desc.starts_with("feat: a rung\n\nThe problem."),
            "{work_desc}"
        );
        assert!(
            work_desc
                .trim_end()
                .ends_with(&format!("ochid: /.claude/{agent_at}")),
            "{work_desc}"
        );
        assert!(
            agent_desc
                .trim_end()
                .ends_with(&format!("ochid: /{work_at}")),
            "{agent_desc}"
        );
        assert_eq!(
            jj_ok(&fx.work, &["log", "-r", "@", "--no-graph", "-T", "empty"]),
            "true"
        );

        let check = crate::validate_desc::ValidateDescParams {
            pos_rev: None,
            pos_count: None,
            revision: "@-".to_string(),
            limit: None,
            repos: vec![
                ("work".to_string(), fx.work.clone()),
                ("agent".to_string(), fx.bot.clone()),
            ],
            other_repo: None,
            id_len: crate::desc_helpers::DEFAULT_ID_LEN,
        };
        crate::validate_desc::validate_desc(&test_ctx(), &check).expect("the pair validates");

        // The line's bookmark advanced to the work commit, locally,
        // and origin's is still the commit below the pair.
        assert_eq!(chid(&fx.work, "main"), chid(&fx.work, "@-"));
        assert_eq!(chid(&fx.work, "main@origin"), chid(&fx.work, "@--"));
    }

    /// `-s w` makes the work half now, its trailer naming the agent
    /// `@` all the same, and leaves the agent `@` as it was for push
    /// to commit as the partner. `@PATH` reads the body from a file.
    #[test]
    fn work_makes_the_half_and_reads_the_body_from_a_file() {
        let fx = Fixture::new("commit-work-half");
        fs::write(fx.work.join("rung.txt"), "work\n").expect("write");
        fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write");
        let body_path = fx.base.join("body.md");
        let body = "From a file.\n\n* A facet.\n  - Its solution.\n";
        fs::write(&body_path, body).expect("write");
        let work_at = chid(&fx.work, "@");
        let agent_at = chid(&fx.bot, "@");

        let mut a = args("w", &fx.work);
        a.body = Some(format!("@{}", body_path.display()));
        let params = CommitParams::try_from(&a).expect("params");
        assert_eq!(params.body.as_deref(), Some(body));
        commit(&mut test_ctx(), &params).expect("the work half is made");

        assert_eq!(chid(&fx.work, "@-"), work_at);
        let work_desc = description(&fx.work, "@-");
        assert!(
            work_desc.starts_with("feat: a rung\n\nFrom a file."),
            "{work_desc}"
        );
        assert!(
            work_desc
                .trim_end()
                .ends_with(&format!("ochid: /.claude/{agent_at}")),
            "{work_desc}"
        );
        assert_eq!(chid(&fx.bot, "@"), agent_at, "the agent @ is left for push");
        assert_eq!(
            jj_ok(&fx.bot, &["log", "-r", "@", "--no-graph", "-T", "empty"]),
            "false"
        );

        a.body = Some(format!("@{}", fx.base.join("missing.md").display()));
        let err = CommitParams::try_from(&a).unwrap_err();
        assert!(err.contains("cannot read the file"), "{err}");
    }

    /// A dry run reads and prints, and commits nothing in either repo.
    #[test]
    fn a_dry_run_commits_nothing() {
        let fx = Fixture::new("commit-dry-run");
        fs::write(fx.work.join("rung.txt"), "work\n").expect("write");
        fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write");
        let (work_at, agent_at) = (chid(&fx.work, "@"), chid(&fx.bot, "@"));
        let mut a = args("b", &fx.work);
        a.dry_run.value = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("dry run");
        assert_eq!(
            chid(&fx.work, "@"),
            work_at,
            "the work @ is still the change"
        );
        assert_eq!(
            chid(&fx.bot, "@"),
            agent_at,
            "the agent @ is still the change"
        );
        assert_eq!(
            jj_ok(&fx.work, &["log", "-r", "@", "--no-graph", "-T", "empty"]),
            "false"
        );
    }

    /// A bookend flag takes the bare cycle title and refuses one that
    /// carries a bookend's suffix already, so a suffix is never
    /// doubled, while a plain commit's title is just a title.
    #[test]
    fn a_bookend_takes_the_bare_title() {
        assert!(refuse_suffixed("feat: x", "--closing").is_ok());
        for t in [
            "feat: x opening",
            "feat: x closing",
            "feat: x dead-end closing",
        ] {
            let err = refuse_suffixed(t, "--closing").unwrap_err();
            assert!(err.contains("bare cycle title"), "{t}: {err}");
        }
        let mut a = parse(&[
            "vc-x1",
            "commit",
            "-s",
            "w",
            "--title",
            "feat: x closing",
            "--body",
            "b",
            "--closing",
            "main",
        ])
        .expect("parses");
        a.repo = None;
        assert!(a.closing.is_some() && a.title.as_deref() == Some("feat: x closing"));
        let a = parse(&[
            "vc-x1",
            "commit",
            "-s",
            "w",
            "--title",
            "fix: the door stops closing",
            "--body",
            "b",
        ])
        .expect("a plain commit's title is just a title");
        assert!(a.closing.is_none());
    }

    /// The commit ids of a revision's parents, in order.
    fn parents(repo: &Path, rev: &str) -> Vec<String> {
        jj_ok(
            repo,
            &[
                "log",
                "-r",
                rev,
                "--no-graph",
                "-T",
                "parents.map(|p| p.commit_id()).join(\" \")",
            ],
        )
        .split_whitespace()
        .map(str::to_string)
        .collect()
    }

    fn commit_id(repo: &Path, rev: &str) -> String {
        jj_ok(repo, &["log", "-r", rev, "--no-graph", "-T", "commit_id"])
    }

    /// `--closing` on a ladder of two rungs makes the work commit the
    /// merge of the fork point and the last rung, its tree the
    /// ladder's plus the closing's, and a review fix folded in by
    /// `squash` keeps both parents.
    #[test]
    fn closing_makes_the_merge_and_a_squash_keeps_it() {
        let fx = Fixture::new("commit-closing");
        let base = commit_id(&fx.work, "main");
        for rung in ["one", "two"] {
            fs::write(fx.work.join(format!("{rung}.txt")), rung).expect("write");
            jj_ok(&fx.work, &["commit", "-m", &format!("feat: rung {rung}")]);
        }
        let tip = commit_id(&fx.work, "@-");
        fs::write(fx.work.join("closing.txt"), "closing\n").expect("write");
        fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write");
        let work_at = chid(&fx.work, "@");
        let agent_at = chid(&fx.bot, "@");

        let mut a = args("b", &fx.work);
        a.title = Some("feat: a cycle".to_string());
        a.closing = Some(Some("main".to_string()));
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the closing is made");

        assert_eq!(parents(&fx.work, "@-"), [base.clone(), tip.clone()]);
        assert_eq!(
            chid(&fx.work, "@-"),
            work_at,
            "the merge keeps @'s change id"
        );
        let files = jj_ok(&fx.work, &["file", "list", "-r", "@-"]);
        for f in ["one.txt", "two.txt", "closing.txt"] {
            assert!(files.contains(f), "{f} missing:\n{files}");
        }
        assert!(
            description(&fx.work, "@-")
                .trim_end()
                .ends_with(&format!("ochid: /.claude/{agent_at}"))
        );
        assert_eq!(
            parents(&fx.work, "@"),
            [commit_id(&fx.work, "@-")],
            "@ sits on the merge"
        );

        fs::write(fx.work.join("fix.txt"), "a review fix\n").expect("write");
        let fold = crate::squash::SquashParams {
            target: crate::squash::Target::Pair {
                pair: Pair {
                    work: fx.work.clone(),
                    agent: fx.bot.clone(),
                },
                sides: crate::options_flags::scope::Scope(vec![
                    crate::options_flags::scope::Side::Work,
                    crate::options_flags::scope::Side::Bot,
                ]),
            },
            text: None,
            dry_run: false,
        };
        crate::squash::squash(&mut test_ctx(), &fold).expect("the fix folds");
        assert_eq!(
            parents(&fx.work, "@-"),
            [base, tip],
            "the squash kept both parents"
        );
        assert!(jj_ok(&fx.work, &["file", "list", "-r", "@-"]).contains("fix.txt"));
    }

    /// A `main` that moved during the cycle is joined by the closing:
    /// its first parent is `main` as it is now, and its tree carries
    /// what `main` gained, with no rebase of the ladder first.
    #[test]
    fn closing_joins_a_main_that_moved() {
        let fx = Fixture::new("commit-closing-moved");
        for rung in ["one", "two"] {
            fs::write(fx.work.join(format!("{rung}.txt")), rung).expect("write");
            jj_ok(&fx.work, &["commit", "-m", &format!("feat: rung {rung}")]);
        }
        let tip = commit_id(&fx.work, "@-");
        jj_ok(&fx.work, &["bookmark", "create", "topic", "-r", "@-"]);
        jj_ok(&fx.work, &["new", "main"]);
        fs::write(fx.work.join("landed.txt"), "another cycle\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: another cycle landed"]);
        jj_ok(&fx.work, &["bookmark", "set", "main", "-r", "@-"]);
        let moved = commit_id(&fx.work, "main");
        jj_ok(&fx.work, &["new", &tip]);
        fs::write(fx.work.join("closing.txt"), "closing\n").expect("write");

        let mut a = args("w", &fx.work);
        a.title = Some("feat: a cycle".to_string());
        a.closing = Some(Some("main".to_string()));
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the closing is made");

        assert_eq!(parents(&fx.work, "@-"), [moved, tip]);
        assert_eq!(
            commit_id(&fx.work, "topic"),
            commit_id(&fx.work, "@-"),
            "the ladder's bookmark advanced to the closing"
        );
        let files = jj_ok(&fx.work, &["file", "list", "-r", "@-"]);
        for f in ["one.txt", "two.txt", "landed.txt", "closing.txt"] {
            assert!(files.contains(f), "{f} missing:\n{files}");
        }
    }

    /// `--closing` with no ladder, the work `@-` on `main`, is refused
    /// before anything is committed.
    #[test]
    fn closing_with_no_ladder_is_refused() {
        let fx = Fixture::new("commit-closing-none");
        fs::write(fx.work.join("closing.txt"), "closing\n").expect("write");
        let work_at = chid(&fx.work, "@");
        let mut a = args("b", &fx.work);
        a.title = Some("feat: a cycle".to_string());
        a.closing = Some(Some("main".to_string()));
        let params = CommitParams::try_from(&a).expect("params");
        let err = commit(&mut test_ctx(), &params)
            .expect_err("no ladder")
            .to_string();
        assert!(err.contains("nothing above"), "{err}");
        assert_eq!(chid(&fx.work, "@"), work_at, "nothing was committed");
    }

    /// An empty work `@` is nothing to commit, and the agent repo is
    /// left as it was.
    #[test]
    fn an_empty_work_at_is_refused_before_anything_is_committed() {
        let fx = Fixture::new("commit-empty");
        fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write");
        let agent_at = chid(&fx.bot, "@");
        let params = CommitParams::try_from(&args("b", &fx.work)).expect("params");
        let err = commit(&mut test_ctx(), &params)
            .expect_err("nothing to commit")
            .to_string();
        assert!(err.contains("nothing to commit"), "{err}");
        assert_eq!(chid(&fx.bot, "@"), agent_at, "the agent @ is untouched");
    }

    /// The opening's bookmark: where it is locally, and on origin.
    fn bookmark_state(repo: &Path, name: &str) -> (Option<String>, Option<String>) {
        let local = jj::local_bookmark_exists(repo, name)
            .expect("local")
            .then(|| commit_id(repo, name));
        let remote = jj::remote_bookmark_targets(repo, "origin")
            .expect("remote")
            .get(name)
            .cloned();
        (local, remote)
    }

    /// `--opening` makes the opening under the bare title with
    /// " opening" appended, and creates the bookmark at the work `@-`,
    /// where the cycle starts, and pushes it, tracked, while the
    /// opening itself stays local.
    #[test]
    fn opening_creates_and_pushes_the_bookmark() {
        let fx = Fixture::new("commit-opening");
        let start = commit_id(&fx.work, "@-");
        fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
        fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write");
        let mut a = args("b", &fx.work);
        a.title = Some("feat: a cycle".to_string());
        a.opening = Some(Some("feat-a-cycle".to_string()));
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the opening is made");

        let desc = description(&fx.work, "@-");
        assert!(desc.starts_with("feat: a cycle opening\n\n"), "{desc}");
        assert_eq!(
            bookmark_state(&fx.work, "feat-a-cycle"),
            (Some(commit_id(&fx.work, "@-")), Some(start)),
            "advanced to the opening locally, at the start on origin"
        );
        assert!(jj::has_tracked_remote(&fx.work, "feat-a-cycle", "origin").expect("tracked"));
        assert!(
            !jj::matches(&fx.work, "@- & ::remote_bookmarks()").expect("matches"),
            "the opening is not published"
        );
    }

    /// `--opening` with no value names the bookmark by the bare
    /// title's slug, the rules' name for a cycle's bookmark, and a
    /// `--title` right after it is not taken for its value. A title
    /// with nothing to slug is refused.
    #[test]
    fn opening_names_the_bookmark_from_the_title() {
        let a = parse(&[
            "vc-x1",
            "commit",
            "-s",
            "w",
            "--opening",
            "--title",
            "feat: vc-x1 single-step commit",
            "--body",
            "b",
        ])
        .expect("parses");
        assert_eq!(a.opening, Some(None));
        assert_eq!(a.title.as_deref(), Some("feat: vc-x1 single-step commit"));

        let fx = Fixture::new("commit-opening-slug");
        let start = commit_id(&fx.work, "@-");
        fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
        let mut a = args("w", &fx.work);
        a.title = Some("feat: vc-x1 single-step commit".to_string());
        a.opening = Some(None);
        let params = CommitParams::try_from(&a).expect("params");
        assert_eq!(
            params.opening.as_ref().map(|o| o.bookmark.as_str()),
            Some("feat-vc-x1-single-step-commit")
        );
        commit(&mut test_ctx(), &params).expect("the opening is made");
        assert_eq!(
            bookmark_state(&fx.work, "feat-vc-x1-single-step-commit"),
            (Some(commit_id(&fx.work, "@-")), Some(start))
        );

        a.title = Some("?!".to_string());
        let err = CommitParams::try_from(&a).unwrap_err();
        assert!(err.contains("no letters or digits"), "{err}");
    }

    /// `--skip` leaves out what it names: `remote` creates the
    /// bookmark and pushes nothing, `local` pushes one already at the
    /// work `@-`, and `both` only checks it.
    #[test]
    fn skip_leaves_out_what_it_names() {
        for (skip, made_before, pushed) in [
            (Skip::Remote, false, false),
            (Skip::Local, true, true),
            (Skip::Both, true, false),
        ] {
            let fx = Fixture::new(&format!("commit-opening-skip-{skip:?}"));
            let start = commit_id(&fx.work, "@-");
            if made_before {
                jj_ok(&fx.work, &["bookmark", "set", "feat-x", "-r", "@-"]);
            }
            fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
            let mut a = args("w", &fx.work);
            a.opening = Some(Some("feat-x".to_string()));
            a.skip = Some(skip);
            let params = CommitParams::try_from(&a).expect("params");
            commit(&mut test_ctx(), &params).expect("the opening is made");
            let (local, remote) = bookmark_state(&fx.work, "feat-x");
            assert_eq!(local, Some(commit_id(&fx.work, "@-")), "{skip:?}");
            assert_eq!(remote, pushed.then_some(start), "{skip:?}");
        }
    }

    /// Every refusal comes before anything is committed: a title that
    /// carries " opening" already, a bookmark to create that exists, a
    /// skipped one not at the work `@-`, and a push from a `@-` on no
    /// remote. `--skip`
    /// needs `--opening`, and `--opening` is not `--closing`.
    #[test]
    fn opening_refusals_commit_nothing() {
        let err = parse(&[
            "vc-x1", "commit", "-s", "b", "--title", "t", "--body", "b", "--skip", "both",
        ])
        .unwrap_err();
        assert!(err.contains("--opening"), "{err}");
        let err = parse(&[
            "vc-x1",
            "commit",
            "-s",
            "b",
            "--title",
            "t",
            "--body",
            "b",
            "--opening",
            "x",
            "--closing",
            "main",
        ])
        .unwrap_err();
        assert!(err.contains("cannot be used with"), "{err}");

        let fx = Fixture::new("commit-opening-refused");
        let mut a = args("w", &fx.work);
        a.title = Some("feat: x opening".to_string());
        a.opening = Some(Some("feat-x".to_string()));
        let err = CommitParams::try_from(&a).unwrap_err();
        assert!(err.contains("already"), "{err}");
        a.title = Some("feat: x".to_string());

        fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
        let work_at = chid(&fx.work, "@");
        let refused = |a: &CommitArgs, why: &str| {
            let params = CommitParams::try_from(a).expect("params");
            let err = commit(&mut test_ctx(), &params).expect_err(why).to_string();
            assert!(err.contains(why), "{why}: {err}");
            assert_eq!(chid(&fx.work, "@"), work_at, "{why}: nothing was committed");
        };

        jj_ok(&fx.work, &["bookmark", "set", "feat-x", "-r", "@-"]);
        refused(&a, "exists");
        a.opening = Some(Some("feat-y".to_string()));
        a.skip = Some(Skip::Both);
        refused(&a, "is not at the work @-");
        a.skip = None;

        jj_ok(&fx.work, &["new", "@-"]);
        fs::write(fx.work.join("local.txt"), "local\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: unpublished"]);
        fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
        let params = CommitParams::try_from(&a).expect("params");
        let err = commit(&mut test_ctx(), &params)
            .expect_err("unpublished")
            .to_string();
        assert!(err.contains("on no remote"), "{err}");
    }

    /// A push that would move a bookmark published elsewhere is
    /// refused before anything is committed.
    #[test]
    fn opening_moves_no_published_bookmark() {
        let fx = Fixture::new("commit-opening-published");
        fs::write(fx.work.join("side.txt"), "side\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: side"]);
        jj_ok(&fx.work, &["git", "push", "--named", "feat-x=@-"]);
        jj_ok(&fx.work, &["bookmark", "delete", "feat-x"]);
        jj_ok(&fx.work, &["new", "main"]);
        fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
        let work_at = chid(&fx.work, "@");
        let mut a = args("w", &fx.work);
        a.opening = Some(Some("feat-x".to_string()));
        let params = CommitParams::try_from(&a).expect("params");
        let err = commit(&mut test_ctx(), &params)
            .expect_err("published elsewhere")
            .to_string();
        assert!(err.contains("is on origin"), "{err}");
        assert_eq!(chid(&fx.work, "@"), work_at, "nothing was committed");
    }

    /// The line's bookmark advances to the work commit locally, and
    /// stays advanced through a squash into it, while origin's is
    /// untouched. `--no-advance` leaves it, `--bookmark` names one of
    /// the nearest, and a line with several is refused before anything
    /// is committed.
    #[test]
    fn the_lines_bookmark_advances_unless_told_otherwise() {
        let fx = Fixture::new("commit-advance");
        let start = commit_id(&fx.work, "main");
        jj_ok(&fx.work, &["bookmark", "create", "topic", "-r", "@-"]);
        fs::write(fx.work.join("one.txt"), "one\n").expect("write");
        let mut a = args("w", &fx.work);

        // Two bookmarks on the line, `main` and `topic`, and none named.
        let work_at = chid(&fx.work, "@");
        let params = CommitParams::try_from(&a).expect("params");
        let err = commit(&mut test_ctx(), &params)
            .expect_err("ambiguous")
            .to_string();
        assert!(
            err.contains("several bookmarks") && err.contains("--no-advance"),
            "{err}"
        );
        assert_eq!(chid(&fx.work, "@"), work_at, "nothing was committed");
        a.bookmark = Some("elsewhere".to_string());
        jj_ok(
            &fx.work,
            &["bookmark", "create", "elsewhere", "-r", "root()"],
        );
        let params = CommitParams::try_from(&a).expect("params");
        let err = commit(&mut test_ctx(), &params)
            .expect_err("not on the line")
            .to_string();
        assert!(err.contains("not the nearest bookmark"), "{err}");

        a.bookmark = Some("topic".to_string());
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the commit is made");
        let one = commit_id(&fx.work, "@-");
        assert_eq!(commit_id(&fx.work, "topic"), one, "topic advanced");
        assert_eq!(commit_id(&fx.work, "main"), start, "main is left");
        assert!(
            !crate::pair::is_published(&fx.work, "topic").expect("published"),
            "a bookmark moved only locally publishes nothing"
        );

        // The line is `topic` alone now, so it advances by default,
        // and a review fix squashed in keeps it on the commit.
        fs::write(fx.work.join("two.txt"), "two\n").expect("write");
        a.bookmark = None;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the commit is made");
        let two = chid(&fx.work, "@-");
        fs::write(fx.work.join("fix.txt"), "fix\n").expect("write");
        jj_ok(&fx.work, &["squash"]);
        assert_eq!(chid(&fx.work, "topic"), two, "topic follows the rewrite");
        assert!(jj_ok(&fx.work, &["file", "list", "-r", "topic"]).contains("fix.txt"));

        fs::write(fx.work.join("three.txt"), "three\n").expect("write");
        a.no_advance = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the commit is made");
        assert_eq!(chid(&fx.work, "topic"), two, "--no-advance leaves it");
        assert!(
            parse(&[
                "vc-x1",
                "commit",
                "-s",
                "w",
                "--title",
                "t",
                "--body",
                "b",
                "--bookmark",
                "x",
                "--no-advance",
            ])
            .is_err()
        );
    }

    /// `--opening` with `--no-advance` leaves its bookmark at the
    /// start, where it was created and pushed.
    #[test]
    fn an_opening_without_the_advance_stays_at_the_start() {
        let fx = Fixture::new("commit-opening-no-advance");
        let start = commit_id(&fx.work, "@-");
        fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
        let mut a = args("w", &fx.work);
        a.opening = Some(Some("feat-x".to_string()));
        a.no_advance = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the opening is made");
        assert_eq!(
            bookmark_state(&fx.work, "feat-x"),
            (Some(start.clone()), Some(start))
        );
    }

    /// A dry run of `--opening` creates and pushes nothing.
    #[test]
    fn an_opening_dry_run_makes_no_bookmark() {
        let fx = Fixture::new("commit-opening-dry-run");
        fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
        let mut a = args("w", &fx.work);
        a.opening = Some(Some("feat-x".to_string()));
        a.dry_run.value = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("dry run");
        assert_eq!(bookmark_state(&fx.work, "feat-x"), (None, None));
    }

    /// Open a cycle titled "feat: a cycle" with `--opening`, the pair
    /// made at once, returning the opening's change id.
    fn open_cycle(fx: &Fixture) -> String {
        fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
        let mut a = args("b", &fx.work);
        a.title = Some("feat: a cycle".to_string());
        a.body = Some("Why the cycle.".to_string());
        a.opening = Some(None);
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the opening is made");
        chid(&fx.work, "@-")
    }

    /// `commit --closing [REV]` with the title and body left out.
    fn closing_args(fx: &Fixture, scope: &str, rev: Option<&str>) -> CommitArgs {
        let mut a = args(scope, &fx.work);
        a.title = None;
        a.body = None;
        a.closing = Some(rev.map(str::to_string));
        a
    }

    /// Set the work config's `[closing] body`.
    fn set_closing_body(fx: &Fixture) {
        let path = fx.work.join(".vc-config.md");
        let mut cfg = fs::read_to_string(&path).expect("read the config");
        cfg.push_str("\n```toml\n[closing]\nbody = 'Closes the cycle \"{title}\".'\n```\n");
        fs::write(&path, cfg).expect("write the config");
    }

    /// A branch of one commit above `REV`, the opening, is a
    /// single-step cycle: `--closing` folds `@` into it and gives it
    /// the bare cycle title read from it, its body and trailer kept,
    /// no new commit, the agent `@` left for push, and the bookmark
    /// following the rewrite.
    #[test]
    fn a_one_commit_branch_is_finalized_in_place() {
        let fx = Fixture::new("commit-finalize");
        let base = commit_id(&fx.work, "main");
        let opening = open_cycle(&fx);
        let trailer = description(&fx.work, "@-")
            .lines()
            .last()
            .expect("trailer")
            .to_string();
        fs::write(fx.work.join("fix.txt"), "fix\n").expect("write");
        fs::write(fx.bot.join("session.jsonl"), "{\"line\":2}\n").expect("write");
        let agent_at = chid(&fx.bot, "@");

        let params = CommitParams::try_from(&closing_args(&fx, "b", None)).expect("params");
        commit(&mut test_ctx(), &params).expect("finalized");

        assert_eq!(
            chid(&fx.work, "@-"),
            opening,
            "the same change, no new commit"
        );
        assert_eq!(parents(&fx.work, "@-"), [base]);
        let desc = description(&fx.work, "@-");
        assert!(
            desc.starts_with("feat: a cycle\n\nWhy the cycle.\n"),
            "{desc}"
        );
        assert_eq!(desc.trim_end().lines().last(), Some(trailer.as_str()));
        let files = jj_ok(&fx.work, &["file", "list", "-r", "@-"]);
        assert!(
            files.contains("fix.txt") && files.contains("opening.txt"),
            "{files}"
        );
        assert_eq!(chid(&fx.bot, "@"), agent_at, "the agent @ is left for push");
        assert_eq!(
            chid(&fx.work, "feat-a-cycle"),
            opening,
            "the bookmark followed"
        );
    }

    /// A branch of two or three commits gets the closing commit, its
    /// title the cycle's read from the opening with " closing", its
    /// body `[closing] body` filled with it, and the line's bookmark
    /// advanced to it. Without the key, `--body` is required.
    #[test]
    fn a_longer_branch_gets_the_closing_commit_titled_from_the_opening() {
        for rungs in 1..=2 {
            let fx = Fixture::new(&format!("commit-closing-derived-{rungs}"));
            let base = commit_id(&fx.work, "main");
            open_cycle(&fx);
            for n in 0..rungs {
                fs::write(fx.work.join(format!("rung{n}.txt")), "rung\n").expect("write");
                jj_ok(&fx.work, &["commit", "-m", &format!("feat: rung {n}")]);
            }
            let tip = commit_id(&fx.work, "@-");
            fs::write(fx.work.join("closing.txt"), "closing\n").expect("write");

            let params =
                CommitParams::try_from(&closing_args(&fx, "w", Some("main"))).expect("params");
            let err = commit(&mut test_ctx(), &params)
                .expect_err("no body")
                .to_string();
            assert!(err.contains("[closing] body"), "{err}");

            set_closing_body(&fx);
            commit(&mut test_ctx(), &params).expect("the closing is made");
            assert_eq!(parents(&fx.work, "@-"), [base, tip], "{rungs} rungs");
            let desc = description(&fx.work, "@-");
            assert!(
                desc.starts_with("feat: a cycle closing\n\nCloses the cycle \"feat: a cycle\".\n"),
                "{desc}"
            );
            assert_eq!(
                commit_id(&fx.work, "feat-a-cycle"),
                commit_id(&fx.work, "@-")
            );
        }
    }

    /// A single-step closing names no parent and a multi-step one
    /// names one: a `REV` with only the opening above it, a bare
    /// `--closing` on a rung, and `--dead-end` with no `REV` are each
    /// refused before anything is committed.
    #[test]
    fn the_closing_form_matches_the_shape() {
        let fx = Fixture::new("commit-closing-form");
        open_cycle(&fx);
        fs::write(fx.work.join("closing.txt"), "closing\n").expect("write");
        let work_at = chid(&fx.work, "@");
        let refused = |a: &CommitArgs, why: &str| {
            let params = CommitParams::try_from(a).expect("params");
            let err = commit(&mut test_ctx(), &params).expect_err(why).to_string();
            assert!(err.contains(why), "{why}: {err}");
            assert_eq!(chid(&fx.work, "@"), work_at, "{why}: nothing was committed");
        };
        refused(
            &closing_args(&fx, "w", Some("main")),
            "a bare --closing finalizes it",
        );
        let mut a = closing_args(&fx, "w", None);
        a.dead_end = true;
        refused(&a, "--closing <REV>");

        jj_ok(&fx.work, &["commit", "-m", "feat: a rung"]);
        fs::write(fx.work.join("closing.txt"), "closing again\n").expect("write");
        let params = CommitParams::try_from(&closing_args(&fx, "w", None)).expect("params");
        let err = commit(&mut test_ctx(), &params)
            .expect_err("a rung at @-")
            .to_string();
        assert!(
            err.contains("a multi-step closing names its parent"),
            "{err}"
        );
    }

    /// `--dead-end` always makes the closing commit, a branch of the
    /// opening alone included, titled " dead-end closing".
    #[test]
    fn a_dead_end_always_gets_the_closing_commit() {
        let fx = Fixture::new("commit-closing-dead-end");
        open_cycle(&fx);
        let opening = commit_id(&fx.work, "@-");
        fs::write(fx.work.join("closing.txt"), "closing\n").expect("write");
        let mut a = closing_args(&fx, "w", Some("main"));
        a.body = Some("Why it stopped.".to_string());
        a.dead_end = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the closing is made");
        assert_eq!(
            parents(&fx.work, "@-"),
            [commit_id(&fx.work, "main"), opening]
        );
        assert!(description(&fx.work, "@-").starts_with("feat: a cycle dead-end closing\n\n"));
    }

    /// `--closing` takes any one revision: a change id closes onto that
    /// commit as a bookmark would, and a revision naming no commit, or
    /// several, or a branch whose first commit is not an opening with
    /// no `--title`, is refused before anything is committed.
    #[test]
    fn closing_takes_any_one_revision() {
        let fx = Fixture::new("commit-closing-rev");
        let base = commit_id(&fx.work, "main");
        let base_chid = chid(&fx.work, "main");
        for rung in ["one", "two"] {
            fs::write(fx.work.join(format!("{rung}.txt")), rung).expect("write");
            jj_ok(&fx.work, &["commit", "-m", &format!("feat: rung {rung}")]);
        }
        jj_ok(&fx.work, &["bookmark", "create", "topic", "-r", "@-"]);
        let tip = commit_id(&fx.work, "@-");
        fs::write(fx.work.join("closing.txt"), "closing\n").expect("write");
        let work_at = chid(&fx.work, "@");

        let mut a = closing_args(&fx, "w", Some("main"));
        a.body = Some("b".to_string());
        for (rev, why) in [
            ("nosuch", "does not resolve"),
            ("main::@-", "names 3 commits"),
            ("main", "is not an opening"),
        ] {
            a.closing = Some(Some(rev.to_string()));
            let params = CommitParams::try_from(&a).expect("params");
            let err = commit(&mut test_ctx(), &params).expect_err(rev).to_string();
            assert!(err.contains(why), "{rev}: {err}");
            assert_eq!(chid(&fx.work, "@"), work_at, "{rev}: nothing was committed");
        }

        a.closing = Some(Some(base_chid));
        a.title = Some("feat: a cycle".to_string());
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the closing is made");
        assert_eq!(parents(&fx.work, "@-"), [base, tip]);
        assert!(description(&fx.work, "@-").starts_with("feat: a cycle closing\n"));
    }

    const DOING: &str = "# Doing\n\nIntro.\n\n## Continuation notes\n\n_None._\n\n\
                         ## In Progress\n\n### feat: a cycle\n\nThe record.\n\n- [r][0]\n\n\
                         # References\n\n[0]: #r\n";
    const CLOSED: &str = "# Closed\n\nIntro.\n\n## Closed\n\n### feat: an older cycle\n\nOld.\n\n# References\n\n[9]: #old\n";

    /// The move takes the In Progress block and its references into the
    /// closed file, replacing its block and references, and resets the
    /// doing file, its references gone. Nothing in progress, or a
    /// missing heading, is refused.
    #[test]
    fn the_record_moves_as_text() {
        let (doing, closed) = moved_record(DOING, CLOSED).expect("moved");
        assert_eq!(
            doing,
            "# Doing\n\nIntro.\n\n## Continuation notes\n\n_None._\n\n## In Progress\n\n\
             _No cycle currently in progress._\n"
        );
        assert_eq!(
            closed,
            "# Closed\n\nIntro.\n\n## Closed\n\n### feat: a cycle\n\nThe record.\n\n- [r][0]\n\n\
             # References\n\n[0]: #r\n"
        );
        let err = moved_record(&doing, &closed).unwrap_err();
        assert!(err.contains("nothing is in progress"), "{err}");
        let err = moved_record(DOING, "# Closed\n").unwrap_err();
        assert!(err.contains("no `## Closed`"), "{err}");
    }

    /// Give the fixture's work repo the record files and `[record]`.
    fn set_record(fx: &Fixture) {
        fs::write(fx.work.join("zDOING.md"), DOING).expect("write");
        fs::write(fx.work.join("zCLOSED.md"), CLOSED).expect("write");
        let path = fx.work.join(".vc-config.md");
        let mut cfg = fs::read_to_string(&path).expect("read the config");
        cfg.push_str("\n```toml\n[record]\ndoing = \"zDOING.md\"\nclosed = \"zCLOSED.md\"\n```\n");
        fs::write(&path, cfg).expect("write the config");
    }

    /// The commit's copy of a file.
    fn file_at(repo: &Path, rev: &str, file: &str) -> String {
        jj_ok(repo, &["file", "show", "-r", rev, &format!("root:{file}")])
    }

    /// With `[record]` set, a closing commit carries the record moved,
    /// and a finalized commit too, its kept body gaining the pointer.
    #[test]
    fn a_closing_carries_the_record_moved() {
        for multi in [false, true] {
            let fx = Fixture::new(&format!("commit-record-{multi}"));
            open_cycle(&fx);
            if multi {
                fs::write(fx.work.join("rung.txt"), "rung\n").expect("write");
                jj_ok(&fx.work, &["commit", "-m", "feat: a rung"]);
            }
            set_record(&fx);
            let mut a = closing_args(&fx, "w", multi.then_some("main"));
            a.body = multi.then(|| "Closing.".to_string());
            let params = CommitParams::try_from(&a).expect("params");
            commit(&mut test_ctx(), &params).expect("closed");

            assert!(
                file_at(&fx.work, "@-", "zDOING.md").ends_with("_No cycle currently in progress._"),
                "multi {multi}"
            );
            assert!(file_at(&fx.work, "@-", "zCLOSED.md").contains("### feat: a cycle"));
            let desc = description(&fx.work, "@-");
            if multi {
                assert!(
                    desc.starts_with("feat: a cycle closing\n\nClosing.\n"),
                    "{desc}"
                );
            } else {
                assert!(
                    desc.starts_with(
                        "feat: a cycle\n\nWhy the cycle.\n\nSee details in zCLOSED.md.\n"
                    ),
                    "{desc}"
                );
            }
        }
    }

    /// `-s p` commits a plain repo with the description and no
    /// trailer.
    #[test]
    fn por_commits_without_a_trailer() {
        let fx = FixturePor::new("commit-por");
        fs::write(fx.work.join("plain.txt"), "plain\n").expect("write");
        let params = CommitParams::try_from(&args("p", &fx.work)).expect("params");
        commit(&mut test_ctx(), &params).expect("the plain commit is made");
        let desc = description(&fx.work, "@-");
        assert!(desc.starts_with("feat: a rung\n\nThe problem."), "{desc}");
        assert!(!desc.contains("ochid:"), "{desc}");
    }
}

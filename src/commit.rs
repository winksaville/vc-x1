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
//! - A trailer holds the partner's change id, and with an `origin`
//!   each half names the other's repo by `orepo:` above it. Neither
//!   holds a SHA, so `-s w`'s trailers are final before the agent
//!   commit is made.
//! - `-s a` commits the agent repo alone, an unpaired commit with the
//!   given title and body and no trailer, since it belongs to no
//!   rung. An agent `@` that is a work commit's pending partner keeps
//!   its change id when committed, so the work commit's trailer still
//!   resolves, and a warning says the back-trailer is missing.
//! - The commit is pushed in the same invocation, as `vc-x1 push`
//!   would after it, when asked: a pair by `push -s b`, its agent half completed
//!   as the partner and its bookmark published, a plain repo by
//!   `push -s p`, and an agent commit by publishing the agent `main`.
//!   Only with `--push`: a commit otherwise stays on the machine,
//!   `--opening`'s bookmark included, which `--local`, or its alias
//!   `--no-push`, says outright.
//! - `-s p` commits a plain repo with the title and body and no
//!   trailer, the same description discipline on a repo with no
//!   partner.
//! - `--body` is the text, or `@PATH` to read it from a file, since
//!   a body in commit-body form has markers and blank lines a shell
//!   quote mangles.
//! - The work side's `@` must hold a change, since a rung changes
//!   the work repo. The agent side's `@` is the session's journal
//!   and is committed as it stands.
//! - `REV`, the positional, is the commit the description is for:
//!   `@` by default, committed as above, else `@-` or any draft below
//!   it, described in place with no new commit, its trailer stamped,
//!   so a commit made by hand becomes a rung's. The draft must be
//!   unpublished, no remote bookmark reaching it, carry no trailer
//!   yet, and hold a change, so the describe is a draft's rewrite and
//!   never a re-describe. `@` is left as it is, as `push` leaves new
//!   work, and a draft below `@-` is committed and not pushed, since
//!   push publishes `@-`. Not with `--opening` or `--closing`, whose
//!   commits are made from `@`.
//! - `--closing <BM|REV>` closes the cycle whose tip the argument
//!   names, a bookmark or a revision as jj resolves them, required,
//!   onto `--onto`'s bookmark, also required, since no target is
//!   assumed. It is read where it is now, so a target that moved
//!   during the cycle is joined by the merge, and left out it is
//!   refused naming the likely one:
//!   - One commit above the target, the opening: a single-step cycle,
//!     finalized in place, no new commit. `@` folds into it when it
//!     sits on it, it takes the bare cycle title, its body and
//!     trailer kept, and the agent `@` is left for push, since the
//!     commit's partner is the opening's.
//!   - More: the closing commit, the target and the tip as its parents
//!     in that order, by the primitive `vc-x1 merge` uses, carrying
//!     the record move and, when the tip is `@-`, what `@` holds.
//!     `--dead-end` always makes it.
//!   - A tip that is not `@-`, any commit on any line: `@` moves
//!     onto it first and returns after, to the same commit when it
//!     held work and to a fresh empty one on its parent when it did
//!     not, so the closing is made from the tip's working copy and
//!     the user's line is as it was, the commits above the tip
//!     included, their line unclosed.
//!   - `--title` is the bare cycle title, read from the opening, the
//!     first commit above the target, when left out, and " closing" is
//!     appended, " dead-end closing" with `--dead-end`. `--body`
//!     defaults to `[closing] body`, its `{title}` the cycle title,
//!     and a finalized commit keeps the opening's.
//! - `--opening <BOOKMARK>` makes the work commit the cycle's
//!   opening: `--title` is the bare cycle title and " opening" is
//!   appended. The bookmark, the one named or the title's slug, is
//!   `vc-x1 open`'s, made first on the work `@`, so it follows the
//!   change into the opening, and the opening creates none: one missing
//!   is refused before the commit, naming `vc-x1 open`. It stays local
//!   like the rest unless `--push` asks, a rung's push.
//! - The line's bookmark advances to the work commit, locally: the
//!   nearest bookmark below the work `@-`, as `push` resolves it, or
//!   the one `--bookmark` names, any on the line at or below the work
//!   commit, so the cycle's bookmark follows a rung past a nearer one
//!   of the user's, or `--opening`'s. jj moves a bookmark
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
use crate::desc_helpers::{
    Trailers, extract_bare_id, extract_ochid_from_desc, orepo_for, repo_url_for, with_ochids,
    with_trailer, with_trailers, without_trailer_lines,
};
use crate::jj;
use crate::land::LandPlan;
use crate::options_flags::dry_run::DryRunFlag;
use crate::options_flags::scope::{REPO_SCOPE_FORMS, RepoScope, Scope, Side, repo_scope_parser};
use crate::pair::{Pair, Snapshots, resolve_pair};
use crate::subcommand::SubcommandRunner;

/// Commit the working copies as a rung's pair, or a plain repo's
/// commit, with the given title and body.
#[derive(Args, Debug)]
pub struct CommitArgs {
    /// Retired: the revision, now `-r`, refused naming it.
    #[arg(value_name = "RETIRED", hide = true)]
    pub retired: Option<String>,

    /// The commit the description is for: `@` by default, committed as
    /// the working copy; `@-` or any draft below it is described in
    /// place with no new commit, its trailer stamped, so a commit made
    /// by hand becomes a rung's. Not with `--opening` or `--closing`
    #[arg(
        short = 'r',
        long = "revision",
        value_name = "REV",
        conflicts_with_all = ["opening", "closing"]
    )]
    pub rev: Option<String>,

    /// Where: a path in the workspace for `-s b|w`, its root or
    /// either repo, or the repo itself for `-s p` [default: the
    /// current directory]
    #[arg(short = 'R', long, value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Which repos, required: `b|both` makes both commits now,
    /// `w|work` the work half with the agent half left to push,
    /// `a|agent` the agent repo alone, unpaired unless `--covers`, and
    /// `p|por` a plain repo's commit
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

    /// Close the cycle whose tip this names, a bookmark or a
    /// revision, onto `--onto`'s bookmark: one commit above it is
    /// finalized in place, more get the closing commit, the merge of
    /// the target and the tip
    #[arg(long, value_name = "BM|REV", conflicts_with = "opening")]
    pub closing: Option<String>,

    /// With `--closing`, the bookmark the cycle closes onto and
    /// `--land` lands on, the closing commit's first parent, read where
    /// it is now. Required: no target is assumed
    #[arg(long, value_name = "BOOKMARK", requires = "closing")]
    pub onto: Option<String>,

    /// With `--closing`, close the cycle as a dead end: the title ends
    /// " dead-end closing", always on a closing commit
    #[arg(long, requires = "closing")]
    pub dead_end: bool,

    /// Make the work commit the cycle's opening: " opening" is
    /// appended to the bare cycle title, and BOOKMARK, which `vc-x1
    /// open` made, must be on the work `@` [default: the bare title's
    /// slug]
    #[arg(long, value_name = "BOOKMARK")]
    pub opening: Option<Option<String>>,

    /// With `--land`, which side of the cycle's bookmark is kept:
    /// `local`, `remote`, or `both`, none deleted
    #[arg(long, value_enum, value_name = "SIDE", requires = "land")]
    pub skip: Option<Skip>,

    /// With `--closing`, land the cycle on `--onto`'s bookmark, the
    /// closing's first parent: commit the agent `@` when the cycle's
    /// commits name it, fast-forward the bookmark to the closing, and
    /// delete the cycle's bookmark, locally unless `--push`
    #[arg(long, requires = "closing")]
    pub land: bool,

    /// The bookmark to advance to the work commit, locally, any on the
    /// work commit's line, at or below it [default: the line's nearest,
    /// or `--opening`'s]
    #[arg(long, value_name = "NAME", conflicts_with_all = ["opening", "no_advance"])]
    pub bookmark: Option<String>,

    /// Leave the line's bookmark where it is, rather than advancing it
    /// to the work commit
    #[arg(long)]
    pub no_advance: bool,

    /// Publish what was committed, a commit otherwise staying on the
    /// machine
    #[arg(long, conflicts_with = "local")]
    pub push: bool,

    /// Keep it on the machine, the default, saying so: nothing is
    /// pushed, `--opening`'s bookmark included
    #[arg(long, visible_alias = "no-push")]
    pub local: bool,

    /// Stamp the `ochid:` alone, with no `orepo:` link. `-s p` stamps no
    /// trailer, so it changes nothing there
    #[arg(long)]
    pub no_url_trailer: bool,

    /// With `-s a`, the work commits the agent commit covers, a revset
    /// in the work repo taken as written: one `ochid:` each, oldest
    /// first, and each must name the agent `@` already [default: none,
    /// an unpaired commit]
    #[arg(long, value_name = "REVSET")]
    pub covers: Option<String>,

    /// Commit although the version's line is unchanged: with `[version]
    /// file` and `pattern` in the config, a work commit that changes no
    /// line of the file starting with the pattern is otherwise refused
    #[arg(long)]
    pub same_version: bool,

    /// Print the descriptions it would write, trailers included, and
    /// commit nothing
    #[command(flatten)]
    pub dry_run: DryRunFlag,
}

/// Which side of the cycle's bookmark `--land` keeps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Skip {
    /// The local bookmark is kept, only `origin`'s deleted.
    Local,
    /// The remote bookmark is kept, only the local one deleted.
    Remote,
    /// Both are kept, none deleted.
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

/// `--opening`'s bookmark, which `vc-x1 open` made on the work `@`.
#[derive(Debug)]
pub struct Opening {
    pub bookmark: String,
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

/// `--closing`'s argument, the tip's bookmark or revision, how the
/// cycle closes, and the bookmark it closes onto, `--onto`'s, `None`
/// when left out, which the plan refuses naming the likely one.
#[derive(Debug)]
pub struct Closing {
    pub rev: String,
    pub dead_end: bool,
    pub onto: Option<String>,
}

/// Refuse a bare cycle title that carries a bookend's suffix, since
/// `--opening` and `--closing` append it and it is never doubled.
pub(crate) fn refuse_suffixed(title: &str, flag: &str) -> Result<(), String> {
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
    pub push: bool,
    /// Land the cycle on the closing's `--onto` bookmark.
    pub land: bool,
    /// With `--land`, the side of the cycle's bookmark kept.
    pub skip: Option<Skip>,
    /// Stamp no `orepo:` link.
    pub no_url_trailer: bool,
    /// Print the descriptions and commit nothing.
    pub dry_run: bool,
    /// The commit the description is for, when it is not `@`: a draft
    /// described in place.
    pub rev: Option<String>,
    /// With `-s a`, the work commits it covers, a revset in the work
    /// repo, each linked by an `ochid:`.
    pub covers: Option<String>,
    /// Commit a work `@` that holds no change, a merge whose parents
    /// are its content, which `cycle --land --merge` makes.
    pub allow_empty: bool,
    /// Commit although the version's line is unchanged, the check
    /// `[version]` in the config asks for.
    pub same_version: bool,
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
        // `--land <TRUNK>` took a value before the target became
        // `--onto`'s, so a value after `--land` is that form, not the
        // retired revision positional.
        if let (Some(value), true) = (&a.retired, a.land) {
            return Err(format!(
                "commit --land: it takes no value now, the target being --onto's: `--onto {value} \
                 --land`"
            ));
        }
        if let Some(value) = &a.retired {
            return Err(crate::options_flags::retired_positional::refusal(
                "commit", value, "-r",
            ));
        }
        if a.opening.is_some() && a.no_advance {
            return Err(
                "commit --opening: its bookmark is on the opening already, made by \
                        `vc-x1 open`, so --no-advance moves nothing"
                    .into(),
            );
        }
        if a.rev.is_some() && (a.opening.is_some() || a.closing.is_some()) {
            return Err(
                "commit -r: a bookend is made from `@`, so -r takes neither \
                        --opening nor --closing"
                    .into(),
            );
        }
        if a.covers.is_some() && !matches!(&a.scope, RepoScope::Sides(s) if s.is_bot_only()) {
            return Err(
                "commit --covers: the work commits an agent commit covers, so it takes -s a".into(),
            );
        }
        if a.rev.is_some() && matches!(&a.scope, RepoScope::Sides(s) if s.is_bot_only()) {
            return Err(
                "commit -s a: -r is the work commit's, so the agent repo alone takes none".into(),
            );
        }
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
                onto: a.onto.clone(),
            }),
            opening,
            advance: match (&a.bookmark, a.no_advance) {
                (_, true) => Advance::Off,
                (Some(b), false) => Advance::Named(b.clone()),
                (None, false) => Advance::Line,
            },
            land: a.land,
            skip: a.skip,
            push: a.push,
            no_url_trailer: a.no_url_trailer,
            dry_run: a.dry_run.value,
            rev: a.rev.clone(),
            covers: a.covers.clone(),
            allow_empty: false,
            same_version: a.same_version,
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

/// Entry point: the pair, or the plain commit, a closing's detour
/// around either.
pub fn commit(ctx: &mut Context, params: &CommitParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("commit: enter {params:?}");
    match &params.target {
        Target::Pair { pair, agent_now } => {
            let now = *agent_now;
            with_detour(ctx, &pair.work, params, |ctx| {
                commit_pair(ctx, pair, now, params)
            })
        }
        Target::Por(repo) => with_detour(ctx, repo, params, |ctx| commit_por(ctx, repo, params)),
        Target::Agent(pair) => commit_agent(ctx, pair, params),
    }
}

/// How `@` returns from a closing's detour: a fresh empty commit on
/// the parent it had, when it was discardable and the move abandoned
/// it, or an edit of the same commit, which the move kept. Both by
/// change id, since a finalize rewrites the tip and rebases the
/// commits above it, giving them new commit ids.
enum Back {
    NewOn(String),
    Edit(String),
}

/// Run a commit, with `@` moved onto the closing's tip first when the
/// tip is not `@-` and returned after, so the closing is made from
/// the tip's working copy and the user's line is as it was. The move
/// is undone with the rest on a failure, and a dry run moves nothing
/// and says so.
fn with_detour(
    ctx: &mut Context,
    repo: &Path,
    params: &CommitParams,
    run: impl FnOnce(&mut Context) -> Result<(), Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(c) = &params.closing else {
        return run(ctx);
    };
    let tip = resolve_closing(repo, &c.rev)?;
    // `@` itself is a branch with nothing committed, closed from `@`.
    if jj::matches(repo, &format!("{tip} & (@ | @-)"))? {
        return run(ctx);
    }
    if params.dry_run {
        info!(
            "commit: [dry-run] '{}' is not the work @-, so @ would move onto it first and \
             return after; what follows reads the branch at @-",
            c.rev
        );
        return run(ctx);
    }
    crate::pair::refuse_conflicts(repo, "commit", "work")?;
    let discardable = jj::is_empty(repo, "@")?
        && jj::desc_of(repo, "@")?.trim().is_empty()
        && jj::local_bookmarks_at(repo, "@")?.is_empty();
    let back = if discardable {
        Back::NewOn(jj::chid_of(repo, "@-")?)
    } else {
        Back::Edit(jj::chid_of(repo, "@")?)
    };
    let snapshot = crate::pair::Snapshot::take(repo)?;
    info!(
        "commit --closing: '{}' is not the work @-, so @ moves onto it, and returns after",
        c.rev
    );
    if let Err(e) = ctx.session(repo)?.new_on(&tip) {
        snapshot.restore("commit", e.as_ref());
        return Err(e);
    }
    if let Err(e) = run(ctx) {
        snapshot.restore("commit", e.as_ref());
        return Err(e);
    }
    let returned = match &back {
        Back::NewOn(parent) => ctx.session(repo)?.new_on(parent),
        Back::Edit(chid) => ctx.session(repo)?.edit(chid),
    };
    returned.map_err(|e| {
        format!("commit --closing: the closing is made, and @ did not return to where it was: {e}")
            .into()
    })
}

/// The draft `REV` names, described in place: the revision as given.
#[derive(Debug)]
struct Draft {
    rev: String,
}

/// Resolve `REV` to the draft it names, `None` for `@` or when no
/// REV was given, refusing before anything is made: a revset that is
/// not one commit, a published commit, whose describe would be a
/// remote rewrite, one that carries a trailer already, so is a pair's
/// commit and `squash --title --body` re-describes it, a conflicted
/// one, and an empty one, since a rung changes the repo.
fn plan_draft(
    repo: &Path,
    params: &CommitParams,
) -> Result<Option<Draft>, Box<dyn std::error::Error>> {
    let Some(rev) = &params.rev else {
        return Ok(None);
    };
    let cids = jj::cids_of(repo, rev)
        .map_err(|e| format!("commit REV: '{rev}' in {}: {e}", repo.display()))?;
    if cids.len() != 1 {
        return Err(format!(
            "commit REV: '{rev}' names {} commits in {}, and the description is one commit's",
            cids.len(),
            repo.display()
        )
        .into());
    }
    if jj::matches(repo, &format!("({rev}) & @"))? {
        return Ok(None);
    }
    if crate::pair::is_published(repo, rev)? {
        return Err(format!(
            "commit REV: '{rev}' is published, an ancestor of a remote bookmark in {}, so \
             describing it is a remote rewrite",
            repo.display()
        )
        .into());
    }
    if crate::desc_helpers::extract_ochid_from_desc(&jj::desc_of(repo, rev)?).is_some() {
        return Err(format!(
            "commit REV: '{rev}' carries an `ochid:` trailer already, so it is a pair's commit: \
             `vc-x1 squash --title --body` re-describes it"
        )
        .into());
    }
    if jj::matches(repo, &format!("({rev}) & conflicts()"))? {
        return Err(format!("commit REV: '{rev}' is conflicted, so it is not described").into());
    }
    if jj::is_empty(repo, rev)? {
        return Err(format!(
            "commit REV: nothing to describe, '{rev}' is empty in {}",
            repo.display()
        )
        .into());
    }
    Ok(Some(Draft { rev: rev.clone() }))
}

/// Refuse a repo that cannot take the commit: a conflicted one, or
/// for the work side an empty `@`, since a rung changes the work
/// repo and an empty commit would carry a trailer naming nothing
/// worth reviewing. A closing is never refused for an empty `@`,
/// since the merge is its content.
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

/// Refuse a work commit that leaves the version as it was, when the
/// config asks: `[version] file` names the file that holds it and
/// `[version] pattern` what its line starts with, and `rev`, the
/// commit being made, must add or remove such a line. Without the keys
/// nothing is checked, and `same_version` commits anyway, saying so.
/// The tool reads the line and never writes it, the version being the
/// project's.
pub(crate) fn check_version(
    repo: &Path,
    rev: &str,
    same_version: bool,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(cfg) = crate::config_md::load(repo)? else {
        return Ok(());
    };
    let key = |k: &str| crate::toml_simple::toml_get(&cfg.map, k).cloned();
    let (file, pattern) = match (key("version.file"), key("version.pattern")) {
        (None, None) => return Ok(()),
        (Some(file), Some(pattern)) => (file, pattern),
        _ => {
            return Err(format!(
                "{label}: the config has one of `[version] file` and `pattern`, and the \
                 version's check needs both"
            )
            .into());
        }
    };
    if same_version {
        info!("{label}: --same-version, so the version's line in {file} is not checked");
        return Ok(());
    }
    let bumped = jj::changed_lines(repo, rev, Path::new(&file))?
        .iter()
        .any(|line| line.starts_with(&pattern));
    if bumped {
        return Ok(());
    }
    Err(format!(
        "{label}: the commit changes no line of {file} starting `{pattern}`, the version's, and \
         a commit bumps the version: bump it, or pass --same-version to commit it as it is"
    )
    .into())
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
    let tip = closing_tip(closing.as_ref()).to_string();
    let closing_bm = closing_bookmark(work, params, closing.as_ref(), &tip)?;
    let land = plan_land(work, params, &tip, closing_bm.as_deref())?;
    // A Land commits the agent `@`, naming every commit that names it,
    // the closing among them.
    let agent_now = agent_now && land.is_none();
    let record = match &closing {
        Some(_) => RecordMove::plan(work)?,
        None => None,
    };
    if let Some(ClosingPlan::Finalize { title, body }) = &closing {
        let body = finalized_body(
            work,
            body.as_deref(),
            record.as_ref().map(|_| RecordMove::NAME),
        )?;
        if let Some(r) = &record {
            r.write(params.dry_run)?;
        }
        check_version(work, "@", params.same_version, "commit")?;
        finalize(ctx, work, Some(pair), title, Some(&body), params.dry_run)?;
        return push_after(
            ctx,
            work,
            Publish::Pair,
            closing_bm.as_deref(),
            params,
            params.dry_run,
            land.as_ref(),
        );
    }
    let draft = plan_draft(work, params)?;
    match &draft {
        None => preflight(work, "work", closing.is_none() && !params.allow_empty)?,
        Some(_) => crate::pair::refuse_conflicts(work, "commit", "work")?,
    }
    preflight(agent, "agent", false)?;
    check_version(
        work,
        draft.as_ref().map_or("@", |d| d.rev.as_str()),
        params.same_version,
        "commit",
    )?;
    warn_unflagged_closing(params);
    let (text, merge) = description(params, closing)?;
    let plan = plan_bookmarks(work, params, draft.as_ref(), closing_bm.as_deref())?;
    let work_at = draft.as_ref().map_or("@", |d| d.rev.as_str());
    let work_chid = jj::chid_of(work, work_at)?;
    let agent_chid = jj::chid_of(agent, "@")?;
    // Without `--no-url-trailer`, each half names the other's repo by
    // `orepo:`. Neither trailer holds a SHA, so either half may be made
    // first and neither is restamped once the other is.
    let links = !params.no_url_trailer;
    let work_desc = with_trailers(
        &text,
        &Trailers::new(orepo_for(agent, links)?, vec![agent_chid.clone()]),
    );
    let agent_desc = with_trailer(&text, orepo_for(work, links)?, &work_chid);

    if params.dry_run {
        if let Some((base, tip)) = &merge {
            info!(
                "commit: [dry-run] the work commit is the merge of {base} and the last rung {tip}"
            );
        }
        dry_run_bookmarks(&plan);
        if let Some(r) = &record {
            r.write(true)?;
        }
        if let Some(d) = &draft {
            info!(
                "commit: [dry-run] describe {} {work_chid} in place, in {}:\n{work_desc}",
                d.rev,
                work.display()
            );
        } else {
            info!(
                "commit: [dry-run] the work commit {work_chid}, in {}:\n{work_desc}",
                work.display()
            );
        }
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
            land.as_ref(),
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
    if agent_now {
        info!("commit agent: {} -> @- {agent_chid}", agent.display());
        if let Err(e) = ctx.session(agent)?.commit(&agent_desc) {
            snapshots.restore(pair, "commit", e.as_ref());
            return Err(e);
        }
    }
    let made = match &draft {
        None => {
            info!("commit work: {} -> @- {work_chid}", work.display());
            ctx.session(work)?.commit(&work_desc)
        }
        Some(d) => {
            info!(
                "commit work: describe {} {work_chid} in place, in {}",
                d.rev,
                work.display()
            );
            ctx.session(work)?.describe(&d.rev, &work_desc)
        }
    };
    if let Err(e) = made {
        snapshots.restore(pair, "commit", e.as_ref());
        return Err(e);
    }
    if !agent_now {
        finish_bookmarks(ctx, work, &plan)?;
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
            land.as_ref(),
        );
    }
    finish_bookmarks(ctx, work, &plan)?;
    info!(
        "commit: the pair is made, review it with `vc-x1 show -s both {work_at} -R {}`",
        work.display()
    );
    push_after(
        ctx,
        work,
        Publish::Pair,
        plan.advance.as_deref(),
        params,
        false,
        land.as_ref(),
    )
}

/// The cycle-record's file, at the repo's root.
pub(crate) const RECORD: &str = "TODO.md";

/// An empty `## In Progress`.
pub(crate) const NONE_IN_PROGRESS: &str = "_No cycle currently in progress._";

/// The body of `heading`'s section in `text`, as byte offsets: from
/// the line after the heading to the next `## ` or `# ` heading, else
/// the end. A heading inside a fenced code block is not one.
pub(crate) fn section(text: &str, heading: &str) -> Option<(usize, usize)> {
    let mut body = None;
    let mut fenced = false;
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        if bare.starts_with("```") {
            fenced = !fenced;
        } else if !fenced {
            if body.is_none() && bare == heading {
                body = Some(at + line.len());
            } else if let Some(start) = body
                && (bare.starts_with("## ") || bare.starts_with("# "))
            {
                return Some((start, at));
            }
        }
        at += line.len();
    }
    body.map(|start| (start, text.len()))
}

/// Where in a section's body its content starts, the first line that
/// `starts` says begins it, outside fenced code: everything before it
/// is the section's intro, kept. The body's end when none does.
fn content_start(body: &str, starts: impl Fn(&str) -> bool) -> usize {
    let mut fenced = false;
    let mut at = 0;
    for line in body.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        if bare.starts_with("```") {
            fenced = !fenced;
        } else if !fenced && starts(bare) {
            return at;
        }
        at += line.len();
    }
    body.len()
}

/// `text` with the content of `heading`'s section, after its intro,
/// replaced by `content`, a blank line after it unless the file ends
/// there.
fn replace_content(
    text: &str,
    (start, end): (usize, usize),
    starts: impl Fn(&str) -> bool,
    content: &str,
) -> String {
    let from = start + content_start(&text[start..end], starts);
    let tail = if end == text.len() { "\n" } else { "\n\n" };
    format!(
        "{}{}{tail}{}",
        &text[..from],
        content.trim_end(),
        &text[end..]
    )
}

/// The record moved within `TODO.md`, as text: the `## In Progress`
/// block, its first `###` heading to the section's end, replaces the
/// `## Closed` block, and `## In Progress` reads "_No cycle currently
/// in progress._", each section's intro kept. `None` when there is no
/// `## In Progress` or nothing is in it, and refused when a cycle is
/// in progress and there is no `## Closed`.
fn moved_record(text: &str) -> Result<Option<String>, String> {
    let entry = |l: &str| l.starts_with("### ");
    let Some(progress) = section(text, "## In Progress") else {
        return Ok(None);
    };
    let from = progress.0 + content_start(&text[progress.0..progress.1], entry);
    let block = text[from..progress.1].trim_end();
    if block.is_empty() {
        return Ok(None);
    }
    let closed = section(text, "## Closed").ok_or_else(|| {
        format!("commit --closing: {RECORD} has no `## Closed` to move the cycle into")
    })?;
    // A section's content is a cycle's block or the empty marker.
    let content = |l: &str| entry(l) || l == NONE_IN_PROGRESS;
    // The later section first, so the earlier one's offsets still hold.
    Ok(Some(if closed.0 > progress.0 {
        let t = replace_content(text, closed, content, block);
        replace_content(&t, progress, content, NONE_IN_PROGRESS)
    } else {
        let t = replace_content(text, progress, content, NONE_IN_PROGRESS);
        replace_content(&t, closed, content, block)
    }))
}

/// A record move planned before anything is made: `TODO.md`'s new
/// text. `zTOC.md`'s ladder is left as it is, so the closing's tree,
/// the landmark, keeps the cycle's final ladder, and the next opening
/// replaces it.
#[derive(Debug)]
pub(crate) struct RecordMove {
    record: PathBuf,
    new_record: String,
}

impl RecordMove {
    /// The pointer a finalized body gains, the record's file.
    const NAME: &str = RECORD;

    /// Plan the move when the repo's `TODO.md` has a cycle in
    /// progress, reading and checking it, so a refusal comes before
    /// anything is written. Nothing to move is said and is no error.
    pub(crate) fn plan(repo: &Path) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let record = repo.join(RECORD);
        if !record.is_file() {
            info!("commit --closing: no {RECORD}, so no cycle-record moves");
            return Ok(None);
        }
        let Some(new_record) = moved_record(&std::fs::read_to_string(&record)?)? else {
            info!("commit --closing: nothing in progress in {RECORD}, so no cycle-record moves");
            return Ok(None);
        };
        Ok(Some(Self { record, new_record }))
    }

    /// Write the move into the working copy, so the commit carries it,
    /// or with `dry_run` say what would move.
    pub(crate) fn write(&self, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
        let record = self.record.display();
        let tag = if dry_run {
            "commit: [dry-run]"
        } else {
            "commit --closing:"
        };
        info!("{tag} move the In Progress block of {record} into its Closed");
        if dry_run {
            return Ok(());
        }
        std::fs::write(&self.record, &self.new_record)?;
        Ok(())
    }
}

/// The closing's bookmark, the one its commit carries and a Land
/// deletes, decided by `--closing`'s argument by the rule
/// [`crate::land::cycle_bookmark`] states, above `--onto`'s.
fn closing_bookmark(
    repo: &Path,
    params: &CommitParams,
    plan: Option<&ClosingPlan>,
    tip: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let Some(c) = &params.closing else {
        return Ok(None);
    };
    if let Some(ClosingPlan::Commit { bookmark }) = plan {
        return Ok(Some(bookmark.clone()));
    }
    let target = onto(repo, c, tip)?;
    let found =
        crate::land::cycle_bookmark(repo, "commit --closing", "--closing", &c.rev, &target, tip)?;
    if found.is_none() {
        info!(
            "commit --closing: no bookmark on the branch above '{target}', so the closing \
             carries none"
        );
    }
    Ok(found)
}

/// The closing's tip, the commit the branch is read up to, `@-` for
/// a finalized commit, which the detour has put there.
fn closing_tip(closing: Option<&ClosingPlan>) -> &str {
    match closing {
        Some(ClosingPlan::Merge { tip, .. }) => tip.as_str(),
        _ => "@-",
    }
}

/// Plan `--land` before anything is made, as [`crate::land::plan`]
/// does, onto the closing's `--onto`. The bookmark deleted is the
/// closing's, by `--closing`'s argument.
fn plan_land(
    repo: &Path,
    params: &CommitParams,
    tip: &str,
    bookmark: Option<&str>,
) -> Result<Option<LandPlan>, Box<dyn std::error::Error>> {
    let (true, Some(c)) = (params.land, &params.closing) else {
        return Ok(None);
    };
    let skip = params.skip;
    let target = onto(repo, c, tip)?;
    let plan = crate::land::plan(repo, "commit --land", target, tip, bookmark, skip, false)?;
    Ok(Some(LandPlan {
        push: params.push,
        ..plan
    }))
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

impl Publish {
    /// The push's scope.
    fn scope(self) -> RepoScope {
        match self {
            Publish::Pair => RepoScope::Sides(Scope(vec![Side::Work, Side::Bot])),
            Publish::Por => RepoScope::Por,
        }
    }
}

/// Push what was committed, as `vc-x1 push` would after it, when the
/// commit asks: the bookmark advanced, or the line's when none was,
/// and with `dry` the push's dry run, so a dry run shows it too. A
/// draft below `@-` is kept local, since push publishes `@-`.
fn push_after(
    ctx: &mut Context,
    repo: &Path,
    publish: Publish,
    bookmark: Option<&str>,
    params: &CommitParams,
    dry: bool,
    land: Option<&LandPlan>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(land) = land {
        return crate::land::run(ctx, repo, publish.scope(), land, "@-", "the closing", dry);
    }
    if params.closing.is_some() && bookmark.is_none() {
        info!(
            "commit: the closing carries no bookmark, and a push would publish it on the line's \
             nearest, the target, so it is kept local: --land lands it"
        );
        return Ok(());
    }
    if let Some(rev) = &params.rev
        && !jj::matches(repo, &format!("({rev}) & (@ | @-)"))?
    {
        info!(
            "commit: '{rev}' is below @-, which push publishes, so it is kept local: `vc-x1 push` \
             publishes it once it takes a revision"
        );
        return Ok(());
    }
    if !params.push {
        info!("commit: kept local, `vc-x1 push` publishes it");
        return Ok(());
    }
    let scope = publish.scope();
    info!("commit: push what was committed");
    crate::push::push(
        ctx,
        &crate::push::PushParams {
            revision: None,
            bookmark: bookmark.map(str::to_string),
            repo: Some(repo.to_path_buf()),
            scope,
            step: false,
            dry_run: dry,
            yes: true,
            // A commit push is a fast-forward, so a forced one is a
            // surprise, refused: `vc-x1 push` asks.
            forced: crate::common::ForcedPush::Refuse,
        },
    )
}

/// Commit the agent repo alone: its `@` with the title and body,
/// then its `main` set to it and pushed when the commit pushes.
/// Unpaired it carries no trailer, and an `@` that is a work commit's
/// pending partner keeps its change id, so the work commit's trailer
/// still resolves, and a warning says the back-trailer is missing.
/// With `--covers` it carries one `ochid:` per work commit covered,
/// the pair a cycle's rungs make with one agent commit.
fn commit_agent(
    ctx: &mut Context,
    pair: &Pair,
    params: &CommitParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let agent = pair.agent.as_path();
    preflight(agent, "agent", true)?;
    let (text, _) = description(params, None)?;
    let chid = jj::chid_of(agent, "@")?;
    let covered = match &params.covers {
        Some(revset) => Some(covered_chids(pair, revset, &chid)?),
        None => None,
    };
    let text = match &covered {
        Some(chids) => with_ochids(&text, orepo_for(&pair.work, !params.no_url_trailer)?, chids),
        None => text,
    };
    if covered.is_none()
        && let Some(work) = crate::pair::work_commit(pair)?
        && let crate::pair::Partner::Pending(at) = crate::pair::partner(pair, &work)?
    {
        warn!(
            "commit -s a: the agent @ {at} is the pending partner of the work commit {}, which \
             names it, and this commit carries no trailer back to it",
            work.chid
        );
    }
    let push = params.push;
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
    match &covered {
        Some(chids) => info!(
            "commit agent: {} -> @- {chid}, covering {} work commit(s)",
            agent.display(),
            chids.len()
        ),
        None => info!("commit agent: {} -> @- {chid}, unpaired", agent.display()),
    }
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
    session.git_push_bookmark("main", crate::jj::Force::Refuse)
}

/// The change ids of the work commits `revset` names, oldest first,
/// each refused unless its `ochid:` names `agent_at`, the change the
/// agent commit is made from, so every link is one both sides agree on.
fn covered_chids(
    pair: &Pair,
    revset: &str,
    agent_at: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let work = pair.work.as_path();
    let cids = jj::cids_of(work, revset)?;
    if cids.is_empty() {
        return Err(format!("commit --covers: '{revset}' names no work commit").into());
    }
    let mut chids = Vec::new();
    let mut strays = Vec::new();
    for cid in &cids {
        let chid = jj::chid_of(work, cid)?;
        let names = extract_ochid_from_desc(&jj::desc_of(work, cid)?)
            .is_some_and(|ochid| extract_bare_id(&ochid) == agent_at);
        if names {
            chids.push(chid);
        } else {
            strays.push(chid);
        }
    }
    if !strays.is_empty() {
        return Err(format!(
            "commit --covers: {} in '{revset}' do not name the agent @ {agent_at}, so the agent \
             commit cannot cover them: {}",
            strays.len(),
            strays.join(", ")
        )
        .into());
    }
    Ok(chids)
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
    /// A branch with nothing committed above the target, its work in
    /// `@` and `bookmark`, `vc-x1 open`'s, on it: a single-step cycle
    /// whose one commit is made now from `@`, under the bare cycle
    /// title and the body given, there being no opening to read.
    Commit { bookmark: String },
}

/// The bookmark a closing merges onto and a Land lands on, `--onto`'s,
/// required: no target is assumed, since a bookmark made is only
/// likely the one a cycle lands on. Left out, it is refused naming the
/// likely one, the nearest bookmark at or below the opening's parent,
/// the opening being the nearest commit below `tip` titled so.
fn onto(repo: &Path, closing: &Closing, tip: &str) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(o) = &closing.onto {
        return Ok(o.clone());
    }
    let opening = format!("heads(::({tip}) & description(regex:\"^[^\\n]* opening(\\n|$)\"))");
    let likely = jj::local_bookmarks_at(repo, &format!("heads(::(({opening})-) & bookmarks())"))?;
    let hint = match likely.as_slice() {
        [] => String::new(),
        [one] => format!(", likely `--onto {one}`"),
        many => format!(", likely one of {}", many.join(", ")),
    };
    Err(format!(
        "commit --closing: name the bookmark the cycle closes onto, --onto <BOOKMARK>{hint}"
    )
    .into())
}

/// Plan `--closing`, refusing before anything is made. The argument
/// names the tip, exactly one commit, and the target is `--onto`'s,
/// read where it is now. The commits between decide the shape: none
/// is refused, one, the opening, is a single-step cycle finalized in
/// place, unless `--dead-end`, and more get the closing commit, the
/// target and the tip its parents. The count reads the branch rather
/// than a flag choosing the shape.
fn plan_closing(
    repo: &Path,
    params: &CommitParams,
) -> Result<Option<ClosingPlan>, Box<dyn std::error::Error>> {
    let Some(c) = &params.closing else {
        return Ok(None);
    };
    let rev = &c.rev;
    let tip = resolve_closing(repo, rev)?;
    let target = onto(repo, c, &tip)?;
    let base = jj::cid_of(repo, &target).map_err(|e| {
        format!(
            "commit --closing: --onto '{target}' does not resolve in {}: {e}",
            repo.display()
        )
    })?;
    info!("commit --closing: close '{rev}' at {tip} onto '{target}' at {base}");
    if let Some(plan) = plan_commit_closing(repo, params, c, &tip, &base, &target)? {
        return Ok(Some(plan));
    }
    if tip == jj::cid_of(repo, "@")? {
        return Err(format!(
            "commit --closing: '{rev}' is the work @, which a closing commits only when the branch \
             has nothing committed above '{target}': name the branch's last commit, @- the common \
             one"
        )
        .into());
    }
    let nodes = jj::cids_of(repo, &format!("{base}..{tip}"))?;
    if nodes.is_empty() {
        return Err(format!(
            "commit --closing: nothing above '{target}' to close, '{rev}' being in ::{target}: a \
             cycle starts with `commit --opening`"
        )
        .into());
    }
    let cycle = match &params.title {
        Some(t) => t.trim_end().to_string(),
        None => cycle_title(repo, &base, &tip, &target)?,
    };
    if nodes.len() == 1 && !c.dead_end {
        info!(
            "commit --closing: one commit above '{target}', a single-step cycle, finalized in \
             place as '{cycle}'"
        );
        return Ok(Some(ClosingPlan::Finalize {
            title: cycle,
            body: params.body.clone(),
        }));
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
        "commit --closing: {} commits above '{target}', so the closing commit merges it and \
         '{rev}'",
        nodes.len()
    );
    Ok(Some(ClosingPlan::Merge {
        base,
        tip,
        title: format!("{cycle}{suffix}"),
        body,
    }))
}

/// The closing of a branch with nothing committed above the target,
/// the single-step cycle `vc-x1 open` leaves: the tip is the work `@`,
/// named by its bookmark or `@`, or the target itself at `@-`, and
/// `@-` is the target, so the cycle's one commit is made now from `@`.
/// `None` when the branch has commits, the other shapes' case. Refused
/// before anything is made: an empty `@`, no bookmark on `@` or several
/// with none named, `--dead-end`, which closes a cycle with commits,
/// and a title or body left out, there being no opening to read.
fn plan_commit_closing(
    repo: &Path,
    params: &CommitParams,
    c: &Closing,
    tip: &str,
    base: &str,
    target: &str,
) -> Result<Option<ClosingPlan>, Box<dyn std::error::Error>> {
    let at = jj::cid_of(repo, "@")?;
    let below = jj::cid_of(repo, "@-")?;
    if !(tip == at || tip == below) || below != base {
        return Ok(None);
    }
    let label = "commit --closing";
    if jj::is_empty(repo, "@")? {
        return Err(format!(
            "{label}: nothing above '{target}' to close, '{}' having no commit above it and \
             nothing to commit: a cycle starts with `vc-x1 open`",
            c.rev
        )
        .into());
    }
    let mut on_at = jj::local_bookmarks_at(repo, "@")?;
    on_at.retain(|b| b != target);
    let bookmark = match on_at.as_slice() {
        [] => {
            return Err(format!(
                "{label}: the work @ carries no bookmark, and a cycle runs on one: make it \
                 first, `vc-x1 open --title \"<cycle title>\"`"
            )
            .into());
        }
        [one] => one.clone(),
        many if many.contains(&c.rev) => c.rev.clone(),
        many => {
            return Err(format!(
                "{label}: the work @ carries several bookmarks, {}, so name the cycle's, \
                 --closing <BOOKMARK>",
                many.join(", ")
            )
            .into());
        }
    };
    if c.dead_end {
        return Err(format!(
            "{label} --dead-end: the branch has no commit above '{target}', and a dead end \
             closes the commits a cycle made"
        )
        .into());
    }
    if params.title.is_none() || params.body.is_none() {
        return Err(format!(
            "{label}: the branch has no commit above '{target}', so its one commit is made now \
             from the work @, and --title and --body are required, there being no opening to \
             read them from"
        )
        .into());
    }
    info!(
        "{label}: no commit above '{target}', a single-step cycle on '{bookmark}', its one \
         commit made now from the work @"
    );
    Ok(Some(ClosingPlan::Commit { bookmark }))
}

/// `--closing`'s argument as one commit id, a bookmark or a revision
/// as jj resolves them, refused unless it names exactly one commit.
fn resolve_closing(repo: &Path, rev: &str) -> Result<String, Box<dyn std::error::Error>> {
    crate::land::resolve_tip(repo, "commit --closing", rev)
}

/// The cycle title read from the opening, the first commit above
/// `base` on the way to `tip`, its " opening" removed. Refused when
/// the branch has no one first commit or it is not an opening, asking
/// for `--title`.
fn cycle_title(
    repo: &Path,
    base: &str,
    tip: &str,
    target: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let roots = jj::cids_of(repo, &format!("roots({base}..{tip})"))?;
    let [root] = roots.as_slice() else {
        return Err(format!(
            "commit --closing: the branch above '{target}' has {} first commits, so there is no \
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
            "commit --closing: the first commit above '{target}', '{first}', is not an opening, \
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
             --closing <BM|REV> with the bare cycle title"
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
/// trailers `vc-x1` stamps.
fn opening_body(repo: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let desc = jj::desc_of(repo, "@-")?;
    Ok(without_trailer_lines(&desc)
        .skip(1)
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
pub(crate) fn finalize(
    ctx: &mut Context,
    repo: &Path,
    pair: Option<&Pair>,
    title: &str,
    body: Option<&str>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    preflight(repo, "work", false)?;
    let trailers = match pair {
        // An older commit's `ocid:` and `pcid:` give way to `orepo:`.
        Some(pair) => match crate::pair::work_commit(pair)? {
            Some(wc) => Some(Trailers {
                orepo: match wc.trailers.orepo {
                    Some(orepo) => Some(orepo),
                    None => repo_url_for(&pair.agent)?,
                },
                ..wc.trailers
            }),
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
    let desc = match &trailers {
        Some(trailers) => with_trailers(&text, trailers),
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
/// made: the bookmark advanced to the work commit.
#[derive(Debug)]
struct BookmarkPlan {
    advance: Option<String>,
    /// Where the advance goes: the work `@-` once `@` is committed,
    /// or a draft's change id, which its describe keeps.
    target: String,
}

/// Resolve the bookmark plan, refusing before anything is made: the
/// opening's checks, then the bookmark to advance, `--opening`'s, the
/// one named, any on the work commit's line, at or below it, so the
/// cycle's bookmark follows a rung past a nearer one of the user's,
/// a closing's own, by `--closing`'s argument, or the line's, the
/// nearest, refused when there are none or several, since a wrong
/// guess moves the wrong branch.
fn plan_bookmarks(
    repo: &Path,
    params: &CommitParams,
    draft: Option<&Draft>,
    closing_bm: Option<&str>,
) -> Result<BookmarkPlan, Box<dyn std::error::Error>> {
    if let Some(o) = &params.opening {
        check_opening(repo, o)?;
    }
    // The work commit's line, the commit included: `@` when `@` is
    // committed, whose bookmark, `vc-x1 open`'s, follows it to `@-`, so
    // `main` below it is not the nearest, or the draft itself.
    let (below, target) = match draft {
        None => ("@".to_string(), "@-".to_string()),
        Some(d) => (d.rev.clone(), jj::chid_of(repo, &d.rev)?),
    };
    let advance = match (&params.advance, &params.opening) {
        (Advance::Off, _) => None,
        (_, Some(o)) => Some(o.bookmark.clone()),
        (Advance::Named(b), None) => {
            let on_line = jj::local_bookmarks_at(repo, &format!("::({below}) & bookmarks()"))?;
            if !on_line.contains(b) {
                return Err(format!(
                    "commit --bookmark: '{b}' is not on the work commit's line in {}, at or below \
                     '{below}', whose bookmarks are {}",
                    repo.display(),
                    if on_line.is_empty() {
                        "none".to_string()
                    } else {
                        on_line.join(", ")
                    }
                )
                .into());
            }
            Some(b.clone())
        }
        (Advance::Line, None) if params.closing.is_some() => closing_bm.map(str::to_string),
        (Advance::Line, None) => Some(crate::squash_push::current_bookmark(repo, &below).map_err(
            |e| format!("commit: {e} with --bookmark <NAME>, or pass --no-advance to move none"),
        )?),
    };
    Ok(BookmarkPlan { advance, target })
}

/// What the bookmark plan would do, for a dry run.
fn dry_run_bookmarks(plan: &BookmarkPlan) {
    if let Some(name) = &plan.advance {
        info!("commit: [dry-run] advance '{name}' to the work commit, locally");
    }
}

/// Carry out the bookmark plan once the commits are made, so a failed
/// commit moves no bookmark: the advance to the work commit, the work
/// `@-` now, or the draft described in place. An opening's bookmark is
/// there already, having followed its change.
fn finish_bookmarks(
    ctx: &mut Context,
    repo: &Path,
    plan: &BookmarkPlan,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(name) = &plan.advance {
        info!("commit: advance '{name}' to the work commit, locally");
        ctx.session(repo)?.bookmark_set(name, &plan.target)?;
    }
    Ok(())
}

/// `--opening`'s bookmark checked before anything is made: `vc-x1
/// open` made it on the work `@`, the change the opening commits, and
/// the opening creates none, so one missing is refused naming the
/// command that makes it.
fn check_opening(repo: &Path, o: &Opening) -> Result<(), Box<dyn std::error::Error>> {
    let name = &o.bookmark;
    if !jj::local_bookmarks_at(repo, "@")?.contains(name) {
        return Err(format!(
            "commit --opening: the bookmark '{name}' is not on the work @ in {}, and the opening \
             creates none: make it first, `vc-x1 open --bookmark {name}`",
            repo.display()
        )
        .into());
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
    let tip = closing_tip(closing.as_ref()).to_string();
    let closing_bm = closing_bookmark(repo, params, closing.as_ref(), &tip)?;
    let land = plan_land(repo, params, &tip, closing_bm.as_deref())?;
    let record = match &closing {
        Some(_) => RecordMove::plan(repo)?,
        None => None,
    };
    if let Some(ClosingPlan::Finalize { title, body }) = &closing {
        let body = finalized_body(
            repo,
            body.as_deref(),
            record.as_ref().map(|_| RecordMove::NAME),
        )?;
        if let Some(r) = &record {
            r.write(params.dry_run)?;
        }
        check_version(repo, "@", params.same_version, "commit")?;
        finalize(ctx, repo, None, title, Some(&body), params.dry_run)?;
        return push_after(
            ctx,
            repo,
            Publish::Por,
            closing_bm.as_deref(),
            params,
            params.dry_run,
            land.as_ref(),
        );
    }
    let draft = plan_draft(repo, params)?;
    match &draft {
        None => preflight(repo, "plain", closing.is_none() && !params.allow_empty)?,
        Some(_) => crate::pair::refuse_conflicts(repo, "commit", "plain")?,
    }
    check_version(
        repo,
        draft.as_ref().map_or("@", |d| d.rev.as_str()),
        params.same_version,
        "commit",
    )?;
    warn_unflagged_closing(params);
    let (desc, merge) = description(params, closing)?;
    let plan = plan_bookmarks(repo, params, draft.as_ref(), closing_bm.as_deref())?;
    let at = draft.as_ref().map_or("@", |d| d.rev.as_str());
    let chid = jj::chid_of(repo, at)?;
    if params.dry_run {
        if let Some((base, tip)) = &merge {
            info!("commit: [dry-run] the merge of {base} and the last rung {tip}");
        }
        dry_run_bookmarks(&plan);
        if let Some(r) = &record {
            r.write(true)?;
        }
        match &draft {
            Some(d) => info!(
                "commit: [dry-run] describe {} {chid} in place, in {}:\n{desc}",
                d.rev,
                repo.display()
            ),
            None => info!(
                "commit: [dry-run] the commit {chid}, in {}:\n{desc}",
                repo.display()
            ),
        }
        return push_after(
            ctx,
            repo,
            Publish::Por,
            plan.advance.as_deref(),
            params,
            true,
            land.as_ref(),
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
    let made = match &draft {
        None => {
            info!("commit: {} -> @- {chid}", repo.display());
            ctx.session(repo)?.commit(&desc)
        }
        Some(d) => {
            info!(
                "commit: describe {} {chid} in place, in {}",
                d.rev,
                repo.display()
            );
            ctx.session(repo)?.describe(&d.rev, &desc)
        }
    };
    if let Err(e) = made {
        snapshot.restore("commit", e.as_ref());
        return Err(e);
    }
    finish_bookmarks(ctx, repo, &plan)?;
    push_after(
        ctx,
        repo,
        Publish::Por,
        plan.advance.as_deref(),
        params,
        false,
        land.as_ref(),
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
        a.closing = Some("@-".to_string());
        let err = CommitParams::try_from(&a).unwrap_err();
        assert!(err.contains("a bookend is the work repo's"), "{err}");

        fs::write(fx.bot.join("notes.md"), "a fix-up\n").expect("write");
        let agent_at = chid(&fx.bot, "@");
        let work_at = chid(&fx.work, "@");
        let mut a = args("a", &fx.work);
        a.push = true;
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

    /// `-s a --covers` links every work commit its revset names, one
    /// `ochid:` each, oldest first, under the title given. A commit in
    /// the revset that does not name the agent `@` refuses it before
    /// anything is made, and `--covers` takes `-s a` alone.
    #[test]
    fn an_agent_commit_covers_the_work_commits_named() {
        let fx = Fixture::new("commit-covers");
        let mut a = args("w", &fx.work);
        a.covers = Some("@-".to_string());
        let err = CommitParams::try_from(&a).unwrap_err();
        assert!(err.contains("takes -s a"), "{err}");

        let agent_at = chid(&fx.bot, "@");
        for rung in ["one", "two"] {
            fs::write(fx.work.join(format!("{rung}.txt")), "work\n").expect("write");
            let mut a = args("w", &fx.work);
            a.title = Some(format!("feat: rung {rung}"));
            let params = CommitParams::try_from(&a).expect("params");
            commit(&mut test_ctx(), &params).expect("the work half is made");
        }
        let one = chid(&fx.work, "@--");
        let two = chid(&fx.work, "@-");
        fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write");

        let mut a = args("a", &fx.work);
        a.title = Some("feat: the cycle".to_string());
        a.covers = Some("@---::@-".to_string());
        let params = CommitParams::try_from(&a).expect("params");
        let err = commit(&mut test_ctx(), &params).unwrap_err().to_string();
        assert!(err.contains("do not name the agent @"), "{err}");
        assert_eq!(chid(&fx.bot, "@"), agent_at, "nothing is committed");

        a.covers = Some("@--::@-".to_string());
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the agent commit is made");
        assert_eq!(chid(&fx.bot, "@-"), agent_at);
        let desc = description(&fx.bot, "@-");
        assert!(
            desc.starts_with("feat: the cycle\n\n")
                && desc
                    .trim_end()
                    .ends_with(&format!("ochid: {one}\nochid: {two}")),
            "{desc}"
        );
    }

    /// A commit stays local by default, `--local` and `--no-push` saying
    /// so, and `--push` publishes the pair, its bookmark advanced and
    /// published and the agent `main` with it.
    #[test]
    fn a_commit_stays_local_unless_push_asks() {
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
        assert_eq!(remote("topic"), None, "the default publishes nothing");

        fs::write(fx.work.join("two.txt"), "two\n").expect("write");
        a.local = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("kept local");
        assert_eq!(remote("topic"), None, "--local says the same");
        a.local = false;

        fs::write(fx.work.join("three.txt"), "three\n").expect("write");
        fs::write(fx.bot.join("session.jsonl"), "{\"line\":2}\n").expect("write");
        a.push = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("pushed");
        assert_eq!(
            remote("topic"),
            Some(commit_id(&fx.work, "@-")),
            "--push publishes"
        );
        assert_eq!(
            chid(&fx.bot, "main@origin"),
            chid(&fx.bot, "@-"),
            "with the agent main"
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
            Trailers::of(&work_desc).ochids == [agent_at.clone()]
                && Trailers::of(&work_desc).orepo.is_some(),
            "{work_desc}"
        );
        assert!(
            agent_desc
                .trim_end()
                .ends_with(&format!("ochid: {work_at}")),
            "{agent_desc}"
        );
        assert_eq!(
            jj_ok(&fx.work, &["log", "-r", "@", "--no-graph", "-T", "empty"]),
            "true"
        );

        let check = crate::validate_desc::ValidateDescParams {
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

    /// Point a fixture's two repos at hosted URLs, nothing pushed.
    fn set_hosted_origins(fx: &Fixture) {
        for (repo, url) in [
            (&fx.work, "https://github.com/o/r.git"),
            (&fx.bot, "git@github.com:o/r.agent-session.git"),
        ] {
            crate::test_helpers::git_ok(repo, &["remote", "set-url", "origin", url]);
        }
    }

    /// With hosted origins `-s w` stamps `orepo:` and the bare `ochid:`,
    /// final before the agent commit is made, leaving the agent repo alone.
    #[test]
    fn work_stamps_a_bare_ochid_and_links() {
        let fx = Fixture::new("commit-url-w");
        set_hosted_origins(&fx);
        fs::write(fx.work.join("rung.txt"), "work\n").expect("write");
        let agent_at = chid(&fx.bot, "@");

        let params = CommitParams::try_from(&args("w", &fx.bot)).expect("params");
        commit(&mut test_ctx(), &params).expect("the work half is made");

        assert!(
            description(&fx.work, "@-").trim_end().ends_with(&format!(
                "\n\norepo: https://github.com/o/r.agent-session\nochid: {agent_at}"
            )),
            "{}",
            description(&fx.work, "@-")
        );
    }

    /// `--no-url-trailer` stamps the `ochid:` alone with hosted origins,
    /// and a plain repo takes it with no trailer to change.
    #[test]
    fn no_url_trailer() {
        let fx = Fixture::new("commit-no-url");
        set_hosted_origins(&fx);
        fs::write(fx.work.join("rung.txt"), "work\n").expect("write");
        let work_at = chid(&fx.work, "@");
        let agent_at = chid(&fx.bot, "@");

        let mut a = args("b", &fx.bot);
        a.no_url_trailer = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the pair is made");

        let work_desc = description(&fx.work, "@-");
        assert!(
            work_desc
                .trim_end()
                .ends_with(&format!("\n\nochid: {agent_at}")),
            "{work_desc}"
        );
        assert!(
            description(&fx.bot, "@-")
                .trim_end()
                .ends_with(&format!("ochid: {work_at}"))
        );

        let fx = FixturePor::new("commit-no-url-por");
        fs::write(fx.work.join("rung.txt"), "work\n").expect("write");
        let mut a = args("p", &fx.work);
        a.no_url_trailer = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the plain commit is made");
        assert!(!description(&fx.work, "@-").contains("ochid:"));
    }

    /// `-s b` makes both halves, each naming the other's repo by `orepo:`
    /// above its `ochid:`, no SHA in either.
    #[test]
    fn both_names_each_others_repo() {
        let fx = Fixture::new("commit-url-b");
        set_hosted_origins(&fx);
        fs::write(fx.work.join("rung.txt"), "work\n").expect("write");
        fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write");
        let work_at = chid(&fx.work, "@");
        let agent_at = chid(&fx.bot, "@");

        let params = CommitParams::try_from(&args("b", &fx.bot)).expect("params");
        commit(&mut test_ctx(), &params).expect("the pair is made");

        let work_desc = description(&fx.work, "@-");
        assert!(
            work_desc.trim_end().ends_with(&format!(
                "\n\norepo: https://github.com/o/r.agent-session\nochid: {agent_at}"
            )),
            "{work_desc}"
        );
        let agent_desc = description(&fx.bot, "@-");
        assert!(
            agent_desc.trim_end().ends_with(&format!(
                "\n\norepo: https://github.com/o/r\nochid: {work_at}"
            )),
            "{agent_desc}"
        );
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
            Trailers::of(&work_desc).ochids == [agent_at.clone()]
                && Trailers::of(&work_desc).orepo.is_some(),
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
        a.closing = Some("@-".to_string());
        a.onto = Some("main".to_string());
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
        // The merge names its partner, and no trailer names a parent,
        // git holding the parents.
        let desc = description(&fx.work, "@-");
        assert_eq!(
            Trailers::of(&desc).ochids,
            std::slice::from_ref(&agent_at),
            "{desc}"
        );
        assert!(!desc.contains("pcid:") && !desc.contains("ocid:"), "{desc}");
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
            ignore_immutable: false,
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
        a.closing = Some("@-".to_string());
        a.onto = Some("main".to_string());
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

    /// `--closing` with no ladder, the work `@-` on `main`, and no
    /// bookmark on `@` to close it from, is refused before anything is
    /// committed, naming `vc-x1 open`.
    #[test]
    fn closing_with_no_ladder_is_refused() {
        let fx = Fixture::new("commit-closing-none");
        fs::write(fx.work.join("closing.txt"), "closing\n").expect("write");
        let work_at = chid(&fx.work, "@");
        let mut a = args("b", &fx.work);
        a.title = Some("feat: a cycle".to_string());
        a.closing = Some("@-".to_string());
        a.onto = Some("main".to_string());
        let params = CommitParams::try_from(&a).expect("params");
        let err = commit(&mut test_ctx(), &params)
            .expect_err("no ladder")
            .to_string();
        assert!(err.contains("vc-x1 open"), "{err}");
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

    /// The bookmark `vc-x1 open` makes on the work `@`, as a test makes
    /// it, so the opening takes it.
    fn open_bookmark(fx: &Fixture, name: &str) {
        jj_ok(&fx.work, &["bookmark", "create", name, "-r", "@"]);
    }

    /// `--opening` makes the opening under the bare title with
    /// " opening" appended, on the bookmark `vc-x1 open` made on the work
    /// `@`, which followed the change into it, and `--push` publishes
    /// both, a rung's push, the bookmark tracked.
    #[test]
    fn opening_takes_the_bookmark_open_made() {
        let fx = Fixture::new("commit-opening");
        let o = crate::open::OpenParams {
            work: fx.work.clone(),
            agent: Some(fx.bot.clone()),
            bookmark: "feat-a-cycle".to_string(),
            dry_run: false,
        };
        crate::open::open(&mut test_ctx(), &o).expect("opened");
        fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
        fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write");
        let mut a = args("b", &fx.work);
        a.title = Some("feat: a cycle".to_string());
        a.opening = Some(None);
        a.push = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the opening is made");

        let desc = description(&fx.work, "@-");
        assert!(desc.starts_with("feat: a cycle opening\n\n"), "{desc}");
        let opening = commit_id(&fx.work, "@-");
        assert_eq!(
            bookmark_state(&fx.work, "feat-a-cycle"),
            (Some(opening.clone()), Some(opening)),
            "the opening, locally and on origin"
        );
        assert!(jj::has_tracked_remote(&fx.work, "feat-a-cycle", "origin").expect("tracked"));
        assert!(
            jj::matches(&fx.work, "@- & ::remote_bookmarks()").expect("matches"),
            "the opening is published"
        );
    }

    /// `--opening` with no value takes the bookmark named by the bare
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
        open_bookmark(&fx, "feat-vc-x1-single-step-commit");
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
            bookmark_state(&fx.work, "feat-vc-x1-single-step-commit").0,
            Some(commit_id(&fx.work, "@-"))
        );

        a.title = Some("?!".to_string());
        let err = CommitParams::try_from(&a).unwrap_err();
        assert!(err.contains("no letters or digits"), "{err}");
    }

    /// An opening's bookmark stays off the remote by default, and by
    /// `--local` and its alias, and `--push` publishes it.
    #[test]
    fn an_opening_kept_local_pushes_no_bookmark() {
        let open = |tag: &str, flag: Option<&str>| {
            let fx = Fixture::new(&format!("commit-opening-local-{tag}"));
            open_bookmark(&fx, "feat-x");
            fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
            let mut argv = vec![
                "vc-x1",
                "commit",
                "-s",
                "w",
                "--opening",
                "feat-x",
                "--title",
                "feat: x",
                "--body",
                "why",
            ];
            argv.extend(flag);
            let mut a = parse(&argv).expect("parses");
            a.repo = Some(fx.work.clone());
            let params = CommitParams::try_from(&a).expect("params");
            commit(&mut test_ctx(), &params).expect("the opening is made");
            bookmark_state(&fx.work, "feat-x")
        };
        for (tag, flag) in [
            ("default", None),
            ("flag", Some("--local")),
            ("alias", Some("--no-push")),
        ] {
            let (local, remote) = open(tag, flag);
            assert!(local.is_some(), "{tag}: the bookmark is there");
            assert_eq!(remote, None, "{tag}: nothing is pushed");
        }
        let (_, remote) = open("push", Some("--push"));
        assert!(remote.is_some(), "--push publishes it");
    }

    /// Every refusal comes before anything is committed: a title that
    /// carries " opening" already, `--no-advance`, which has nothing to
    /// move, and a bookmark `vc-x1 open` has not made on the work `@`,
    /// named in the refusal. `--skip` needs `--land`, and `--opening` is
    /// not `--closing`.
    #[test]
    fn opening_refusals_commit_nothing() {
        let err = parse(&[
            "vc-x1", "commit", "-s", "b", "--title", "t", "--body", "b", "--skip", "both",
        ])
        .unwrap_err();
        assert!(err.contains("--land"), "{err}");
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
        a.no_advance = true;
        let err = CommitParams::try_from(&a).unwrap_err();
        assert!(err.contains("--no-advance moves nothing"), "{err}");
        a.no_advance = false;

        fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
        let work_at = chid(&fx.work, "@");
        let params = CommitParams::try_from(&a).expect("params");
        let err = commit(&mut test_ctx(), &params)
            .expect_err("no bookmark")
            .to_string();
        assert!(
            err.contains("is not on the work @") && err.contains("vc-x1 open --bookmark feat-x"),
            "{err}"
        );
        assert_eq!(chid(&fx.work, "@"), work_at, "nothing was committed");

        jj_ok(&fx.work, &["bookmark", "create", "feat-x", "-r", "@-"]);
        let err = commit(&mut test_ctx(), &params)
            .expect_err("below @")
            .to_string();
        assert!(err.contains("is not on the work @"), "{err}");
        assert_eq!(chid(&fx.work, "@"), work_at, "nothing was committed");
    }

    /// The line's bookmark advances to the work commit locally, and
    /// stays advanced through a squash into it, while origin's is
    /// untouched. `--no-advance` leaves it, `--bookmark` names any on
    /// the line, nearest or not, one off the line is refused, and a
    /// line with several nearest is refused before anything is
    /// committed.
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
            &["new", "--no-edit", "root()", "-m", "a side line"],
        );
        jj_ok(
            &fx.work,
            &[
                "bookmark",
                "create",
                "elsewhere",
                "-r",
                "description(substring:\"a side line\")",
            ],
        );
        let params = CommitParams::try_from(&a).expect("params");
        let err = commit(&mut test_ctx(), &params)
            .expect_err("not on the line")
            .to_string();
        assert!(err.contains("not on the work commit's line"), "{err}");
        assert!(
            err.contains("main, topic") || err.contains("topic, main"),
            "{err}"
        );
        assert_eq!(chid(&fx.work, "@"), work_at, "nothing was committed");

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

        // `main` sits below `topic` now, not the nearest, and is still
        // the line's to name.
        fs::write(fx.work.join("main.txt"), "main\n").expect("write");
        a.bookmark = Some("main".to_string());
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the commit is made");
        assert_eq!(
            commit_id(&fx.work, "main"),
            commit_id(&fx.work, "@-"),
            "main, below the nearest, advanced"
        );
        assert_eq!(commit_id(&fx.work, "topic"), one, "topic is left");
        jj_ok(
            &fx.work,
            &["bookmark", "set", "main", "-r", &start, "--allow-backwards"],
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

    /// A plain commit on a branch `vc-x1 open` made, its bookmark on
    /// `@` and `main` on `@-`, keeps the cycle's bookmark on the commit
    /// and leaves `main`, the bookmark on `@` being the line's nearest.
    #[test]
    fn a_plain_commit_on_an_opened_branch_leaves_main() {
        let fx = Fixture::new("commit-opened-plain");
        let start = commit_id(&fx.work, "main");
        open_bookmark(&fx, "feat-x");
        fs::write(fx.work.join("one.txt"), "one\n").expect("write");
        let at = chid(&fx.work, "@");
        let params = CommitParams::try_from(&args("w", &fx.work)).expect("params");
        commit(&mut test_ctx(), &params).expect("the commit is made");
        assert_eq!(chid(&fx.work, "@-"), at, "@ was committed");
        assert_eq!(chid(&fx.work, "feat-x"), at, "feat-x is on the commit");
        assert_eq!(commit_id(&fx.work, "main"), start, "main is left");
    }

    /// A dry run of `--opening` commits nothing and leaves the bookmark
    /// where `vc-x1 open` made it.
    #[test]
    fn an_opening_dry_run_commits_nothing() {
        let fx = Fixture::new("commit-opening-dry-run");
        open_bookmark(&fx, "feat-x");
        fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
        let at = chid(&fx.work, "@");
        let mut a = args("w", &fx.work);
        a.opening = Some(Some("feat-x".to_string()));
        a.dry_run.value = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("dry run");
        assert_eq!(chid(&fx.work, "@"), at, "nothing was committed");
        assert_eq!(chid(&fx.work, "feat-x"), at, "the bookmark is on @ still");
    }

    /// Open a cycle titled "feat: a cycle": its bookmark made on the work
    /// `@`, as `vc-x1 open` makes it, then `--opening`, the pair made at
    /// once, returning the opening's change id.
    fn open_cycle(fx: &Fixture) -> String {
        open_bookmark(fx, "feat-a-cycle");
        fs::write(fx.work.join("opening.txt"), "opening\n").expect("write");
        let mut a = args("b", &fx.work);
        a.title = Some("feat: a cycle".to_string());
        a.body = Some("Why the cycle.".to_string());
        a.opening = Some(None);
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the opening is made");
        chid(&fx.work, "@-")
    }

    /// `commit --closing <BM|REV>` with the title and body left out.
    fn closing_args(fx: &Fixture, scope: &str, rev: &str) -> CommitArgs {
        let mut a = args(scope, &fx.work);
        a.title = None;
        a.body = None;
        a.closing = Some(rev.to_string());
        a.onto = Some("main".to_string());
        a
    }

    /// Set the work config's `[closing] body`.
    fn set_closing_body(fx: &Fixture) {
        let path = fx.work.join(".vc-config.md");
        let mut cfg = fs::read_to_string(&path).expect("read the config");
        cfg.push_str("\n```toml\n[closing]\nbody = 'Closes the cycle \"{title}\".'\n```\n");
        fs::write(&path, cfg).expect("write the config");
    }

    /// Finalizing keeps every trailer, once each and out of the body.
    #[test]
    fn finalizing_keeps_the_trailers() {
        let fx = Fixture::new("commit-finalize-orepo");
        open_cycle(&fx);
        let before = Trailers::of(&description(&fx.work, "@-"));
        assert!(
            before.orepo.is_some() && !before.ochids.is_empty(),
            "{before:?}"
        );

        let params = CommitParams::try_from(&closing_args(&fx, "w", "@-")).expect("params");
        commit(&mut test_ctx(), &params).expect("finalized");

        let desc = description(&fx.work, "@-");
        assert_eq!(Trailers::of(&desc), before, "{desc}");
        assert_eq!(desc.matches("orepo:").count(), 1, "{desc}");
    }

    /// A branch of one commit above the trunk, the opening, is a
    /// single-step cycle: `--closing @-` folds `@` into it and gives it
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

        let params = CommitParams::try_from(&closing_args(&fx, "b", "@-")).expect("params");
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

            let params = CommitParams::try_from(&closing_args(&fx, "w", "@-")).expect("params");
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

    /// `--closing` names the tip as a bookmark or a revision, the two
    /// forms naming the same commit, and the shape is read from the
    /// branch: with a rung above the opening and `@` on the rung, the
    /// opening's bookmark names a one-commit branch, finalized in place
    /// by a detour, `@` moving onto it and returning to a fresh empty
    /// commit on the rung, the rung itself left above, unclosed.
    #[test]
    fn a_closing_names_its_tip_as_a_bookmark_or_a_revision() {
        let fx = Fixture::new("commit-closing-tip");
        let opening = open_cycle(&fx);
        jj_ok(&fx.work, &["commit", "-m", "feat: a rung"]);
        let rung = chid(&fx.work, "@-");
        let at = chid(&fx.work, "@");

        let params =
            CommitParams::try_from(&closing_args(&fx, "w", "feat-a-cycle")).expect("params");
        commit(&mut test_ctx(), &params).expect("finalized by a detour");

        assert_eq!(chid(&fx.work, "@-"), rung, "@ returned onto the rung");
        assert_ne!(
            chid(&fx.work, "@"),
            at,
            "to a fresh commit, the empty one abandoned"
        );
        assert!(jj::is_empty(&fx.work, "@").expect("empty"));
        assert_eq!(
            chid(&fx.work, "@--"),
            opening,
            "the rung still sits on the opening"
        );
        let desc = description(&fx.work, "@--");
        assert!(
            desc.starts_with("feat: a cycle\n\nWhy the cycle.\n"),
            "{desc}"
        );
        assert_eq!(
            description(&fx.work, "@-").trim_end(),
            "feat: a rung",
            "the rung is as it was"
        );

        // The revision form names the same tip, refused now that the
        // opening carries no " opening" to read the title from.
        let params = CommitParams::try_from(&closing_args(&fx, "w", &opening)).expect("params");
        let err = commit(&mut test_ctx(), &params)
            .expect_err("not an opening")
            .to_string();
        assert!(err.contains("is not an opening"), "{err}");
    }

    /// A `@` holding work is kept through a closing's detour: the
    /// closing of a tip below `@-` is made from an empty working copy,
    /// which is no refusal for a closing, carries nothing of `@`, and
    /// `@` returns to the same commit with its work, the rung above
    /// the tip left unclosed.
    #[test]
    fn a_detour_keeps_the_work_at_holds() {
        let fx = Fixture::new("commit-closing-detour");
        open_cycle(&fx);
        // In rung one's tree, since the closing reads the config from
        // the tip's working copy.
        set_closing_body(&fx);
        fs::write(fx.work.join("one.txt"), "one\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: rung one"]);
        let one = chid(&fx.work, "@-");
        fs::write(fx.work.join("two.txt"), "two\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: rung two"]);
        let two = chid(&fx.work, "@-");
        fs::write(fx.work.join("next.txt"), "next work\n").expect("write");
        let at = chid(&fx.work, "@");

        let params = CommitParams::try_from(&closing_args(&fx, "w", &one)).expect("params");
        commit(&mut test_ctx(), &params).expect("closed by a detour");

        assert_eq!(chid(&fx.work, "@"), at, "@ is the same commit");
        assert_eq!(chid(&fx.work, "@-"), two, "on rung two, unclosed");
        let files = jj_ok(&fx.work, &["file", "list", "-r", "@"]);
        assert!(files.contains("next.txt"), "with its work: {files}");
        let closing = jj_ok(
            &fx.work,
            &[
                "log",
                "-r",
                "description(substring:\"feat: a cycle closing\")",
                "--no-graph",
                "-T",
                "commit_id",
            ],
        );
        let closing = closing.trim();
        assert_eq!(
            parents(&fx.work, closing),
            [commit_id(&fx.work, "main"), commit_id(&fx.work, &one)]
        );
        let carried = jj_ok(&fx.work, &["file", "list", "-r", closing]);
        assert!(
            !carried.contains("next.txt") && !carried.contains("two.txt"),
            "the closing carries nothing above its tip: {carried}"
        );
    }

    /// The target is `--onto`'s bookmark, the closing's first parent,
    /// with none assumed: left out it is refused naming the bookmark
    /// below the opening, and one that does not resolve is refused,
    /// both before anything is made.
    #[test]
    fn the_target_is_ontos_and_none_is_assumed() {
        let fx = Fixture::new("commit-closing-onto");
        jj_ok(&fx.work, &["bookmark", "create", "dev", "-r", "main"]);
        open_cycle(&fx);
        fs::write(fx.work.join("rung.txt"), "rung\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: a rung"]);
        let tip = commit_id(&fx.work, "@-");
        jj_ok(&fx.work, &["new", "main"]);
        fs::write(fx.work.join("landed.txt"), "landed\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: main moved"]);
        jj_ok(&fx.work, &["bookmark", "set", "main", "-r", "@-"]);
        jj_ok(&fx.work, &["new", &tip]);
        fs::write(fx.work.join("closing.txt"), "closing\n").expect("write");
        let work_at = chid(&fx.work, "@");

        let mut a = closing_args(&fx, "w", "@-");
        a.body = Some("Closing.".to_string());
        for (onto, why) in [
            (None, "--onto <BOOKMARK>, likely `--onto dev`"),
            (
                Some("no-such-target"),
                "--onto 'no-such-target' does not resolve",
            ),
        ] {
            a.onto = onto.map(str::to_string);
            let params = CommitParams::try_from(&a).expect("params");
            let err = commit(&mut test_ctx(), &params)
                .expect_err("refused")
                .to_string();
            assert!(err.contains(why), "{onto:?}: {err}");
            assert_eq!(
                chid(&fx.work, "@"),
                work_at,
                "{onto:?}: nothing was committed"
            );
        }

        a.onto = Some("dev".to_string());
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("closed onto dev");
        assert_eq!(parents(&fx.work, "@-"), [commit_id(&fx.work, "dev"), tip]);
    }

    /// `--dead-end` always makes the closing commit, a branch of the
    /// opening alone included, titled " dead-end closing".
    #[test]
    fn a_dead_end_always_gets_the_closing_commit() {
        let fx = Fixture::new("commit-closing-dead-end");
        open_cycle(&fx);
        let opening = commit_id(&fx.work, "@-");
        fs::write(fx.work.join("closing.txt"), "closing\n").expect("write");
        let mut a = closing_args(&fx, "w", "@-");
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

    /// `--closing` takes any one revision as the tip: a change id names
    /// it as a bookmark would, and a revision naming no commit, or
    /// several, or the trunk itself, or a branch whose first commit is
    /// not an opening with no `--title`, is refused before anything is
    /// committed.
    #[test]
    fn closing_takes_any_one_revision() {
        let fx = Fixture::new("commit-closing-rev");
        let base = commit_id(&fx.work, "main");
        for rung in ["one", "two"] {
            fs::write(fx.work.join(format!("{rung}.txt")), rung).expect("write");
            jj_ok(&fx.work, &["commit", "-m", &format!("feat: rung {rung}")]);
        }
        jj_ok(&fx.work, &["bookmark", "create", "topic", "-r", "@-"]);
        let tip = commit_id(&fx.work, "@-");
        let tip_chid = chid(&fx.work, "@-");
        fs::write(fx.work.join("closing.txt"), "closing\n").expect("write");
        let work_at = chid(&fx.work, "@");

        let mut a = closing_args(&fx, "w", "@-");
        a.body = Some("b".to_string());
        for (rev, why) in [
            ("nosuch", "does not resolve"),
            ("main::@-", "names 3 commits"),
            ("main", "nothing above 'main' to close"),
            ("@-", "is not an opening"),
        ] {
            a.closing = Some(rev.to_string());
            let params = CommitParams::try_from(&a).expect("params");
            let err = commit(&mut test_ctx(), &params).expect_err(rev).to_string();
            assert!(err.contains(why), "{rev}: {err}");
            assert_eq!(chid(&fx.work, "@"), work_at, "{rev}: nothing was committed");
        }

        a.closing = Some(tip_chid);
        a.title = Some("feat: a cycle".to_string());
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("the closing is made");
        assert_eq!(parents(&fx.work, "@-"), [base, tip]);
        assert!(description(&fx.work, "@-").starts_with("feat: a cycle closing\n"));
    }

    /// A `TODO.md` in the Todo format, a cycle in progress whose block
    /// holds a fenced `## ` line, and an older cycle closed.
    const RECORD_TEXT: &str = "# Todo\n\n## Continuation notes\n\n_None._\n\n\
                               ## In Progress\n\nIntro.\n\n### feat: a cycle\n\nThe record.\n\n\
                               ```\n## not a heading\n```\n\n- [r][1]\n\n\
                               ## Todo\n\n### A todo\n\n\
                               ## Closed\n\nClosed intro.\n\n### feat: an older cycle\n\nOld.\n\n\
                               # References\n\n[1]: #r\n";

    /// The move takes the In Progress block, fenced lines and all, into
    /// `## Closed` in place of the older cycle, and resets `## In
    /// Progress`, each intro and the references kept. Nothing in
    /// progress, or no `## In Progress`, moves nothing, and a cycle
    /// with no `## Closed` is refused.
    #[test]
    fn the_record_moves_as_text() {
        let moved = moved_record(RECORD_TEXT).expect("moved").expect("a cycle");
        assert_eq!(
            moved,
            "# Todo\n\n## Continuation notes\n\n_None._\n\n\
             ## In Progress\n\nIntro.\n\n_No cycle currently in progress._\n\n\
             ## Todo\n\n### A todo\n\n\
             ## Closed\n\nClosed intro.\n\n### feat: a cycle\n\nThe record.\n\n\
             ```\n## not a heading\n```\n\n- [r][1]\n\n\
             # References\n\n[1]: #r\n"
        );
        assert_eq!(moved_record(&moved), Ok(None), "nothing in progress");
        assert_eq!(
            moved_record("# Todo\n\n## Todo\n"),
            Ok(None),
            "no In Progress"
        );
        let err = moved_record("## In Progress\n\n### feat: a cycle\n").unwrap_err();
        assert!(err.contains("no `## Closed`"), "{err}");
    }

    /// The fixture's `zTOC.md`, its ladder listing the cycle's rung.
    const TOC_TEXT: &str = "# TOC\n\n## Ladder\n\n- 0 (done) [r](TODO.md#r)\n\n## The TOC\n";

    /// Give the fixture's work repo a `TODO.md` with a cycle in
    /// progress, and a `zTOC.md` listing its rung.
    fn set_record(fx: &Fixture) {
        fs::write(fx.work.join(RECORD), RECORD_TEXT).expect("write");
        fs::write(fx.work.join("zTOC.md"), TOC_TEXT).expect("write");
    }

    /// The commit's copy of a file.
    fn file_at(repo: &Path, rev: &str, file: &str) -> String {
        jj_ok(repo, &["file", "show", "-r", rev, &format!("root:{file}")])
    }

    /// A closing commit carries the record moved in `TODO.md` and
    /// `zTOC.md`'s ladder as it was, and a finalized commit too, its kept
    /// body gaining the pointer.
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
            let mut a = closing_args(&fx, "w", "@-");
            a.body = multi.then(|| "Closing.".to_string());
            let params = CommitParams::try_from(&a).expect("params");
            commit(&mut test_ctx(), &params).expect("closed");

            let record = file_at(&fx.work, "@-", RECORD);
            assert!(
                record.contains("## In Progress\n\nIntro.\n\n_No cycle currently in progress._"),
                "multi {multi}: {record}"
            );
            assert!(
                record.contains("Closed intro.\n\n### feat: a cycle\n"),
                "{record}"
            );
            assert!(!record.contains("an older cycle"), "{record}");
            assert_eq!(
                file_at(&fx.work, "@-", "zTOC.md"),
                TOC_TEXT.trim_end(),
                "the ladder stays"
            );
            let desc = description(&fx.work, "@-");
            if multi {
                assert!(
                    desc.starts_with("feat: a cycle closing\n\nClosing.\n"),
                    "{desc}"
                );
            } else {
                assert!(
                    desc.starts_with(
                        "feat: a cycle\n\nWhy the cycle.\n\nSee details in TODO.md.\n"
                    ),
                    "{desc}"
                );
            }
        }
    }

    /// A branch with nothing committed, `vc-x1 open`'s bookmark on a `@`
    /// holding the work, closes in one commit made from `@` and lands,
    /// the tip named `@-`, the bookmark, or `@`: `main` is `@` committed
    /// under the bare title and published, the agent `main` with it.
    #[test]
    fn a_branch_with_nothing_committed_closes_and_lands_in_one_commit() {
        for rev in ["@-", "feat-abc", "@"] {
            let fx = Fixture::new(&format!("commit-land-uncommitted-{}", rev.len()));
            open_bookmark(&fx, "feat-abc");
            fs::write(fx.work.join("abc.txt"), "abc\n").expect("write");
            let work_at = chid(&fx.work, "@");
            let mut a = closing_args(&fx, "b", rev);
            a.title = Some("feat: add abc.txt".to_string());
            a.body = Some("Created abc.".to_string());
            a.land = true;
            a.push = true;
            let params = CommitParams::try_from(&a).expect("params");
            commit(&mut test_ctx(), &params).expect("landed");

            assert_eq!(
                chid(&fx.work, "main"),
                work_at,
                "{rev}: main is @, committed"
            );
            assert_eq!(
                chid(&fx.work, "main@origin"),
                work_at,
                "{rev}: and published"
            );
            let desc = description(&fx.work, "main");
            assert!(
                desc.starts_with("feat: add abc.txt\n\nCreated abc."),
                "{rev}: {desc}"
            );
            assert!(
                jj_ok(&fx.work, &["file", "list", "-r", "main"]).contains("abc.txt"),
                "{rev}"
            );
            assert_eq!(
                chid(&fx.bot, "main@origin"),
                chid(&fx.bot, "@-"),
                "{rev}: with the agent main"
            );
            assert!(
                !jj::local_bookmark_exists(&fx.work, "feat-abc").expect("local"),
                "{rev}: the cycle's bookmark is gone"
            );
        }
    }

    /// A branch with nothing committed is refused before anything is
    /// made when `@` is empty, carries no bookmark, the title or body is
    /// left out, or `--dead-end` asks for a closing commit.
    #[test]
    fn a_branch_with_nothing_committed_is_refused_before_anything_is_made() {
        let fx = Fixture::new("commit-closing-uncommitted-refused");
        let main = commit_id(&fx.work, "main");
        let mut a = closing_args(&fx, "b", "@-");
        a.title = Some("feat: add abc.txt".to_string());
        a.body = Some("Created abc.".to_string());
        let refused = |a: &CommitArgs, why: &str| {
            let at = chid(&fx.work, "@");
            let params = CommitParams::try_from(a).expect("params");
            let err = commit(&mut test_ctx(), &params).expect_err(why).to_string();
            assert!(err.contains(why), "{why}: {err}");
            assert_eq!(chid(&fx.work, "@"), at, "{why}: nothing was committed");
            assert_eq!(commit_id(&fx.work, "main"), main, "{why}: main stays");
        };
        open_bookmark(&fx, "feat-abc");
        refused(&a, "nothing above 'main' to close");
        fs::write(fx.work.join("abc.txt"), "abc\n").expect("write");
        jj_ok(&fx.work, &["bookmark", "delete", "feat-abc"]);
        refused(&a, "vc-x1 open");
        open_bookmark(&fx, "feat-abc");
        a.dead_end = true;
        refused(&a, "--dead-end");
        a.dead_end = false;
        a.body = None;
        refused(&a, "--title and --body are required");
    }

    /// `--land` on a single-step cycle finalizes the opening, fast-
    /// forwards `main` to it and pushes it with the agent `main`, and
    /// deletes the cycle's bookmark locally and on origin.
    #[test]
    fn a_single_step_cycle_lands() {
        let fx = Fixture::new("commit-land-single");
        let opening = open_cycle(&fx);
        let mut a = closing_args(&fx, "b", "@-");
        a.land = true;
        a.push = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("landed");

        assert_eq!(
            chid(&fx.work, "main"),
            opening,
            "main is the finalized commit"
        );
        assert_eq!(chid(&fx.work, "main@origin"), opening, "and published");
        assert_eq!(
            chid(&fx.bot, "main@origin"),
            chid(&fx.bot, "@-"),
            "with the agent main"
        );
        assert!(!jj::local_bookmark_exists(&fx.work, "feat-a-cycle").expect("local"));
        assert!(
            !jj::remote_bookmark_targets(&fx.work, "origin")
                .expect("remote")
                .contains_key("feat-a-cycle"),
            "the cycle's bookmark is gone on origin"
        );
    }

    /// A bare `--land` on a ladder lands the closing merge on the
    /// default trunk, `main`, and `--skip both` keeps the cycle's
    /// bookmark.
    #[test]
    fn a_ladder_lands_through_its_closing() {
        let fx = Fixture::new("commit-land-ladder");
        open_cycle(&fx);
        fs::write(fx.work.join("rung.txt"), "rung\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: a rung"]);
        fs::write(fx.work.join("closing.txt"), "closing\n").expect("write");
        let mut a = closing_args(&fx, "b", "@-");
        a.body = Some("Closing.".to_string());
        a.land = true;
        a.push = true;
        a.skip = Some(Skip::Both);
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("landed");

        assert_eq!(
            parents(&fx.work, "main").len(),
            2,
            "main is the closing merge"
        );
        assert_eq!(
            commit_id(&fx.work, "main@origin"),
            commit_id(&fx.work, "main")
        );
        assert!(jj::local_bookmark_exists(&fx.work, "feat-a-cycle").expect("kept"));
    }

    /// Whether `name` is a bookmark on origin.
    fn on_origin(repo: &Path, name: &str) -> bool {
        jj::remote_bookmark_targets(repo, "origin")
            .expect("remote")
            .contains_key(name)
    }

    /// With the cycle's bookmark and a user's own both at the tip, a
    /// revision names no one bookmark, so it is refused listing both
    /// before anything is made; the bookmark form names the one to
    /// delete, locally and on origin, and the other is left, merged.
    #[test]
    fn a_land_deletes_the_bookmark_named() {
        let fx = Fixture::new("commit-land-named");
        open_cycle(&fx);
        fs::write(fx.work.join("rung.txt"), "rung\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: a rung"]);
        jj_ok(&fx.work, &["bookmark", "set", "feat-a-cycle", "-r", "@-"]);
        jj_ok(&fx.work, &["bookmark", "create", "mine", "-r", "@-"]);
        let rung = commit_id(&fx.work, "@-");
        let at = chid(&fx.work, "@");
        let main = commit_id(&fx.work, "main");

        let mut a = closing_args(&fx, "b", "@-");
        a.body = Some("Closing.".to_string());
        a.land = true;
        a.push = true;
        let params = CommitParams::try_from(&a).expect("params");
        let err = commit(&mut test_ctx(), &params)
            .expect_err("several")
            .to_string();
        assert!(
            err.contains("several bookmarks")
                && err.contains("feat-a-cycle")
                && err.contains("mine"),
            "{err}"
        );
        assert_eq!(chid(&fx.work, "@"), at, "nothing was committed");
        assert_eq!(commit_id(&fx.work, "main"), main, "nothing was landed");

        a.closing = Some("feat-a-cycle".to_string());
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("landed");
        assert_eq!(parents(&fx.work, "main"), [main, rung.clone()]);
        assert_eq!(
            commit_id(&fx.work, "main@origin"),
            commit_id(&fx.work, "main")
        );
        assert!(!jj::local_bookmark_exists(&fx.work, "feat-a-cycle").expect("local"));
        assert!(!on_origin(&fx.work, "feat-a-cycle"), "deleted on origin");
        assert_eq!(commit_id(&fx.work, "mine"), rung, "the user's is left");
    }

    /// A revision with no bookmark on it takes the nearest parent's:
    /// the cycle's bookmark left on the opening is the one deleted.
    #[test]
    fn a_land_takes_the_nearest_parents_bookmark() {
        let fx = Fixture::new("commit-land-nearest");
        open_cycle(&fx);
        fs::write(fx.work.join("rung.txt"), "rung\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: a rung"]);
        let mut a = closing_args(&fx, "b", "@-");
        a.body = Some("Closing.".to_string());
        a.land = true;
        a.push = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("landed");
        assert_eq!(parents(&fx.work, "main").len(), 2, "main is the closing");
        assert!(!jj::local_bookmark_exists(&fx.work, "feat-a-cycle").expect("local"));
        assert!(!on_origin(&fx.work, "feat-a-cycle"), "deleted on origin");
    }

    /// Two rungs on `main` with no bookmark above it, closed with the
    /// title given.
    fn unbookmarked_branch(tag: &str) -> (Fixture, CommitArgs) {
        let fx = Fixture::new(tag);
        for rung in ["one", "two"] {
            fs::write(fx.work.join(format!("{rung}.txt")), rung).expect("write");
            jj_ok(&fx.work, &["commit", "-m", &format!("feat: rung {rung}")]);
        }
        let mut a = closing_args(&fx, "b", "@-");
        a.title = Some("feat: a cycle".to_string());
        a.body = Some("Closing.".to_string());
        (fx, a)
    }

    /// A branch with no bookmark above the trunk lands with nothing
    /// deleted, and a closing with none and no `--land` is kept local,
    /// since its push would publish it on the trunk.
    #[test]
    fn a_branch_with_no_bookmark_lands_and_deletes_none() {
        let (fx, mut a) = unbookmarked_branch("commit-close-none");
        let main = commit_id(&fx.work, "main");
        a.push = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("closed, kept local");
        assert_eq!(parents(&fx.work, "@-").len(), 2, "the closing is made");
        assert_eq!(commit_id(&fx.work, "main"), main, "main is not moved");
        assert_eq!(commit_id(&fx.work, "main@origin"), main, "nothing pushed");

        let (fx, mut a) = unbookmarked_branch("commit-land-none");
        a.land = true;
        a.push = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("landed");
        assert_eq!(parents(&fx.work, "main").len(), 2, "main is the closing");
        assert_eq!(
            commit_id(&fx.work, "main@origin"),
            commit_id(&fx.work, "main")
        );
    }

    /// The live fixture `tmp/t-85-8.1-2ecac` replayed with local
    /// remotes, the acceptance check of **feat: vc-x1 commit takes a
    /// revision**: a cycle's opening on its bookmark, a draft made by
    /// hand above it under a second bookmark, never pushed, and an
    /// empty `@`, landed by three lines with no `jj` typed:
    ///
    /// 1. `commit @- -s b --title --body --bookmark <cycle's>`, the
    ///    draft described and paired, the cycle's bookmark moved past
    ///    the user's onto it, and pushed.
    /// 2. `commit -s b --closing @- --onto main --land`, refused naming both
    ///    bookmarks, then `--closing <cycle's> --onto main --land`, the closing
    ///    merged, landed, and the cycle's bookmark deleted.
    /// 3. `delete-bookmark -s w --bookmark <user's> --skip remote`.
    #[test]
    fn the_fixture_lands_by_three_lines() {
        let fx = Fixture::new("commit-acceptance");
        let base = commit_id(&fx.work, "main");
        open_cycle(&fx);
        fs::write(fx.work.join("opening.txt"), "opening\nline 2\n").expect("write");
        jj_ok(&fx.work, &["new"]);
        jj_ok(&fx.work, &["bookmark", "create", "line-2", "-r", "@-"]);
        let draft = chid(&fx.work, "@-");
        assert!(jj::is_empty(&fx.work, "@").expect("empty"), "@ is empty");

        // Line 1.
        let mut a = args("b", &fx.work);
        a.rev = Some("@-".to_string());
        a.title = Some("feat: add line-2".to_string());
        a.body = Some("Added line 2 to opening.txt".to_string());
        a.bookmark = Some("feat-a-cycle".to_string());
        a.push = true;
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("line 1");
        assert_eq!(chid(&fx.work, "@-"), draft, "described in place");
        let desc = description(&fx.work, "@-");
        assert!(desc.starts_with("feat: add line-2\n"), "{desc}");
        let agent = chid(&fx.bot, "@-");
        assert!(desc.contains(&format!("ochid: {agent}")), "{desc}");
        assert!(description(&fx.bot, "@-").contains(&format!("ochid: {draft}")));
        assert_eq!(
            chid(&fx.work, "feat-a-cycle"),
            draft,
            "moved past the user's"
        );
        assert_eq!(chid(&fx.work, "feat-a-cycle@origin"), draft, "and pushed");
        assert!(!on_origin(&fx.work, "line-2"), "the user's stays local");

        // Line 2, first as a revision, which names two bookmarks.
        let mut a = closing_args(&fx, "b", "@-");
        a.body = Some("Closing.".to_string());
        a.land = true;
        a.push = true;
        let params = CommitParams::try_from(&a).expect("params");
        let err = commit(&mut test_ctx(), &params)
            .expect_err("two bookmarks")
            .to_string();
        assert!(
            err.contains("feat-a-cycle") && err.contains("line-2"),
            "{err}"
        );
        a.closing = Some("feat-a-cycle".to_string());
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("line 2");
        assert_eq!(
            parents(&fx.work, "main"),
            [base, commit_id(&fx.work, &draft)]
        );
        assert_eq!(
            commit_id(&fx.work, "main@origin"),
            commit_id(&fx.work, "main")
        );
        assert_eq!(chid(&fx.bot, "main@origin"), chid(&fx.bot, "main"));
        assert!(!jj::local_bookmark_exists(&fx.work, "feat-a-cycle").expect("local"));
        assert!(!on_origin(&fx.work, "feat-a-cycle"), "deleted on origin");

        // Line 3.
        crate::delete_bookmark::delete_bookmark(
            &mut test_ctx(),
            &fx.work,
            "line-2",
            Some(crate::delete_bookmark::Keep::Remote),
            false,
        )
        .expect("line 3");
        assert!(!jj::local_bookmark_exists(&fx.work, "line-2").expect("local"));
        assert_eq!(
            jj::local_bookmarks_at(&fx.work, "all()").expect("bookmarks"),
            ["main"],
            "main alone is left"
        );
    }

    /// Every refusal of `--land` comes before anything is made: a trunk
    /// whose local is not at origin's, and a branch that does not
    /// descend from it, the trunk the default `main` throughout.
    #[test]
    fn a_land_is_refused_before_anything_is_made() {
        let fx = Fixture::new("commit-land-refused");
        open_cycle(&fx);
        let opening = chid(&fx.work, "@-");
        let refused = |a: &CommitArgs, why: &str| {
            let at = chid(&fx.work, "@");
            let params = CommitParams::try_from(a).expect("params");
            let err = commit(&mut test_ctx(), &params).expect_err(why).to_string();
            assert!(err.contains(why), "{why}: {err}");
            assert_eq!(chid(&fx.work, "@"), at, "{why}: nothing was committed");
            assert_eq!(
                chid(&fx.work, "@-"),
                opening,
                "{why}: nothing was finalized"
            );
        };
        let mut a = closing_args(&fx, "w", "@-");
        a.land = true;
        a.push = true;
        jj_ok(
            &fx.work,
            &["new", "--no-edit", "main", "-m", "a side commit"],
        );
        jj_ok(
            &fx.work,
            &[
                "bookmark",
                "set",
                "main",
                "-r",
                "description(substring:\"a side commit\")",
                "--allow-backwards",
            ],
        );
        refused(&a, "sync first");

        jj_ok(&fx.work, &["new", "main@origin"]);
        fs::write(fx.work.join("other.txt"), "other\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: another cycle landed"]);
        jj_ok(
            &fx.work,
            &["bookmark", "set", "main", "-r", "@-", "--allow-backwards"],
        );
        jj_ok(&fx.work, &["git", "push", "--bookmark", "main"]);
        jj_ok(&fx.work, &["new", &opening]);
        refused(&a, "rebase the branch");
    }

    /// `--land <TRUNK>`, the form before the target was `--onto`'s, is
    /// refused naming `--onto`, not taken for the retired revision.
    #[test]
    fn a_value_after_land_names_onto() {
        let a = parse(&[
            "vc-x1",
            "commit",
            "-s",
            "b",
            "--closing",
            "single-step-abc",
            "--title",
            "feat: add abc.txt",
            "--body",
            "Created abc",
            "--land",
            "main",
            "--push",
        ])
        .expect("parses");
        let err = CommitParams::try_from(&a).unwrap_err();
        assert!(err.contains("`--onto main --land`"), "{err}");
        assert!(!err.contains("-r main"), "{err}");
    }

    /// The target `--land` moves must be a local bookmark: an `--onto`
    /// naming one that does not exist, and one that is a revision and
    /// no bookmark, are refused before anything is made.
    #[test]
    fn a_land_needs_a_bookmark_for_its_target() {
        let fx = Fixture::new("commit-land-no-bookmark");
        open_cycle(&fx);
        fs::write(fx.work.join("rung.txt"), "rung\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: a rung"]);
        fs::write(fx.work.join("closing.txt"), "closing\n").expect("write");
        let (at, tip) = (chid(&fx.work, "@"), chid(&fx.work, "@-"));
        let main = commit_id(&fx.work, "main");
        let refused = |a: &CommitArgs, why: &str| {
            let params = CommitParams::try_from(a).expect("params");
            let err = commit(&mut test_ctx(), &params).expect_err(why).to_string();
            assert!(err.contains(why), "{why}: {err}");
            assert_eq!(chid(&fx.work, "@"), at, "{why}: nothing was committed");
            assert_eq!(chid(&fx.work, "@-"), tip, "{why}: the branch is as it was");
            assert_eq!(commit_id(&fx.work, "main"), main, "{why}: main stays");
        };

        let mut a = closing_args(&fx, "w", "@-");
        a.body = Some("Closing.".to_string());
        a.land = true;
        a.push = true;
        a.onto = Some("no-such-target".to_string());
        refused(&a, "'no-such-target' does not resolve");

        a.onto = Some(main.clone());
        refused(&a, "is not a local bookmark");
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

    /// A draft below `@`, a commit made by hand with no description,
    /// is described in place by REV: the title, body, and trailer
    /// written onto it, its change id kept, the agent commit made as
    /// its partner, `@` left as it is, and the line's bookmark
    /// advanced to it.
    #[test]
    fn a_draft_named_by_rev_is_described_in_place() {
        let fx = Fixture::new("commit-rev");
        fs::write(fx.work.join("draft.md"), "a draft\n").expect("write");
        jj_ok(&fx.work, &["new"]);
        fs::write(fx.work.join("next.md"), "next work\n").expect("write");
        fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write");
        let draft = chid(&fx.work, "@-");
        let at = chid(&fx.work, "@");
        let agent_at = chid(&fx.bot, "@");
        let mut a = args("b", &fx.work);
        a.rev = Some("@-".to_string());
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("described in place");
        assert_eq!(chid(&fx.work, "@-"), draft, "the draft keeps its change id");
        let desc = description(&fx.work, "@-");
        assert!(desc.starts_with("feat: a rung\n"), "{desc}");
        assert!(desc.contains(&format!("ochid: {agent_at}")), "{desc}");
        assert_eq!(chid(&fx.work, "@"), at, "@ is left as it is");
        assert!(
            !jj::is_empty(&fx.work, "@").expect("empty?"),
            "@ keeps its work"
        );
        assert_eq!(
            chid(&fx.bot, "@-"),
            agent_at,
            "the agent @ became the partner"
        );
        let agent_desc = description(&fx.bot, "@-");
        assert!(
            agent_desc.contains(&format!("ochid: {draft}")),
            "{agent_desc}"
        );
        assert_eq!(
            commit_id(&fx.work, "main"),
            commit_id(&fx.work, "@-"),
            "main, the line's bookmark, advanced to the draft"
        );
    }

    /// REV is refused for what is not a draft, before anything is
    /// made: a published commit, a pair's commit that carries a
    /// trailer already, and an empty one; and at params time with a
    /// bookend or with `-s a`.
    #[test]
    fn rev_refuses_what_is_not_a_draft() {
        let fx = Fixture::new("commit-rev-refused");
        let mut a = args("b", &fx.work);
        a.rev = Some("@-".to_string());
        let params = CommitParams::try_from(&a).expect("params");
        let err = commit(&mut test_ctx(), &params).unwrap_err().to_string();
        assert!(err.contains("is published"), "{err}");

        fs::write(fx.work.join("a.md"), "a\n").expect("write");
        let params = CommitParams::try_from(&args("b", &fx.work)).expect("params");
        commit(&mut test_ctx(), &params).expect("the pair");
        let before = description(&fx.work, "@-");
        let mut a = args("b", &fx.work);
        a.rev = Some("@-".to_string());
        a.title = Some("feat: again".to_string());
        let params = CommitParams::try_from(&a).expect("params");
        let err = commit(&mut test_ctx(), &params).unwrap_err().to_string();
        assert!(err.contains("carries an `ochid:` trailer already"), "{err}");
        assert_eq!(
            description(&fx.work, "@-"),
            before,
            "the pair's commit is untouched"
        );

        jj_ok(&fx.work, &["new", "-m", "an empty draft"]);
        jj_ok(&fx.work, &["new"]);
        let mut a = args("b", &fx.work);
        a.rev = Some("@-".to_string());
        let params = CommitParams::try_from(&a).expect("params");
        let err = commit(&mut test_ctx(), &params).unwrap_err().to_string();
        assert!(err.contains("nothing to describe"), "{err}");

        let err =
            parse(&["vc-x1", "commit", "-r", "@-", "-s", "b", "--closing", "@-"]).unwrap_err();
        assert!(err.contains("cannot be used with"), "{err}");
        let mut a = args("a", &fx.work);
        a.rev = Some("@-".to_string());
        let err = CommitParams::try_from(&a).unwrap_err();
        assert!(err.contains("-r is the work commit's"), "{err}");
    }

    /// `-r` and `--revision` name the commit, and the retired positional
    /// parses, hidden, and is refused naming `-r`.
    #[test]
    fn the_revision_is_a_flag_and_the_positional_is_refused() {
        let base = ["vc-x1", "commit", "-s", "b", "--title", "t", "--body", "b"];
        let a = parse(&[&base[..], &["-r", "@-"]].concat()).expect("parses");
        assert_eq!(a.rev.as_deref(), Some("@-"));
        let a = parse(&[&base[..], &["--revision", "@--"]].concat()).expect("parses");
        assert_eq!(a.rev.as_deref(), Some("@--"));
        let a = parse(&[&base[..], &["@-"]].concat()).expect("parses");
        let err = CommitParams::try_from(&a).unwrap_err();
        assert!(err.contains("the positional is retired"), "{err}");
        assert!(err.contains("`-r @-`"), "{err}");
    }

    /// A draft below `@-` is described and paired but kept local,
    /// since push publishes `@-`: the remote is as it was.
    #[test]
    fn a_draft_below_at_minus_is_kept_local() {
        let fx = Fixture::new("commit-rev-deep");
        fs::write(fx.work.join("a.md"), "a\n").expect("write");
        jj_ok(&fx.work, &["new"]);
        fs::write(fx.work.join("b.md"), "b\n").expect("write");
        jj_ok(&fx.work, &["new"]);
        let deep = chid(&fx.work, "@--");
        let remote_before = jj::remote_bookmark_targets(&fx.work, "origin").expect("remote");
        let mut a = args("b", &fx.work);
        a.push = true;
        a.rev = Some("@--".to_string());
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("described in place");
        assert_eq!(chid(&fx.work, "@--"), deep);
        assert!(description(&fx.work, "@--").contains("ochid: "));
        assert_eq!(
            jj::remote_bookmark_targets(&fx.work, "origin").expect("remote"),
            remote_before,
            "nothing was pushed"
        );
    }

    /// `-s p` describes a plain repo's draft in place, no trailer.
    #[test]
    fn a_plain_repos_draft_is_described_in_place() {
        let fx = FixturePor::new("commit-rev-por");
        fs::write(fx.work.join("a.md"), "a\n").expect("write");
        jj_ok(&fx.work, &["new"]);
        let draft = chid(&fx.work, "@-");
        let mut a = args("p", &fx.work);
        a.rev = Some("@-".to_string());
        let params = CommitParams::try_from(&a).expect("params");
        commit(&mut test_ctx(), &params).expect("described in place");
        assert_eq!(chid(&fx.work, "@-"), draft);
        let desc = description(&fx.work, "@-");
        assert!(desc.starts_with("feat: a rung\n"), "{desc}");
        assert!(!desc.contains("ochid:"), "{desc}");
    }
}

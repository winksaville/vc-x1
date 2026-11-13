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
//!   which `--local`, or its alias `--no-push`, says outright.
//! - `-s p` commits a plain repo with the title and body and no
//!   trailer, the same description discipline on a repo with no
//!   partner.
//! - `--body` is the text, or `@PATH` to read it from a file, since
//!   a body in commit-body form has markers and blank lines a shell
//!   quote mangles.
//! - The work side's `@` must hold a change, since a rung changes
//!   the work repo. The agent side's `@` is the session's journal
//!   and is committed as it stands.
//! - `-r REV` is the commit the description is for: `@` by default,
//!   committed as above, else `@-` or any draft below it, described in
//!   place with no new commit, its trailer stamped, so a commit made by
//!   hand becomes a rung's. The draft must be unpublished, no remote
//!   bookmark reaching it, carry no trailer yet, and hold a change, so
//!   the describe is a draft's rewrite and never a re-describe. `@` is
//!   left as it is, as `push` leaves new work, and a draft below `@-`
//!   is committed and not pushed, since push publishes `@-`.
//! - A cycle's acts are `vc-x1 cycle`'s: its opening, its rungs, its
//!   closing, and its Land. `cycle --commit` and `cycle --close` make
//!   their commits by [`commit`] here, and a single-step cycle's
//!   closing by [`finalize`], and the record's move is [`RecordMove`].
//! - The line's bookmark advances to the work commit, locally: the
//!   nearest bookmark below the work `@-`, as `push` resolves it, or
//!   the one `--bookmark` names, any on the line at or below the work
//!   commit, so the cycle's bookmark follows a rung past a nearer one
//!   of the user's. jj moves a bookmark
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
    with_trailer, with_trailers,
};
use crate::jj;
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
    /// by hand becomes a rung's
    #[arg(short = 'r', long = "revision", value_name = "REV")]
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

    /// Commit title, the Conventional Commit line
    #[arg(long, value_name = "STR", required = true)]
    pub title: Option<String>,

    /// Commit body, the problem and its solutions, or @PATH to read
    /// it from a file
    #[arg(long, value_name = "STR|@PATH", required = true)]
    pub body: Option<String>,

    /// Retired: the opening, now `cycle --commit`, refused naming it.
    #[arg(long = "opening", value_name = "RETIRED", hide = true, num_args = 0..=1)]
    pub retired_opening: Option<Option<String>>,

    /// Retired: the closing, now `cycle --close`, refused naming it.
    #[arg(long = "closing", value_name = "RETIRED", hide = true, num_args = 0..=1)]
    pub retired_closing: Option<Option<String>>,

    /// Retired: the Land, now `cycle --land`, refused naming it.
    #[arg(long = "land", value_name = "RETIRED", hide = true, num_args = 0..=1)]
    pub retired_land: Option<Option<String>>,

    /// The bookmark to advance to the work commit, locally, any on the
    /// work commit's line, at or below it [default: the line's nearest]
    #[arg(long, value_name = "NAME", conflicts_with = "no_advance")]
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
    /// pushed
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

/// Which local bookmark the commit advances to the work commit.
#[derive(Debug, PartialEq, Eq)]
pub enum Advance {
    /// The line's: the nearest below the work `@-`.
    Line,
    /// The one `--bookmark` names.
    Named(String),
    /// None, `--no-advance`.
    Off,
}

/// Refuse a bare cycle title that carries a bookend's suffix, since
/// the opening and the closing append it and it is never doubled.
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
    /// The title and the body.
    pub title: Option<String>,
    pub body: Option<String>,
    /// The local bookmark advanced to the work commit.
    pub advance: Advance,
    /// Push what was committed: the flag given, else `[commit] push`.
    pub push: bool,
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
    /// A Land follows in the same invocation and publishes what is
    /// committed, so the commit does not say it is kept local.
    pub land_follows: bool,
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
        for (flag, given, stage) in [
            ("--opening", a.retired_opening.is_some(), "cycle --commit"),
            ("--closing", a.retired_closing.is_some(), "cycle --close"),
            (
                "--land",
                a.retired_land.is_some(),
                "cycle --land <BOOKMARK>",
            ),
        ] {
            if given {
                return Err(format!(
                    "commit {flag}: retired, a cycle's acts being `vc-x1 cycle`'s: `vc-x1 {stage}`"
                ));
            }
        }
        if let Some(value) = &a.retired {
            return Err(crate::options_flags::retired_positional::refusal(
                "commit", value, "-r",
            ));
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
                    Target::Agent(pair)
                } else {
                    Target::Pair {
                        pair,
                        agent_now: sides.is_both(),
                    }
                }
            }
        };
        Ok(CommitParams {
            target,
            title: a.title.clone(),
            body,
            advance: match (&a.bookmark, a.no_advance) {
                (_, true) => Advance::Off,
                (Some(b), false) => Advance::Named(b.clone()),
                (None, false) => Advance::Line,
            },
            push: a.push,
            no_url_trailer: a.no_url_trailer,
            dry_run: a.dry_run.value,
            rev: a.rev.clone(),
            covers: a.covers.clone(),
            allow_empty: false,
            same_version: a.same_version,
            land_follows: false,
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

/// Entry point: the pair, the plain commit, or the agent's alone.
pub fn commit(ctx: &mut Context, params: &CommitParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("commit: enter {params:?}");
    match &params.target {
        Target::Pair { pair, agent_now } => commit_pair(ctx, pair, *agent_now, params),
        Target::Por(repo) => commit_por(ctx, repo, params),
        Target::Agent(pair) => commit_agent(ctx, pair, params),
    }
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
    let draft = plan_draft(work, params)?;
    match &draft {
        None => preflight(work, "work", !params.allow_empty)?,
        Some(_) => crate::pair::refuse_conflicts(work, "commit", "work")?,
    }
    preflight(agent, "agent", false)?;
    check_version(
        work,
        draft.as_ref().map_or("@", |d| d.rev.as_str()),
        params.same_version,
        "commit",
    )?;
    let text = description(params)?;
    let plan = plan_bookmarks(work, params, draft.as_ref())?;
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
        dry_run_bookmarks(&plan);
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
        );
    }
    let snapshots = Snapshots::take(pair)?;
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
    finish_bookmarks(ctx, work, &plan)?;
    if agent_now {
        info!(
            "commit: the pair is made, review it with `vc-x1 show -s both {work_at} -R {}`",
            work.display()
        );
    } else {
        info!(
            "commit: the work half is made, its trailer naming the agent @ {agent_chid}, which \
             push commits as the partner. Review it with `vc-x1 show -s work @- -R {}`",
            work.display()
        );
    }
    push_after(
        ctx,
        work,
        Publish::Pair,
        plan.advance.as_deref(),
        params,
        false,
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
    let head = &text[..from];
    // A blank line before the content, whether the section has an intro
    // or the content follows its heading directly.
    let gap = if head.ends_with("\n\n") {
        ""
    } else if head.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    let tail = if end == text.len() { "\n" } else { "\n\n" };
    format!("{head}{gap}{}{tail}{}", content.trim_end(), &text[end..])
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
        format!("cycle --close: {RECORD} has no `## Closed` to move the cycle into")
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
    /// `TODO.md`'s text after the move, which `cycle --close` trims for
    /// a single-step cycle.
    pub(crate) new_record: String,
}

impl RecordMove {
    /// Plan the move when the repo's `TODO.md` has a cycle in
    /// progress, reading and checking it, so a refusal comes before
    /// anything is written. Nothing to move is said and is no error.
    pub(crate) fn plan(repo: &Path) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let record = repo.join(RECORD);
        if !record.is_file() {
            info!("cycle --close: no {RECORD}, so no cycle-record moves");
            return Ok(None);
        }
        let Some(new_record) = moved_record(&std::fs::read_to_string(&record)?)? else {
            info!("cycle --close: nothing in progress in {RECORD}, so no cycle-record moves");
            return Ok(None);
        };
        Ok(Some(Self { record, new_record }))
    }

    /// Write the move into the working copy, so the commit carries it,
    /// or with `dry_run` say what would move.
    pub(crate) fn write(&self, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
        let record = self.record.display();
        let tag = if dry_run {
            "cycle --close: [dry-run]"
        } else {
            "cycle --close:"
        };
        info!("{tag} move the In Progress block of {record} into its Closed");
        if dry_run {
            return Ok(());
        }
        std::fs::write(&self.record, &self.new_record)?;
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
) -> Result<(), Box<dyn std::error::Error>> {
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
        if params.land_follows {
            info!("commit: kept local until the Land that follows publishes it");
        } else {
            info!("commit: kept local, `vc-x1 push` publishes it");
        }
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
    let text = description(params)?;
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

/// The commit's description, its title and its body.
fn description(params: &CommitParams) -> Result<String, Box<dyn std::error::Error>> {
    match (&params.title, &params.body) {
        (Some(title), Some(body)) => Ok(format!("{title}\n\n{body}")),
        _ => Err("commit: --title and --body are required".into()),
    }
}

/// Finalize a single-step cycle's one commit in place, `cycle
/// --close`'s squash of the closing into the opening: fold the work
/// `@` into `@-` and give it `title` and `body`, its trailer kept, so
/// its change id and its partner stand. The agent `@` is left for push,
/// which commits or folds it as the partner. The line's bookmark
/// follows the rewrite.
pub(crate) fn finalize(
    ctx: &mut Context,
    repo: &Path,
    pair: Option<&Pair>,
    title: &str,
    body: &str,
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
                    "cycle --close: the work @- carries no `ochid:` trailer, so it is not a \
                     pair's commit to squash the closing into"
                        .into(),
                );
            }
        },
        None => None,
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
/// bookmark to advance, the one named, any on the work commit's line,
/// at or below it, so the cycle's bookmark follows a rung past a nearer
/// one of the user's, or the line's, the nearest, refused when there
/// are none or several, since a wrong guess moves the wrong branch.
fn plan_bookmarks(
    repo: &Path,
    params: &CommitParams,
    draft: Option<&Draft>,
) -> Result<BookmarkPlan, Box<dyn std::error::Error>> {
    // The work commit's line, the commit included: `@` when `@` is
    // committed, whose bookmark, `cycle --open`'s, follows it to `@-`,
    // so `main` below it is not the nearest, or the draft itself.
    let (below, target) = match draft {
        None => ("@".to_string(), "@-".to_string()),
        Some(d) => (d.rev.clone(), jj::chid_of(repo, &d.rev)?),
    };
    let advance = match &params.advance {
        Advance::Off => None,
        Advance::Named(b) => {
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
        Advance::Line => Some(
            crate::squash_push::current_bookmark(repo, &below).map_err(|e| {
                format!("commit: {e} with --bookmark <NAME>, or pass --no-advance to move none")
            })?,
        ),
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

/// Commit a plain repo with the description and no trailer.
fn commit_por(
    ctx: &mut Context,
    repo: &Path,
    params: &CommitParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let draft = plan_draft(repo, params)?;
    match &draft {
        None => preflight(repo, "plain", !params.allow_empty)?,
        Some(_) => crate::pair::refuse_conflicts(repo, "commit", "plain")?,
    }
    check_version(
        repo,
        draft.as_ref().map_or("@", |d| d.rev.as_str()),
        params.same_version,
        "commit",
    )?;
    let desc = description(params)?;
    let plan = plan_bookmarks(repo, params, draft.as_ref())?;
    let at = draft.as_ref().map_or("@", |d| d.rev.as_str());
    let chid = jj::chid_of(repo, at)?;
    if params.dry_run {
        dry_run_bookmarks(&plan);
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
        );
    }
    let snapshot = crate::pair::Snapshot::take(repo)?;
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

    fn commit_id(repo: &Path, rev: &str) -> String {
        jj_ok(repo, &["log", "-r", rev, "--no-graph", "-T", "commit_id"])
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
    /// untouched, and `-s p` on a side is still the resolver's refusal.
    #[test]
    fn the_agent_repo_is_committed_alone() {
        let fx = Fixture::new("commit-agent");
        let err = CommitParams::try_from(&args("p", &fx.bot)).unwrap_err();
        assert!(err.contains("the agent side"), "{err}");

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

    /// The bookmark `vc-x1 open` makes on the work `@`, as a test makes
    /// it, so the opening takes it.
    fn open_bookmark(fx: &Fixture, name: &str) {
        jj_ok(&fx.work, &["bookmark", "create", name, "-r", "@"]);
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
        // A `## Closed` with no intro, the file's last line, still gets a
        // blank line between its heading and the block.
        let bare = "## In Progress\n\n### feat: a cycle\n\nThe record.\n\n## Closed\n";
        assert_eq!(
            moved_record(bare).expect("moved").expect("a cycle"),
            "## In Progress\n\n_No cycle currently in progress._\n\n## Closed\n\n\
             ### feat: a cycle\n\nThe record.\n"
        );
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

        let mut a = args("a", &fx.work);
        a.rev = Some("@-".to_string());
        let err = CommitParams::try_from(&a).unwrap_err();
        assert!(err.contains("-r is the work commit's"), "{err}");
    }

    /// The cycle's flags are retired: each parses, hidden, and is refused
    /// naming the `cycle` stage that took its place.
    #[test]
    fn the_cycles_flags_are_retired() {
        let fx = Fixture::new("commit-retired-flags");
        for (flag, stage) in [
            (vec!["--opening"], "cycle --commit"),
            (vec!["--closing", "@-"], "cycle --close"),
            (vec!["--land"], "cycle --land"),
        ] {
            let mut argv = vec!["vc-x1", "commit", "-s", "b", "--title", "t", "--body", "b"];
            argv.extend(&flag);
            let mut a = parse(&argv).expect("parses");
            a.repo = Some(fx.work.clone());
            let err = CommitParams::try_from(&a).unwrap_err();
            assert!(err.contains("retired") && err.contains(stage), "{err}");
        }
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

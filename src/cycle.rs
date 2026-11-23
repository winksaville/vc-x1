//! The `cycle` subcommand: a cycle's stages, each a subcommand of its
//! own, beside [`rung`]'s acts. The bookmark is [`crate::open`]'s to
//! make, the commits [`crate::commit`]'s, and the Land
//! [`crate::land`]'s, none of which is a command of its own for a
//! cycle.
//!
//! - `plan` opens a cycle: the bookmark first, on the work `@`, as
//!   [`crate::open`] makes it, then the cycle's record, the edits
//!   [`record`] plans. Everything is planned and checked before
//!   anything is made, so a refusal leaves the workspace as it was.
//! - `ladder` makes `zTOC.md`'s ladder from the rung headings in
//!   `TODO.md`, each `(done)` computed from the commits on the cycle's
//!   branch and a `(current)` kept, so a rung is written once and no
//!   marker is typed. [`rung`] starts a rung and commits it.
//! - `close` and `land` make the closing and the Land, by `commit`'s
//!   and `land`'s own code, `close --land` chaining the two, and `push`
//!   publishes. `status` reads where the cycle stands.
//! - The record is the agent-files' shape and not the work's, so the
//!   tool writes it without knowing the work. The version is the
//!   project's, and no stage bumps it, `rung next --ver` running the
//!   project's own bump.

pub(crate) mod record;
pub(crate) mod rung;
mod squash;

use std::path::PathBuf;

use clap::{Args, Subcommand};
use log::{debug, info, warn};

use crate::context::Context;
use crate::jj;
use crate::open::OpenParams;
use crate::options_flags::dry_run::DryRunFlag;
use crate::options_flags::scope::{RepoScope, repo_scope_parser};
use crate::subcommand::SubcommandRunner;

/// Run a cycle's stages.
#[derive(Args, Debug)]
pub struct CycleArgs {
    #[command(subcommand)]
    pub command: CycleCommand,
}

/// The `cycle` subcommands, its stages.
#[derive(Subcommand, Debug)]
pub enum CycleCommand {
    /// Plan a cycle: `start` makes the bookmark and the record, `commit` commits the plan
    #[command(long_about = "\
        A cycle's plan, a phase of its own: `start` makes the bookmark and\n\
        writes the record, the plan is deliberated and its rungs headed in\n\
        it, and `commit` commits it, the opening, the ladder made from the\n\
        headings. `rung next` and `rung commit` then run each work rung, and\n\
        `cycle close` ends the cycle, a trapezoid, linear, or one commit by\n\
        --squash. `start`, the work, and `cycle close --squash` is a\n\
        one-off's path.")]
    Plan(PlanArgs),

    /// Make zTOC.md's ladder from the rung headings in TODO.md
    #[command(long_about = "\
        Make the ladder in zTOC.md from the rung headings, the `##### `\n\
        headings under `#### Rungs` of the cycle in progress in TODO.md, in\n\
        their order, so a rung is written once. Each line is marked\n\
        `(done)` and numbered by its position when a commit of the rung's\n\
        title is on the cycle's branch, a rung `rung next` started is kept\n\
        `(current)`, and every other rung is its link alone. No marker is\n\
        made here. Refused, with nothing edited: a commit on the branch that\n\
        is no rung's, a rung written twice, a ladder not opened and closed\n\
        by the cycle title plus \" opening\" and \" closing\", and a line of\n\
        the ladder there that is no rung by its grammar. The branch is the\n\
        commits the cycle's bookmark reaches that no other local bookmark\n\
        does, the bookmark the title's slug or --bookmark's, and before the\n\
        bookmark is made the ladder has no rung done.")]
    Ladder(Where),

    /// Say where the cycle stands, changing nothing
    #[command(long_about = "\
        Say where the cycle stands and change nothing: the title, the\n\
        bookmark and whether it is on the work @, the commits on the branch\n\
        and how many are not pushed, the ladder as the rungs and the commits\n\
        make it, a note when zTOC.md's differs, and the stage that comes\n\
        next. What another stage would refuse, it says.")]
    Status(Where),

    /// Close the cycle: a trapezoid on TARGET, --linear, or --squash one commit
    #[command(long_about = "\
        Close the cycle from the work @, which is on the cycle's tip. Its\n\
        shape is said here, and only here:\n\n\
        \x20 - a trapezoid, the default: `cycle close <TARGET>`, the closing a\n\
        \x20   merge whose first parent is TARGET, read where it is now, named\n\
        \x20   each time as a target always is;\n\
        \x20 - --linear: the closing a plain commit on the ladder;\n\
        \x20 - --squash: the cycle one commit under the bare cycle title, its\n\
        \x20   rungs and the work @ squashed into the opening, whose change id\n\
        \x20   jj keeps, and its agent side re-pointed, each agent commit naming\n\
        \x20   a rung rewritten to name the one work commit, which names them\n\
        \x20   all. With no commit the work @ is committed now, a one-off. On a\n\
        \x20   cycle closed and not landed, the closing is squashed in with the\n\
        \x20   rungs, converting it, refused when the trapezoid's target has\n\
        \x20   moved since. --body is required, the commit holding the work,\n\
        \x20   unless --default-body asks for the pointer at the record, and\n\
        \x20   an agent commit `main@origin` reaches is rewritten only with\n\
        \x20   --yes.\n\n\
        A trapezoid's or a linear closing is titled the cycle title plus\n\
        \" closing\" with `[closing] body`, unless --title or --body is given,\n\
        and refused with no opening committed. --join REV,... joins other\n\
        heads, each a parent after @'s, and --dead-end titles the closing \"\n\
        dead-end closing\". The ladder is rewritten, every rung done, and the\n\
        record moved to `## Closed`, both in the commit. The acceptance check\n\
        comes first, by hand, and again after. Refused, nothing rewritten, by\n\
        --squash: a work commit of the branch a remote bookmark other than\n\
        the cycle's reaches.\n\n\
        --land lands the closing in the same invocation, as `cycle land`\n\
        does, on the trapezoid's TARGET when bare, `--land <TARGET>` for\n\
        --linear and --squash, its checks made before the closing is, and\n\
        --push then publishes the target. Without --land, --push is the\n\
        draft push of the cycle's bookmark. -s is required: `b` the pair, `w`\n\
        the work half alone, refused by --squash, `p` a plain repo's commit.\n\
        With `[version] file` and `pattern` in the config, a closing that\n\
        changes no line of the file starting with the pattern is refused, and\n\
        --same-version commits it as it is.")]
    Close(CloseArgs),

    /// Land the closed cycle on TARGET
    #[command(long_about = "\
        Land a closed cycle on TARGET, any bookmark, none assumed: the agent\n\
        @ committed when the cycle's commits name it, the target\n\
        fast-forwarded to the cycle's last commit, and the cycle's bookmark\n\
        deleted, --skip keeping a side. The cycle is the one `## Closed`\n\
        holds, its bookmark the title's slug or --bookmark's, and a cycle\n\
        still in progress is refused. A target that moved is refused, naming\n\
        the ways on, and --merge is one: a merge commit of the target and\n\
        the cycle's last commit, made from an empty @ and then landed, a\n\
        conflicted one undone and left to you, --title and --body its own.\n\
        --push publishes the target and deletes the cycle's bookmark on\n\
        origin. -s is required, `b` or `p`.")]
    Land(LandArgs),

    /// Publish what the cycle's state calls for
    #[command(long_about = "\
        Publish what the cycle's state calls for. A cycle in progress, or\n\
        closed and not landed, is a draft, and its bookmark is pushed. A\n\
        cycle landed here publishes the target its landmark is on and\n\
        deletes the cycle's bookmark on origin, and with nothing to publish\n\
        it says so. -s is required, `b` or `p`.")]
    Push(PushArgs),
}

/// Where, and the cycle's bookmark when it is not the title's slug,
/// the arguments every stage and every rung act shares.
#[derive(Args, Debug)]
pub struct Where {
    /// The cycle's bookmark, in place of the title's slug; bare, with
    /// `rung commit --title`, the title's slug made or moved for it
    #[arg(long, value_name = "NAME", num_args = 0..=1, default_missing_value = "")]
    pub bookmark: Option<String>,

    /// Where: a path in the workspace, its root or either repo, or a
    /// plain repo [default: the current directory]
    #[arg(short = 'R', long, value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Say what would be done, and do none of it
    #[command(flatten)]
    pub dry_run: DryRunFlag,
}

impl Where {
    /// The cycle this names, and the agent repo of a dual workspace.
    pub(crate) fn running(&self) -> Result<(Running, Option<PathBuf>), String> {
        if self.bookmark.as_deref() == Some("") {
            return Err(
                "--bookmark: a bare --bookmark is `rung commit --title`'s, the title's slug: name \
                 the bookmark, --bookmark <NAME>"
                    .into(),
            );
        }
        self.running_bare()
    }

    /// As [`Where::running`], a bare `--bookmark` taken as none, for the
    /// one act that reads it, `rung commit --title`.
    pub(crate) fn running_bare(&self) -> Result<(Running, Option<PathBuf>), String> {
        let (work, agent) = resolve_repos(self.repo.as_deref())?;
        Ok((
            Running {
                work,
                bookmark: self.bookmark.clone().filter(|b| !b.is_empty()),
                dry_run: self.dry_run.value,
            },
            agent,
        ))
    }
}

/// Which repos a stage that writes takes, required.
#[derive(Args, Debug)]
pub struct Scope {
    /// Which repos, required: `b|both` a dual workspace's pair, `w|work`
    /// the work half with the agent half left to push, and `p|por` a
    /// plain repo
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = repo_scope_parser(&["work", "both", "por"]),
        help = "Which repos, required: `w|work`, `b|both`, or `p|por`"
    )]
    pub scope: RepoScope,
}

/// `cycle plan`'s arguments.
#[derive(Args, Debug)]
pub struct PlanArgs {
    #[command(subcommand)]
    pub command: PlanCommand,
}

/// The `cycle plan` subcommands.
#[derive(Subcommand, Debug)]
pub enum PlanCommand {
    /// Start a plan: make its bookmark on the work `@`, then write its record
    #[command(long_about = "\
        Start a cycle's plan: make the bookmark on the work @ first, the\n\
        slug of --title or --bookmark's name, kept local, then write the\n\
        cycle's record. In TODO.md what `## Closed` holds is deleted and the\n\
        `## Todo` entry titled as --title is moved into an `## In Progress`\n\
        scaffold, an empty one when no entry matches, and in zTOC.md the\n\
        ladder is replaced with the opening and the closing. A workspace with\n\
        no TODO.md or no zTOC.md gets a skeleton of the file first, the\n\
        headings the record needs, and a file there is kept. Every check is\n\
        made before anything changes: a cycle in progress is refused, as is\n\
        a name in use. --no-record makes the bookmark alone, writing no\n\
        file, a branch the later stages will not take, as does --bookmark\n\
        without --title. With --ver the opening's version is set last, by\n\
        the config's `[version] bump`, so the plan under review carries it.\n\
        Nothing is committed or pushed.")]
    Start(PlanStartArgs),

    /// Commit the plan: the ladder made from its headings, then the opening committed
    #[command(long_about = "\
        Commit the plan, the cycle's opening: the ladder in zTOC.md is made\n\
        from the rung headings under `#### Rungs` and rewritten into the\n\
        commit, the opening `(done)`, and the working copies committed by\n\
        the code `vc-x1 commit` runs, the title the cycle title plus \"\n\
        opening\" and the body `[opening] body`, a --body given taking its\n\
        place. Refused when the plan is committed already. -s is\n\
        required: `b` the pair, `w` the work half alone, `p` a plain repo's\n\
        commit. --push publishes the cycle's bookmark, a draft.")]
    Commit(PlanCommitArgs),
}

/// `cycle plan start`'s arguments.
#[derive(Args, Debug)]
pub struct PlanStartArgs {
    /// The bare cycle title, a Conventional Commit: the bookmark is its
    /// slug, and the record is written under it
    #[arg(long, value_name = "STR")]
    pub title: Option<String>,

    /// Make the bookmark alone and write no record
    #[arg(long)]
    pub no_record: bool,

    /// The opening's version, set by the config's `[version] bump` once
    /// the record is written [default: the version left as it is]
    #[arg(long, value_name = "VERSION")]
    pub ver: Option<String>,

    #[command(flatten)]
    pub at: Where,
}

/// `cycle plan commit`'s arguments.
#[derive(Args, Debug)]
pub struct PlanCommitArgs {
    /// Which repos, required: `b|both` a dual workspace's pair,
    /// `w|work` the work half with the agent half left to push, and
    /// `p|por` a plain repo
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = repo_scope_parser(&["work", "both", "por"]),
        help = "Which repos, required: `w|work`, `b|both`, or `p|por`"
    )]
    pub scope: RepoScope,

    /// The opening's body, or @PATH to read it from a file [default:
    /// `[opening] body`]
    #[arg(long, value_name = "STR|@PATH")]
    pub body: Option<String>,

    /// Publish the cycle's bookmark once committed, a draft push
    #[arg(long)]
    pub push: bool,

    /// Commit although the version's line is unchanged
    #[arg(long)]
    pub same_version: bool,

    #[command(flatten)]
    pub at: Where,
}

/// `cycle close`'s arguments.
#[derive(Args, Debug)]
pub struct CloseArgs {
    /// The target, the trapezoid's first parent, read where it is now:
    /// required for a trapezoid, the default shape
    #[arg(value_name = "TARGET")]
    pub target: Option<String>,

    /// Make the cycle one commit: its rungs, and a closing already made,
    /// squashed into the opening
    #[arg(long, conflicts_with_all = ["target", "linear", "join", "dead_end", "title"])]
    pub squash: bool,

    /// Make the closing a plain commit on the ladder, no merge
    #[arg(long, conflicts_with = "target")]
    pub linear: bool,

    /// Other heads joined, each a parent of the closing after @'s own
    #[arg(long, value_name = "REV", value_delimiter = ',')]
    pub join: Vec<String>,

    #[command(flatten)]
    pub scope: Scope,

    /// The closing's title [default: the cycle title, plus " closing"
    /// on a ladder]
    #[arg(long, value_name = "STR")]
    pub title: Option<String>,

    /// The commit's body, the problem and its solution, or @PATH to
    /// read it from a file [default: `[closing] body` for a ladder's
    /// closing]
    #[arg(long, value_name = "STR|@PATH")]
    pub body: Option<String>,

    /// With --squash, give the one commit the default body, `[closing]
    /// body`, a pointer at the record, in place of --body
    #[arg(long, conflicts_with = "body")]
    pub default_body: bool,

    /// With --squash, rewrite agent commits `main@origin` reaches, the
    /// next agent push then a forced one
    #[arg(long)]
    pub yes: bool,

    /// Close the cycle as a dead end: its closing's title ending
    /// " dead-end closing"
    #[arg(long)]
    pub dead_end: bool,

    /// Land the closing in the same invocation, as `cycle land` does:
    /// bare, on the trapezoid's TARGET
    #[arg(
        long,
        value_name = "TARGET",
        num_args = 0..=1,
        default_missing_value = "",
        help_heading = "Land"
    )]
    pub land: Option<String>,

    /// With `--land`, which side of the cycle's bookmark is kept:
    /// `local`, `remote`, or `both`, none deleted
    #[arg(
        long,
        value_enum,
        value_name = "SIDE",
        requires = "land",
        help_heading = "Land"
    )]
    pub skip: Option<crate::land::Skip>,

    /// With `--land`, also delete the bookmarks on the commits strictly
    /// between the target and the cycle's last commit
    #[arg(long, requires = "land", help_heading = "Land")]
    pub delete_intermediate_bookmarks: bool,

    /// Publish: the cycle's bookmark, a draft push, or with `--land`
    /// the target
    #[arg(long)]
    pub push: bool,

    /// Commit although the version's line is unchanged
    #[arg(long)]
    pub same_version: bool,

    #[command(flatten)]
    pub at: Where,
}

/// `cycle land`'s arguments.
#[derive(Args, Debug)]
pub struct LandArgs {
    /// The target, the bookmark the cycle lands on
    #[arg(value_name = "TARGET")]
    pub target: String,

    #[command(flatten)]
    pub scope: Scope,

    /// Join a target that moved by a merge commit, the target its first
    /// parent and the cycle's last commit its second, made and then
    /// landed, a tree no rung validated [default: a target that moved is
    /// refused]
    #[arg(long)]
    pub merge: bool,

    /// With `--merge`, the merge commit's title
    #[arg(long, value_name = "STR", requires = "merge")]
    pub title: Option<String>,

    /// With `--merge`, the merge commit's body, or @PATH to read it from
    /// a file
    #[arg(long, value_name = "STR|@PATH", requires = "merge")]
    pub body: Option<String>,

    /// Which side of the cycle's bookmark is kept: `local`, `remote`, or
    /// `both`, none deleted
    #[arg(long, value_enum, value_name = "SIDE")]
    pub skip: Option<crate::land::Skip>,

    /// Also delete the bookmarks on the commits strictly between the
    /// target and the cycle's last commit, a stack of cycles landing
    /// together [default: named as merged and kept]
    #[arg(long)]
    pub delete_intermediate_bookmarks: bool,

    /// Publish the target, and delete the cycle's bookmark on origin
    #[arg(long)]
    pub push: bool,

    #[command(flatten)]
    pub at: Where,
}

/// `cycle push`'s arguments.
#[derive(Args, Debug)]
pub struct PushArgs {
    #[command(flatten)]
    pub scope: Scope,

    #[command(flatten)]
    pub at: Where,
}

/// `cycle plan`'s inputs: the bookmark's, as [`crate::open::open`]
/// takes them, and the title the record is written under, `None` when
/// no record is.
#[derive(Debug)]
pub struct OpenCycle {
    pub open: OpenParams,
    pub record: Option<String>,
    /// Whether the plan ends saying what comes next, a plan of its own
    /// and not one a commit follows in the same invocation.
    pub hint: bool,
    /// The opening's version, set once the record is written.
    pub ver: Option<String>,
}

/// The cycle a stage after `cycle plan` works on: its work repo and
/// the bookmark named, the title's slug when none is.
#[derive(Debug, Clone)]
pub struct Running {
    pub work: PathBuf,
    pub bookmark: Option<String>,
    pub dry_run: bool,
}

/// A rung's commit's inputs, `rung commit`'s and `cycle close`'s: the
/// cycle, the repos the commit is made in, and the title and body
/// given.
#[derive(Debug, Clone)]
pub struct CommitRung {
    pub running: Running,
    /// The agent repo, and whether its commit is made now, `None` for
    /// a plain repo.
    pub agent: Option<(PathBuf, bool)>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub push: bool,
    pub same_version: bool,
}

/// `cycle close`'s inputs: the commit's, as a rung's, and the
/// closing's shape.
#[derive(Debug, Clone)]
pub struct CloseCycle {
    pub rung: CommitRung,
    /// The other heads joined, each a revision of one commit.
    pub heads: Vec<String>,
    /// The target, the closing's first parent, for a trapezoid.
    pub trapezoid: Option<String>,
    pub default_body: bool,
    pub dead_end: bool,
    /// The shape named: one commit, or a ladder with its own closing.
    pub single_step: bool,
    /// The Land that follows the closing, in the same invocation.
    pub land: Option<LandCycle>,
}

/// `cycle close --squash`'s inputs: the single-step commit it ends in,
/// and whether agent commits `main@origin` reaches may be rewritten.
#[derive(Debug)]
pub struct SquashCycle {
    pub one: CloseCycle,
    pub yes: bool,
}

/// `cycle land`'s inputs: the cycle's work repo, the agent repo of a
/// dual workspace, and the target.
#[derive(Debug, Clone)]
pub struct LandCycle {
    pub running: Running,
    pub agent: Option<PathBuf>,
    pub dest: String,
    pub merge: bool,
    pub skip: Option<crate::land::Skip>,
    pub delete_intermediate: bool,
    pub push: bool,
    /// The merge commit's title and body, when given.
    pub title: Option<String>,
    pub body: Option<String>,
}

/// `cycle push`'s inputs: the cycle and the repos a push publishes.
#[derive(Debug)]
pub struct PushCycle {
    pub running: Running,
    pub agent: Option<PathBuf>,
}

/// Inputs to the cycle op, flat, owned, clap-free, a stage each.
#[derive(Debug)]
pub enum CycleParams {
    Open(OpenCycle),
    PlanCommit(CommitRung),
    Ladder(Running),
    Close(CloseCycle),
    Squash(SquashCycle),
    Land(LandCycle),
    Status(Running),
    Push(PushCycle),
}

/// The work repo and, in a dual workspace, the agent repo that `-R`
/// names, a plain repo having no agent.
fn resolve_repos(repo: Option<&std::path::Path>) -> Result<(PathBuf, Option<PathBuf>), String> {
    match crate::pair::resolve_pair(repo) {
        Ok(pair) => Ok((pair.work, Some(pair.agent))),
        Err(_) => {
            let repos = crate::common::resolve_scoped_repos(&RepoScope::Por, repo)
                .map_err(|e| e.to_string())?;
            match repos.as_slice() {
                [repo] => Ok((repo.clone(), None)),
                other => Err(format!("cycle: expected one repo, got {}", other.len())),
            }
        }
    }
}

/// The agent repo a commit is made beside, and whether its commit is
/// made now, by `-s`: refused when it asks for a pair where there is
/// none or a plain commit where there is one.
fn commit_scope(
    scope: &RepoScope,
    agent: Option<PathBuf>,
    label: &str,
) -> Result<Option<(PathBuf, bool)>, String> {
    match (scope, agent) {
        (RepoScope::Por, None) => Ok(None),
        (RepoScope::Por, Some(_)) => Err(format!(
            "{label} -s p: this is a dual workspace, whose rung is a pair: -s b or -s w"
        )),
        (RepoScope::Sides(_), None) => Err(format!(
            "{label}: this is a plain repo, with no agent repo to pair with: -s p"
        )),
        (RepoScope::Sides(sides), Some(agent)) => Ok(Some((agent, sides.is_both()))),
    }
}

/// The agent repo a publishing stage takes, by `-s`: the work half
/// alone is refused, since it would publish trailers naming agent
/// commits not yet published.
fn publish_scope(
    scope: &RepoScope,
    agent: Option<PathBuf>,
    label: &str,
) -> Result<Option<PathBuf>, String> {
    if matches!(scope, RepoScope::Sides(s) if !s.is_both()) {
        return Err(format!(
            "{label} -s w: the work half alone would publish trailers naming agent commits its \
             agent half may not have published: -s b"
        ));
    }
    Ok(commit_scope(scope, agent, label)?.map(|(a, _)| a))
}

/// A rung's commit's params, `rung commit`'s and `cycle close`'s alike,
/// the body read and neither the title nor the body blank.
#[allow(clippy::too_many_arguments)]
fn rung_params(
    running: Running,
    agent: Option<PathBuf>,
    scope: &RepoScope,
    title: Option<&str>,
    body: Option<&str>,
    push: bool,
    same_version: bool,
    label: &str,
) -> Result<CommitRung, String> {
    let body = match body {
        Some(b) => Some(crate::desc_helpers::read_body(b, label)?),
        None => None,
    };
    if title.is_some_and(|t| t.trim().is_empty())
        || body.as_deref().is_some_and(|b| b.trim().is_empty())
    {
        return Err(format!("{label}: a --title or --body given is not blank"));
    }
    Ok(CommitRung {
        agent: commit_scope(scope, agent, label)?,
        running,
        title: title.map(|t| t.trim().to_string()),
        body,
        push,
        same_version,
    })
}

/// `cycle plan`'s params, a subcommand's.
fn plan_params(a: &PlanArgs) -> Result<CycleParams, String> {
    match &a.command {
        PlanCommand::Start(s) => plan_start_params(s),
        PlanCommand::Commit(c) => {
            let (running, agent) = c.at.running()?;
            Ok(CycleParams::PlanCommit(rung_params(
                running,
                agent,
                &c.scope,
                None,
                c.body.as_deref(),
                c.push,
                c.same_version,
                "cycle plan commit",
            )?))
        }
    }
}

/// `cycle plan start`'s params.
fn plan_start_params(a: &PlanStartArgs) -> Result<CycleParams, String> {
    let (running, agent) = a.at.running()?;
    if a.title.is_none() && a.at.bookmark.is_none() {
        return Err("cycle plan start: --title or --bookmark names the bookmark".into());
    }
    if a.ver.as_deref().is_some_and(|v| v.trim().is_empty()) {
        return Err("cycle plan start: --ver is blank".into());
    }
    let bookmark = crate::open::bookmark_name(a.at.bookmark.as_deref(), a.title.as_deref())?;
    if let Some(t) = &a.title {
        crate::commit::refuse_suffixed(t, "cycle plan --title")
            .map_err(|e| e.replacen("commit ", "", 1))?;
    }
    Ok(CycleParams::Open(OpenCycle {
        open: OpenParams {
            work: running.work,
            agent,
            bookmark,
            dry_run: running.dry_run,
        },
        record: match (&a.title, a.no_record) {
            (Some(t), false) => Some(t.trim().to_string()),
            _ => None,
        },
        hint: true,
        ver: a.ver.as_deref().map(|v| v.trim().to_string()),
    }))
}

/// `cycle close`'s params, its Land's with them when `--land` chains
/// one. The shape is the trapezoid on TARGET unless `--linear` or
/// `--squash` says otherwise, and a trapezoid with no TARGET is
/// refused, a target never assumed.
fn close_params(a: &CloseArgs) -> Result<CycleParams, String> {
    let label = if a.squash {
        "cycle close --squash"
    } else {
        "cycle close"
    };
    if !a.squash && !a.linear && a.target.is_none() {
        return Err(
            "cycle close: the closing is a trapezoid by default, its first parent the target, \
             named each time: `cycle close <TARGET>`, or --linear for a plain closing, or \
             --squash for one commit"
                .into(),
        );
    }
    // clap counts a flag at its default as present, so `requires` cannot
    // say these need --squash.
    for (given, flag) in [(a.default_body, "--default-body"), (a.yes, "--yes")] {
        if given && !a.squash {
            return Err(format!(
                "cycle close {flag}: it is the one commit's, and needs --squash"
            ));
        }
    }
    if a.squash && matches!(&a.scope.scope, RepoScope::Sides(s) if !s.is_both()) {
        return Err(format!(
            "{label} -s w: the agent side is re-pointed with the work side: -s b, or -s p for a \
             plain repo"
        ));
    }
    if a.squash && a.body.is_none() && !a.default_body {
        return Err(format!(
            "{label}: the one commit holds the work, so it gives --body, its problem and \
             solution, or --default-body for the pointer at the record"
        ));
    }
    let (running, agent) = a.at.running()?;
    let land = match a.land.as_deref() {
        Some(dest) => {
            let dest = match (dest, &a.target) {
                ("", Some(t)) => t.clone(),
                ("", None) => {
                    return Err(format!(
                        "{label} --land: a bare --land lands on the trapezoid's TARGET, and there \
                         is none: --land <TARGET>"
                    ));
                }
                (d, _) => d.to_string(),
            };
            Some(LandCycle {
                agent: publish_scope(&a.scope.scope, agent.clone(), &format!("{label} --land"))?,
                running: running.clone(),
                dest,
                merge: false,
                skip: a.skip,
                delete_intermediate: a.delete_intermediate_bookmarks,
                push: a.push,
                title: None,
                body: None,
            })
        }
        None => None,
    };
    let rung = rung_params(
        running,
        agent,
        &a.scope.scope,
        a.title.as_deref(),
        a.body.as_deref(),
        // With a Land the push is the Land's, the target's.
        a.push && land.is_none(),
        a.same_version,
        label,
    )?;
    let close = CloseCycle {
        rung,
        heads: a.join.clone(),
        trapezoid: if a.squash || a.linear {
            None
        } else {
            a.target.clone()
        },
        default_body: a.default_body,
        dead_end: a.dead_end,
        single_step: a.squash,
        land,
    };
    Ok(if a.squash {
        CycleParams::Squash(SquashCycle {
            one: close,
            yes: a.yes,
        })
    } else {
        CycleParams::Close(close)
    })
}

/// `cycle land`'s params.
fn land_params(a: &LandArgs) -> Result<CycleParams, String> {
    let (running, agent) = a.at.running()?;
    let body = match &a.body {
        Some(b) => Some(crate::desc_helpers::read_body(b, "cycle land")?),
        None => None,
    };
    Ok(CycleParams::Land(LandCycle {
        agent: publish_scope(&a.scope.scope, agent, "cycle land")?,
        running,
        dest: a.target.clone(),
        merge: a.merge,
        skip: a.skip,
        delete_intermediate: a.delete_intermediate_bookmarks,
        push: a.push,
        title: a.title.clone(),
        body,
    }))
}

impl TryFrom<&CycleArgs> for CycleParams {
    type Error = String;

    /// Resolve the workspace and the stage asked for.
    fn try_from(a: &CycleArgs) -> Result<Self, String> {
        match &a.command {
            CycleCommand::Plan(p) => plan_params(p),
            CycleCommand::Ladder(w) => Ok(CycleParams::Ladder(w.running()?.0)),
            CycleCommand::Status(w) => Ok(CycleParams::Status(w.running()?.0)),
            CycleCommand::Close(c) => close_params(c),
            CycleCommand::Land(l) => land_params(l),
            CycleCommand::Push(p) => {
                let (running, agent) = p.at.running()?;
                Ok(CycleParams::Push(PushCycle {
                    agent: publish_scope(&p.scope.scope, agent, "cycle push")?,
                    running,
                }))
            }
        }
    }
}

impl SubcommandRunner for CycleArgs {
    type Params = CycleParams;

    /// Delegate to the `TryFrom<&CycleArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        CycleParams::try_from(self)
    }

    /// Run the `cycle` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        cycle(ctx, params)
    }
}

/// Entry point: the stage asked for.
pub fn cycle(ctx: &mut Context, params: &CycleParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("cycle: enter {params:?}");
    match params {
        CycleParams::Open(o) => open(ctx, o),
        CycleParams::PlanCommit(c) => plan_commit(ctx, c),
        CycleParams::Ladder(r) => ladder(r),
        CycleParams::Close(c) => close(ctx, c, "cycle close"),
        CycleParams::Squash(c) => squash::squash(ctx, c),
        CycleParams::Land(l) => land(ctx, l),
        CycleParams::Push(p) => push(ctx, p),
        CycleParams::Status(r) => {
            for line in status(r)? {
                info!("{line}");
            }
            Ok(())
        }
    }
}

/// The opening's body when none is given: `[opening] body` in the
/// config, its `{title}` the cycle title, else a sentence naming the
/// cycle, then the pointer to its record, [`with_pointer`].
fn opening_body(repo: &std::path::Path, cycle: &str) -> Result<String, Box<dyn std::error::Error>> {
    let configured = crate::config_md::load(repo)?
        .and_then(|cfg| crate::toml_simple::toml_get(&cfg.map, "opening.body").cloned());
    let body = match configured {
        Some(body) => body,
        None => "Opens the cycle \"{title}\".".to_string(),
    };
    Ok(with_pointer(&body.replace("{title}", cycle), cycle))
}

/// A bookend's body with the pointer to the cycle's record, its own
/// paragraph, two lines, the second the cycle's heading exactly as
/// written in the record, so a search finds it at any commit.
fn with_pointer(body: &str, cycle: &str) -> String {
    format!(
        "{}\n\nSee details in {}:\n### {cycle}",
        body.trim_end(),
        crate::commit::RECORD
    )
}

/// The params of the commit a stage makes, `vc-x1 commit`'s: the pair
/// or the plain repo, the description, and the cycle's bookmark the
/// one advanced.
fn commit_params(
    c: &CommitRung,
    title: String,
    body: String,
    bookmark: String,
) -> crate::commit::CommitParams {
    let target = match &c.agent {
        Some((agent, agent_now)) => crate::commit::Target::Pair {
            pair: crate::pair::Pair {
                work: c.running.work.clone(),
                agent: agent.clone(),
            },
            agent_now: *agent_now,
        },
        None => crate::commit::Target::Por(c.running.work.clone()),
    };
    crate::commit::CommitParams {
        target,
        title: Some(title),
        body: Some(body),
        advance: crate::commit::Advance::Named(bookmark),
        push: c.push,
        no_url_trailer: false,
        dry_run: c.running.dry_run,
        rev: None,
        covers: None,
        allow_empty: false,
        same_version: c.same_version,
        land_follows: false,
    }
}

/// `rung commit`, `label` naming it in what it says:
/// commit the working copies as a rung of the cycle in
/// progress. The cycle's first commit is its opening, titled and
/// bodied by default, and any other is a work rung, whose title is a
/// rung heading not yet committed. The ladder is rewritten first, the
/// rung done, so the commit carries it, and the cycle's bookmark is
/// the one advanced. Every check is made before the ladder is written.
fn commit_rung(
    ctx: &mut Context,
    c: &CommitRung,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let r = &c.running;
    let work = r.work.as_path();
    let bookmark = cycle_bookmark(r, label)?;
    let commits = branch_titles(work, &bookmark)?;
    let path = work.join(crate::commit::RECORD);
    let todo =
        std::fs::read_to_string(&path).map_err(|e| format!("{label}: {}: {e}", path.display()))?;
    let cycle = record::cycle_title(&todo).map_err(|e| format!("{label}: {e}"))?;
    let rungs = record::rung_titles(&todo).map_err(|e| format!("{label}: {e}"))?;
    let (opening, closing) = (format!("{cycle} opening"), format!("{cycle} closing"));
    let first = commits.is_empty();
    let title = match (&c.title, first) {
        (None, true) => opening.clone(),
        (Some(t), true) if *t == opening => opening.clone(),
        (Some(t), true) => {
            return Err(format!(
                "{label}: the cycle has no commit yet, so this one is its opening, \
                 '{opening}', and not '{t}': leave --title out"
            )
            .into());
        }
        (None, false) => {
            return Err(format!(
                "{label}: the opening is committed, so this is a work rung, which gives \
                 --title, a heading under `#### Rungs`, and --body"
            )
            .into());
        }
        (Some(t), false) => t.clone(),
    };
    if title == closing {
        return Err(format!(
            "{label}: '{title}' is the cycle's closing, which `cycle close` makes"
        )
        .into());
    }
    if commits.contains(&title) {
        return Err(format!(
            "{label}: the rung '{title}' is committed already: `vc-x1 squash` continues it"
        )
        .into());
    }
    if !rungs.contains(&title) {
        return Err(format!(
            "{label}: '{title}' is no rung of the cycle, a `##### ` heading under `#### \
             Rungs` in {}: write its heading first",
            crate::commit::RECORD
        )
        .into());
    }
    let body = match (&c.body, first) {
        (Some(b), _) => b.clone(),
        (None, true) => opening_body(work, &cycle)?,
        (None, false) => {
            return Err(
                format!("{label}: a work rung gives --body, its problem and solution").into(),
            );
        }
    };
    crate::pair::refuse_conflicts(work, label, "work")?;
    if jj::is_empty(work, "@")? {
        return Err(format!(
            "{label}: nothing to commit, the work repo's @ is empty ('{}')",
            work.display()
        )
        .into());
    }
    crate::commit::check_version(work, "@", c.same_version, label)?;
    let mut with = commits.clone();
    with.push(title.clone());
    // A commit ends its rung, so the ladder it carries marks none
    // current: the next rung is started by `rung next`.
    let plan = record::plan_ladder(work, &with, None).map_err(|e| format!("{label}: {e}"))?;
    for finding in &plan.ladder.findings {
        warn!("{label}: {finding}");
    }
    info!(
        "{label}: {} '{title}' on '{bookmark}'",
        if first { "the opening" } else { "the rung" }
    );
    let params = commit_params(c, title, body, bookmark);
    let Some(edit) = plan.edit.as_ref().filter(|_| !r.dry_run) else {
        if plan.edit.is_some() {
            info!(
                "{label}: [dry-run] rewrite the ladder in {}, the rung done",
                record::LADDER_FILE
            );
        }
        return crate::commit::commit(ctx, &params);
    };
    // The ladder is written before the commit, which carries it, and
    // put back as it was when the commit is not made.
    let old = std::fs::read_to_string(&edit.path)?;
    std::fs::write(&edit.path, &edit.text)?;
    info!(
        "{label}: the ladder in {} is rewritten, the rung done",
        record::LADDER_FILE
    );
    crate::commit::commit(ctx, &params).inspect_err(|_| {
        if let Err(e) = std::fs::write(&edit.path, &old) {
            warn!(
                "{label}: the ladder in {} was not put back: {e}",
                edit.path.display()
            );
        }
    })
}

/// The revset of the cycle's branch: the commits `bookmark` reaches,
/// and those above it up to the work `@-`, a commit made by hand not
/// having advanced it, that no other local bookmark reaches.
fn branch_revset(bookmark: &str) -> String {
    let own = format!("bookmarks(exact:\"{bookmark}\")");
    format!("((::({own}) | ({own})::@-) ~ ::(bookmarks() ~ {own}))")
}

/// The titles of the commits on the cycle's branch in `repo`, oldest
/// first: the commits `bookmark` reaches, and those above it up to the
/// work `@-`, that no other local bookmark reaches, so no destination
/// is named or stored, less a working copy not yet described.
pub(crate) fn branch_titles(
    repo: &std::path::Path,
    bookmark: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut titles = Vec::new();
    for cid in jj::cids_of(repo, &branch_revset(bookmark))? {
        let desc = jj::desc_of(repo, &cid)?;
        let title = desc.lines().next().unwrap_or_default().trim_end();
        if !title.is_empty() {
            titles.push(title.to_string());
        }
    }
    Ok(titles)
}

/// The bookmark of the cycle in progress in `r`'s workspace: the one
/// named, else the slug of the title in `TODO.md`, refused when the
/// work repo has no such bookmark.
pub(crate) fn cycle_bookmark(
    r: &Running,
    label: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let name = bookmark_name(r, label)?;
    if !jj::local_bookmark_exists(&r.work, &name)? {
        return Err(format!(
            "{label}: the cycle's bookmark '{name}' is not in {}: name it, --bookmark <NAME>",
            r.work.display()
        )
        .into());
    }
    Ok(name)
}

/// The name of the cycle's bookmark, whether or not it is made: the
/// one named, else the slug of the title in `TODO.md`.
fn bookmark_name(r: &Running, label: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(match &r.bookmark {
        Some(name) => name.clone(),
        None => {
            let path = r.work.join(crate::commit::RECORD);
            let todo = std::fs::read_to_string(&path)
                .map_err(|e| format!("{label}: {}: {e}", path.display()))?;
            let title = record::cycle_title(&todo).map_err(|e| format!("{label}: {e}"))?;
            crate::check::anchors::anchor_slug(&title)
        }
    })
}

/// `cycle ladder`: make the ladder from the rung headings and the cycle's
/// commits, and write it, or with a dry run print it. A rung marked
/// `(current)` keeps its marker, and none is made, `rung next` being
/// what starts a rung. `TODO.md` is the ladder's source, so a cycle
/// whose bookmark is not made yet has its ladder too, no rung done,
/// refused when the ladder there has a rung done, which a missing
/// bookmark would lose.
fn ladder(r: &Running) -> Result<(), Box<dyn std::error::Error>> {
    let bookmark = bookmark_name(r, "cycle ladder")?;
    let commits = if jj::local_bookmark_exists(&r.work, &bookmark)? {
        branch_titles(&r.work, &bookmark)?
    } else {
        let path = r.work.join(record::LADDER_FILE);
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("cycle ladder: {}: {e}", path.display()))?;
        if record::has_done(&text) {
            return Err(format!(
                "cycle ladder: the cycle's bookmark '{bookmark}' is not in {}, and the ladder \
                 in {} has a rung done, which a ladder made without the cycle's commits would \
                 lose: name the bookmark, --bookmark <NAME>",
                r.work.display(),
                record::LADDER_FILE
            )
            .into());
        }
        info!("cycle ladder: no bookmark '{bookmark}' yet, so no rung is done");
        Vec::new()
    };
    let current = record::current_rung(&r.work).map_err(|e| format!("cycle ladder: {e}"))?;
    let plan = record::plan_ladder(&r.work, &commits, current.as_deref())
        .map_err(|e| format!("cycle ladder: {e}"))?;
    for line in &plan.ladder.lines {
        info!("{line}");
    }
    for finding in &plan.ladder.findings {
        warn!("cycle ladder: {finding}");
    }
    match (&plan.edit, r.dry_run) {
        (None, _) => info!(
            "cycle ladder: the ladder in {} is as the rungs make it",
            record::LADDER_FILE
        ),
        (Some(_), true) => info!(
            "cycle ladder: [dry-run] write the ladder into {}",
            record::LADDER_FILE
        ),
        (Some(edit), false) => {
            std::fs::write(&edit.path, &edit.text)
                .map_err(|e| format!("cycle ladder: {}: {e}", edit.path.display()))?;
            info!(
                "cycle ladder: the ladder is written into {}",
                record::LADDER_FILE
            );
        }
    }
    Ok(())
}

/// What is wrong with `title` as a cycle title, a line each: one that
/// is no Conventional Commit, `<type>: <desc>` or `<type>(<scope>):
/// <desc>`, and one over 50 characters, the bookends then longer still.
/// Empty when it is right.
pub(crate) fn title_problems(title: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let conventional = title.split_once(": ").is_some_and(|(kind, desc)| {
        let kind = kind.strip_suffix('!').unwrap_or(kind);
        let (kind, scope) = match kind.split_once('(') {
            Some((k, rest)) => (k, rest.strip_suffix(')')),
            None => (kind, Some("x")),
        };
        !kind.is_empty()
            && kind.chars().all(|c| c.is_ascii_lowercase())
            && scope.is_some_and(|s| !s.is_empty())
            && !desc.trim().is_empty()
    });
    if !conventional {
        problems.push(format!(
            "'{title}' is no Conventional Commit, `<type>: <desc>`, `feat: x` for one"
        ));
    }
    let n = title.chars().count();
    if n > 50 {
        problems.push(format!(
            "'{title}' is {n} characters, over the 50 a title keeps to, its bookends longer still"
        ));
    }
    problems
}

/// `cycle plan start`: plan the record, make the bookmark, then write
/// the record, so a record that cannot be written makes no bookmark,
/// and a name in use writes no record, and with `--ver` set the
/// opening's version last.
fn open(ctx: &mut Context, o: &OpenCycle) -> Result<(), Box<dyn std::error::Error>> {
    let label = "cycle plan start --ver";
    let dry_run = o.open.dry_run;
    if let Some(title) = &o.record {
        for problem in title_problems(title) {
            warn!("cycle plan: {problem}, and every commit of the cycle carries it");
        }
    }
    // The bump's config is checked with the plan's other refusals, so a
    // `--ver` that cannot run makes nothing.
    if let Some(ver) = &o.ver {
        rung::bump_words(&o.open.work, ver, label)?;
    }
    let plan = match &o.record {
        Some(title) => {
            record::plan_open(&o.open.work, title).map_err(|e| format!("cycle plan: {e}"))?
        }
        None => {
            info!("cycle plan: no record is written, the bookmark being made alone");
            record::OpenRecord::default()
        }
    };
    crate::open::open(ctx, &o.open)?;
    let tag = if dry_run {
        "cycle plan: [dry-run]"
    } else {
        "cycle plan:"
    };
    for line in &plan.did {
        info!("{tag} {line}");
    }
    if dry_run {
        if let Some(ver) = &o.ver {
            rung::bump(&o.open.work, ver, true, label)?;
        }
        return Ok(());
    }
    // The files as they were, `None` for one the plan makes, put back
    // when the bump fails after them.
    let before: Vec<(PathBuf, Option<String>)> = plan
        .edits
        .iter()
        .map(|e| (e.path.clone(), std::fs::read_to_string(&e.path).ok()))
        .collect();
    for edit in &plan.edits {
        std::fs::write(&edit.path, &edit.text)
            .map_err(|e| format!("cycle plan: {}: {e}", edit.path.display()))?;
    }
    if let Some(ver) = &o.ver
        && let Err(e) = rung::bump(&o.open.work, ver, false, label)
    {
        for (path, text) in &before {
            let undone = match text {
                Some(text) => std::fs::write(path, text),
                None => std::fs::remove_file(path),
            };
            if let Err(w) = undone {
                warn!("{label}: {} was not put back: {w}", path.display());
            }
        }
        if let Err(d) = ctx.session(&o.open.work)?.bookmark_delete(&o.open.bookmark) {
            warn!(
                "{label}: the bookmark '{}' was not deleted: {d}",
                o.open.bookmark
            );
        }
        return Err(format!("{e}, so the plan is undone, the record and the bookmark").into());
    }
    if o.hint && !plan.edits.is_empty() {
        info!(
            "cycle plan: the record is written and nothing is committed: plan the cycle in it, \
             then `cycle plan commit -s <SCOPE>` commits the plan, the opening"
        );
    }
    Ok(())
}

/// `cycle plan commit`: commit the plan, the cycle's opening, the
/// ladder made from the headings and rewritten into the commit by
/// [`commit_rung`]. Refused when the branch has a commit already, and
/// on a plan of the one heading, which has no opening.
fn plan_commit(ctx: &mut Context, c: &CommitRung) -> Result<(), Box<dyn std::error::Error>> {
    let label = "cycle plan commit";
    let r = &c.running;
    let bookmark = cycle_bookmark(r, label)?;
    let commits = branch_titles(&r.work, &bookmark)?;
    if let Some(first) = commits.first() {
        return Err(format!(
            "{label}: the plan is committed already, '{first}': `rung next` starts the next rung"
        )
        .into());
    }
    commit_rung(ctx, c, label)
}

/// A closing's body when none is given: `[closing] body` in the
/// config, its `{title}` the cycle title, else a sentence naming the
/// cycle, then the pointer to its record, [`with_pointer`].
fn closing_body(repo: &std::path::Path, cycle: &str) -> Result<String, Box<dyn std::error::Error>> {
    let configured = crate::config_md::load(repo)?
        .and_then(|cfg| crate::toml_simple::toml_get(&cfg.map, "closing.body").cloned());
    let body = match configured {
        Some(body) => body,
        None => "Closes the cycle \"{title}\".".to_string(),
    };
    Ok(with_pointer(&body.replace("{title}", cycle), cycle))
}

/// The closing commit's parents when they are not the work `@`'s own:
/// the destination first for a trapezoid, then the parents `@` has,
/// then each other head listed, none twice. `None` for a linear
/// closing of `@`'s line alone, which reparents nothing.
fn closing_parents(
    work: &std::path::Path,
    c: &CloseCycle,
) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
    let label = "cycle close";
    let own = jj::parent_cids_of(work, "@")?;
    let mut parents: Vec<String> = Vec::new();
    if let Some(dest) = &c.trapezoid {
        let cid = crate::land::resolve_tip(work, &format!("{label} {dest}"), dest)?;
        info!(
            "{label}: a trapezoid, '{dest}' at {} its first parent",
            short(&cid)
        );
        parents.push(cid);
    }
    for cid in &own {
        if !parents.contains(cid) {
            parents.push(cid.clone());
        }
    }
    for head in &c.heads {
        if jj::matches(work, &format!("({head}) & @"))? {
            continue;
        }
        let cid = crate::land::resolve_tip(work, label, head)?;
        if !parents.contains(&cid) {
            info!("{label}: '{head}' at {} is joined, a parent", short(&cid));
            parents.push(cid);
        }
    }
    Ok((parents != own).then_some(parents))
}

/// A commit id cut to the length the ids are shown at.
fn short(id: &str) -> String {
    id.chars()
        .take(crate::desc_helpers::DEFAULT_ID_LEN)
        .collect()
}

/// `cycle close`: close the cycle in progress from the work `@`, a
/// ladder's closing commit, a merge when a target or another head is
/// joined. With `single_step`, `close --squash`'s and a single-step's one
/// rung's, the cycle is one commit instead: made now from `@` with no
/// commit on the branch, or the opening with `@` folded in, under the
/// bare cycle title. The ladder is rewritten, every rung done, and the
/// record moved to `## Closed`, both in the commit. Every check is made
/// before a file is written.
fn close(ctx: &mut Context, c: &CloseCycle, label: &str) -> Result<(), Box<dyn std::error::Error>> {
    let r = &c.rung.running;
    let work = r.work.as_path();
    let bookmark = cycle_bookmark(r, label)?;
    let commits = branch_titles(work, &bookmark)?;
    let path = work.join(crate::commit::RECORD);
    let todo =
        std::fs::read_to_string(&path).map_err(|e| format!("{label}: {}: {e}", path.display()))?;
    let cycle = record::cycle_title(&todo).map_err(|e| format!("{label}: {e}"))?;
    if !jj::local_bookmarks_at(work, "@ | @-")?.contains(&bookmark) {
        return Err(format!(
            "{label}: the cycle's bookmark '{bookmark}' is not on the work @ or @-, and a \
             closing is made from the cycle's tip: `jj new {bookmark}` moves @ onto it"
        )
        .into());
    }
    // A rung started and not committed holds the closing back, but for
    // the closing itself, and for a single-step cycle's opening or its
    // one rung, which its one commit is.
    if let Some(current) = record::current_rung(work).map_err(|e| format!("{label}: {e}"))?
        && !commits.contains(&current)
        && current != format!("{cycle} closing")
        && !(c.single_step && current == format!("{cycle} opening"))
    {
        return Err(format!(
            "{label}: the rung '{current}' is started and not committed: `rung commit` \
             commits it, or `rung stop` leaves it"
        )
        .into());
    }
    crate::pair::refuse_conflicts(work, label, "work")?;
    if let Some(l) = &c.land {
        // The Land's own checks, before the closing is made: a
        // trapezoid onto the destination joins one that moved.
        let joined = c.trapezoid.as_deref() == Some(l.dest.as_str());
        land_checks(work, &l.dest, (!joined).then_some("@-"), l.agent.is_none())?;
    }
    // The shape is named, and the branch's commit count checks it: one
    // commit fits either, the opening, and the name decides.
    let single = c.single_step;
    match (single, commits.len()) {
        (true, n) if n >= 2 => {
            return Err(format!(
                "{label}: the branch has {n} commits, a ladder, and the one commit takes its \
                 rungs squashed into the opening first, as `cycle close --squash` does"
            )
            .into());
        }
        (false, 0) => {
            return Err(format!(
                "{label}: the branch has no commit, so there is no opening for a closing to \
                 follow: `rung commit` makes the opening first, or `cycle close --squash` commits the \
                 work @ as the cycle's one commit"
            )
            .into());
        }
        _ => {}
    }
    let closing = format!("{cycle} closing");
    let title = match (&c.rung.title, single, c.dead_end) {
        (Some(t), ..) => t.clone(),
        (None, true, _) => cycle.clone(),
        (None, false, true) => format!("{cycle} dead-end closing"),
        (None, false, false) => closing.clone(),
    };
    let body = match (&c.rung.body, single && !c.default_body) {
        (Some(b), _) => b.clone(),
        (None, true) => {
            return Err(format!(
                "{label}: the cycle's one commit holds the work, so it gives --body, its \
                 problem and solution, or --default-body for the pointer at the record"
            )
            .into());
        }
        (None, false) => closing_body(work, &cycle)?,
    };
    let parents = match single {
        true if c.trapezoid.is_some() || !c.heads.is_empty() => {
            return Err(format!(
                "{label}: the cycle's one commit joins nothing, so it takes no head and no \
                 TARGET: `cycle close <TARGET>` makes a closing commit that can"
            )
            .into());
        }
        true => None,
        false => closing_parents(work, c)?,
    };
    // The ladder as the closing leaves it: one unlinked rung for a
    // single-step cycle, else every rung done, the closing among them.
    let done = if single {
        vec![cycle.clone()]
    } else {
        let mut done = commits.clone();
        done.push(closing);
        done
    };
    crate::commit::check_version(work, "@", c.rung.same_version, label)?;
    let ladder = record::plan_ladder(work, &done, None).map_err(|e| format!("{label}: {e}"))?;
    let mut moved = crate::commit::RecordMove::plan(work)?;
    // A single-step cycle's one commit has no rungs, so the block goes to
    // `## Closed` without its `#### Rungs`, which the ladder read above.
    if single && let Some(m) = moved.as_mut() {
        m.new_record = record::without_closed_rungs(&m.new_record);
    }
    let bookmark_of_closing = bookmark.clone();
    // Said once every check has passed, so it is never about a closing
    // that a refusal then stops.
    let unwritten = record::unwritten_items(&todo);
    if !unwritten.is_empty() {
        warn!(
            "{label}: the record's {} still read `_To write._`, and the closing moves it to \
             `## Closed` as it is",
            unwritten.join(", ")
        );
    }
    let fold = single && commits.len() == 1;
    info!(
        "{label}: {} on '{bookmark}', '{title}'",
        match (single, fold) {
            (true, true) => "a single-step cycle, its closing squashed into its one commit",
            (true, false) => "a single-step cycle, its one commit made now from the work @",
            (false, _) => "a ladder, its closing commit made from the work @",
        }
    );
    let pair = c.rung.agent.as_ref().map(|(agent, _)| crate::pair::Pair {
        work: r.work.clone(),
        agent: agent.clone(),
    });
    if r.dry_run {
        if ladder.edit.is_some() {
            info!(
                "{label}: [dry-run] rewrite the ladder in {}, every rung done",
                record::LADDER_FILE
            );
        }
        if let Some(m) = &moved {
            m.write(true)?;
        }
        if let Some(parents) = &parents {
            info!(
                "{label}: [dry-run] the closing's parents are {}",
                parents
                    .iter()
                    .map(|p| short(p))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        if fold {
            crate::commit::finalize(ctx, work, pair.as_ref(), &title, &body, true)?;
        } else {
            // A dry run writes no record, so the work @ may hold nothing.
            let mut params = commit_params(&c.rung, title, body, bookmark);
            params.allow_empty = true;
            params.land_follows = c.land.is_some();
            crate::commit::commit(ctx, &params)?;
        }
        if let Some(l) = &c.land {
            info!(
                "cycle land: [dry-run] land the closing on '{}'{}, and delete the cycle's \
                 bookmark",
                l.dest,
                if l.push { " and publish it" } else { "" }
            );
        }
        return Ok(());
    }
    // The files as they were, put back when the closing is not made.
    let mut written: Vec<(PathBuf, String)> = Vec::new();
    for file in [crate::commit::RECORD, record::LADDER_FILE] {
        let path = work.join(file);
        if path.is_file() {
            written.push((path.clone(), std::fs::read_to_string(&path)?));
        }
    }
    let snapshot = crate::pair::Snapshot::take(work)?;
    let made = (|| -> Result<(), Box<dyn std::error::Error>> {
        if let Some(edit) = &ladder.edit {
            std::fs::write(&edit.path, &edit.text)?;
            info!(
                "{label}: the ladder in {} is rewritten, every rung done",
                record::LADDER_FILE
            );
        }
        if let Some(m) = &moved {
            m.write(false)?;
        }
        if fold {
            return crate::commit::finalize(ctx, work, pair.as_ref(), &title, &body, false);
        }
        if let Some(parents) = &parents {
            crate::merge::set_parents(ctx, work, "@", parents, label)?;
        }
        let mut params = commit_params(&c.rung, title, body, bookmark);
        params.land_follows = c.land.is_some();
        crate::commit::commit(ctx, &params)
    })();
    if let Err(e) = &made {
        snapshot.restore(label, e.as_ref());
        for (path, text) in &written {
            if let Err(w) = std::fs::write(path, text) {
                warn!("{label}: {} was not put back: {w}", path.display());
            }
        }
    }
    made?;
    match &c.land {
        Some(l) => land_closed(ctx, l, Some(bookmark_of_closing)),
        None => Ok(()),
    }
}

/// The checks a Land makes of its destination, made before a closing
/// that a Land follows: `dest` is a local bookmark where its `origin`
/// is, and `tip`, when given, descends from it, each refusal naming
/// the ways on.
fn land_checks(
    work: &std::path::Path,
    dest: &str,
    tip: Option<&str>,
    plain: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !jj::local_bookmark_exists(work, dest)? {
        return Err(
            format!("cycle land: '{dest}' is not a local bookmark, and a Land moves one").into(),
        );
    }
    let at = jj::cid_of(work, dest)?;
    let remote = jj::remote_bookmark_targets(work, "origin")?
        .get(dest)
        .cloned();
    if remote.as_deref() != Some(at.as_str()) {
        let refusal =
            crate::land::not_at_origin(work, "cycle land", dest, &at, remote.as_deref(), plain)?;
        return Err(refusal.into());
    }
    if let Some(tip) = tip
        && !jj::matches(work, &format!("{at} & ::({tip})"))?
    {
        return Err(moved(dest, &at).into());
    }
    Ok(())
}

/// The refusal of a destination that moved, naming the ways on.
fn moved(dest: &str, at: &str) -> String {
    format!(
        "cycle land: the cycle does not descend from '{dest}' at {}, which moved after the \
         cycle forked, and a Land only fast-forwards. The ways on:\n\
         \x20 - a merge at the Land, `cycle land {dest} --merge`, a commit no rung validated\n\
         \x20 - the closing made again as a trapezoid, `cycle close {dest}`, after `jj \
         abandon` of the one made\n\
         \x20 - a rebase, `jj rebase -b <cycle's bookmark> -d {dest}`, every rung then validated \
         again\n\
         \x20 - or your own",
        short(at)
    )
}

/// `cycle land`: land the closed cycle on the destination.
fn land(ctx: &mut Context, l: &LandCycle) -> Result<(), Box<dyn std::error::Error>> {
    land_closed(ctx, l, None)
}

/// Land the cycle whose last commit is made, its bookmark the one
/// given, else `--bookmark`'s, else the slug of the title `## Closed`
/// holds, else the one local bookmark besides the destination. A cycle
/// still in progress is refused, a destination that moved is refused
/// naming the ways on, and `--merge` joins it by a merge commit first.
/// The Land itself is [`crate::land::land`].
fn land_closed(
    ctx: &mut Context,
    l: &LandCycle,
    closed: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let work = l.running.work.as_path();
    let path = work.join(crate::commit::RECORD);
    let todo = match path.is_file() {
        true => Some(std::fs::read_to_string(&path)?),
        false => None,
    };
    if let Some(running) = todo.as_deref().and_then(|t| record::cycle_title(t).ok()) {
        return Err(format!(
            "cycle land: the cycle '{running}' is in progress in {}, and a Land is of a cycle \
             closed: `cycle close` first, or `cycle close --land {}`",
            crate::commit::RECORD,
            l.dest
        )
        .into());
    }
    let cycle = todo.as_deref().and_then(record::closed_title);
    let source = match closed.or_else(|| l.running.bookmark.clone()) {
        Some(named) => named,
        None => {
            let slug = cycle.as_deref().map(crate::check::anchors::anchor_slug);
            match slug {
                Some(slug) if jj::local_bookmark_exists(work, &slug)? => slug,
                _ => crate::land::default_source(work, &l.dest)?,
            }
        }
    };
    land_checks(work, &l.dest, None, l.agent.is_none())?;
    let dest_at = jj::cid_of(work, &l.dest)?;
    let descends = jj::matches(work, &format!("{dest_at} & ::({source})"))?;
    match (descends, l.merge) {
        (true, true) => info!(
            "cycle land: '{}' has not moved, so the Land is a fast-forward and no merge is made",
            l.dest
        ),
        (true, false) => {}
        (false, false) => return Err(moved(&l.dest, &dest_at).into()),
        (false, true) => land_merge(ctx, l, &source, &dest_at, cycle.as_deref())?,
    }
    let scope = match &l.agent {
        Some(_) => RepoScope::Sides(crate::options_flags::scope::Scope(vec![
            crate::options_flags::scope::Side::Work,
            crate::options_flags::scope::Side::Bot,
        ])),
        None => RepoScope::Por,
    };
    crate::land::land(
        ctx,
        &crate::land::LandParams {
            repo: l.running.work.clone(),
            scope,
            source: Some(source),
            onto: l.dest.clone(),
            skip: l.skip,
            delete_intermediate: l.delete_intermediate,
            push: l.push,
            dry_run: l.running.dry_run,
        },
    )
}

/// `--merge`: join a destination that moved, the merge of it and the
/// cycle's last commit committed from an empty work `@` on that commit,
/// the cycle's bookmark advanced to it. A conflicted merge is undone
/// and refused, the resolving being by hand.
fn land_merge(
    ctx: &mut Context,
    l: &LandCycle,
    source: &str,
    dest_at: &str,
    cycle: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let work = l.running.work.as_path();
    let label = "cycle land --merge";
    let tip = crate::land::resolve_tip(work, label, source)?;
    if !jj::matches(work, &format!("{tip} & @-"))? || !jj::is_empty(work, "@")? {
        return Err(format!(
            "{label}: the merge is made from an empty work @ on the cycle's last commit, \
             '{source}': `jj new {source}` puts it there"
        )
        .into());
    }
    let named = cycle.map_or_else(|| format!("'{source}'"), |c| format!("the cycle \"{c}\""));
    let title = match (&l.title, cycle) {
        (Some(t), _) => t.clone(),
        (None, Some(c)) => format!("{c} merge"),
        (None, None) => format!("chore: merge {source} into {}", l.dest),
    };
    let body = match &l.body {
        Some(b) => b.clone(),
        None => format!(
            "Merges {named} into '{}', which moved after the cycle forked. See details in {}.",
            l.dest,
            crate::commit::RECORD
        ),
    };
    info!(
        "{label}: '{}' moved, so '{title}' merges it at {} and '{source}' at {}",
        l.dest,
        short(dest_at),
        short(&tip)
    );
    if l.running.dry_run {
        return Ok(());
    }
    let rung = CommitRung {
        running: Running {
            work: l.running.work.clone(),
            bookmark: Some(source.to_string()),
            dry_run: false,
        },
        agent: l.agent.clone().map(|a| (a, true)),
        title: None,
        body: None,
        push: false,
        // A merge at the Land is no rung, and bumps nothing.
        same_version: true,
    };
    let snapshot = crate::pair::Snapshot::take(work)?;
    let made = (|| -> Result<(), Box<dyn std::error::Error>> {
        crate::merge::set_parents(ctx, work, "@", &[dest_at.to_string(), tip.clone()], label)?;
        if jj::matches(work, "@ & conflicts()")? {
            return Err(format!(
                "{label}: the merge of '{}' and '{source}' is conflicted, so it is not made: \
                 `jj new {} {source}` makes it for you to resolve, then `cycle land {}`",
                l.dest, l.dest, l.dest
            )
            .into());
        }
        let mut params = commit_params(&rung, title, body, source.to_string());
        params.allow_empty = true;
        crate::commit::commit(ctx, &params)
    })();
    if let Err(e) = &made {
        snapshot.restore(label, e.as_ref());
    }
    made
}

/// The bookmarks a closed cycle landed on, named and quoted: those at
/// or after its landmark, the commit titled with the bare cycle title
/// or its closing's. `None` when no bookmark holds it.
fn landed_on(
    work: &std::path::Path,
    cycle: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let on = landed_bookmarks(work, cycle)?;
    Ok((!on.is_empty()).then(|| {
        on.iter()
            .map(|b| format!("'{b}'"))
            .collect::<Vec<_>>()
            .join(", ")
    }))
}

/// The local bookmarks at or after a closed cycle's landmark, the commit
/// titled with the bare cycle title or its closing's, the newest such
/// commit a bookmark holds. Empty when none does.
fn landed_bookmarks(
    work: &std::path::Path,
    cycle: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let escaped = cycle.replace('\\', "\\\\").replace('"', "\\\"");
    let candidates = jj::cids_of(
        work,
        &format!("::bookmarks() & description(substring:\"{escaped}\")"),
    )?;
    let titles = [
        cycle.to_string(),
        format!("{cycle} closing"),
        format!("{cycle} dead-end closing"),
    ];
    for cid in candidates.iter().rev() {
        let desc = jj::desc_of(work, cid)?;
        if !titles
            .iter()
            .any(|t| desc.lines().next() == Some(t.as_str()))
        {
            continue;
        }
        let on = jj::local_bookmarks_at(work, &format!("bookmarks() & {cid}::"))?;
        if !on.is_empty() {
            return Ok(on);
        }
    }
    Ok(Vec::new())
}

/// `cycle push`: publish what the cycle's state calls for. A cycle
/// in progress, or closed and not landed, is a draft, and its bookmark
/// is pushed. A cycle landed here publishes the target its landmark is
/// on, the one such bookmark not where its `origin` is, and its own
/// bookmark is deleted on `origin` when it is there. Nothing to publish
/// is said, and no error.
fn push(ctx: &mut Context, p: &PushCycle) -> Result<(), Box<dyn std::error::Error>> {
    let label = "cycle push";
    let r = &p.running;
    let work = r.work.as_path();
    let path = work.join(crate::commit::RECORD);
    let todo = std::fs::read_to_string(&path).map_err(|e| {
        format!(
            "{label}: {}: {e}, and the cycle is read from its record: `vc-x1 push` publishes \
             a bookmark named",
            path.display()
        )
    })?;
    let scope = match &p.agent {
        Some(_) => RepoScope::Sides(crate::options_flags::scope::Scope(vec![
            crate::options_flags::scope::Side::Work,
            crate::options_flags::scope::Side::Bot,
        ])),
        None => RepoScope::Por,
    };
    let publish = |ctx: &mut Context, name: &str| -> Result<(), Box<dyn std::error::Error>> {
        // At the work `@-` the pair goes out with it, and elsewhere the
        // bookmark alone, where it is.
        let at_head = jj::local_bookmarks_at(work, "@-")?
            .iter()
            .any(|b| b == name);
        crate::push::push(
            ctx,
            &crate::push::PushParams {
                revision: (!at_head).then(|| name.to_string()),
                bookmark: at_head.then(|| name.to_string()),
                repo: Some(r.work.clone()),
                scope: scope.clone(),
                step: false,
                dry_run: r.dry_run,
                yes: true,
                forced: crate::common::ForcedPush::Refuse,
            },
        )
    };
    let remote = jj::remote_bookmark_targets(work, "origin")?;
    // A draft: a cycle in progress, or one closed whose bookmark stands.
    let draft = match record::cycle_title(&todo) {
        Ok(_) => Some(cycle_bookmark(r, "cycle push")?),
        Err(_) => record::closed_title(&todo)
            .map(|c| {
                r.bookmark
                    .clone()
                    .unwrap_or_else(|| crate::check::anchors::anchor_slug(&c))
            })
            .filter(|b| jj::local_bookmark_exists(work, b).unwrap_or(false)),
    };
    if let Some(bookmark) = draft {
        info!("{label}: the cycle is a draft, so its bookmark '{bookmark}' is pushed");
        return publish(ctx, &bookmark);
    }
    let Some(cycle) = record::closed_title(&todo) else {
        info!(
            "{label}: no cycle in progress or closed in {}, so nothing is published",
            path.display()
        );
        return Ok(());
    };
    let landed = landed_bookmarks(work, &cycle)?;
    if landed.is_empty() {
        return Err(format!(
            "{label}: the cycle '{cycle}' is closed and its bookmark gone, and no bookmark holds \
             its landmark, so there is no target to publish"
        )
        .into());
    }
    let ahead: Vec<&String> = landed
        .iter()
        .filter(|b| {
            let local = jj::cid_of(work, b).ok();
            remote.get(b.as_str()) != local.as_ref()
        })
        .collect();
    let slug = crate::check::anchors::anchor_slug(&cycle);
    let stale = remote.contains_key(&slug);
    match ahead.as_slice() {
        [] if !stale => {
            info!(
                "{label}: '{cycle}' is landed on {} and published, so nothing is published",
                landed
                    .iter()
                    .map(|b| format!("'{b}'"))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            return Ok(());
        }
        [] => {}
        [target] => {
            info!("{label}: '{cycle}' is landed on '{target}' here, so '{target}' is published");
            publish(ctx, target)?;
        }
        several => {
            return Err(format!(
                "{label}: '{cycle}' is landed on several bookmarks not yet published, {}, so \
                 which is the target is not known: `vc-x1 push -s b -r <target>` publishes one",
                several
                    .iter()
                    .map(|b| format!("'{b}'"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .into());
        }
    }
    if stale {
        info!("{label}: the cycle's bookmark '{slug}' is still on origin, and is deleted there");
        crate::delete_bookmark::delete_bookmark(
            ctx,
            work,
            &slug,
            Some(crate::delete_bookmark::Keep::Local),
            r.dry_run,
        )?;
    }
    Ok(())
}

/// `cycle status`: where the cycle stands, a line each, read from the
/// record, the ladder, and the branch, nothing changed. What another
/// stage would refuse is said here and refuses nothing.
fn status(r: &Running) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let work = r.work.as_path();
    let mut out = Vec::new();
    let path = work.join(crate::commit::RECORD);
    if !path.is_file() {
        out.push(format!(
            "no {} in {}, so there is no cycle-record to read",
            crate::commit::RECORD,
            work.display()
        ));
        return Ok(out);
    }
    let todo = std::fs::read_to_string(&path)?;
    let Ok(cycle) = record::cycle_title(&todo) else {
        out.push("cycle: none in progress".to_string());
        let closed = record::closed_title(&todo);
        let slug = closed.as_deref().map(crate::check::anchors::anchor_slug);
        match (closed, slug) {
            (Some(c), Some(slug)) if jj::local_bookmark_exists(work, &slug)? => {
                out.push(format!(
                    "closed: '{c}', its bookmark '{slug}' not yet landed"
                ));
                out.push("next: `cycle land <TARGET> -s <SCOPE>` lands it".to_string());
            }
            (Some(c), _) => {
                out.push(match landed_on(work, &c)? {
                    Some(on) => format!("closed: '{c}', landed on {on}"),
                    None => {
                        format!("closed: '{c}', its bookmark gone and its commits on no bookmark")
                    }
                });
                let remote = jj::remote_bookmark_targets(work, "origin")?;
                let unpublished: Vec<String> = landed_bookmarks(work, &c)?
                    .into_iter()
                    .filter(|b| remote.get(b.as_str()) != jj::cid_of(work, b).ok().as_ref())
                    .collect();
                out.push(match unpublished.as_slice() {
                    [] => "next: `cycle plan start --title \"<title>\"` opens one".to_string(),
                    names => format!(
                        "next: `cycle push -s <SCOPE>` publishes {}, landed here and not on \
                         origin",
                        names
                            .iter()
                            .map(|b| format!("'{b}'"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                });
            }
            _ => out.push("next: `cycle plan start --title \"<title>\"` opens one".to_string()),
        }
        return Ok(out);
    };
    out.push(format!("cycle: {cycle}"));
    let bookmark = match cycle_bookmark(r, "cycle status") {
        Ok(b) => b,
        Err(e) => {
            out.push(format!("bookmark: not found: {e}"));
            return Ok(out);
        }
    };
    let place = match jj::local_bookmarks_at(work, "@")?.contains(&bookmark) {
        true => "on the work @, nothing committed",
        false if jj::local_bookmarks_at(work, "@-")?.contains(&bookmark) => "on the work @-",
        false => "not on the work @ or @-",
    };
    out.push(format!("bookmark: {bookmark}, {place}"));
    let commits = branch_titles(work, &bookmark)?;
    let branch = branch_revset(&bookmark);
    let local = jj::cids_of(work, &format!("{branch} ~ ::remote_bookmarks()"))?
        .iter()
        .filter(|cid| jj::desc_of(work, cid).is_ok_and(|d| !d.trim().is_empty()))
        .count();
    out.push(format!(
        "commits: {} on the branch, {local} not pushed",
        commits.len()
    ));
    out.push(format!(
        "work @: {}",
        if jj::is_empty(work, "@")? {
            "empty"
        } else {
            "holds changes"
        }
    ));
    let current = record::current_rung(work).ok().flatten();
    let rungs = record::rung_titles(&todo).and_then(|rungs| {
        let ladder = record::ladder_of(&cycle, &rungs, &commits, current.as_deref())?;
        Ok((rungs, ladder))
    });
    let (rungs, ladder) = match rungs {
        Ok(made) => made,
        Err(e) => {
            out.push(format!("ladder: not made: {e}"));
            return Ok(out);
        }
    };
    out.push("ladder:".to_string());
    // The titles alone: a link is for `zTOC.md`, where an editor follows
    // it, and in a terminal only doubles the line.
    out.extend(
        ladder
            .lines
            .iter()
            .map(|l| format!("  {}", record::unlinked(l))),
    );
    out.extend(ladder.findings.iter().map(|f| format!("note: {f}")));
    let file = work.join(record::LADDER_FILE);
    if file.is_file()
        && record::written_ladder(&std::fs::read_to_string(&file)?, &ladder.lines)
            .is_ok_and(|new| std::fs::read_to_string(&file).is_ok_and(|old| old != new))
    {
        out.push(format!(
            "note: the ladder in {} differs, and `cycle ladder` writes this one",
            record::LADDER_FILE
        ));
    }
    let closing = format!("{cycle} closing");
    let started = current.filter(|c| rungs.contains(c) && !commits.contains(c));
    let next = rungs.iter().find(|r| !commits.contains(r));
    out.push(match (&started, commits.len(), next) {
        (Some(c), ..) if *c == closing => format!(
            "next: '{c}' is started: `cycle close <TARGET> -s <SCOPE>` makes it a trapezoid, \
             `--linear` a plain closing, or `cycle close --squash -s <SCOPE> --body ...` makes \
             the cycle one commit"
        ),
        (Some(c), 0, _) => format!(
            "next: '{c}' is started: `cycle plan commit -s <SCOPE>` commits the plan, the opening"
        ),
        (Some(c), ..) => {
            format!("next: '{c}' is started: `rung commit -s <SCOPE> --body ...` commits it")
        }
        (None, 0, _) => "next: `cycle plan commit -s <SCOPE>` commits the plan, the opening, or \
                         `cycle close --squash -s <SCOPE> --body ...` commits the work @ as the \
                         cycle's one commit"
            .to_string(),
        (None, _, Some(c)) if *c == closing => format!(
            "next: `rung next` starts '{c}', or `cycle close --squash -s <SCOPE> --body ...` \
             makes the cycle one commit"
        ),
        (None, _, Some(c)) => format!("next: `rung next` starts '{c}'"),
        (None, _, None) => "next: every rung is committed".to_string(),
    });
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::record::tests::{TODO, ZTOC};
    use super::*;
    use crate::test_helpers::{Fixture, description, jj_ok, test_ctx};
    use crate::{Cli, Commands};
    use clap::Parser;
    use std::fs;
    use std::path::Path;

    fn parse(args: &[&str]) -> Result<CycleArgs, String> {
        match Cli::try_parse_from(args) {
            Ok(Cli {
                command: Some(Commands::Cycle(a)),
                ..
            }) => Ok(a),
            Ok(_) => panic!("expected Cycle"),
            Err(e) => Err(e.to_string()),
        }
    }

    /// The params of `vc-x1 cycle <args>` in `repo`, `-s b` added for a
    /// stage that requires a scope when the test names none.
    fn params(repo: &Path, args: &[&str]) -> Result<CycleParams, String> {
        let mut all = vec!["vc-x1", "cycle"];
        all.extend_from_slice(args);
        let scoped = ["close", "squash", "land"];
        if args.first().is_some_and(|a| scoped.contains(a)) && !args.contains(&"-s") {
            all.extend_from_slice(&["-s", "b"]);
        }
        if !args.is_empty() {
            all.extend_from_slice(&["-R", repo.to_str().expect("a utf-8 path")]);
        }
        CycleParams::try_from(&parse(&all)?)
    }

    fn run(repo: &Path, args: &[&str]) -> Result<(), String> {
        cycle(&mut test_ctx(), &params(repo, args)?).map_err(|e| e.to_string())
    }

    /// Commit `@` as a rung by the code `rung commit` runs, `--title`
    /// given as `rung commit` gives the current rung's, so a test names
    /// the rung without starting it, and `-s b` added when no `-s` is.
    fn commit(repo: &Path, args: &[&str]) -> Result<(), String> {
        let mut title = None;
        let mut all = vec!["vc-x1", "rung", "commit"];
        let mut it = args.iter();
        while let Some(a) = it.next() {
            match *a {
                "--title" => title = it.next().map(|t| t.to_string()),
                a => all.push(a),
            }
        }
        if !args.contains(&"-s") {
            all.extend_from_slice(&["-s", "b"]);
        }
        all.extend_from_slice(&["-R", repo.to_str().expect("a utf-8 path")]);
        let a = match Cli::try_parse_from(all) {
            Ok(Cli {
                command: Some(Commands::Rung(a)),
                ..
            }) => a,
            Ok(_) => panic!("expected Rung"),
            Err(e) => return Err(e.to_string()),
        };
        let rung::RungParams::Commit(mut c) = rung::RungParams::try_from(&a)? else {
            panic!("expected rung commit");
        };
        c.rung.title = title;
        commit_rung(&mut test_ctx(), &c.rung, "rung commit").map_err(|e| e.to_string())
    }

    /// Run `vc-x1 rung <args>` in `repo`.
    fn rung(repo: &Path, args: &[&str]) -> Result<(), String> {
        let mut all = vec!["vc-x1", "rung"];
        all.extend_from_slice(args);
        all.extend_from_slice(&["-R", repo.to_str().expect("a utf-8 path")]);
        let a = match Cli::try_parse_from(all) {
            Ok(Cli {
                command: Some(Commands::Rung(a)),
                ..
            }) => a,
            Ok(_) => panic!("expected Rung"),
            Err(e) => return Err(e.to_string()),
        };
        let p = rung::RungParams::try_from(&a)?;
        <rung::RungArgs as SubcommandRunner>::run(&mut test_ctx(), &p).map_err(|e| e.to_string())
    }

    /// The ladder lines of the working copy's `zTOC.md`, the bullets
    /// under `## Ladder`.
    fn ladder_lines(repo: &Path) -> Vec<String> {
        fs::read_to_string(repo.join("zTOC.md"))
            .expect("read zTOC.md")
            .lines()
            .skip_while(|l| *l != "## Ladder")
            .skip(1)
            .take_while(|l| !l.starts_with("## "))
            .filter(|l| l.starts_with("- "))
            .map(String::from)
            .collect()
    }

    /// The local bookmarks on the work `@`.
    fn bookmarks_at(repo: &Path) -> String {
        jj_ok(
            repo,
            &["log", "-r", "@", "--no-graph", "-T", "local_bookmarks"],
        )
    }

    fn write_record(fx: &Fixture) {
        fs::write(fx.work.join("TODO.md"), TODO).expect("write");
        fs::write(fx.work.join("zTOC.md"), ZTOC).expect("write");
    }

    /// A stage is required, each a subcommand, the plan's options are
    /// its own, and it needs a name, a bare title.
    #[test]
    fn a_stage_is_required() {
        let err = parse(&["vc-x1", "cycle"]).unwrap_err();
        assert!(err.contains("plan") && err.contains("close"), "{err}");
        let err = parse(&["vc-x1", "cycle", "--no-record"]).unwrap_err();
        assert!(err.contains("--no-record"), "{err}");
        let err = parse(&["vc-x1", "cycle", "--open"]).unwrap_err();
        assert!(err.contains("--open"), "the stage flags are gone: {err}");
        let fx = Fixture::new("cycle-stage");
        let err = params(&fx.work, &["plan", "start"]).unwrap_err();
        assert!(
            err.contains("--title") && err.contains("--bookmark"),
            "{err}"
        );
        let err = params(&fx.work, &["plan", "start", "--title", "feat: x opening"]).unwrap_err();
        assert!(
            err.contains("opening") && err.contains("cycle plan"),
            "{err}"
        );
    }

    /// `cycle plan` makes the bookmark on the work `@` and writes the
    /// record into the working copy, nothing committed, and a dry run
    /// does neither.
    #[test]
    fn open_makes_the_bookmark_and_the_record() {
        let fx = Fixture::new("cycle-open");
        write_record(&fx);
        run(
            &fx.work,
            &["plan", "start", "--title", "feat: a cycle", "--dry-run"],
        )
        .expect("a dry run");
        assert_eq!(bookmarks_at(&fx.work), "", "a dry run makes none");
        assert_eq!(fs::read_to_string(fx.work.join("TODO.md")).unwrap(), TODO);

        run(&fx.work, &["plan", "start", "--title", "feat: a cycle"]).expect("opened");
        assert_eq!(bookmarks_at(&fx.work), "feat-a-cycle");
        let todo = fs::read_to_string(fx.work.join("TODO.md")).unwrap();
        assert!(todo.contains("## In Progress\n\nThe record's home.\n\n### feat: a cycle\n"));
        assert!(!todo.contains("chore: the last cycle"), "{todo}");
        let ztoc = fs::read_to_string(fx.work.join("zTOC.md")).unwrap();
        assert!(
            ztoc.contains(
                "- [feat: a cycle opening](TODO.md#feat-a-cycle-opening)\n\
                 - [feat: a cycle closing](TODO.md#feat-a-cycle-closing)\n"
            ),
            "{ztoc}"
        );
        assert_eq!(
            jj_ok(&fx.work, &["log", "-r", "@", "--no-graph", "-T", "empty"]),
            "false",
            "the record is in the working copy"
        );
    }

    /// A cycle in progress is refused before the bookmark is made, and
    /// a name in use before the record is written.
    #[test]
    fn open_refuses_before_anything_is_made() {
        let fx = Fixture::new("cycle-open-refused");
        write_record(&fx);
        run(&fx.work, &["plan", "start", "--title", "feat: a cycle"]).expect("opened");
        let err = run(&fx.work, &["plan", "start", "--title", "fix: another"]).unwrap_err();
        assert!(err.contains("in progress"), "{err}");
        assert_eq!(bookmarks_at(&fx.work), "feat-a-cycle", "no second bookmark");

        let fx = Fixture::new("cycle-open-in-use");
        write_record(&fx);
        jj_ok(
            &fx.work,
            &["bookmark", "create", "feat-a-cycle", "-r", "main"],
        );
        let err = run(&fx.work, &["plan", "start", "--title", "feat: a cycle"]).unwrap_err();
        assert!(err.contains("exists"), "{err}");
        assert_eq!(fs::read_to_string(fx.work.join("TODO.md")).unwrap(), TODO);
    }

    /// A fresh workspace's record is `init`'s skeletons, which a plan
    /// writes the cycle into, and a workspace without them, made before
    /// `init` wrote them, gets them from the plan, a dry run writing
    /// none. `--no-record` and a bookmark with no title get the bookmark
    /// alone.
    #[test]
    fn open_makes_the_bookmark_alone() {
        let fx = Fixture::new("cycle-open-alone");
        let skeleton = |name: &str| fs::read_to_string(fx.work.join(name)).unwrap();
        assert_eq!(skeleton("TODO.md"), record::TODO_SKELETON, "init's");
        assert_eq!(skeleton("zTOC.md"), record::LADDER_SKELETON, "init's");
        run(
            &fx.work,
            &["plan", "start", "--title", "feat: a cycle", "--no-record"],
        )
        .expect("alone");
        assert_eq!(bookmarks_at(&fx.work), "feat-a-cycle");
        assert_eq!(
            skeleton("TODO.md"),
            record::TODO_SKELETON,
            "--no-record writes none"
        );

        let fx = Fixture::new("cycle-open-skeleton");
        run(&fx.work, &["plan", "start", "--title", "feat: a cycle"]).expect("planned");
        let todo = fs::read_to_string(fx.work.join("TODO.md")).unwrap();
        assert!(
            todo.contains("## In Progress\n\n### feat: a cycle\n")
                && todo.contains("##### feat: a cycle opening")
                && todo.contains("## Todo\n\n## Closed\n"),
            "{todo}"
        );
        let ztoc = fs::read_to_string(fx.work.join("zTOC.md")).unwrap();
        assert!(
            ztoc.contains(
                "## Ladder\n\n- [feat: a cycle opening](TODO.md#feat-a-cycle-opening)\n\
                 - [feat: a cycle closing](TODO.md#feat-a-cycle-closing)\n"
            ),
            "{ztoc}"
        );
        run(&fx.work, &["ladder"]).expect("the skeleton's record is a cycle's");

        let fx = Fixture::new("cycle-open-neither");
        fs::remove_file(fx.work.join("TODO.md")).expect("remove init's");
        fs::remove_file(fx.work.join("zTOC.md")).expect("remove init's");
        run(
            &fx.work,
            &["plan", "start", "--title", "feat: a cycle", "--dry-run"],
        )
        .expect("a dry run");
        assert!(!fx.work.join("TODO.md").exists(), "a dry run writes none");
        run(&fx.work, &["plan", "start", "--title", "feat: a cycle"]).expect("planned");
        assert!(
            fs::read_to_string(fx.work.join("TODO.md"))
                .unwrap()
                .contains("### feat: a cycle\n"),
            "the skeleton is made, the record in it"
        );

        let fx = Fixture::new("cycle-open-one-missing");
        fs::write(fx.work.join("TODO.md"), TODO).expect("write");
        fs::remove_file(fx.work.join("zTOC.md")).expect("remove init's");
        run(&fx.work, &["plan", "start", "--title", "feat: a cycle"]).expect("planned");
        let todo = fs::read_to_string(fx.work.join("TODO.md")).unwrap();
        assert!(
            todo.contains("The record's home."),
            "the file there is kept: {todo}"
        );
        assert!(
            fs::read_to_string(fx.work.join("zTOC.md"))
                .unwrap()
                .contains("- [feat: a cycle opening]"),
            "the missing one is made"
        );

        let fx = Fixture::new("cycle-open-no-record");
        write_record(&fx);
        run(
            &fx.work,
            &["plan", "start", "--title", "feat: a cycle", "--no-record"],
        )
        .expect("opened");
        assert_eq!(bookmarks_at(&fx.work), "feat-a-cycle");
        assert_eq!(fs::read_to_string(fx.work.join("TODO.md")).unwrap(), TODO);

        let fx = Fixture::new("cycle-open-named");
        write_record(&fx);
        run(&fx.work, &["plan", "start", "--bookmark", "mine"]).expect("opened");
        assert_eq!(bookmarks_at(&fx.work), "mine");
        assert_eq!(fs::read_to_string(fx.work.join("TODO.md")).unwrap(), TODO);
    }

    /// `cycle ladder` makes the ladder from the rung headings, a commit of
    /// a rung's title making it done, and a commit that is no rung's is
    /// refused, the ladder left as it was.
    #[test]
    fn ladder_is_made_from_the_headings_and_the_commits() {
        let fx = Fixture::new("cycle-ladder");
        write_record(&fx);
        run(&fx.work, &["plan", "start", "--title", "feat: a cycle"]).expect("opened");
        let todo = fs::read_to_string(fx.work.join("TODO.md")).unwrap();
        let planned = todo.replace(
            "##### feat: a cycle closing",
            "##### feat: one rung\n\nIts intent.\n\n##### feat: a cycle closing",
        );
        fs::write(fx.work.join("TODO.md"), planned).expect("write");
        let ladder = || fs::read_to_string(fx.work.join("zTOC.md")).unwrap();

        run(&fx.work, &["ladder", "--dry-run"]).expect("a dry run");
        assert!(
            !ladder().contains("feat: one rung"),
            "a dry run writes none"
        );
        run(&fx.work, &["ladder"]).expect("made");
        assert!(
            ladder().contains(
                "- [feat: a cycle opening](TODO.md#feat-a-cycle-opening)\n\
                 - [feat: one rung](TODO.md#feat-one-rung)\n\
                 - [feat: a cycle closing](TODO.md#feat-a-cycle-closing)\n"
            ),
            "{}",
            ladder()
        );

        jj_ok(&fx.work, &["commit", "-m", "feat: a cycle opening"]);
        run(&fx.work, &["ladder"]).expect("made");
        assert!(
            ladder().contains(
                "- 0 (done) [feat: a cycle opening](TODO.md#feat-a-cycle-opening)\n\
                 - [feat: one rung](TODO.md#feat-one-rung)\n"
            ),
            "{}",
            ladder()
        );

        let before = ladder();
        jj_ok(&fx.work, &["commit", "-m", "wip: not a rung"]);
        let err = run(&fx.work, &["ladder"]).unwrap_err();
        assert!(err.contains("'wip: not a rung'"), "{err}");
        assert_eq!(ladder(), before);
    }

    /// `cycle ladder` reads the cycle in progress, so it takes no title, and
    /// once a rung is done needs the cycle's bookmark, named when it is
    /// not the slug.
    #[test]
    fn ladder_needs_the_cycle_and_its_bookmark() {
        let fx = Fixture::new("cycle-ladder-needs");
        write_record(&fx);
        let err = params(&fx.work, &["ladder", "--title", "feat: x"]).unwrap_err();
        assert!(err.contains("unexpected argument '--title'"), "{err}");
        let err = run(&fx.work, &["ladder"]).unwrap_err();
        assert!(err.contains("no cycle in progress"), "{err}");

        run(&fx.work, &["plan", "start", "--title", "feat: a cycle"]).expect("opened");
        commit(&fx.work, &[]).expect("the opening");
        jj_ok(&fx.work, &["bookmark", "rename", "feat-a-cycle", "mine"]);
        let err = run(&fx.work, &["ladder"]).unwrap_err();
        assert!(
            err.contains("'feat-a-cycle'") && err.contains("--bookmark"),
            "{err}"
        );
        run(&fx.work, &["ladder", "--bookmark", "mine"]).expect("made");
    }

    /// A file's text at a revision.
    fn file_at(repo: &Path, rev: &str, file: &str) -> String {
        jj_ok(
            repo,
            &["file", "show", "-r", rev, &format!("root:\"{file}\"")],
        )
    }

    /// An opened cycle with one work rung planned, its edits in `@`.
    fn planned(tag: &str) -> Fixture {
        let fx = Fixture::new(tag);
        write_record(&fx);
        run(&fx.work, &["plan", "start", "--title", "feat: a cycle"]).expect("opened");
        let todo = fs::read_to_string(fx.work.join("TODO.md")).unwrap();
        let planned = todo.replace(
            "##### feat: a cycle closing",
            "##### feat: one rung\n\nIts intent.\n\n##### feat: a cycle closing",
        );
        fs::write(fx.work.join("TODO.md"), planned).expect("write");
        fx
    }

    /// The cycle's first commit is its opening, titled and bodied by
    /// default, the ladder in it showing it done and no rung current,
    /// the cycle's bookmark on it, and the pair stamped.
    #[test]
    fn the_first_commit_is_the_opening() {
        let fx = planned("cycle-commit-opening");
        commit(&fx.work, &["--dry-run"]).expect("a dry run");
        assert_eq!(bookmarks_at(&fx.work), "feat-a-cycle", "nothing committed");
        assert!(
            !fs::read_to_string(fx.work.join("zTOC.md"))
                .unwrap()
                .contains("(done) [feat: a cycle opening]"),
            "a dry run writes no ladder"
        );

        commit(&fx.work, &[]).expect("the opening");
        let desc = description(&fx.work, "@-");
        assert!(
            desc.starts_with(
                "feat: a cycle opening\n\nOpens the cycle \"feat: a cycle\".\n\nSee details in \
                 TODO.md:\n### feat: a cycle\n"
            ),
            "{desc}"
        );
        assert!(desc.contains("ochid: "), "{desc}");
        assert_eq!(
            jj_ok(
                &fx.work,
                &["log", "-r", "@-", "--no-graph", "-T", "local_bookmarks"]
            ),
            "feat-a-cycle"
        );
        assert!(
            file_at(&fx.work, "@-", "zTOC.md").contains(
                "- 0 (done) [feat: a cycle opening](TODO.md#feat-a-cycle-opening)\n\
                 - [feat: one rung](TODO.md#feat-one-rung)\n\
                 - [feat: a cycle closing](TODO.md#feat-a-cycle-closing)\n"
            ),
            "the commit carries the ladder"
        );
        assert!(
            description(&fx.bot, "@-").starts_with("feat: a cycle opening\n"),
            "the agent half is made too"
        );
    }

    /// A work rung gives its title, a rung heading, and its body, and
    /// the ladder in its commit shows it done.
    #[test]
    fn a_work_rung_is_committed_under_its_heading() {
        let fx = planned("cycle-commit-rung");
        commit(&fx.work, &[]).expect("the opening");
        fs::write(fx.work.join("work.txt"), "the rung's work\n").expect("write");
        let rung = [
            "--title",
            "feat: one rung",
            "--body",
            "Why.\n\nSolution: how.",
        ];
        commit(&fx.work, &rung).expect("the rung");
        let desc = description(&fx.work, "@-");
        assert!(
            desc.starts_with("feat: one rung\n\nWhy.\n\nSolution: how.\n"),
            "{desc}"
        );
        assert!(
            file_at(&fx.work, "@-", "zTOC.md").contains(
                "- 1 (done) [feat: one rung](TODO.md#feat-one-rung)\n\
                 - [feat: a cycle closing](TODO.md#feat-a-cycle-closing)\n"
            ),
            "the commit carries the ladder"
        );
        assert_eq!(
            jj_ok(
                &fx.work,
                &["log", "-r", "@-", "--no-graph", "-T", "local_bookmarks"]
            ),
            "feat-a-cycle",
            "the cycle's bookmark is the one advanced"
        );
        assert_eq!(
            jj_ok(
                &fx.work,
                &["log", "-r", "main", "--no-graph", "-T", "description"]
            ),
            jj_ok(
                &fx.work,
                &[
                    "log",
                    "-r",
                    "main@origin",
                    "--no-graph",
                    "-T",
                    "description"
                ]
            ),
            "main is left where it was"
        );
    }

    /// Refused before anything is made, the ladder as it was: a first
    /// commit under another title, a work rung with no title or body, a
    /// title that is no rung's, one committed already, the closing's,
    /// and an empty working copy.
    #[test]
    fn a_commit_that_is_no_rungs_is_refused() {
        let fx = planned("cycle-commit-refused");
        let rung = ["--title", "feat: one rung", "--body", "b"];
        let err = commit(&fx.work, &rung).unwrap_err();
        assert!(err.contains("its opening"), "{err}");
        commit(&fx.work, &[]).expect("the opening");
        let ladder = fs::read_to_string(fx.work.join("zTOC.md")).unwrap();

        let err = commit(&fx.work, &rung).unwrap_err();
        assert!(err.contains("nothing to commit"), "{err}");
        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        let err = commit(&fx.work, &[]).unwrap_err();
        assert!(err.contains("a work rung"), "{err}");
        let err = commit(&fx.work, &["--title", "feat: one rung"]).unwrap_err();
        assert!(err.contains("--body"), "{err}");
        let err = commit(&fx.work, &["--title", "feat: unplanned", "--body", "b"]).unwrap_err();
        assert!(err.contains("no rung of the cycle"), "{err}");
        let err = commit(
            &fx.work,
            &["--title", "feat: a cycle closing", "--body", "b"],
        )
        .unwrap_err();
        assert!(err.contains("cycle close"), "{err}");
        let err = commit(
            &fx.work,
            &["--title", "feat: a cycle opening", "--body", "b"],
        )
        .unwrap_err();
        assert!(err.contains("committed already"), "{err}");
        assert_eq!(fs::read_to_string(fx.work.join("zTOC.md")).unwrap(), ladder);
        assert_eq!(
            description(&fx.work, "@-").lines().next(),
            Some("feat: a cycle opening")
        );
    }

    /// `-s` is required with the stages that write commits and taken
    /// by no other, it is checked against the workspace, `w` makes the
    /// work half alone, and a plain repo's commit carries no trailer.
    #[test]
    fn the_scope_is_required_and_fits_the_workspace() {
        for stage in [vec!["close"], vec!["land", "main"], vec!["push"]] {
            let mut argv = vec!["vc-x1", "cycle"];
            argv.extend(&stage);
            let err = parse(&argv).unwrap_err();
            assert!(err.contains("--scope"), "{stage:?}: {err}");
        }
        let err = commit(Path::new("."), &["-s"]).unwrap_err();
        assert!(err.contains("--scope"), "rung commit: {err}");
        for stage in ["ladder", "status"] {
            parse(&["vc-x1", "cycle", stage]).expect("no scope needed");
        }
        let fx = planned("cycle-commit-scope");
        let err = params(&fx.work, &["status", "-s", "b"]).unwrap_err();
        assert!(err.contains("unexpected argument '-s'"), "{err}");
        let err = params(&fx.work, &["land", "main", "-s", "p"]).unwrap_err();
        assert!(err.contains("dual workspace"), "{err}");
        let err = commit(&fx.work, &["-s", "p"]).unwrap_err();
        assert!(err.contains("dual workspace"), "{err}");
        let agent_at = jj_ok(
            &fx.bot,
            &["log", "-r", "@", "--no-graph", "-T", "change_id.short(12)"],
        );
        commit(&fx.work, &["-s", "w"]).expect("the work half");
        assert!(description(&fx.work, "@-").contains(&format!("ochid: {agent_at}")));
        assert_eq!(
            jj_ok(
                &fx.bot,
                &["log", "-r", "@", "--no-graph", "-T", "change_id.short(12)"]
            ),
            agent_at,
            "the agent @ is left for push"
        );

        let por = crate::test_helpers::FixturePor::new("cycle-commit-por");
        fs::write(por.work.join("TODO.md"), TODO).expect("write");
        fs::write(por.work.join("zTOC.md"), ZTOC).expect("write");
        let err = commit(&por.work, &["-s", "b"]).unwrap_err();
        assert!(err.contains("plain repo"), "{err}");
        run(&por.work, &["plan", "start", "--title", "feat: a cycle"]).expect("opened");
        commit(&por.work, &["-s", "p"]).expect("the opening");
        let desc = description(&por.work, "@-");
        assert!(desc.starts_with("feat: a cycle opening\n"), "{desc}");
        assert!(!desc.contains("ochid:"), "{desc}");
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

    fn title_of(repo: &Path, rev: &str) -> String {
        description(repo, rev)
            .lines()
            .next()
            .unwrap_or_default()
            .to_string()
    }

    /// The titles of the commits above `main`, oldest first.
    fn above_main(repo: &Path) -> Vec<String> {
        jj_ok(
            repo,
            &[
                "log",
                "-r",
                "main..@-",
                "--no-graph",
                "--reversed",
                "-T",
                "description.first_line() ++ \"\\n\"",
            ],
        )
        .lines()
        .map(str::to_string)
        .collect()
    }

    /// The record is moved and the ladder is one unlinked rung, in the
    /// commit `rev`.
    fn assert_closed_single(repo: &Path, rev: &str) {
        let todo = file_at(repo, rev, "TODO.md");
        let closed = &todo[todo.find("## Closed").expect("closed")..];
        assert!(
            !closed.contains("#### Rungs"),
            "a single step has no rungs: {closed}"
        );
        assert!(todo.contains("_No cycle currently in progress._"), "{todo}");
        assert!(
            todo.contains("## Closed\n\nThe last cycle's record.\n\n### feat: a cycle\n"),
            "{todo}"
        );
        assert!(
            file_at(repo, rev, "zTOC.md").contains("## Ladder\n\n- 0 (done) feat: a cycle\n\n")
        );
    }

    /// A branch with nothing committed closes as one commit made from
    /// the work `@`, under the bare cycle title, its body required.
    #[test]
    fn a_cycle_with_no_commit_closes_as_one_commit() {
        let fx = Fixture::new("cycle-close-none");
        write_record(&fx);
        run(&fx.work, &["plan", "start", "--title", "feat: a cycle"]).expect("opened");
        fs::write(fx.work.join("work.txt"), "the work\n").expect("write");
        let err = run(&fx.work, &["close", "--squash"]).unwrap_err();
        assert!(
            err.contains("--body") && err.contains("--default-body"),
            "{err}"
        );
        let err = run(&fx.work, &["close", "main", "--squash", "--body", "b"]).unwrap_err();
        assert!(
            err.contains("cannot be used with"),
            "the one commit joins nothing: {err}"
        );
        run(
            &fx.work,
            &[
                "close",
                "--squash",
                "--body",
                "Why.\n\nSolution: how.",
                "--dry-run",
            ],
        )
        .expect("a dry run");
        assert!(above_main(&fx.work).is_empty(), "a dry run commits nothing");

        run(
            &fx.work,
            &["close", "--squash", "--body", "Why.\n\nSolution: how."],
        )
        .expect("closed");
        assert_eq!(above_main(&fx.work), ["feat: a cycle"]);
        assert!(
            description(&fx.work, "@-").contains("Solution: how.\n\n"),
            "the body, then trailers"
        );
        assert_closed_single(&fx.work, "@-");
        assert_eq!(
            jj_ok(
                &fx.work,
                &["log", "-r", "@-", "--no-graph", "-T", "local_bookmarks"]
            ),
            "feat-a-cycle"
        );
        assert!(
            title_of(&fx.bot, "@-") == "feat: a cycle",
            "the agent half is made"
        );
    }

    /// A branch of one commit, the opening, closes into it: the work
    /// `@` folded in, the title losing " opening", the trailer kept.
    #[test]
    fn a_cycle_of_one_commit_closes_into_it() {
        let fx = Fixture::new("cycle-close-one");
        write_record(&fx);
        run(&fx.work, &["plan", "start", "--title", "feat: a cycle"]).expect("opened");
        commit(&fx.work, &[]).expect("the opening");
        let opening = jj_ok(
            &fx.work,
            &["log", "-r", "@-", "--no-graph", "-T", "change_id.short(12)"],
        );
        let trailer = |d: String| {
            d.lines()
                .find(|l| l.starts_with("ochid:"))
                .map(str::to_string)
        };
        let stamped = trailer(description(&fx.work, "@-"));
        fs::write(fx.work.join("work.txt"), "the work\n").expect("write");

        run(&fx.work, &["close", "--squash", "--default-body"]).expect("closed");
        assert_eq!(above_main(&fx.work), ["feat: a cycle"]);
        assert_eq!(
            jj_ok(
                &fx.work,
                &["log", "-r", "@-", "--no-graph", "-T", "change_id.short(12)"]
            ),
            opening,
            "the opening's change is the one kept"
        );
        let desc = description(&fx.work, "@-");
        assert!(
            desc.starts_with("feat: a cycle\n\nCloses the cycle \"feat: a cycle\"."),
            "{desc}"
        );
        assert_eq!(trailer(desc), stamped, "its trailer is kept");
        assert_eq!(
            file_at(&fx.work, "@-", "work.txt"),
            "the work",
            "the work @ is folded in"
        );
        assert_closed_single(&fx.work, "@-");
    }

    /// A ladder closes with its own commit, linear by default, titled
    /// and bodied by default, the record moved and every rung done.
    #[test]
    fn a_ladder_closes_with_a_closing_commit() {
        let fx = planned("cycle-close-ladder");
        commit(&fx.work, &[]).expect("the opening");
        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        commit(&fx.work, &["--title", "feat: one rung", "--body", "b"]).expect("the rung");
        let tip = jj_ok(
            &fx.work,
            &["log", "-r", "@-", "--no-graph", "-T", "commit_id"],
        );
        jj_ok(&fx.work, &["new", "feat-a-cycle-"]);
        let err = run(&fx.work, &["close", "--linear"]).unwrap_err();
        assert!(err.contains("not on the work @ or @-"), "{err}");
        jj_ok(&fx.work, &["new", "feat-a-cycle"]);

        run(&fx.work, &["close", "--linear"]).expect("closed");
        assert_eq!(
            above_main(&fx.work),
            [
                "feat: a cycle opening",
                "feat: one rung",
                "feat: a cycle closing"
            ]
        );
        assert_eq!(
            parents(&fx.work, "@-"),
            [tip],
            "linear, the ladder's tip its one parent"
        );
        let desc = description(&fx.work, "@-");
        assert!(
            desc.contains(
                "Closes the cycle \"feat: a cycle\".\n\nSee details in TODO.md:\n### feat: a cycle\n"
            ),
            "{desc}"
        );
        let todo = file_at(&fx.work, "@-", "TODO.md");
        assert!(todo.contains("_No cycle currently in progress._"), "{todo}");
        assert!(
            todo.contains("## Closed\n\nThe last cycle's record.\n\n### feat: a cycle\n"),
            "{todo}"
        );
        assert!(
            file_at(&fx.work, "@-", "zTOC.md").contains(
                "- 1 (done) [feat: one rung](TODO.md#feat-one-rung)\n\
                 - 2 (done) [feat: a cycle closing](TODO.md#feat-a-cycle-closing)\n"
            ),
            "every rung is done"
        );
        assert_eq!(
            jj_ok(
                &fx.work,
                &["log", "-r", "@-", "--no-graph", "-T", "local_bookmarks"]
            ),
            "feat-a-cycle"
        );
    }

    /// A trapezoid, the default, makes the target the closing's first
    /// parent, and a head `--join` lists is another, after the ladder's
    /// tip.
    #[test]
    fn a_trapezoid_joins_the_destination_and_the_heads_listed() {
        let fx = planned("cycle-close-trapezoid");
        commit(&fx.work, &[]).expect("the opening");
        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        commit(&fx.work, &["--title", "feat: one rung", "--body", "b"]).expect("the rung");
        let tip = jj_ok(
            &fx.work,
            &["log", "-r", "@-", "--no-graph", "-T", "commit_id"],
        );
        let main = jj_ok(
            &fx.work,
            &["log", "-r", "main", "--no-graph", "-T", "commit_id"],
        );
        // Another line off main, joined as a head.
        jj_ok(&fx.work, &["new", "main", "-m", "feat: another line"]);
        fs::write(fx.work.join("side.txt"), "side\n").expect("write");
        let side = jj_ok(
            &fx.work,
            &["log", "-r", "@", "--no-graph", "-T", "commit_id"],
        );
        jj_ok(&fx.work, &["new", "feat-a-cycle"]);
        let side = jj_ok(
            &fx.work,
            &["log", "-r", &side, "--no-graph", "-T", "commit_id"],
        );

        run(&fx.work, &["close", "main", "--join", &side]).expect("closed");
        assert_eq!(title_of(&fx.work, "@-"), "feat: a cycle closing");
        assert_eq!(parents(&fx.work, "@-"), [main, tip, side]);
        assert_eq!(
            file_at(&fx.work, "@-", "side.txt"),
            "side",
            "the head's tree is merged"
        );
        assert_eq!(file_at(&fx.work, "@-", "work.txt"), "work");
    }

    /// A dead end is always a closing commit, titled so, and a branch
    /// with no commit has none to close.
    #[test]
    fn a_dead_end_is_a_closing_commit() {
        let fx = planned("cycle-close-dead-end");
        let err = run(&fx.work, &["close", "--linear", "--dead-end"]).unwrap_err();
        assert!(err.contains("no commit"), "{err}");
        commit(&fx.work, &[]).expect("the opening");
        fs::write(fx.work.join("work.txt"), "kept\n").expect("write");
        run(&fx.work, &["close", "--linear", "--dead-end"]).expect("closed");
        assert_eq!(
            above_main(&fx.work),
            ["feat: a cycle opening", "feat: a cycle dead-end closing"]
        );
    }

    fn cid(repo: &Path, rev: &str) -> String {
        jj_ok(repo, &["log", "-r", rev, "--no-graph", "-T", "commit_id"])
    }

    /// A ladder of one work rung, its two commits made, `@` on its tip.
    fn laddered(tag: &str) -> Fixture {
        let fx = planned(tag);
        commit(&fx.work, &[]).expect("the opening");
        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        commit(&fx.work, &["--title", "feat: one rung", "--body", "b"]).expect("the rung");
        fx
    }

    /// `cycle close --land` lands the closing it makes: the destination is
    /// fast-forwarded to it and the cycle's bookmark deleted, and a dry
    /// run makes neither.
    #[test]
    fn close_and_land_chain() {
        let fx = laddered("cycle-land-chain");
        let main = cid(&fx.work, "main");
        run(
            &fx.work,
            &["close", "--linear", "--land", "main", "--dry-run"],
        )
        .expect("a dry run");
        assert_eq!(cid(&fx.work, "main"), main, "a dry run lands nothing");
        assert_eq!(
            title_of(&fx.work, "@-"),
            "feat: one rung",
            "and closes nothing"
        );

        run(&fx.work, &["close", "--linear", "--land", "main"]).expect("closed and landed");
        assert_eq!(title_of(&fx.work, "main"), "feat: a cycle closing");
        assert_eq!(cid(&fx.work, "main"), cid(&fx.work, "@-"));
        assert_eq!(
            jj_ok(
                &fx.work,
                &[
                    "log",
                    "-r",
                    "@-",
                    "--no-graph",
                    "-T",
                    "local_bookmarks.map(|b| b.name())"
                ]
            ),
            "main",
            "the cycle's bookmark is deleted"
        );
        assert_ne!(
            cid(&fx.work, "main@origin"),
            cid(&fx.work, "main"),
            "nothing is pushed"
        );
    }

    /// A single-step cycle closes and lands in one invocation, and
    /// `--push` publishes the destination.
    #[test]
    fn a_single_step_cycle_closes_lands_and_publishes() {
        let fx = Fixture::new("cycle-land-single");
        write_record(&fx);
        run(&fx.work, &["plan", "start", "--title", "feat: a cycle"]).expect("opened");
        fs::write(fx.work.join("work.txt"), "the work\n").expect("write");
        run(
            &fx.work,
            &[
                "close", "--squash", "--body", "Why.", "--land", "main", "--push",
            ],
        )
        .expect("landed");
        assert_eq!(title_of(&fx.work, "main"), "feat: a cycle");
        assert_eq!(
            cid(&fx.work, "main@origin"),
            cid(&fx.work, "main"),
            "published"
        );
        assert_eq!(
            cid(&fx.bot, "main@origin"),
            cid(&fx.bot, "main"),
            "the agent repo too"
        );
    }

    /// `cycle land` alone lands a cycle already closed, found by the title
    /// `## Closed` holds, and a cycle in progress is refused.
    #[test]
    fn land_is_of_a_cycle_closed() {
        let fx = laddered("cycle-land-closed");
        let err = run(&fx.work, &["land", "main"]).unwrap_err();
        assert!(
            err.contains("in progress") && err.contains("cycle close"),
            "{err}"
        );
        let err = params(&fx.work, &["land", "main", "--title", "t"]).unwrap_err();
        assert!(err.contains("--merge"), "{err}");
        let err = params(&fx.work, &["land", "main", "-s", "w"]).unwrap_err();
        assert!(err.contains("-s w") && err.contains("-s b"), "{err}");
        run(&fx.work, &["close", "--linear"]).expect("closed");
        let err = run(&fx.work, &["land", "trunk"]).unwrap_err();
        assert!(err.contains("not a local bookmark"), "{err}");
        run(&fx.work, &["land", "main"]).expect("landed");
        assert_eq!(title_of(&fx.work, "main"), "feat: a cycle closing");
    }

    /// A destination that moved is refused naming the ways on, and
    /// `--merge` joins it by a merge commit, which is what lands.
    #[test]
    fn a_destination_that_moved_is_refused_or_merged() {
        let fx = laddered("cycle-land-moved");
        run(&fx.work, &["close", "--linear"]).expect("closed");
        let closing = cid(&fx.work, "@-");
        // The destination moves on origin, and is synced here.
        jj_ok(&fx.work, &["new", "main", "-m", "feat: someone else's"]);
        fs::write(fx.work.join("other.txt"), "theirs\n").expect("write");
        jj_ok(&fx.work, &["bookmark", "set", "main", "-r", "@"]);
        jj_ok(&fx.work, &["git", "push", "--bookmark", "main"]);
        let main = cid(&fx.work, "main");
        jj_ok(&fx.work, &["new", "feat-a-cycle"]);

        let err = run(&fx.work, &["land", "main"]).unwrap_err();
        assert!(
            err.contains("moved") && err.contains("--merge") && err.contains("jj rebase"),
            "{err}"
        );
        assert_eq!(cid(&fx.work, "main"), main, "nothing moved");

        run(&fx.work, &["land", "main", "--merge"]).expect("merged and landed");
        assert_eq!(title_of(&fx.work, "main"), "feat: a cycle merge");
        assert_eq!(parents(&fx.work, "main"), [main, closing]);
        assert_eq!(file_at(&fx.work, "main", "other.txt"), "theirs");
        assert_eq!(file_at(&fx.work, "main", "work.txt"), "work");
        assert!(
            description(&fx.work, "main").contains("ochid: "),
            "a pair's commit"
        );
    }

    /// A rung is started by `rung next`, its marker the working copy's
    /// alone, left by `rung stop`, and ended by `rung commit`, whose
    /// commit marks it done and none current. A started work rung holds
    /// back the closing, and the closing is `cycle close`'s.
    #[test]
    fn a_rung_is_started_stopped_and_committed() {
        let fx = planned("cycle-rung");
        run(&fx.work, &["ladder"]).expect("laddered");
        let err = rung(&fx.work, &["commit", "-s", "b"]).unwrap_err();
        assert!(
            err.contains("no rung is current") && err.contains("rung next"),
            "{err}"
        );
        let err = rung(&fx.work, &["stop"]).unwrap_err();
        assert!(err.contains("no rung is current"), "{err}");

        let err = rung(&fx.work, &["next"]).unwrap_err();
        assert!(
            err.contains("cycle plan commit"),
            "the opening is the plan's: {err}"
        );
        run(&fx.work, &["plan", "commit", "-s", "b"]).expect("the plan committed");
        assert_eq!(title_of(&fx.work, "@-"), "feat: a cycle opening");
        let committed = file_at(&fx.work, "@-", "zTOC.md");
        assert!(
            committed.contains("- 0 (done) [feat: a cycle opening]"),
            "{committed}"
        );
        assert!(!committed.contains("(current)"), "{committed}");
        let err = run(&fx.work, &["plan", "commit", "-s", "b"]).unwrap_err();
        assert!(err.contains("committed already"), "{err}");

        rung(&fx.work, &["next"]).expect("the work rung started");
        assert_eq!(
            ladder_lines(&fx.work)[1],
            "- 1 (current) [feat: one rung](TODO.md#feat-one-rung)"
        );
        let err = rung(&fx.work, &["next"]).unwrap_err();
        assert!(err.contains("'feat: one rung' is started"), "{err}");
        run(&fx.work, &["ladder"]).expect("laddered again");
        assert!(
            ladder_lines(&fx.work)[1].contains("(current)"),
            "the ladder keeps it"
        );
        rung(&fx.work, &["stop"]).expect("stopped");
        assert!(
            !ladder_lines(&fx.work)
                .iter()
                .any(|l| l.contains("(current)"))
        );
        rung(&fx.work, &["next"]).expect("the work rung started again");
        let err = run(&fx.work, &["close", "--linear"]).unwrap_err();
        assert!(err.contains("'feat: one rung' is started"), "{err}");
        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        let err = rung(&fx.work, &["commit", "-s", "b"]).unwrap_err();
        assert!(
            err.starts_with("rung commit: a work rung gives --body"),
            "{err}"
        );
        rung(&fx.work, &["commit", "-s", "b", "--body", "Why.\n\nHow."]).expect("the rung");
        assert_eq!(title_of(&fx.work, "@-"), "feat: one rung");

        rung(&fx.work, &["next"]).expect("the closing started");
        let err = rung(&fx.work, &["commit", "-s", "b"]).unwrap_err();
        assert!(err.contains("cycle close"), "{err}");
        run(&fx.work, &["close", "--linear"]).expect("closed");
        assert_eq!(title_of(&fx.work, "@-"), "feat: a cycle closing");
        let closed = file_at(&fx.work, "@-", "zTOC.md");
        assert!(!closed.contains("(current)"), "{closed}");
    }

    /// `TODO.md` is the ladder's source: with the cycle's bookmark not
    /// made, the ladder is made with no rung done, and refused when the
    /// ladder there has one done, which it would lose.
    #[test]
    fn the_ladder_needs_no_bookmark_before_a_commit() {
        let fx = planned("cycle-ladder-no-bookmark");
        jj_ok(&fx.work, &["bookmark", "delete", "feat-a-cycle"]);
        run(&fx.work, &["ladder"]).expect("laddered without the bookmark");
        assert_eq!(
            ladder_lines(&fx.work),
            [
                "- [feat: a cycle opening](TODO.md#feat-a-cycle-opening)",
                "- [feat: one rung](TODO.md#feat-one-rung)",
                "- [feat: a cycle closing](TODO.md#feat-a-cycle-closing)",
            ]
        );
        let ztoc = fx.work.join("zTOC.md");
        let done = fs::read_to_string(&ztoc).expect("read").replace(
            "- [feat: a cycle opening]",
            "- 0 (done) [feat: a cycle opening]",
        );
        fs::write(&ztoc, &done).expect("write");
        let err = run(&fx.work, &["ladder"]).unwrap_err();
        assert!(
            err.contains("has a rung done") && err.contains("--bookmark"),
            "{err}"
        );
        assert_eq!(
            fs::read_to_string(&ztoc).expect("read"),
            done,
            "nothing written"
        );
    }

    /// A one-off is `cycle plan start`, the work, and `cycle close
    /// --squash`, the plan never committed: a ladder's closing has no
    /// opening to follow.
    #[test]
    fn a_one_off_squashes_after_plan_start() {
        let fx = planned("cycle-rung-single");
        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        let err = run(&fx.work, &["close", "--linear"]).unwrap_err();
        assert!(
            err.contains("no opening") && err.contains("cycle close --squash"),
            "{err}"
        );
        run(&fx.work, &["close", "--squash", "--body", "Why.\n\nHow."]).expect("closed");
        assert_eq!(title_of(&fx.work, "@-"), "feat: a cycle");
        let closed = file_at(&fx.work, "@-", "zTOC.md");
        assert!(closed.contains("- 0 (done) feat: a cycle"), "{closed}");
        assert!(!closed.contains("(current)"), "{closed}");
    }

    #[test]
    /// `rung next` before the plan's commit is refused, a bare `--ver`
    /// included, naming `cycle plan commit` and the one commit by `cycle
    /// close --squash`, nothing changed.
    fn rung_next_before_the_plan_is_refused() {
        let fx = planned("cycle-rung-next-plan");
        let ladder = fs::read_to_string(fx.work.join("zTOC.md")).unwrap();
        for args in [vec!["next"], vec!["next", "--ver", "1.0.0"]] {
            let err = rung(&fx.work, &args).unwrap_err();
            assert!(
                err.contains("cycle plan commit") && err.contains("cycle close --squash"),
                "{args:?}: {err}"
            );
        }
        assert_eq!(
            fs::read_to_string(fx.work.join("zTOC.md")).unwrap(),
            ladder,
            "nothing changed"
        );
    }

    /// With no cycle in progress, `rung commit --title` commits the work
    /// `@` verbatim, the record untouched: the first of a run on the
    /// bookmark a bare `--bookmark` makes, the title's slug, the next
    /// advancing it with no flag, and `cycle land` lands the run once.
    /// Refused: no `--body`, an empty `@`, no flag with `main` on `@-`,
    /// and `--title` while a cycle is in progress.
    #[test]
    fn rung_commit_without_a_cycle_iterates_on_one_bookmark() {
        let fx = Fixture::new("cycle-alone");
        let todo = fs::read_to_string(fx.work.join("TODO.md")).unwrap();
        let err = rung(&fx.work, &["commit", "-s", "b"]).unwrap_err();
        assert!(
            err.contains("no cycle is in progress") && err.contains("--title"),
            "{err}"
        );
        let alone = ["commit", "-s", "b", "--title", "feat: create abc"];
        let err = rung(&fx.work, &alone).unwrap_err();
        assert!(err.contains("--body"), "{err}");
        let mut with_body = alone.to_vec();
        with_body.extend(["--body", "Why.\n\nSolution: how."]);
        let err = rung(&fx.work, &with_body).unwrap_err();
        assert!(err.contains("empty"), "{err}");
        assert_eq!(bookmarks_at(&fx.work), "", "nothing made");

        fs::write(fx.work.join("abc.txt"), "abc\n").expect("write");
        let err = rung(&fx.work, &with_body).unwrap_err();
        assert!(
            err.contains("'main', a target") && err.contains("--bookmark"),
            "{err}"
        );
        let mut first = with_body.clone();
        first.push("--bookmark");
        rung(&fx.work, &first).expect("committed");
        assert_eq!(title_of(&fx.work, "@-"), "feat: create abc");
        assert_eq!(
            jj_ok(
                &fx.work,
                &["log", "-r", "@-", "--no-graph", "-T", "local_bookmarks"]
            ),
            "feat-create-abc"
        );
        assert!(
            description(&fx.bot, "@-").starts_with("feat: create abc\n"),
            "the agent half is made too"
        );

        fs::write(fx.work.join("def.txt"), "def\n").expect("write");
        rung(
            &fx.work,
            &[
                "commit",
                "-s",
                "b",
                "--title",
                "feat: create def",
                "--body",
                "b",
            ],
        )
        .expect("the run's second");
        assert_eq!(
            jj_ok(
                &fx.work,
                &["log", "-r", "@-", "--no-graph", "-T", "local_bookmarks"]
            ),
            "feat-create-abc",
            "the bookmark advanced"
        );
        assert_eq!(
            fs::read_to_string(fx.work.join("TODO.md")).unwrap(),
            todo,
            "the record is untouched"
        );
        run(&fx.work, &["land", "main"]).expect("landed");
        assert_eq!(title_of(&fx.work, "main"), "feat: create def");
        assert_eq!(title_of(&fx.work, "main-"), "feat: create abc");

        let fx = planned("cycle-alone-in-progress");
        let err = rung(
            &fx.work,
            &["commit", "-s", "b", "--title", "feat: x", "--body", "b"],
        )
        .unwrap_err();
        assert!(err.contains("is in progress"), "{err}");
        let err = rung(&fx.work, &["commit", "-s", "b", "--record"]).unwrap_err();
        assert!(
            err.contains("unexpected argument"),
            "--record is gone: {err}"
        );
    }

    /// `--bookmark NAME` is moved forward when it is behind `@-`, said;
    /// sideways it asks, refused with no terminal and moved by `--yes`;
    /// and `main` is never taken, whatever its place.
    #[test]
    fn rung_commit_bookmark_moves_by_direction() {
        let fx = Fixture::new("cycle-alone-move");
        let commit_as = |title: &str, file: &str, extra: &[&str]| {
            fs::write(fx.work.join(file), "x\n").expect("write");
            let mut args = vec!["commit", "-s", "b", "--title", title, "--body", "b"];
            args.extend_from_slice(extra);
            rung(&fx.work, &args)
        };
        // `old` behind @-: one commit on main, then `old` left below it.
        commit_as("feat: one", "one.txt", &["--bookmark", "old"]).expect("one");
        jj_ok(
            &fx.work,
            &["bookmark", "set", "old", "-r", "main", "--allow-backwards"],
        );
        commit_as("feat: two", "two.txt", &["--bookmark", "old"]).expect("moved forward");
        assert_eq!(
            jj_ok(
                &fx.work,
                &[
                    "log",
                    "-r",
                    "old",
                    "--no-graph",
                    "-T",
                    "description.first_line()"
                ]
            ),
            "feat: two",
            "the bookmark moved forward and advanced"
        );

        // `side` on another line: asked, refused without a terminal.
        jj_ok(&fx.work, &["bookmark", "set", "side", "-r", "main"]);
        jj_ok(&fx.work, &["new", "main", "-m", "a side line"]);
        fs::write(fx.work.join("side.txt"), "side\n").expect("write");
        jj_ok(&fx.work, &["bookmark", "set", "side", "-r", "@"]);
        jj_ok(&fx.work, &["new", "old"]);
        let err = commit_as("feat: three", "three.txt", &["--bookmark", "side"]).unwrap_err();
        assert!(err.contains("--yes"), "{err}");
        commit_as("feat: three", "three.txt", &["--bookmark", "side", "--yes"]).expect("moved");
        assert_eq!(
            jj_ok(
                &fx.work,
                &[
                    "log",
                    "-r",
                    "side",
                    "--no-graph",
                    "-T",
                    "description.first_line()"
                ]
            ),
            "feat: three"
        );

        let err = commit_as("feat: four", "four.txt", &["--bookmark", "main"]).unwrap_err();
        assert!(err.contains("a target"), "{err}");
    }

    /// `cycle plan start --ver` sets the opening's version by the
    /// config's bump once the record is written, and `cycle plan commit`
    /// commits the plan.
    #[test]
    fn plan_start_sets_the_version_and_plan_commit_commits_it() {
        let fx = Fixture::new("cycle-plan-ver");
        write_record(&fx);
        let path = fx.work.join(".vc-config.md");
        let mut cfg = fs::read_to_string(&path).expect("read the config");
        cfg.push_str(
            "\n```toml\n[version]\nfile = \"VERSION.md\"\npattern = 'version = '\nbump = \
             \"sh bump.sh {ver}\"\n```\n",
        );
        fs::write(&path, cfg).expect("write the config");
        fs::write(
            fx.work.join("bump.sh"),
            "printf 'version = \"%s\"\\n' \"$1\" > VERSION.md\n",
        )
        .expect("write");
        run(
            &fx.work,
            &[
                "plan",
                "start",
                "--title",
                "feat: a cycle",
                "--ver",
                "1.0.0-0",
            ],
        )
        .expect("planned");
        assert_eq!(
            fs::read_to_string(fx.work.join("VERSION.md")).unwrap(),
            "version = \"1.0.0-0\"\n"
        );
        run(&fx.work, &["plan", "commit", "-s", "b"]).expect("the plan committed");
        assert_eq!(title_of(&fx.work, "@-"), "feat: a cycle opening");
    }

    /// `cycle plan start --ver` is all or nothing: with no `[version]
    /// bump` it is refused before anything is made, and a bump command
    /// that fails puts the record back and deletes the bookmark.
    #[test]
    fn plan_start_ver_makes_nothing_when_the_bump_cannot_run() {
        let fx = Fixture::new("cycle-plan-ver-none");
        write_record(&fx);
        let start = [
            "plan",
            "start",
            "--title",
            "feat: a cycle",
            "--ver",
            "1.0.0",
        ];
        let err = run(&fx.work, &start).unwrap_err();
        assert!(
            err.starts_with("cycle plan start --ver: the config has no `[version] bump`"),
            "{err}"
        );
        assert_eq!(bookmarks_at(&fx.work), "", "no bookmark");
        assert_eq!(fs::read_to_string(fx.work.join("TODO.md")).unwrap(), TODO);

        let fx = Fixture::new("cycle-plan-ver-fails");
        write_record(&fx);
        let path = fx.work.join(".vc-config.md");
        let mut cfg = fs::read_to_string(&path).expect("read the config");
        cfg.push_str("\n```toml\n[version]\nbump = \"false {ver}\"\n```\n");
        fs::write(&path, cfg).expect("write the config");
        let err = run(&fx.work, &start).unwrap_err();
        assert!(
            err.contains("`false 1.0.0` failed") && err.contains("the plan is undone"),
            "{err}"
        );
        assert_eq!(bookmarks_at(&fx.work), "", "the bookmark deleted");
        assert_eq!(fs::read_to_string(fx.work.join("TODO.md")).unwrap(), TODO);
        assert_eq!(fs::read_to_string(fx.work.join("zTOC.md")).unwrap(), ZTOC);
    }

    /// `--ver` without `[version] bump` in the config is refused, and no
    /// rung is marked.
    #[test]
    fn rung_next_ver_needs_the_configs_bump() {
        let fx = planned("cycle-rung-ver");
        commit(&fx.work, &[]).expect("the plan");
        let err = rung(&fx.work, &["next", "--ver", "1.2.3-1"]).unwrap_err();
        assert!(err.contains("no `[version] bump`"), "{err}");
        assert!(
            !ladder_lines(&fx.work)
                .iter()
                .any(|l| l.contains("(current)"))
        );
    }

    /// `cycle status` says where the cycle stands at each stage, and
    /// changes nothing.
    #[test]
    fn status_says_where_the_cycle_stands() {
        let fx = Fixture::new("cycle-status");
        let status = |fx: &Fixture| match params(&fx.work, &["status"]).expect("params") {
            CycleParams::Status(r) => super::status(&r).expect("status").join("\n"),
            _ => panic!("expected Status"),
        };
        fs::remove_file(fx.work.join("TODO.md")).expect("remove init's");
        assert!(status(&fx).contains("no TODO.md"), "{}", status(&fx));
        write_record(&fx);
        let s = status(&fx);
        assert!(
            s.contains("cycle: none in progress") && s.contains("cycle plan"),
            "{s}"
        );

        let fx = planned("cycle-status-planned");
        let s = status(&fx);
        assert!(s.contains("cycle: feat: a cycle\n"), "{s}");
        assert!(
            s.contains("bookmark: feat-a-cycle, on the work @, nothing committed"),
            "{s}"
        );
        assert!(s.contains("commits: 0 on the branch, 0 not pushed"), "{s}");
        assert!(s.contains("note: the ladder in zTOC.md differs"), "{s}");
        assert!(
            s.contains("next: `cycle plan commit -s <SCOPE>` commits the plan"),
            "{s}"
        );

        commit(&fx.work, &[]).expect("the opening");
        let s = status(&fx);
        assert!(s.contains("commits: 1 on the branch, 1 not pushed"), "{s}");
        assert!(s.contains("  - feat: one rung\n"), "{s}");
        assert!(!s.contains("(current)"), "{s}");
        assert!(s.contains("work @: empty"), "{s}");
        assert!(!s.contains("differs"), "{s}");
        assert!(
            s.contains("next: `rung next` starts 'feat: one rung'"),
            "{s}"
        );

        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        commit(&fx.work, &["--title", "feat: one rung", "--body", "b"]).expect("the rung");
        assert!(status(&fx).contains("next: `rung next` starts 'feat: a cycle closing'"));
        jj_ok(&fx.work, &["commit", "-m", "wip: by hand"]);
        let s = status(&fx);
        assert!(
            s.contains("ladder: not made") && s.contains("'wip: by hand'"),
            "{s}"
        );
        jj_ok(&fx.work, &["abandon", "@-"]);

        run(&fx.work, &["close", "--linear"]).expect("closed");
        let s = status(&fx);
        assert!(
            s.contains("closed: 'feat: a cycle'") && s.contains("cycle land"),
            "{s}"
        );
        run(&fx.work, &["land", "main"]).expect("landed");
        let s = status(&fx);
        assert!(
            s.contains("closed: 'feat: a cycle', landed on 'main'"),
            "{s}"
        );
        assert!(
            s.contains("next: `cycle push -s <SCOPE>` publishes 'main'"),
            "{s}"
        );
    }

    /// Set the work config's `[version]` check on `VERSION.md`.
    fn set_version_check(fx: &Fixture) {
        let path = fx.work.join(".vc-config.md");
        let mut cfg = fs::read_to_string(&path).expect("read the config");
        cfg.push_str("\n```toml\n[version]\nfile = \"VERSION.md\"\npattern = 'version = '\n```\n");
        fs::write(&path, cfg).expect("write the config");
    }

    /// With `[version]` in the config a commit that changes no line of
    /// the version's file starting with the pattern is refused before
    /// anything is written, `--same-version` commits anyway, and the
    /// tool never writes the line.
    #[test]
    fn a_commit_with_the_version_unchanged_is_refused() {
        let fx = planned("cycle-version");
        set_version_check(&fx);
        fs::write(
            fx.work.join("VERSION.md"),
            "name = \"x\"\nversion = \"1.0.0-0\"\n",
        )
        .expect("write");
        commit(&fx.work, &[]).expect("the opening sets the version");
        let ladder = fs::read_to_string(fx.work.join("zTOC.md")).unwrap();

        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        let rung = ["--title", "feat: one rung", "--body", "b"];
        let err = commit(&fx.work, &rung).unwrap_err();
        assert!(
            err.contains("VERSION.md") && err.contains("--same-version"),
            "{err}"
        );
        assert_eq!(fs::read_to_string(fx.work.join("zTOC.md")).unwrap(), ladder);
        // A line that only holds the pattern is not the version's.
        fs::write(
            fx.work.join("VERSION.md"),
            "name = \"y\"\nversion = \"1.0.0-0\"\ndep = { version = \"2\" }\n",
        )
        .expect("write");
        assert!(
            commit(&fx.work, &rung).is_err(),
            "the version's line is as it was"
        );

        fs::write(
            fx.work.join("VERSION.md"),
            "name = \"y\"\nversion = \"1.0.0-1\"\n",
        )
        .expect("write");
        commit(&fx.work, &rung).expect("the version is bumped");
        assert_eq!(title_of(&fx.work, "@-"), "feat: one rung");

        let err = run(&fx.work, &["close", "--linear"]).unwrap_err();
        assert!(err.contains("--same-version"), "{err}");
        assert_eq!(title_of(&fx.work, "@-"), "feat: one rung", "nothing closed");
        run(&fx.work, &["close", "--linear", "--same-version"]).expect("closed as it is");
        assert_eq!(title_of(&fx.work, "@-"), "feat: a cycle closing");
        assert_eq!(
            file_at(&fx.work, "@-", "VERSION.md"),
            "name = \"y\"\nversion = \"1.0.0-1\"",
            "the tool never writes the version"
        );
    }

    /// A target not where its `origin` is names the way on by how the
    /// two stand: ahead is a push, behind a sync, diverged neither.
    #[test]
    fn a_target_off_its_origin_names_the_way_on() {
        let fx = Fixture::new("cycle-land-off-origin");
        let checks = |fx: &Fixture| {
            land_checks(&fx.work, "main", None, false)
                .unwrap_err()
                .to_string()
        };
        let base = jj_ok(
            &fx.work,
            &["log", "-r", "main", "--no-graph", "-T", "commit_id"],
        );
        // Ahead: a commit on main, not pushed.
        fs::write(fx.work.join("ahead.txt"), "ahead\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "ahead"]);
        jj_ok(&fx.work, &["bookmark", "set", "main", "-r", "@-"]);
        // The work @- elsewhere, as in a workspace whose cycle is on top.
        jj_ok(&fx.work, &["new", "main"]);
        fs::write(fx.work.join("other.txt"), "other\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "other"]);
        let err = checks(&fx);
        assert!(
            err.contains("ahead") && err.contains("vc-x1 push -s b -r main"),
            "{err}"
        );
        assert!(!err.contains("sync"), "{err}");
        // The command named works where @- is not the target.
        crate::push::push(
            &mut test_ctx(),
            &crate::push::PushParams {
                revision: Some("main".to_string()),
                bookmark: None,
                repo: Some(fx.work.clone()),
                scope: RepoScope::Sides(crate::options_flags::scope::Scope(vec![
                    crate::options_flags::scope::Side::Work,
                    crate::options_flags::scope::Side::Bot,
                ])),
                step: false,
                dry_run: false,
                yes: true,
                forced: crate::common::ForcedPush::Refuse,
            },
        )
        .expect("the push named publishes main");
        land_checks(&fx.work, "main", None, false).expect("main is where origin is");
        // Behind: main set back to where it was.
        jj_ok(
            &fx.work,
            &["bookmark", "set", "main", "-r", &base, "--allow-backwards"],
        );
        let err = checks(&fx);
        assert!(
            err.contains("behind") && err.contains("vc-x1 sync -s b"),
            "{err}"
        );
        // Diverged: a commit of its own beside the published one.
        jj_ok(&fx.work, &["new", &base]);
        fs::write(fx.work.join("mine.txt"), "mine\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "mine"]);
        jj_ok(
            &fx.work,
            &["bookmark", "set", "main", "-r", "@-", "--allow-backwards"],
        );
        let err = checks(&fx);
        assert!(
            err.contains("diverged") && !err.contains("vc-x1 push"),
            "{err}"
        );
    }

    /// A Land with no cycle-record finds the one other bookmark, and with
    /// several names `--bookmark`, the flag `cycle land` has.
    #[test]
    fn several_bookmarks_name_the_flag_cycle_has() {
        let fx = Fixture::new("cycle-land-several");
        for name in ["one", "two"] {
            jj_ok(&fx.work, &["new", "main"]);
            fs::write(fx.work.join(format!("{name}.txt")), "x\n").expect("write");
            jj_ok(&fx.work, &["commit", "-m", name]);
            jj_ok(&fx.work, &["bookmark", "create", name, "-r", "@-"]);
        }
        let err = run(&fx.work, &["land", "main"]).unwrap_err();
        assert!(
            err.starts_with("cycle land:") && err.contains("--bookmark <NAME>"),
            "{err}"
        );
        assert!(!err.contains("--source"), "{err}");
        run(&fx.work, &["land", "main", "--bookmark", "two"]).expect("landed");
        assert_eq!(title_of(&fx.work, "main"), "two");
    }

    /// `cycle close` is a trapezoid by default, its target named, or
    /// `--linear` or `--squash`: the retired shape flags are refused,
    /// `--squash` beside a trapezoid's, a dead end's, or a heading's flag
    /// too, `--default-body` needs `--squash`, and a ladder's closing is
    /// refused with no opening, naming `--squash`. Over the opening
    /// alone it is the closing, its own commit above it.
    #[test]
    fn close_is_a_ladders_closing() {
        let fx = planned("cycle-close-shape");
        for gone in ["--single-step", "--multi-step", "--trapezoid"] {
            let err = parse(&["vc-x1", "cycle", "close", "-s", "b", gone]).unwrap_err();
            assert!(err.contains("unexpected argument"), "{gone}: {err}");
        }
        let err = parse(&["vc-x1", "cycle", "squash", "-s", "b"]).unwrap_err();
        assert!(err.contains("unrecognized subcommand"), "{err}");
        let err = params(&fx.work, &["close"]).unwrap_err();
        assert!(
            err.contains("a trapezoid by default") && err.contains("cycle close <TARGET>"),
            "{err}"
        );
        for clash in [
            vec!["close", "--squash", "--body", "b", "--dead-end"],
            vec!["close", "main", "--squash", "--body", "b"],
            vec!["close", "main", "--linear"],
        ] {
            let err = params(&fx.work, &clash).unwrap_err();
            assert!(err.contains("cannot be used with"), "{clash:?}: {err}");
        }
        let err = params(&fx.work, &["close", "--linear", "--default-body"]).unwrap_err();
        assert!(err.contains("--squash"), "{err}");
        let err = params(&fx.work, &["close", "--linear", "--land"]).unwrap_err();
        assert!(
            err.contains("a bare --land lands on the trapezoid's TARGET"),
            "{err}"
        );
        let err = params(&fx.work, &["close", "--squash", "-s", "w", "--body", "b"]).unwrap_err();
        assert!(err.contains("-s b"), "{err}");

        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        let err = run(&fx.work, &["close", "--linear"]).unwrap_err();
        assert!(
            err.contains("no opening") && err.contains("cycle close --squash"),
            "{err}"
        );

        // One commit, the opening: the closing is its own commit above it.
        commit(&fx.work, &[]).expect("the opening");
        let opening = jj_ok(
            &fx.work,
            &["log", "-r", "@-", "--no-graph", "-T", "commit_id"],
        );
        fs::write(fx.work.join("more.txt"), "more\n").expect("write");
        run(&fx.work, &["close", "--linear", "--dry-run"]).expect("a dry run");
        assert_eq!(
            title_of(&fx.work, "@-"),
            "feat: a cycle opening",
            "nothing made"
        );
        // The ladder check refuses a closing over an unmade work rung, so
        // this plan's ladder is the bookends alone.
        let todo = fs::read_to_string(fx.work.join("TODO.md")).unwrap();
        let bookends = todo.replace("##### feat: one rung\n\nIts intent.\n\n", "");
        fs::write(fx.work.join("TODO.md"), bookends).expect("write");
        run(&fx.work, &["close", "--linear"]).expect("closed");
        assert_eq!(
            above_main(&fx.work),
            ["feat: a cycle opening", "feat: a cycle closing"],
            "the opening kept, the closing its own commit"
        );
        assert_eq!(parents(&fx.work, "@-"), [opening]);
    }

    /// `cycle close --squash` makes a ladder one commit: the rungs squashed into
    /// the opening, whose change id is kept, under the bare title, the
    /// record closed in it, and the agent side re-pointed, each agent
    /// commit naming the one work commit and it naming every one.
    #[test]
    fn a_ladder_squashes_into_one_commit() {
        let fx = planned("cycle-squash");
        commit(&fx.work, &[]).expect("the opening");
        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        commit(&fx.work, &["--title", "feat: one rung", "--body", "b"]).expect("the rung");
        let opening = jj_ok(
            &fx.work,
            &[
                "log",
                "-r",
                "@--",
                "--no-graph",
                "-T",
                "change_id.short(12)",
            ],
        );
        let agents: Vec<String> = ["@--", "@-"]
            .iter()
            .flat_map(|rev| crate::desc_helpers::extract_ochids(&description(&fx.work, rev)))
            .collect();
        assert_eq!(agents.len(), 2, "{agents:?}");

        run(
            &fx.work,
            &[
                "close",
                "--squash",
                "--body",
                "Why.\n\nSolution: how.",
                "--dry-run",
            ],
        )
        .expect("a dry run");
        assert_eq!(title_of(&fx.work, "@-"), "feat: one rung", "nothing made");

        run(
            &fx.work,
            &["close", "--squash", "--body", "Why.\n\nSolution: how."],
        )
        .expect("squashed");
        assert_eq!(above_main(&fx.work), ["feat: a cycle"]);
        assert_eq!(
            jj_ok(
                &fx.work,
                &["log", "-r", "@-", "--no-graph", "-T", "change_id.short(12)"]
            ),
            opening,
            "the opening's change id is kept"
        );
        let desc = description(&fx.work, "@-");
        assert!(
            desc.starts_with("feat: a cycle\n\nWhy.\n\nSolution: how.\n"),
            "{desc}"
        );
        assert_eq!(crate::desc_helpers::extract_ochids(&desc), agents, "{desc}");
        for agent in &agents {
            let named = crate::desc_helpers::extract_ochids(&description(&fx.bot, agent));
            assert_eq!(named, [opening.as_str()], "{agent} re-pointed");
        }
        let todo = file_at(&fx.work, "@-", "TODO.md");
        assert!(
            todo.contains("_No cycle currently in progress._") && !todo.contains("#### Rungs"),
            "{todo}"
        );
        assert!(file_at(&fx.work, "@-", "zTOC.md").contains("- 0 (done) feat: a cycle"));
        assert!(
            fx.work.join("work.txt").is_file(),
            "the rung's work is in it"
        );
    }

    /// `close --squash` on a cycle closed and not landed converts it: the
    /// trapezoid's closing squashed in with the rung, the opening's change
    /// id kept, every agent commit re-pointed to it, the closing's among
    /// them, and the closed record a single-step's. A target that moved
    /// since is refused, nothing rewritten.
    #[test]
    fn a_closed_draft_squashes_into_one_commit() {
        let fx = planned("cycle-squash-closed");
        commit(&fx.work, &[]).expect("the opening");
        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        commit(&fx.work, &["--title", "feat: one rung", "--body", "b"]).expect("the rung");
        run(&fx.work, &["close", "main"]).expect("closed as a trapezoid");
        assert_eq!(title_of(&fx.work, "@-"), "feat: a cycle closing");
        let opening = jj_ok(
            &fx.work,
            &[
                "log",
                "-r",
                "roots(main..@-)",
                "--no-graph",
                "-T",
                "change_id.short(12)",
            ],
        );
        let agents: Vec<String> = jj_ok(
            &fx.work,
            &[
                "log",
                "-r",
                "main..@-",
                "--no-graph",
                "-T",
                "commit_id ++ \"\\n\"",
            ],
        )
        .lines()
        .flat_map(|cid| crate::desc_helpers::extract_ochids(&description(&fx.work, cid)))
        .collect();
        assert_eq!(
            agents.len(),
            3,
            "the opening's, the rung's, the closing's: {agents:?}"
        );

        run(
            &fx.work,
            &["close", "--squash", "--body", "Why.\n\nHow.", "--dry-run"],
        )
        .expect("a dry run");
        assert_eq!(
            title_of(&fx.work, "@-"),
            "feat: a cycle closing",
            "nothing made"
        );

        run(&fx.work, &["close", "--squash", "--body", "Why.\n\nHow."]).expect("converted");
        assert_eq!(above_main(&fx.work), ["feat: a cycle"]);
        assert_eq!(parents(&fx.work, "@-").len(), 1, "no merge left");
        assert_eq!(
            jj_ok(
                &fx.work,
                &["log", "-r", "@-", "--no-graph", "-T", "change_id.short(12)"]
            ),
            opening,
            "the opening's change id is kept"
        );
        let desc = description(&fx.work, "@-");
        assert!(
            desc.starts_with("feat: a cycle\n\nWhy.\n\nHow.\n"),
            "{desc}"
        );
        let mut named = crate::desc_helpers::extract_ochids(&desc);
        named.sort();
        let mut expected = agents.clone();
        expected.sort();
        assert_eq!(named, expected, "{desc}");
        for agent in &agents {
            let points = crate::desc_helpers::extract_ochids(&description(&fx.bot, agent));
            assert_eq!(points, [opening.as_str()], "{agent} re-pointed");
        }
        let todo = file_at(&fx.work, "@-", "TODO.md");
        assert!(
            todo.contains("### feat: a cycle") && !todo.contains("#### Rungs"),
            "{todo}"
        );
        let ladder = file_at(&fx.work, "@-", "zTOC.md");
        assert!(ladder.contains("- 0 (done) feat: a cycle\n"), "{ladder}");
        assert!(!ladder.contains("opening"), "{ladder}");
        assert_eq!(
            jj_ok(
                &fx.work,
                &[
                    "log",
                    "-r",
                    "feat-a-cycle",
                    "--no-graph",
                    "-T",
                    "change_id.short(12)"
                ]
            ),
            opening,
            "the bookmark is on the one commit"
        );
        assert!(jj::is_empty(&fx.work, "@").expect("@"), "@ empty on it");
    }

    /// A trapezoid whose target moved after the cycle opened is not
    /// converted, the one commit then a tree no rung validated.
    #[test]
    fn a_closed_draft_on_a_moved_target_is_not_converted() {
        let fx = planned("cycle-squash-moved");
        commit(&fx.work, &[]).expect("the opening");
        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        commit(&fx.work, &["--title", "feat: one rung", "--body", "b"]).expect("the rung");
        jj_ok(&fx.work, &["new", "main", "-m", "moved"]);
        fs::write(fx.work.join("moved.txt"), "moved\n").expect("write");
        jj_ok(&fx.work, &["bookmark", "set", "main", "-r", "@"]);
        jj_ok(&fx.work, &["new", "feat-a-cycle"]);
        run(&fx.work, &["close", "main"]).expect("closed onto the moved main");
        let closing = jj_ok(
            &fx.work,
            &["log", "-r", "@-", "--no-graph", "-T", "commit_id"],
        );
        let err = run(&fx.work, &["close", "--squash", "--body", "b"]).unwrap_err();
        assert!(err.contains("moved after the cycle opened"), "{err}");
        assert_eq!(
            jj_ok(
                &fx.work,
                &["log", "-r", "@-", "--no-graph", "-T", "commit_id"]
            ),
            closing,
            "nothing rewritten"
        );
    }

    /// An agent commit `main@origin` reaches is rewritten only with
    /// `--yes`, nothing rewritten without it.
    #[test]
    fn squash_rewrites_a_published_agent_commit_only_with_yes() {
        let fx = planned("cycle-squash-yes");
        commit(&fx.work, &[]).expect("the opening");
        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        commit(&fx.work, &["--title", "feat: one rung", "--body", "b"]).expect("the rung");
        run(&fx.work, &["push", "-s", "b"]).expect("the draft pushed");
        let body = ["close", "--squash", "--body", "Why.\n\nSolution: how."];
        let err = run(&fx.work, &body).unwrap_err();
        assert!(
            err.contains("main@origin") && err.contains("--yes"),
            "{err}"
        );
        assert_eq!(
            title_of(&fx.work, "@-"),
            "feat: one rung",
            "nothing rewritten"
        );
        let mut yes = body.to_vec();
        yes.push("--yes");
        run(&fx.work, &yes).expect("squashed");
        assert_eq!(above_main(&fx.work), ["feat: a cycle"]);
    }

    /// `cycle push` publishes what the cycle's state calls for: the
    /// draft's bookmark, then the target it landed on here with the
    /// draft's bookmark deleted on origin, then nothing.
    #[test]
    fn push_publishes_what_the_state_calls_for() {
        let fx = planned("cycle-push-stage");
        let on_origin = |name: &str| {
            jj::remote_bookmark_targets(&fx.work, "origin")
                .expect("remote")
                .contains_key(name)
        };
        let err = params(&fx.work, &["push"]).unwrap_err();
        assert!(err.contains("--scope"), "{err}");
        let err = params(&fx.work, &["push", "-s", "w"]).unwrap_err();
        assert!(err.contains("-s b"), "{err}");
        let err = params(&fx.work, &["push", "-s", "b", "--title", "t"]).unwrap_err();
        assert!(err.contains("unexpected argument '--title'"), "{err}");

        commit(&fx.work, &[]).expect("the opening");
        run(&fx.work, &["push", "-s", "b"]).expect("the draft pushed");
        assert!(on_origin("feat-a-cycle"), "the draft's bookmark");

        let todo = fs::read_to_string(fx.work.join("TODO.md")).unwrap();
        fs::write(
            fx.work.join("TODO.md"),
            todo.replace("##### feat: one rung\n\nIts intent.\n\n", ""),
        )
        .expect("write");
        run(&fx.work, &["close", "--linear"]).expect("closed");
        run(&fx.work, &["push", "-s", "b"]).expect("the closed draft pushed");
        run(&fx.work, &["land", "main"]).expect("landed here");
        assert_ne!(
            cid(&fx.work, "main"),
            cid(&fx.work, "main@origin"),
            "not yet published"
        );
        assert!(
            on_origin("feat-a-cycle"),
            "the draft's bookmark still on origin"
        );

        run(&fx.work, &["push", "-s", "b"]).expect("the target published");
        assert_eq!(cid(&fx.work, "main"), cid(&fx.work, "main@origin"));
        assert!(
            !on_origin("feat-a-cycle"),
            "the draft's bookmark deleted on origin"
        );
        run(&fx.work, &["push", "-s", "b"]).expect("nothing left, no error");
    }

    /// `cycle close --squash` needs the one commit's body, or the pointer, and
    /// a ladder's closing has its own.
    #[test]
    fn squash_needs_a_body() {
        let fx = planned("cycle-close-lacks");
        params(&fx.work, &["close", "--linear"]).expect("a ladder's closing has a body");
        let err = params(&fx.work, &["close", "--squash"]).unwrap_err();
        assert!(
            err.contains("--body") && err.contains("--default-body"),
            "{err}"
        );
        params(&fx.work, &["close", "--squash", "--default-body"]).expect("the pointer");
        let err = params(
            &fx.work,
            &["close", "--squash", "--body", "b", "--default-body"],
        )
        .unwrap_err();
        assert!(err.contains("cannot be used with"), "{err}");
    }

    /// A cycle title is a Conventional Commit of 50 characters at most,
    /// and each fault is named.
    #[test]
    fn a_title_is_checked() {
        assert!(title_problems("feat: x").is_empty());
        assert!(title_problems("fix(push)!: name -r").is_empty());
        let p = title_problems("Feat y to check TODO.md updating");
        assert_eq!(p.len(), 1, "{p:?}");
        assert!(p[0].contains("Conventional Commit"), "{p:?}");
        assert_eq!(title_problems("feat: ").len(), 1);
        assert_eq!(title_problems("FEAT: x").len(), 1);
        let long = "feat: a title that runs on well past the fifty it keeps to";
        let p = title_problems(long);
        assert!(p.len() == 1 && p[0].contains("over the 50"), "{p:?}");
    }
}

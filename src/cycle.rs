//! The `cycle` subcommand: a cycle's acts, each a stage named by a
//! flag. The bookmark is [`crate::open`]'s to make, the commits
//! [`crate::commit`]'s, and the Land [`crate::land`]'s, none of which
//! is a command of its own for a cycle.
//!
//! - `--open` opens a cycle: the bookmark first, on the work `@`, as
//!   [`crate::open`] makes it, then the cycle's record, the edits
//!   [`record`] plans. Everything is planned and checked before
//!   anything is made, so a refusal leaves the workspace as it was.
//! - `--ladder` makes `zTOC.md`'s ladder from the rung headings in
//!   `TODO.md`, each marker computed from the commits on the cycle's
//!   branch, so a rung is written once and no marker is typed.
//! - `--commit`, `--close`, and `--land` make a rung, the closing,
//!   and the Land, by `commit`'s and `land`'s own code, and chain in
//!   that order. `--status` reads where the cycle stands.
//! - The record is the agent-files' shape and not the work's, so the
//!   tool writes it without knowing the work. The version is the
//!   project's, and no stage bumps it.
//! - A stage is a flag and not a subcommand, the form every `vc-x1`
//!   command has.

pub(crate) mod record;

use std::path::PathBuf;

use clap::Args;
use log::{debug, info, warn};

use crate::context::Context;
use crate::jj;
use crate::open::OpenParams;
use crate::options_flags::dry_run::DryRunFlag;
use crate::options_flags::scope::{RepoScope, repo_scope_parser};
use crate::subcommand::SubcommandRunner;

/// Run a cycle's stages.
#[derive(Args, Debug)]
#[group(id = "stage", required = true, multiple = true, args = ["open", "ladder", "commit", "close", "land", "status", "push"])]
pub struct CycleArgs {
    /// Open a cycle: make its bookmark on the work `@`, then write its
    /// record, `TODO.md`'s `## In Progress` scaffold and `zTOC.md`'s
    /// ladder
    #[arg(long, help_heading = "Stages", conflicts_with_all = ["ladder", "commit", "close", "land", "push"])]
    pub open: bool,

    /// Make `zTOC.md`'s ladder from the rung headings under `####
    /// Rungs` in `TODO.md`, each marker computed from the cycle's
    /// commits
    #[arg(long, help_heading = "Stages", conflicts_with_all = ["commit", "close", "land", "title", "push"])]
    pub ladder: bool,

    /// Commit the working copies as a rung of the cycle in progress,
    /// the ladder rewritten into it: the cycle's first commit is its
    /// opening, its title and body defaulted, and a work rung gives
    /// both
    #[arg(long, help_heading = "Stages", conflicts_with_all = ["close", "land"])]
    pub commit: bool,

    /// Close the cycle in progress from the work `@`, each REV listed
    /// another head joined, a parent of the closing: a branch with no
    /// commit or one is a single-step cycle, its closing squashed into
    /// its one commit under the bare cycle title, and a longer one gets
    /// the closing commit, titled and bodied by default. The record
    /// moves to `## Closed` in it
    #[arg(
        long,
        help_heading = "Stages",
        value_name = "REV",
        num_args = 0..,
        value_delimiter = ','
    )]
    pub close: Option<Vec<String>>,

    /// Land the closed cycle on BOOKMARK, the destination: commit the
    /// agent `@` when the cycle's commits name it, fast-forward the
    /// destination to the cycle's last commit, and delete the cycle's
    /// bookmark. After `--close` it lands the closing just made
    #[arg(long, help_heading = "Stages", value_name = "BOOKMARK")]
    pub land: Option<String>,

    /// Say where the cycle stands: its title, its bookmark, the ladder
    /// with its markers, the commits not yet pushed, and the stages open
    /// from here. It reads, and changes nothing
    #[arg(
        long,
        help_heading = "Stages",
        conflicts_with_all = ["open", "ladder", "commit", "close", "land", "title", "push"]
    )]
    pub status: bool,

    /// With `--open`, the bare cycle title, a Conventional Commit: the
    /// bookmark is its slug, and the record is written under it. With
    /// `--commit`, the rung's title, a heading under `#### Rungs`, and
    /// with `--close` the closing's, as written [default: the opening's
    /// for a cycle's first commit, and for a closing the cycle title,
    /// plus " closing" on a ladder]
    #[arg(long, value_name = "STR")]
    pub title: Option<String>,

    /// The commit's body, the problem and its solution, or @PATH to
    /// read it from a file [default: `[opening] body` for a cycle's
    /// first commit, and `[closing] body` for a ladder's closing]
    #[arg(long, value_name = "STR|@PATH")]
    pub body: Option<String>,

    /// With `--close`, make the closing a trapezoid, a merge whose
    /// first parent is BOOKMARK, the destination, read where it is now
    /// [default: linear, the heads alone]
    #[arg(
        long,
        value_name = "BOOKMARK",
        requires = "close",
        help_heading = "Close"
    )]
    pub trapezoid: Option<String>,

    /// With `--close`, give a single-step cycle's one commit the
    /// default body, `[closing] body`, a pointer at the record, in
    /// place of the `--body` it otherwise requires
    #[arg(
        long,
        requires = "close",
        conflicts_with = "body",
        help_heading = "Close"
    )]
    pub default_body: bool,

    /// With `--close`, close the cycle as a dead end: a closing commit,
    /// with `--multi-step`, its title ending " dead-end closing"
    #[arg(long, requires = "close", help_heading = "Close")]
    pub dead_end: bool,

    /// With `--close`, required or `--multi-step`: the cycle is one
    /// commit, made now from the work `@` when the branch has none, or
    /// its opening with the closing squashed into it when it has one
    #[arg(
        long,
        requires = "close",
        conflicts_with = "multi_step",
        help_heading = "Close"
    )]
    pub single_step: bool,

    /// With `--close`, required or `--single-step`: the cycle is a
    /// ladder, and the closing is its own commit above the opening and
    /// any work rungs
    #[arg(long, requires = "close", help_heading = "Close")]
    pub multi_step: bool,

    /// The cycle's bookmark, in place of the title's slug: the name
    /// `--open` makes, and the one another stage reads the cycle's
    /// commits from and advances
    #[arg(long, value_name = "NAME")]
    pub bookmark: Option<String>,

    /// With `--open`, make the bookmark alone and write no record
    #[arg(long, requires = "open", help_heading = "Open")]
    pub no_record: bool,

    /// With `--land`, join a destination that moved by a merge commit,
    /// the destination its first parent and the cycle's last commit its
    /// second, made and then landed, a tree no rung validated [default:
    /// a destination that moved is refused]
    #[arg(
        long,
        requires = "land",
        conflicts_with = "close",
        help_heading = "Land"
    )]
    pub merge: bool,

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
    /// between the destination and the cycle's last commit, a stack of
    /// cycles landing together [default: named as merged and kept]
    #[arg(long, requires = "land", help_heading = "Land")]
    pub delete_intermediate_bookmarks: bool,

    /// Which repos, required with `--commit`, `--close`, and `--land`,
    /// as on every command that writes: `b|both` a dual workspace's
    /// pair, `w|work` the work half with the agent half left to push,
    /// not with `--land`, and `p|por` a plain repo. The other stages
    /// take none
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = repo_scope_parser(&["work", "both", "por"]),
        required_unless_present_any = ["open", "ladder", "status"],
        help = "Which repos, required with --commit, --close, --land, and --push: `w|work`, \
                `b|both`, or `p|por`"
    )]
    pub scope: Option<RepoScope>,

    /// Publish. Alone, a stage: what the cycle's state calls for, its
    /// bookmark while it is a draft, and once landed here the target,
    /// the cycle's bookmark deleted on `origin`. With `--commit` or
    /// `--close`, the cycle's bookmark, a draft push, and with `--land`
    /// the target. Nothing is pushed without it
    #[arg(long, help_heading = "Stages")]
    pub push: bool,

    /// With `--commit` or `--close`, commit although the version's line
    /// is unchanged: with `[version] file` and `pattern` in the config,
    /// a commit that changes no line of the file starting with the
    /// pattern is otherwise refused
    #[arg(long, help_heading = "Commit")]
    pub same_version: bool,

    /// Where: a path in the workspace, its root or either repo, or a
    /// plain repo [default: the current directory]
    #[arg(short = 'R', long, value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Say what each stage would do, and do none of it
    #[command(flatten)]
    pub dry_run: DryRunFlag,
}

/// `--open`'s inputs: the bookmark's, as [`crate::open::open`] takes them, and the
/// title the record is written under, `None` when no record is.
#[derive(Debug)]
pub struct OpenCycle {
    pub open: OpenParams,
    pub record: Option<String>,
}

/// The cycle a stage after `--open` works on: its work repo and the
/// bookmark named, the title's slug when none is.
#[derive(Debug)]
pub struct Running {
    pub work: PathBuf,
    pub bookmark: Option<String>,
    pub dry_run: bool,
}

/// `--commit`'s inputs: the cycle, the repos the commit is made in,
/// and the title and body given.
#[derive(Debug)]
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

/// `--close`'s inputs: the commit's, as a rung's, and the closing's
/// shape.
#[derive(Debug)]
pub struct CloseCycle {
    pub rung: CommitRung,
    /// The other heads joined, each a revision of one commit.
    pub heads: Vec<String>,
    /// The destination, the closing's first parent, for a trapezoid.
    pub trapezoid: Option<String>,
    pub default_body: bool,
    pub dead_end: bool,
    /// The shape named: one commit, or a ladder with its own closing.
    pub single_step: bool,
    /// The Land that follows the closing, in the same invocation.
    pub land: Option<LandCycle>,
}

/// `--land`'s inputs: the cycle's work repo, the agent repo of a dual
/// workspace, and the destination.
#[derive(Debug)]
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

/// `--push` alone: the cycle and the repos a push publishes.
#[derive(Debug)]
pub struct PushCycle {
    pub running: Running,
    pub agent: Option<PathBuf>,
}

/// Inputs to the cycle op, flat, owned, clap-free, a stage each.
#[derive(Debug)]
pub enum CycleParams {
    Open(OpenCycle),
    Ladder(Running),
    Commit(CommitRung),
    Close(CloseCycle),
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
/// made now, by `-s`, required: refused when it is left out, and when
/// it asks for a pair where there is none or a plain commit where
/// there is one.
fn commit_scope(
    scope: Option<&RepoScope>,
    agent: Option<PathBuf>,
    stage: &str,
) -> Result<Option<(PathBuf, bool)>, String> {
    match (scope, agent) {
        (None, _) => Err(format!(
            "cycle {stage}: -s is required, which repos: `b|both`, `w|work`, or `p|por`"
        )),
        (Some(RepoScope::Por), None) => Ok(None),
        (Some(RepoScope::Por), Some(_)) => Err(format!(
            "cycle {stage} -s p: this is a dual workspace, whose rung is a pair: -s b or -s w"
        )),
        (Some(RepoScope::Sides(_)), None) => Err(format!(
            "cycle {stage}: this is a plain repo, with no agent repo to pair with: -s p"
        )),
        (Some(RepoScope::Sides(sides)), Some(agent)) => Ok(Some((agent, sides.is_both()))),
    }
}

/// What `--close` lacks, every piece at once, so one refusal names all
/// of it: the shape, and with `--single-step` the body its one commit
/// needs. `None` when nothing is.
fn close_missing(a: &CycleArgs) -> Option<String> {
    let bodied = a.body.is_some() || a.default_body;
    let mut missing: Vec<&str> = Vec::new();
    if !a.single_step && !a.multi_step {
        missing.push(
            "the cycle's shape, --single-step, its one commit, or --multi-step, a ladder whose \
             closing is its own commit, which the branch's commits check and, with one commit, \
             the opening, it decides",
        );
        if !bodied {
            missing.push("with --single-step, --body or --default-body");
        }
    } else if a.single_step && !bodied {
        missing.push(
            "--body, the problem and solution of the one commit, which holds the work, or \
             --default-body for the pointer at the record",
        );
    }
    match missing.as_slice() {
        [] => None,
        [one] => Some(format!("cycle --close: it needs {one}")),
        many => Some(format!("cycle --close: it needs {}", many.join("; and "))),
    }
}

impl CycleArgs {
    /// The cycle a stage after `--open` works on.
    fn running(&self, work: PathBuf) -> Running {
        Running {
            work,
            bookmark: self.bookmark.clone(),
            dry_run: self.dry_run.value,
        }
    }

    /// `--open`'s params.
    fn open_params(&self, work: PathBuf, agent: Option<PathBuf>) -> Result<CycleParams, String> {
        if self.title.is_none() && self.bookmark.is_none() {
            return Err("cycle --open: --title or --bookmark names the bookmark".into());
        }
        let bookmark = crate::open::bookmark_name(self.bookmark.as_deref(), self.title.as_deref())?;
        if let Some(t) = &self.title {
            crate::commit::refuse_suffixed(t, "cycle --open --title")
                .map_err(|e| e.replacen("commit ", "", 1))?;
        }
        Ok(CycleParams::Open(OpenCycle {
            open: OpenParams {
                work,
                agent,
                bookmark,
                dry_run: self.dry_run.value,
            },
            record: match (&self.title, self.no_record) {
                (Some(t), false) => Some(t.trim().to_string()),
                _ => None,
            },
        }))
    }

    /// `--land`'s params, `None` when it is not asked for. With `alone`
    /// the title and body are the merge commit's, and after `--close`
    /// they are the closing's.
    fn land_params(
        &self,
        work: PathBuf,
        agent: Option<PathBuf>,
        alone: bool,
    ) -> Result<Option<LandCycle>, String> {
        let Some(dest) = &self.land else {
            return Ok(None);
        };
        if matches!(&self.scope, Some(RepoScope::Sides(s)) if !s.is_both()) {
            return Err(
                "cycle --land -s w: a work-only Land would publish trailers naming agent commits \
                 its agent half may not have published: -s b"
                    .into(),
            );
        }
        commit_scope(self.scope.as_ref(), agent.clone(), "--land")?;
        if alone && self.same_version {
            return Err("cycle --land: it takes no --same-version, a Land bumping nothing".into());
        }
        if alone && (self.title.is_some() || self.body.is_some()) && !self.merge {
            return Err(
                "cycle --land: a Land makes no commit of its own, so --title and --body are \
                 --merge's, the merge commit's"
                    .into(),
            );
        }
        let body = match (&self.body, alone) {
            (Some(b), true) => Some(crate::desc_helpers::read_body(b, "cycle --land")?),
            _ => None,
        };
        Ok(Some(LandCycle {
            running: self.running(work),
            agent,
            dest: dest.clone(),
            merge: self.merge,
            skip: self.skip,
            delete_intermediate: self.delete_intermediate_bookmarks,
            push: self.push,
            title: self.title.clone().filter(|_| alone),
            body,
        }))
    }

    /// A commit's params, `--commit`'s and `--close`'s alike.
    fn rung(
        &self,
        work: PathBuf,
        agent: Option<PathBuf>,
        stage: &str,
    ) -> Result<CommitRung, String> {
        let body = match &self.body {
            Some(b) => Some(crate::desc_helpers::read_body(
                b,
                &format!("cycle {stage}"),
            )?),
            None => None,
        };
        if self.title.as_deref().is_some_and(|t| t.trim().is_empty())
            || body.as_deref().is_some_and(|b| b.trim().is_empty())
        {
            return Err(format!(
                "cycle {stage}: a --title or --body given is not blank"
            ));
        }
        Ok(CommitRung {
            agent: commit_scope(self.scope.as_ref(), agent, stage)?,
            running: self.running(work),
            title: self.title.as_ref().map(|t| t.trim().to_string()),
            body,
            push: self.push,
            same_version: self.same_version,
        })
    }
}

impl TryFrom<&CycleArgs> for CycleParams {
    type Error = String;

    /// Resolve the workspace and the stage asked for.
    fn try_from(a: &CycleArgs) -> Result<Self, String> {
        let (work, agent) = resolve_repos(a.repo.as_deref())?;
        let only = |stage: &str, given: &[(&str, bool)]| match given.iter().find(|(_, g)| *g) {
            Some((flag, _)) => Err(format!("cycle {stage}: it takes no {flag}")),
            None => Ok(()),
        };
        let commits = [
            ("--same-version", a.same_version),
            ("-s", a.scope.is_some()),
            ("--push", a.push),
            ("--body", a.body.is_some()),
        ];
        if a.open {
            only("--open", &commits)?;
            a.open_params(work, agent)
        } else if a.ladder {
            only("--ladder", &commits)?;
            Ok(CycleParams::Ladder(a.running(work)))
        } else if a.status {
            only("--status", &commits)?;
            Ok(CycleParams::Status(a.running(work)))
        } else if let Some(heads) = &a.close {
            // A dead end's shape is wrong whatever else is given, so it is
            // said first.
            if a.dead_end && a.single_step {
                return Err(
                    "cycle --close --dead-end: a dead end closes the commits a cycle made with a \
                     closing commit of its own: --multi-step"
                        .into(),
                );
            }
            if let Some(missing) = close_missing(a) {
                return Err(missing);
            }
            let land = a.land_params(work.clone(), agent.clone(), false)?;
            let mut rung = a.rung(work, agent, "--close")?;
            // With a Land the push is the Land's, the destination's.
            rung.push = rung.push && land.is_none();
            Ok(CycleParams::Close(CloseCycle {
                rung,
                heads: heads.clone(),
                trapezoid: a.trapezoid.clone(),
                default_body: a.default_body,
                dead_end: a.dead_end,
                single_step: a.single_step,
                land,
            }))
        } else if let Some(land) = a.land_params(work.clone(), agent.clone(), true)? {
            Ok(CycleParams::Land(land))
        } else if a.commit {
            Ok(CycleParams::Commit(a.rung(work, agent, "--commit")?))
        } else {
            only(
                "--push",
                &[
                    ("--same-version", a.same_version),
                    ("--body", a.body.is_some()),
                    ("--title", a.title.is_some()),
                ],
            )?;
            if matches!(&a.scope, Some(RepoScope::Sides(s)) if !s.is_both()) {
                return Err(
                    "cycle --push -s w: the work half alone would publish trailers naming agent \
                     commits not yet published: -s b"
                        .into(),
                );
            }
            Ok(CycleParams::Push(PushCycle {
                agent: commit_scope(a.scope.as_ref(), agent, "--push")?.map(|(a, _)| a),
                running: a.running(work),
            }))
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
        CycleParams::Ladder(r) => ladder(r),
        CycleParams::Commit(c) => commit_rung(ctx, c),
        CycleParams::Close(c) => close(ctx, c),
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
/// config, its `{title}` the cycle title, else a body naming the
/// cycle and pointing at its record.
fn opening_body(repo: &std::path::Path, cycle: &str) -> Result<String, Box<dyn std::error::Error>> {
    let configured = crate::config_md::load(repo)?
        .and_then(|cfg| crate::toml_simple::toml_get(&cfg.map, "opening.body").cloned());
    let body = match configured {
        Some(body) => body,
        None => format!(
            "Opens the cycle \"{{title}}\". See details in {}.",
            crate::commit::RECORD
        ),
    };
    Ok(body.replace("{title}", cycle))
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

/// `--commit`: commit the working copies as a rung of the cycle in
/// progress. The cycle's first commit is its opening, titled and
/// bodied by default, and any other is a work rung, whose title is a
/// rung heading not yet committed. The ladder is rewritten first, the
/// rung done, so the commit carries it, and the cycle's bookmark is
/// the one advanced. Every check is made before the ladder is written.
fn commit_rung(ctx: &mut Context, c: &CommitRung) -> Result<(), Box<dyn std::error::Error>> {
    let stage = "--commit";
    let r = &c.running;
    let work = r.work.as_path();
    let bookmark = cycle_bookmark(r, stage)?;
    let commits = branch_titles(work, &bookmark)?;
    let path = work.join(crate::commit::RECORD);
    let todo = std::fs::read_to_string(&path)
        .map_err(|e| format!("cycle {stage}: {}: {e}", path.display()))?;
    let cycle = record::cycle_title(&todo).map_err(|e| format!("cycle {stage}: {e}"))?;
    let rungs = record::rung_titles(&todo).map_err(|e| format!("cycle {stage}: {e}"))?;
    let (opening, closing) = (format!("{cycle} opening"), format!("{cycle} closing"));
    let first = commits.is_empty();
    let title = match (&c.title, first) {
        (None, true) => opening.clone(),
        (Some(t), true) if *t == opening => opening.clone(),
        (Some(t), true) => {
            return Err(format!(
                "cycle {stage}: the cycle has no commit yet, so this one is its opening, \
                 '{opening}', and not '{t}': leave --title out"
            )
            .into());
        }
        (None, false) => {
            return Err(format!(
                "cycle {stage}: the opening is committed, so this is a work rung, which gives \
                 --title, a heading under `#### Rungs`, and --body"
            )
            .into());
        }
        (Some(t), false) => t.clone(),
    };
    if title == closing {
        return Err(format!(
            "cycle {stage}: '{title}' is the cycle's closing, which `cycle --close` makes"
        )
        .into());
    }
    if commits.contains(&title) {
        return Err(format!(
            "cycle {stage}: the rung '{title}' is committed already: `vc-x1 squash` continues it"
        )
        .into());
    }
    if !rungs.contains(&title) {
        return Err(format!(
            "cycle {stage}: '{title}' is no rung of the cycle, a `##### ` heading under `#### \
             Rungs` in {}: write its heading first",
            crate::commit::RECORD
        )
        .into());
    }
    let body = match (&c.body, first) {
        (Some(b), _) => b.clone(),
        (None, true) => opening_body(work, &cycle)?,
        (None, false) => {
            return Err(format!(
                "cycle {stage}: a work rung gives --body, its problem and solution"
            )
            .into());
        }
    };
    crate::pair::refuse_conflicts(work, "cycle --commit", "work")?;
    if jj::is_empty(work, "@")? {
        return Err(format!(
            "cycle {stage}: nothing to commit, the work repo's @ is empty ('{}')",
            work.display()
        )
        .into());
    }
    crate::commit::check_version(work, "@", c.same_version, "cycle --commit")?;
    let mut with = commits.clone();
    with.push(title.clone());
    let plan = record::plan_ladder(work, &with).map_err(|e| format!("cycle {stage}: {e}"))?;
    for finding in &plan.ladder.findings {
        warn!("cycle {stage}: {finding}");
    }
    info!(
        "cycle {stage}: {} '{title}' on '{bookmark}'",
        if first { "the opening" } else { "the rung" }
    );
    let params = commit_params(c, title, body, bookmark);
    let Some(edit) = plan.edit.as_ref().filter(|_| !r.dry_run) else {
        if plan.edit.is_some() {
            info!(
                "cycle {stage}: [dry-run] rewrite the ladder in {}, the rung done",
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
        "cycle {stage}: the ladder in {} is rewritten, the rung done",
        record::LADDER_FILE
    );
    crate::commit::commit(ctx, &params).inspect_err(|_| {
        if let Err(e) = std::fs::write(&edit.path, &old) {
            warn!(
                "cycle {stage}: the ladder in {} was not put back: {e}",
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
    stage: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let name = match &r.bookmark {
        Some(name) => name.clone(),
        None => {
            let path = r.work.join(crate::commit::RECORD);
            let todo = std::fs::read_to_string(&path)
                .map_err(|e| format!("cycle {stage}: {}: {e}", path.display()))?;
            let title = record::cycle_title(&todo).map_err(|e| format!("cycle {stage}: {e}"))?;
            crate::validate_anchors::anchor_slug(&title)
        }
    };
    if !jj::local_bookmark_exists(&r.work, &name)? {
        return Err(format!(
            "cycle {stage}: the cycle's bookmark '{name}' is not in {}: name it, --bookmark <NAME>",
            r.work.display()
        )
        .into());
    }
    Ok(name)
}

/// `--ladder`: make the ladder from the rung headings and the cycle's
/// commits, and write it, or with a dry run print it.
fn ladder(r: &Running) -> Result<(), Box<dyn std::error::Error>> {
    let bookmark = cycle_bookmark(r, "--ladder")?;
    let commits = branch_titles(&r.work, &bookmark)?;
    let plan =
        record::plan_ladder(&r.work, &commits).map_err(|e| format!("cycle --ladder: {e}"))?;
    for line in &plan.ladder.lines {
        info!("{line}");
    }
    for finding in &plan.ladder.findings {
        warn!("cycle --ladder: {finding}");
    }
    match (&plan.edit, r.dry_run) {
        (None, _) => info!(
            "cycle --ladder: the ladder in {} is as the rungs make it",
            record::LADDER_FILE
        ),
        (Some(_), true) => info!(
            "cycle --ladder: [dry-run] write the ladder into {}",
            record::LADDER_FILE
        ),
        (Some(edit), false) => {
            std::fs::write(&edit.path, &edit.text)
                .map_err(|e| format!("cycle --ladder: {}: {e}", edit.path.display()))?;
            info!(
                "cycle --ladder: the ladder is written into {}",
                record::LADDER_FILE
            );
        }
    }
    Ok(())
}

/// `--open`: plan the record, make the bookmark, then write the
/// record, so a record that cannot be written makes no bookmark, and a
/// name in use writes no record.
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

fn open(ctx: &mut Context, o: &OpenCycle) -> Result<(), Box<dyn std::error::Error>> {
    let dry_run = o.open.dry_run;
    if let Some(title) = &o.record {
        for problem in title_problems(title) {
            warn!("cycle --open: {problem}, and every commit of the cycle carries it");
        }
    }
    let plan = match &o.record {
        Some(title) => {
            record::plan_open(&o.open.work, title).map_err(|e| format!("cycle --open: {e}"))?
        }
        None => {
            info!("cycle --open: no record is written, the bookmark being made alone");
            record::OpenRecord::default()
        }
    };
    crate::open::open(ctx, &o.open)?;
    let tag = if dry_run {
        "cycle --open: [dry-run]"
    } else {
        "cycle --open:"
    };
    for line in &plan.did {
        info!("{tag} {line}");
    }
    if dry_run {
        return Ok(());
    }
    for edit in &plan.edits {
        std::fs::write(&edit.path, &edit.text)
            .map_err(|e| format!("cycle --open: {}: {e}", edit.path.display()))?;
    }
    if !plan.edits.is_empty() {
        info!(
            "cycle --open: the record is written and nothing is committed: plan the cycle in it, \
             and bump the version"
        );
    }
    Ok(())
}

/// A closing's body when none is given: `[closing] body` in the
/// config, its `{title}` the cycle title, else a body naming the
/// cycle and pointing at its record.
fn closing_body(repo: &std::path::Path, cycle: &str) -> Result<String, Box<dyn std::error::Error>> {
    let configured = crate::config_md::load(repo)?
        .and_then(|cfg| crate::toml_simple::toml_get(&cfg.map, "closing.body").cloned());
    let body = match configured {
        Some(body) => body,
        None => format!(
            "Closes the cycle \"{{title}}\". See details in {}.",
            crate::commit::RECORD
        ),
    };
    Ok(body.replace("{title}", cycle))
}

/// The closing commit's parents when they are not the work `@`'s own:
/// the destination first for a trapezoid, then the parents `@` has,
/// then each other head listed, none twice. `None` for a linear
/// closing of `@`'s line alone, which reparents nothing.
fn closing_parents(
    work: &std::path::Path,
    c: &CloseCycle,
) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
    let label = "cycle --close";
    let own = jj::parent_cids_of(work, "@")?;
    let mut parents: Vec<String> = Vec::new();
    if let Some(dest) = &c.trapezoid {
        let cid = crate::land::resolve_tip(work, &format!("{label} --trapezoid"), dest)?;
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

/// `--close`: close the cycle in progress from the work `@`. The
/// branch's commit count is its shape: none or one is a single-step
/// cycle, the closing squashed into its one commit, made now from `@`
/// or the opening with `@` folded in, under the bare cycle title; more
/// is a ladder, whose closing is its own commit, a merge when a
/// destination or another head is joined. The ladder is rewritten,
/// every rung done, and the record moved to `## Closed`, both in the
/// commit. Every check is made before a file is written.
fn close(ctx: &mut Context, c: &CloseCycle) -> Result<(), Box<dyn std::error::Error>> {
    let stage = "--close";
    let r = &c.rung.running;
    let work = r.work.as_path();
    let bookmark = cycle_bookmark(r, stage)?;
    let commits = branch_titles(work, &bookmark)?;
    let path = work.join(crate::commit::RECORD);
    let todo = std::fs::read_to_string(&path)
        .map_err(|e| format!("cycle {stage}: {}: {e}", path.display()))?;
    let cycle = record::cycle_title(&todo).map_err(|e| format!("cycle {stage}: {e}"))?;
    if !jj::local_bookmarks_at(work, "@ | @-")?.contains(&bookmark) {
        return Err(format!(
            "cycle {stage}: the cycle's bookmark '{bookmark}' is not on the work @ or @-, and a \
             closing is made from the cycle's tip: `jj new {bookmark}` moves @ onto it"
        )
        .into());
    }
    crate::pair::refuse_conflicts(work, "cycle --close", "work")?;
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
                "cycle {stage} --single-step: the branch has {n} commits, a ladder, which is \
                 never squashed into one: --multi-step makes its closing commit"
            )
            .into());
        }
        (false, 0) => {
            return Err(format!(
                "cycle {stage} --multi-step: the branch has no commit, so there is no opening \
                 for a closing to follow: `cycle --commit` makes the opening first, or \
                 --single-step commits the work @ as the cycle's one commit"
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
                "cycle {stage} --single-step: the cycle's one commit holds the work, so it gives \
                 --body, its problem and solution, or --default-body for the pointer at the record"
            )
            .into());
        }
        (None, false) => closing_body(work, &cycle)?,
    };
    let parents = match single {
        true if c.trapezoid.is_some() || !c.heads.is_empty() => {
            return Err(format!(
                "cycle {stage} --single-step: the cycle's one commit joins nothing, so it takes \
                 no head and no --trapezoid: --multi-step makes a closing commit that can"
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
    crate::commit::check_version(work, "@", c.rung.same_version, "cycle --close")?;
    let ladder = record::plan_ladder(work, &done).map_err(|e| format!("cycle {stage}: {e}"))?;
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
            "cycle {stage}: the record's {} still read `_To write._`, and the closing moves it to \
             `## Closed` as it is",
            unwritten.join(", ")
        );
    }
    let fold = single && commits.len() == 1;
    info!(
        "cycle {stage}: {} on '{bookmark}', '{title}'",
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
                "cycle {stage}: [dry-run] rewrite the ladder in {}, every rung done",
                record::LADDER_FILE
            );
        }
        if let Some(m) = &moved {
            m.write(true)?;
        }
        if let Some(parents) = &parents {
            info!(
                "cycle {stage}: [dry-run] the closing's parents are {}",
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
                "cycle --land: [dry-run] land the closing on '{}'{}, and delete the cycle's \
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
                "cycle {stage}: the ladder in {} is rewritten, every rung done",
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
            crate::merge::set_parents(ctx, work, "@", parents, "cycle --close")?;
        }
        let mut params = commit_params(&c.rung, title, body, bookmark);
        params.land_follows = c.land.is_some();
        crate::commit::commit(ctx, &params)
    })();
    if let Err(e) = &made {
        snapshot.restore("cycle --close", e.as_ref());
        for (path, text) in &written {
            if let Err(w) = std::fs::write(path, text) {
                warn!("cycle {stage}: {} was not put back: {w}", path.display());
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
        return Err(format!(
            "cycle --land: '{dest}' is not a local bookmark, and a Land moves one"
        )
        .into());
    }
    let at = jj::cid_of(work, dest)?;
    let remote = jj::remote_bookmark_targets(work, "origin")?
        .get(dest)
        .cloned();
    if remote.as_deref() != Some(at.as_str()) {
        let refusal =
            crate::land::not_at_origin(work, "cycle --land", dest, &at, remote.as_deref(), plain)?;
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
        "cycle --land: the cycle does not descend from '{dest}' at {}, which moved after the \
         cycle forked, and a Land only fast-forwards. The ways on:\n\
         \x20 - a merge at the Land, `cycle --land {dest} --merge`, a commit no rung validated\n\
         \x20 - the closing made again as a trapezoid, `cycle --close --trapezoid {dest}`, \
         after `jj abandon` of the one made\n\
         \x20 - a rebase, `jj rebase -b <cycle's bookmark> -d {dest}`, every rung then validated \
         again\n\
         \x20 - or your own",
        short(at)
    )
}

/// `--land`: land the closed cycle on the destination.
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
            "cycle --land: the cycle '{running}' is in progress in {}, and a Land is of a cycle \
             closed: `cycle --close` first, or `cycle --close --land {}`",
            crate::commit::RECORD,
            l.dest
        )
        .into());
    }
    let cycle = todo.as_deref().and_then(record::closed_title);
    let source = match closed.or_else(|| l.running.bookmark.clone()) {
        Some(named) => named,
        None => {
            let slug = cycle.as_deref().map(crate::validate_anchors::anchor_slug);
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
            "cycle --land: '{}' has not moved, so the Land is a fast-forward and no merge is made",
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
    let label = "cycle --land --merge";
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
                 `jj new {} {source}` makes it for you to resolve, then `cycle --land {}`",
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

/// `--push` alone: publish what the cycle's state calls for. A cycle
/// in progress, or closed and not landed, is a draft, and its bookmark
/// is pushed. A cycle landed here publishes the target its landmark is
/// on, the one such bookmark not where its `origin` is, and its own
/// bookmark is deleted on `origin` when it is there. Nothing to publish
/// is said, and no error.
fn push(ctx: &mut Context, p: &PushCycle) -> Result<(), Box<dyn std::error::Error>> {
    let label = "cycle --push";
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
        Ok(_) => Some(cycle_bookmark(r, "--push")?),
        Err(_) => record::closed_title(&todo)
            .map(|c| {
                r.bookmark
                    .clone()
                    .unwrap_or_else(|| crate::validate_anchors::anchor_slug(&c))
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
    let slug = crate::validate_anchors::anchor_slug(&cycle);
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

/// `--status`: where the cycle stands, a line each, read from the
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
        let slug = closed.as_deref().map(crate::validate_anchors::anchor_slug);
        match (closed, slug) {
            (Some(c), Some(slug)) if jj::local_bookmark_exists(work, &slug)? => {
                out.push(format!(
                    "closed: '{c}', its bookmark '{slug}' not yet landed"
                ));
                out.push("next: `cycle --land <BOOKMARK> -s <SCOPE>` lands it".to_string());
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
                    [] => "next: `cycle --open --title \"<title>\"` opens one".to_string(),
                    names => format!(
                        "next: `cycle --push -s <SCOPE>` publishes {}, landed here and not on \
                         origin",
                        names
                            .iter()
                            .map(|b| format!("'{b}'"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                });
            }
            _ => out.push("next: `cycle --open --title \"<title>\"` opens one".to_string()),
        }
        return Ok(out);
    };
    out.push(format!("cycle: {cycle}"));
    let bookmark = match cycle_bookmark(r, "--status") {
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
    let rungs = record::rung_titles(&todo).and_then(|rungs| {
        let ladder = record::ladder_of(&cycle, &rungs, &commits)?;
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
            "note: the ladder in {} differs, and `cycle --ladder` writes this one",
            record::LADDER_FILE
        ));
    }
    let closing = format!("{cycle} closing");
    let current = rungs.iter().find(|r| !commits.contains(r));
    out.push(match (commits.len(), current) {
        (0, _) => {
            "next: `cycle --commit -s <SCOPE>` makes the opening, or `cycle --close -s <SCOPE> \
                   --single-step --body` closes a single-step cycle"
                .to_string()
        }
        (1, Some(c)) if *c == closing => "next: `cycle --close -s <SCOPE> --single-step \
                                          --body` squashes the closing into the opening, or \
                                          `--multi-step` makes a closing commit above it"
            .to_string(),
        (_, Some(c)) if *c == closing => {
            "next: `cycle --close -s <SCOPE> --multi-step` makes the closing, `--trapezoid \
             <BOOKMARK>` a merge"
                .to_string()
        }
        (_, Some(c)) => format!("next: `cycle --commit -s <SCOPE> --title \"{c}\" --body ...`"),
        (_, None) => "next: every rung is committed".to_string(),
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
        let scoped = ["--commit", "--close", "--land"];
        if args.iter().any(|a| scoped.contains(a)) && !args.contains(&"-s") {
            all.extend_from_slice(&["-s", "b"]);
        }
        let mut a = parse(&all)?;
        a.repo = Some(repo.to_path_buf());
        CycleParams::try_from(&a)
    }

    fn run(repo: &Path, args: &[&str]) -> Result<(), String> {
        cycle(&mut test_ctx(), &params(repo, args)?).map_err(|e| e.to_string())
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

    /// A stage is required, the open's options need `--open`, and it
    /// needs a name, a bare title.
    #[test]
    fn a_stage_is_required() {
        let err = parse(&["vc-x1", "cycle"]).unwrap_err();
        assert!(err.contains("--open"), "{err}");
        let err = parse(&["vc-x1", "cycle", "--no-record"]).unwrap_err();
        assert!(err.contains("--open"), "{err}");
        let fx = Fixture::new("cycle-stage");
        let err = params(&fx.work, &["--open"]).unwrap_err();
        assert!(
            err.contains("--title") && err.contains("--bookmark"),
            "{err}"
        );
        let err = params(&fx.work, &["--open", "--title", "feat: x opening"]).unwrap_err();
        assert!(
            err.contains("opening") && err.contains("cycle --open"),
            "{err}"
        );
    }

    /// `--open` makes the bookmark on the work `@` and writes the
    /// record into the working copy, nothing committed, and a dry run
    /// does neither.
    #[test]
    fn open_makes_the_bookmark_and_the_record() {
        let fx = Fixture::new("cycle-open");
        write_record(&fx);
        run(
            &fx.work,
            &["--open", "--title", "feat: a cycle", "--dry-run"],
        )
        .expect("a dry run");
        assert_eq!(bookmarks_at(&fx.work), "", "a dry run makes none");
        assert_eq!(fs::read_to_string(fx.work.join("TODO.md")).unwrap(), TODO);

        run(&fx.work, &["--open", "--title", "feat: a cycle"]).expect("opened");
        assert_eq!(bookmarks_at(&fx.work), "feat-a-cycle");
        let todo = fs::read_to_string(fx.work.join("TODO.md")).unwrap();
        assert!(todo.contains("## In Progress\n\nThe record's home.\n\n### feat: a cycle\n"));
        assert!(!todo.contains("chore: the last cycle"), "{todo}");
        let ztoc = fs::read_to_string(fx.work.join("zTOC.md")).unwrap();
        assert!(
            ztoc.contains("- 0 (current) [feat: a cycle opening](TODO.md#feat-a-cycle-opening)\n"),
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
        run(&fx.work, &["--open", "--title", "feat: a cycle"]).expect("opened");
        let err = run(&fx.work, &["--open", "--title", "fix: another"]).unwrap_err();
        assert!(err.contains("in progress"), "{err}");
        assert_eq!(bookmarks_at(&fx.work), "feat-a-cycle", "no second bookmark");

        let fx = Fixture::new("cycle-open-in-use");
        write_record(&fx);
        jj_ok(
            &fx.work,
            &["bookmark", "create", "feat-a-cycle", "-r", "main"],
        );
        let err = run(&fx.work, &["--open", "--title", "feat: a cycle"]).unwrap_err();
        assert!(err.contains("exists"), "{err}");
        assert_eq!(fs::read_to_string(fx.work.join("TODO.md")).unwrap(), TODO);
    }

    /// A workspace without its record is refused, nothing made, and
    /// `--no-record` and a bookmark with no title get the bookmark alone.
    #[test]
    fn open_makes_the_bookmark_alone() {
        let fx = Fixture::new("cycle-open-alone");
        let err = run(&fx.work, &["--open", "--title", "feat: a cycle"]).unwrap_err();
        assert!(
            err.contains("no TODO.md or zTOC.md") && err.contains("--no-record"),
            "{err}"
        );
        assert_eq!(bookmarks_at(&fx.work), "", "nothing made");
        fs::write(fx.work.join("TODO.md"), TODO).expect("write");
        let err = run(&fx.work, &["--open", "--title", "feat: a cycle"]).unwrap_err();
        assert!(err.contains("no zTOC.md,"), "{err}");
        run(
            &fx.work,
            &["--open", "--title", "feat: a cycle", "--no-record"],
        )
        .expect("alone");
        assert_eq!(bookmarks_at(&fx.work), "feat-a-cycle");

        let fx = Fixture::new("cycle-open-no-record");
        write_record(&fx);
        run(
            &fx.work,
            &["--open", "--title", "feat: a cycle", "--no-record"],
        )
        .expect("opened");
        assert_eq!(bookmarks_at(&fx.work), "feat-a-cycle");
        assert_eq!(fs::read_to_string(fx.work.join("TODO.md")).unwrap(), TODO);

        let fx = Fixture::new("cycle-open-named");
        write_record(&fx);
        run(&fx.work, &["--open", "--bookmark", "mine"]).expect("opened");
        assert_eq!(bookmarks_at(&fx.work), "mine");
        assert_eq!(fs::read_to_string(fx.work.join("TODO.md")).unwrap(), TODO);
    }

    /// `--ladder` makes the ladder from the rung headings, a commit of
    /// a rung's title making it done, and a commit that is no rung's is
    /// refused, the ladder left as it was.
    #[test]
    fn ladder_is_made_from_the_headings_and_the_commits() {
        let fx = Fixture::new("cycle-ladder");
        write_record(&fx);
        run(&fx.work, &["--open", "--title", "feat: a cycle"]).expect("opened");
        let todo = fs::read_to_string(fx.work.join("TODO.md")).unwrap();
        let planned = todo.replace(
            "##### feat: a cycle closing",
            "##### feat: one rung\n\nIts intent.\n\n##### feat: a cycle closing",
        );
        fs::write(fx.work.join("TODO.md"), planned).expect("write");
        let ladder = || fs::read_to_string(fx.work.join("zTOC.md")).unwrap();

        run(&fx.work, &["--ladder", "--dry-run"]).expect("a dry run");
        assert!(
            !ladder().contains("feat: one rung"),
            "a dry run writes none"
        );
        run(&fx.work, &["--ladder"]).expect("made");
        assert!(
            ladder().contains(
                "- 0 (current) [feat: a cycle opening](TODO.md#feat-a-cycle-opening)\n\
                 - [feat: one rung](TODO.md#feat-one-rung)\n\
                 - [feat: a cycle closing](TODO.md#feat-a-cycle-closing)\n"
            ),
            "{}",
            ladder()
        );

        jj_ok(&fx.work, &["commit", "-m", "feat: a cycle opening"]);
        run(&fx.work, &["--ladder"]).expect("made");
        assert!(
            ladder().contains(
                "- 0 (done) [feat: a cycle opening](TODO.md#feat-a-cycle-opening)\n\
                 - 1 (current) [feat: one rung](TODO.md#feat-one-rung)\n"
            ),
            "{}",
            ladder()
        );

        let before = ladder();
        jj_ok(&fx.work, &["commit", "-m", "wip: not a rung"]);
        let err = run(&fx.work, &["--ladder"]).unwrap_err();
        assert!(err.contains("'wip: not a rung'"), "{err}");
        assert_eq!(ladder(), before);
    }

    /// `--ladder` reads the cycle in progress, so it takes no title, and
    /// needs the cycle's bookmark, named when it is not the slug.
    #[test]
    fn ladder_needs_the_cycle_and_its_bookmark() {
        let fx = Fixture::new("cycle-ladder-needs");
        write_record(&fx);
        let err = params(&fx.work, &["--ladder", "--title", "feat: x"]).unwrap_err();
        assert!(err.contains("cannot be used with"), "{err}");
        let err = run(&fx.work, &["--ladder"]).unwrap_err();
        assert!(err.contains("no cycle in progress"), "{err}");

        run(&fx.work, &["--open", "--title", "feat: a cycle"]).expect("opened");
        jj_ok(&fx.work, &["bookmark", "rename", "feat-a-cycle", "mine"]);
        let err = run(&fx.work, &["--ladder"]).unwrap_err();
        assert!(
            err.contains("'feat-a-cycle'") && err.contains("--bookmark"),
            "{err}"
        );
        run(&fx.work, &["--ladder", "--bookmark", "mine"]).expect("made");
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
        run(&fx.work, &["--open", "--title", "feat: a cycle"]).expect("opened");
        let todo = fs::read_to_string(fx.work.join("TODO.md")).unwrap();
        let planned = todo.replace(
            "##### feat: a cycle closing",
            "##### feat: one rung\n\nIts intent.\n\n##### feat: a cycle closing",
        );
        fs::write(fx.work.join("TODO.md"), planned).expect("write");
        fx
    }

    /// The cycle's first commit is its opening, titled and bodied by
    /// default, the ladder in it showing it done and the next rung
    /// current, the cycle's bookmark on it, and the pair stamped.
    #[test]
    fn the_first_commit_is_the_opening() {
        let fx = planned("cycle-commit-opening");
        run(&fx.work, &["--commit", "--dry-run"]).expect("a dry run");
        assert_eq!(bookmarks_at(&fx.work), "feat-a-cycle", "nothing committed");
        assert!(
            fs::read_to_string(fx.work.join("zTOC.md"))
                .unwrap()
                .contains("- 0 (current)"),
            "a dry run writes no ladder"
        );

        run(&fx.work, &["--commit"]).expect("the opening");
        let desc = description(&fx.work, "@-");
        assert!(
            desc.starts_with(
                "feat: a cycle opening\n\nOpens the cycle \"feat: a cycle\". See details in \
                 TODO.md.\n"
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
                 - 1 (current) [feat: one rung](TODO.md#feat-one-rung)\n\
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
        run(&fx.work, &["--commit"]).expect("the opening");
        fs::write(fx.work.join("work.txt"), "the rung's work\n").expect("write");
        let rung = [
            "--commit",
            "--title",
            "feat: one rung",
            "--body",
            "Why.\n\nSolution: how.",
        ];
        run(&fx.work, &rung).expect("the rung");
        let desc = description(&fx.work, "@-");
        assert!(
            desc.starts_with("feat: one rung\n\nWhy.\n\nSolution: how.\n"),
            "{desc}"
        );
        assert!(
            file_at(&fx.work, "@-", "zTOC.md").contains(
                "- 1 (done) [feat: one rung](TODO.md#feat-one-rung)\n\
                 - 2 (current) [feat: a cycle closing](TODO.md#feat-a-cycle-closing)\n"
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
        let rung = ["--commit", "--title", "feat: one rung", "--body", "b"];
        let err = run(&fx.work, &rung).unwrap_err();
        assert!(err.contains("its opening"), "{err}");
        run(&fx.work, &["--commit"]).expect("the opening");
        let ladder = fs::read_to_string(fx.work.join("zTOC.md")).unwrap();

        let err = run(&fx.work, &rung).unwrap_err();
        assert!(err.contains("nothing to commit"), "{err}");
        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        let err = run(&fx.work, &["--commit"]).unwrap_err();
        assert!(err.contains("a work rung"), "{err}");
        let err = run(&fx.work, &["--commit", "--title", "feat: one rung"]).unwrap_err();
        assert!(err.contains("--body"), "{err}");
        let err = run(
            &fx.work,
            &["--commit", "--title", "feat: unplanned", "--body", "b"],
        )
        .unwrap_err();
        assert!(err.contains("no rung of the cycle"), "{err}");
        let err = run(
            &fx.work,
            &[
                "--commit",
                "--title",
                "feat: a cycle closing",
                "--body",
                "b",
            ],
        )
        .unwrap_err();
        assert!(err.contains("cycle --close"), "{err}");
        let err = run(
            &fx.work,
            &[
                "--commit",
                "--title",
                "feat: a cycle opening",
                "--body",
                "b",
            ],
        )
        .unwrap_err();
        assert!(err.contains("committed already"), "{err}");
        assert_eq!(fs::read_to_string(fx.work.join("zTOC.md")).unwrap(), ladder);
        assert_eq!(
            description(&fx.work, "@-").lines().next(),
            Some("feat: a cycle opening")
        );
    }

    /// `-s` is required with the stages that write commits and refused
    /// with the rest, it is checked against the workspace, `w` makes
    /// the work half alone, and a plain repo's commit carries no trailer.
    #[test]
    fn the_scope_is_required_and_fits_the_workspace() {
        for stage in [vec!["--commit"], vec!["--close"], vec!["--land", "main"]] {
            let mut argv = vec!["vc-x1", "cycle"];
            argv.extend(&stage);
            let err = parse(&argv).unwrap_err();
            assert!(err.contains("--scope"), "{stage:?}: {err}");
        }
        for stage in ["--ladder", "--status"] {
            parse(&["vc-x1", "cycle", stage]).expect("no scope needed");
        }
        let fx = planned("cycle-commit-scope");
        let err = params(&fx.work, &["--status", "-s", "b"]).unwrap_err();
        assert!(err.contains("takes no -s"), "{err}");
        let err = params(&fx.work, &["--land", "main", "-s", "p"]).unwrap_err();
        assert!(err.contains("dual workspace"), "{err}");
        let err = params(&fx.work, &["--commit", "-s", "p"]).unwrap_err();
        assert!(err.contains("dual workspace"), "{err}");
        let agent_at = jj_ok(
            &fx.bot,
            &["log", "-r", "@", "--no-graph", "-T", "change_id.short(12)"],
        );
        run(&fx.work, &["--commit", "-s", "w"]).expect("the work half");
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
        let err = params(&por.work, &["--commit", "-s", "b"]).unwrap_err();
        assert!(err.contains("plain repo"), "{err}");
        run(&por.work, &["--open", "--title", "feat: a cycle"]).expect("opened");
        run(&por.work, &["--commit", "-s", "p"]).expect("the opening");
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
        run(&fx.work, &["--open", "--title", "feat: a cycle"]).expect("opened");
        fs::write(fx.work.join("work.txt"), "the work\n").expect("write");
        let err = run(&fx.work, &["--close", "--single-step"]).unwrap_err();
        assert!(
            err.contains("--body") && err.contains("--default-body"),
            "{err}"
        );
        let err = run(
            &fx.work,
            &[
                "--close",
                "--single-step",
                "--trapezoid",
                "main",
                "--body",
                "b",
            ],
        )
        .unwrap_err();
        assert!(err.contains("joins nothing"), "{err}");
        run(
            &fx.work,
            &[
                "--close",
                "--single-step",
                "--body",
                "Why.\n\nSolution: how.",
                "--dry-run",
            ],
        )
        .expect("a dry run");
        assert!(above_main(&fx.work).is_empty(), "a dry run commits nothing");

        run(
            &fx.work,
            &[
                "--close",
                "--single-step",
                "--body",
                "Why.\n\nSolution: how.",
            ],
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
        run(&fx.work, &["--open", "--title", "feat: a cycle"]).expect("opened");
        run(&fx.work, &["--commit"]).expect("the opening");
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

        run(&fx.work, &["--close", "--single-step", "--default-body"]).expect("closed");
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
        run(&fx.work, &["--commit"]).expect("the opening");
        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        run(
            &fx.work,
            &["--commit", "--title", "feat: one rung", "--body", "b"],
        )
        .expect("the rung");
        let tip = jj_ok(
            &fx.work,
            &["log", "-r", "@-", "--no-graph", "-T", "commit_id"],
        );
        jj_ok(&fx.work, &["new", "feat-a-cycle-"]);
        let err = run(&fx.work, &["--close", "--multi-step"]).unwrap_err();
        assert!(err.contains("not on the work @ or @-"), "{err}");
        jj_ok(&fx.work, &["new", "feat-a-cycle"]);

        run(&fx.work, &["--close", "--multi-step"]).expect("closed");
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
            desc.contains("Closes the cycle \"feat: a cycle\". See details in TODO.md."),
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

    /// `--trapezoid` makes the destination the closing's first parent,
    /// and a head listed is another parent, after the ladder's tip.
    #[test]
    fn a_trapezoid_joins_the_destination_and_the_heads_listed() {
        let fx = planned("cycle-close-trapezoid");
        run(&fx.work, &["--commit"]).expect("the opening");
        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        run(
            &fx.work,
            &["--commit", "--title", "feat: one rung", "--body", "b"],
        )
        .expect("the rung");
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

        run(
            &fx.work,
            &["--close", &side, "--multi-step", "--trapezoid", "main"],
        )
        .expect("closed");
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
        let err = run(&fx.work, &["--close", "--multi-step", "--dead-end"]).unwrap_err();
        assert!(err.contains("no commit"), "{err}");
        run(&fx.work, &["--commit"]).expect("the opening");
        fs::write(fx.work.join("work.txt"), "kept\n").expect("write");
        run(&fx.work, &["--close", "--multi-step", "--dead-end"]).expect("closed");
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
        run(&fx.work, &["--commit"]).expect("the opening");
        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        run(
            &fx.work,
            &["--commit", "--title", "feat: one rung", "--body", "b"],
        )
        .expect("the rung");
        fx
    }

    /// `--close --land` lands the closing it makes: the destination is
    /// fast-forwarded to it and the cycle's bookmark deleted, and a dry
    /// run makes neither.
    #[test]
    fn close_and_land_chain() {
        let fx = laddered("cycle-land-chain");
        let main = cid(&fx.work, "main");
        run(
            &fx.work,
            &["--close", "--multi-step", "--land", "main", "--dry-run"],
        )
        .expect("a dry run");
        assert_eq!(cid(&fx.work, "main"), main, "a dry run lands nothing");
        assert_eq!(
            title_of(&fx.work, "@-"),
            "feat: one rung",
            "and closes nothing"
        );

        run(&fx.work, &["--close", "--multi-step", "--land", "main"]).expect("closed and landed");
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
        run(&fx.work, &["--open", "--title", "feat: a cycle"]).expect("opened");
        fs::write(fx.work.join("work.txt"), "the work\n").expect("write");
        run(
            &fx.work,
            &[
                "--close",
                "--single-step",
                "--body",
                "Why.",
                "--land",
                "main",
                "--push",
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

    /// `--land` alone lands a cycle already closed, found by the title
    /// `## Closed` holds, and a cycle in progress is refused.
    #[test]
    fn land_is_of_a_cycle_closed() {
        let fx = laddered("cycle-land-closed");
        let err = run(&fx.work, &["--land", "main"]).unwrap_err();
        assert!(
            err.contains("in progress") && err.contains("cycle --close"),
            "{err}"
        );
        let err = params(&fx.work, &["--land", "main", "--title", "t"]).unwrap_err();
        assert!(err.contains("--merge"), "{err}");
        let err = params(&fx.work, &["--land", "main", "-s", "w"]).unwrap_err();
        assert!(err.contains("work-only"), "{err}");
        run(&fx.work, &["--close", "--multi-step"]).expect("closed");
        let err = run(&fx.work, &["--land", "trunk"]).unwrap_err();
        assert!(err.contains("not a local bookmark"), "{err}");
        run(&fx.work, &["--land", "main"]).expect("landed");
        assert_eq!(title_of(&fx.work, "main"), "feat: a cycle closing");
    }

    /// A destination that moved is refused naming the ways on, and
    /// `--merge` joins it by a merge commit, which is what lands.
    #[test]
    fn a_destination_that_moved_is_refused_or_merged() {
        let fx = laddered("cycle-land-moved");
        run(&fx.work, &["--close", "--multi-step"]).expect("closed");
        let closing = cid(&fx.work, "@-");
        // The destination moves on origin, and is synced here.
        jj_ok(&fx.work, &["new", "main", "-m", "feat: someone else's"]);
        fs::write(fx.work.join("other.txt"), "theirs\n").expect("write");
        jj_ok(&fx.work, &["bookmark", "set", "main", "-r", "@"]);
        jj_ok(&fx.work, &["git", "push", "--bookmark", "main"]);
        let main = cid(&fx.work, "main");
        jj_ok(&fx.work, &["new", "feat-a-cycle"]);

        let err = run(&fx.work, &["--land", "main"]).unwrap_err();
        assert!(
            err.contains("moved") && err.contains("--merge") && err.contains("jj rebase"),
            "{err}"
        );
        assert_eq!(cid(&fx.work, "main"), main, "nothing moved");

        run(&fx.work, &["--land", "main", "--merge"]).expect("merged and landed");
        assert_eq!(title_of(&fx.work, "main"), "feat: a cycle merge");
        assert_eq!(parents(&fx.work, "main"), [main, closing]);
        assert_eq!(file_at(&fx.work, "main", "other.txt"), "theirs");
        assert_eq!(file_at(&fx.work, "main", "work.txt"), "work");
        assert!(
            description(&fx.work, "main").contains("ochid: "),
            "a pair's commit"
        );
    }

    /// `--status` says where the cycle stands at each stage, and
    /// changes nothing.
    #[test]
    fn status_says_where_the_cycle_stands() {
        let fx = Fixture::new("cycle-status");
        let status = |fx: &Fixture| match params(&fx.work, &["--status"]).expect("params") {
            CycleParams::Status(r) => super::status(&r).expect("status").join("\n"),
            _ => panic!("expected Status"),
        };
        assert!(status(&fx).contains("no TODO.md"), "{}", status(&fx));
        write_record(&fx);
        let s = status(&fx);
        assert!(
            s.contains("cycle: none in progress") && s.contains("cycle --open"),
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
            s.contains("next: `cycle --commit -s <SCOPE>` makes the opening"),
            "{s}"
        );

        run(&fx.work, &["--commit"]).expect("the opening");
        let s = status(&fx);
        assert!(s.contains("commits: 1 on the branch, 1 not pushed"), "{s}");
        assert!(s.contains("  - 1 (current) feat: one rung\n"), "{s}");
        assert!(s.contains("work @: empty"), "{s}");
        assert!(!s.contains("differs"), "{s}");
        assert!(
            s.contains("next: `cycle --commit -s <SCOPE> --title \"feat: one rung\" --body ...`"),
            "{s}"
        );

        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        run(
            &fx.work,
            &["--commit", "--title", "feat: one rung", "--body", "b"],
        )
        .expect("the rung");
        assert!(
            status(&fx).contains("next: `cycle --close -s <SCOPE> --multi-step` makes the closing")
        );
        jj_ok(&fx.work, &["commit", "-m", "wip: by hand"]);
        let s = status(&fx);
        assert!(
            s.contains("ladder: not made") && s.contains("'wip: by hand'"),
            "{s}"
        );
        jj_ok(&fx.work, &["abandon", "@-"]);

        run(&fx.work, &["--close", "--multi-step"]).expect("closed");
        let s = status(&fx);
        assert!(
            s.contains("closed: 'feat: a cycle'") && s.contains("cycle --land"),
            "{s}"
        );
        run(&fx.work, &["--land", "main"]).expect("landed");
        let s = status(&fx);
        assert!(
            s.contains("closed: 'feat: a cycle', landed on 'main'"),
            "{s}"
        );
        assert!(
            s.contains("next: `cycle --push -s <SCOPE>` publishes 'main'"),
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
        run(&fx.work, &["--commit"]).expect("the opening sets the version");
        let ladder = fs::read_to_string(fx.work.join("zTOC.md")).unwrap();

        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        let rung = ["--commit", "--title", "feat: one rung", "--body", "b"];
        let err = run(&fx.work, &rung).unwrap_err();
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
            run(&fx.work, &rung).is_err(),
            "the version's line is as it was"
        );

        fs::write(
            fx.work.join("VERSION.md"),
            "name = \"y\"\nversion = \"1.0.0-1\"\n",
        )
        .expect("write");
        run(&fx.work, &rung).expect("the version is bumped");
        assert_eq!(title_of(&fx.work, "@-"), "feat: one rung");

        let err = run(&fx.work, &["--close", "--multi-step"]).unwrap_err();
        assert!(err.contains("--same-version"), "{err}");
        assert_eq!(title_of(&fx.work, "@-"), "feat: one rung", "nothing closed");
        run(&fx.work, &["--close", "--multi-step", "--same-version"]).expect("closed as it is");
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
    /// several names `--bookmark`, the flag `cycle --land` has.
    #[test]
    fn several_bookmarks_name_the_flag_cycle_has() {
        let fx = Fixture::new("cycle-land-several");
        for name in ["one", "two"] {
            jj_ok(&fx.work, &["new", "main"]);
            fs::write(fx.work.join(format!("{name}.txt")), "x\n").expect("write");
            jj_ok(&fx.work, &["commit", "-m", name]);
            jj_ok(&fx.work, &["bookmark", "create", name, "-r", "@-"]);
        }
        let err = run(&fx.work, &["--land", "main"]).unwrap_err();
        assert!(
            err.starts_with("cycle --land:") && err.contains("--bookmark <NAME>"),
            "{err}"
        );
        assert!(!err.contains("--source"), "{err}");
        run(&fx.work, &["--land", "main", "--bookmark", "two"]).expect("landed");
        assert_eq!(title_of(&fx.work, "main"), "two");
    }

    /// `--close` names the shape, one of the two, and the branch's
    /// commit count checks it: no commit is no ladder, two are no single
    /// step, and one commit, the opening, takes either, the name
    /// deciding whether the closing is squashed into it or its own.
    #[test]
    fn the_closing_names_its_shape_and_the_count_checks_it() {
        let fx = planned("cycle-close-shape");
        let err = params(&fx.work, &["--close"]).unwrap_err();
        assert!(
            err.contains("--single-step") && err.contains("--multi-step"),
            "{err}"
        );
        let err = parse(&[
            "vc-x1",
            "cycle",
            "--close",
            "-s",
            "b",
            "--single-step",
            "--multi-step",
        ])
        .unwrap_err();
        assert!(err.contains("cannot be used with"), "{err}");
        let err = params(&fx.work, &["--close", "--single-step", "--dead-end"]).unwrap_err();
        assert!(err.contains("--multi-step"), "{err}");

        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        let err = run(&fx.work, &["--close", "--multi-step"]).unwrap_err();
        assert!(
            err.contains("no opening") && err.contains("--single-step"),
            "{err}"
        );

        // One commit, the opening: --multi-step makes a closing above it.
        run(&fx.work, &["--commit"]).expect("the opening");
        let opening = jj_ok(
            &fx.work,
            &["log", "-r", "@-", "--no-graph", "-T", "commit_id"],
        );
        fs::write(fx.work.join("more.txt"), "more\n").expect("write");
        run(&fx.work, &["--close", "--multi-step", "--dry-run"]).expect("a dry run");
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
        run(&fx.work, &["--close", "--multi-step"]).expect("closed");
        assert_eq!(
            above_main(&fx.work),
            ["feat: a cycle opening", "feat: a cycle closing"],
            "the opening kept, the closing its own commit"
        );
        assert_eq!(parents(&fx.work, "@-"), [opening]);

        // Two commits: no single step.
        let fx = planned("cycle-close-shape-two");
        run(&fx.work, &["--commit"]).expect("the opening");
        fs::write(fx.work.join("work.txt"), "work\n").expect("write");
        run(
            &fx.work,
            &["--commit", "--title", "feat: one rung", "--body", "b"],
        )
        .expect("rung");
        let err = run(&fx.work, &["--close", "--single-step", "--body", "b"]).unwrap_err();
        assert!(
            err.contains("2 commits") && err.contains("--multi-step"),
            "{err}"
        );
    }

    /// `--push` alone publishes what the cycle's state calls for: the
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
        let err = params(&fx.work, &["--push"]).unwrap_err();
        assert!(err.contains("--scope"), "{err}");
        let err = params(&fx.work, &["--push", "-s", "w"]).unwrap_err();
        assert!(err.contains("-s b"), "{err}");
        let err = params(&fx.work, &["--push", "-s", "b", "--title", "t"]).unwrap_err();
        assert!(err.contains("takes no --title"), "{err}");

        run(&fx.work, &["--commit"]).expect("the opening");
        run(&fx.work, &["--push", "-s", "b"]).expect("the draft pushed");
        assert!(on_origin("feat-a-cycle"), "the draft's bookmark");

        let todo = fs::read_to_string(fx.work.join("TODO.md")).unwrap();
        fs::write(
            fx.work.join("TODO.md"),
            todo.replace("##### feat: one rung\n\nIts intent.\n\n", ""),
        )
        .expect("write");
        run(&fx.work, &["--close", "--multi-step"]).expect("closed");
        run(&fx.work, &["--push", "-s", "b"]).expect("the closed draft pushed");
        run(&fx.work, &["--land", "main"]).expect("landed here");
        assert_ne!(
            cid(&fx.work, "main"),
            cid(&fx.work, "main@origin"),
            "not yet published"
        );
        assert!(
            on_origin("feat-a-cycle"),
            "the draft's bookmark still on origin"
        );

        run(&fx.work, &["--push", "-s", "b"]).expect("the target published");
        assert_eq!(cid(&fx.work, "main"), cid(&fx.work, "main@origin"));
        assert!(
            !on_origin("feat-a-cycle"),
            "the draft's bookmark deleted on origin"
        );
        run(&fx.work, &["--push", "-s", "b"]).expect("nothing left, no error");
    }

    /// A refusal of `--close` names everything it lacks at once.
    #[test]
    fn the_closing_names_all_it_lacks() {
        let fx = planned("cycle-close-lacks");
        let err = params(&fx.work, &["--close"]).unwrap_err();
        assert!(
            err.contains("shape") && err.contains("with --single-step, --body or --default-body"),
            "{err}"
        );
        let err = params(&fx.work, &["--close", "--body", "b"]).unwrap_err();
        assert!(
            err.contains("shape") && !err.contains("--default-body"),
            "{err}"
        );
        let err = params(&fx.work, &["--close", "--single-step"]).unwrap_err();
        assert!(err.contains("--body") && !err.contains("shape"), "{err}");
        params(&fx.work, &["--close", "--multi-step"]).expect("a ladder's closing has a body");
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

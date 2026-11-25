//! The `rung` subcommand: a rung's acts, each a subcommand of its own.
//!
//! - `next` starts the next rung, the first not done, marking it
//!   `(current)` in the working copy's `zTOC.md`, and with `--ver`
//!   sets the version by the config's `[version] bump`.
//! - `stop` clears the marker, the edits in `@` left as they are.
//! - `commit` commits the current rung, by the code `cycle close`
//!   shares, its title the rung's, the ladder it carries marking it
//!   `(done)` and none current.
//!
//! A single-step is the plan, one rung, and `cycle close --squash`, so
//! a rung is always a ladder's.
//!
//! With no cycle in progress, `commit --title` commits the work `@`
//! verbatim, with no record, onto the bookmark on `@-`, advancing it, so
//! a run of them iterates on one bookmark and `cycle land` lands it
//! once. `--bookmark [NAME]` makes or moves one first, the title's slug
//! when bare: made when absent, moved forward when behind `@-`, asked
//! about when sideways or ahead, `--yes` answering, and never a target.
//!
//! The marker lives in the working copy alone: a commit ends its rung
//! and starts none, so no commit's tree claims work not yet begun.

use std::path::Path;

use clap::{Args, Subcommand};
use log::info;

use super::{
    CommitRung, LandCycle, Running, Where, branch_titles, commit_params, commit_rung,
    cycle_bookmark, land_checks, land_closed, publish_scope, record, rung_params,
};
use crate::context::Context;
use crate::jj;
use crate::options_flags::scope::{RepoScope, repo_scope_parser};
use crate::subcommand::SubcommandRunner;

/// Run a rung's acts.
#[derive(Args, Debug)]
pub struct RungArgs {
    #[command(subcommand)]
    pub command: RungCommand,
}

/// The `rung` subcommands.
#[derive(Subcommand, Debug)]
pub enum RungCommand {
    /// Start the next rung: mark the first rung not done `(current)` in zTOC.md
    #[command(
        long_about = "Start the next rung: mark the first rung not done `(current)`\n\
        in the working copy's zTOC.md, refused while a rung is current and\n\
        before the plan is committed, `cycle plan commit` committing the\n\
        opening, which is no rung started here.\n\
        With --ver, set the version first, by the command the config's\n\
        `[version] bump` names, `{ver}` the version given, so the dev\n\
        build under review carries it. No commit is made."
    )]
    Next(NextArgs),

    /// Leave the current rung: clear its `(current)` marker, the edits left alone
    Stop(Where),

    /// Commit the current rung: its title the rung's, the ladder marking it `(done)`
    #[command(
        long_about = "Commit the working copies as the current rung, its title the\n\
        rung's, by the code `vc-x1 commit` runs, so a dual workspace gets\n\
        the pair, each half stamped with the other's change id. The ladder\n\
        is rewritten into the commit, the rung `(done)` and none current,\n\
        so the next rung starts by `rung next`, and the cycle's bookmark is\n\
        the one advanced. A work rung gives --body, the text or @PATH. -s is required: `b`\n\
        the pair, `w` the work half alone, `p` a plain repo's commit. --push\n\
        publishes the rung on the cycle's bookmark, a draft. With `[version]\n\
        file` and `pattern` in the config, a commit that changes no line of\n\
        the file starting with the pattern is refused, and --same-version\n\
        commits it as it is. The closing is `cycle close`'s, and a\n\
        single-step's one commit `cycle close --squash`'s. Refused with no\n\
        rung current.\n\n\
        With no cycle in progress, --title commits the work @ verbatim,\n\
        --body required, TODO.md and zTOC.md untouched and not needed, onto\n\
        the bookmark on @-, which it advances, so a run of them iterates on\n\
        one bookmark and `cycle land <TARGET>` lands it once. Refused with no\n\
        bookmark on @-, several, or `main`, development never done on a\n\
        target. --bookmark [NAME] makes or moves one first, the title's slug\n\
        when bare: made on @ when absent, used when on @-, moved forward and\n\
        said when behind @-, and asked about when sideways or ahead, `[y/N]`,\n\
        --yes answering and no terminal refused. `main` is never moved.\n\
        --land <TARGET> lands the commit in the same invocation, as `cycle\n\
        land` does, and with it --push publishes the target."
    )]
    Commit(CommitArgs),
}

/// `rung next`'s arguments.
#[derive(Args, Debug)]
pub struct NextArgs {
    /// The rung's version, set by the config's `[version] bump` before
    /// the rung is marked [default: the version left as it is]
    #[arg(long, value_name = "VERSION")]
    pub ver: Option<String>,

    #[command(flatten)]
    pub at: Where,
}

/// `rung commit`'s arguments.
#[derive(Args, Debug)]
pub struct CommitArgs {
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

    /// The commit's body, the problem and its solution, or @PATH to
    /// read it from a file
    #[arg(long, value_name = "STR|@PATH")]
    pub body: Option<String>,

    /// Publish the cycle's bookmark once committed, a draft push
    #[arg(long)]
    pub push: bool,

    /// Commit although the version's line is unchanged
    #[arg(long)]
    pub same_version: bool,

    /// With no cycle in progress, the commit's title: the work @ is
    /// committed as it is, on a bookmark made for it, the title's slug
    /// or --bookmark's
    #[arg(long, value_name = "STR")]
    pub title: Option<String>,

    /// With --title, land the commit on TARGET in the same invocation,
    /// as `cycle land` does, --push then publishing the target
    #[arg(long, value_name = "TARGET")]
    pub land: Option<String>,

    /// With --title, move the bookmark --bookmark names when it is
    /// sideways or ahead of @-, without asking
    #[arg(long)]
    pub yes: bool,

    #[command(flatten)]
    pub at: Where,
}

/// `rung commit`'s inputs: the commit's, with or without a cycle in
/// progress, the Land that follows, and for `--title`, whether
/// `--bookmark` was given, its name in the running cycle's place, and
/// whether a sideways move is answered.
#[derive(Debug)]
pub struct RungCommit {
    pub rung: CommitRung,
    pub land: Option<LandCycle>,
    pub bookmark: bool,
    pub yes: bool,
}

/// Inputs to the rung op, clap-free.
#[derive(Debug)]
pub enum RungParams {
    Next {
        running: Running,
        ver: Option<String>,
    },
    Stop(Running),
    Commit(Box<RungCommit>),
}

impl TryFrom<&RungArgs> for RungParams {
    type Error = String;

    fn try_from(a: &RungArgs) -> Result<Self, String> {
        match &a.command {
            RungCommand::Next(n) => Ok(RungParams::Next {
                running: n.at.running()?.0,
                ver: match n.ver.as_deref().map(str::trim) {
                    Some("") => return Err("rung next: --ver is blank".into()),
                    v => v.map(String::from),
                },
            }),
            RungCommand::Stop(w) => Ok(RungParams::Stop(w.running()?.0)),
            RungCommand::Commit(c) => {
                let (running, agent) = match &c.title {
                    Some(_) => c.at.running_bare()?,
                    None => c.at.running()?,
                };
                if c.yes && c.title.is_none() {
                    return Err(
                        "rung commit --yes: it answers --bookmark's move, with --title".into(),
                    );
                }
                if let Some(t) = &c.title {
                    crate::commit::refuse_suffixed(t, "rung commit --title")
                        .map_err(|e| e.replacen("commit ", "", 1))?;
                }
                let land = match &c.land {
                    Some(dest) => Some(LandCycle {
                        agent: publish_scope(&c.scope, agent.clone(), "rung commit --land")?,
                        running: running.clone(),
                        dest: dest.clone(),
                        merge: false,
                        skip: None,
                        delete_intermediate: false,
                        push: c.push,
                        title: None,
                        body: None,
                    }),
                    None => None,
                };
                let rung = rung_params(
                    running,
                    agent,
                    &c.scope,
                    c.title.as_deref(),
                    c.body.as_deref(),
                    // With a Land the push is the Land's, the target's.
                    c.push && land.is_none(),
                    c.same_version,
                    "rung commit",
                )?;
                Ok(RungParams::Commit(Box::new(RungCommit {
                    rung,
                    land,
                    bookmark: c.at.bookmark.is_some(),
                    yes: c.yes,
                })))
            }
        }
    }
}

impl SubcommandRunner for RungArgs {
    type Params = RungParams;

    fn to_params(&self) -> Result<Self::Params, String> {
        RungParams::try_from(self)
    }

    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        match params {
            RungParams::Next { running, ver } => next(running, ver.as_deref()),
            RungParams::Stop(r) => stop(r),
            RungParams::Commit(c) => commit(ctx, c),
        }
    }
}

/// The cycle's state as a rung's act needs it: its title, its rung
/// headings, the titles committed on its branch, and the rung marked
/// `(current)`.
struct State {
    cycle: String,
    rungs: Vec<String>,
    commits: Vec<String>,
    current: Option<String>,
}

fn state(r: &Running, act: &str) -> Result<State, Box<dyn std::error::Error>> {
    let bookmark = cycle_bookmark(r, &format!("rung {act}"))?;
    let commits = branch_titles(&r.work, &bookmark)?;
    let path = r.work.join(crate::commit::RECORD);
    let todo = std::fs::read_to_string(&path)
        .map_err(|e| format!("rung {act}: {}: {e}", path.display()))?;
    let cycle = record::cycle_title(&todo).map_err(|e| format!("rung {act}: {e}"))?;
    let rungs = record::rung_titles(&todo).map_err(|e| format!("rung {act}: {e}"))?;
    let current = record::current_rung(&r.work).map_err(|e| format!("rung {act}: {e}"))?;
    Ok(State {
        cycle,
        rungs,
        commits,
        current,
    })
}

impl State {
    /// The rung marked `(current)` that is started, a rung not done.
    fn started(&self) -> Option<&str> {
        self.current
            .as_deref()
            .filter(|c| self.rungs.iter().any(|r| r == c) && !self.commits.iter().any(|r| r == c))
    }
}

/// Write the ladder planned with `current` marked, or with a dry run
/// say so.
fn write_ladder(
    r: &Running,
    s: &State,
    current: Option<&str>,
    act: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let plan = record::plan_ladder(&r.work, &s.commits, current)
        .map_err(|e| format!("rung {act}: {e}"))?;
    for finding in &plan.ladder.findings {
        log::warn!("rung {act}: {finding}");
    }
    match (&plan.edit, r.dry_run) {
        (None, _) => {}
        (Some(_), true) => info!(
            "rung {act}: [dry-run] write the ladder into {}",
            record::LADDER_FILE
        ),
        (Some(edit), false) => std::fs::write(&edit.path, &edit.text)
            .map_err(|e| format!("rung {act}: {}: {e}", edit.path.display()))?,
    }
    Ok(())
}

/// `rung next`: mark the first rung not done `(current)`, after
/// setting the version when `--ver` is given.
fn next(r: &Running, ver: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let s = state(r, "next")?;
    if let Some(c) = s.started() {
        return Err(format!(
            "rung next: the rung '{c}' is started: `rung commit` commits it, or `rung stop` \
             leaves it"
        )
        .into());
    }
    if s.commits.contains(&s.cycle) {
        return Err(format!(
            "rung next: the cycle is one commit, '{}', and closed: it has no rung to start",
            s.cycle
        )
        .into());
    }
    let Some(rung) = s.rungs.iter().find(|r| !s.commits.contains(r)).cloned() else {
        return Err("rung next: every rung is committed, so there is none to start".into());
    };
    // The opening is the plan's commit, `cycle plan commit`'s, and no
    // rung `rung next` starts.
    if rung == format!("{} opening", s.cycle) {
        return Err(
            "rung next: the plan is not committed: `cycle plan commit -s <SCOPE>` commits it, the \
             opening, and `rung next` then starts the first work rung, or for one commit the work \
             then `cycle close --squash -s <SCOPE> --body ...`"
                .into(),
        );
    }
    if let Some(ver) = ver {
        bump(&r.work, ver, r.dry_run, "rung next --ver")
            .map_err(|e| format!("{e}, so no rung is marked"))?;
    }
    write_ladder(r, &s, Some(&rung), "next")?;
    info!(
        "rung next: {}'{rung}' is current",
        if r.dry_run { "[dry-run] " } else { "" }
    );
    Ok(())
}

/// `rung stop`: clear the `(current)` marker, the edits left alone.
fn stop(r: &Running) -> Result<(), Box<dyn std::error::Error>> {
    let s = state(r, "stop")?;
    let Some(current) = s.current.clone() else {
        return Err("rung stop: no rung is current, so there is no marker to clear".into());
    };
    write_ladder(r, &s, None, "stop")?;
    info!(
        "rung stop: {}'{current}' is no longer current, the work @ left as it is",
        if r.dry_run { "[dry-run] " } else { "" }
    );
    Ok(())
}

/// `rung commit`: with a cycle in progress, commit the current rung
/// under its title, the closing refused for `cycle close`, and with
/// none, `--title` commits the work `@` as a cycle of its own.
fn commit(ctx: &mut Context, rc: &RungCommit) -> Result<(), Box<dyn std::error::Error>> {
    let c = &rc.rung;
    let path = c.running.work.join(crate::commit::RECORD);
    let in_progress = match path.is_file() {
        true => record::cycle_title(&std::fs::read_to_string(&path)?).ok(),
        false => None,
    };
    match (in_progress, &c.title) {
        (Some(cycle), Some(_)) => Err(format!(
            "rung commit --title: the cycle '{cycle}' is in progress, and a rung's title is the \
             ladder's: leave --title out"
        )
        .into()),
        (Some(_), None) => commit_current(ctx, rc),
        (None, Some(title)) => commit_alone(ctx, rc, title),
        (None, None) => Err(format!(
            "rung commit: no cycle is in progress in {}: `cycle plan` opens one, or --title \
             commits the work @ as a cycle of its own",
            crate::commit::RECORD
        )
        .into()),
    }
}

/// `rung commit` in a cycle: the current rung, under its title.
fn commit_current(ctx: &mut Context, rc: &RungCommit) -> Result<(), Box<dyn std::error::Error>> {
    let c = &rc.rung;
    let s = state(&c.running, "commit")?;
    let Some(rung) = s.started().map(String::from) else {
        return Err(match &s.current {
            Some(done) => format!(
                "rung commit: the rung marked current, '{done}', is committed already: `vc-x1 \
                 squash` continues it, and `rung next` starts the next"
            ),
            None => "rung commit: no rung is current: `rung next` starts the next".to_string(),
        }
        .into());
    };
    if rung == format!("{} closing", s.cycle) {
        return Err(format!(
            "rung commit: '{rung}' is the cycle's closing, which `cycle close` makes"
        )
        .into());
    }
    if rc.land.is_some() {
        return Err(format!(
            "rung commit --land: '{rung}' is a rung of a ladder, and a Land is of a cycle closed: \
             `cycle close --land` closes and lands it"
        )
        .into());
    }
    let c = CommitRung {
        title: Some(rung),
        ..c.clone()
    };
    commit_rung(ctx, &c, "rung commit")
}

/// `rung commit --title` with no cycle in progress: the work `@`
/// committed verbatim onto a bookmark, the record untouched: the one
/// `--bookmark` names, made or moved first, else the one on `@-`. Every
/// check is made before a bookmark is made or moved.
fn commit_alone(
    ctx: &mut Context,
    rc: &RungCommit,
    title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let c = &rc.rung;
    let label = "rung commit --title";
    let work = c.running.work.as_path();
    let dry_run = c.running.dry_run;
    let body = c.body.clone().ok_or_else(|| {
        format!("{label}: the commit holds the work, so it gives --body, its problem and solution")
    })?;
    if jj::is_empty(work, "@")? {
        return Err(format!("{label}: the work @ is empty, so there is nothing to commit").into());
    }
    crate::commit::check_version(work, "@", c.same_version, label)?;
    if let Some(l) = &rc.land {
        land_checks(work, &l.dest, Some("@-"), l.agent.is_none())?;
    }
    let target = |name: &str| name == "main" || rc.land.as_ref().is_some_and(|l| l.dest == name);
    let (bookmark, place) = if rc.bookmark {
        let name = crate::open::bookmark_name(c.running.bookmark.as_deref(), Some(title))?;
        if target(&name) {
            return Err(format!(
                "{label} --bookmark: '{name}' is a target, and development is never done on one: \
                 name another"
            )
            .into());
        }
        let place = placement(work, &name)?;
        (name, place)
    } else {
        let on = jj::local_bookmarks_at(work, "@-")?;
        let name = match on.as_slice() {
            [one] => one.clone(),
            [] => {
                return Err(format!(
                    "{label}: no bookmark is on @-, so the commit would advance none: --bookmark \
                     makes one, the title's slug, or --bookmark <NAME>"
                )
                .into());
            }
            many => {
                return Err(format!(
                    "{label}: several bookmarks are on @-, {}: --bookmark <NAME> says which",
                    many.join(", ")
                )
                .into());
            }
        };
        if target(&name) {
            return Err(format!(
                "{label}: the bookmark on @- is '{name}', a target, and development is never done \
                 on one: --bookmark makes one, the title's slug, or --bookmark <NAME>"
            )
            .into());
        }
        (name, Place::On)
    };
    if let Place::Sideways(at) = &place {
        info!(
            "{label}: '{bookmark}' is at {at}, not below @-, and moving it to @- leaves the \
             commits it reaches that @- does not"
        );
        if !dry_run {
            confirm(rc.yes, ctx.stdin_is_tty, &bookmark)?;
        }
    }
    let dry = if dry_run { "[dry-run] " } else { "" };
    match &place {
        Place::Absent => {
            let open = crate::open::OpenParams {
                work: work.to_path_buf(),
                agent: c.agent.as_ref().map(|(a, _)| a.clone()),
                bookmark: bookmark.clone(),
                dry_run,
                label,
            };
            crate::open::open(ctx, &open)?;
        }
        Place::On => info!("{label}: {dry}'{bookmark}' is on @-, and the commit advances it"),
        Place::Behind(at) | Place::Sideways(at) => {
            info!("{label}: {dry}move '{bookmark}' from {at} to @-, and the commit advances it");
            if !dry_run {
                ctx.session(work)?.bookmark_set(&bookmark, "@-")?;
            }
        }
    }
    if dry_run {
        info!("{label}: [dry-run] commit the work @ as '{title}' on '{bookmark}'");
        return Ok(());
    }
    let mut params = commit_params(c, title.to_string(), body, bookmark.clone());
    params.land_follows = rc.land.is_some();
    crate::commit::commit(ctx, &params)?;
    match &rc.land {
        Some(l) => land_closed(ctx, l, Some(bookmark)),
        None => Ok(()),
    }
}

/// Where a bookmark `--bookmark` names is, against `@-`.
#[derive(Debug, PartialEq, Eq)]
enum Place {
    /// Not made yet.
    Absent,
    /// On `@-`.
    On,
    /// Below `@-`, at the short id, moved forward with nothing left.
    Behind(String),
    /// Sideways or ahead of `@-`, at the short id, moved only when asked.
    Sideways(String),
}

/// Where `name` is against the work `@-`.
fn placement(work: &Path, name: &str) -> Result<Place, Box<dyn std::error::Error>> {
    if !jj::local_bookmark_exists(work, name)? {
        return Ok(Place::Absent);
    }
    let own = format!("bookmarks(exact:\"{name}\")");
    if jj::matches(work, &format!("@- & {own}"))? {
        return Ok(Place::On);
    }
    let at = jj::cid_short_of(work, &own)?;
    Ok(if jj::matches(work, &format!("{own} & ::@-"))? {
        Place::Behind(at)
    } else {
        Place::Sideways(at)
    })
}

/// Ask before a sideways move, unless `--yes`. With no terminal to ask,
/// it is refused rather than hung, and a declined answer is an error.
fn confirm(yes: bool, stdin_is_tty: bool, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let label = "rung commit --title";
    if yes {
        return Ok(());
    }
    if !stdin_is_tty {
        return Err(format!(
            "{label}: moving '{name}' sideways asks, and stdin is not a terminal: --yes moves it, \
             or --bookmark <NAME> names another"
        )
        .into());
    }
    let answer = crate::common::prompt(&format!("{label}: move '{name}' to @-? [y/N] "))?;
    let answer = answer.trim().to_ascii_lowercase();
    if answer != "y" && answer != "yes" {
        return Err(
            format!("{label}: declined (got {answer:?}), nothing moved or committed").into(),
        );
    }
    Ok(())
}

/// The config's `[version] bump` as words, `{ver}` replaced by `ver`,
/// refused when the config has no `bump` or it names no `{ver}`.
pub(super) fn bump_words(
    work: &Path,
    ver: &str,
    label: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let command = crate::config_md::load(work)?
        .and_then(|cfg| crate::toml_simple::toml_get(&cfg.map, "version.bump").cloned())
        .ok_or_else(|| {
            format!(
                "{label}: the config has no `[version] bump`, the command that sets the version: \
                 add it, or bump the version by hand and leave --ver out"
            )
        })?;
    if !command.contains("{ver}") {
        return Err(format!(
            "{label}: `[version] bump`, `{command}`, has no `{{ver}}` for the version"
        )
        .into());
    }
    Ok(command
        .split_whitespace()
        .map(|w| w.replace("{ver}", ver))
        .collect())
}

/// Set the version to `ver` by the config's `[version] bump`, run from
/// the workspace root, `label` the command and flag that asked.
pub(super) fn bump(
    work: &Path,
    ver: &str,
    dry_run: bool,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let words = bump_words(work, ver, label)?;
    let line = words.join(" ");
    if dry_run {
        info!("{label}: [dry-run] run `{line}`");
        return Ok(());
    }
    let Some((program, args)) = words.split_first() else {
        return Err(format!("{label}: `[version] bump` is empty").into());
    };
    info!("{label}: run `{line}`");
    // Register entry 8 (clippy.toml): the project's version is set by
    // its own tool, the command `[version] bump` names, which is the
    // whole job of `rung next --ver`.
    #[allow(clippy::disallowed_methods)]
    let status = std::process::Command::new(program)
        .args(args)
        .current_dir(work)
        .status()
        .map_err(|e| format!("{label}: cannot run `{line}`: {e}"))?;
    if !status.success() {
        return Err(format!("{label}: `{line}` failed ({status})").into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The command is split at whitespace and `{ver}` filled, and a
    /// config without `bump`, or one with no `{ver}`, is refused.
    #[test]
    fn the_bump_command_is_the_configs() {
        let dir = crate::test_helpers::unique_base("rung-bump");
        std::fs::create_dir_all(&dir).expect("scratch dir");
        let write = |bump: &str| {
            std::fs::write(
                dir.join(".vc-config.md"),
                format!("# c\n\n```toml\n[version]\nfile = \"Cargo.toml\"\npattern = 'version = '\n{bump}```\n"),
            )
            .expect("write config");
        };
        write("bump = \"cargo xtask bump {ver}\"\n");
        assert_eq!(
            bump_words(&dir, "1.2.3-1", "rung next --ver").expect("words"),
            ["cargo", "xtask", "bump", "1.2.3-1"]
        );
        write("");
        let err = bump_words(&dir, "1.2.3", "rung next --ver")
            .unwrap_err()
            .to_string();
        assert!(err.contains("no `[version] bump`"), "{err}");
        write("bump = \"cargo xtask bump\"\n");
        let err = bump_words(&dir, "1.2.3", "rung next --ver")
            .unwrap_err()
            .to_string();
        assert!(err.contains("no `{ver}`"), "{err}");
        std::fs::remove_dir_all(&dir).expect("remove scratch dir");
    }
}

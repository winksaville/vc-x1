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
//! The marker lives in the working copy alone: a commit ends its rung
//! and starts none, so no commit's tree claims work not yet begun.

use std::path::Path;

use clap::{Args, Subcommand};
use log::info;

use super::{
    CommitRung, Running, Where, branch_titles, commit_rung, cycle_bookmark, record, rung_params,
};
use crate::context::Context;
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
        in the working copy's zTOC.md, refused while a rung is current.\n\
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
        the one advanced. The opening's body defaults to `[opening] body`,\n\
        and a work rung gives --body, the text or @PATH. -s is required: `b`\n\
        the pair, `w` the work half alone, `p` a plain repo's commit. --push\n\
        publishes the rung on the cycle's bookmark, a draft. With `[version]\n\
        file` and `pattern` in the config, a commit that changes no line of\n\
        the file starting with the pattern is refused, and --same-version\n\
        commits it as it is. The closing is `cycle close`'s. Refused with\n\
        no rung current."
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
    /// read it from a file [default: `[opening] body` for the opening]
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

/// Inputs to the rung op, clap-free.
#[derive(Debug)]
pub enum RungParams {
    Next {
        running: Running,
        ver: Option<String>,
    },
    Stop(Running),
    Commit(CommitRung),
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
                let (running, agent) = c.at.running()?;
                Ok(RungParams::Commit(rung_params(
                    running,
                    agent,
                    &c.scope,
                    None,
                    c.body.as_deref(),
                    c.push,
                    c.same_version,
                    "rung commit",
                )?))
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
    if let Some(ver) = ver {
        bump(&r.work, ver, r.dry_run)?;
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

/// `rung commit`: commit the current rung under its title, the
/// closing refused for `cycle close`.
fn commit(ctx: &mut Context, c: &CommitRung) -> Result<(), Box<dyn std::error::Error>> {
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
    let c = CommitRung {
        running: Running {
            work: c.running.work.clone(),
            bookmark: c.running.bookmark.clone(),
            dry_run: c.running.dry_run,
        },
        agent: c.agent.clone(),
        title: Some(rung),
        body: c.body.clone(),
        push: c.push,
        same_version: c.same_version,
    };
    commit_rung(ctx, &c, "rung commit")
}

/// The config's `[version] bump` as words, `{ver}` replaced by `ver`,
/// refused when the config has no `bump` or it names no `{ver}`.
fn bump_words(work: &Path, ver: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let command = crate::config_md::load(work)?
        .and_then(|cfg| crate::toml_simple::toml_get(&cfg.map, "version.bump").cloned())
        .ok_or(
            "rung next --ver: the config has no `[version] bump`, the command that sets the \
             version: add it, or bump the version by hand and leave --ver out",
        )?;
    if !command.contains("{ver}") {
        return Err(format!(
            "rung next --ver: `[version] bump`, `{command}`, has no `{{ver}}` for the version"
        )
        .into());
    }
    Ok(command
        .split_whitespace()
        .map(|w| w.replace("{ver}", ver))
        .collect())
}

/// Set the version to `ver` by the config's `[version] bump`, run from
/// the workspace root.
fn bump(work: &Path, ver: &str, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let words = bump_words(work, ver)?;
    let line = words.join(" ");
    if dry_run {
        info!("rung next: [dry-run] run `{line}`");
        return Ok(());
    }
    let Some((program, args)) = words.split_first() else {
        return Err("rung next --ver: `[version] bump` is empty".into());
    };
    info!("rung next: run `{line}`");
    // Register entry 8 (clippy.toml): the project's version is set by
    // its own tool, the command `[version] bump` names, which is the
    // whole job of `rung next --ver`.
    #[allow(clippy::disallowed_methods)]
    let status = std::process::Command::new(program)
        .args(args)
        .current_dir(work)
        .status()
        .map_err(|e| format!("rung next --ver: cannot run `{line}`: {e}"))?;
    if !status.success() {
        return Err(
            format!("rung next --ver: `{line}` failed ({status}), so no rung is marked").into(),
        );
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
            bump_words(&dir, "1.2.3-1").expect("words"),
            ["cargo", "xtask", "bump", "1.2.3-1"]
        );
        write("");
        let err = bump_words(&dir, "1.2.3").unwrap_err().to_string();
        assert!(err.contains("no `[version] bump`"), "{err}");
        write("bump = \"cargo xtask bump\"\n");
        let err = bump_words(&dir, "1.2.3").unwrap_err().to_string();
        assert!(err.contains("no `{ver}`"), "{err}");
        std::fs::remove_dir_all(&dir).expect("remove scratch dir");
    }
}

//! The `cycle` subcommand: a cycle's acts, each a stage named by a
//! flag, taken out of `commit`, `open`, and `land`.
//!
//! - `--open` opens a cycle: the bookmark first, on the work `@`, as
//!   [`crate::open`] makes it, then the cycle's record, the edits
//!   [`record`] plans. Everything is planned and checked before
//!   anything is made, so a refusal leaves the workspace as it was.
//! - `--ladder` makes `zTOC.md`'s ladder from the rung headings in
//!   `TODO.md`, each marker computed from the commits on the cycle's
//!   branch, so a rung is written once and no marker is typed.
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
#[group(id = "stage", required = true, args = ["open", "ladder", "commit"])]
pub struct CycleArgs {
    /// Open a cycle: make its bookmark on the work `@`, then write its
    /// record, `TODO.md`'s `## In Progress` scaffold and `zTOC.md`'s
    /// ladder
    #[arg(long, help_heading = "Stages", conflicts_with_all = ["ladder", "commit"])]
    pub open: bool,

    /// Make `zTOC.md`'s ladder from the rung headings under `####
    /// Rungs` in `TODO.md`, each marker computed from the cycle's
    /// commits
    #[arg(long, help_heading = "Stages", conflicts_with_all = ["commit", "title"])]
    pub ladder: bool,

    /// Commit the working copies as a rung of the cycle in progress,
    /// the ladder rewritten into it: the cycle's first commit is its
    /// opening, its title and body defaulted, and a work rung gives
    /// both
    #[arg(long, help_heading = "Stages")]
    pub commit: bool,

    /// With `--open`, the bare cycle title, a Conventional Commit: the
    /// bookmark is its slug, and the record is written under it. With
    /// `--commit`, the rung's title, a heading under `#### Rungs`
    /// [default: the opening's, for a cycle's first commit]
    #[arg(long, value_name = "STR")]
    pub title: Option<String>,

    /// With `--commit`, the rung's body, the problem and its solution,
    /// or @PATH to read it from a file [default: `[opening] body`, for
    /// a cycle's first commit]
    #[arg(long, value_name = "STR|@PATH", requires = "commit")]
    pub body: Option<String>,

    /// The cycle's bookmark, in place of the title's slug: the name
    /// `--open` makes, and the one another stage reads the cycle's
    /// commits from and advances
    #[arg(long, value_name = "NAME")]
    pub bookmark: Option<String>,

    /// With `--open`, make the bookmark alone and write no record
    #[arg(long, requires = "open", help_heading = "Open")]
    pub no_record: bool,

    /// With `--commit`, which repos: `b|both` both commits now,
    /// `w|work` the work half with the agent half left to push, and
    /// `p|por` a plain repo's commit [default: `b` in a dual
    /// workspace, `p` in a plain repo]
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = repo_scope_parser(&["work", "both", "por"]),
        requires = "commit",
        help_heading = "Commit"
    )]
    pub scope: Option<RepoScope>,

    /// With `--commit`, publish the rung on the cycle's bookmark, a
    /// draft push, a commit otherwise staying on the machine
    #[arg(long, requires = "commit", help_heading = "Commit")]
    pub push: bool,

    /// Where: a path in the workspace, its root or either repo, or a
    /// plain repo [default: the current directory]
    #[arg(short = 'R', long, value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Say what each stage would do, and do none of it
    #[command(flatten)]
    pub dry_run: DryRunFlag,
}

/// `--open`'s inputs: the bookmark's, as `open` takes them, and the
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
}

/// Inputs to the cycle op, flat, owned, clap-free, a stage each.
#[derive(Debug)]
pub enum CycleParams {
    Open(OpenCycle),
    Ladder(Running),
    Commit(CommitRung),
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
/// made now, by `-s`: both in a dual workspace and the plain repo's
/// commit in a plain one when it is left out, and refused when it
/// asks for a pair where there is none or a plain commit where there
/// is one.
fn commit_scope(
    scope: Option<&RepoScope>,
    agent: Option<PathBuf>,
    stage: &str,
) -> Result<Option<(PathBuf, bool)>, String> {
    match (scope, agent) {
        (None, agent) => Ok(agent.map(|a| (a, true))),
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
        let bookmark = crate::open::bookmark_name(self.bookmark.as_deref(), self.title.as_deref())
            .map_err(|e| e.replacen("open", "cycle --open", 1))?;
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

    /// `--commit`'s params.
    fn commit_params(&self, work: PathBuf, agent: Option<PathBuf>) -> Result<CycleParams, String> {
        let body = match &self.body {
            Some(b) => Some(crate::desc_helpers::read_body(b, "cycle --commit")?),
            None => None,
        };
        if self.title.as_deref().is_some_and(|t| t.trim().is_empty())
            || body.as_deref().is_some_and(|b| b.trim().is_empty())
        {
            return Err("cycle --commit: a --title or --body given is not blank".into());
        }
        Ok(CycleParams::Commit(CommitRung {
            agent: commit_scope(self.scope.as_ref(), agent, "--commit")?,
            running: self.running(work),
            title: self.title.as_ref().map(|t| t.trim().to_string()),
            body,
            push: self.push,
        }))
    }
}

impl TryFrom<&CycleArgs> for CycleParams {
    type Error = String;

    /// Resolve the workspace and the stage asked for.
    fn try_from(a: &CycleArgs) -> Result<Self, String> {
        let (work, agent) = resolve_repos(a.repo.as_deref())?;
        if a.open {
            a.open_params(work, agent)
        } else if a.ladder {
            Ok(CycleParams::Ladder(a.running(work)))
        } else {
            a.commit_params(work, agent)
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
    let target = match &c.agent {
        Some((agent, agent_now)) => crate::commit::Target::Pair {
            pair: crate::pair::Pair {
                work: r.work.clone(),
                agent: agent.clone(),
            },
            agent_now: *agent_now,
        },
        None => crate::commit::Target::Por(r.work.clone()),
    };
    let params = crate::commit::CommitParams {
        target,
        title: Some(title),
        body: Some(body),
        closing: None,
        opening: None,
        advance: crate::commit::Advance::Named(bookmark),
        push: c.push,
        land: false,
        skip: None,
        no_url_trailer: false,
        dry_run: r.dry_run,
        rev: None,
        covers: None,
    };
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

/// The titles of the commits on the cycle's branch in `repo`, oldest
/// first: the commits `bookmark` reaches, and those above it up to the
/// work `@-`, that no other local bookmark reaches, so no destination
/// is named or stored, less a working copy not yet described.
pub(crate) fn branch_titles(
    repo: &std::path::Path,
    bookmark: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let own = format!("bookmarks(exact:\"{bookmark}\")");
    // The commits above the bookmark up to the work `@-` are the
    // branch's too, a commit made by hand not having advanced it.
    let branch = format!("(::({own}) | ({own})::@-) ~ ::(bookmarks() ~ {own})");
    let mut titles = Vec::new();
    for cid in jj::cids_of(repo, &branch)? {
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
fn open(ctx: &mut Context, o: &OpenCycle) -> Result<(), Box<dyn std::error::Error>> {
    let dry_run = o.open.dry_run;
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

    fn params(repo: &Path, args: &[&str]) -> Result<CycleParams, String> {
        let mut all = vec!["vc-x1", "cycle"];
        all.extend_from_slice(args);
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

    /// `--no-record`, a bookmark with no title, and a workspace with no
    /// `TODO.md` each get the bookmark alone.
    #[test]
    fn open_makes_the_bookmark_alone() {
        let fx = Fixture::new("cycle-open-alone");
        run(&fx.work, &["--open", "--title", "feat: a cycle"]).expect("no TODO.md");
        assert_eq!(bookmarks_at(&fx.work), "feat-a-cycle");
        assert!(!fx.work.join("TODO.md").exists());

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

    /// `-s` is the pair in a dual workspace unless it says the work
    /// half alone, and a plain repo's commit carries no trailer.
    #[test]
    fn the_scope_follows_the_workspace() {
        let fx = planned("cycle-commit-scope");
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
        run(&por.work, &["--commit"]).expect("the opening");
        let desc = description(&por.work, "@-");
        assert!(desc.starts_with("feat: a cycle opening\n"), "{desc}");
        assert!(!desc.contains("ochid:"), "{desc}");
    }
}

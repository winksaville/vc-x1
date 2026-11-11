//! The `cycle` subcommand: a cycle's acts, each a stage named by a
//! flag, taken out of `commit`, `open`, and `land`.
//!
//! - `--open` opens a cycle: the bookmark first, on the work `@`, as
//!   [`crate::open`] makes it, then the cycle's record, the edits
//!   [`record`] plans. Everything is planned and checked before
//!   anything is made, so a refusal leaves the workspace as it was.
//! - The record is the agent-files' shape and not the work's, so the
//!   tool writes it without knowing the work. The version is the
//!   project's, and no stage bumps it.
//! - A stage is a flag and not a subcommand, the form every `vc-x1`
//!   command has.

pub(crate) mod record;

use std::path::PathBuf;

use clap::Args;
use log::{debug, info};

use crate::context::Context;
use crate::open::OpenParams;
use crate::options_flags::dry_run::DryRunFlag;
use crate::options_flags::scope::RepoScope;
use crate::subcommand::SubcommandRunner;

/// Run a cycle's stages.
#[derive(Args, Debug)]
#[group(id = "stage", required = true, args = ["open"])]
pub struct CycleArgs {
    /// Open a cycle: make its bookmark on the work `@`, then write its
    /// record, `TODO.md`'s `## In Progress` scaffold and `zTOC.md`'s
    /// ladder
    #[arg(long, help_heading = "Stages")]
    pub open: bool,

    /// The bare cycle title, a Conventional Commit: the bookmark is its
    /// slug, and the record is written under it
    #[arg(long, value_name = "STR")]
    pub title: Option<String>,

    /// With `--open`, the bookmark's name, in place of the title's slug
    #[arg(long, value_name = "NAME", requires = "open", help_heading = "Open")]
    pub bookmark: Option<String>,

    /// With `--open`, make the bookmark alone and write no record
    #[arg(long, requires = "open", help_heading = "Open")]
    pub no_record: bool,

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

/// Inputs to the cycle op, flat, owned, clap-free, a stage each.
#[derive(Debug)]
pub enum CycleParams {
    Open(OpenCycle),
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

impl TryFrom<&CycleArgs> for CycleParams {
    type Error = String;

    /// Resolve the workspace and the stage asked for.
    fn try_from(a: &CycleArgs) -> Result<Self, String> {
        let (work, agent) = resolve_repos(a.repo.as_deref())?;
        if a.title.is_none() && a.bookmark.is_none() {
            return Err("cycle --open: --title or --bookmark names the bookmark".into());
        }
        let bookmark = crate::open::bookmark_name(a.bookmark.as_deref(), a.title.as_deref())
            .map_err(|e| e.replacen("open", "cycle --open", 1))?;
        if let Some(t) = &a.title {
            crate::commit::refuse_suffixed(t, "cycle --open --title")
                .map_err(|e| e.replacen("commit ", "", 1))?;
        }
        Ok(CycleParams::Open(OpenCycle {
            open: OpenParams {
                work,
                agent,
                bookmark,
                dry_run: a.dry_run.value,
            },
            record: match (&a.title, a.no_record) {
                (Some(t), false) => Some(t.trim().to_string()),
                _ => None,
            },
        }))
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
    }
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
    use crate::test_helpers::{Fixture, jj_ok, test_ctx};
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
}

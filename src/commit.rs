//! The `commit` subcommand: make a rung's commit pair, one commit
//! per repo of a dual workspace, each stamped with the other's
//! change id, before anything is pushed.
//!
//! The pair is what the per-rung flow reviews: the description and
//! the diff together, as a commit, by `jj show @-` in either repo.
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
//! - `-s a` is refused: an agent commit made first names a work `@`
//!   that push would then commit with a trailer of its own, so that
//!   direction is where an unpaired commit is real.
//! - `-s p` commits a plain repo with the title and body and no
//!   trailer, the same description discipline on a repo with no
//!   partner.
//! - `--body` is the text, or `@PATH` to read it from a file, since
//!   a body in commit-body form has markers and blank lines a shell
//!   quote mangles.
//! - The work side's `@` must hold a change, since a rung changes
//!   the work repo. The agent side's `@` is the session's journal
//!   and is committed as it stands.
//! - A failure after the first commit restores both repos from the
//!   `jj op` snapshots taken just before it, as push does inside its
//!   mutation window.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info};

use crate::context::Context;
use crate::desc_helpers::{ochid_prefix_for, with_trailer};
use crate::jj;
use crate::options_flags::dry_run::DryRunFlag;
use crate::options_flags::scope::{REPO_SCOPE_FORMS, RepoScope, parse_repo_scope};
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
    /// `p|por` a plain repo's commit, and `a` is refused
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = parse_repo_scope,
        required = true,
        help = format!("Which repos, required: {REPO_SCOPE_FORMS}, `a` refused")
    )]
    pub scope: RepoScope,

    /// Commit title, the Conventional Commit line
    #[arg(long, value_name = "STR", required = true)]
    pub title: String,

    /// Commit body, the problem and its solutions, or @PATH to read
    /// it from a file
    #[arg(long, value_name = "STR|@PATH", required = true)]
    pub body: String,

    /// Print the descriptions it would write, trailers included, and
    /// commit nothing
    #[command(flatten)]
    pub dry_run: DryRunFlag,
}

/// Inputs to the commit op, flat, owned, clap-free: the repos `-s`
/// and `-R` resolved to, and the description.
#[derive(Debug)]
pub struct CommitParams {
    /// The pair, work then agent, or the one plain repo.
    pub target: Target,
    pub title: String,
    pub body: String,
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
}

impl TryFrom<&CommitArgs> for CommitParams {
    type Error = String;

    /// Resolve the scope into its repos, refusing the agent half
    /// alone, read the body when it names a file, and check the
    /// description is not blank.
    fn try_from(a: &CommitArgs) -> Result<Self, String> {
        let body = crate::desc_helpers::read_body(&a.body, "commit")?;
        if a.title.trim().is_empty() || body.trim().is_empty() {
            return Err("commit: --title and --body must both be non-empty".into());
        }
        if let RepoScope::Sides(sides) = &a.scope
            && sides.is_bot_only()
        {
            return Err(
                "commit -s a: the agent half alone is an unpaired commit, since push \
                        would commit the work @ with a trailer of its own. Use `-s w` for the \
                        work half now and the agent half at push, or `-s b` for both now"
                    .into(),
            );
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
                Target::Pair {
                    pair,
                    agent_now: sides.is_both(),
                }
            }
        };
        Ok(CommitParams {
            target,
            title: a.title.clone(),
            body,
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
    preflight(work, "work", true)?;
    preflight(agent, "agent", false)?;
    let work_chid = jj::chid_of(work, "@")?;
    let agent_chid = jj::chid_of(agent, "@")?;
    let agent_prefix = ochid_prefix_for(agent)?;
    let text = format!("{}\n\n{}", params.title, params.body);
    let work_desc = with_trailer(&text, &format!("{agent_prefix}{agent_chid}"));
    let agent_desc = with_trailer(&text, &format!("/{work_chid}"));

    if params.dry_run {
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
        return Ok(());
    }
    let snapshots = Snapshots::take(pair)?;
    info!("commit work: {} -> @- {work_chid}", work.display());
    ctx.session(work)?.commit(&work_desc)?;
    if !agent_now {
        info!(
            "commit: the work half is made, its trailer naming the agent @ {agent_chid}, which \
             push commits as the partner. Review it with `jj show @- -R {}`",
            work.display()
        );
        return Ok(());
    }
    info!("commit agent: {} -> @- {agent_chid}", agent.display());
    if let Err(e) = ctx.session(agent)?.commit(&agent_desc) {
        snapshots.restore(pair, "commit", e.as_ref());
        return Err(e);
    }
    info!(
        "commit: the pair is made, review it with `jj show @- -R {}` and `jj show @- -R {}`",
        work.display(),
        agent.display()
    );
    Ok(())
}

/// Commit a plain repo with the description and no trailer.
fn commit_por(
    ctx: &mut Context,
    repo: &Path,
    params: &CommitParams,
) -> Result<(), Box<dyn std::error::Error>> {
    preflight(repo, "plain", true)?;
    let chid = jj::chid_of(repo, "@")?;
    let desc = format!("{}\n\n{}", params.title, params.body);
    if params.dry_run {
        info!(
            "commit: [dry-run] the commit {chid}, in {}:\n{desc}",
            repo.display()
        );
        return Ok(());
    }
    info!("commit: {} -> @- {chid}", repo.display());
    ctx.session(repo)?.commit(&desc)?;
    Ok(())
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
        a.body = "  \n".to_string();
        let err = CommitParams::try_from(&a).unwrap_err();
        assert!(err.contains("non-empty"), "{err}");
    }

    /// The agent half alone is refused by name, and `-s p` on a
    /// side is the resolver's refusal.
    #[test]
    fn the_agent_half_and_por_on_a_side_are_refused() {
        let fx = Fixture::new("commit-half");
        let err = CommitParams::try_from(&args("a", &fx.work)).unwrap_err();
        assert!(err.starts_with("commit -s a:"), "{err}");
        assert!(err.contains("`-s w`") && err.contains("`-s b`"), "{err}");
        let err = CommitParams::try_from(&args("p", &fx.bot)).unwrap_err();
        assert!(err.contains("the agent side"), "{err}");
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

        // Nothing moved: `main` is still the commit below the pair.
        assert_eq!(chid(&fx.work, "main"), chid(&fx.work, "@--"));
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
        a.body = format!("@{}", body_path.display());
        let params = CommitParams::try_from(&a).expect("params");
        assert_eq!(params.body, body);
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

        a.body = format!("@{}", fx.base.join("missing.md").display());
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

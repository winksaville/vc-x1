//! The `squash` subcommand: continue a rung by folding the working
//! copies into the pair `commit` made, before it is pushed.
//!
//! A review's fix is edited in the work `@` and folded into the work
//! commit, and the journal the review wrote is folded into the agent
//! commit, so the pair stays one rung's two commits. A squash keeps
//! the destination's change id, so both trailers stay right without
//! being touched.
//!
//! - `-s b` folds both sides, `-s w` the work side, `-s a` the agent
//!   side, and `-s p` a plain repo's `@` into its `@-`.
//! - The agent side goes by where the partner stands: committed
//!   (`commit -s b`), so the journal folds into it; pending (`commit
//!   -s w`), so the journal already is the partner and push commits
//!   it; or changed since the commit, which push restamps. Only the
//!   first has anything to fold.
//! - A published side is refused: rewriting it is a remote rewrite,
//!   which stays an explicit, approved act outside this command.
//! - `--title` with `--body` rewrites the descriptions, keeping each
//!   trailer, the one sanctioned re-describe of a trailer-carrying
//!   commit: by this command, and never of a published one.
//! - A failure after the first mutation restores both repos from
//!   `jj op` snapshots.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info};

use crate::context::Context;
use crate::desc_helpers::{read_body, with_trailer};
use crate::jj;
use crate::options_flags::dry_run::DryRunFlag;
use crate::options_flags::scope::{
    ALL_REPO_SCOPES, REPO_SCOPE_FORMS, RepoScope, Scope, repo_scope_parser,
};
use crate::pair::{
    Pair, Partner, Snapshots, WorkCommit, is_published, partner, refuse_conflicts, resolve_pair,
    work_commit,
};
use crate::subcommand::SubcommandRunner;

/// Fold the working copies into the pair, or a plain repo's `@`
/// into its `@-`, optionally rewriting the description.
#[derive(Args, Debug)]
pub struct SquashArgs {
    /// Where: a path in the workspace for `-s b|w|a`, its root or
    /// either repo, or the repo itself for `-s p` [default: the
    /// current directory]
    #[arg(short = 'R', long, value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Which sides, required: `b|both`, `w|work`, `a|agent` of the
    /// pair, or `p|por` a plain repo
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = repo_scope_parser(ALL_REPO_SCOPES),
        required = true,
        help = format!("Which sides, required: {REPO_SCOPE_FORMS}")
    )]
    pub scope: RepoScope,

    /// A new title for the pair's descriptions, with --body
    #[arg(long, value_name = "STR", requires = "body")]
    pub title: Option<String>,

    /// A new body, or @PATH to read it from a file, with --title
    #[arg(long, value_name = "STR|@PATH", requires = "title")]
    pub body: Option<String>,

    /// Print what it would fold and re-describe, and change nothing
    #[command(flatten)]
    pub dry_run: DryRunFlag,
}

/// Inputs to the squash op, flat, owned, clap-free.
#[derive(Debug)]
pub struct SquashParams {
    pub target: Target,
    /// The new description's text, title and body joined, when one
    /// was given.
    pub text: Option<String>,
    /// Print the steps and change nothing.
    pub dry_run: bool,
}

/// What `-s` resolved to.
#[derive(Debug)]
pub enum Target {
    /// The pair and the sides of it to fold.
    Pair { pair: Pair, sides: Scope },
    /// A plain repo.
    Por(PathBuf),
}

impl TryFrom<&SquashArgs> for SquashParams {
    type Error = String;

    /// Resolve the scope, and read and join the description when one
    /// was given, refusing a blank one.
    fn try_from(a: &SquashArgs) -> Result<Self, String> {
        let text = match (&a.title, &a.body) {
            (Some(title), Some(body)) => {
                let body = read_body(body, "squash")?;
                if title.trim().is_empty() || body.trim().is_empty() {
                    return Err("squash: --title and --body must both be non-empty".into());
                }
                Some(format!("{title}\n\n{body}"))
            }
            _ => None,
        };
        let target = match &a.scope {
            RepoScope::Por => {
                let repos = crate::common::resolve_scoped_repos(&a.scope, a.repo.as_deref())
                    .map_err(|e| e.to_string())?;
                match repos.as_slice() {
                    [repo] => Target::Por(repo.clone()),
                    other => {
                        return Err(format!(
                            "squash -s p: expected one repo, got {}",
                            other.len()
                        ));
                    }
                }
            }
            RepoScope::Sides(sides) => Target::Pair {
                pair: resolve_pair(a.repo.as_deref()).map_err(|e| e.to_string())?,
                sides: sides.clone(),
            },
        };
        Ok(SquashParams {
            target,
            text,
            dry_run: a.dry_run.value,
        })
    }
}

impl SubcommandRunner for SquashArgs {
    type Params = SquashParams;

    /// Delegate to the `TryFrom<&SquashArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        SquashParams::try_from(self)
    }

    /// Run the `squash` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        squash(ctx, params)
    }
}

/// Entry point: the pair's sides, or the plain repo.
pub fn squash(ctx: &mut Context, params: &SquashParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("squash: enter {params:?}");
    match &params.target {
        Target::Pair { pair, sides } => {
            squash_pair(ctx, pair, sides, params.text.as_deref(), params.dry_run)
        }
        Target::Por(repo) => squash_por(ctx, repo, params.text.as_deref(), params.dry_run),
    }
}

/// Refuse rewriting `rev` in `repo` when it is published.
fn refuse_published(repo: &Path, label: &str) -> Result<(), Box<dyn std::error::Error>> {
    if is_published(repo, "@-")? {
        return Err(format!(
            "squash: the {label} @- is published, so folding into it or re-describing it is a \
             remote rewrite. Make the change the next rung, or rewrite it by hand with approval"
        )
        .into());
    }
    Ok(())
}

/// What the agent side of a pair squash does, decided before any
/// mutation.
#[derive(Debug, PartialEq, Eq)]
enum AgentStep {
    /// Fold the journal into the committed partner, and re-describe
    /// it when a description was given.
    Fold,
    /// Nothing: the partner is pending or changed, push's to commit.
    Leave,
}

/// Fold the scoped sides of the pair, refusing before any mutation
/// a work commit that is not a pair's, a conflicted or published
/// side, and a run with nothing to do.
fn squash_pair(
    ctx: &mut Context,
    pair: &Pair,
    sides: &Scope,
    text: Option<&str>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(work) = work_commit(pair)? else {
        return Err(
            "squash: the work repo's @- carries no `ochid:` trailer, so it is not a \
                    pair's commit. Make the pair with `vc-x1 commit` first"
                .into(),
        );
    };
    let work_fold = sides.has_work() && !jj::is_empty(&pair.work, "@")?;
    if sides.has_work() {
        refuse_conflicts(&pair.work, "squash", "work")?;
        if work_fold || text.is_some() {
            refuse_published(&pair.work, "work")?;
        }
    }
    let agent = if sides.has_bot() {
        agent_step(pair, &work, text)?
    } else {
        AgentStep::Leave
    };
    let agent_fold = agent == AgentStep::Fold && !jj::is_empty(&pair.agent, "@")?;
    let work_describe = sides.has_work() && text.is_some();
    let agent_describe = agent == AgentStep::Fold && text.is_some();
    if !(work_fold || work_describe || agent_fold || agent_describe) {
        info!("squash: nothing to fold and no description to write");
        return Ok(());
    }
    if dry_run {
        let say = |done: bool, what: &str| {
            if done {
                info!("squash: [dry-run] would {what}");
            }
        };
        say(work_fold, &format!("fold the work @ into @- {}", work.chid));
        say(work_describe, "re-describe the work @-, its trailer kept");
        say(
            agent_fold,
            "fold the agent @, the journal, into the partner at @-",
        );
        say(agent_describe, "re-describe the agent @-, its trailer kept");
        return Ok(());
    }

    let snapshots = Snapshots::take(pair)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        if work_fold {
            info!("squash work: @ -> @- {}", work.chid);
            ctx.session(&pair.work)?.squash_into("@", "@-")?;
        }
        if let Some(text) = text
            && sides.has_work()
        {
            info!(
                "squash work: re-describe @- {}, its trailer kept",
                work.chid
            );
            ctx.session(&pair.work)?
                .describe("@-", &with_trailer(text, &work.trailer))?;
        }
        if agent_fold {
            info!("squash agent: @ -> @-, the journal into the partner");
            ctx.session(&pair.agent)?.squash_into("@", "@-")?;
        }
        if let Some(text) = text
            && agent == AgentStep::Fold
        {
            info!("squash agent: re-describe @-, its trailer kept");
            ctx.session(&pair.agent)?
                .describe("@-", &with_trailer(text, &format!("/{}", work.chid)))?;
        }
        Ok(())
    })();
    if let Err(e) = result {
        snapshots.restore(pair, "squash", e.as_ref());
        return Err(e);
    }
    info!(
        "squash: done, review it with `vc-x1 show -s both @- -R {}`",
        pair.work.display()
    );
    Ok(())
}

/// Decide the agent side from where the partner stands, refusing a
/// conflicted agent repo, and a published partner when there is
/// something to rewrite in it.
fn agent_step(
    pair: &Pair,
    work: &WorkCommit,
    text: Option<&str>,
) -> Result<AgentStep, Box<dyn std::error::Error>> {
    refuse_conflicts(&pair.agent, "squash", "agent")?;
    Ok(match partner(pair, work)? {
        Partner::Committed(_) => {
            if text.is_some() || !jj::is_empty(&pair.agent, "@")? {
                refuse_published(&pair.agent, "agent")?;
            }
            AgentStep::Fold
        }
        Partner::Pending(agent_at) => {
            info!(
                "squash agent: the agent @ {agent_at} is the pending partner, the journal is \
                 push's to commit, with the work commit's text"
            );
            AgentStep::Leave
        }
        Partner::Changed { stamped, agent_at } => {
            info!(
                "squash agent: the work commit names {stamped}, and the agent @ is {agent_at}: \
                 the agent repo changed since the commit, which push restamps"
            );
            AgentStep::Leave
        }
    })
}

/// Fold a plain repo's `@` into its `@-`, re-describing it when a
/// description was given.
fn squash_por(
    ctx: &mut Context,
    repo: &Path,
    text: Option<&str>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    refuse_conflicts(repo, "squash", "plain")?;
    let fold = !jj::is_empty(repo, "@")?;
    if !fold && text.is_none() {
        info!("squash: nothing to fold and no description to write");
        return Ok(());
    }
    refuse_published(repo, "plain")?;
    if dry_run {
        info!(
            "squash: [dry-run] would{} {} in {}",
            if fold { " fold @ into @-" } else { "" },
            if text.is_some() {
                "and re-describe @-"
            } else {
                ""
            },
            repo.display()
        );
        return Ok(());
    }
    if fold {
        info!("squash: @ -> @- in {}", repo.display());
        ctx.session(repo)?.squash_into("@", "@-")?;
    }
    if let Some(text) = text {
        info!("squash: re-describe @- in {}", repo.display());
        ctx.session(repo)?.describe("@-", text)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commit::{CommitParams, Target as CommitTarget, commit};
    use crate::options_flags::scope::Side;
    use crate::test_helpers::{Fixture, FixturePor, chid, description, jj_ok, test_ctx};
    use crate::{Cli, Commands};
    use clap::Parser;
    use std::fs;

    fn parse(args: &[&str]) -> Result<SquashArgs, String> {
        match Cli::try_parse_from(args) {
            Ok(Cli {
                command: Some(Commands::Squash(a)),
                ..
            }) => Ok(a),
            Ok(_) => panic!("expected Squash"),
            Err(e) => Err(e.to_string()),
        }
    }

    fn pair(fx: &Fixture) -> Pair {
        Pair {
            work: fx.work.clone(),
            agent: fx.bot.clone(),
        }
    }

    fn both() -> Scope {
        Scope(vec![Side::Work, Side::Bot])
    }

    fn is_empty(repo: &Path, rev: &str) -> bool {
        jj_ok(repo, &["log", "-r", rev, "--no-graph", "-T", "empty"]) == "true"
    }

    fn files(repo: &Path, rev: &str) -> String {
        jj_ok(repo, &["file", "list", "-r", rev])
    }

    /// Make a pair the way a rung does, `-s b` or `-s w`.
    fn make_pair(fx: &Fixture, agent_now: bool) {
        fs::write(fx.work.join("rung.txt"), "work\n").expect("write");
        fs::write(fx.bot.join("session.jsonl"), "{\"line\":1}\n").expect("write");
        let params = CommitParams {
            target: CommitTarget::Pair {
                pair: pair(fx),
                agent_now,
            },
            title: Some("feat: a rung".to_string()),
            body: Some("The problem.".to_string()),
            closing: None,
            opening: None,
            advance: crate::commit::Advance::Line,
            push: Some(false),
            dry_run: false,
        };
        commit(&mut test_ctx(), &params).expect("the pair is made");
    }

    fn run(fx: &Fixture, sides: Scope, text: Option<&str>) -> Result<(), String> {
        let params = SquashParams {
            target: Target::Pair {
                pair: pair(fx),
                sides,
            },
            text: text.map(str::to_string),
            dry_run: false,
        };
        squash(&mut test_ctx(), &params).map_err(|e| e.to_string())
    }

    /// `-s` is required, and `--title` and `--body` come together.
    #[test]
    fn scope_is_required_and_the_description_comes_whole() {
        let err = parse(&["vc-x1", "squash"]).unwrap_err();
        assert!(err.contains("--scope"), "{err}");
        let err = parse(&["vc-x1", "squash", "-s", "b", "--title", "t"]).unwrap_err();
        assert!(err.contains("--body"), "{err}");
        let err = parse(&["vc-x1", "squash", "-s", "b", "--body", "b"]).unwrap_err();
        assert!(err.contains("--title"), "{err}");
    }

    /// A pair made at once: a fix and a journal tail fold into their
    /// sides, each commit keeping its change id and trailer.
    #[test]
    fn both_folds_each_side_into_its_commit() {
        let fx = Fixture::new("squash-both");
        make_pair(&fx, true);
        let (work_id, agent_id) = (chid(&fx.work, "@-"), chid(&fx.bot, "@-"));
        let (work_desc, agent_desc) = (description(&fx.work, "@-"), description(&fx.bot, "@-"));
        fs::write(fx.work.join("fix.txt"), "a review fix\n").expect("write");
        fs::write(fx.bot.join("tail.jsonl"), "{\"line\":2}\n").expect("write");

        run(&fx, both(), None).expect("squash");

        assert_eq!(chid(&fx.work, "@-"), work_id);
        assert_eq!(chid(&fx.bot, "@-"), agent_id);
        assert!(files(&fx.work, "@-").contains("fix.txt"));
        assert!(files(&fx.bot, "@-").contains("tail.jsonl"));
        assert_eq!(description(&fx.work, "@-"), work_desc);
        assert_eq!(description(&fx.bot, "@-"), agent_desc);
        assert!(is_empty(&fx.work, "@") && is_empty(&fx.bot, "@"));
    }

    /// A pair made in two steps: the fix folds into the work commit
    /// and the pending partner, the journal, is left for push.
    #[test]
    fn a_pending_partner_is_left_for_push() {
        let fx = Fixture::new("squash-pending");
        make_pair(&fx, false);
        let agent_at = chid(&fx.bot, "@");
        fs::write(fx.work.join("fix.txt"), "a review fix\n").expect("write");

        run(&fx, both(), None).expect("squash");

        assert!(files(&fx.work, "@-").contains("fix.txt"));
        assert_eq!(
            chid(&fx.bot, "@"),
            agent_at,
            "the pending partner is untouched"
        );
        assert!(!is_empty(&fx.bot, "@"));
    }

    /// A description rewrite keeps both trailers, so the pair still
    /// validates in both directions.
    #[test]
    fn a_new_description_keeps_the_trailers() {
        let fx = Fixture::new("squash-describe");
        make_pair(&fx, true);
        let (work_id, agent_id) = (chid(&fx.work, "@-"), chid(&fx.bot, "@-"));

        run(
            &fx,
            both(),
            Some("feat: a better title\n\nA clearer problem."),
        )
        .expect("squash");

        let work_desc = description(&fx.work, "@-");
        let agent_desc = description(&fx.bot, "@-");
        assert!(
            work_desc.starts_with("feat: a better title\n\nA clearer problem."),
            "{work_desc}"
        );
        assert!(
            work_desc
                .trim_end()
                .ends_with(&format!("ochid: /.claude/{agent_id}")),
            "{work_desc}"
        );
        assert!(
            agent_desc.starts_with("feat: a better title"),
            "{agent_desc}"
        );
        assert!(
            agent_desc
                .trim_end()
                .ends_with(&format!("ochid: /{work_id}")),
            "{agent_desc}"
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
        crate::validate_desc::validate_desc(&test_ctx(), &check).expect("both directions valid");
    }

    /// A published pair is refused before anything moves, and a work
    /// commit with no trailer is not a pair's.
    #[test]
    fn a_published_pair_and_an_unpaired_commit_are_refused() {
        let fx = Fixture::new("squash-published");
        make_pair(&fx, true);
        jj_ok(&fx.work, &["bookmark", "set", "main", "-r", "@-"]);
        jj_ok(&fx.work, &["git", "push", "--bookmark", "main"]);
        fs::write(fx.work.join("fix.txt"), "too late\n").expect("write");
        let err = run(&fx, both(), None).unwrap_err();
        assert!(err.contains("published"), "{err}");
        assert!(!is_empty(&fx.work, "@"), "nothing folded");

        let fx = Fixture::new("squash-unpaired");
        fs::write(fx.work.join("hand.txt"), "by hand\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: by hand"]);
        let err = run(&fx, both(), None).unwrap_err();
        assert!(err.contains("no `ochid:` trailer"), "{err}");
    }

    /// A dry run decides and prints, and folds nothing.
    #[test]
    fn a_dry_run_folds_nothing() {
        let fx = Fixture::new("squash-dry-run");
        make_pair(&fx, true);
        fs::write(fx.work.join("fix.txt"), "a review fix\n").expect("write");
        let params = SquashParams {
            target: Target::Pair {
                pair: pair(&fx),
                sides: both(),
            },
            text: Some("feat: x\n\nY.".to_string()),
            dry_run: true,
        };
        squash(&mut test_ctx(), &params).expect("dry run");
        assert!(!is_empty(&fx.work, "@"), "the fix is still in @");
        assert!(description(&fx.work, "@-").starts_with("feat: a rung"));
    }

    /// `-s p` folds a plain repo's `@` and re-describes its `@-`.
    #[test]
    fn por_folds_and_re_describes() {
        let fx = FixturePor::new("squash-por");
        fs::write(fx.work.join("a.txt"), "a\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "feat: plain"]);
        let id = chid(&fx.work, "@-");
        fs::write(fx.work.join("b.txt"), "b\n").expect("write");
        let params = SquashParams {
            target: Target::Por(fx.work.clone()),
            text: Some("feat: plainer\n\nA body.".to_string()),
            dry_run: false,
        };
        squash(&mut test_ctx(), &params).expect("squash -s p");
        assert_eq!(chid(&fx.work, "@-"), id);
        assert!(files(&fx.work, "@-").contains("b.txt"));
        assert!(description(&fx.work, "@-").starts_with("feat: plainer"));
        assert!(is_empty(&fx.work, "@"));
    }
}

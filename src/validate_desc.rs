//! The `validate-desc` subcommand: scan a revision range and check
//! each commit's `ochid:` trailer against the cross-referenced
//! repo: flagging missing, malformed, or dangling links.
//!
//! Read-only diagnostic; exits non-zero if any commit has issues.
//! The bot's "did the ochid wiring stay consistent?" check.
//!
//! The first command under the scope rule: `-s` is required and
//! says whose commits are checked, `w`, `a`, or `b` for both
//! directions, one table per side under its label, and `-R` is
//! optional and says where. `-s p` is refused, since a plain repo
//! has no other repo to check against.

use std::path::{Path, PathBuf};

use clap::Args;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::Repo;

use log::{debug, info};

use crate::common;
use crate::context::Context;
use crate::desc_helpers::{
    DEFAULT_ID_LEN, TitleMatch, extract_bare_id, find_matching_commit, ochid_prefix_for,
    validate_ochid,
};
use crate::options_flags::scope::{REPO_SCOPE_FORMS, RepoScope, repo_scope_parser, side_keyword};
use crate::subcommand::SubcommandRunner;

#[derive(Args, Debug)]
pub struct ValidateDescArgs {
    /// Retired: the revision, now `-r`, refused naming it.
    #[arg(value_name = "RETIRED", hide = true)]
    pub retired_rev: Option<String>,

    /// Retired: the count, now `-n`, refused naming it, kept as typed.
    #[arg(value_name = "RETIRED", hide = true)]
    pub retired_count: Option<String>,

    /// Revision(s) to scan, with optional .. notation
    #[arg(short, long, default_value = "@")]
    pub revision: String,

    /// Number of commits to scan
    #[arg(short = 'n', long = "commits", value_name = "COMMITS")]
    pub limit: Option<usize>,

    /// Where: a path in the workspace, its root or either repo
    /// [default: the current directory]
    #[arg(short = 'R', long, value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Whose commits are checked, required: `w|work`, `a|agent`, or
    /// `b|both`, each side against the other. `p|por` is refused,
    /// a plain repo having no other repo
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = repo_scope_parser(&["work", "agent", "both"]),
        required = true,
        help = format!("Whose commits are checked, required: {REPO_SCOPE_FORMS}, `p` refused")
    )]
    pub scope: RepoScope,

    /// Path to the other repo, for one side [default: the workspace's
    /// other side]
    #[arg(long = "other-repo", value_name = "PATH")]
    pub other_repo: Option<PathBuf>,

    /// Expected changeID length
    #[arg(long = "id-len", default_value_t = DEFAULT_ID_LEN)]
    pub id_len: usize,
}

/// Inputs to the validate-desc op, flat, owned, clap-free.
///
/// Mirrors `ValidateDescArgs`: `--revision` / `--commits`
/// (`revision` / `limit`), `--other-repo`, `--id-len`, and `-s`
/// with `-R` resolved into `repos`, each side's label and path in
/// scope order. The retired positionals are refused before it is
/// built.
#[derive(Debug)]
pub struct ValidateDescParams {
    pub revision: String,
    pub limit: Option<usize>,
    pub repos: Vec<(String, PathBuf)>,
    pub other_repo: Option<PathBuf>,
    pub id_len: usize,
}

impl TryFrom<&ValidateDescArgs> for ValidateDescParams {
    type Error = String;

    /// Resolve `-s` and `-R` into the sides to check, labeled by
    /// their keywords, and copy the rest over. `-s p` is refused
    /// here, before anything is read.
    fn try_from(a: &ValidateDescArgs) -> Result<Self, String> {
        crate::options_flags::retired_positional::refuse_rev_count(
            "validate-desc",
            a.retired_rev.as_deref(),
            a.retired_count.as_deref(),
        )?;
        let sides = match &a.scope {
            RepoScope::Sides(s) => s,
            RepoScope::Por => {
                return Err(
                    "-s p: a plain repo has no other repo to check its descriptions \
                            against. validate-desc takes `-s w|a|b` in a dual workspace"
                        .to_string(),
                );
            }
        };
        let paths = crate::common::resolve_scoped_repos(&a.scope, a.repo.as_deref())
            .map_err(|e| e.to_string())?;
        let repos = sides
            .0
            .iter()
            .zip(paths)
            .map(|(side, path)| (side_keyword(*side).to_string(), path))
            .collect();
        Ok(Self {
            revision: a.revision.clone(),
            limit: a.limit,
            repos,
            other_repo: a.other_repo.clone(),
            id_len: a.id_len,
        })
    }
}

/// Status of a single commit's ochid validation.
enum CommitStatus {
    Ok,
    Lost,                     // ochid marked as "lost" (unrecoverable)
    None_,                    // ochid marked as "none" (no counterpart)
    NeedsFixed(String),       // summary of issues
    MissingNoTitle,           // no ochid, no title to match on
    MissingNoMatch,           // no ochid, no title match in other repo
    MissingAmbiguous(usize),  // no ochid, multiple title matches
    MissingWithMatch(String), // no ochid, unique title match found
}

impl SubcommandRunner for ValidateDescArgs {
    type Params = ValidateDescParams;

    /// Delegate to the `TryFrom<&ValidateDescArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        ValidateDescParams::try_from(self)
    }

    /// Run the existing `validate_desc` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        validate_desc(ctx, params)
    }
}

/// Run the `validate-desc` subcommand: for each scoped side, scan
/// the resolved revision range and report each commit's ochid status
/// against the other repo, under the side's label when there is
/// more than one. Errors if any commit on any side has issues.
///
/// `ctx` is unused today (validate-desc reads `.vc-config.toml`,
/// not the user config, and doesn't touch the `--log` path); it's
/// present for the uniform subcommand-layer signature.
pub fn validate_desc(
    _ctx: &Context,
    params: &ValidateDescParams,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("validate-desc: enter");
    let labeled = params.repos.len() > 1;
    let mut issues_total = 0;
    for (label, repo) in &params.repos {
        if labeled {
            info!("{label} ({}):", repo.display());
        }
        issues_total += validate_side(params, repo)?;
        if labeled {
            info!("");
        }
    }
    if issues_total > 0 {
        debug!("validate-desc: exit with issues");
        Err(format!("{issues_total} commit(s) have issues").into())
    } else {
        debug!("validate-desc: exit");
        Ok(())
    }
}

/// Scan one side's revision range against its other repo and print
/// the table, returning how many commits have issues. A side with
/// no other repo, a single-repo workspace, prints a notice and
/// counts nothing.
fn validate_side(
    params: &ValidateDescParams,
    repo_path: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let (workspace, repo) = common::load_repo(repo_path)?;

    // Resolve other repo: --other-repo flag, or side-aware
    // resolution from the workspace config (the work repo pairs
    // with the bot repo and vice versa). A single-repo / POR
    // workspace has no bot side: nothing to validate against, so
    // the command no-ops instead of erroring (por equalization;
    // topology from the workspace config, not a flag).
    let other_repo_path = match &params.other_repo {
        Some(p) => p.clone(),
        None => match common::other_repo_path(repo_path)? {
            Some(p) => p,
            None => {
                info!("validate-desc: single-repo workspace (no bot side), nothing to validate");
                return Ok(0);
            }
        },
    };

    let (other_workspace, other_repo) = common::load_repo(&other_repo_path)?;
    let other_prefix = ochid_prefix_for(&other_repo_path)?;

    let spec = common::resolve_spec(&params.revision, params.limit);

    let (ids, _anchor_index) = common::collect_ids(
        &workspace,
        &repo,
        &spec.rev,
        spec.desc_count,
        spec.anc_count,
    )?;
    if ids.is_empty() {
        return Err(format!("no commits found for revision '{}'", params.revision).into());
    }

    let root_id = repo.store().root_commit_id().clone();
    let mut valid = 0;
    let mut lost = 0;
    let mut none = 0;
    let mut issues_count = 0;
    let mut missing = 0;

    let col_header = "STAT CHANGEID      TITLE";
    info!("{col_header}");

    for commit_id in &ids {
        if *commit_id == root_id {
            continue;
        }
        let commit = repo.store().get_commit(commit_id)?;
        let desc = commit.description();
        let change_hex = jj_lib::hex_util::encode_reverse_hex(commit.change_id().as_bytes());
        let change_short = &change_hex[..change_hex.len().min(12)];
        let first_line = desc.lines().next().unwrap_or(""); // OK: obvious
        let display_title = if first_line.is_empty() {
            "(no description set)"
        } else {
            first_line
        };

        let current_ochid = common::extract_ochid(&commit);

        let status = if let Some(ref ochid_val) = current_ochid {
            let bare = extract_bare_id(ochid_val);
            if bare == "lost" {
                CommitStatus::Lost
            } else if bare == "none" {
                CommitStatus::None_
            } else {
                let issues = validate_ochid(
                    ochid_val,
                    &other_prefix,
                    params.id_len,
                    &other_workspace,
                    &other_repo,
                );
                if issues.any() {
                    CommitStatus::NeedsFixed(issues.summary())
                } else {
                    CommitStatus::Ok
                }
            }
        } else {
            match find_matching_commit(&commit, &other_workspace, &other_repo)? {
                TitleMatch::NoTitle => CommitStatus::MissingNoTitle,
                TitleMatch::One(id) => {
                    let short_id = &id[..id.len().min(params.id_len)];
                    CommitStatus::MissingWithMatch(format!("{other_prefix}{short_id}"))
                }
                TitleMatch::Ambiguous(n) => CommitStatus::MissingAmbiguous(n),
                TitleMatch::None => CommitStatus::MissingNoMatch,
            }
        };

        match status {
            CommitStatus::Ok => {
                valid += 1;
                info!("ok   {change_short}  {display_title}");
            }
            CommitStatus::Lost => {
                lost += 1;
                info!("lost {change_short}  {display_title}");
            }
            CommitStatus::None_ => {
                none += 1;
                info!("none {change_short}  {display_title}");
            }
            CommitStatus::NeedsFixed(summary) => {
                issues_count += 1;
                info!("err  {change_short}  {display_title}  [{summary}]");
            }
            CommitStatus::MissingWithMatch(ochid) => {
                missing += 1;
                info!("miss {change_short}  {display_title}  [match: {ochid}]");
            }
            CommitStatus::MissingNoTitle => {
                missing += 1;
                info!("miss {change_short}  {display_title}");
            }
            CommitStatus::MissingAmbiguous(n) => {
                missing += 1;
                info!("miss {change_short}  {display_title}  [{n} title matches, ambiguous]");
            }
            CommitStatus::MissingNoMatch => {
                missing += 1;
                info!("miss {change_short}  {display_title}  [no matching title in other repo]");
            }
        }
    }

    let total = valid + lost + none + issues_count + missing;
    if total > 10 {
        info!("{col_header}");
    }
    info!("");
    info!(
        "{valid} valid, {lost} lost, {none} none, {issues_count} issues, {missing} missing \
         (of {} total)",
        ids.len()
    );
    Ok(issues_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options_flags::scope::{Scope, Side};
    use crate::test_helpers::{Fixture, FixturePor, chid, jj_ok};
    use crate::{Cli, Commands};
    use clap::Parser;

    fn parse(args: &[&str]) -> Result<ValidateDescArgs, String> {
        match Cli::try_parse_from(args) {
            Ok(Cli {
                command: Some(Commands::ValidateDesc(a)),
                ..
            }) => Ok(a),
            Ok(_) => panic!("expected ValidateDesc"),
            Err(e) => Err(e.to_string()),
        }
    }

    /// `-s` is required, and `-s p` is refused at params time with
    /// the reason: a plain repo has nothing to check against.
    #[test]
    fn scope_is_required_and_por_is_refused() {
        let err = parse(&["vc-x1", "validate-desc"]).unwrap_err();
        assert!(err.contains("--scope"), "{err}");
        let fx = FixturePor::new("vdesc-por-refused");
        let mut a = parse(&["vc-x1", "validate-desc", "-s", "p"]).expect("parses");
        a.repo = Some(fx.work.clone());
        let err = ValidateDescParams::try_from(&a).unwrap_err();
        assert!(err.starts_with("-s p:"), "{err}");
        assert!(err.contains("`-s w|a|b`"), "{err}");
    }

    /// `-s b` from either repo is both sides in work-then-agent
    /// order, labeled by their keywords, and `-s a` the agent side
    /// alone. A fresh workspace's commits pass in both directions.
    #[test]
    fn both_sides_are_labeled_and_a_fresh_workspace_passes() {
        let fx = Fixture::new("vdesc-both");
        let ctx = Context::load().expect("load user config");
        let mut a = parse(&["vc-x1", "validate-desc", "-r", "@-", "-s", "b"]).expect("parses");
        a.repo = Some(fx.bot.clone());
        let params = ValidateDescParams::try_from(&a).expect("params");
        let labels: Vec<&str> = params.repos.iter().map(|(l, _)| l.as_str()).collect();
        assert_eq!(labels, ["work", "agent"]);
        assert_eq!(params.repos[1].1, fx.bot.canonicalize().expect("canonical"));
        validate_desc(&ctx, &params).expect("a fresh workspace's trailers are valid");

        a.scope = RepoScope::Sides(Scope(vec![Side::Bot]));
        let params = ValidateDescParams::try_from(&a).expect("params");
        assert_eq!(params.repos.len(), 1);
        assert_eq!(params.repos[0].0, "agent");
    }

    /// A URL-form trailer passes when its ID resolves in the other
    /// repo and fails when it does not, the URL itself not checked.
    #[test]
    fn a_url_form_trailer_is_valid_when_its_id_resolves() {
        let fx = Fixture::new("vdesc-url");
        let ctx = Context::load().expect("load user config");
        let mut a = parse(&["vc-x1", "validate-desc", "-r", "@", "-s", "w"]).expect("parses");
        a.repo = Some(fx.work.clone());
        let params = ValidateDescParams::try_from(&a).expect("params");
        let url = "https://example.com/o/r.agent-session";

        let agent_at = chid(&fx.bot, "@");
        let desc = format!("feat: x\n\nochid: {url}#{agent_at}\n");
        jj_ok(&fx.work, &["describe", "-m", &desc]);
        validate_desc(&ctx, &params).expect("the ID resolves");

        let desc = format!("feat: x\n\nochid: {url}#zzzzzzzzzzzk\n");
        jj_ok(&fx.work, &["describe", "-m", &desc]);
        validate_desc(&ctx, &params).expect_err("the ID resolves to nothing");
    }
}

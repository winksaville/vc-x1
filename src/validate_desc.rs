//! The `validate-desc` subcommand: scan a revision range and check
//! each commit's `ochid:` trailer against the cross-referenced
//! repo — flagging missing, malformed, or dangling links.
//!
//! Read-only diagnostic; exits non-zero if any commit has issues.
//! The bot's "did the ochid wiring stay consistent?" check.

use std::path::PathBuf;

use clap::Args;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::Repo;

use log::{debug, info};

use crate::common;
use crate::context::Context;
use crate::desc_helpers::{
    DEFAULT_ID_LEN, TitleMatch, VC_CONFIG_FILE, extract_bare_id, find_matching_commit,
    ochid_prefix_from_config, other_repo_from_config, validate_ochid,
};
use crate::toml_simple;

#[derive(Args, Debug)]
pub struct ValidateDescArgs {
    /// Revision (with optional .. notation)
    #[arg(value_name = "REVISION")]
    pub pos_rev: Option<String>,

    /// Number of commits (per dotted side)
    #[arg(value_name = "COMMITS")]
    pub pos_count: Option<usize>,

    /// Revision(s) to scan
    #[arg(short, long, default_value = "@")]
    pub revision: String,

    /// Number of commits to scan
    #[arg(short = 'n', long = "commits", value_name = "COMMITS")]
    pub limit: Option<usize>,

    /// Path to jj repo
    #[arg(short = 'R', long, default_value = ".")]
    pub repo: PathBuf,

    /// Path to the other repo [default: from .vc-config.toml]
    #[arg(long = "other-repo")]
    pub other_repo: Option<PathBuf>,

    /// Expected changeID length
    #[arg(long = "id-len", default_value_t = DEFAULT_ID_LEN)]
    pub id_len: usize,
}

/// Inputs to the validate-desc op, flat, owned, clap-free.
///
/// Mirrors `ValidateDescArgs`: positional `REVISION` / `COMMITS`
/// (`pos_rev` / `pos_count`), `--revision` / `--commits`
/// (`revision` / `limit`), `--repo`, `--other-repo`, `--id-len`.
pub struct ValidateDescParams {
    pub pos_rev: Option<String>,
    pub pos_count: Option<usize>,
    pub revision: String,
    pub limit: Option<usize>,
    pub repo: PathBuf,
    pub other_repo: Option<PathBuf>,
    pub id_len: usize,
}

impl From<&ValidateDescArgs> for ValidateDescParams {
    /// Convert clap-derived `ValidateDescArgs` into the flat
    /// `ValidateDescParams` (total — every field copies straight over).
    fn from(a: &ValidateDescArgs) -> Self {
        Self {
            pos_rev: a.pos_rev.clone(),
            pos_count: a.pos_count,
            revision: a.revision.clone(),
            limit: a.limit,
            repo: a.repo.clone(),
            other_repo: a.other_repo.clone(),
            id_len: a.id_len,
        }
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

/// Run the `validate-desc` subcommand: scan the resolved revision
/// range and report each commit's ochid status against the other
/// repo; errors if any commit has issues.
///
/// `ctx` is unused today (validate-desc reads `.vc-config.toml`,
/// not the user config, and doesn't touch the `--log` path); it's
/// present for the uniform subcommand-layer signature.
pub fn validate_desc(
    _ctx: &Context,
    params: &ValidateDescParams,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("validate-desc: enter");
    let (workspace, repo) = common::load_repo(&params.repo)?;

    // Resolve other repo: --other-repo flag, or fall back to .vc-config.toml
    let other_repo_path = if let Some(ref p) = params.other_repo {
        p.clone()
    } else {
        let config = toml_simple::toml_load(&params.repo.join(VC_CONFIG_FILE))?;
        params.repo.join(other_repo_from_config(&config)?)
    };

    let (other_workspace, other_repo) = common::load_repo(&other_repo_path)?;
    let other_config = toml_simple::toml_load(&other_repo_path.join(VC_CONFIG_FILE))?;
    let other_prefix = ochid_prefix_from_config(&other_config)?;

    let spec = common::resolve_spec(
        params.pos_rev.as_deref(),
        params.pos_count,
        &params.revision,
        params.limit,
        "@",
    );

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
        "{valid} valid, {lost} lost, {none} none, {issues_count} issues, {missing} missing (of {} total)",
        ids.len()
    );
    if issues_count > 0 {
        debug!("validate-desc: exit with issues");
        Err(format!("{issues_count} commit(s) have issues").into())
    } else {
        debug!("validate-desc: exit");
        Ok(())
    }
}

use std::path::PathBuf;

use clap::Args;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::Repo;

use crate::common;
use crate::desc_helpers::{
    DEFAULT_ID_LEN, TitleMatch, VC_CONFIG_FILE, extract_bare_id, find_matching_commit,
    ochid_prefix_from_config, validate_ochid,
};
use crate::toml_simple;

#[derive(Args, Debug)]
pub struct ValidateDescArgs {
    /// Path to the other repo (e.g. .claude or .)
    pub other_repo: PathBuf,

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

    /// Expected changeID length
    #[arg(long = "id-len", default_value_t = DEFAULT_ID_LEN)]
    pub id_len: usize,
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

pub fn validate_desc(args: &ValidateDescArgs) -> Result<(), Box<dyn std::error::Error>> {
    let (workspace, repo) = common::load_repo(&args.repo)?;
    let (other_workspace, other_repo) = common::load_repo(&args.other_repo)?;
    let other_config = toml_simple::toml_load(&args.other_repo.join(VC_CONFIG_FILE))?;
    let other_prefix = ochid_prefix_from_config(&other_config)?;

    let spec = common::resolve_spec(
        args.pos_rev.as_deref(),
        args.pos_count,
        &args.revision,
        args.limit,
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
        return Err(format!("no commits found for revision '{}'", args.revision).into());
    }

    let root_id = repo.store().root_commit_id().clone();
    let mut valid = 0;
    let mut lost = 0;
    let mut none = 0;
    let mut issues_count = 0;
    let mut missing = 0;

    let col_header = "STAT CHANGEID      TITLE";
    println!("{col_header}");

    for commit_id in &ids {
        if *commit_id == root_id {
            continue;
        }
        let commit = repo.store().get_commit(commit_id)?;
        let desc = commit.description();
        let change_hex = jj_lib::hex_util::encode_reverse_hex(commit.change_id().as_bytes());
        let change_short = &change_hex[..change_hex.len().min(12)];
        let first_line = desc.lines().next().unwrap_or("");
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
                    args.id_len,
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
                    let short_id = &id[..id.len().min(args.id_len)];
                    CommitStatus::MissingWithMatch(format!("{other_prefix}{short_id}"))
                }
                TitleMatch::Ambiguous(n) => CommitStatus::MissingAmbiguous(n),
                TitleMatch::None => CommitStatus::MissingNoMatch,
            }
        };

        match status {
            CommitStatus::Ok => {
                valid += 1;
                println!("ok   {change_short}  {display_title}");
            }
            CommitStatus::Lost => {
                lost += 1;
                println!("lost {change_short}  {display_title}");
            }
            CommitStatus::None_ => {
                none += 1;
                println!("none {change_short}  {display_title}");
            }
            CommitStatus::NeedsFixed(summary) => {
                issues_count += 1;
                println!("err  {change_short}  {display_title}  [{summary}]");
            }
            CommitStatus::MissingWithMatch(ochid) => {
                missing += 1;
                println!("miss {change_short}  {display_title}  [match: {ochid}]");
            }
            CommitStatus::MissingNoTitle => {
                missing += 1;
                println!("miss {change_short}  {display_title}");
            }
            CommitStatus::MissingAmbiguous(n) => {
                missing += 1;
                println!("miss {change_short}  {display_title}  [{n} title matches, ambiguous]");
            }
            CommitStatus::MissingNoMatch => {
                missing += 1;
                println!("miss {change_short}  {display_title}  [no matching title in other repo]");
            }
        }
    }

    let total = valid + lost + none + issues_count + missing;
    if total > 10 {
        println!("{col_header}");
    }
    println!(
        "\n{valid} valid, {lost} lost, {none} none, {issues_count} issues, {missing} missing (of {} total)",
        ids.len()
    );
    if issues_count > 0 {
        Err(format!("{issues_count} commit(s) have issues").into())
    } else {
        Ok(())
    }
}

use std::path::PathBuf;

use clap::Args;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::Repo;

use crate::common;

#[derive(Args, Debug)]
pub struct DescArgs {
    /// Revision (with optional .. notation)
    #[arg()]
    pub pos_rev: Option<String>,

    /// Count (items per dotted side)
    #[arg()]
    pub pos_count: Option<usize>,

    /// Revision to describe (default: @)
    #[arg(short, long, default_value = "@")]
    pub revision: String,

    /// Path to jj repo (default: current directory)
    #[arg(short = 'R', long, default_value = ".")]
    pub repo: PathBuf,

    /// Maximum number of commits to describe
    #[arg(short, long)]
    pub limit: Option<usize>,
}

fn format_commit(commit: &jj_lib::commit::Commit) -> String {
    let change_hex = encode_reverse_hex(commit.change_id().as_bytes());
    let change_short = &change_hex[..change_hex.len().min(12)];
    let commit_hex = commit.id().hex();
    let commit_short = &commit_hex[..commit_hex.len().min(12)];

    let desc = commit.description();
    if desc.is_empty() {
        format!("{} {} (no description set)", change_short, commit_short)
    } else {
        let mut lines = desc.lines();
        let first_line = lines.next().unwrap_or("");
        let mut result = format!("{} {} {}", change_short, commit_short, first_line);
        for line in lines {
            result.push('\n');
            result.push_str(line);
        }
        result
    }
}

pub fn desc(args: &DescArgs) -> Result<(), Box<dyn std::error::Error>> {
    let resolved = common::resolve_dot_args(
        args.pos_rev.as_deref(),
        args.pos_count,
        &args.revision,
        args.limit,
        "@",
    );

    let (workspace, repo) = common::load_repo(&args.repo)?;

    let commit_ids = common::resolve_revset(&workspace, &repo, &resolved.revset)?;

    if commit_ids.is_empty() {
        return Err(format!("no commit found for revision '{}'", resolved.revset).into());
    }

    if let Some(both_count) = resolved.both_count {
        return print_both_desc(&repo, args, both_count);
    }

    let root_commit_id = repo.store().root_commit_id().clone();
    let mut results: Vec<String> = Vec::new();
    let mut count = 0;
    for commit_id in &commit_ids {
        if *commit_id == root_commit_id {
            continue;
        }
        let commit = repo.store().get_commit(commit_id)?;
        results.push(format_commit(&commit));

        count += 1;
        if let Some(limit) = resolved.limit
            && count >= limit
        {
            break;
        }
    }

    for line in &results {
        println!("{line}");
    }

    Ok(())
}

fn print_both_desc(
    repo: &std::sync::Arc<jj_lib::repo::ReadonlyRepo>,
    args: &DescArgs,
    both_count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let bare_rev = args.pos_rev.as_deref().unwrap_or("@");
    let spec = common::parse_dot_rev(bare_rev);
    let (workspace, repo2) = common::load_repo(&args.repo)?;
    let anchor_ids = common::resolve_revset(&workspace, &repo2, &spec.rev)?;
    if anchor_ids.is_empty() {
        return Err(format!("no commit found for revision '{}'", spec.rev).into());
    }
    let anchor_id = &anchor_ids[0];
    let root_commit_id = repo.store().root_commit_id().clone();

    let ancestor_ids = common::resolve_revset(&workspace, repo, &format!("::{}", spec.rev))?;
    let descendant_ids = common::resolve_revset(&workspace, repo, &format!("{}::", spec.rev))?;

    // Descendants (newest first from jj), excluding anchor
    let mut desc_lines: Vec<String> = Vec::new();
    for commit_id in &descendant_ids {
        if *commit_id == root_commit_id || *commit_id == *anchor_id {
            continue;
        }
        let commit = repo.store().get_commit(commit_id)?;
        desc_lines.push(format_commit(&commit));
    }
    if both_count > 0 {
        let start = desc_lines.len().saturating_sub(both_count);
        desc_lines = desc_lines[start..].to_vec();
    } else {
        desc_lines.clear();
    }

    // Anchor
    let anchor_commit = repo.store().get_commit(anchor_id)?;

    // Ancestors (newest first from jj), excluding anchor
    let mut anc_lines: Vec<String> = Vec::new();
    if both_count > 0 {
        let mut count = 0;
        for commit_id in &ancestor_ids {
            if *commit_id == root_commit_id || *commit_id == *anchor_id {
                continue;
            }
            let commit = repo.store().get_commit(commit_id)?;
            anc_lines.push(format_commit(&commit));
            count += 1;
            if count >= both_count {
                break;
            }
        }
    }

    for line in &desc_lines {
        println!("{line}");
    }
    println!("{}", format_commit(&anchor_commit));
    for line in &anc_lines {
        println!("{line}");
    }

    Ok(())
}

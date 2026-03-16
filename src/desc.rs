use std::path::PathBuf;

use clap::Args;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::Repo;

use crate::common;

#[derive(Args, Debug)]
pub struct DescArgs {
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

pub fn desc(args: &DescArgs) -> Result<(), Box<dyn std::error::Error>> {
    let (workspace, repo) = common::load_repo(&args.repo)?;

    let commit_ids = common::resolve_revset(&workspace, &repo, &args.revision)?;

    if commit_ids.is_empty() {
        return Err(format!("no commit found for revision '{}'", args.revision).into());
    }

    let mut count = 0;
    for commit_id in &commit_ids {
        let commit = repo.store().get_commit(commit_id)?;

        let change_hex = encode_reverse_hex(commit.change_id().as_bytes());
        let change_short = &change_hex[..change_hex.len().min(12)];
        let commit_hex = commit.id().hex();
        let commit_short = &commit_hex[..commit_hex.len().min(12)];

        let desc = commit.description();
        if desc.is_empty() {
            println!("{} {} (no description set)", change_short, commit_short);
        } else {
            let mut lines = desc.lines();
            let first_line = lines.next().unwrap_or("");
            println!("{} {} {}", change_short, commit_short, first_line);
            for line in lines {
                println!("{line}");
            }
        }

        count += 1;
        if let Some(limit) = args.limit
            && count >= limit
        {
            break;
        }
    }

    Ok(())
}

use std::path::PathBuf;

use clap::Args;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::Repo;

use crate::common;

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Revision to list (default: @)
    #[arg(short, long, default_value = "@")]
    pub revision: String,

    /// Path to jj repo (default: current directory)
    #[arg(short = 'R', long, default_value = ".")]
    pub repo: PathBuf,

    /// Maximum number of commits to show
    #[arg(short, long)]
    pub limit: Option<usize>,
}

pub fn list(args: &ListArgs) -> Result<(), Box<dyn std::error::Error>> {
    let (workspace, repo) = common::load_repo(&args.repo)?;

    let commit_ids = common::resolve_revset(&workspace, &repo, &args.revision)?;

    let root_commit_id = repo.store().root_commit_id().clone();

    let mut count = 0;
    for commit_id in &commit_ids {
        if *commit_id == root_commit_id {
            continue;
        }
        let commit = repo.store().get_commit(commit_id)?;

        let change_hex = encode_reverse_hex(commit.change_id().as_bytes());
        let change_short = &change_hex[..change_hex.len().min(12)];

        let commit_hex = commit.id().hex();
        let commit_short = &commit_hex[..commit_hex.len().min(12)];

        let first_line = commit.description().lines().next().unwrap_or("");

        println!("{} {} {}", change_short, commit_short, first_line);

        count += 1;
        if let Some(limit) = args.limit
            && count >= limit
        {
            break;
        }
    }

    Ok(())
}

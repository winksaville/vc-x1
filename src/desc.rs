use std::path::PathBuf;

use clap::Args;
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
        let desc = commit.description();
        if desc.is_empty() {
            println!("(no description set)");
        } else {
            print!("{desc}");
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

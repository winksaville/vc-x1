use std::path::PathBuf;

use clap::Args;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::Repo;

use crate::common;

#[derive(Args, Debug)]
pub struct ChidArgs {
    /// Revision to look up (default: @)
    #[arg(short, long, default_value = "@")]
    pub revision: String,

    /// Path to jj repo (default: current directory)
    #[arg(short = 'R', long, default_value = ".")]
    pub repo: PathBuf,

    /// Maximum number of changeIDs to show
    #[arg(short, long)]
    pub limit: Option<usize>,
}

pub fn chid(args: &ChidArgs) -> Result<(), Box<dyn std::error::Error>> {
    let (workspace, repo) = common::load_repo(&args.repo)?;

    let limit = args.limit.unwrap_or(1);

    let revset_str = if limit > 1 {
        format!("ancestors({})", args.revision)
    } else {
        args.revision.clone()
    };

    let commit_ids = common::resolve_revset(&workspace, &repo, &revset_str)?;

    if commit_ids.is_empty() {
        return Err(format!("no commit found for revision '{}'", args.revision).into());
    }

    let root_commit_id = repo.store().root_commit_id().clone();

    let mut count = 0;
    for commit_id in &commit_ids {
        if *commit_id == root_commit_id {
            continue;
        }
        let commit = repo.store().get_commit(commit_id)?;
        let change_hex = encode_reverse_hex(commit.change_id().as_bytes());
        let change_short = &change_hex[..change_hex.len().min(12)];

        println!("{change_short}");

        count += 1;
        if count >= limit {
            break;
        }
    }

    Ok(())
}

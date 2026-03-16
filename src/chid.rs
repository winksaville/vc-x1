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
}

pub fn chid(args: &ChidArgs) -> Result<(), Box<dyn std::error::Error>> {
    let (workspace, repo) = common::load_repo(&args.repo)?;

    let commit_ids = common::resolve_revset(&workspace, &repo, &args.revision)?;

    if commit_ids.is_empty() {
        return Err(format!("no commit found for revision '{}'", args.revision).into());
    }

    let commit_id = &commit_ids[0];
    let commit = repo.store().get_commit(commit_id)?;
    let change_hex = encode_reverse_hex(commit.change_id().as_bytes());
    let change_short = &change_hex[..change_hex.len().min(12)];

    println!("{change_short}");

    Ok(())
}

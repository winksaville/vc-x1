use std::path::PathBuf;

use clap::Args;
use jj_lib::repo::Repo;

use crate::common;

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Revision (with optional .. notation)
    #[arg(value_name = "REVISION")]
    pub pos_rev: Option<String>,

    /// Number of commits to show (per dotted side)
    #[arg(value_name = "COMMITS")]
    pub pos_count: Option<usize>,

    /// Revision to list (default: @)
    #[arg(short, long, default_value = "@")]
    pub revision: String,

    /// Path to jj repo (default: current directory)
    #[arg(short = 'R', long, default_value = ".")]
    pub repo: PathBuf,

    /// Number of commits to show
    #[arg(
        short = 'n',
        long = "commits",
        alias = "limit",
        short_alias = 'l',
        value_name = "COMMITS"
    )]
    pub limit: Option<usize>,
}

pub fn list(args: &ListArgs) -> Result<(), Box<dyn std::error::Error>> {
    let spec = common::resolve_spec(
        args.pos_rev.as_deref(),
        args.pos_count,
        &args.revision,
        args.limit,
        "@",
    );

    let (workspace, repo) = common::load_repo(&args.repo)?;
    let (ids, anchor_index) = common::collect_ids(
        &workspace,
        &repo,
        &spec.rev,
        spec.desc_count,
        spec.anc_count,
    )?;

    for (i, commit_id) in ids.iter().enumerate() {
        let commit = repo.store().get_commit(commit_id)?;
        let line = common::format_commit_short(&commit);
        if i == anchor_index {
            println!("{}", common::bold(&line));
        } else {
            println!("{line}");
        }
    }

    Ok(())
}

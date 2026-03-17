use std::path::PathBuf;

use clap::Args;
use jj_lib::repo::Repo;

use crate::common;

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Revision (with optional .. notation)
    #[arg()]
    pub pos_rev: Option<String>,

    /// Count (items per dotted side)
    #[arg()]
    pub pos_count: Option<usize>,

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

    let root_commit_id = repo.store().root_commit_id().clone();

    if let Some(both_count) = resolved.both_count {
        let lines = common::collect_both(
            &repo,
            &args.repo,
            args.pos_rev.as_deref(),
            both_count,
            common::format_commit_short,
        )?;
        for line in &lines {
            println!("{line}");
        }
        return Ok(());
    }

    let mut results: Vec<String> = Vec::new();
    let mut count = 0;
    for commit_id in &commit_ids {
        if *commit_id == root_commit_id {
            continue;
        }
        let commit = repo.store().get_commit(commit_id)?;
        results.push(common::format_commit_short(&commit));

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

use std::path::PathBuf;

use clap::Args;
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

    let root_commit_id = repo.store().root_commit_id().clone();

    if let Some(both_count) = resolved.both_count {
        let (lines, anchor) = common::collect_both(
            &repo,
            &args.repo,
            args.pos_rev.as_deref(),
            both_count,
            common::format_commit_full,
        )?;
        for (i, line) in lines.iter().enumerate() {
            if i == anchor {
                println!("{}", common::bold_first_line(line));
            } else {
                println!("{line}");
            }
        }
        return Ok(());
    }

    // Resolve the primary commit ID for highlighting
    let primary_ids = common::resolve_revset(&workspace, &repo, &resolved.primary_rev)?;
    let primary_id = primary_ids.first();

    let mut results: Vec<(String, bool)> = Vec::new();
    let mut count = 0;
    for commit_id in &commit_ids {
        if *commit_id == root_commit_id {
            continue;
        }
        let commit = repo.store().get_commit(commit_id)?;
        let is_primary = primary_id == Some(commit_id);
        results.push((common::format_commit_full(&commit), is_primary));

        count += 1;
        if let Some(limit) = resolved.limit
            && count >= limit
        {
            break;
        }
    }

    for (line, is_primary) in &results {
        if *is_primary {
            println!("{}", common::bold_first_line(line));
        } else {
            println!("{line}");
        }
    }

    Ok(())
}

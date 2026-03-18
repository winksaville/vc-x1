use std::path::PathBuf;
use std::sync::Arc;

use chrono::Offset;
use clap::Args;
use jj_lib::backend::TreeValue;
use jj_lib::commit::Commit;
use jj_lib::matchers::EverythingMatcher;
use jj_lib::merged_tree::TreeDiffIterator;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::{ReadonlyRepo, Repo};
use jj_lib::workspace::Workspace;
use pollster::FutureExt;

use crate::common;

/// Parsed file limit: None (suppress), Some(n) (cap at n), or all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileLimit {
    /// Don't show changed files section at all.
    None,
    /// Show up to N files (with first/last split if truncated).
    Cap(usize),
    /// Show all files.
    All,
}

impl FileLimit {
    fn parse(s: &str) -> Result<Self, String> {
        match s {
            "0" => Ok(FileLimit::None),
            "all" => Ok(FileLimit::All),
            _ => s
                .parse::<usize>()
                .map(FileLimit::Cap)
                .map_err(|_| format!("invalid file limit '{s}': expected number, 0, or 'all'")),
        }
    }
}

#[derive(Args, Debug)]
pub struct ShowArgs {
    /// Revision (with optional .. notation)
    #[arg()]
    pub pos_rev: Option<String>,

    /// Count (commits per dotted side)
    #[arg()]
    pub pos_count: Option<usize>,

    /// Revision to show (default: @)
    #[arg(short, long, default_value = "@")]
    pub revision: String,

    /// Max changed files: number, 0 (none), or 'all' (default: 50)
    #[arg(short = 'f', long = "files", default_value = "50")]
    pub files: String,

    /// Maximum number of commits to show
    #[arg(short, long)]
    pub limit: Option<usize>,

    /// Path to jj repo (default: current directory)
    #[arg(short = 'R', long, default_value = ".")]
    pub repo: PathBuf,
}

pub fn show(args: &ShowArgs) -> Result<(), Box<dyn std::error::Error>> {
    let file_limit = FileLimit::parse(&args.files)?;

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

    // Resolve the primary commit ID for highlighting
    let primary_ids = common::resolve_revset(&workspace, &repo, &resolved.primary_rev)?;
    let primary_id = primary_ids.first();

    if let Some(both_count) = resolved.both_count {
        show_both(
            &workspace,
            &repo,
            args,
            both_count,
            file_limit,
            &root_commit_id,
        )?;
        return Ok(());
    }

    let mut count = 0;
    let mut first = true;
    for commit_id in &commit_ids {
        if *commit_id == root_commit_id {
            continue;
        }

        if !first {
            // Separator between commits
            println!("────────────────────────────────────────");
        }
        first = false;

        let commit = repo.store().get_commit(commit_id)?;
        let is_primary = primary_id == Some(commit_id);

        show_one_commit(&commit, &workspace, &repo, file_limit, is_primary)?;

        count += 1;
        if let Some(limit) = resolved.limit
            && count >= limit
        {
            break;
        }
    }

    Ok(())
}

fn show_both(
    workspace: &Workspace,
    repo: &Arc<ReadonlyRepo>,
    args: &ShowArgs,
    both_count: usize,
    file_limit: FileLimit,
    root_commit_id: &jj_lib::backend::CommitId,
) -> Result<(), Box<dyn std::error::Error>> {
    let bare_rev = args.pos_rev.as_deref().unwrap_or("@");
    let spec = common::parse_dot_rev(bare_rev);
    let anchor_ids = common::resolve_revset(workspace, repo, &spec.rev)?;
    if anchor_ids.is_empty() {
        return Err(format!("no commit found for revision '{}'", spec.rev).into());
    }
    let anchor_id = &anchor_ids[0];

    let ancestor_ids = common::resolve_revset(workspace, repo, &format!("::{}", spec.rev))?;
    let descendant_ids = common::resolve_revset(workspace, repo, &format!("{}::", spec.rev))?;

    // Descendants (newest first), excluding anchor
    let mut desc_ids: Vec<_> = descendant_ids
        .iter()
        .filter(|id| **id != *root_commit_id && **id != *anchor_id)
        .collect();
    if both_count > 0 {
        let start = desc_ids.len().saturating_sub(both_count);
        desc_ids = desc_ids[start..].to_vec();
    } else {
        desc_ids.clear();
    }

    // Ancestors (newest first), excluding anchor
    let mut anc_ids: Vec<_> = Vec::new();
    if both_count > 0 {
        let mut c = 0;
        for id in &ancestor_ids {
            if *id == *root_commit_id || *id == *anchor_id {
                continue;
            }
            anc_ids.push(id);
            c += 1;
            if c >= both_count {
                break;
            }
        }
    }

    let mut first = true;
    for id in &desc_ids {
        if !first {
            println!("────────────────────────────────────────");
        }
        first = false;
        let commit = repo.store().get_commit(id)?;
        show_one_commit(&commit, workspace, repo, file_limit, false)?;
    }

    if !first {
        println!("────────────────────────────────────────");
    }
    first = false;
    let anchor_commit = repo.store().get_commit(anchor_id)?;
    show_one_commit(&anchor_commit, workspace, repo, file_limit, true)?;

    for id in &anc_ids {
        if !first {
            println!("────────────────────────────────────────");
        }
        first = false;
        let commit = repo.store().get_commit(id)?;
        show_one_commit(&commit, workspace, repo, file_limit, false)?;
    }

    Ok(())
}

fn show_one_commit(
    commit: &Commit,
    workspace: &Workspace,
    repo: &Arc<ReadonlyRepo>,
    file_limit: FileLimit,
    is_primary: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Ids
    let ids_line = common::format_commit_short(commit);
    if is_primary {
        println!("Ids:       {}", common::bold(&ids_line));
    } else {
        println!("Ids:       {ids_line}");
    }

    // Author
    let author = commit.author();
    println!(
        "Author:    {} <{}> ({})",
        author.name,
        author.email,
        format_timestamp(&author.timestamp)
    );

    // Committer
    let committer = commit.committer();
    println!(
        "Committer: {} <{}> ({})",
        committer.name,
        committer.email,
        format_timestamp(&committer.timestamp)
    );

    // Parents
    let root_commit_id = repo.store().root_commit_id().clone();
    for parent_id in commit.parent_ids() {
        if *parent_id == root_commit_id {
            continue;
        }
        let parent = repo.store().get_commit(parent_id)?;
        println!("Parent:    {}", common::format_commit_short(&parent));
    }

    // Children
    let children_ids =
        common::resolve_revset(workspace, repo, &format!("children({})", commit.id().hex()))?;
    for child_id in &children_ids {
        if *child_id == root_commit_id {
            continue;
        }
        let child = repo.store().get_commit(child_id)?;
        println!("Child:     {}", common::format_commit_short(&child));
    }

    // Branches
    let branches_str = format_branches(commit, workspace, repo)?;
    println!("Branches:  {branches_str}");

    // Follows
    let follows = find_nearest_tag(commit, workspace, repo, true)?;
    println!("Follows:   {}", follows.as_deref().unwrap_or(""));

    // Precedes
    let precedes = find_nearest_tag(commit, workspace, repo, false)?;
    println!("Precedes:  {}", precedes.as_deref().unwrap_or(""));

    // Description (body only — title is on the Ids line)
    print_description(commit);

    // Changed files
    if file_limit != FileLimit::None {
        print_diff(commit, repo, file_limit)?;
    }

    Ok(())
}

fn find_nearest_tag(
    commit: &Commit,
    workspace: &Workspace,
    repo: &Arc<ReadonlyRepo>,
    ancestor: bool,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let commit_hex = commit.id().hex();

    let revset_str = if ancestor {
        format!("heads(tags() & ::{})", commit_hex)
    } else {
        format!("roots(tags() & {}::)", commit_hex)
    };

    let tag_ids = common::resolve_revset(workspace, repo, &revset_str)?;
    if tag_ids.is_empty() {
        return Ok(None);
    }

    let tag_id = &tag_ids[0];
    let view = repo.view();
    for (name, local_remote) in view.tags() {
        if local_remote.local_target.added_ids().any(|id| id == tag_id) {
            return Ok(Some(name.as_str().to_string()));
        }
    }

    Ok(None)
}

fn format_branches(
    commit: &Commit,
    workspace: &Workspace,
    repo: &Arc<ReadonlyRepo>,
) -> Result<String, Box<dyn std::error::Error>> {
    let commit_hex = commit.id().hex();
    let view = repo.view();
    let mut names: Vec<String> = Vec::new();

    for (name, local_remote) in view.bookmarks() {
        for bookmark_id in local_remote.local_target.added_ids() {
            let revset_str = format!("{} & ::{}", commit_hex, bookmark_id.hex());
            let results = common::resolve_revset(workspace, repo, &revset_str)?;
            if !results.is_empty() {
                names.push(name.as_str().to_string());
                break;
            }
        }

        for (remote_name, remote_ref) in &local_remote.remote_refs {
            if remote_name.as_str() == "git" {
                continue;
            }
            for bookmark_id in remote_ref.target.added_ids() {
                let revset_str = format!("{} & ::{}", commit_hex, bookmark_id.hex());
                let results = common::resolve_revset(workspace, repo, &revset_str)?;
                if !results.is_empty() {
                    names.push(format!("{}@{}", name.as_str(), remote_name.as_str()));
                    break;
                }
            }
        }
    }

    Ok(names.join(", "))
}

fn format_timestamp(ts: &jj_lib::backend::Timestamp) -> String {
    let millis = ts.timestamp.0;
    let secs = millis / 1000;
    let tz_minutes = ts.tz_offset;

    let dt = chrono::DateTime::from_timestamp(secs, 0)
        .unwrap_or_default()
        .with_timezone(
            &chrono::FixedOffset::east_opt(tz_minutes * 60).unwrap_or(chrono::Utc.fix()),
        );

    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn print_description(commit: &Commit) {
    let desc = commit.description();
    let body: String = desc
        .lines()
        .skip(1)
        .skip_while(|l| l.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    println!("Description:");
    for line in body.lines() {
        println!("    {line}");
    }
}

fn print_diff(
    commit: &Commit,
    repo: &Arc<ReadonlyRepo>,
    file_limit: FileLimit,
) -> Result<(), Box<dyn std::error::Error>> {
    let parent_tree = commit.parent_tree(repo.as_ref()).block_on()?;
    let commit_tree = commit.tree();

    let diff_iter = TreeDiffIterator::new(&parent_tree, &commit_tree, &EverythingMatcher);

    let mut lines: Vec<String> = Vec::new();
    for entry in diff_iter {
        let diff = entry.values?;
        let before = diff.before.as_resolved().cloned().flatten();
        let after = diff.after.as_resolved().cloned().flatten();

        if matches!(&before, Some(TreeValue::Tree(_))) || matches!(&after, Some(TreeValue::Tree(_)))
        {
            continue;
        }

        let path_str = entry.path.as_internal_file_string().to_string();

        let line = match (before, after) {
            (None, Some(value)) => {
                format!("    Added {} {path_str}", file_type_str(&value))
            }
            (Some(_), None) => {
                format!(
                    "    Removed {} {path_str}",
                    file_type_str_from_merged(&diff.before),
                )
            }
            (Some(_), Some(value)) => {
                format!("    Modified {} {path_str}", file_type_str(&value))
            }
            (None, None) => continue,
        };
        lines.push(line);
    }

    let total = lines.len();
    let cap = match file_limit {
        FileLimit::All | FileLimit::None => 0,
        FileLimit::Cap(n) => n,
    };
    let show_all = cap == 0 || total <= cap;

    if show_all {
        println!("Changed files: {total}");
        for line in &lines {
            println!("{line}");
        }
    } else {
        let half = cap / 2;
        let tail_start = total.saturating_sub(half);
        let skipped = total - half - half;
        println!("Changed files: {total} (showing {cap})");
        for line in &lines[..half] {
            println!("{line}");
        }
        println!("    ... {skipped} more ...");
        for line in &lines[tail_start..] {
            println!("{line}");
        }
    }

    Ok(())
}

fn file_type_str(value: &TreeValue) -> &'static str {
    match value {
        TreeValue::File {
            executable: false, ..
        } => "regular file",
        TreeValue::File {
            executable: true, ..
        } => "executable file",
        TreeValue::Symlink(_) => "symlink",
        TreeValue::Tree(_) => "tree",
        TreeValue::GitSubmodule(_) => "git submodule",
    }
}

fn file_type_str_from_merged(merged: &jj_lib::merge::Merge<Option<TreeValue>>) -> &'static str {
    if let Some(Some(value)) = merged.as_resolved() {
        file_type_str(value)
    } else {
        "file"
    }
}

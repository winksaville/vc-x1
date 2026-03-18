use std::path::PathBuf;
use std::sync::Arc;

use chrono::Offset;
use clap::Args;
use jj_lib::backend::TreeValue;
use jj_lib::commit::Commit;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::matchers::EverythingMatcher;
use jj_lib::merged_tree::TreeDiffIterator;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::{ReadonlyRepo, Repo};
use pollster::FutureExt;

use crate::common;

#[derive(Args, Debug)]
pub struct ShowArgs {
    /// Revision to show (default: @)
    #[arg(short, long, default_value = "@")]
    pub revision: String,

    /// Path to jj repo (default: current directory)
    #[arg(short = 'R', long, default_value = ".")]
    pub repo: PathBuf,
}

pub fn show(args: &ShowArgs) -> Result<(), Box<dyn std::error::Error>> {
    let (workspace, repo) = common::load_repo(&args.repo)?;
    let commit_ids = common::resolve_revset(&workspace, &repo, &args.revision)?;

    if commit_ids.is_empty() {
        return Err(format!("no commit found for revision '{}'", args.revision).into());
    }

    let root_commit_id = repo.store().root_commit_id().clone();
    let commit_id = &commit_ids[0];
    if *commit_id == root_commit_id {
        return Err("cannot show root commit".into());
    }

    let commit = repo.store().get_commit(commit_id)?;

    print_header(&commit, &repo);
    print_description(&commit);
    print_diff(&commit, &repo)?;

    Ok(())
}

fn print_header(commit: &Commit, repo: &Arc<ReadonlyRepo>) {
    let commit_hex = commit.id().hex();
    let change_hex = encode_reverse_hex(commit.change_id().as_bytes());

    println!("Commit ID: {commit_hex}");
    println!("Change ID: {change_hex}");

    // Bookmarks
    let bookmark_str = format_bookmarks(commit, repo);
    if !bookmark_str.is_empty() {
        println!("Bookmarks: {bookmark_str}");
    }

    // Author
    let author = commit.author();
    println!(
        "Author   : {} <{}> ({})",
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
}

fn format_bookmarks(commit: &Commit, repo: &Arc<ReadonlyRepo>) -> String {
    let view = repo.view();
    let mut names: Vec<String> = Vec::new();

    for (name, local_remote) in view.bookmarks() {
        // Local bookmark
        if local_remote
            .local_target
            .added_ids()
            .any(|id| id == commit.id())
        {
            names.push(name.as_str().to_string());
        }
        // Remote bookmarks
        for (remote_name, remote_ref) in &local_remote.remote_refs {
            if remote_ref.target.added_ids().any(|id| id == commit.id()) {
                names.push(format!("{}@{}", name.as_str(), remote_name.as_str()));
            }
        }
    }

    names.join(" ")
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
    println!();
    if desc.is_empty() {
        println!("    (no description set)");
    } else {
        for line in desc.lines() {
            println!("    {line}");
        }
    }
    println!();
}

fn print_diff(
    commit: &Commit,
    repo: &Arc<ReadonlyRepo>,
) -> Result<(), Box<dyn std::error::Error>> {
    let parent_tree = commit.parent_tree(repo.as_ref()).block_on()?;
    let commit_tree = commit.tree();

    let diff_iter = TreeDiffIterator::new(&parent_tree, &commit_tree, &EverythingMatcher);

    for entry in diff_iter {
        let diff = entry.values?;
        let before = diff.before.as_resolved().cloned().flatten();
        let after = diff.after.as_resolved().cloned().flatten();

        // Skip tree (directory) entries — only show files
        if matches!(&before, Some(TreeValue::Tree(_)))
            || matches!(&after, Some(TreeValue::Tree(_)))
        {
            continue;
        }

        let path_str = entry.path.as_internal_file_string();

        match (before, after) {
            (None, Some(value)) => {
                println!("Added {} {path_str}:", file_type_str(&value));
            }
            (Some(_), None) => {
                println!(
                    "Removed {} {path_str}:",
                    file_type_str_from_merged(&diff.before),
                );
            }
            (Some(_), Some(value)) => {
                println!("Modified {} {path_str}:", file_type_str(&value));
            }
            (None, None) => {
                // Conflicted or both absent — skip
            }
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

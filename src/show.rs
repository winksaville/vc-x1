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

use log::info;

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
    #[command(flatten)]
    pub common: common::CommonArgs,

    /// Max changed files: number, 0 (none), or 'all'
    #[arg(short = 'f', long = "files", default_value = "50")]
    pub files: String,
}

pub fn show(args: &ShowArgs) -> Result<(), Box<dyn std::error::Error>> {
    let file_limit = FileLimit::parse(&args.files)?;
    let c = &args.common;

    let spec = common::resolve_spec(c.pos_rev.as_deref(), c.pos_count, &c.revision, c.limit, "@");
    let hdr = common::resolve_header(&c.label, c.no_label);

    common::for_each_repo(&c.repos, &hdr, |workspace, repo| {
        let (ids, anchor_index) =
            common::collect_ids(workspace, repo, &spec.rev, spec.desc_count, spec.anc_count)?;

        let mut first = true;
        for (i, commit_id) in ids.iter().enumerate() {
            if !first {
                info!("────────────────────────────────────────");
            }
            first = false;

            let commit = repo.store().get_commit(commit_id)?;
            show_one_commit(&commit, workspace, repo, file_limit, i == anchor_index)?;
        }
        Ok(())
    })
}

fn show_one_commit(
    commit: &Commit,
    workspace: &Workspace,
    repo: &Arc<ReadonlyRepo>,
    file_limit: FileLimit,
    is_primary: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Author
    let author = commit.author();
    info!(
        "Author:    {} <{}> ({})",
        author.name,
        author.email,
        format_timestamp(&author.timestamp)
    );

    // Committer
    let committer = commit.committer();
    info!(
        "Committer: {} <{}> ({})",
        committer.name,
        committer.email,
        format_timestamp(&committer.timestamp)
    );

    // Ids
    let ids_line = common::format_commit_short(commit);
    if is_primary {
        info!("Ids:       {}", common::bold(&ids_line));
    } else {
        info!("Ids:       {ids_line}");
    }

    // Parents
    let root_commit_id = repo.store().root_commit_id().clone();
    for parent_id in commit.parent_ids() {
        if *parent_id == root_commit_id {
            continue;
        }
        let parent = repo.store().get_commit(parent_id)?;
        info!("Parent:    {}", common::format_commit_short(&parent));
    }

    // Children
    let children_ids =
        common::resolve_revset(workspace, repo, &format!("children({})", commit.id().hex()))?;
    for child_id in &children_ids {
        if *child_id == root_commit_id {
            continue;
        }
        let child = repo.store().get_commit(child_id)?;
        info!("Child:     {}", common::format_commit_short(&child));
    }

    // Branches
    let branches_str = format_branches(commit, workspace, repo)?;
    info!("Branches:  {branches_str}");

    // Follows
    let follows = find_nearest_tag(commit, workspace, repo, true)?;
    info!("Follows:   {}", follows.as_deref().unwrap_or(""));

    // Precedes
    let precedes = find_nearest_tag(commit, workspace, repo, false)?;
    info!("Precedes:  {}", precedes.as_deref().unwrap_or(""));

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

    info!("Description:");
    for line in body.lines() {
        info!("    {line}");
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
        info!("Changed files: {total}");
        for line in &lines {
            info!("{line}");
        }
    } else {
        info!("Changed files: {total}, showing first {cap}");
        for line in &lines[..cap] {
            info!("{line}");
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::Parser;

    use crate::{Cli, Commands};

    fn parse(args: &[&str]) -> super::ShowArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Show(a) => a,
            _ => panic!("expected Show"),
        }
    }

    #[test]
    fn defaults() {
        let args = parse(&["vc-x1", "show"]);
        assert_eq!(args.common.revision, "@");
        assert!(args.common.repos.is_empty());
        assert_eq!(args.files, "50");
        assert_eq!(args.common.pos_rev, None);
        assert_eq!(args.common.pos_count, None);
        assert_eq!(args.common.limit, None);
    }

    #[test]
    fn with_revision() {
        let args = parse(&["vc-x1", "show", "-r", "@-"]);
        assert_eq!(args.common.revision, "@-");
    }

    #[test]
    fn with_repo() {
        let args = parse(&["vc-x1", "show", "-R", ".claude"]);
        assert_eq!(args.common.repos, vec![PathBuf::from(".claude")]);
    }

    #[test]
    fn positional_rev() {
        let args = parse(&["vc-x1", "show", "@-"]);
        assert_eq!(args.common.pos_rev, Some("@-".to_string()));
        assert_eq!(args.common.pos_count, None);
    }

    #[test]
    fn positional_rev_and_count() {
        let args = parse(&["vc-x1", "show", "@..", "3"]);
        assert_eq!(args.common.pos_rev, Some("@..".to_string()));
        assert_eq!(args.common.pos_count, Some(3));
    }

    #[test]
    fn with_file_limit_flag() {
        let args = parse(&["vc-x1", "show", "-f", "0"]);
        assert_eq!(args.files, "0");
    }

    #[test]
    fn with_file_limit_all() {
        let args = parse(&["vc-x1", "show", "-f", "all"]);
        assert_eq!(args.files, "all");
    }

    #[test]
    fn with_commit_limit() {
        let args = parse(&["vc-x1", "show", "-n", "5"]);
        assert_eq!(args.common.limit, Some(5));
    }

    #[test]
    fn all_opts() {
        let args = parse(&[
            "vc-x1", "show", "-r", "@--", "-R", "/tmp", "-f", "100", "-n", "3",
        ]);
        assert_eq!(args.common.revision, "@--");
        assert_eq!(args.common.repos, vec![PathBuf::from("/tmp")]);
        assert_eq!(args.files, "100");
        assert_eq!(args.common.limit, Some(3));
    }

    #[test]
    fn multi_repo() {
        let args = parse(&["vc-x1", "show", "-R", ".", "-R", ".claude"]);
        assert_eq!(
            args.common.repos,
            vec![PathBuf::from("."), PathBuf::from(".claude")]
        );
    }
}

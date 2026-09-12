//! The work side of a transcript line: the work commits its agent
//! commit's push published, and their diffs as the window.
//!
//! An agent commit's `ochid:` trailers name every work commit the same
//! push published, so the window is those commits' diffs. A line that
//! is a transcript write narrows the window to the file it wrote, and
//! a line of discussion takes the diffs whole.
//!
//! - `Hunk` and `hunks_of`: a commit's changed regions per file, by
//!   jj-lib's line diff, the new side's line range with the removed
//!   and added lines.
//! - `written_files`: the work files a transcript entry wrote, among
//!   the files a commit changed.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures::AsyncReadExt as _;
use jj_lib::backend::TreeValue;
use jj_lib::commit::Commit;
use jj_lib::diff::{ContentDiff, DiffHunkKind};
use jj_lib::matchers::EverythingMatcher;
use jj_lib::merged_tree::TreeDiffIterator;
use jj_lib::repo::{ReadonlyRepo, Repo};
use jj_lib::repo_path::RepoPath;
use jj_lib::store::Store;
use pollster::FutureExt;

use crate::transcript::{ContentBlock, Entry, EntryKind};

/// One changed region of a file in a commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    /// The file, relative to the repo root.
    pub file: PathBuf,
    /// The first line of the region on the new side, 1-based. For a
    /// pure removal, the line the removed lines sat before.
    pub start: usize,
    /// The region's line count on the new side, zero for a removal.
    pub len: usize,
    /// The lines the commit removed, without newlines.
    pub removed: Vec<String>,
    /// The lines the commit added, without newlines.
    pub added: Vec<String>,
}

/// A file's bytes on one side of a diff, empty when the side has no
/// plain file.
fn content(
    store: &Arc<Store>,
    path: &RepoPath,
    value: Option<&TreeValue>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let Some(TreeValue::File { id, .. }) = value else {
        return Ok(Vec::new());
    };
    let mut reader = store.read_file(path, id).block_on()?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).block_on()?;
    Ok(buf)
}

/// The lines of a byte range, without newlines.
fn lines_of(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(str::to_string)
        .collect()
}

/// The files `commit` changed, relative to the repo root, trees left
/// out.
pub fn changed_files(
    repo: &Arc<ReadonlyRepo>,
    commit: &Commit,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let parent_tree = commit.parent_tree(repo.as_ref()).block_on()?;
    let tree = commit.tree();
    let mut out = Vec::new();
    for entry in TreeDiffIterator::new(&parent_tree, &tree, &EverythingMatcher) {
        let diff = entry.values?;
        let after = diff.after.as_resolved().cloned().flatten();
        let before = diff.before.as_resolved().cloned().flatten();
        if matches!(after, Some(TreeValue::Tree(_))) || matches!(before, Some(TreeValue::Tree(_))) {
            continue;
        }
        out.push(PathBuf::from(entry.path.as_internal_file_string()));
    }
    Ok(out)
}

/// The changed regions of `commit`, every file's or only those in
/// `only`, in file order and within a file in line order.
///
/// A merge commit diffs against its merged parents, so a trapezoid
/// merge's regions are the ladder's net change.
pub fn hunks_of(
    repo: &Arc<ReadonlyRepo>,
    commit: &Commit,
    only: Option<&[PathBuf]>,
) -> Result<Vec<Hunk>, Box<dyn std::error::Error>> {
    let parent_tree = commit.parent_tree(repo.as_ref()).block_on()?;
    let tree = commit.tree();
    let store = repo.store();
    let mut hunks = Vec::new();
    for entry in TreeDiffIterator::new(&parent_tree, &tree, &EverythingMatcher) {
        let file = PathBuf::from(entry.path.as_internal_file_string());
        if only.is_some_and(|o| !o.contains(&file)) {
            continue;
        }
        let diff = entry.values?;
        let before = diff.before.as_resolved().cloned().flatten();
        let after = diff.after.as_resolved().cloned().flatten();
        if matches!(after, Some(TreeValue::Tree(_))) || matches!(before, Some(TreeValue::Tree(_))) {
            continue;
        }
        let old = content(store, &entry.path, before.as_ref())?;
        let new = content(store, &entry.path, after.as_ref())?;
        let line_diff = ContentDiff::by_line([&old[..], &new[..]]);
        for range in line_diff.hunk_ranges() {
            if range.kind == DiffHunkKind::Matching {
                continue;
            }
            let (old_r, new_r) = (range.ranges[0].clone(), range.ranges[1].clone());
            let start = new[..new_r.start].iter().filter(|b| **b == b'\n').count() + 1;
            let added = lines_of(&new[new_r]);
            hunks.push(Hunk {
                file: file.clone(),
                start,
                len: added.len(),
                removed: lines_of(&old[old_r]),
                added,
            });
        }
    }
    Ok(hunks)
}

/// The work files a transcript entry wrote, among `changed`, empty
/// for anything that is not a write.
///
/// - `Write`, `Edit`, and `MultiEdit` name one by `file_path`, compared
///   by trailing components since a transcript holds the writing
///   machine's absolute paths, the longest match winning.
/// - A `Bash` call can write several, so it names every changed file
///   whose repo-relative path its command holds, and every changed
///   file whose name it holds when no other changed file shares that
///   name.
pub fn written_files(entry: &Entry, changed: &[PathBuf]) -> Vec<PathBuf> {
    let EntryKind::Assistant { content, .. } = &entry.kind else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = Vec::new();
    for b in content {
        let ContentBlock::ToolUse { name, input, .. } = b else {
            continue;
        };
        match name.as_str() {
            "Write" | "Edit" | "MultiEdit" => {
                let Some(fp) = input["file_path"].as_str().map(Path::new) else {
                    continue;
                };
                if let Some(c) = changed
                    .iter()
                    .filter(|c| fp.ends_with(c))
                    .max_by_key(|c| c.components().count())
                {
                    out.push(c.clone());
                }
            }
            "Bash" => {
                let Some(cmd) = input["command"].as_str() else {
                    continue;
                };
                let name_of = |c: &PathBuf| {
                    c.file_name()
                        .and_then(|n| n.to_str())
                        .map(str::to_string)
                        .unwrap_or_default() // OK: a changed file has a name
                };
                for c in changed {
                    let by_path = c
                        .to_str()
                        .is_some_and(|s| s.contains('/') && cmd.contains(s));
                    let name = name_of(c);
                    let unique = changed.iter().filter(|o| name_of(o) == name).count() == 1;
                    if by_path || (unique && !name.is_empty() && cmd.contains(&name)) {
                        out.push(c.clone());
                    }
                }
            }
            _ => {}
        }
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common;
    use crate::lookup::partner::tests::by_chid;
    use crate::lookup::tests::dr1;

    /// Every push's work commit has a region adding each line the
    /// fixture records as first arriving in it.
    #[test]
    fn dr1_hunks_carry_each_arriving_line() {
        let Some((root, rel)) = dr1() else { return };
        let (ws, repo) = common::load_repo(&root).unwrap();
        let pushes = rel["pushes"].as_array().unwrap();
        let mut checked = 0;
        for w in rel["writes"].as_array().unwrap() {
            let Some(arrives) = w["arrives"].as_u64() else {
                continue;
            };
            let commit = by_chid(
                &ws,
                &repo,
                pushes[arrives as usize]["work"].as_str().unwrap(),
            );
            let file = Path::new(w["file"].as_str().unwrap());
            let hunks = hunks_of(&repo, &commit, Some(&[file.to_path_buf()])).unwrap();
            assert!(hunks.iter().all(|h| h.file == file));
            let text = w["text"].as_str().unwrap();
            let h = hunks
                .iter()
                .find(|h| h.added.iter().any(|l| l == text))
                .unwrap_or_else(|| panic!("{text}"));
            let at = h.added.iter().position(|l| l == text).unwrap();
            let lines = crate::lookup::blame::lines_at(&commit, file).unwrap();
            assert_eq!(lines[h.start - 1 + at], text);
            checked += 1;
        }
        assert_eq!(checked, 10);
    }

    /// The classifier names the files for each tool and none for text.
    #[test]
    fn written_files_names_what_a_call_wrote() {
        let entry = |v: serde_json::Value| {
            crate::transcript::parse_str(&v.to_string())
                .entries
                .remove(0)
        };
        let tool = |name: &str, input: serde_json::Value| {
            entry(
                serde_json::json!({"type": "assistant", "message": {"content": [
                {"type": "tool_use", "id": "t", "name": name, "input": input}]}}),
            )
        };
        let p = PathBuf::from;
        let changed = vec![p("notes.md"), p("docs/notes.md"), p("Cargo.toml")];
        let w = tool(
            "Edit",
            serde_json::json!({"file_path": "/m/w/docs/notes.md"}),
        );
        assert_eq!(written_files(&w, &changed), vec![p("docs/notes.md")]);
        let w = tool(
            "Bash",
            serde_json::json!({"command": "echo x >> docs/notes.md"}),
        );
        assert_eq!(written_files(&w, &changed), vec![p("docs/notes.md")]);
        let w = tool("Bash", serde_json::json!({"command": "echo x >> notes.md"}));
        assert_eq!(
            written_files(&w, &changed),
            Vec::<PathBuf>::new(),
            "two are named notes.md"
        );
        let w = tool(
            "Bash",
            serde_json::json!({"command": "sed -i s/a/b/ Cargo.toml && echo x >> docs/notes.md"}),
        );
        assert_eq!(
            written_files(&w, &changed),
            vec![p("Cargo.toml"), p("docs/notes.md")]
        );
        let t = entry(
            serde_json::json!({"type": "assistant", "message": {"content": [
            {"type": "text", "text": "notes.md"}]}}),
        );
        assert!(written_files(&t, &changed).is_empty());
    }
}

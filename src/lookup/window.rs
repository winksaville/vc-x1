//! The agent side of a work line: its partner's time-window in the
//! session files, and the transcript write that put the line in its
//! file.
//!
//! The agent repo's session files are the timeline and its commits
//! only cut them into windows, so a window is read from a partner's
//! diff and the write is searched for over the timeline, not over
//! the window alone.
//!
//! - `Span` and `window_of`: the lines an agent commit appended to
//!   each session file, which is two spans across a restart.
//! - `Timeline`: every session file's entries on disk, sessions in
//!   the order they started and each in file order.
//! - `writes_line`: the classifier, `Write`, `Edit`, and `MultiEdit`
//!   by name and a `Bash` call by what its command carries, never a
//!   push call quoting the line.
//! - `Timeline::find_write`: the search, backwards from the window's
//!   end for a line written and set aside, then forwards past it for
//!   a line a later amend brought in.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures::AsyncReadExt as _;
use jj_lib::backend::TreeValue;
use jj_lib::commit::Commit;
use jj_lib::matchers::EverythingMatcher;
use jj_lib::merged_tree::TreeDiffIterator;
use jj_lib::repo::{ReadonlyRepo, Repo};
use jj_lib::repo_path::RepoPath;
use jj_lib::store::Store;
use pollster::FutureExt;
use serde_json::Value;

use crate::transcript::{ContentBlock, Entry, EntryKind, parse_str};

/// Lines `start..=end`, 1-based, that a commit appended to one
/// session file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// The session file, relative to the agent repo's root.
    pub file: PathBuf,
    /// The first appended line.
    pub start: usize,
    /// The last appended line.
    pub end: usize,
}

/// Line count of a file's content at one side of a diff, zero when
/// the side has no file.
fn line_count(
    store: &Arc<Store>,
    path: &RepoPath,
    value: Option<&TreeValue>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let Some(TreeValue::File { id, .. }) = value else {
        return Ok(0);
    };
    let mut reader = store.read_file(path, id).block_on()?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).block_on()?;
    Ok(buf.split_inclusive(|b| *b == b'\n').count())
}

/// The spans `commit` appended to the agent repo's session files: for
/// each `.jsonl` the commit changed, the lines past its parent's count.
///
/// Session files are append-only, so the lines past the parent's
/// count are exactly what the commit added. A file that shrank is
/// not a window and is skipped.
pub fn window_of(
    repo: &Arc<ReadonlyRepo>,
    commit: &Commit,
) -> Result<Vec<Span>, Box<dyn std::error::Error>> {
    let parent_tree = commit.parent_tree(repo.as_ref()).block_on()?;
    let tree = commit.tree();
    let store = repo.store();
    let mut spans = Vec::new();
    for entry in TreeDiffIterator::new(&parent_tree, &tree, &EverythingMatcher) {
        let path = entry.path.as_internal_file_string().to_string();
        if !path.ends_with(".jsonl") {
            continue;
        }
        let diff = entry.values?;
        let before = diff.before.as_resolved().cloned().flatten();
        let after = diff.after.as_resolved().cloned().flatten();
        let n_before = line_count(store, &entry.path, before.as_ref())?;
        let n_after = line_count(store, &entry.path, after.as_ref())?;
        if n_after > n_before {
            spans.push(Span {
                file: PathBuf::from(path),
                start: n_before + 1,
                end: n_after,
            });
        }
    }
    Ok(spans)
}

/// One entry's place in the timeline.
pub struct Placed {
    /// The session file, relative to the agent repo's root.
    pub file: PathBuf,
    /// The parsed entry, its line number included.
    pub entry: Entry,
}

/// Every session file's entries, sessions ordered by their first
/// timestamp and each in file order, which is the order the lines
/// were appended.
pub struct Timeline {
    /// The entries in timeline order.
    pub entries: Vec<Placed>,
}

impl Timeline {
    /// Load every `.jsonl` at the top of the agent repo's root.
    ///
    /// Malformed lines, a live session's truncated last line, are
    /// left out, since no write can be among them.
    pub fn load(agent_root: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut sessions = Vec::new();
        for dirent in std::fs::read_dir(agent_root)? {
            let path = dirent?.path();
            if path.extension().is_none_or(|e| e != "jsonl") {
                continue;
            }
            let text = std::fs::read_to_string(&path)?;
            let parsed = parse_str(&text);
            let first = parsed
                .entries
                .iter()
                .find_map(|e| e.meta.timestamp.clone())
                .unwrap_or_default(); // OK: an undated session sorts first
            let file = PathBuf::from(path.file_name().ok_or("session file has no name")?);
            sessions.push((first, file, parsed.entries));
        }
        sessions.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
        let entries = sessions
            .into_iter()
            .flat_map(|(_, file, entries)| {
                entries.into_iter().map(move |entry| Placed {
                    file: file.clone(),
                    entry,
                })
            })
            .collect();
        Ok(Timeline { entries })
    }

    /// The timeline position of `file`'s line `line`.
    pub fn position(&self, file: &Path, line: usize) -> Option<usize> {
        self.entries
            .iter()
            .position(|p| p.file == file && p.entry.line_no == line)
    }

    /// The position of a window's last entry: the latest span end in
    /// timeline order.
    pub fn window_end(&self, spans: &[Span]) -> Option<usize> {
        spans
            .iter()
            .filter_map(|s| self.position(&s.file, s.end))
            .max()
    }

    /// The transcript write of `text` into `rel`: backwards from
    /// `end` first, since the line was written before the push, then
    /// forwards past it, for a line a later amend brought in.
    pub fn find_write(&self, end: usize, rel: &Path, text: &str) -> Option<usize> {
        let end = end.min(self.entries.len().saturating_sub(1));
        (0..=end)
            .rev()
            .chain(end + 1..self.entries.len())
            .find(|&i| writes_line(&self.entries[i].entry, rel, text))
    }
}

/// True when a tool call's `file_path` names `rel`, compared by
/// trailing components, since a transcript holds the writing
/// machine's absolute paths.
fn names_file(file_path: Option<&str>, rel: &Path) -> bool {
    file_path.is_some_and(|p| Path::new(p).ends_with(rel))
}

/// True when the command runs a push, which quotes a commit body and
/// never writes a work file.
///
/// A push runs where a shell segment starts with it, after `&&`,
/// `||`, `;`, `|`, or a newline and past any `NAME=value`
/// assignments. A mention elsewhere, a heredoc's prose naming the
/// command, is not a run, and the probes' writes include such
/// heredocs.
fn is_push_call(command: &str) -> bool {
    const PUSHES: [&str; 4] = [
        "vc-x1 push",
        "vc-x1-dev push",
        "vc-x1 squash-push",
        "vc-x1-dev squash-push",
    ];
    command
        .split(['\n', ';', '|', '&'])
        .map(|segment| {
            let mut rest = segment.trim_start();
            while let Some((word, tail)) = rest.split_once(' ')
                && word.contains('=')
                && !word.starts_with('-')
            {
                rest = tail.trim_start();
            }
            rest
        })
        .any(|segment| PUSHES.iter().any(|p| segment.starts_with(p)))
}

/// True when one input writes `text` into `rel`.
fn input_writes(name: &str, input: &Value, rel: &Path, text: &str) -> bool {
    let carries = |key: &str| input[key].as_str().is_some_and(|s| s.contains(text));
    match name {
        "Write" => names_file(input["file_path"].as_str(), rel) && carries("content"),
        "Edit" => names_file(input["file_path"].as_str(), rel) && carries("new_string"),
        "MultiEdit" => {
            names_file(input["file_path"].as_str(), rel)
                && input["edits"].as_array().is_some_and(|edits| {
                    edits
                        .iter()
                        .any(|e| e["new_string"].as_str().is_some_and(|s| s.contains(text)))
                })
        }
        "Bash" => input["command"].as_str().is_some_and(|c| {
            let file = rel.file_name().and_then(|f| f.to_str()).unwrap_or(""); // OK: a rel path from locate has a name
            !is_push_call(c) && c.contains(text) && !file.is_empty() && c.contains(file)
        }),
        _ => false,
    }
}

/// True when the entry is a tool call that writes `text` into `rel`.
///
/// Assistant text, tool results, and push calls quoting the line are
/// not writes, which is what the probes found the first hit often is.
/// The text is compared trimmed, and blank text matches nothing.
pub fn writes_line(entry: &Entry, rel: &Path, text: &str) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }
    let EntryKind::Assistant { content, .. } = &entry.kind else {
        return false;
    };
    content.iter().any(|b| match b {
        ContentBlock::ToolUse { name, input, .. } => input_writes(name, input, rel, text),
        _ => false,
    })
}

/// One entry as a line of the rendered window: file and line, time,
/// role, and a gist of what it holds.
pub fn render(p: &Placed) -> String {
    let e = &p.entry;
    let time = crate::bot_session::short_time(e.meta.timestamp.as_deref());
    let (role, gist) = match &e.kind {
        EntryKind::User { content, .. } => match content.first() {
            Some(ContentBlock::ToolResult { text, .. }) => ("result", first_line(text)),
            Some(ContentBlock::Text { text }) => ("user", first_line(text)),
            _ => ("user", String::new()),
        },
        EntryKind::Assistant { content, .. } => match content.first() {
            Some(ContentBlock::ToolUse { name, input, .. }) => {
                ("tool", crate::bot_session::tool_use_gist(name, input))
            }
            Some(ContentBlock::Text { text }) => ("assistant", first_line(text)),
            Some(ContentBlock::Thinking { .. }) => ("thinking", String::new()),
            _ => ("assistant", String::new()),
        },
        EntryKind::System { subtype } => ("system", subtype.clone().unwrap_or_default()), // OK: obvious
        EntryKind::Other { entry_type } => (entry_type.as_str(), String::new()),
    };
    format!("{}:{} {time} {role} {gist}", p.file.display(), e.line_no)
        .trim_end()
        .to_string()
}

/// A text's first line, cut to a readable width.
fn first_line(text: &str) -> String {
    let line = text.lines().next().unwrap_or(""); // OK: obvious
    let cut: String = line.chars().take(80).collect();
    if cut.len() < line.len() {
        format!("{cut}...")
    } else {
        cut
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common;
    use crate::lookup::partner::tests::by_chid;
    use crate::lookup::tests::dr1;

    /// Every push's window, read from its agent commit's diff, is the
    /// window the fixture build read with jj.
    #[test]
    fn dr1_windows_match_the_record() {
        let Some((root, rel)) = dr1() else { return };
        let (aw, ar) = common::load_repo(&root.join(".claude")).unwrap();
        for p in rel["pushes"].as_array().unwrap() {
            let agent = by_chid(&aw, &ar, p["agent"].as_str().unwrap());
            let found = window_of(&ar, &agent).unwrap();
            let mut want: Vec<Span> = p["window"]
                .as_array()
                .unwrap()
                .iter()
                .map(|w| Span {
                    file: PathBuf::from(format!("{}.jsonl", w["session"].as_str().unwrap())),
                    start: w["start"].as_u64().unwrap() as usize,
                    end: w["end"].as_u64().unwrap() as usize,
                })
                .collect();
            let mut found = found;
            found.sort_by(|a, b| a.file.cmp(&b.file));
            want.sort_by(|a, b| a.file.cmp(&b.file));
            assert_eq!(found, want, "{}", p["title"]);
        }
    }

    /// The timeline puts the restart's second session after the first,
    /// and a window spanning both ends in the second.
    #[test]
    fn dr1_timeline_orders_sessions_and_spans_a_restart() {
        let Some((root, rel)) = dr1() else { return };
        let t = Timeline::load(&root.join(".claude")).unwrap();
        let s1 = PathBuf::from(format!("{}.jsonl", rel["sessions"]["s1"].as_str().unwrap()));
        let s2 = PathBuf::from(format!("{}.jsonl", rel["sessions"]["s2"].as_str().unwrap()));
        assert_eq!(t.entries.first().unwrap().file, s1);
        assert_eq!(t.entries.last().unwrap().file, s2);
        let (aw, ar) = common::load_repo(&root.join(".claude")).unwrap();
        let restart = rel["pushes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["window"].as_array().unwrap().len() == 2)
            .unwrap();
        let agent = by_chid(&aw, &ar, restart["agent"].as_str().unwrap());
        let spans = window_of(&ar, &agent).unwrap();
        let end = t.window_end(&spans).unwrap();
        assert_eq!(t.entries[end].file, s2);
    }

    /// The classifier takes a write and refuses assistant text, a
    /// result, another file, and a push call quoting the line.
    #[test]
    fn classifier_takes_writes_and_refuses_the_rest() {
        let line = |v: serde_json::Value| parse_str(&v.to_string()).entries.remove(0);
        let tool = |name: &str, input: serde_json::Value| {
            line(
                serde_json::json!({"type": "assistant", "message": {"content": [
                {"type": "tool_use", "id": "t", "name": name, "input": input}]}}),
            )
        };
        let rel = Path::new("docs/notes.md");
        let text = "A line to find.";
        assert!(writes_line(
            &tool(
                "Write",
                serde_json::json!({"file_path": "/x/docs/notes.md", "content": "A line to find.\n"})
            ),
            rel,
            text
        ));
        assert!(writes_line(
            &tool(
                "Edit",
                serde_json::json!({"file_path": "/y/docs/notes.md", "old_string": "a", "new_string": "  A line to find."})
            ),
            rel,
            text
        ));
        assert!(writes_line(
            &tool(
                "MultiEdit",
                serde_json::json!({"file_path": "/y/docs/notes.md", "edits": [{"new_string": "x"}, {"new_string": "A line to find."}]})
            ),
            rel,
            text
        ));
        assert!(writes_line(
            &tool(
                "Bash",
                serde_json::json!({"command": "echo 'A line to find.' >> docs/notes.md"})
            ),
            rel,
            text
        ));
        assert!(!writes_line(
            &tool(
                "Edit",
                serde_json::json!({"file_path": "/y/docs/other.md", "new_string": "A line to find."})
            ),
            rel,
            text
        ));
        assert!(!writes_line(
            &tool(
                "Bash",
                serde_json::json!({"command": "vc-x1 push b --body \"notes.md: A line to find.\""})
            ),
            rel,
            text
        ));
        assert!(!writes_line(
            &tool(
                "Bash",
                serde_json::json!({"command": "cd /w && VC=1 vc-x1-dev push b --body \"notes.md: A line to find.\""})
            ),
            rel,
            text
        ));
        assert!(writes_line(
            &tool(
                "Bash",
                serde_json::json!({"command": "cat > docs/notes.md <<'EOF'\nA line to find.\nmade with `vc-x1 push`, twice\nEOF"})
            ),
            rel,
            text
        ));
        assert!(!writes_line(
            &line(
                serde_json::json!({"type": "assistant", "message": {"content": [
                {"type": "text", "text": "notes.md gets A line to find."}]}})
            ),
            rel,
            text
        ));
        assert!(!writes_line(
            &tool(
                "Write",
                serde_json::json!({"file_path": "/x/docs/notes.md", "content": "\n"})
            ),
            rel,
            "   "
        ));
    }
}

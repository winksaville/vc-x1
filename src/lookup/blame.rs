//! Blame: the commit a work line arrived in, and the commit that
//! first wrote it when the two differ because the line moved.
//!
//! Blame reports where a line arrived, whatever its history, so a
//! cycle-record line moved from In Progress to Closed blames to the
//! closing while its write is in the opening's window. The reach
//! back is a text search through the ancestors' diffs, which finds
//! the earliest commit whose diff carries the line.
//!
//! - `Origin`: the blamed commit, the line's number there, the
//!   earlier writing commit when the line moved, and the line's text
//!   at the starting tree.
//! - `origin`: blame at a starting commit, whose tree must hold the
//!   line, the working copy's or a landmark's.
//! - `lines_at`: a file's lines at a commit, for a line the working
//!   copy no longer holds.

use std::path::Path;
use std::sync::Arc;

use jj_lib::annotate::FileAnnotator;
use jj_lib::commit::Commit;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::{ReadonlyRepo, Repo};
use jj_lib::repo_path::RepoPathBuf;
use jj_lib::workspace::Workspace;
use pollster::FutureExt;

use crate::common;

/// Where a line came from.
#[derive(Debug)]
pub struct Origin {
    /// The commit blame names, where the line arrived.
    pub commit: Commit,
    /// The line's number in `commit`'s file, 1-based.
    pub line_at_origin: usize,
    /// The earliest ancestor whose diff carries the line's text,
    /// when that is not `commit`: the line moved, and this is where
    /// it was written.
    pub written: Option<Commit>,
    /// The line's text at the starting tree, without its newline.
    pub text: String,
}

/// The shortest line the reach back trusts: shorter text recurs
/// too easily for a diff search to name its first writer.
const REACH_MIN_CHARS: usize = 8;

/// A file's lines at `commit`, the newline stripped from each.
#[cfg(test)]
pub fn lines_at(commit: &Commit, rel: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let path = RepoPathBuf::from_relative_path(rel)?;
    let annotator = FileAnnotator::from_commit(commit, &path).block_on()?;
    let annotation = annotator.to_annotation();
    let text = String::from_utf8_lossy(annotation.text());
    Ok(text.lines().map(str::to_string).collect())
}

/// Blame line `line` of `rel` at `start`, and reach back past a
/// move.
///
/// - The annotator runs over `start`'s ancestors, so `start` is a
///   tree that holds the line, the working copy's or a landmark's.
/// - The reach back asks for the roots of the ancestors whose diff
///   carries the line's text. One root other than the blamed commit
///   is the writer, several is ambiguity, and either way the blamed
///   commit stands.
pub fn origin(
    workspace: &Workspace,
    repo: &Arc<ReadonlyRepo>,
    start: &Commit,
    rel: &Path,
    line: usize,
) -> Result<Origin, Box<dyn std::error::Error>> {
    let mut found = arrived(workspace, repo, start, rel, line)?;
    found.written = reach_back(workspace, repo, &found.commit, &found.text)?;
    Ok(found)
}

/// Blame alone, with no reach back: the commit the line arrived in.
///
/// What a session file's line wants, since a session file is
/// append-only and a line arrives where it was written.
pub fn arrived(
    workspace: &Workspace,
    repo: &Arc<ReadonlyRepo>,
    start: &Commit,
    rel: &Path,
    line: usize,
) -> Result<Origin, Box<dyn std::error::Error>> {
    let path = RepoPathBuf::from_relative_path(rel)?;
    let mut annotator = FileAnnotator::from_commit(start, &path).block_on()?;
    let domain = common::resolve_expression(workspace, repo, &format!("::{}", start.id().hex()))?;
    annotator.compute(repo.as_ref(), &domain).block_on()?;
    let annotation = annotator.to_annotation();
    let count = annotation.line_origins().count();
    let Some((found, text)) = annotation.line_origins().nth(line - 1) else {
        return Err(format!(
            "{}: {count} line(s) at {}, so there is no line {line}",
            rel.display(),
            &start.id().hex()[..12]
        )
        .into());
    };
    let found = match found {
        Ok(resolved) => resolved,
        Err(pending) => pending,
    };
    let commit = repo.store().get_commit(&found.commit_id)?;
    let text = String::from_utf8_lossy(text)
        .trim_end_matches(['\n', '\r'])
        .to_string();
    Ok(Origin {
        commit,
        line_at_origin: found.line_number + 1,
        written: None,
        text,
    })
}

/// The earliest ancestor of `commit` whose diff carries `text`,
/// when it is not `commit` itself and the text is long enough to
/// trust.
fn reach_back(
    workspace: &Workspace,
    repo: &Arc<ReadonlyRepo>,
    commit: &Commit,
    text: &str,
) -> Result<Option<Commit>, Box<dyn std::error::Error>> {
    let needle = text.trim();
    if needle.chars().count() < REACH_MIN_CHARS {
        return Ok(None);
    }
    let escaped = needle.replace('\\', "\\\\").replace('"', "\\\"");
    let revset = format!(
        "roots(diff_lines(substring:\"{escaped}\") & ::{})",
        commit.id().hex()
    );
    let ids = common::resolve_revset(workspace, repo, &revset)?;
    match ids.as_slice() {
        [id] if id != commit.id() => Ok(Some(repo.store().get_commit(id)?)),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lookup::partner::chid;
    use crate::lookup::partner::tests::by_chid;
    use crate::lookup::tests::dr1;

    /// At every push whose tree holds a line, the commit that wrote
    /// it, the reach back's answer or else blame's, is the push that
    /// first carried it, and a line that really moved blames to the
    /// closing that moved it.
    #[test]
    fn dr1_blame_reaches_back_to_the_commit_that_first_carried_the_line() {
        let Some((root, rel)) = dr1() else { return };
        let (ws, repo) = common::load_repo(&root).unwrap();
        let pushes = rel["pushes"].as_array().unwrap();
        let mut checked = 0;
        let mut moved = 0;
        for w in rel["writes"].as_array().unwrap() {
            let Some(arrives) = w["arrives"].as_u64() else {
                continue;
            };
            let file = Path::new(w["file"].as_str().unwrap());
            let text = w["text"].as_str().unwrap();
            for (i, p) in pushes.iter().enumerate().skip(arrives as usize) {
                let commit = by_chid(&ws, &repo, p["work"].as_str().unwrap());
                let lines = lines_at(&commit, file).unwrap();
                let Some(idx) = lines.iter().position(|l| l == text) else {
                    continue;
                };
                let o = origin(&ws, &repo, &commit, file, idx + 1).unwrap();
                assert_eq!(o.text, text);
                let writer = o.written.as_ref().unwrap_or(&o.commit);
                assert_eq!(
                    chid(writer),
                    pushes[arrives as usize]["work"],
                    "{text} at push {i}"
                );
                if w["moved_at"].as_u64() == Some(i as u64) {
                    assert_eq!(chid(&o.commit), p["work"], "{text} moved at push {i}");
                    assert!(o.written.is_some(), "{text} moved at push {i}");
                    moved += 1;
                }
                checked += 1;
            }
        }
        assert!(checked >= 20, "{checked}");
        assert_eq!(moved, 2);
    }

    /// From main's tree, every line still on main blames to the commit
    /// jj's own annotate names, recorded by the fixture's build.
    #[test]
    fn dr1_blame_on_main_matches_jj_annotate() {
        let Some((root, rel)) = dr1() else { return };
        let (ws, repo) = common::load_repo(&root).unwrap();
        let main = by_chid(&ws, &repo, "main");
        for w in rel["writes"].as_array().unwrap() {
            let Some(line) = w["line_on_main"].as_u64() else {
                continue;
            };
            let file = Path::new(w["file"].as_str().unwrap());
            let o = origin(&ws, &repo, &main, file, line as usize).unwrap();
            assert_eq!(o.text, w["text"]);
            assert_eq!(chid(&o.commit), w["blamed_on_main"], "{}", w["text"]);
        }
        let err = origin(&ws, &repo, &main, Path::new("notes.md"), 99).unwrap_err();
        assert!(err.to_string().contains("no line 99"), "{err}");
    }
}

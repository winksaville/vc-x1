//! The cycle-record's mechanical edits, as text: what `cycle --open`
//! writes into `TODO.md` and `zTOC.md`.
//!
//! - The record's shape is the agent-files', so the tool knows it
//!   without knowing the work: `TODO.md`'s `## In Progress`, `## Todo`,
//!   and `## Closed`, and `zTOC.md`'s `## Ladder`.
//! - Every function here takes a file's text and returns its new
//!   text, so a refusal comes before anything is written and a dry
//!   run is the same code.
//! - A heading inside a fenced code block is not one, as the closing's
//!   record move has it.

use std::path::{Path, PathBuf};

use crate::commit::{NONE_IN_PROGRESS, RECORD, section};
use crate::validate_anchors::anchor_slug;

/// The ladder's file, beside the record.
pub(crate) const LADDER_FILE: &str = "zTOC.md";

/// The ladder's heading in [`LADDER_FILE`].
const LADDER: &str = "## Ladder";

/// What an item of the scaffold reads until it is written.
const TO_WRITE: &str = "_To write._";

/// The lines of `text` outside fenced code, each with its byte offset
/// and its text less the line ending.
fn prose_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut fenced = false;
    let mut at = 0;
    text.split_inclusive('\n').filter_map(move |line| {
        let start = at;
        at += line.len();
        let bare = line.trim_end_matches(['\n', '\r']);
        if bare.starts_with("```") {
            fenced = !fenced;
            return None;
        }
        (!fenced).then_some((start, bare))
    })
}

/// A section's body with its content replaced: the intro, everything
/// before the first line `starts` accepts, is kept, then `content`,
/// none when it is empty, and a blank line before the next heading
/// unless the file ends there.
fn with_content(
    text: &str,
    (start, end): (usize, usize),
    starts: impl Fn(&str) -> bool,
    content: &str,
) -> String {
    let body = &text[start..end];
    let from = prose_lines(body)
        .find(|(_, l)| starts(l))
        .map_or(body.len(), |(at, _)| at);
    let intro = body[..from].trim_end();
    let content = content.trim_end();
    let mut new = String::new();
    if !intro.trim().is_empty() {
        new.push_str(intro);
        new.push('\n');
    }
    if !content.is_empty() {
        new.push('\n');
        new.push_str(content);
        new.push('\n');
    }
    if end != text.len() {
        new.push('\n');
    }
    format!("{}{new}{}", &text[..start], &text[end..])
}

/// The `## Todo` entry titled `title` in `text`: its byte range, the
/// heading to the next entry or the section's end, and its text.
fn todo_entry(text: &str, title: &str) -> Option<((usize, usize), String)> {
    let (start, end) = section(text, "## Todo")?;
    let body = &text[start..end];
    let heading = format!("### {}", title.trim());
    let mut lines = prose_lines(body).skip_while(|(_, l)| l.trim_end() != heading);
    let (from, first) = lines.next()?;
    let to = lines
        .find(|(_, l)| l.starts_with("### "))
        .map_or(body.len(), |(at, _)| at);
    let entry = body[from + first.len()..to].trim().to_string();
    Some(((start + from, start + to), entry))
}

/// The `## In Progress` scaffold of the cycle `title`: the title, the
/// items, `problem` under Problem, and the opening and closing rung
/// subsections, in the specimen's shape.
fn scaffold(title: &str, problem: &str) -> String {
    let problem = if problem.is_empty() {
        TO_WRITE
    } else {
        problem
    };
    format!(
        "### {title}\n\n\
         #### Problem\n\n{problem}\n\n\
         #### Solution\n\n{TO_WRITE}\n\n\
         #### Acceptance check\n\n{TO_WRITE}\n\n\
         #### Deliberation\n\n_None._\n\n\
         #### Rungs\n\n\
         The rung subsections, in the ladder's order. The ladder itself is in `{LADDER_FILE} > \
         {LADDER}`.\n\n\
         ##### {title} opening\n\n\
         The cycle's setup commit: create the bookmark, delete `## Closed`'s contents, move the \
         Todo entry\ninto this block, and bump the version-of-record.\n\n\
         ##### {title} closing\n\n\
         Closing out the cycle.\n"
    )
}

/// `TODO.md`'s text with the cycle `title` opened, and what was done,
/// a line each: what `## Closed` holds is deleted, the `## Todo` entry
/// titled so is moved into an `## In Progress` scaffold, an empty one
/// when none matches. Refused when there is no `## In Progress` or a
/// cycle is in it.
pub(crate) fn opened_record(text: &str, title: &str) -> Result<(String, Vec<String>), String> {
    let title = title.trim();
    let entry = |l: &str| l.starts_with("### ");
    let content = |l: &str| entry(l) || l == NONE_IN_PROGRESS;
    let progress = section(text, "## In Progress")
        .ok_or_else(|| format!("{RECORD} has no `## In Progress` to open the cycle in"))?;
    if let Some((_, running)) = prose_lines(&text[progress.0..progress.1]).find(|(_, l)| entry(l)) {
        return Err(format!(
            "{RECORD} has a cycle in progress, '{}': it closes first",
            running.trim_start_matches("### ")
        ));
    }
    let mut did = Vec::new();
    let found = todo_entry(text, title);
    let problem = match &found {
        Some((_, entry)) => {
            did.push(format!(
                "move the `## Todo` entry '{title}' into `## In Progress`"
            ));
            entry.as_str()
        }
        None => {
            did.push(format!(
                "no `## Todo` entry is titled '{title}', so `## In Progress` gets an empty scaffold"
            ));
            ""
        }
    };
    // The edits, each a range and the text that replaces it, applied
    // last first, so an earlier one's offsets still hold.
    let mut edits: Vec<((usize, usize), String)> = Vec::new();
    let opened = with_content(text, progress, content, &scaffold(title, problem));
    edits.push((
        progress,
        opened[progress.0..opened.len() - (text.len() - progress.1)].to_string(),
    ));
    if let Some((range, _)) = found {
        edits.push((range, String::new()));
    }
    match section(text, "## Closed") {
        Some(closed) => {
            let emptied = with_content(text, closed, entry, "");
            let body = emptied[closed.0..emptied.len() - (text.len() - closed.1)].to_string();
            if body != text[closed.0..closed.1] {
                did.insert(0, "delete what `## Closed` holds".to_string());
            }
            edits.push((closed, body));
        }
        None => did.insert(0, "no `## Closed`, so nothing is deleted".to_string()),
    }
    edits.sort_by_key(|((start, _), _)| std::cmp::Reverse(*start));
    let mut new = text.to_string();
    for ((start, end), with) in edits {
        new.replace_range(start..end, &with);
    }
    Ok((new, did))
}

/// A ladder's line for a rung not yet started: the title linked to
/// its subsection in the record.
fn rung_link(title: &str) -> String {
    format!("[{title}]({RECORD}#{})", anchor_slug(title))
}

/// `zTOC.md`'s text with the ladder of a cycle just opened in place of
/// the last cycle's: the opening, current, and the closing. Refused
/// when there is no `## Ladder`.
pub(crate) fn opened_ladder(text: &str, title: &str) -> Result<String, String> {
    let title = title.trim();
    let rungs = [format!("{title} opening"), format!("{title} closing")];
    // A new cycle replaces the last one's ladder whatever its lines
    // were, so the grammar is not asked of them.
    let ladder = section(text, LADDER)
        .ok_or_else(|| format!("{LADDER_FILE} has no `{LADDER}` to write the cycle's ladder in"))?;
    let made = ladder_of(title, &rungs, &[])?;
    Ok(with_content(text, ladder, |_| true, &made.lines.join("\n")))
}

/// The title of the cycle in progress in `TODO.md`'s `text`, its
/// `### ` heading under `## In Progress`. Refused when there is none.
pub(crate) fn cycle_title(text: &str) -> Result<String, String> {
    let (start, end) = section(text, "## In Progress")
        .ok_or_else(|| format!("{RECORD} has no `## In Progress`"))?;
    prose_lines(&text[start..end])
        .find_map(|(_, l)| l.strip_prefix("### "))
        .map(|t| t.trim().to_string())
        .ok_or_else(|| format!("{RECORD} has no cycle in progress: `cycle --open` opens one"))
}

/// The title of the cycle `## Closed` holds in `TODO.md`'s `text`, the
/// last one closed, `None` when it holds none.
pub(crate) fn closed_title(text: &str) -> Option<String> {
    let (start, end) = section(text, "## Closed")?;
    prose_lines(&text[start..end])
        .find_map(|(_, l)| l.strip_prefix("### "))
        .map(|t| t.trim().to_string())
}

/// The rung titles of the cycle in progress, in their order: the
/// `##### ` headings under its `#### Rungs`, to the next heading of
/// that level or above. Refused when there is no `#### Rungs` or it
/// holds no rung.
pub(crate) fn rung_titles(text: &str) -> Result<Vec<String>, String> {
    let (start, end) = section(text, "## In Progress")
        .ok_or_else(|| format!("{RECORD} has no `## In Progress`"))?;
    let mut lines =
        prose_lines(&text[start..end]).skip_while(|(_, l)| l.trim_end() != "#### Rungs");
    if lines.next().is_none() {
        return Err(format!(
            "{RECORD}'s cycle has no `#### Rungs` heading to read its rungs from"
        ));
    }
    let above = |l: &str| {
        ["# ", "## ", "### ", "#### "]
            .iter()
            .any(|h| l.starts_with(h))
    };
    let rungs: Vec<String> = lines
        .take_while(|(_, l)| !above(l))
        .filter_map(|(_, l)| l.strip_prefix("##### "))
        .map(|t| t.trim().to_string())
        .collect();
    if rungs.is_empty() {
        return Err(format!(
            "{RECORD}'s `#### Rungs` holds no rung, a `##### <title>` heading each"
        ));
    }
    Ok(rungs)
}

/// A ladder made: its lines, and what a reader should know that the
/// lines do not refuse, a line each.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Ladder {
    pub lines: Vec<String>,
    pub findings: Vec<String>,
}

/// The ladder of the cycle `cycle`, a line per rung in `rungs`' order,
/// `commits` being the titles of the commits on its branch. A rung is
/// `(done)` when a commit has its title, `(current)` when it is the
/// first without one, each numbered by its position, and a rung not
/// started is its link alone. A single-step cycle, its one commit
/// under the bare cycle title, is one unlinked rung.
///
/// Refused: a title written twice, a ladder not opened by the cycle
/// title plus " opening" or not closed by it plus " closing", and a
/// commit whose title is no rung's, each naming what it found. A done
/// rung below the current one is a finding, the ladder still made.
pub(crate) fn ladder_of(
    cycle: &str,
    rungs: &[String],
    commits: &[String],
) -> Result<Ladder, String> {
    for (i, r) in rungs.iter().enumerate() {
        if rungs[..i].contains(r) {
            return Err(format!(
                "the rung '{r}' is written twice under `#### Rungs`"
            ));
        }
    }
    let (opening, closing) = (format!("{cycle} opening"), format!("{cycle} closing"));
    if rungs.first() != Some(&opening) {
        return Err(format!(
            "the first rung under `#### Rungs` is not the cycle's opening, '{opening}'"
        ));
    }
    if rungs.len() < 2 || rungs.last() != Some(&closing) {
        return Err(format!(
            "the last rung under `#### Rungs` is not the cycle's closing, '{closing}'"
        ));
    }
    let strays: Vec<&String> = commits
        .iter()
        .filter(|c| c.as_str() != cycle && !rungs.contains(c))
        .collect();
    if !strays.is_empty() {
        return Err(format!(
            "{} on the cycle's branch match no rung under `#### Rungs`, and every commit of a \
             cycle is a rung's: {}",
            if strays.len() == 1 {
                "a commit's title".to_string()
            } else {
                format!("{} commits' titles", strays.len())
            },
            strays
                .iter()
                .map(|s| format!("'{s}'"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if commits.iter().any(|c| c == cycle) {
        if commits.len() > 1 {
            return Err(format!(
                "the branch has a commit titled '{cycle}', a single-step cycle's one commit, \
                 beside {} more",
                commits.len() - 1
            ));
        }
        return Ok(Ladder {
            lines: vec![format!("- 0 (done) {cycle}")],
            findings: Vec::new(),
        });
    }
    let current = rungs.iter().position(|r| !commits.contains(r));
    let mut findings = Vec::new();
    let lines = rungs
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let link = rung_link(r);
            if commits.contains(r) {
                if let Some(c) = current
                    && c < i
                {
                    findings.push(format!(
                        "the rung '{r}' is done and is below the current one, '{}'",
                        rungs[c]
                    ));
                }
                format!("- {i} (done) {link}")
            } else if current == Some(i) {
                format!("- {i} (current) {link}")
            } else {
                format!("- {link}")
            }
        })
        .collect();
    Ok(Ladder { lines, findings })
}

/// Whether `line`, a left-margin bullet, is a rung by the ladder's
/// grammar: `- `, then optionally a number and `(current)` or `(done)`,
/// then the title, linked to its slug in the record or, for a
/// single-step cycle, bare.
fn is_rung(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("- ") else {
        return false;
    };
    let digits = rest.len() - rest.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    let rest = match digits {
        0 => rest,
        _ => {
            let after = &rest[digits..];
            match after
                .strip_prefix(" (current) ")
                .or_else(|| after.strip_prefix(" (done) "))
            {
                Some(title) => title,
                None => return false,
            }
        }
    };
    match rest.strip_prefix('[') {
        Some(linked) => linked
            .split_once(&format!("]({RECORD}#"))
            .and_then(|(title, slug)| Some((title, slug.strip_suffix(')')?)))
            .is_some_and(|(title, slug)| !title.is_empty() && slug == anchor_slug(title)),
        None => !rest.trim().is_empty() && rest == rest.trim() && !rest.contains("]("),
    }
}

/// `zTOC.md`'s text with `lines` as its ladder. Refused, with nothing
/// edited, when there is no `## Ladder`, or when a line of the ladder
/// there does not fit the grammar, each such line named: a rung is a
/// left-margin bullet, and anything else but a blank line and the
/// skeleton's `_None._` is a line the rewrite would lose.
pub(crate) fn written_ladder(text: &str, lines: &[String]) -> Result<String, String> {
    let ladder = section(text, LADDER)
        .ok_or_else(|| format!("{LADDER_FILE} has no `{LADDER}` to write the cycle's ladder in"))?;
    let misfits: Vec<String> = text[ladder.0..ladder.1]
        .lines()
        .filter(|l| !l.trim().is_empty() && l.trim() != "_None._" && !is_rung(l))
        .map(|l| format!("  {l}"))
        .collect();
    if !misfits.is_empty() {
        return Err(format!(
            "{LADDER_FILE}'s `{LADDER}` holds {} that a rung's grammar does not fit, `- [N \
             (current|done)] [<title>]({RECORD}#<slug>)`, so nothing is edited:\n{}",
            if misfits.len() == 1 {
                "a line".to_string()
            } else {
                format!("{} lines", misfits.len())
            },
            misfits.join("\n")
        ));
    }
    Ok(with_content(text, ladder, |_| true, &lines.join("\n")))
}

/// The cycle's ladder planned from the workspace at `root`: the cycle
/// title and the rung headings read from `TODO.md`, `commits` the
/// titles on the branch, and `zTOC.md`'s new text, `None` when the
/// ladder there is as made already.
#[derive(Debug)]
pub(crate) struct LadderPlan {
    pub ladder: Ladder,
    pub edit: Option<FileEdit>,
}

/// Plan the ladder, reading both files and writing neither.
pub(crate) fn plan_ladder(root: &Path, commits: &[String]) -> Result<LadderPlan, String> {
    let read = |name: &str| {
        let p = root.join(name);
        std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))
    };
    let todo = read(RECORD)?;
    let ladder = ladder_of(&cycle_title(&todo)?, &rung_titles(&todo)?, commits)?;
    let old = read(LADDER_FILE)?;
    let text = written_ladder(&old, &ladder.lines)?;
    let edit = (text != old).then(|| FileEdit {
        path: root.join(LADDER_FILE),
        text,
    });
    Ok(LadderPlan { ladder, edit })
}

/// One file's edit, planned: its path and its new text.
#[derive(Debug)]
pub(crate) struct FileEdit {
    pub path: PathBuf,
    pub text: String,
}

/// What `cycle --open` writes, planned before anything is made.
#[derive(Debug, Default)]
pub(crate) struct OpenRecord {
    /// The files written, the record then the ladder.
    pub edits: Vec<FileEdit>,
    /// What the edits do, and what was left out and why, a line each.
    pub did: Vec<String>,
}

/// Plan the record of the cycle `title` in the workspace at `root`,
/// reading and checking both files, so a refusal comes before the
/// bookmark is made. A workspace with no `TODO.md` gets none, said, and
/// one with no `zTOC.md` gets the record alone.
pub(crate) fn plan_open(root: &Path, title: &str) -> Result<OpenRecord, String> {
    let mut plan = OpenRecord::default();
    let record = root.join(RECORD);
    if !record.is_file() {
        plan.did
            .push(format!("no {RECORD}, so the bookmark is made alone"));
        return Ok(plan);
    }
    let read = |p: &Path| std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()));
    let (text, did) = opened_record(&read(&record)?, title)?;
    plan.did = did;
    plan.edits.push(FileEdit { path: record, text });
    let ladder = root.join(LADDER_FILE);
    if ladder.is_file() {
        let text = opened_ladder(&read(&ladder)?, title)?;
        plan.did.push(format!(
            "replace the ladder in {LADDER_FILE} with the opening and the closing"
        ));
        plan.edits.push(FileEdit { path: ladder, text });
    } else {
        plan.did
            .push(format!("no {LADDER_FILE}, so no ladder is written"));
    }
    Ok(plan)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A `TODO.md` between cycles: a last cycle in `## Closed`, two
    /// `## Todo` entries, and a fenced heading that is none.
    pub(crate) const TODO: &str = "\
# Todo list

## Continuation notes

_None._

## In Progress

The record's home.

_No cycle currently in progress._

## Todo

Entries in priority order.

### feat: a cycle

What is wrong, in a sentence.

```
### not a heading
```

### fix: another

Its text.

## Closed

The last cycle's record.

### chore: the last cycle

#### Problem

Old.

# References

[1]: notes/x.md
";

    /// A `zTOC.md` holding the last cycle's ladder.
    pub(crate) const ZTOC: &str = "\
# TOC

## Ladder

- 0 (done) chore: the last cycle

## The TOC

- [TODO.md](TODO.md): the todo list.
";

    /// The entry is moved, not copied, into the scaffold, `## Closed` is
    /// emptied with its intro kept, and the rest is as it was.
    #[test]
    fn open_moves_the_entry_and_empties_closed() {
        let (new, did) = opened_record(TODO, "feat: a cycle").expect("opened");
        assert_eq!(did.len(), 2, "{did:?}");
        assert!(did[0].contains("`## Closed`"), "{did:?}");
        let expect = "\
## In Progress

The record's home.

### feat: a cycle

#### Problem

What is wrong, in a sentence.

```
### not a heading
```

#### Solution

_To write._

#### Acceptance check

_To write._

#### Deliberation

_None._

#### Rungs

The rung subsections, in the ladder's order. The ladder itself is in `zTOC.md > ## Ladder`.

##### feat: a cycle opening

The cycle's setup commit: create the bookmark, delete `## Closed`'s contents, move the Todo entry
into this block, and bump the version-of-record.

##### feat: a cycle closing

Closing out the cycle.

## Todo

Entries in priority order.

### fix: another

Its text.

## Closed

The last cycle's record.

# References
";
        assert!(new.contains(expect), "{new}");
        assert!(
            new.starts_with("# Todo list\n\n## Continuation notes\n\n_None._\n\n## In Progress")
        );
        assert!(new.ends_with("# References\n\n[1]: notes/x.md\n"), "{new}");
        assert_eq!(
            new.matches("feat: a cycle\n").count(),
            1,
            "moved, not copied"
        );
    }

    /// No entry of the title is an empty scaffold, said, and the
    /// `## Todo` entries stay.
    #[test]
    fn open_without_an_entry_writes_an_empty_scaffold() {
        let (new, did) = opened_record(TODO, "feat: unplanned").expect("opened");
        assert!(did.iter().any(|d| d.contains("empty scaffold")), "{did:?}");
        assert!(
            new.contains("### feat: unplanned\n\n#### Problem\n\n_To write._\n"),
            "{new}"
        );
        assert!(new.contains("### feat: a cycle\n\nWhat is wrong"), "{new}");
    }

    /// A cycle in progress, and a record with no `## In Progress`, are
    /// refused, the first naming the cycle.
    #[test]
    fn open_refuses_a_cycle_in_progress() {
        let (running, _) = opened_record(TODO, "feat: a cycle").expect("opened");
        let err = opened_record(&running, "fix: another").unwrap_err();
        assert!(
            err.contains("in progress") && err.contains("feat: a cycle"),
            "{err}"
        );
        let err = opened_record("# Todo list\n\n## Todo\n", "feat: x").unwrap_err();
        assert!(err.contains("no `## In Progress`"), "{err}");
    }

    /// A record whose sections are the file's last and hold nothing
    /// keeps its one trailing newline.
    #[test]
    fn open_at_the_files_end() {
        let text = "## In Progress\n\n_No cycle currently in progress._\n";
        let (new, did) = opened_record(text, "feat: x").expect("opened");
        assert!(new.ends_with("Closing out the cycle.\n"), "{new}");
        assert!(!new.contains(NONE_IN_PROGRESS), "{new}");
        assert!(did.iter().any(|d| d.contains("no `## Closed`")), "{did:?}");
    }

    /// The ladder is replaced with the opening, current, and the
    /// closing, each linked by its title's slug, and `## The TOC` stays.
    #[test]
    fn open_replaces_the_ladder() {
        let new = opened_ladder(ZTOC, "feat: a cycle").expect("ladder");
        assert_eq!(
            new,
            "\
# TOC

## Ladder

- 0 (current) [feat: a cycle opening](TODO.md#feat-a-cycle-opening)
- [feat: a cycle closing](TODO.md#feat-a-cycle-closing)

## The TOC

- [TODO.md](TODO.md): the todo list.
"
        );
        let err = opened_ladder("# TOC\n", "feat: x").unwrap_err();
        assert!(err.contains("no `## Ladder`"), "{err}");
    }

    /// The cycle title and the rungs are read from the record, the
    /// rungs' own subheadings and a later item's headings left out.
    #[test]
    fn the_rungs_are_the_headings_under_rungs() {
        let (opened, _) = opened_record(TODO, "feat: a cycle").expect("opened");
        assert_eq!(cycle_title(&opened).unwrap(), "feat: a cycle");
        assert_eq!(
            rung_titles(&opened).unwrap(),
            ["feat: a cycle opening", "feat: a cycle closing"]
        );
        let more = opened.replace(
            "##### feat: a cycle closing",
            "##### feat: a rung\n\n###### a detail\n\n```\n##### fenced\n```\n\n\
             ##### feat: a cycle closing",
        );
        assert_eq!(
            rung_titles(&more).unwrap(),
            [
                "feat: a cycle opening",
                "feat: a rung",
                "feat: a cycle closing"
            ]
        );
        let err = cycle_title(TODO).unwrap_err();
        assert!(err.contains("no cycle in progress"), "{err}");
        let err = rung_titles(&opened.replace("#### Rungs", "#### Steps")).unwrap_err();
        assert!(err.contains("no `#### Rungs`"), "{err}");
    }

    fn rungs() -> Vec<String> {
        [
            "feat: c opening",
            "feat: one",
            "feat: two",
            "feat: c closing",
        ]
        .map(String::from)
        .to_vec()
    }

    fn titles(t: &[&str]) -> Vec<String> {
        t.iter().map(|s| s.to_string()).collect()
    }

    /// A rung with a commit is done, the first without one is current,
    /// each numbered by its position, and the rest are links alone.
    #[test]
    fn the_markers_are_computed_from_the_commits() {
        let made = ladder_of(
            "feat: c",
            &rungs(),
            &titles(&["feat: c opening", "feat: one"]),
        )
        .expect("made");
        assert_eq!(
            made.lines,
            [
                "- 0 (done) [feat: c opening](TODO.md#feat-c-opening)",
                "- 1 (done) [feat: one](TODO.md#feat-one)",
                "- 2 (current) [feat: two](TODO.md#feat-two)",
                "- [feat: c closing](TODO.md#feat-c-closing)",
            ]
        );
        assert!(made.findings.is_empty());

        let all = ladder_of("feat: c", &rungs(), &rungs()).expect("made");
        assert!(all.lines.iter().all(|l| l.contains("(done)")), "{all:?}");

        let made = ladder_of(
            "feat: c",
            &rungs(),
            &titles(&["feat: c opening", "feat: two"]),
        )
        .expect("made");
        assert_eq!(made.lines[1], "- 1 (current) [feat: one](TODO.md#feat-one)");
        assert_eq!(made.lines[2], "- 2 (done) [feat: two](TODO.md#feat-two)");
        assert_eq!(made.findings.len(), 1, "{:?}", made.findings);
    }

    /// A single-step cycle's one commit, under the bare title, is one
    /// unlinked rung.
    #[test]
    fn a_single_step_cycle_is_one_unlinked_rung() {
        let two = titles(&["feat: c opening", "feat: c closing"]);
        let made = ladder_of("feat: c", &two, &titles(&["feat: c"])).expect("made");
        assert_eq!(made.lines, ["- 0 (done) feat: c"]);
        let err = ladder_of("feat: c", &two, &titles(&["feat: c", "feat: c opening"])).unwrap_err();
        assert!(err.contains("single-step"), "{err}");
    }

    /// A commit that is no rung's, a rung written twice, and a ladder
    /// without its bookends are refused, each named.
    #[test]
    fn a_ladder_that_does_not_fit_the_record_is_refused() {
        let err = ladder_of("feat: c", &rungs(), &titles(&["feat: c opening", "wip"])).unwrap_err();
        assert!(err.contains("'wip'") && err.contains("no rung"), "{err}");
        let mut twice = rungs();
        twice.insert(2, "feat: one".into());
        let err = ladder_of("feat: c", &twice, &[]).unwrap_err();
        assert!(err.contains("written twice"), "{err}");
        let err = ladder_of("feat: c", &rungs()[1..], &[]).unwrap_err();
        assert!(err.contains("opening"), "{err}");
        let err = ladder_of("feat: c", &rungs()[..3], &[]).unwrap_err();
        assert!(err.contains("closing"), "{err}");
    }

    /// The grammar: a rung is a left-margin bullet, numbered and
    /// marked or neither, its link's slug the title's.
    #[test]
    fn the_ladders_grammar() {
        for fits in [
            "- 0 (done) [feat: c opening](TODO.md#feat-c-opening)",
            "- 12 (current) [feat: one](TODO.md#feat-one)",
            "- [feat: c closing](TODO.md#feat-c-closing)",
            "- 0 (done) feat: c",
            "- feat: c",
        ] {
            assert!(is_rung(fits), "{fits}");
        }
        for misfit in [
            "* [feat: one](TODO.md#feat-one)",
            "- 1 [feat: one](TODO.md#feat-one)",
            "- 1 (started) [feat: one](TODO.md#feat-one)",
            "- (done) [feat: one](TODO.md#feat-one)",
            "- [feat: one](TODO.md#feat-two)",
            "- [feat: one](#feat-one)",
            "  - [feat: one](TODO.md#feat-one)",
            "- ",
        ] {
            assert!(!is_rung(misfit), "{misfit}");
        }
    }

    /// A ladder is written over one whose lines fit, and a line that
    /// does not is named with nothing edited.
    #[test]
    fn a_ladder_is_written_over_one_that_fits() {
        let lines = titles(&["- 0 (current) [feat: c opening](TODO.md#feat-c-opening)"]);
        let new = written_ladder(ZTOC, &lines).expect("written");
        assert!(
            new.contains("## Ladder\n\n- 0 (current) [feat: c opening]"),
            "{new}"
        );
        assert!(new.contains("\n\n## The TOC\n"), "{new}");
        let skeleton = ZTOC.replace("- 0 (done) chore: the last cycle", "_None._");
        written_ladder(&skeleton, &lines).expect("the skeleton fits");

        let odd = ZTOC.replace("- 0 (done) chore", "A note.\n- 0 done chore");
        let err = written_ladder(&odd, &lines).unwrap_err();
        assert!(
            err.contains("2 lines") && err.contains("A note.") && err.contains("nothing is edited"),
            "{err}"
        );
    }
}

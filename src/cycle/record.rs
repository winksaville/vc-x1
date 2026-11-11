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
    let ladder = section(text, LADDER)
        .ok_or_else(|| format!("{LADDER_FILE} has no `{LADDER}` to write the cycle's ladder in"))?;
    let lines = format!(
        "- 0 (current) {}\n- {}",
        rung_link(&format!("{title} opening")),
        rung_link(&format!("{title} closing"))
    );
    Ok(with_content(text, ladder, |_| true, &lines))
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
}

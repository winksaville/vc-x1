//! The cycle-record's mechanical edits, as text: what `cycle plan`
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

use crate::check::anchors::anchor_slug;
use crate::commit::{NONE_IN_PROGRESS, RECORD, section};

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
         The cycle's first commit, carrying its plan: the bookmark, the last cycle's record \
         deleted, and the\nTodo entry moved into this block.\n\n\
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

/// The items of the cycle in progress in `TODO.md`'s `text` still
/// reading the scaffold's placeholder: Problem, Solution, and
/// Acceptance check, in that order.
pub(crate) fn unwritten_items(text: &str) -> Vec<&'static str> {
    let Some((start, end)) = section(text, "## In Progress") else {
        return Vec::new();
    };
    let body = &text[start..end];
    let lines: Vec<(usize, &str)> = prose_lines(body).collect();
    ["Problem", "Solution", "Acceptance check"]
        .into_iter()
        .filter(|item| {
            let heading = format!("#### {item}");
            let Some(i) = lines.iter().position(|(_, l)| l.trim_end() == heading) else {
                return false;
            };
            let text: Vec<&str> = lines[i + 1..]
                .iter()
                .take_while(|(_, l)| !l.starts_with('#'))
                .map(|(_, l)| l.trim())
                .filter(|l| !l.is_empty())
                .collect();
            text == [TO_WRITE]
        })
        .collect()
}

/// `TODO.md`'s text with the `#### Rungs` item of the cycle `## Closed`
/// holds removed, its heading to the next heading of its level or
/// above, for a single-step cycle, whose one commit has no rungs.
pub(crate) fn without_closed_rungs(text: &str) -> String {
    let Some((start, end)) = section(text, "## Closed") else {
        return text.to_string();
    };
    let body = &text[start..end];
    let mut lines = prose_lines(body).skip_while(|(_, l)| l.trim_end() != "#### Rungs");
    let Some((from, _)) = lines.next() else {
        return text.to_string();
    };
    let above = |l: &str| {
        ["# ", "## ", "### ", "#### "]
            .iter()
            .any(|h| l.starts_with(h))
    };
    let to = lines
        .find(|(_, l)| above(l))
        .map_or(body.len(), |(at, _)| at);
    let kept = format!("{}{}", body[..from].trim_end(), "\n\n");
    let tail = &body[to..];
    let new_body = if tail.is_empty() && end == text.len() {
        format!("{}\n", kept.trim_end())
    } else if tail.is_empty() {
        kept
    } else {
        format!("{kept}{tail}")
    };
    format!("{}{new_body}{}", &text[..start], &text[end..])
}

/// A ladder's line for a rung not yet started: the title linked to
/// its subsection in the record.
fn rung_link(title: &str) -> String {
    format!("[{title}]({RECORD}#{})", anchor_slug(title))
}

/// `zTOC.md`'s text with the ladder of a cycle just opened in place of
/// the last cycle's: the opening and the closing, neither marked. Refused
/// when there is no `## Ladder`.
pub(crate) fn opened_ladder(text: &str, title: &str) -> Result<String, String> {
    let title = title.trim();
    let rungs = [format!("{title} opening"), format!("{title} closing")];
    // A new cycle replaces the last one's ladder whatever its lines
    // were, so the grammar is not asked of them.
    let ladder = section(text, LADDER)
        .ok_or_else(|| format!("{LADDER_FILE} has no `{LADDER}` to write the cycle's ladder in"))?;
    let made = ladder_of(title, &rungs, &[], None)?;
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
        .ok_or_else(|| format!("{RECORD} has no cycle in progress: `cycle plan start` opens one"))
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
/// `commits` being the titles of the commits on its branch and
/// `current` the rung started and not yet committed. A rung is
/// `(done)`, numbered by its position, when a commit has its title, the
/// current rung `(current)`, numbered too, and every other rung is its
/// link alone. The marker `(current)` is the working copy's, written by
/// `rung next` and never by a commit, so no commit's tree claims work
/// not yet begun. A single-step cycle, its one commit under the bare
/// cycle title, is one unlinked rung, and so is a plan of one heading,
/// the cycle title itself, the simple single-step, its heading's
/// anchor being the cycle's own.
///
/// Refused: a title written twice, a ladder of more than the one
/// heading not opened by the cycle title plus " opening" or not closed
/// by it plus " closing", and a
/// commit whose title is no rung's, each naming what it found. A done
/// rung below the first rung not done is a finding, the ladder still
/// made, and so is a `current` that is done or no rung, its marker
/// dropped.
pub(crate) fn ladder_of(
    cycle: &str,
    rungs: &[String],
    commits: &[String],
    current: Option<&str>,
) -> Result<Ladder, String> {
    for (i, r) in rungs.iter().enumerate() {
        if rungs[..i].contains(r) {
            return Err(format!(
                "the rung '{r}' is written twice under `#### Rungs`"
            ));
        }
    }
    let (opening, closing) = (format!("{cycle} opening"), format!("{cycle} closing"));
    let one = is_one_rung(cycle, rungs);
    if !one && rungs.first() != Some(&opening) {
        return Err(format!(
            "the first rung under `#### Rungs` is not the cycle's opening, '{opening}', nor is \
             it the cycle title alone, a single-step's one rung"
        ));
    }
    if !one && (rungs.len() < 2 || rungs.last() != Some(&closing)) {
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
    let next = rungs.iter().position(|r| !commits.contains(r));
    let mut findings = Vec::new();
    let current = current.filter(|c| {
        let started = rungs.iter().any(|r| r == c) && !commits.iter().any(|r| r == c);
        if !started {
            findings.push(format!(
                "the rung marked (current), '{c}', is {}, so the marker is dropped",
                if rungs.iter().any(|r| r == c) {
                    "done"
                } else {
                    "no rung under `#### Rungs`"
                }
            ));
        }
        started
    });
    let lines = rungs
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let link = if one { r.clone() } else { rung_link(r) };
            if commits.contains(r) {
                if let Some(n) = next
                    && n < i
                {
                    findings.push(format!(
                        "the rung '{r}' is done and is below '{}', a rung not done",
                        rungs[n]
                    ));
                }
                format!("- {i} (done) {link}")
            } else if current == Some(r.as_str()) {
                format!("- {i} (current) {link}")
            } else {
                format!("- {link}")
            }
        })
        .collect();
    Ok(Ladder { lines, findings })
}

/// Whether `rungs` is a simple single-step's plan, the one heading, the
/// cycle title itself.
pub(crate) fn is_one_rung(cycle: &str, rungs: &[String]) -> bool {
    matches!(rungs, [only] if only == cycle)
}

/// `TODO.md`'s text with a plan of only its bookends, the cycle title
/// plus " opening" and " closing", made the simple single-step's one
/// heading, the cycle title: the opening's heading renamed, its text
/// kept, and the closing's subsection removed to the next heading of
/// its level or above. Refused when the plan is not the bookends alone.
pub(crate) fn one_rung_record(text: &str, cycle: &str) -> Result<String, String> {
    let rungs = rung_titles(text)?;
    let (opening, closing) = (format!("{cycle} opening"), format!("{cycle} closing"));
    if rungs != [opening.clone(), closing.clone()] {
        return Err(format!(
            "the plan under `#### Rungs` is not the bookends alone, '{opening}' and '{closing}'"
        ));
    }
    let (start, end) = section(text, "## In Progress")
        .ok_or_else(|| format!("{RECORD} has no `## In Progress`"))?;
    let body = &text[start..end];
    let heading = |l: &str| {
        ["# ", "## ", "### ", "#### ", "##### "]
            .iter()
            .any(|h| l.starts_with(h))
    };
    let named = |l: &str, t: &str| l.strip_prefix("##### ").map(str::trim) == Some(t);
    let lines: Vec<(usize, &str)> = prose_lines(body)
        .skip_while(|(_, l)| l.trim_end() != "#### Rungs")
        .collect();
    let open = lines.iter().find(|(_, l)| named(l, &opening));
    let close = lines.iter().position(|(_, l)| named(l, &closing));
    let (Some(&(open_at, open_line)), Some(close)) = (open, close) else {
        return Err(format!(
            "{RECORD}'s bookend headings were not found to rewrite"
        ));
    };
    let close_from = lines[close].0;
    let close_to = lines[close + 1..]
        .iter()
        .find(|(_, l)| heading(l))
        .map_or(body.len(), |(at, _)| *at);
    let mut new = body.to_string();
    // The later span first, so the earlier offsets stand.
    new.replace_range(close_from..close_to, "");
    new.replace_range(
        open_at..open_at + open_line.len(),
        &format!("##### {cycle}"),
    );
    Ok(format!("{}{new}{}", &text[..start], &text[end..]))
}

/// `TODO.md`'s text with the cycle in progress's Problem and Solution
/// written from a commit's `body` where they still read the
/// scaffold's placeholder: the paragraphs before the one opening
/// "Solution:" its problem, and that one on its solution, a pointer
/// paragraph, `See details in ...`, left out. The Acceptance check
/// reads `_None._`, the commit being its own.
pub(crate) fn with_body_items(text: &str, body: &str) -> String {
    let paragraphs: Vec<&str> = body
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty() && !p.starts_with("See details in "))
        .collect();
    let at = paragraphs.iter().position(|p| p.starts_with("Solution:"));
    let (problem, solution) = match at {
        Some(i) => (
            paragraphs[..i].join("\n\n"),
            paragraphs[i..]
                .join("\n\n")
                .trim_start_matches("Solution:")
                .trim()
                .to_string(),
        ),
        None => (paragraphs.join("\n\n"), String::new()),
    };
    let Some((start, end)) = section(text, "## In Progress") else {
        return text.to_string();
    };
    let mut block = text[start..end].to_string();
    for (item, with) in [
        ("Problem", problem.as_str()),
        ("Solution", solution.as_str()),
        ("Acceptance check", "_None._"),
    ] {
        let placeholder = format!("#### {item}\n\n{TO_WRITE}\n");
        if !with.is_empty() {
            block = block.replacen(&placeholder, &format!("#### {item}\n\n{with}\n"), 1);
        }
    }
    format!("{}{block}{}", &text[..start], &text[end..])
}

/// A ladder line with its link reduced to the title it shows,
/// `- 1 (done) [feat: x](TODO.md#feat-x)` to `- 1 (done) feat: x`,
/// for a terminal, where the link cannot be followed.
pub(crate) fn unlinked(line: &str) -> String {
    let Some(open) = line.find('[') else {
        return line.to_string();
    };
    let Some(close) = line[open..].find("](").map(|i| open + i) else {
        return line.to_string();
    };
    if !line.ends_with(')') {
        return line.to_string();
    }
    format!("{}{}", &line[..open], &line[open + 1..close])
}

/// A ladder line's parts by the ladder's grammar: its number and
/// marker, `(done)` or `(current)`, when it has them, and its title,
/// `None` when the line is no rung.
fn rung_parts(line: &str) -> Option<(Option<&str>, &str)> {
    let rest = line.strip_prefix("- ")?;
    let digits = rest.len() - rest.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    let (marker, rest) = match digits {
        0 => (None, rest),
        _ => {
            let after = &rest[digits..];
            ["current", "done"].into_iter().find_map(|m| {
                after
                    .strip_prefix(&format!(" ({m}) "))
                    .map(|title| (Some(m), title))
            })?
        }
    };
    match rest.strip_prefix('[') {
        Some(linked) => linked
            .split_once(&format!("]({RECORD}#"))
            .and_then(|(title, slug)| Some((title, slug.strip_suffix(')')?)))
            .filter(|(title, slug)| !title.is_empty() && *slug == anchor_slug(title))
            .map(|(title, _)| (marker, title)),
        None => (!rest.trim().is_empty() && rest == rest.trim() && !rest.contains("]("))
            .then_some((marker, rest)),
    }
}

/// The rung marked `(current)` in `zTOC.md`'s `text`, the one started
/// and not yet committed, `None` when no line of the ladder is.
pub(crate) fn current_of(text: &str) -> Option<String> {
    let (start, end) = section(text, LADDER)?;
    text[start..end]
        .lines()
        .filter_map(rung_parts)
        .find_map(|(marker, title)| (marker == Some("current")).then(|| title.to_string()))
}

/// Whether the ladder in `zTOC.md`'s `text` has a rung marked `(done)`.
pub(crate) fn has_done(text: &str) -> bool {
    section(text, LADDER).is_some_and(|(start, end)| {
        text[start..end]
            .lines()
            .filter_map(rung_parts)
            .any(|(marker, _)| marker == Some("done"))
    })
}

/// The rung marked `(current)` in the workspace at `root`, read from
/// its `zTOC.md`.
pub(crate) fn current_rung(root: &Path) -> Result<Option<String>, String> {
    let p = root.join(LADDER_FILE);
    let text = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    Ok(current_of(&text))
}

/// Whether `line`, a left-margin bullet, is a rung by the ladder's
/// grammar: `- `, then optionally a number and `(done)` or
/// `(current)`, then the title, linked to its slug in the record or,
/// for a single-step cycle, bare.
fn is_rung(line: &str) -> bool {
    rung_parts(line).is_some()
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
             (done|current)] [<title>]({RECORD}#<slug>)`, so nothing is edited:\n{}",
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
/// titles on the branch, `current` the rung marked `(current)`, and
/// `zTOC.md`'s new text, `None` when the ladder there is as made
/// already.
#[derive(Debug)]
pub(crate) struct LadderPlan {
    pub ladder: Ladder,
    pub edit: Option<FileEdit>,
}

/// Plan the ladder, reading both files and writing neither.
pub(crate) fn plan_ladder(
    root: &Path,
    commits: &[String],
    current: Option<&str>,
) -> Result<LadderPlan, String> {
    let read = |name: &str| {
        let p = root.join(name);
        std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))
    };
    let todo = read(RECORD)?;
    let ladder = ladder_of(&cycle_title(&todo)?, &rung_titles(&todo)?, commits, current)?;
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

/// What `cycle plan` writes, planned before anything is made.
#[derive(Debug, Default)]
pub(crate) struct OpenRecord {
    /// The files written, the record then the ladder.
    pub edits: Vec<FileEdit>,
    /// What the edits do, and what was left out and why, a line each.
    pub did: Vec<String>,
}

/// The `TODO.md` `vc-x1 init` writes, and `cycle plan` when there is
/// none: the headings the
/// record needs, a cycle's home and the one it moves to, and the
/// sections the agent-files read first and rank by, `## Todo` and `##
/// Closed` empty, so no placeholder is left when a cycle fills them.
pub(crate) const TODO_SKELETON: &str = "\
# Todo list

## Continuation notes

_None._

## In Progress

_No cycle currently in progress._

## Todo

## Closed
";

/// The `zTOC.md` `vc-x1 init` writes, and `cycle plan` when there is
/// none: the ladder's heading.
pub(crate) const LADDER_SKELETON: &str = "\
# TOC

## Ladder

_None._
";

/// Plan the record of the cycle `title` in the workspace at `root`,
/// reading and checking both files, so a refusal comes before the
/// bookmark is made. A file missing, in a workspace made before `vc-x1
/// init` wrote them, is planned from its skeleton, [`TODO_SKELETON`] or
/// [`LADDER_SKELETON`], as `init` writes it.
pub(crate) fn plan_open(root: &Path, title: &str) -> Result<OpenRecord, String> {
    let mut plan = OpenRecord::default();
    let record = root.join(RECORD);
    let ladder = root.join(LADDER_FILE);
    let read = |name: &str, p: &Path, skeleton: &str, did: &mut Vec<String>| {
        if p.is_file() {
            return std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()));
        }
        did.push(format!(
            "write {name}, a skeleton, since {} has none",
            root.display()
        ));
        Ok(skeleton.to_string())
    };
    let mut made = Vec::new();
    let todo = read(RECORD, &record, TODO_SKELETON, &mut made)?;
    let toc = read(LADDER_FILE, &ladder, LADDER_SKELETON, &mut made)?;
    let (text, did) = opened_record(&todo, title)?;
    plan.did = made;
    plan.did.extend(did);
    plan.edits.push(FileEdit { path: record, text });
    let text = opened_ladder(&toc, title)?;
    plan.did.push(format!(
        "replace the ladder in {LADDER_FILE} with the opening and the closing"
    ));
    plan.edits.push(FileEdit { path: ladder, text });
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

The cycle's first commit, carrying its plan: the bookmark, the last cycle's record deleted, and the
Todo entry moved into this block.

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

    /// The ladder is replaced with the opening and the closing, neither
    /// marked, each linked by its title's slug, and `## The TOC` stays.
    #[test]
    fn open_replaces_the_ladder() {
        let new = opened_ladder(ZTOC, "feat: a cycle").expect("ladder");
        assert_eq!(
            new,
            "\
# TOC

## Ladder

- [feat: a cycle opening](TODO.md#feat-a-cycle-opening)
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

    /// A rung with a commit is done, numbered by its position, and the
    /// rest are links alone, none marked current.
    #[test]
    fn the_markers_are_computed_from_the_commits() {
        let made = ladder_of(
            "feat: c",
            &rungs(),
            &titles(&["feat: c opening", "feat: one"]),
            None,
        )
        .expect("made");
        assert_eq!(
            made.lines,
            [
                "- 0 (done) [feat: c opening](TODO.md#feat-c-opening)",
                "- 1 (done) [feat: one](TODO.md#feat-one)",
                "- [feat: two](TODO.md#feat-two)",
                "- [feat: c closing](TODO.md#feat-c-closing)",
            ]
        );
        assert!(made.findings.is_empty());

        let all = ladder_of("feat: c", &rungs(), &rungs(), None).expect("made");
        assert!(all.lines.iter().all(|l| l.contains("(done)")), "{all:?}");

        let made = ladder_of(
            "feat: c",
            &rungs(),
            &titles(&["feat: c opening", "feat: two"]),
            None,
        )
        .expect("made");
        assert_eq!(made.lines[1], "- [feat: one](TODO.md#feat-one)");
        assert_eq!(made.lines[2], "- 2 (done) [feat: two](TODO.md#feat-two)");
        assert_eq!(
            made.findings,
            ["the rung 'feat: two' is done and is below 'feat: one', a rung not done"]
        );
        let none = ladder_of("feat: c", &rungs(), &[], None).expect("made");
        assert!(
            none.lines.iter().all(|l| l.starts_with("- [")),
            "{:?}",
            none.lines
        );
    }

    /// A single-step cycle's one commit, under the bare title, is one
    /// unlinked rung.
    #[test]
    fn a_single_step_cycle_is_one_unlinked_rung() {
        let two = titles(&["feat: c opening", "feat: c closing"]);
        let made = ladder_of("feat: c", &two, &titles(&["feat: c"]), None).expect("made");
        assert_eq!(made.lines, ["- 0 (done) feat: c"]);
        let err = ladder_of(
            "feat: c",
            &two,
            &titles(&["feat: c", "feat: c opening"]),
            None,
        )
        .unwrap_err();
        assert!(err.contains("single-step"), "{err}");
    }

    /// A commit that is no rung's, a rung written twice, and a ladder
    /// without its bookends are refused, each named.
    #[test]
    fn a_ladder_that_does_not_fit_the_record_is_refused() {
        let err = ladder_of(
            "feat: c",
            &rungs(),
            &titles(&["feat: c opening", "wip"]),
            None,
        )
        .unwrap_err();
        assert!(err.contains("'wip'") && err.contains("no rung"), "{err}");
        let mut twice = rungs();
        twice.insert(2, "feat: one".into());
        let err = ladder_of("feat: c", &twice, &[], None).unwrap_err();
        assert!(err.contains("written twice"), "{err}");
        let err = ladder_of("feat: c", &rungs()[1..], &[], None).unwrap_err();
        assert!(err.contains("opening"), "{err}");
        let err = ladder_of("feat: c", &rungs()[..3], &[], None).unwrap_err();
        assert!(err.contains("closing"), "{err}");
    }

    /// The rung started is marked `(current)`, numbered, and one done
    /// or no rung keeps no marker, said as a finding.
    /// A plan of one heading, the cycle title, is the simple
    /// single-step's one rung, unlinked, started and then done.
    #[test]
    fn one_heading_is_a_single_steps_one_rung() {
        let one = titles(&["feat: c"]);
        let plain = ladder_of("feat: c", &one, &[], None).expect("made");
        assert_eq!(plain.lines, ["- feat: c"]);
        let started = ladder_of("feat: c", &one, &[], Some("feat: c")).expect("made");
        assert_eq!(started.lines, ["- 0 (current) feat: c"]);
        let done = ladder_of("feat: c", &one, &one, None).expect("made");
        assert_eq!(done.lines, ["- 0 (done) feat: c"]);
        let err = ladder_of("feat: c", &titles(&["feat: other"]), &[], None).unwrap_err();
        assert!(err.contains("the cycle title alone"), "{err}");
        assert_eq!(
            current_of("## Ladder\n\n- 0 (current) feat: c\n"),
            Some("feat: c".into())
        );
    }

    /// A plan of only its bookends becomes the one heading, the
    /// opening's text kept and the closing's subsection gone, and a plan
    /// with work rungs is refused.
    #[test]
    fn the_bookends_become_the_one_heading() {
        let record = "\
## In Progress

### feat: c

#### Rungs

##### feat: c opening

The plan.

##### feat: c closing

Closing out the cycle.

## Todo
";
        let one = one_rung_record(record, "feat: c").expect("rewritten");
        assert_eq!(
            one,
            "\
## In Progress

### feat: c

#### Rungs

##### feat: c

The plan.

## Todo
"
        );
        assert_eq!(rung_titles(&one).expect("rungs"), ["feat: c"]);
        let laddered = record.replace(
            "##### feat: c closing",
            "##### feat: work\n\nIt.\n\n##### feat: c closing",
        );
        let err = one_rung_record(&laddered, "feat: c").unwrap_err();
        assert!(err.contains("bookends alone"), "{err}");
    }

    #[test]
    fn the_current_rung_is_marked_until_it_is_done() {
        let opened = titles(&["feat: c opening"]);
        let made = ladder_of("feat: c", &rungs(), &opened, Some("feat: one")).expect("made");
        assert_eq!(
            made.lines[..3],
            [
                "- 0 (done) [feat: c opening](TODO.md#feat-c-opening)",
                "- 1 (current) [feat: one](TODO.md#feat-one)",
                "- [feat: two](TODO.md#feat-two)",
            ]
        );
        assert!(made.findings.is_empty(), "{:?}", made.findings);

        let none = ladder_of("feat: c", &rungs(), &[], Some("feat: c opening")).expect("made");
        assert_eq!(
            none.lines[0],
            "- 0 (current) [feat: c opening](TODO.md#feat-c-opening)"
        );

        let done = ladder_of("feat: c", &rungs(), &opened, Some("feat: c opening")).expect("made");
        assert!(
            !done.lines.iter().any(|l| l.contains("(current)")),
            "{done:?}"
        );
        assert_eq!(
            done.findings,
            ["the rung marked (current), 'feat: c opening', is done, so the marker is dropped"]
        );
        let gone = ladder_of("feat: c", &rungs(), &opened, Some("feat: renamed")).expect("made");
        assert!(gone.findings[0].contains("no rung"), "{gone:?}");
    }

    /// The current rung is read back from the ladder, linked or bare,
    /// and a ladder with none has none.
    #[test]
    fn the_current_rung_is_read_from_the_ladder() {
        let marked = ZTOC.replace(
            "- 0 (done) chore: the last cycle",
            "- 0 (done) [feat: c opening](TODO.md#feat-c-opening)\n\
             - 1 (current) [feat: one](TODO.md#feat-one)",
        );
        assert_eq!(current_of(&marked).as_deref(), Some("feat: one"));
        assert_eq!(current_of(ZTOC), None);
        let bare = ZTOC.replace("- 0 (done) chore", "- 0 (current) chore");
        assert_eq!(current_of(&bare).as_deref(), Some("chore: the last cycle"));
    }

    /// A link is reduced to its title, and an unlinked line is as it was.
    #[test]
    fn a_ladder_line_unlinked() {
        assert_eq!(
            unlinked("- 1 (done) [feat: one](TODO.md#feat-one)"),
            "- 1 (done) feat: one"
        );
        assert_eq!(unlinked("- [feat: two](TODO.md#feat-two)"), "- feat: two");
        assert_eq!(unlinked("- 0 (done) feat: x"), "- 0 (done) feat: x");
    }

    /// The grammar: a rung is a left-margin bullet, numbered and
    /// marked `(done)` or `(current)` or neither, its link's slug the
    /// title's.
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

    /// The items still reading the scaffold's placeholder are named, and
    /// written ones are not.
    #[test]
    fn the_unwritten_items_are_named() {
        let (opened, _) = opened_record(TODO, "feat: a cycle").expect("opened");
        assert_eq!(unwritten_items(&opened), ["Solution", "Acceptance check"]);
        let (empty, _) = opened_record(TODO, "feat: unplanned").expect("opened");
        assert_eq!(
            unwritten_items(&empty),
            ["Problem", "Solution", "Acceptance check"]
        );
        let written = empty.replace("#### Solution\n\n_To write._", "#### Solution\n\nDo it.");
        assert_eq!(unwritten_items(&written), ["Problem", "Acceptance check"]);
        assert!(unwritten_items(TODO).is_empty(), "no cycle in progress");
    }

    /// A single-step cycle's closed block loses its `#### Rungs`, and
    /// the rest of the file is as it was.
    #[test]
    fn a_single_step_closed_block_has_no_rungs() {
        let moved = "\
## In Progress

_No cycle currently in progress._

## Closed

The last cycle's record.

### feat: a cycle

#### Deliberation

_None._

#### Rungs

The rung subsections.

##### feat: a cycle opening

Its intent.

##### feat: a cycle closing

Closing out the cycle.

# References

[1]: x.md
";
        let stripped = without_closed_rungs(moved);
        assert_eq!(
            stripped,
            "\
## In Progress

_No cycle currently in progress._

## Closed

The last cycle's record.

### feat: a cycle

#### Deliberation

_None._

# References

[1]: x.md
"
        );
        assert_eq!(without_closed_rungs(TODO), TODO, "nothing to strip");
        let at_end = moved.replace("# References\n\n[1]: x.md\n", "");
        let stripped = without_closed_rungs(&at_end);
        assert!(
            stripped.ends_with("#### Deliberation\n\n_None._\n"),
            "{stripped}"
        );
    }

    /// A ladder is written over one whose lines fit, a `(current)` line
    /// among them, and a line that does not is named with nothing
    /// edited.
    #[test]
    fn a_ladder_is_written_over_one_that_fits() {
        let lines = titles(&["- 0 (done) [feat: c opening](TODO.md#feat-c-opening)"]);
        let new = written_ladder(ZTOC, &lines).expect("written");
        assert!(
            new.contains("## Ladder\n\n- 0 (done) [feat: c opening]"),
            "{new}"
        );
        let older = ZTOC.replace(
            "- 0 (done) chore: the last cycle",
            "- 0 (done) [feat: c opening](TODO.md#feat-c-opening)\n\
             - 1 (current) [feat: one](TODO.md#feat-one)",
        );
        let new = written_ladder(&older, &lines).expect("a ladder with a current rung fits");
        assert!(!new.contains("(current)"), "{new}");
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

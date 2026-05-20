//! Shared core for the `validate-todo` / `fix-todo` subcommands:
//! parse `notes/todo.md`'s numbered sections and compute the
//! renumber / re-indent plan.
//!
//! - `## Todo` and `## Bugs` hold manually-numbered entries
//!   (`1.` `2.` … at column 0); every other section is skipped.
//! - `analyze` walks the file once: it returns one `Change` per
//!   entry whose number or continuation indent is off, plus the
//!   fully renumbered / re-indented file content.
//! - Continuation indent is normalized against each entry's
//!   *measured* base indent, so nested `- ` sub-bullets keep
//!   their relative depth.

/// Default todo file, relative to the workspace root.
pub const TODO_FILE: &str = "notes/todo.md";

/// A manually-numbered section of the todo file.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Section {
    /// The `## Todo` backlog list.
    Todo,
    /// The `## Bugs` list.
    Bugs,
}

impl Section {
    /// The section's markdown header text (`## Todo` / `## Bugs`).
    pub fn header(self) -> &'static str {
        match self {
            Section::Todo => "## Todo",
            Section::Bugs => "## Bugs",
        }
    }
}

/// One entry whose number and/or continuation indent needs to
/// change for the section to be sequentially numbered.
///
/// - `num_old` / `num_new` — the displayed number and its
///   sequential (1-based) position; equal when only the indent is
///   off.
/// - `new_first_line` — the entry's first line with the corrected
///   number.
/// - `indent_old` / `indent_new` — the measured continuation-base
///   indent and the expected one (the number prefix's width);
///   both `None` when the entry has no continuation lines.
pub struct Change {
    pub section: Section,
    pub line_no: usize,
    pub new_first_line: String,
    pub num_old: usize,
    pub num_new: usize,
    pub indent_old: Option<usize>,
    pub indent_new: Option<usize>,
}

/// Result of scanning a todo file: per-section entry counts, the
/// entries needing a fix, and the fully corrected file content.
pub struct Analysis {
    pub todo_count: usize,
    pub bugs_count: usize,
    pub changes: Vec<Change>,
    pub fixed: String,
}

/// An entry being accumulated during the `analyze` walk — its
/// `N. ` number, that line minus its prefix, and the raw
/// continuation lines (blank lines included).
struct PendingEntry {
    section: Section,
    line_no: usize,
    num_old: usize,
    num_new: usize,
    rest: String,
    cont: Vec<String>,
}

/// Number of decimal digits in `n` (entry numbers are ≥ 1).
fn digit_count(n: usize) -> usize {
    n.to_string().len()
}

/// Count of leading ASCII spaces on `line`.
fn leading_spaces(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

/// Returns `entry` / `entries` — the count noun for `n`.
pub fn entry_word(n: usize) -> &'static str {
    if n == 1 { "entry" } else { "entries" }
}

/// Build the `[line: …]` tag for an entry — its line number,
/// then what changed (`was N`, `indent A → B`, or both). Used by
/// `validate-todo` and `fix-todo` for the same per-entry line.
pub fn change_tag(c: &Change) -> String {
    let mut parts: Vec<String> = Vec::new();
    if c.num_old != c.num_new {
        parts.push(format!("was {}", c.num_old));
    }
    if let (Some(o), Some(n)) = (c.indent_old, c.indent_new)
        && o != n
    {
        parts.push(format!("indent {o} → {n}"));
    }
    format!("[{}: {}]", c.line_no, parts.join(", "))
}

/// Classify a line as a `## ` section header.
///
/// - `None` — not a level-2 header (covers `# `, `### `, body
///   text).
/// - `Some(None)` — a `## ` header that isn't `## Todo` /
///   `## Bugs` (e.g. `## In Progress`, `## Done`).
/// - `Some(Some(section))` — the `## Todo` or `## Bugs` header.
fn section_header(line: &str) -> Option<Option<Section>> {
    if !line.starts_with("## ") {
        return None;
    }
    Some(match line.trim_end() {
        "## Todo" => Some(Section::Todo),
        "## Bugs" => Some(Section::Bugs),
        _ => None,
    })
}

/// Parse a leading `N. ` entry prefix at column 0.
///
/// Returns `(number, prefix_len)` — the entry number and the byte
/// length of the `N. ` prefix — when `line` begins with one or
/// more ASCII digits followed by `". "` (the renumber
/// convention's `^\d+\. ` anchor). Returns `None` for intro lines
/// (which carry a leading space), continuation lines, and
/// headings.
fn parse_entry_prefix(line: &str) -> Option<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 || bytes.get(i) != Some(&b'.') || bytes.get(i + 1) != Some(&b' ') {
        return None;
    }
    let num: usize = line[..i].parse().ok()?;
    Some((num, i + 2))
}

/// Finalize a pending entry: emit its corrected lines into `out`
/// and, when the number or continuation indent changed, push a
/// `Change`.
///
/// The continuation block is shifted as a whole by
/// `prefix_width − measured_base`, so a deliberately nested
/// sub-bullet keeps its depth relative to the entry's base.
fn finish_entry(p: PendingEntry, changes: &mut Vec<Change>, out: &mut Vec<String>) {
    let prefix_width = digit_count(p.num_new) + 2; // ". " is two chars
    let new_first_line = format!("{}. {}", p.num_new, p.rest);

    // Measured base = minimum indent of the *indented* non-blank
    // continuation lines. A column-0 line can't be a real list
    // continuation (continuations must be indented), so it's
    // excluded from the base and emitted verbatim below — a stray
    // one doesn't drag the base down to 0.
    let base = p
        .cont
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| leading_spaces(l))
        .filter(|&n| n > 0)
        .min();
    let (indent_old, indent_new) = match base {
        Some(b) => (Some(b), Some(prefix_width)),
        None => (None, None),
    };

    out.push(new_first_line.clone());
    let delta: isize = base.map_or(0, |b| prefix_width as isize - b as isize);
    for line in &p.cont {
        let indent = leading_spaces(line);
        if line.trim().is_empty() || indent == 0 {
            // Blank line, or a column-0 line that isn't a real
            // continuation — pass through unchanged.
            out.push(line.clone());
        } else {
            let shifted = (indent as isize + delta).max(0) as usize;
            out.push(format!(
                "{}{}",
                " ".repeat(shifted),
                line.trim_start_matches(' ')
            ));
        }
    }

    if p.num_old != p.num_new || indent_old != indent_new {
        changes.push(Change {
            section: p.section,
            line_no: p.line_no,
            new_first_line,
            num_old: p.num_old,
            num_new: p.num_new,
            indent_old,
            indent_new,
        });
    }
}

/// Scan a todo file's `## Todo` and `## Bugs` sections.
///
/// - Walks lines once, tracking the current `## ` section and the
///   entry being accumulated.
/// - An entry first-line is `N. ` at column 0; expected numbering
///   is `1..N` in document order.
/// - Any heading (`#`, `###`, …) ends the current entry's
///   continuation block; `## Todo` / `## Bugs` headers also reset
///   the section.
/// - Returns one `Change` per off entry plus `fixed`, the file
///   content with every entry renumbered and re-indented. `fixed`
///   round-trips an already-correct file unchanged.
pub fn analyze(content: &str) -> Analysis {
    let mut changes = Vec::new();
    let mut out: Vec<String> = Vec::new();
    let mut todo_count = 0;
    let mut bugs_count = 0;
    let mut section: Option<Section> = None;
    let mut entry_idx = 0usize;
    let mut pending: Option<PendingEntry> = None;

    // `split('\n')` (not `lines()`) so a trailing newline survives
    // the join below and `fixed` round-trips byte-for-byte.
    for (i, line) in content.split('\n').enumerate() {
        if let Some(kind) = section_header(line) {
            // A new ## section — finish the pending entry, switch.
            if let Some(p) = pending.take() {
                finish_entry(p, &mut changes, &mut out);
            }
            section = kind;
            entry_idx = 0;
            out.push(line.to_string());
            continue;
        }
        // Any other heading (`#`, `###`, …) ends the current
        // entry's continuation block without changing the section.
        if line.starts_with('#') {
            if let Some(p) = pending.take() {
                finish_entry(p, &mut changes, &mut out);
            }
            out.push(line.to_string());
            continue;
        }
        let Some(sec) = section else {
            // Not inside a ## Todo / ## Bugs section.
            out.push(line.to_string());
            continue;
        };
        if let Some((num, prefix_len)) = parse_entry_prefix(line) {
            // A new entry — finish the previous one we were on.
            if let Some(p) = pending.take() {
                finish_entry(p, &mut changes, &mut out);
            }
            entry_idx += 1;
            match sec {
                Section::Todo => todo_count += 1,
                Section::Bugs => bugs_count += 1,
            }
            pending = Some(PendingEntry {
                section: sec,
                line_no: i + 1,
                num_old: num,
                num_new: entry_idx,
                rest: line[prefix_len..].to_string(),
                cont: Vec::new(),
            });
        } else if let Some(p) = pending.as_mut() {
            // Append the line of the current entry.
            p.cont.push(line.to_string());
        } else {
            // Section intro text before the first entry.
            out.push(line.to_string());
        }
    }
    // End of file — finish any entry still open.
    if let Some(p) = pending.take() {
        finish_entry(p, &mut changes, &mut out);
    }

    Analysis {
        todo_count,
        bugs_count,
        changes,
        fixed: out.join("\n"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A correctly-numbered, correctly-indented file: no changes,
    /// and `fixed` round-trips it byte-for-byte.
    #[test]
    fn clean_file_has_no_changes() {
        let doc = "# Todo\n\n## Todo\n\n intro\n\n1. First.\n   cont.\n2. Second.\n3. Third.\n   more.\n\n## Bugs\n\n bugs intro\n\n1. A bug.\n";
        let a = analyze(doc);
        assert_eq!(a.todo_count, 3);
        assert_eq!(a.bugs_count, 1);
        assert!(a.changes.is_empty());
        assert_eq!(a.fixed, doc);
    }

    /// Entries numbered 2, 3 renumber to 1, 2.
    #[test]
    fn gap_renumbers() {
        let a = analyze("## Todo\n\n2. a.\n3. b.\n");
        assert_eq!(a.changes.len(), 2);
        assert_eq!((a.changes[0].num_old, a.changes[0].num_new), (2, 1));
        assert_eq!(a.changes[0].new_first_line, "1. a.");
        assert_eq!(a.changes[0].line_no, 3);
        assert_eq!((a.changes[1].num_old, a.changes[1].num_new), (3, 2));
        assert_eq!(a.fixed, "## Todo\n\n1. a.\n2. b.\n");
    }

    /// A two-digit entry renumbered to one digit shifts its
    /// continuation indent 4 → 3.
    #[test]
    fn indent_shifts_with_prefix_width() {
        let a = analyze("## Todo\n\n10. x.\n    cont.\n");
        assert_eq!(a.changes.len(), 1);
        let c = &a.changes[0];
        assert_eq!((c.num_old, c.num_new), (10, 1));
        assert_eq!((c.indent_old, c.indent_new), (Some(4), Some(3)));
        assert_eq!(a.fixed, "## Todo\n\n1. x.\n   cont.\n");
    }

    /// A correctly-numbered entry with the wrong continuation
    /// indent is still flagged and rewritten.
    #[test]
    fn indent_only_issue_flagged() {
        let a = analyze("## Todo\n\n1. x.\n  cont.\n");
        assert_eq!(a.changes.len(), 1);
        let c = &a.changes[0];
        assert_eq!((c.num_old, c.num_new), (1, 1));
        assert_eq!((c.indent_old, c.indent_new), (Some(2), Some(3)));
        assert_eq!(a.fixed, "## Todo\n\n1. x.\n   cont.\n");
    }

    /// Continuation indent is measured against the minimum (base)
    /// indent, so nested sub-bullets don't trip a correct entry.
    #[test]
    fn nested_sub_bullets_use_min_indent() {
        let a = analyze("## Todo\n\n1. x.\n   - sub.\n     deep.\n");
        assert!(a.changes.is_empty());
    }

    /// When the base indent is off, the whole continuation block
    /// shifts together — nested depth is preserved, not flattened.
    #[test]
    fn nested_block_shifts_preserving_depth() {
        let a = analyze("## Todo\n\n1. x.\n  - sub.\n    deep.\n");
        assert_eq!(a.fixed, "## Todo\n\n1. x.\n   - sub.\n     deep.\n");
    }

    /// `## Bugs` is numbered independently of `## Todo`.
    #[test]
    fn bugs_numbered_independently() {
        let a = analyze("## Todo\n\n1. t.\n\n## Bugs\n\n2. b.\n");
        assert_eq!(a.todo_count, 1);
        assert_eq!(a.bugs_count, 1);
        assert_eq!(a.changes.len(), 1);
        assert_eq!(a.changes[0].section, Section::Bugs);
        assert_eq!((a.changes[0].num_old, a.changes[0].num_new), (2, 1));
    }

    /// A `### ` heading inside `## Todo` ends the prior entry and
    /// is not itself counted.
    #[test]
    fn heading_in_todo_section_ignored() {
        let a = analyze("## Todo\n\n intro\n\n### Current In Progress\n1. x.\n2. y.\n");
        assert_eq!(a.todo_count, 2);
        assert!(a.changes.is_empty());
    }

    /// A column-0 `N. ` line in `## Done` is not scanned.
    #[test]
    fn done_section_not_scanned() {
        let a = analyze("## Todo\n\n1. x.\n\n## Done\n\n9. done thing\n");
        assert_eq!(a.todo_count, 1);
        assert!(a.changes.is_empty());
    }

    /// Blank lines within an entry's continuation block carry no
    /// indent and don't affect the measured base.
    #[test]
    fn blank_continuation_lines_ignored() {
        let a = analyze("## Todo\n\n1. x.\n   cont.\n\n   more.\n2. y.\n");
        assert_eq!(a.todo_count, 2);
        assert!(a.changes.is_empty());
    }

    /// `fixed` is a fixed point: re-running on it yields no
    /// further changes and identical content.
    #[test]
    fn fixed_is_idempotent() {
        let once = analyze("## Todo\n\n3. a.\n5. b.\n   c.\n").fixed;
        let twice = analyze(&once);
        assert!(twice.changes.is_empty());
        assert_eq!(twice.fixed, once);
    }

    /// A stray column-0 line inside an entry's continuation block
    /// isn't a real continuation: it doesn't pull the measured
    /// base down, and it passes through verbatim.
    #[test]
    fn column_zero_line_is_not_a_continuation() {
        let doc = "## Todo\n\n1. x.\n   real cont.\n0\n   more cont.\n";
        let a = analyze(doc);
        assert!(a.changes.is_empty());
        assert_eq!(a.fixed, doc);
    }

    /// Build a `Change` for `change_tag` tests.
    fn change(num_old: usize, num_new: usize, indent: Option<(usize, usize)>) -> Change {
        Change {
            section: Section::Todo,
            line_no: 1,
            new_first_line: String::new(),
            num_old,
            num_new,
            indent_old: indent.map(|(o, _)| o),
            indent_new: indent.map(|(_, n)| n),
        }
    }

    /// A pure renumber tags the line number and old number.
    #[test]
    fn tag_number_only() {
        assert_eq!(change_tag(&change(3, 1, None)), "[1: was 3]");
    }

    /// A pure re-indent tags the line number and indent shift.
    #[test]
    fn tag_indent_only() {
        assert_eq!(change_tag(&change(1, 1, Some((2, 3)))), "[1: indent 2 → 3]");
    }

    /// A renumber that also shifts indent tags both.
    #[test]
    fn tag_number_and_indent() {
        assert_eq!(
            change_tag(&change(10, 9, Some((4, 3)))),
            "[1: was 10, indent 4 → 3]"
        );
    }
}

//! Shared core for the `validate-todo` / `fix-todo` subcommands:
//! parse `notes/todo.md`'s numbered sections and compute the
//! renumber / re-indent plan.
//!
//! - `## Todo` and `## Bugs` hold manually-numbered entries
//!   (`1.` `2.` … at column 0); every other section is skipped.
//! - `analyze` walks the file once and returns one `Change` per
//!   entry whose number or continuation indent is off.
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
/// - `indent_old` / `indent_new` — the measured continuation-base
///   indent and the expected one (the number prefix's width);
///   both `None` when the entry has no continuation lines.
pub struct Change {
    pub section: Section,
    pub line_no: usize,
    pub first_line: String,
    pub num_old: usize,
    pub num_new: usize,
    pub indent_old: Option<usize>,
    pub indent_new: Option<usize>,
}

/// Result of scanning a todo file: per-section entry counts and
/// the list of entries needing a fix.
pub struct Analysis {
    pub todo_count: usize,
    pub bugs_count: usize,
    pub changes: Vec<Change>,
}

/// An entry being accumulated during the `analyze` walk — its
/// first line plus the indents of its continuation lines.
struct PendingEntry {
    section: Section,
    line_no: usize,
    first_line: String,
    num_old: usize,
    num_new: usize,
    cont_indents: Vec<usize>,
}

/// Number of decimal digits in `n` (entry numbers are ≥ 1).
fn digit_count(n: usize) -> usize {
    n.to_string().len()
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
/// Returns the entry number when `line` begins with one or more
/// ASCII digits followed by `". "` — the renumber convention's
/// `^\d+\. ` anchor. Returns `None` for intro lines (which carry
/// a leading space), continuation lines, and headings.
fn parse_entry_prefix(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 || bytes.get(i) != Some(&b'.') || bytes.get(i + 1) != Some(&b' ') {
        return None;
    }
    line[..i].parse().ok()
}

/// Finalize a pending entry: measure its continuation-indent base
/// and, when the number or indent needs to change, push a
/// `Change`.
fn finish_entry(p: PendingEntry, changes: &mut Vec<Change>) {
    let prefix_width = digit_count(p.num_new) + 2; // ". " is two chars
    let indent_old = p.cont_indents.iter().copied().min();
    let indent_new = indent_old.map(|_| prefix_width);
    if p.num_old != p.num_new || indent_old != indent_new {
        changes.push(Change {
            section: p.section,
            line_no: p.line_no,
            first_line: p.first_line,
            num_old: p.num_old,
            num_new: p.num_new,
            indent_old,
            indent_new,
        });
    }
}

/// Scan a todo file's `## Todo` and `## Bugs` sections and report
/// every entry whose number or continuation indent needs to
/// change.
///
/// - Walks lines once, tracking the current `## ` section and the
///   entry being accumulated.
/// - An entry first-line is `N. ` at column 0; expected numbering
///   is `1..N` in document order.
/// - Any heading (`#`, `###`, …) ends the current entry's
///   continuation block; `## Todo` / `## Bugs` headers also reset
///   the section.
/// - An entry's continuation indent is reported against the
///   *measured* base (the minimum non-blank continuation indent),
///   so nested sub-bullets keep their relative depth.
pub fn analyze(content: &str) -> Analysis {
    let mut changes = Vec::new();
    let mut todo_count = 0;
    let mut bugs_count = 0;
    let mut section: Option<Section> = None;
    let mut entry_idx = 0usize;
    let mut pending: Option<PendingEntry> = None;

    for (i, line) in content.lines().enumerate() {
        if let Some(kind) = section_header(line) {
            // We have a new section we're going to number its entries
            if let Some(p) = pending.take() {
                // Flush any pending entry
                finish_entry(p, &mut changes);
            }
            section = kind;
            entry_idx = 0;
            continue;
        }
        // Any other heading (`#`, `###`, …) ends the current
        // entry's continuation block without changing the section.
        if line.starts_with('#') {
            if let Some(p) = pending.take() {
                finish_entry(p, &mut changes);
            }
            continue;
        }
        let Some(sec) = section else {
            // We're not processing a section
            continue;
        };
        if let Some(num) = parse_entry_prefix(line) {
            // New entry for the current section
            if let Some(p) = pending.take() {
                // Flush any previous entry we were working on
                finish_entry(p, &mut changes);
            }

            // Update information for this entry
            entry_idx += 1;
            match sec {
                Section::Todo => todo_count += 1,
                Section::Bugs => bugs_count += 1,
            }
            pending = Some(PendingEntry {
                section: sec,
                line_no: i + 1,
                first_line: line.to_string(),
                num_old: num,
                num_new: entry_idx,
                cont_indents: Vec::new(),
            });
        } else if let Some(p) = pending.as_mut() {
            // Continuation line: record the leading-space count of
            // non-blank lines; blank lines carry no indent.
            if !line.trim().is_empty() {
                let indent = line.len() - line.trim_start_matches(' ').len();
                p.cont_indents.push(indent);
            }
        }
    }

    if let Some(p) = pending.take() {
        finish_entry(p, &mut changes);
    }

    Analysis {
        todo_count,
        bugs_count,
        changes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A correctly-numbered, correctly-indented file: no changes.
    #[test]
    fn clean_file_has_no_changes() {
        let doc = "# Todo\n\n## Todo\n\n intro\n\n1. First.\n   cont.\n2. Second.\n3. Third.\n   more.\n\n## Bugs\n\n bugs intro\n\n1. A bug.\n";
        let a = analyze(doc);
        assert_eq!(a.todo_count, 3);
        assert_eq!(a.bugs_count, 1);
        assert!(a.changes.is_empty());
    }

    /// Entries numbered 2, 3 renumber to 1, 2.
    #[test]
    fn gap_renumbers() {
        let a = analyze("## Todo\n\n2. a.\n3. b.\n");
        assert_eq!(a.changes.len(), 2);
        assert_eq!((a.changes[0].num_old, a.changes[0].num_new), (2, 1));
        assert_eq!(a.changes[0].first_line, "2. a.");
        assert_eq!(a.changes[0].line_no, 3);
        assert_eq!((a.changes[1].num_old, a.changes[1].num_new), (3, 2));
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
    }

    /// A correctly-numbered entry with the wrong continuation
    /// indent is still flagged.
    #[test]
    fn indent_only_issue_flagged() {
        let a = analyze("## Todo\n\n1. x.\n  cont.\n");
        assert_eq!(a.changes.len(), 1);
        let c = &a.changes[0];
        assert_eq!((c.num_old, c.num_new), (1, 1));
        assert_eq!((c.indent_old, c.indent_new), (Some(2), Some(3)));
    }

    /// Continuation indent is measured against the minimum
    /// (base) indent, so nested sub-bullets don't trip it.
    #[test]
    fn nested_sub_bullets_use_min_indent() {
        let a = analyze("## Todo\n\n1. x.\n   - sub.\n     deep.\n");
        assert!(a.changes.is_empty());
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
}

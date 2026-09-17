//! `agent-files size [DIR]`: the set's line count, and the slide of
//! the per-file table that records it.
//!
//! The count is `wc -l` over the set's markdown, `AGENTS.md`,
//! `custom.md`, and `agent-data/*.md`. The
//! `agent-data/agent-files-vX.Y.Z` marker has no `.md` extension, so
//! it falls out by that rule rather than by a name this module has to
//! know. The marker is empty, so its lines were never what the rule
//! was for: what it buys is that the marker takes no row in the table
//! and no place in the file count.
//!
//! - `count_set`: one [`FileCount`] per set file, in the order the
//!   table reads, the two root files then `agent-data/` sorted.
//!   `agent-data/rationale.md` is counted but not totalled: it is the
//!   rules' why, and a rule gaining one would otherwise read as the
//!   set growing.
//! - `report`: the per-file listing, an uncounted file's number in
//!   angle brackets so it is shown without being summed.
//! - `find_table` / `slide` / `render`: the per-file table in the
//!   notes file, parsed, given a new leftmost column and shortened by
//!   its rightmost one, and written back. The window keeps the width
//!   it had, so widening it is an edit to the file rather than a flag
//!   here.
//! - `SizeArgs`: the clap surface. Dry-run by default and
//!   `--no-dry-run` applies, the spelling `fix-desc` and `fix-todo`
//!   already carry.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Args;
use log::{error, info, warn};

use crate::common;

use super::diff::{self, AGENT_DATA, AGENTS_MD, CUSTOM_MD};

/// The set file whose lines are shown but never totalled.
pub const RATIONALE: &str = "agent-data/rationale.md";

/// The notes file holding the per-file table, relative to the set
/// directory.
pub const SIZE_NOTE: &str = "notes/agent-files-size.md";

/// The per-file table's first header cell, which is what tells it
/// from the `## Counts` table above it.
pub const FILE_HEADER: &str = "File";

/// The row whose cells are the totals rather than a file's lines.
pub const TOTAL_ROW: &str = "total";

/// One set file's line count and whether the total takes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileCount {
    /// The file's path relative to the set directory.
    pub path: String,
    /// Its `wc -l`, so a file not ending in a newline does not get a
    /// line the shell would not count.
    pub lines: usize,
    /// False for the why-file, whose lines are shown and not summed.
    pub counted: bool,
}

impl FileCount {
    /// The cell this count writes: the bare number, or the number in
    /// angle brackets when the total leaves it out.
    pub fn cell(&self) -> String {
        if self.counted {
            self.lines.to_string()
        } else {
            format!("<{}>", self.lines)
        }
    }
}

/// `wc -l` for `path`: newlines, so a file with no final newline
/// counts the same here as it does in the shell.
fn wc_l(path: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    Ok(bytes.iter().filter(|b| **b == b'\n').count())
}

/// The set's markdown under `dir`, each with its line count, in the
/// order the table reads: `AGENTS.md`, `custom.md`, then the
/// `agent-data/` files sorted. A missing file is simply absent, so a
/// set without `custom.md` is a set of one fewer.
pub fn count_set(dir: &Path) -> Result<Vec<FileCount>, Box<dyn std::error::Error>> {
    let mut paths: Vec<String> = Vec::new();
    for name in [AGENTS_MD, CUSTOM_MD] {
        if dir.join(name).is_file() {
            paths.push(name.to_string());
        }
    }
    let mut data: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir.join(AGENT_DATA)) {
        for e in entries.flatten() {
            let name = e.file_name();
            let Some(name) = name.to_str() else { continue };
            if e.path().is_file() && name.ends_with(".md") {
                data.push(format!("{AGENT_DATA}/{name}"));
            }
        }
    }
    data.sort();
    paths.extend(data);

    paths
        .into_iter()
        .map(|path| {
            let lines = wc_l(&dir.join(&path))?;
            let counted = path != RATIONALE;
            Ok(FileCount {
                path,
                lines,
                counted,
            })
        })
        .collect()
}

/// The lines the total takes.
pub fn total(counts: &[FileCount]) -> usize {
    counts.iter().filter(|c| c.counted).map(|c| c.lines).sum()
}

/// The per-file listing and its summary, one aligned line per file,
/// the counts right-aligned so the column reads as numbers.
pub fn report(counts: &[FileCount]) -> String {
    let name_width = counts.iter().map(|c| c.path.len()).max().unwrap_or(0);
    let cell_width = counts.iter().map(|c| c.cell().len()).max().unwrap_or(0);
    let mut out = String::new();
    for c in counts {
        out.push_str(&format!(
            "{:<name_width$}  {:>cell_width$}\n",
            c.path,
            c.cell()
        ));
    }
    let counted = counts.iter().filter(|c| c.counted).count();
    out.push_str(&format!("{counted} files, {} lines\n", total(counts)));
    out
}

/// A markdown table as the slide needs it: the header's cells and one
/// `(name, cells)` per body row, the name being the row's first cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    /// The header cells after the `File` one, so one label per data
    /// column, newest first.
    pub labels: Vec<String>,
    /// The body rows, `total` among them, in file order.
    pub rows: Vec<(String, Vec<String>)>,
}

/// Where the table sat, so the rewrite replaces those lines alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableSpan {
    /// The header line's index.
    pub start: usize,
    /// One past the last body line.
    pub end: usize,
    pub table: Table,
}

/// Split a markdown table row into its cells, the empties either side
/// of the outer pipes dropped.
fn cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let inner = trimmed
        .strip_prefix('|')
        .unwrap_or(trimmed)
        .strip_suffix('|')
        .unwrap_or(trimmed);
    inner.split('|').map(|c| c.trim().to_string()).collect()
}

/// The per-file table in `text`: the one whose first header cell is
/// `File`, which is what tells it from the `## Counts` table. `None`
/// when the file holds no such table.
pub fn find_table(text: &str) -> Option<TableSpan> {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.iter().position(|l| {
        l.trim_start().starts_with('|') && cells(l).first().map(String::as_str) == Some(FILE_HEADER)
    })?;
    // The header, then the alignment row, then the body until the
    // table stops.
    let mut end = start + 2;
    while end < lines.len() && lines[end].trim_start().starts_with('|') {
        end += 1;
    }
    let labels = cells(lines[start]).split_off(1);
    let rows = lines[start + 2..end]
        .iter()
        .map(|l| {
            let mut c = cells(l);
            let name = if c.is_empty() {
                String::new()
            } else {
                c.remove(0)
            };
            (name, c)
        })
        .collect();
    Some(TableSpan {
        start,
        end,
        table: Table { labels, rows },
    })
}

/// The table with `label`'s column inserted at the left and the
/// rightmost column dropped, so the window keeps its width.
///
/// A file in the set gets its cell, one only in the kept columns gets
/// an empty one, and a row left with nothing but empties goes with the
/// column that carried its last number. `total` stays last whatever
/// the file order does.
pub fn slide(table: &Table, label: &str, counts: &[FileCount]) -> Table {
    let keep = table.labels.len().saturating_sub(1);
    let old_cell = |name: &str, i: usize| -> String {
        table
            .rows
            .iter()
            .find(|(n, _)| n == name)
            .and_then(|(_, c)| c.get(i))
            .cloned()
            .unwrap_or_default()
    };

    let mut names: Vec<String> = counts.iter().map(|c| c.path.clone()).collect();
    for (name, _) in &table.rows {
        if name != TOTAL_ROW && !names.contains(name) {
            names.push(name.clone());
        }
    }
    names.sort_by_key(|n| order_key(n));

    let new_cell = |name: &str| -> String {
        counts
            .iter()
            .find(|c| c.path == name)
            .map(FileCount::cell)
            .unwrap_or_default()
    };

    let mut rows: Vec<(String, Vec<String>)> = Vec::new();
    for name in names {
        let mut row = vec![new_cell(&name)];
        row.extend((0..keep).map(|i| old_cell(&name, i)));
        if row.iter().any(|c| !c.is_empty()) {
            rows.push((name, row));
        }
    }
    let mut totals = vec![total(counts).to_string()];
    totals.extend((0..keep).map(|i| old_cell(TOTAL_ROW, i)));
    rows.push((TOTAL_ROW.to_string(), totals));

    let mut labels = vec![label.to_string()];
    labels.extend(table.labels.iter().take(keep).cloned());
    Table { labels, rows }
}

/// The table's reading order: the two root files first, then
/// everything else lexically, so a file gone from the set still sorts
/// into the slot it held.
fn order_key(name: &str) -> (u8, String) {
    match name {
        AGENTS_MD => (0, String::new()),
        CUSTOM_MD => (1, String::new()),
        other => (2, other.to_string()),
    }
}

/// The table as markdown, the counts right-aligned as the file writes
/// them.
pub fn render(table: &Table) -> String {
    let mut out = format!("| {FILE_HEADER} | {} |\n", table.labels.join(" | "));
    out.push_str("|---");
    for _ in &table.labels {
        out.push_str("|---:");
    }
    out.push_str("|\n");
    for (name, row) in &table.rows {
        out.push_str(&format!("| {name} | {} |\n", row.join(" | ")));
    }
    out
}

/// CLI args for `agent-files size`.
#[derive(Args, Debug)]
pub struct SizeArgs {
    /// A directory holding a copy of the set [default: this
    /// workspace]
    #[arg(value_name = "DIR")]
    pub dir: Option<PathBuf>,

    /// The notes file whose per-file table slides [default:
    /// DIR/notes/agent-files-size.md]
    #[arg(long = "file", value_name = "PATH")]
    pub file: Option<PathBuf>,

    /// The new column's label, `--label=TEXT` when it opens on a
    /// `-` [default: the set's version file]
    #[arg(long = "label", value_name = "TEXT")]
    pub label: Option<String>,

    /// Slide the table in place [default: dry-run]
    #[arg(long = "no-dry-run")]
    pub no_dry_run: bool,
}

impl SizeArgs {
    /// Count the set, then slide its table. Non-zero on an error
    /// alone: a count with no table to slide is a complete answer.
    pub fn run(&self) -> ExitCode {
        match self.size() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                error!("agent-files size: {e}");
                ExitCode::FAILURE
            }
        }
    }

    /// Report the count, and slide the per-file table when the notes
    /// file has one.
    fn size(&self) -> Result<(), Box<dyn std::error::Error>> {
        let root = common::find_workspace_root();
        let dir = diff::resolve_here(root.as_deref(), self.dir.as_deref())?;
        let counts = count_set(&dir.path)?;
        info!("agent-files size: {} ({})", dir.shown, dir.source);
        info!("{}", report(&counts).trim_end());

        let file = self
            .file
            .clone()
            .unwrap_or_else(|| dir.path.join(SIZE_NOTE));
        if !file.is_file() {
            info!("{}: no such file, so no table to slide", file.display());
            return Ok(());
        }
        let text = std::fs::read_to_string(&file)
            .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
        let Some(span) = find_table(&text) else {
            info!(
                "{}: no table headed `| {FILE_HEADER} |`, so nothing to slide",
                file.display()
            );
            return Ok(());
        };

        let label = self.label(&dir.path)?;
        if span.table.labels.first() == Some(&label) {
            warn!("column {label} is already the leftmost one: --label names another");
        }
        let slid = slide(&span.table, &label, &counts);
        let dropped = span
            .table
            .labels
            .last()
            .cloned()
            .unwrap_or_else(|| "none".to_string());
        info!(
            "{}: column {label} in, column {dropped} out",
            file.display()
        );

        let rendered = render(&slid);
        if !self.no_dry_run {
            info!("{}", rendered.trim_end());
            info!("re-run with --no-dry-run to apply");
            return Ok(());
        }
        let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
        lines.splice(
            span.start..span.end,
            rendered.lines().map(str::to_string).collect::<Vec<_>>(),
        );
        let mut out = lines.join("\n");
        if text.ends_with('\n') {
            out.push('\n');
        }
        std::fs::write(&file, out).map_err(|e| format!("cannot write {}: {e}", file.display()))?;
        info!("{}: slid", file.display());
        Ok(())
    }

    /// The new column's label: the flag, else the set's version file.
    /// An error names both when neither is there, since a column with
    /// no label says nothing about what it counts.
    fn label(&self, dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(l) = &self.label {
            return Ok(l.clone());
        }
        match super::versions(dir).first() {
            Some(v) => Ok(v.clone()),
            None => Err(format!(
                "no agent-data/agent-files-v* file in {}: give --label",
                dir.display()
            )
            .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct T {
        #[command(flatten)]
        a: SizeArgs,
    }

    /// A set directory under the test tmp root holding `(path,
    /// content)` files.
    fn set_dir(tag: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = crate::test_tmp_root::resolve_tmp_root().join(format!("vc_x1_af_size_{tag}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(AGENT_DATA)).expect("mkdir agent-data");
        std::fs::create_dir_all(dir.join("notes")).expect("mkdir notes");
        for (path, content) in files {
            std::fs::write(dir.join(path), content).expect("write");
        }
        dir
    }

    /// The table this project's notes file carries, three columns
    /// wide, with a file the count leaves out and one that has gone
    /// from the set.
    const TABLE: &str = "\
| File | v0.2.5 | v0.2.4 | v0.2.3 |
|---|---:|---:|---:|
| AGENTS.md | 384 | 384 | 384 |
| custom.md | 12 | 12 | 12 |
| agent-data/code.md | 94 | 94 | 94 |
| agent-data/messaging.md |  | 40 | 40 |
| agent-data/rationale.md | <543> | 541 | 541 |
| total | 490 | 530 | 530 |
";

    /// The operands and flags parse, and the defaults are the bare
    /// command's.
    #[test]
    fn flags_parse() {
        let a = T::try_parse_from(["t"]).expect("bare").a;
        assert!(a.dir.is_none() && a.file.is_none() && a.label.is_none() && !a.no_dry_run);
        let a = T::try_parse_from([
            "t",
            "../x",
            "--file",
            "n/s.md",
            // A label opening on `-` needs the `=` form, which is
            // every pre-versioning label the notes file uses.
            "--label=- v0.1.0",
            "--no-dry-run",
        ])
        .expect("full")
        .a;
        assert_eq!(a.dir, Some(PathBuf::from("../x")));
        assert_eq!(a.file, Some(PathBuf::from("n/s.md")));
        assert_eq!(a.label.as_deref(), Some("- v0.1.0"));
        assert!(a.no_dry_run);
    }

    /// The count is `wc -l`, the two root files lead and
    /// `agent-data/` follows sorted, the version marker is not
    /// markdown so it is absent, and the why-file is counted without
    /// being totalled.
    #[test]
    fn count_set_orders_and_excludes() {
        let dir = set_dir(
            "count",
            &[
                ("AGENTS.md", "a\nb\nc\n"),
                ("custom.md", "x\n"),
                ("agent-data/prose.md", "1\n2\n"),
                ("agent-data/code.md", "1\n2\n3\n4\n"),
                ("agent-data/rationale.md", "w\nh\ny\n"),
                ("agent-data/agent-files-v0.2.5", ""),
                ("agent-data/notes.txt", "not markdown\n"),
            ],
        );
        let counts = count_set(&dir).expect("count");
        assert_eq!(
            counts.iter().map(|c| c.path.as_str()).collect::<Vec<_>>(),
            vec![
                "AGENTS.md",
                "custom.md",
                "agent-data/code.md",
                "agent-data/prose.md",
                "agent-data/rationale.md",
            ]
        );
        assert_eq!(counts[0].lines, 3);
        assert!(counts.iter().all(|c| c.counted == (c.path != RATIONALE)));
        // 3 + 1 + 4 + 2, the why-file's 3 left out.
        assert_eq!(total(&counts), 10);
    }

    /// A file with no final newline counts as the shell counts it,
    /// so the notes file's `wc -l` and this agree.
    #[test]
    fn a_file_without_a_final_newline_counts_its_newlines() {
        let dir = set_dir("nonl", &[("AGENTS.md", "a\nb")]);
        assert_eq!(count_set(&dir).expect("count")[0].lines, 1);
    }

    /// The listing aligns, an uncounted file's number is in angle
    /// brackets, and the summary counts only what it totalled.
    #[test]
    fn report_brackets_the_uncounted_number() {
        let counts = vec![
            FileCount {
                path: "AGENTS.md".to_string(),
                lines: 384,
                counted: true,
            },
            FileCount {
                path: RATIONALE.to_string(),
                lines: 543,
                counted: false,
            },
        ];
        let text = report(&counts);
        assert!(text.contains("AGENTS.md                  384\n"), "{text}");
        assert!(text.contains("agent-data/rationale.md  <543>\n"), "{text}");
        assert!(text.ends_with("1 files, 384 lines\n"), "{text}");
    }

    /// The table is found by its `File` header, past the `## Counts`
    /// table above it, and its span is the header, the rule, and the
    /// body alone.
    #[test]
    fn find_table_picks_the_per_file_one() {
        let text = format!(
            "# Size\n\n| Landed | Cycle | Lines |\n|---|---|---:|\n| 2026-09-16 | c | 1774 |\n\n\
             prose\n\n{TABLE}\ntail\n"
        );
        let span = find_table(&text).expect("found");
        let lines: Vec<&str> = text.lines().collect();
        assert!(
            lines[span.start].starts_with("| File |"),
            "{:?}",
            lines[span.start]
        );
        assert_eq!(lines[span.end], "");
        assert_eq!(span.table.labels, vec!["v0.2.5", "v0.2.4", "v0.2.3"]);
        assert_eq!(span.table.rows.len(), 6);
        assert_eq!(span.table.rows[0].0, "AGENTS.md");
        assert_eq!(span.table.rows[0].1, vec!["384", "384", "384"]);
        assert!(find_table("no table here\n").is_none());
    }

    /// The slide: the new column leads, the oldest goes, the width
    /// holds, a new file takes its sorted slot, a file gone from the
    /// set keeps its remaining numbers, and one whose last number
    /// left with the dropped column goes with it.
    #[test]
    fn slide_inserts_left_and_drops_right() {
        let table = find_table(TABLE).expect("found").table;
        let counts = vec![
            FileCount {
                path: "AGENTS.md".to_string(),
                lines: 390,
                counted: true,
            },
            FileCount {
                path: "custom.md".to_string(),
                lines: 12,
                counted: true,
            },
            FileCount {
                path: "agent-data/code.md".to_string(),
                lines: 94,
                counted: true,
            },
            FileCount {
                path: "agent-data/jj.md".to_string(),
                lines: 391,
                counted: true,
            },
            FileCount {
                path: RATIONALE.to_string(),
                lines: 543,
                counted: false,
            },
        ];
        let slid = slide(&table, "v0.2.6", &counts);
        assert_eq!(slid.labels, vec!["v0.2.6", "v0.2.5", "v0.2.4"]);
        let row = |name: &str| {
            slid.rows
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, c)| c.clone())
        };
        assert_eq!(row("AGENTS.md"), Some(vec_s(&["390", "384", "384"])));
        // New to the set, so its kept cells are empty and it sorts
        // into place among the agent-data files.
        assert_eq!(row("agent-data/jj.md"), Some(vec_s(&["391", "", ""])));
        // Gone from the set, and still carrying a number in a kept
        // column.
        assert_eq!(row("agent-data/messaging.md"), Some(vec_s(&["", "", "40"])));
        assert_eq!(
            row(RATIONALE),
            Some(vec_s(&["<543>", "<543>", "541"])),
            "the why-file's number rides along uncounted"
        );
        assert_eq!(row("total"), Some(vec_s(&["887", "490", "530"])));
        assert_eq!(
            slid.rows
                .iter()
                .map(|(n, _)| n.as_str())
                .collect::<Vec<_>>(),
            vec![
                "AGENTS.md",
                "custom.md",
                "agent-data/code.md",
                "agent-data/jj.md",
                "agent-data/messaging.md",
                RATIONALE,
                "total",
            ]
        );

        // Slid twice, messaging.md's last number leaves with the
        // column that held it, and the row goes.
        let again = slide(&slid, "v0.2.7", &counts);
        assert_eq!(again.labels, vec!["v0.2.7", "v0.2.6", "v0.2.5"]);
        assert!(
            !again
                .rows
                .iter()
                .any(|(n, _)| n == "agent-data/messaging.md"),
            "{again:?}"
        );
    }

    /// `String` rows, so the assertions above read as literals.
    fn vec_s(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    /// Rendering round-trips: the table parses back to what it was,
    /// so a slide with no change to the set rewrites the same bytes.
    #[test]
    fn render_round_trips() {
        let table = find_table(TABLE).expect("found").table;
        assert_eq!(render(&table), TABLE);
        assert_eq!(find_table(&render(&table)).expect("re-found").table, table);
    }

    /// The dry run leaves the file alone and `--no-dry-run` replaces
    /// the table's lines and nothing else.
    #[test]
    fn dry_run_then_write() {
        let dir = set_dir(
            "write",
            &[
                ("AGENTS.md", "a\nb\nc\n"),
                ("agent-data/rationale.md", "w\nh\ny\n"),
                ("agent-data/agent-files-v0.9.0", ""),
                (SIZE_NOTE, &format!("# Size\n\nprose\n\n{TABLE}\ntail\n")),
            ],
        );
        let note = dir.join(SIZE_NOTE);
        let before = std::fs::read_to_string(&note).expect("read");

        let args = SizeArgs {
            dir: Some(dir.clone()),
            file: None,
            label: None,
            no_dry_run: false,
        };
        args.size().expect("dry run");
        assert_eq!(std::fs::read_to_string(&note).expect("read"), before);

        let args = SizeArgs {
            no_dry_run: true,
            ..args
        };
        args.size().expect("write");
        let after = std::fs::read_to_string(&note).expect("read");
        assert!(after.starts_with("# Size\n\nprose\n\n"), "{after}");
        assert!(after.ends_with("\ntail\n"), "{after}");
        // The label came from the version file, and the oldest column
        // went.
        assert!(
            after.contains("| File | v0.9.0 | v0.2.5 | v0.2.4 |\n"),
            "{after}"
        );
        assert!(!after.contains("v0.2.3"), "{after}");
        assert!(after.contains("| AGENTS.md | 3 | 384 | 384 |\n"), "{after}");
        assert!(
            after.contains("| agent-data/rationale.md | <3> | <543> | 541 |\n"),
            "{after}"
        );
        assert!(after.contains("| total | 3 | 490 | 530 |\n"), "{after}");
    }

    /// No version file and no `--label` is an error naming both, and
    /// a notes file without the table is a complete run that says so.
    #[test]
    fn label_and_missing_table() {
        let dir = set_dir("nolabel", &[("AGENTS.md", "a\n"), (SIZE_NOTE, "# Size\n")]);
        let args = SizeArgs {
            dir: Some(dir.clone()),
            file: None,
            label: None,
            no_dry_run: false,
        };
        // No table, so the label is never asked for and the run ends
        // on the count.
        args.size().expect("no table is fine");

        let with_table = set_dir(
            "nolabel2",
            &[
                ("AGENTS.md", "a\n"),
                (SIZE_NOTE, &format!("# Size\n\n{TABLE}")),
            ],
        );
        let args = SizeArgs {
            dir: Some(with_table.clone()),
            file: None,
            label: None,
            no_dry_run: false,
        };
        let err = args.size().expect_err("no version file").to_string();
        assert!(err.contains("agent-files-v*"), "{err}");
        assert!(err.contains("--label"), "{err}");

        let args = SizeArgs {
            label: Some("- v0.1.0".to_string()),
            ..args
        };
        args.size().expect("a label given");
    }
}

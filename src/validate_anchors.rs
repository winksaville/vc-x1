//! The `validate-anchors` subcommand: check a markdown record's
//! own links, so a dead anchor is found by a command rather than
//! by a reader.
//!
//! Three checks, all same-file:
//!
//! - every `](#slug)` link and `[N]: #slug` definition resolves to
//!   a heading in the same file
//! - every `[[N]]` citation has an `[N]:` definition, and every
//!   definition is cited
//! - no two headings slug to the same anchor, which markdown
//!   silently disambiguates, so the second link lands on the first
//!   heading
//!
//! Cross-file targets (`](other.md#slug)`, `[N]: /notes/x.md#y`)
//! are recognized and skipped: resolving one means slugging another
//! file's headings, which is its own step (see the `## Todo` entry
//! **Reference defs: go file-relative, with anchors**).
//!
//! Fenced code and code spans are excluded throughout. A `[[N]]` in
//! a code span is a quoted identifier rather than a citation, and a
//! `#` line inside a fence is code rather than a heading.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info, warn};

use crate::context::Context;
use crate::subcommand::SubcommandRunner;

/// Clap-derived args for `validate-anchors`.
#[derive(Args, Debug)]
pub struct ValidateAnchorsArgs {
    /// Markdown files to check. Default: every tracked `.md` in
    /// the workspace, excluding the agent repo and build output.
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}

/// Inputs to the validate-anchors op, flat, owned, clap-free.
pub struct ValidateAnchorsParams {
    pub files: Vec<PathBuf>,
}

impl From<&ValidateAnchorsArgs> for ValidateAnchorsParams {
    /// Convert clap-derived args into the flat params (total).
    fn from(a: &ValidateAnchorsArgs) -> Self {
        Self {
            files: a.files.clone(),
        }
    }
}

impl SubcommandRunner for ValidateAnchorsArgs {
    type Params = ValidateAnchorsParams;

    /// Delegate to the `From<&ValidateAnchorsArgs>` impl (total).
    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(ValidateAnchorsParams::from(self))
    }

    /// Run the `validate_anchors` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        validate_anchors(ctx, params)
    }
}

/// One problem found in one file.
#[derive(Debug, PartialEq, Eq)]
pub struct Finding {
    /// 1-based line the problem is on, or 0 when the problem is
    /// the file's as a whole (an uncited definition names its own
    /// line, so every finding today has one).
    pub line: usize,
    pub message: String,
}

/// A heading's GitHub anchor.
///
/// The algorithm [Markdown anchor
/// links](../agent-data/notes.md#markdown-anchor-links) documents:
/// lowercase, drop every character that is not alphanumeric, a
/// hyphen, or an underscore, and map each remaining space to one
/// hyphen. Adjacent spaces are not collapsed, so `a + b` slugs to
/// `a--b` while `a: b` slugs to `a-b`.
pub fn anchor_slug(heading: &str) -> String {
    heading
        .chars()
        .filter_map(|c| match c {
            'a'..='z' | '0'..='9' | '-' | '_' => Some(c),
            'A'..='Z' => Some(c.to_ascii_lowercase()),
            ' ' => Some('-'),
            _ => None,
        })
        .collect()
}

/// Blank out code spans in one line, keeping its length so column
/// arithmetic still lines up.
///
/// Backticks pair left to right. An unpaired backtick opens a span
/// that runs to the end of the line, which is the reading that
/// errs toward ignoring text rather than inventing a citation in
/// it.
fn blank_code_spans(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_span = false;
    for c in line.chars() {
        if c == '`' {
            in_span = !in_span;
            out.push(' ');
        } else if in_span {
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    out
}

/// The file's lines with fenced blocks and code spans blanked, so
/// every later scan sees prose only.
fn prose_lines(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_fence = false;
    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            out.push(String::new());
            continue;
        }
        if in_fence {
            out.push(String::new());
            continue;
        }
        out.push(blank_code_spans(line));
    }
    out
}

/// Collect `(slug, line)` for every heading, in document order.
fn headings(lines: &[String]) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let Some(rest) = line.strip_prefix('#') else {
            continue;
        };
        let text = rest.trim_start_matches('#');
        // `#foo` is not a heading, `# foo` is.
        if !text.starts_with(' ') {
            continue;
        }
        out.push((anchor_slug(text.trim()), i + 1));
    }
    out
}

/// Every same-file anchor target: `](#slug)` inline links and
/// `[N]: #slug` definitions, as `(slug, line)`.
///
/// A target naming another file is skipped here, since resolving
/// it is the cross-file check this rung leaves out.
fn anchor_targets(lines: &[String]) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        for (idx, _) in line.match_indices("](#") {
            let rest = &line[idx + 3..];
            if let Some(end) = rest.find(')') {
                out.push((rest[..end].to_string(), i + 1));
            }
        }
        if let Some(rest) = ref_def(line)
            && let Some(slug) = rest.strip_prefix('#')
        {
            out.push((slug.trim().to_string(), i + 1));
        }
    }
    out
}

/// The target of a `[N]: <target>` definition line, if the line is
/// one.
fn ref_def(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix('[')?;
    let (num, rest) = rest.split_once(']')?;
    if num.is_empty() || !num.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(rest.strip_prefix(':')?.trim())
}

/// The reference number a `[N]: ...` definition line declares.
fn ref_def_number(line: &str) -> Option<u32> {
    let rest = line.trim_start().strip_prefix('[')?;
    let (num, rest) = rest.split_once(']')?;
    rest.strip_prefix(':')?;
    num.parse().ok()
}

/// Every reference number the line uses, in either of the two
/// forms that use one.
///
/// - `[[N]]`, the notes convention's citation, doubled so the
///   brackets render.
/// - `[text][N]`, a markdown reference link, which is how a ladder
///   rung names its own subsection.
///
/// Both are uses, so a definition either form reaches is cited.
fn citations(line: &str) -> Vec<u32> {
    let mut out = Vec::new();
    for (idx, _) in line.match_indices("[[") {
        let rest = &line[idx + 2..];
        if let Some(end) = rest.find("]]")
            && let Ok(n) = rest[..end].parse::<u32>()
        {
            out.push(n);
        }
    }
    for (idx, _) in line.match_indices("][") {
        let rest = &line[idx + 2..];
        if let Some(end) = rest.find(']')
            && let Ok(n) = rest[..end].parse::<u32>()
        {
            out.push(n);
        }
    }
    out
}

/// Check one markdown file's own links, returning every finding in
/// document order.
///
/// Pure: the caller reads the file and reports. See the module doc
/// for the three checks.
pub fn analyze(content: &str) -> Vec<Finding> {
    let lines = prose_lines(content);
    let mut findings: Vec<Finding> = Vec::new();

    let heads = headings(&lines);
    let slugs: BTreeSet<&str> = heads.iter().map(|(s, _)| s.as_str()).collect();

    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    for (slug, line) in &heads {
        if let Some(first) = seen.get(slug.as_str()) {
            findings.push(Finding {
                line: *line,
                message: format!(
                    "duplicate heading anchor '#{slug}', first at line {first}: a link to it \
                     reaches the first heading only"
                ),
            });
        } else {
            seen.insert(slug.as_str(), *line);
        }
    }

    for (slug, line) in anchor_targets(&lines) {
        if !slugs.contains(slug.as_str()) {
            findings.push(Finding {
                line,
                message: format!("'#{slug}' matches no heading in this file"),
            });
        }
    }

    let mut defined: BTreeMap<u32, usize> = BTreeMap::new();
    for (i, line) in lines.iter().enumerate() {
        if let Some(n) = ref_def_number(line) {
            defined.insert(n, i + 1);
        }
    }
    let mut cited: BTreeMap<u32, usize> = BTreeMap::new();
    for (i, line) in lines.iter().enumerate() {
        for n in citations(line) {
            cited.entry(n).or_insert(i + 1);
        }
    }
    for (n, line) in &cited {
        if !defined.contains_key(n) {
            findings.push(Finding {
                line: *line,
                message: format!("[[{n}]] is cited but never defined"),
            });
        }
    }
    for (n, line) in &defined {
        if !cited.contains_key(n) {
            findings.push(Finding {
                line: *line,
                message: format!("[{n}] is defined but never cited"),
            });
        }
    }

    findings.sort_by_key(|f| f.line);
    findings
}

/// Directories the default set never descends into.
///
/// Build output and the two repo internals hold no records, and
/// `.claude` is the agent repo, whose session data is a journal
/// rather than a record. `notes/chores` is frozen history, which
/// nothing edits, so a finding there names a file no commit may
/// touch ([Frozen
/// history](../agent-data/notes.md#frozen-history-chores-and-done)).
const SKIP_DIRS: &[&str] = &["target", ".git", ".jj", ".claude", "tmp", "chores"];

/// Files the default set skips for the same frozen-history reason
/// as `notes/chores`.
const SKIP_FILES: &[&str] = &["done.md"];

/// Markdown files to check when the caller named none: every `.md`
/// under `root` that a commit may still edit.
///
/// Naming a file explicitly checks it regardless, which is how the
/// frozen history is read when someone wants it.
fn default_files(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if path.is_dir() {
                if SKIP_DIRS.contains(&name.as_ref()) {
                    continue;
                }
                walk(&path, out);
            } else if name.ends_with(".md") && !SKIP_FILES.contains(&name.as_ref()) {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out.sort();
    out
}

/// Run the `validate-anchors` subcommand: check each file's own
/// links and report every finding, erroring when any was found.
///
/// `ctx` is unused: the check reads plain files and neither the
/// user config nor the `--log` path applies. It is present for the
/// uniform subcommand-layer signature.
pub fn validate_anchors(
    _ctx: &Context,
    params: &ValidateAnchorsParams,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("validate-anchors: enter");
    let files = if params.files.is_empty() {
        let root = crate::common::find_workspace_root()
            .ok_or("not inside a workspace: name the files to check")?;
        default_files(&root)
    } else {
        params.files.clone()
    };

    let mut total = 0usize;
    for file in &files {
        let content = std::fs::read_to_string(file)
            .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
        for finding in analyze(&content) {
            warn!("{}:{}: {}", file.display(), finding.line, finding.message);
            total += 1;
        }
    }

    let n = files.len();
    let word = if n == 1 { "file" } else { "files" };
    if total == 0 {
        info!("validate-anchors: {n} {word} checked, all links resolve");
        debug!("validate-anchors: exit");
        Ok(())
    } else {
        info!("validate-anchors: {n} {word} checked, {total} problem(s) found");
        debug!("validate-anchors: exit with findings");
        Err(format!("{total} anchor problem(s) found").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The documented algorithm, on the cases the rule names: a
    /// colon leaves one hyphen, a slash between spaces leaves two.
    #[test]
    fn slug_follows_the_documented_algorithm() {
        assert_eq!(anchor_slug("Cycle protocol"), "cycle-protocol");
        assert_eq!(anchor_slug("a: b"), "a-b");
        assert_eq!(anchor_slug("a + b"), "a--b");
        assert_eq!(
            anchor_slug("Conventional-commit shape (ladder / commit)"),
            "conventional-commit-shape-ladder--commit"
        );
    }

    #[test]
    fn resolving_links_are_clean() {
        let md = "# One\n\n## Two words\n\nSee [it](#two-words) and [[1]].\n\n[1]: #one\n";
        assert_eq!(analyze(md), vec![]);
    }

    #[test]
    fn dead_anchor_is_a_finding() {
        let md = "# One\n\nSee [it](#nope).\n";
        let f = analyze(md);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("'#nope'"), "{f:?}");
        assert_eq!(f[0].line, 3);
    }

    /// The shape our own `.vc-config.md` was in: citations with no
    /// definitions.
    #[test]
    fn undefined_citation_is_a_finding() {
        let md = "# One\n\nSee [[3]] and [[4]].\n\n[1]: #one\n\nAlso [[1]].\n";
        let found = analyze(md);
        let messages: Vec<&str> = found.iter().map(|f| f.message.as_str()).collect();
        assert!(messages.iter().any(|m| m.contains("[[3]] is cited")));
        assert!(messages.iter().any(|m| m.contains("[[4]] is cited")));
        assert_eq!(messages.len(), 2, "{messages:?}");
    }

    /// A ladder rung names its subsection with a reference link,
    /// `[title][N]`, which is a use like `[[N]]` is. Counting only
    /// the doubled form reported every rung's definition as
    /// uncited.
    #[test]
    fn reference_link_counts_as_a_citation() {
        let md = "# One\n\n- [a rung][1]\n\n## A rung\n\n[1]: #a-rung\n";
        assert_eq!(analyze(md), vec![]);
    }

    #[test]
    fn uncited_definition_is_a_finding() {
        let md = "# One\n\nNothing cites it.\n\n[9]: #one\n";
        let f = analyze(md);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("[9] is defined"), "{f:?}");
    }

    #[test]
    fn duplicate_heading_anchor_is_a_finding() {
        let md = "# Todo\n\n## Todo\n";
        let f = analyze(md);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("duplicate heading anchor"), "{f:?}");
    }

    /// A cross-file target is recognized and left alone: resolving
    /// it means slugging another file, which this check does not do.
    #[test]
    fn cross_file_targets_are_skipped() {
        let md = "# One\n\nSee [it](notes/other.md#whatever) and [[1]].\n\n[1]: other.md#thing\n";
        assert_eq!(analyze(md), vec![]);
    }

    /// A fenced specimen is code, so its `#` lines are not headings
    /// and its `[[N]]` are not citations.
    #[test]
    fn fenced_code_is_not_prose() {
        let md = "# One\n\n```\n## Not a heading\n[[7]]\n```\n";
        assert_eq!(analyze(md), vec![]);
    }

    /// A `[[N]]` in a code span is a quoted identifier, which
    /// notes.md says is data rather than a citation.
    #[test]
    fn code_span_citation_is_not_a_citation() {
        let md = "# One\n\nThe token `[[7]]` is literal text.\n";
        assert_eq!(analyze(md), vec![]);
    }
}

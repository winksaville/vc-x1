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
use log::{debug, info, trace, warn};

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

/// One anchor target found in the prose.
struct Target {
    slug: String,
    line: usize,
    /// The target named another file, so its slug belongs to that
    /// file's headings and this check does not resolve it.
    cross_file: bool,
}

/// Every anchor target: `](...#slug)` inline links and
/// `[N]: ...#slug` definitions, same-file and cross-file alike.
///
/// Cross-file ones are collected rather than dropped so the report
/// can say how many links it did not resolve, which is exactly the
/// coverage this check lacks until the cross-file half lands.
fn anchor_targets(lines: &[String]) -> Vec<Target> {
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        for (idx, _) in line.match_indices("](") {
            let rest = &line[idx + 2..];
            let Some(end) = rest.find(')') else {
                continue;
            };
            let dest = &rest[..end];
            if let Some(slug) = dest.strip_prefix('#') {
                out.push(Target {
                    slug: slug.to_string(),
                    line: i + 1,
                    cross_file: false,
                });
            } else if let Some((_file, slug)) = dest.split_once('#') {
                out.push(Target {
                    slug: slug.to_string(),
                    line: i + 1,
                    cross_file: true,
                });
            }
        }
        if let Some(rest) = ref_def(line) {
            if let Some(slug) = rest.strip_prefix('#') {
                out.push(Target {
                    slug: slug.trim().to_string(),
                    line: i + 1,
                    cross_file: false,
                });
            } else if let Some((_file, slug)) = rest.split_once('#') {
                out.push(Target {
                    slug: slug.trim().to_string(),
                    line: i + 1,
                    cross_file: true,
                });
            }
        }
    }
    out
}

/// Levenshtein distance, for naming the slug a failing target most
/// likely meant.
fn edit_distance(a: &str, b: &str) -> usize {
    let b_chars: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b_chars.len()).collect();
    let mut cur = vec![0usize; b_chars.len() + 1];
    for (i, ca) in a.chars().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b_chars.iter().enumerate() {
            let cost = usize::from(ca != *cb);
            cur[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b_chars.len()]
}

/// The heading slug a failing target most likely meant, when one is
/// close enough to be worth naming.
///
/// The threshold scales with the slug's length, so a short anchor
/// needs a near-exact neighbour while a long one tolerates a
/// rename of one word. Nothing is suggested when the nearest is
/// further than that, since a wrong suggestion costs more than
/// none.
fn nearest_slug<'a>(slug: &str, slugs: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let limit = (slug.len() / 3).max(2);
    slugs
        .map(|s| (edit_distance(slug, s), s))
        .filter(|(d, _)| *d <= limit)
        .min_by_key(|(d, _)| *d)
        .map(|(_, s)| s)
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

/// What one file's check looked at, beside what it found.
///
/// The counts exist so a pass says how much it covered: a file with
/// no links passed identically to one with four hundred while the
/// summary counted files.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub headings: usize,
    /// Same-file anchor targets resolved against those headings.
    pub anchors_checked: usize,
    pub anchors_failed: usize,
    /// Targets naming another file, which this check does not
    /// resolve. The coverage gap, reported rather than hidden.
    pub anchors_cross_file: usize,
    pub refs_defined: usize,
    pub refs_cited: usize,
}

/// One thing the check looked at, for `-vv`.
#[derive(Debug, PartialEq, Eq)]
pub struct Site {
    pub line: usize,
    pub detail: String,
}

/// One file's whole result: what was found, what was looked at, and
/// how much of each.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub findings: Vec<Finding>,
    pub sites: Vec<Site>,
    pub counts: Counts,
}

/// Check one markdown file's own links, returning every finding in
/// document order beside the tally of what was checked.
///
/// Pure: the caller reads the file and reports. See the module doc
/// for the three checks.
pub fn analyze(content: &str) -> Report {
    let lines = prose_lines(content);
    let mut findings: Vec<Finding> = Vec::new();
    let mut sites: Vec<Site> = Vec::new();
    let mut counts = Counts::default();

    let heads = headings(&lines);
    let slugs: BTreeSet<&str> = heads.iter().map(|(s, _)| s.as_str()).collect();
    counts.headings = heads.len();

    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    for (slug, line) in &heads {
        sites.push(Site {
            line: *line,
            detail: format!("heading slugs to '#{slug}'"),
        });
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

    for target in anchor_targets(&lines) {
        let Target {
            slug,
            line,
            cross_file,
        } = target;
        if cross_file {
            counts.anchors_cross_file += 1;
            sites.push(Site {
                line,
                detail: format!("'#{slug}' names another file, not resolved"),
            });
            continue;
        }
        counts.anchors_checked += 1;
        if slugs.contains(slug.as_str()) {
            sites.push(Site {
                line,
                detail: format!("'#{slug}' resolves"),
            });
            continue;
        }
        counts.anchors_failed += 1;
        let message = match nearest_slug(&slug, slugs.iter().copied()) {
            Some(near) => {
                format!("'#{slug}' matches no heading in this file. Did you mean '#{near}'?")
            }
            None => format!("'#{slug}' matches no heading in this file"),
        };
        sites.push(Site {
            line,
            detail: format!("'#{slug}' does not resolve"),
        });
        findings.push(Finding { line, message });
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
    counts.refs_defined = defined.len();
    counts.refs_cited = cited.len();
    for (n, line) in &cited {
        if defined.contains_key(n) {
            sites.push(Site {
                line: *line,
                detail: format!("[[{n}]] has a definition"),
            });
            continue;
        }
        findings.push(Finding {
            line: *line,
            message: format!("[[{n}]] is cited but never defined"),
        });
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
    sites.sort_by_key(|s| s.line);
    Report {
        findings,
        sites,
        counts,
    }
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

    let mut total = Counts::default();
    let mut findings = 0usize;
    for file in &files {
        let content = std::fs::read_to_string(file)
            .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
        let report = analyze(&content);
        for site in &report.sites {
            trace!("{}:{}: {}", file.display(), site.line, site.detail);
        }
        for finding in &report.findings {
            warn!("{}:{}: {}", file.display(), finding.line, finding.message);
        }
        let c = &report.counts;
        debug!(
            "{}: {} heading(s), {} link(s) checked, {} failed, {} cross-file, {} ref(s) defined, \
             {} cited",
            file.display(),
            c.headings,
            c.anchors_checked,
            c.anchors_failed,
            c.anchors_cross_file,
            c.refs_defined,
            c.refs_cited
        );
        findings += report.findings.len();
        total.headings += c.headings;
        total.anchors_checked += c.anchors_checked;
        total.anchors_failed += c.anchors_failed;
        total.anchors_cross_file += c.anchors_cross_file;
        total.refs_defined += c.refs_defined;
        total.refs_cited += c.refs_cited;
    }

    let n = files.len();
    let word = if n == 1 { "file" } else { "files" };
    // The link count is the summary's point: it separates a pass
    // that checked something from one that had nothing to check.
    let checked = total.anchors_checked + total.refs_cited;
    info!(
        "validate-anchors: {n} {word}, {checked} link(s) checked, {} failed, {} cross-file \
         skipped, {} heading(s)",
        findings, total.anchors_cross_file, total.headings
    );
    if findings == 0 {
        debug!("validate-anchors: exit");
        Ok(())
    } else {
        debug!("validate-anchors: exit with findings");
        Err(format!("{findings} anchor problem(s) found").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The findings alone, which is what most of these assert on.
    fn findings(md: &str) -> Vec<Finding> {
        analyze(md).findings
    }

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
        assert_eq!(findings(md), vec![]);
    }

    #[test]
    fn dead_anchor_is_a_finding() {
        let md = "# One\n\nSee [it](#nope).\n";
        let f = findings(md);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("'#nope'"), "{f:?}");
        assert_eq!(f[0].line, 3);
    }

    /// The shape our own `.vc-config.md` was in: citations with no
    /// definitions.
    #[test]
    fn undefined_citation_is_a_finding() {
        let md = "# One\n\nSee [[3]] and [[4]].\n\n[1]: #one\n\nAlso [[1]].\n";
        let found = findings(md);
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
        assert_eq!(findings(md), vec![]);
    }

    /// Counted rather than dropped, so the summary can say how much
    /// of the file it did not resolve.
    #[test]
    fn cross_file_targets_are_counted() {
        let md = "# One\n\nSee [it](notes/other.md#whatever) and [[1]].\n\n[1]: other.md#thing\n";
        let c = analyze(md).counts;
        assert_eq!(c.anchors_cross_file, 2);
        assert_eq!(c.anchors_checked, 0);
    }

    /// The counts are what separate a pass that checked something
    /// from one that had nothing to check.
    #[test]
    fn counts_describe_what_was_checked() {
        let md = "# One\n\n## Two words\n\nSee [it](#two-words) and [[1]].\n\n[1]: #one\n";
        let c = analyze(md).counts;
        assert_eq!(c.headings, 2);
        assert_eq!(
            c.anchors_checked, 2,
            "the inline link and the [1] definition"
        );
        assert_eq!(c.anchors_failed, 0);
        assert_eq!(c.refs_defined, 1);
        assert_eq!(c.refs_cited, 1);
    }

    /// A file with nothing to check reports zero rather than
    /// passing as though it had checked something.
    #[test]
    fn a_file_with_no_links_counts_none() {
        let c = analyze("# One\n\nProse only.\n").counts;
        assert_eq!(c.anchors_checked, 0);
        assert_eq!(c.refs_cited, 0);
        assert_eq!(c.headings, 1);
    }

    /// A near miss names the heading it probably meant, which is the
    /// fix the reader would otherwise open the file to find.
    #[test]
    fn a_near_miss_suggests_the_nearest_heading() {
        let md = "# One\n\n## Todo format\n\nSee [it](#todo-formats).\n";
        let f = findings(md);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(
            f[0].message.contains("Did you mean '#todo-format'?"),
            "{f:?}"
        );
    }

    /// Nothing is suggested when nothing is close, since a wrong
    /// suggestion costs more than none.
    #[test]
    fn a_far_miss_suggests_nothing() {
        let md = "# One\n\n## Todo format\n\nSee [it](#completely-different).\n";
        let f = findings(md);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(!f[0].message.contains("Did you mean"), "{f:?}");
    }

    /// Every heading and every resolved link is a site, which is
    /// what `-vv` prints.
    #[test]
    fn sites_cover_headings_and_resolved_links() {
        let md = "# One\n\n## Two words\n\nSee [it](#two-words).\n";
        let details: Vec<String> = analyze(md).sites.into_iter().map(|s| s.detail).collect();
        assert!(
            details
                .iter()
                .any(|d| d.contains("heading slugs to '#one'"))
        );
        assert!(details.iter().any(|d| d.contains("'#two-words' resolves")));
    }

    #[test]
    fn uncited_definition_is_a_finding() {
        let md = "# One\n\nNothing cites it.\n\n[9]: #one\n";
        let f = findings(md);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("[9] is defined"), "{f:?}");
    }

    #[test]
    fn duplicate_heading_anchor_is_a_finding() {
        let md = "# Todo\n\n## Todo\n";
        let f = findings(md);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("duplicate heading anchor"), "{f:?}");
    }

    /// A cross-file target is recognized and left alone: resolving
    /// it means slugging another file, which this check does not do.
    #[test]
    fn cross_file_targets_are_skipped() {
        let md = "# One\n\nSee [it](notes/other.md#whatever) and [[1]].\n\n[1]: other.md#thing\n";
        assert_eq!(findings(md), vec![]);
    }

    /// A fenced specimen is code, so its `#` lines are not headings
    /// and its `[[N]]` are not citations.
    #[test]
    fn fenced_code_is_not_prose() {
        let md = "# One\n\n```\n## Not a heading\n[[7]]\n```\n";
        assert_eq!(findings(md), vec![]);
    }

    /// A `[[N]]` in a code span is a quoted identifier, which
    /// notes.md says is data rather than a citation.
    #[test]
    fn code_span_citation_is_not_a_citation() {
        let md = "# One\n\nThe token `[[7]]` is literal text.\n";
        assert_eq!(findings(md), vec![]);
    }
}

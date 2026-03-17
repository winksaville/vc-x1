use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use jj_lib::backend::CommitId;
use jj_lib::commit::Commit;
use jj_lib::config::StackedConfig;
use jj_lib::fileset::FilesetAliasesMap;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::{ReadonlyRepo, Repo, StoreFactories};
use jj_lib::repo_path::RepoPathUiConverter;
use jj_lib::revset::{
    RevsetAliasesMap, RevsetDiagnostics, RevsetExtensions, RevsetParseContext,
    RevsetWorkspaceContext, SymbolResolver,
};
use jj_lib::settings::UserSettings;
use jj_lib::workspace::{Workspace, default_working_copy_factories};
use pollster::FutureExt;

/// Direction(s) indicated by `..` notation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DotDirection {
    /// No dots: just the revision itself.
    None,
    /// `x..` — ancestors (x at top, older below).
    Ancestors,
    /// `..x` — descendants (newer above, x at bottom).
    Descendants,
    /// `..x..` — both directions, x in the middle.
    Both,
}

/// Result of parsing positional `..` notation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DotSpec {
    /// The bare revision (without `..`).
    pub rev: String,
    /// Which direction(s) the dots indicate.
    pub direction: DotDirection,
    /// Count per dotted side (x is not counted).
    pub count: usize,
}

/// Parse a positional REV string for `..` notation.
///
/// Returns the bare revision, direction, and default count (0).
/// Count is set separately from the second positional arg.
pub fn parse_dot_rev(rev: &str) -> DotSpec {
    if let Some(inner) = rev.strip_prefix("..").and_then(|s| s.strip_suffix("..")) {
        DotSpec {
            rev: inner.to_string(),
            direction: DotDirection::Both,
            count: 0,
        }
    } else if let Some(inner) = rev.strip_prefix("..") {
        DotSpec {
            rev: inner.to_string(),
            direction: DotDirection::Descendants,
            count: 0,
        }
    } else if let Some(inner) = rev.strip_suffix("..") {
        DotSpec {
            rev: inner.to_string(),
            direction: DotDirection::Ancestors,
            count: 0,
        }
    } else {
        DotSpec {
            rev: rev.to_string(),
            direction: DotDirection::None,
            count: 0,
        }
    }
}

/// Resolve effective revision, limit, and whether results need reversing
/// from positional args combined with named flags.
///
/// Named flags (`-r`, `-l`) take precedence over positional args.
pub struct ResolvedArgs {
    /// The jj revset string to evaluate.
    pub revset: String,
    /// Maximum number of commits to return (None = unlimited).
    pub limit: Option<usize>,
    /// For `..x..`, the count per side.
    pub both_count: Option<usize>,
    /// The bare primary revision (for highlighting).
    pub primary_rev: String,
}

/// Resolve positional `..` args with named flag overrides.
///
/// - `pos_rev`: first positional arg (may contain `..`)
/// - `pos_count`: second positional arg
/// - `flag_rev`: `-r` flag value (if different from default)
/// - `flag_limit`: `-l` flag value
/// - `default_rev`: the default revision (e.g. "@")
pub fn resolve_dot_args(
    pos_rev: Option<&str>,
    pos_count: Option<usize>,
    flag_rev: &str,
    flag_limit: Option<usize>,
    default_rev: &str,
) -> ResolvedArgs {
    // If named flags were explicitly set, they take full precedence
    let flag_rev_set = flag_rev != default_rev;
    let flag_limit_set = flag_limit.is_some();

    if flag_rev_set || (flag_limit_set && pos_rev.is_none()) {
        return ResolvedArgs {
            revset: flag_rev.to_string(),
            limit: flag_limit,
            both_count: None,
            primary_rev: flag_rev.to_string(),
        };
    }

    // Use positional args
    let rev_str = pos_rev.unwrap_or(default_rev);
    let mut spec = parse_dot_rev(rev_str);

    // If there's a positional count, set it
    if let Some(c) = pos_count {
        spec.count = c;
    }

    // Named limit overrides positional count
    if flag_limit_set {
        spec.count = flag_limit.unwrap();
    }

    let primary = spec.rev.clone();

    match spec.direction {
        DotDirection::None => {
            if spec.count > 0 {
                // bare `x 5` → `x.. 5` (ancestors)
                ResolvedArgs {
                    revset: format!("::{}", spec.rev),
                    limit: Some(spec.count + 1), // +1 for x itself
                    both_count: None,
                    primary_rev: primary,
                }
            } else {
                ResolvedArgs {
                    revset: spec.rev,
                    limit: Some(1),
                    both_count: None,
                    primary_rev: primary,
                }
            }
        }
        DotDirection::Ancestors => {
            if spec.count > 0 {
                ResolvedArgs {
                    revset: format!("::{}", spec.rev),
                    limit: Some(spec.count + 1),
                    both_count: None,
                    primary_rev: primary,
                }
            } else {
                ResolvedArgs {
                    revset: format!("::{}", spec.rev),
                    limit: None,
                    both_count: None,
                    primary_rev: primary,
                }
            }
        }
        DotDirection::Descendants => {
            if spec.count > 0 {
                ResolvedArgs {
                    revset: format!("{}::", spec.rev),
                    limit: Some(spec.count + 1),
                    both_count: None,
                    primary_rev: primary,
                }
            } else {
                ResolvedArgs {
                    revset: format!("{}::", spec.rev),
                    limit: None,
                    both_count: None,
                    primary_rev: primary,
                }
            }
        }
        DotDirection::Both => ResolvedArgs {
            revset: format!("::{}|{}::", spec.rev, spec.rev),
            limit: None,
            both_count: Some(spec.count),
            primary_rev: primary,
        },
    }
}

/// Wrap text in ANSI bold escape codes.
pub fn bold(s: &str) -> String {
    format!("\x1b[1m{s}\x1b[0m")
}

/// Bold only the first line of a multi-line string.
pub fn bold_first_line(s: &str) -> String {
    if let Some(pos) = s.find('\n') {
        format!("\x1b[1m{}\x1b[0m{}", &s[..pos], &s[pos..])
    } else {
        bold(s)
    }
}

/// Format: just the short changeID.
pub fn format_chid(commit: &Commit) -> String {
    let change_hex = encode_reverse_hex(commit.change_id().as_bytes());
    change_hex[..change_hex.len().min(12)].to_string()
}

/// Format: changeID commitID first-line (single line).
pub fn format_commit_short(commit: &Commit) -> String {
    let change_hex = encode_reverse_hex(commit.change_id().as_bytes());
    let change_short = &change_hex[..change_hex.len().min(12)];
    let commit_hex = commit.id().hex();
    let commit_short = &commit_hex[..commit_hex.len().min(12)];
    let first_line = commit.description().lines().next().unwrap_or("");
    if first_line.is_empty() {
        format!("{change_short} {commit_short} (no description set)")
    } else {
        format!("{change_short} {commit_short} {first_line}")
    }
}

/// Format: changeID commitID first-line, then remaining description lines.
pub fn format_commit_full(commit: &Commit) -> String {
    let change_hex = encode_reverse_hex(commit.change_id().as_bytes());
    let change_short = &change_hex[..change_hex.len().min(12)];
    let commit_hex = commit.id().hex();
    let commit_short = &commit_hex[..commit_hex.len().min(12)];
    let desc = commit.description();
    if desc.is_empty() {
        format!("{change_short} {commit_short} (no description set)")
    } else {
        let mut lines = desc.lines();
        let first_line = lines.next().unwrap_or("");
        let mut result = format!("{change_short} {commit_short} {first_line}");
        for line in lines {
            result.push('\n');
            result.push_str(line);
        }
        result
    }
}

/// Collect formatted lines for `..x..` (both directions).
///
/// `formatter` converts a `Commit` into a display string.
/// Returns `(lines, anchor_index)` — the lines in display order
/// (descendants newest first, anchor, ancestors newest first)
/// and the index of the anchor commit within the lines.
pub fn collect_both<F>(
    repo: &Arc<ReadonlyRepo>,
    repo_path: &Path,
    pos_rev: Option<&str>,
    both_count: usize,
    formatter: F,
) -> Result<(Vec<String>, usize), Box<dyn std::error::Error>>
where
    F: Fn(&Commit) -> String,
{
    let bare_rev = pos_rev.unwrap_or("@");
    let spec = parse_dot_rev(bare_rev);
    let (workspace, _) = load_repo(repo_path)?;
    let anchor_ids = resolve_revset(&workspace, repo, &spec.rev)?;
    if anchor_ids.is_empty() {
        return Err(format!("no commit found for revision '{}'", spec.rev).into());
    }
    let anchor_id = &anchor_ids[0];
    let root_commit_id = repo.store().root_commit_id().clone();

    let ancestor_ids = resolve_revset(&workspace, repo, &format!("::{}", spec.rev))?;
    let descendant_ids = resolve_revset(&workspace, repo, &format!("{}::", spec.rev))?;

    // Descendants (newest first from jj), excluding anchor
    let mut desc_lines: Vec<String> = Vec::new();
    for commit_id in &descendant_ids {
        if *commit_id == root_commit_id || *commit_id == *anchor_id {
            continue;
        }
        let commit = repo.store().get_commit(commit_id)?;
        desc_lines.push(formatter(&commit));
    }
    if both_count > 0 {
        let start = desc_lines.len().saturating_sub(both_count);
        desc_lines = desc_lines[start..].to_vec();
    } else {
        desc_lines.clear();
    }

    // Anchor
    let anchor_commit = repo.store().get_commit(anchor_id)?;

    // Ancestors (newest first from jj), excluding anchor
    let mut anc_lines: Vec<String> = Vec::new();
    if both_count > 0 {
        let mut count = 0;
        for commit_id in &ancestor_ids {
            if root_commit_id == *commit_id || *commit_id == *anchor_id {
                continue;
            }
            let commit = repo.store().get_commit(commit_id)?;
            anc_lines.push(formatter(&commit));
            count += 1;
            if count >= both_count {
                break;
            }
        }
    }

    let anchor_index = desc_lines.len();
    let mut result = desc_lines;
    result.push(formatter(&anchor_commit));
    result.extend(anc_lines);
    Ok((result, anchor_index))
}

pub fn load_repo(
    path: &Path,
) -> Result<(Workspace, Arc<ReadonlyRepo>), Box<dyn std::error::Error>> {
    let config = StackedConfig::with_defaults();
    let settings = UserSettings::from_config(config)?;
    let store_factories = StoreFactories::default();
    let working_copy_factories = default_working_copy_factories();

    let workspace = Workspace::load(&settings, path, &store_factories, &working_copy_factories)?;
    let repo = workspace.repo_loader().load_at_head().block_on()?;

    Ok((workspace, repo))
}

/// Parse and evaluate a revset string, returning commit IDs.
pub fn resolve_revset(
    workspace: &Workspace,
    repo: &Arc<ReadonlyRepo>,
    revset_str: &str,
) -> Result<Vec<CommitId>, Box<dyn std::error::Error>> {
    let aliases_map = RevsetAliasesMap::new();
    let fileset_aliases_map = FilesetAliasesMap::new();
    let extensions = RevsetExtensions::default();
    let workspace_root = workspace.workspace_root().to_path_buf();
    let path_converter = RepoPathUiConverter::Fs {
        cwd: workspace_root.clone(),
        base: workspace_root,
    };
    let workspace_name = workspace.workspace_name();
    let workspace_ctx = RevsetWorkspaceContext {
        path_converter: &path_converter,
        workspace_name,
    };
    let context = RevsetParseContext {
        aliases_map: &aliases_map,
        local_variables: HashMap::new(),
        user_email: "",
        date_pattern_context: chrono::Utc::now().fixed_offset().into(),
        default_ignored_remote: None,
        fileset_aliases_map: &fileset_aliases_map,
        use_glob_by_default: false,
        extensions: &extensions,
        workspace: Some(workspace_ctx),
    };

    let mut diagnostics = RevsetDiagnostics::new();
    let expression = jj_lib::revset::parse(&mut diagnostics, revset_str, &context)?;

    let no_extensions: &[Box<dyn jj_lib::revset::SymbolResolverExtension>] = &[];
    let symbol_resolver = SymbolResolver::new(repo.as_ref(), no_extensions);
    let resolved = expression.resolve_user_expression(repo.as_ref(), &symbol_resolver)?;
    let revset = resolved.evaluate(repo.as_ref())?;

    let mut commit_ids = Vec::new();
    for result in revset.iter() {
        commit_ids.push(result?);
    }
    Ok(commit_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dot_rev_bare() {
        let spec = parse_dot_rev("@");
        assert_eq!(spec.rev, "@");
        assert_eq!(spec.direction, DotDirection::None);
    }

    #[test]
    fn parse_dot_rev_ancestors() {
        let spec = parse_dot_rev("@..");
        assert_eq!(spec.rev, "@");
        assert_eq!(spec.direction, DotDirection::Ancestors);
    }

    #[test]
    fn parse_dot_rev_descendants() {
        let spec = parse_dot_rev("..@");
        assert_eq!(spec.rev, "@");
        assert_eq!(spec.direction, DotDirection::Descendants);
    }

    #[test]
    fn parse_dot_rev_both() {
        let spec = parse_dot_rev("..@..");
        assert_eq!(spec.rev, "@");
        assert_eq!(spec.direction, DotDirection::Both);
    }

    #[test]
    fn parse_dot_rev_changeid() {
        let spec = parse_dot_rev("abcd..");
        assert_eq!(spec.rev, "abcd");
        assert_eq!(spec.direction, DotDirection::Ancestors);
    }

    #[test]
    fn parse_dot_rev_both_changeid() {
        let spec = parse_dot_rev("..abcd..");
        assert_eq!(spec.rev, "abcd");
        assert_eq!(spec.direction, DotDirection::Both);
    }

    #[test]
    fn resolve_defaults() {
        let r = resolve_dot_args(None, None, "@", None, "@");
        assert_eq!(r.revset, "@");
        assert_eq!(r.limit, Some(1));
        assert!(r.both_count.is_none());
        assert_eq!(r.primary_rev, "@");
    }

    #[test]
    fn resolve_bare_rev_with_count() {
        let r = resolve_dot_args(Some("@"), Some(5), "@", None, "@");
        assert_eq!(r.revset, "::@");
        assert_eq!(r.limit, Some(6)); // 5 + 1 for x
    }

    #[test]
    fn resolve_ancestors() {
        let r = resolve_dot_args(Some("@.."), Some(3), "@", None, "@");
        assert_eq!(r.revset, "::@");
        assert_eq!(r.limit, Some(4)); // 3 + 1
    }

    #[test]
    fn resolve_descendants() {
        let r = resolve_dot_args(Some("..@"), Some(3), "@", None, "@");
        assert_eq!(r.revset, "@::");
        assert_eq!(r.limit, Some(4)); // 3 + 1
    }

    #[test]
    fn resolve_both() {
        let r = resolve_dot_args(Some("..@.."), Some(3), "@", None, "@");
        assert_eq!(r.revset, "::@|@::");
        assert!(r.limit.is_none());
        assert_eq!(r.both_count, Some(3));
    }

    #[test]
    fn resolve_flag_overrides_positional() {
        let r = resolve_dot_args(Some("@.."), Some(5), "@-", None, "@");
        assert_eq!(r.revset, "@-"); // flag_rev takes precedence
    }

    #[test]
    fn resolve_ancestors_no_count() {
        let r = resolve_dot_args(Some("@.."), None, "@", None, "@");
        assert_eq!(r.revset, "::@");
        assert!(r.limit.is_none()); // no count, no limit
    }

    #[test]
    fn resolve_flag_limit_overrides_pos_count() {
        let r = resolve_dot_args(Some("@.."), Some(5), "@", Some(3), "@");
        assert_eq!(r.revset, "::@");
        assert_eq!(r.limit, Some(4)); // flag limit 3 + 1
    }
}

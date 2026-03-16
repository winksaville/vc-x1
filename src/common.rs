use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use jj_lib::backend::CommitId;
use jj_lib::config::StackedConfig;
use jj_lib::fileset::FilesetAliasesMap;
use jj_lib::repo::{ReadonlyRepo, StoreFactories};
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
    /// Whether to reverse the results (for descendants/both).
    pub reverse: bool,
    /// For `..x..`, the count per side.
    pub both_count: Option<usize>,
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
            reverse: false,
            both_count: None,
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

    match spec.direction {
        DotDirection::None => {
            if spec.count > 0 {
                // bare `x 5` → `x.. 5` (ancestors)
                ResolvedArgs {
                    revset: format!("::{}", spec.rev),
                    limit: Some(spec.count + 1), // +1 for x itself
                    reverse: false,
                    both_count: None,
                }
            } else {
                ResolvedArgs {
                    revset: spec.rev,
                    limit: Some(1),
                    reverse: false,
                    both_count: None,
                }
            }
        }
        DotDirection::Ancestors => {
            if spec.count > 0 {
                ResolvedArgs {
                    revset: format!("::{}", spec.rev),
                    limit: Some(spec.count + 1),
                    reverse: false,
                    both_count: None,
                }
            } else {
                ResolvedArgs {
                    revset: format!("::{}", spec.rev),
                    limit: None,
                    reverse: false,
                    both_count: None,
                }
            }
        }
        DotDirection::Descendants => {
            if spec.count > 0 {
                ResolvedArgs {
                    revset: format!("{}::", spec.rev),
                    limit: Some(spec.count + 1),
                    reverse: false,
                    both_count: None,
                }
            } else {
                ResolvedArgs {
                    revset: format!("{}::", spec.rev),
                    limit: None,
                    reverse: false,
                    both_count: None,
                }
            }
        }
        DotDirection::Both => ResolvedArgs {
            revset: format!("::{}|{}::", spec.rev, spec.rev),
            limit: None,
            reverse: false,
            both_count: Some(spec.count),
        },
    }
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
        assert!(!r.reverse);
        assert!(r.both_count.is_none());
    }

    #[test]
    fn resolve_bare_rev_with_count() {
        let r = resolve_dot_args(Some("@"), Some(5), "@", None, "@");
        assert_eq!(r.revset, "::@");
        assert_eq!(r.limit, Some(6)); // 5 + 1 for x
        assert!(!r.reverse);
    }

    #[test]
    fn resolve_ancestors() {
        let r = resolve_dot_args(Some("@.."), Some(3), "@", None, "@");
        assert_eq!(r.revset, "::@");
        assert_eq!(r.limit, Some(4)); // 3 + 1
        assert!(!r.reverse);
    }

    #[test]
    fn resolve_descendants() {
        let r = resolve_dot_args(Some("..@"), Some(3), "@", None, "@");
        assert_eq!(r.revset, "@::");
        assert_eq!(r.limit, Some(4)); // 3 + 1
        assert!(!r.reverse);
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

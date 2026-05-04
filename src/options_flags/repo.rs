//! `--repo` — pick a repo target via the user-config
//! account chain. See [options_flags](README.md) for shared
//! architecture.

use clap::Args;

use crate::config::RepoSelector;

/// Parse the `--repo` value into a `config::RepoSelector`.
///
/// - `<cat>` → `RepoSelector { category, value: None }`.
/// - `<cat>=<val>` → `RepoSelector { category, value: Some(val) }`.
/// - Empty input, empty category, or empty value (after `=`)
///   errors.
pub fn parse_repo_arg(s: &str) -> Result<RepoSelector, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("--repo: value is empty".into());
    }
    match s.split_once('=') {
        Some((cat, val)) => {
            let cat = cat.trim();
            let val = val.trim();
            if cat.is_empty() {
                return Err(format!("--repo: missing category in '{s}'"));
            }
            if val.is_empty() {
                return Err(format!("--repo: empty value in '{s}'"));
            }
            Ok(RepoSelector {
                category: cat.to_string(),
                value: Some(val.to_string()),
            })
        }
        None => Ok(RepoSelector {
            category: s.to_string(),
            value: None,
        }),
    }
}

/// `FlagParser` impl for `--repo`. Documentation-level —
/// consumers can use either `parse_repo_arg` directly or
/// `RepoParser::parse`.
pub struct RepoParser;

impl super::FlagParser for RepoParser {
    type Value = RepoSelector;

    fn parse(s: &str) -> Result<Self::Value, String> {
        parse_repo_arg(s)
    }
}

/// `--repo` leaf — see
/// [Consuming an OF](README.md#consuming-an-of).
#[derive(Args, Debug, Clone, Default)]
pub struct RepoFlag {
    /// Repo target — `<cat>` or `<cat>=<val>`.
    ///
    /// - `<cat>` looks up the value via the account chain
    ///   in the user config.
    /// - `<cat>=<val>` uses the literal value, no config
    ///   lookup needed.
    /// - Specific category meanings (`remote`, `local`,
    ///   …) depend on the consumer subcommand.
    #[arg(
        long,
        value_name = "CAT[=VAL]",
        value_parser = parse_repo_arg,
        verbatim_doc_comment
    )]
    pub repo: Option<RepoSelector>,
}

impl super::FlagBundle for RepoFlag {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_arg_category_only() {
        let sel = parse_repo_arg("remote").unwrap();
        assert_eq!(sel.category, "remote");
        assert_eq!(sel.value, None);
    }

    #[test]
    fn repo_arg_category_and_value() {
        let sel = parse_repo_arg("local=/tmp/fixtures").unwrap();
        assert_eq!(sel.category, "local");
        assert_eq!(sel.value.as_deref(), Some("/tmp/fixtures"));
    }

    #[test]
    fn repo_arg_trims_whitespace() {
        let sel = parse_repo_arg("  remote = git@github.com:foo  ").unwrap();
        assert_eq!(sel.category, "remote");
        assert_eq!(sel.value.as_deref(), Some("git@github.com:foo"));
    }

    #[test]
    fn repo_arg_empty_errors() {
        let err = parse_repo_arg("").unwrap_err();
        assert!(err.contains("empty"), "got: {err}");
    }

    #[test]
    fn repo_arg_missing_category_errors() {
        let err = parse_repo_arg("=foo").unwrap_err();
        assert!(err.contains("missing category"), "got: {err}");
    }

    #[test]
    fn repo_arg_empty_value_errors() {
        let err = parse_repo_arg("remote=").unwrap_err();
        assert!(err.contains("empty value"), "got: {err}");
    }
}

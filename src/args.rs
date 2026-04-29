//! Shared CLI parameter types and value parsers used across
//! multiple subcommands.
//!
//! - `ScopeKind` — typed value of `--scope` for init and clone
//!   (and any future subcommand that wants the same
//!   `code,bot|por` choice).
//! - `parse_scope_kind` — `value_parser` for the `--scope`
//!   field; one error wording shared across subcommands.
//! - `parse_repo_arg` — `value_parser` for the `--repo
//!   <cat>[=<val>]` field; produces `config::RepoSelector`.
//!
//! Per-subcommand `#[derive(Args)]` structs stay with their
//! subcommand because `#[arg(...)]` doc-comments drive
//! subcommand-specific `--help` text. Only the cross-cutting
//! types and parsers live here.
//!
//! TODO: the 0.42.0 sum-type cycle (`--scope=...|<path>` everywhere)
//! will likely extend `ScopeKind` with `Single(_)` / path-form
//! variants and pull more parsers in. See `notes/todo.md` and
//! `notes/chores-06.md > Generalize --scope`.

use crate::config::RepoSelector;

/// Typed value of `--scope` — the kinds of repo set a
/// subcommand can target.
///
/// - `CodeBot` (default) — dual-repo: code + `.claude` bot
///   session.
/// - `Por` — single repo (Plain Old Repo); no `.claude/`, no
///   `.vc-config.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    CodeBot,
    Por,
}

/// Parse the `--scope` value into a `ScopeKind`.
///
/// - Accepts `code,bot` / `bot,code` (commutative) and `por`.
/// - Standalone `code` or `bot` errors — these are
///   config-lookup keywords; subcommands using `ScopeKind`
///   have no config-driven sides to look up against. Use `por`
///   for single-repo or `code,bot` for dual.
pub fn parse_scope_kind(s: &str) -> Result<ScopeKind, String> {
    match s {
        "code,bot" | "bot,code" => Ok(ScopeKind::CodeBot),
        "por" => Ok(ScopeKind::Por),
        "code" | "bot" => Err(format!(
            "'--scope={s}' is not a valid scope kind — use 'code,bot' (dual) or 'por' (single)"
        )),
        _ => Err(format!(
            "'--scope={s}' is not recognized — expected 'code,bot' or 'por'"
        )),
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- parse_scope_kind ----------

    #[test]
    fn scope_kind_code_bot() {
        assert_eq!(parse_scope_kind("code,bot").unwrap(), ScopeKind::CodeBot);
    }

    #[test]
    fn scope_kind_bot_code_commutative() {
        assert_eq!(parse_scope_kind("bot,code").unwrap(), ScopeKind::CodeBot);
    }

    #[test]
    fn scope_kind_por() {
        assert_eq!(parse_scope_kind("por").unwrap(), ScopeKind::Por);
    }

    #[test]
    fn scope_kind_code_alone_errors() {
        let err = parse_scope_kind("code").unwrap_err();
        assert!(err.contains("not a valid scope kind"), "got: {err}");
    }

    #[test]
    fn scope_kind_bot_alone_errors() {
        let err = parse_scope_kind("bot").unwrap_err();
        assert!(err.contains("not a valid scope kind"), "got: {err}");
    }

    #[test]
    fn scope_kind_unknown_errors() {
        let err = parse_scope_kind("xyz").unwrap_err();
        assert!(err.contains("not recognized"), "got: {err}");
    }

    // ---------- parse_repo_arg ----------

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

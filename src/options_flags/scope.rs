//! `--scope` — code,bot|por target set selector.
//! See [options_flags](README.md) for shared architecture.

use clap::Args;

/// Typed value of `--scope` — the kinds of repo set a
/// subcommand can target.
///
/// - `CodeBot` (default) — dual-repo: code + `.claude` bot
///   session.
/// - `Por` — single repo (Plain Old Repo); no `.claude/`,
///   no `.vc-config.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScopeKind {
    #[default]
    CodeBot,
    Por,
}

/// Parse the `--scope` value into a `ScopeKind`.
///
/// - Accepts `code,bot` / `bot,code` (commutative) and `por`.
/// - Standalone `code` or `bot` errors — these are
///   config-lookup keywords; subcommands using `ScopeKind`
///   have no config-driven sides to look up against.
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

/// `FlagParser` impl for `--scope`. Documentation-level —
/// consumers can use either `parse_scope_kind` directly or
/// `ScopeParser::parse`.
pub struct ScopeParser;

impl super::FlagParser for ScopeParser {
    type Value = ScopeKind;

    fn parse(s: &str) -> Result<Self::Value, String> {
        parse_scope_kind(s)
    }
}

/// `--scope` leaf — see
/// [Consuming an OF](README.md#consuming-an-of).
#[derive(Args, Debug, Clone, Default)]
pub struct ScopeFlag {
    /// Scope — `code,bot` (dual, default) or `por` (single).
    #[arg(
        long,
        short,
        value_name = "SCOPE",
        value_parser = parse_scope_kind,
        default_value = "code,bot",
        verbatim_doc_comment
    )]
    pub scope: ScopeKind,
}

impl super::FlagBundle for ScopeFlag {}

#[cfg(test)]
mod tests {
    use super::*;

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
}

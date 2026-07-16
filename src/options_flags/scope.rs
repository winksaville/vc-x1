//! `--scope` — which side(s) of a dual workspace a command
//! operates on. See [options_flags](README.md) for shared
//! architecture.
//!
//! - `Side` is the clap `ValueEnum` — one of `code`, `bot`.
//! - `Scope` is a newtype over `Vec<Side>` — the parsed role
//!   set (`code`, `bot`, `code,bot`, `bot,code`).
//! - Path-based single-repo operation lives on `-R/--repo`,
//!   not in `--scope`; `Scope` carries role information only.

use clap::ValueEnum;

/// One side of a dual-repo workspace.
///
/// - `Code` — the primary (work) repo.
/// - `Bot` — the Claude Code bot repo (typically at `.claude/`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Side {
    Code,
    Bot,
}

/// Parsed `--scope` value — the requested role set of a
/// dual-repo workspace.
///
/// Newtype over `Vec<Side>`; the vector preserves the order the
/// keywords were given in (`code,bot` vs `bot,code`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope(pub Vec<Side>);

/// Parse a `--scope` value string.
///
/// Accepts exactly the four role-keyword forms — `code`, `bot`,
/// `code,bot`, `bot,code` — preserving order. Anything else (an
/// empty string, a bare name, duplicate or out-of-set
/// combinations, a path) is an error: path-based single-repo
/// operation uses `-R/--repo`, not `--scope`.
pub fn parse_scope(s: &str) -> Result<Scope, String> {
    match s {
        "code" => Ok(Scope(vec![Side::Code])),
        "bot" => Ok(Scope(vec![Side::Bot])),
        "code,bot" => Ok(Scope(vec![Side::Code, Side::Bot])),
        "bot,code" => Ok(Scope(vec![Side::Bot, Side::Code])),
        "" => Err("--scope: value is empty".into()),
        other => Err(format!(
            "--scope: '{other}' is not a recognized form. \
             Expected one of `code`, `bot`, `code,bot`, `bot,code`. \
             For single-repo operation by path, use `-R/--repo`."
        )),
    }
}

impl Scope {
    /// True when the role set includes the code side.
    pub fn has_code(&self) -> bool {
        self.0.contains(&Side::Code)
    }

    /// True when the role set includes the bot side.
    pub fn has_bot(&self) -> bool {
        self.0.contains(&Side::Bot)
    }

    /// Exactly the code side — a single-side dual-repo op.
    pub fn is_code_only(&self) -> bool {
        self.has_code() && !self.has_bot()
    }

    /// Exactly the bot side — a single-side dual-repo op.
    #[allow(dead_code)]
    pub fn is_bot_only(&self) -> bool {
        !self.has_code() && self.has_bot()
    }

    /// Both sides — a full dual-repo op.
    pub fn is_both(&self) -> bool {
        self.has_code() && self.has_bot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_only() {
        let s = Scope(vec![Side::Code]);
        assert!(s.is_code_only());
        assert!(!s.is_bot_only());
        assert!(!s.is_both());
    }

    #[test]
    fn bot_only() {
        let s = Scope(vec![Side::Bot]);
        assert!(!s.is_code_only());
        assert!(s.is_bot_only());
        assert!(!s.is_both());
    }

    #[test]
    fn both_code_then_bot() {
        let s = Scope(vec![Side::Code, Side::Bot]);
        assert!(!s.is_code_only());
        assert!(!s.is_bot_only());
        assert!(s.is_both());
    }

    #[test]
    fn both_bot_then_code() {
        // Order doesn't matter — contains-based checks.
        let s = Scope(vec![Side::Bot, Side::Code]);
        assert!(s.is_both());
    }

    #[test]
    fn empty_roles() {
        // Empty side list isn't a state anything constructs (the
        // parser rejects empty input), but the helpers still need
        // to be well-defined on it.
        let s = Scope(vec![]);
        assert!(!s.is_code_only());
        assert!(!s.is_bot_only());
        assert!(!s.is_both());
    }

    #[test]
    fn parse_keyword_code() {
        assert_eq!(parse_scope("code").unwrap(), Scope(vec![Side::Code]));
    }

    #[test]
    fn parse_keyword_bot() {
        assert_eq!(parse_scope("bot").unwrap(), Scope(vec![Side::Bot]));
    }

    #[test]
    fn parse_keyword_code_bot_preserves_order() {
        assert_eq!(
            parse_scope("code,bot").unwrap(),
            Scope(vec![Side::Code, Side::Bot])
        );
    }

    #[test]
    fn parse_keyword_bot_code_preserves_order() {
        assert_eq!(
            parse_scope("bot,code").unwrap(),
            Scope(vec![Side::Bot, Side::Code])
        );
    }

    #[test]
    fn parse_bare_name_errors() {
        let err = parse_scope("foo").unwrap_err();
        assert!(err.contains("'foo'"), "got: {err}");
    }

    #[test]
    fn parse_empty_errors() {
        let err = parse_scope("").unwrap_err();
        assert!(err.contains("empty"), "got: {err}");
    }

    #[test]
    fn parse_path_form_errors() {
        // Path forms are no longer a `--scope` value — `-R/--repo`
        // handles single-repo operation. The error points there.
        let err = parse_scope("./foo").unwrap_err();
        assert!(err.contains("-R/--repo"), "got: {err}");
    }

    #[test]
    fn parse_unknown_keyword_combo_errors() {
        // Duplicates and out-of-set combos are rejected; only the
        // exact four keyword forms are accepted.
        assert!(parse_scope("code,code").is_err());
        assert!(parse_scope("code,bot,code").is_err());
        assert!(parse_scope("bot,bot").is_err());
    }
}

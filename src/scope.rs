//! Which side(s) of a workspace a command should operate on.
//!
//! - `Side` is the clap `ValueEnum` — one of `code`, `bot`.
//! - `Scope` is an enum:
//!   - `Roles(Vec<Side>)` — dual-repo workspace; the parsed
//!     keyword form (`code`, `bot`, `code,bot`, `bot,code`).
//!   - `Single(PathBuf)` — explicit single-repo mode; ignores
//!     `.vc-config.toml` and operates on the one repo at the
//!     given path. Wired up incrementally (parser lands in
//!     0.42.0-2, consumers in later steps).
//! - Helpers (`has_code`, `is_both`, etc.) reflect the `Roles`
//!   arm only; on `Single(_)` they all return `false`. Callers
//!   that must distinguish modes match on the enum directly.
//! - See `notes/chores-07.md > --scope enum refactor (0.42.0)`
//!   for the migration plan, and `notes/chores-06.md > 0.41.0-4`
//!   for the vocabulary capture.

use clap::ValueEnum;
use std::path::PathBuf;

/// One side of a dual-repo workspace.
///
/// - `Code` — the primary (app) repo.
/// - `Bot` — the Claude Code session repo (typically at `.claude/`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Side {
    Code,
    Bot,
}

/// Parsed `--scope` value.
///
/// - `Roles(_)` — dual-repo workspace; the requested side set.
/// - `Single(_)` — explicit single-repo mode at the given path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scope {
    Roles(Vec<Side>),
    Single(PathBuf),
}

/// Parse a `--scope` value string.
///
/// Accepted forms:
///
/// - Keyword: exactly one of `code`, `bot`, `code,bot`, `bot,code`
///   → `Scope::Roles(...)`. Order is preserved.
/// - Path: starts with `./`, `../`, `/`, `~/`, or is the bare `~`
///   → `Scope::Single(PathBuf)`. The raw string is stored; tilde
///   and `$VAR` expansion happens at the consumer (see
///   `init::expand_vars`).
///
/// Anything else — including a bare unprefixed name like `foo`, an
/// empty string, or an unrecognized keyword combination — is an
/// error. The error message hints at the path-prefix requirement
/// so users see the disambiguation rule when they mistype.
pub fn parse_scope(s: &str) -> Result<Scope, String> {
    if s == "~"
        || s.starts_with("./")
        || s.starts_with("../")
        || s.starts_with('/')
        || s.starts_with("~/")
    {
        return Ok(Scope::Single(PathBuf::from(s)));
    }
    match s {
        "code" => Ok(Scope::Roles(vec![Side::Code])),
        "bot" => Ok(Scope::Roles(vec![Side::Bot])),
        "code,bot" => Ok(Scope::Roles(vec![Side::Code, Side::Bot])),
        "bot,code" => Ok(Scope::Roles(vec![Side::Bot, Side::Code])),
        "" => Err("--scope: value is empty".into()),
        other => Err(format!(
            "--scope: '{other}' is not a recognized form. \
             Expected one of `code`, `bot`, `code,bot`, `bot,code`, \
             or a path (`./X`, `../X`, `/X`, `~/X`). \
             Bare names without `./` are reserved for keywords; \
             prefix paths with `./` to disambiguate."
        )),
    }
}

impl Scope {
    /// True when the `Roles` arm includes the code side.
    pub fn has_code(&self) -> bool {
        matches!(self, Scope::Roles(v) if v.contains(&Side::Code))
    }

    /// True when the `Roles` arm includes the bot side.
    pub fn has_bot(&self) -> bool {
        matches!(self, Scope::Roles(v) if v.contains(&Side::Bot))
    }

    /// Roles-only with exactly the code side — single-side dual-repo op.
    pub fn is_code_only(&self) -> bool {
        self.has_code() && !self.has_bot()
    }

    /// Roles-only with exactly the bot side — single-side dual-repo op.
    pub fn is_bot_only(&self) -> bool {
        !self.has_code() && self.has_bot()
    }

    /// Roles arm including both sides — full dual-repo op.
    pub fn is_both(&self) -> bool {
        self.has_code() && self.has_bot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_only() {
        let s = Scope::Roles(vec![Side::Code]);
        assert!(s.is_code_only());
        assert!(!s.is_bot_only());
        assert!(!s.is_both());
    }

    #[test]
    fn bot_only() {
        let s = Scope::Roles(vec![Side::Bot]);
        assert!(!s.is_code_only());
        assert!(s.is_bot_only());
        assert!(!s.is_both());
    }

    #[test]
    fn both_code_then_bot() {
        let s = Scope::Roles(vec![Side::Code, Side::Bot]);
        assert!(!s.is_code_only());
        assert!(!s.is_bot_only());
        assert!(s.is_both());
    }

    #[test]
    fn both_bot_then_code() {
        // Order doesn't matter — contains-based checks.
        let s = Scope::Roles(vec![Side::Bot, Side::Code]);
        assert!(s.is_both());
    }

    #[test]
    fn empty_roles() {
        // Empty side list isn't a state anything constructs (the
        // parser rejects empty input), but the helpers still need
        // to be well-defined on it.
        let s = Scope::Roles(vec![]);
        assert!(!s.is_code_only());
        assert!(!s.is_bot_only());
        assert!(!s.is_both());
    }

    #[test]
    fn single_helpers_all_false() {
        // `Single(_)` is a distinct mode; the Roles-shaped helpers
        // all report false so callers that match on helpers
        // naturally fall through to mode-aware logic.
        let s = Scope::Single(PathBuf::from("/some/repo"));
        assert!(!s.has_code());
        assert!(!s.has_bot());
        assert!(!s.is_code_only());
        assert!(!s.is_bot_only());
        assert!(!s.is_both());
    }

    // ---------- parse_scope ----------

    #[test]
    fn parse_keyword_code() {
        assert_eq!(parse_scope("code").unwrap(), Scope::Roles(vec![Side::Code]));
    }

    #[test]
    fn parse_keyword_bot() {
        assert_eq!(parse_scope("bot").unwrap(), Scope::Roles(vec![Side::Bot]));
    }

    #[test]
    fn parse_keyword_code_bot_preserves_order() {
        assert_eq!(
            parse_scope("code,bot").unwrap(),
            Scope::Roles(vec![Side::Code, Side::Bot])
        );
    }

    #[test]
    fn parse_keyword_bot_code_preserves_order() {
        assert_eq!(
            parse_scope("bot,code").unwrap(),
            Scope::Roles(vec![Side::Bot, Side::Code])
        );
    }

    #[test]
    fn parse_path_dotslash() {
        assert_eq!(
            parse_scope("./foo").unwrap(),
            Scope::Single(PathBuf::from("./foo"))
        );
    }

    #[test]
    fn parse_path_dotdotslash() {
        assert_eq!(
            parse_scope("../sibling").unwrap(),
            Scope::Single(PathBuf::from("../sibling"))
        );
    }

    #[test]
    fn parse_path_absolute() {
        assert_eq!(
            parse_scope("/abs/path").unwrap(),
            Scope::Single(PathBuf::from("/abs/path"))
        );
    }

    #[test]
    fn parse_path_tilde_slash() {
        // Stored raw; the consumer expands with `expand_vars`.
        assert_eq!(
            parse_scope("~/work/x").unwrap(),
            Scope::Single(PathBuf::from("~/work/x"))
        );
    }

    #[test]
    fn parse_path_bare_tilde() {
        assert_eq!(parse_scope("~").unwrap(), Scope::Single(PathBuf::from("~")));
    }

    #[test]
    fn parse_bare_name_errors_with_hint() {
        let err = parse_scope("foo").unwrap_err();
        assert!(err.contains("'foo'"), "got: {err}");
        assert!(err.contains("./X") || err.contains("./"), "got: {err}");
    }

    #[test]
    fn parse_empty_errors() {
        let err = parse_scope("").unwrap_err();
        assert!(err.contains("empty"), "got: {err}");
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

//! `--scope` — which side(s) of a dual workspace a command
//! operates on. See [options_flags](README.md) for shared
//! architecture.
//!
//! - `Side` is the role enum — `Work` or `Bot`; the CLI
//!   keywords that name them are `work` and `bot`.
//! - `Scope` is a newtype over `Vec<Side>` — the parsed role
//!   set (`work`, `bot`, `work,bot`, `bot,work`).
//! - Path-based single-repo operation lives on `-R/--repo`,
//!   not in `--scope`; `Scope` carries role information only.

use clap::ValueEnum;

/// One side of a dual-repo workspace.
///
/// - `Work` — the primary work repo.
/// - `Bot` — the bot repo (typically at `.claude/`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Side {
    Work,
    Bot,
}

/// Parsed `--scope` value — the requested role set of a
/// dual-repo workspace.
///
/// Newtype over `Vec<Side>`; the vector preserves the order the
/// keywords were given in (`work,bot` vs `bot,work`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope(pub Vec<Side>);

/// Parse a `--scope` value string.
///
/// Accepts exactly the four role-keyword forms — `work`, `bot`,
/// `work,bot`, `bot,work` — preserving order. Anything else (an
/// empty string, a bare name, duplicate or out-of-set
/// combinations, a path) is an error: path-based single-repo
/// operation uses `-R/--repo`, not `--scope`.
pub fn parse_scope(s: &str) -> Result<Scope, String> {
    match s {
        "work" => Ok(Scope(vec![Side::Work])),
        "bot" => Ok(Scope(vec![Side::Bot])),
        "work,bot" => Ok(Scope(vec![Side::Work, Side::Bot])),
        "bot,work" => Ok(Scope(vec![Side::Bot, Side::Work])),
        "" => Err("--scope: value is empty".into()),
        other => Err(format!(
            "--scope: '{other}' is not a recognized form. \
             Expected one of `work`, `bot`, `work,bot`, `bot,work`. \
             For single-repo operation by path, use `-R/--repo`."
        )),
    }
}

impl Scope {
    /// True when the role set includes the work side.
    pub fn has_work(&self) -> bool {
        self.0.contains(&Side::Work)
    }

    /// True when the role set includes the bot side.
    pub fn has_bot(&self) -> bool {
        self.0.contains(&Side::Bot)
    }

    /// Exactly the work side — a single-side dual-repo op.
    pub fn is_work_only(&self) -> bool {
        self.has_work() && !self.has_bot()
    }

    /// Exactly the bot side — a single-side dual-repo op.
    #[allow(dead_code)]
    pub fn is_bot_only(&self) -> bool {
        !self.has_work() && self.has_bot()
    }

    /// Both sides — a full dual-repo op.
    pub fn is_both(&self) -> bool {
        self.has_work() && self.has_bot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_only() {
        let s = Scope(vec![Side::Work]);
        assert!(s.is_work_only());
        assert!(!s.is_bot_only());
        assert!(!s.is_both());
    }

    #[test]
    fn bot_only() {
        let s = Scope(vec![Side::Bot]);
        assert!(!s.is_work_only());
        assert!(s.is_bot_only());
        assert!(!s.is_both());
    }

    #[test]
    fn both_work_then_bot() {
        let s = Scope(vec![Side::Work, Side::Bot]);
        assert!(!s.is_work_only());
        assert!(!s.is_bot_only());
        assert!(s.is_both());
    }

    #[test]
    fn both_bot_then_work() {
        // Order doesn't matter — contains-based checks.
        let s = Scope(vec![Side::Bot, Side::Work]);
        assert!(s.is_both());
    }

    #[test]
    fn empty_roles() {
        // Empty side list isn't a state anything constructs (the
        // parser rejects empty input), but the helpers still need
        // to be well-defined on it.
        let s = Scope(vec![]);
        assert!(!s.is_work_only());
        assert!(!s.is_bot_only());
        assert!(!s.is_both());
    }

    #[test]
    fn parse_keyword_work() {
        assert_eq!(parse_scope("work").unwrap(), Scope(vec![Side::Work]));
    }

    #[test]
    fn parse_keyword_bot() {
        assert_eq!(parse_scope("bot").unwrap(), Scope(vec![Side::Bot]));
    }

    #[test]
    fn parse_keyword_work_bot_preserves_order() {
        assert_eq!(
            parse_scope("work,bot").unwrap(),
            Scope(vec![Side::Work, Side::Bot])
        );
    }

    #[test]
    fn parse_keyword_bot_work_preserves_order() {
        assert_eq!(
            parse_scope("bot,work").unwrap(),
            Scope(vec![Side::Bot, Side::Work])
        );
    }

    #[test]
    fn parse_former_code_keyword_now_errors() {
        // `code` was the pre-0.74.0 spelling; the rename dropped it
        // (unreleased, no alias carried forward).
        assert!(parse_scope("code").is_err());
        assert!(parse_scope("code,bot").is_err());
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
        assert!(parse_scope("work,work").is_err());
        assert!(parse_scope("work,bot,work").is_err());
        assert!(parse_scope("bot,bot").is_err());
    }
}

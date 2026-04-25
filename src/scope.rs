//! Which side(s) of a workspace a command should operate on.
//!
//! - `Side` is the clap `ValueEnum` — one of `code`, `bot`.
//! - `Scope` wraps the parsed list; a missing flag (None at the arg
//!   layer) means "caller decides", typically via workspace-state
//!   default resolution.
//! - See `notes/chores-06.md > Generalize --scope across commands
//!   (0.40.0)` for the full rationale and the vocabulary history.

use clap::ValueEnum;

/// One side of the workspace.
///
/// - `Code` — the primary (app) repo.
/// - `Bot` — the Claude Code session repo (typically at `.claude/`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Side {
    Code,
    Bot,
}

/// Parsed `--scope` value: the set of sides the command applies to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope(pub Vec<Side>);

impl Scope {
    /// True when the scope includes the code side.
    pub fn has_code(&self) -> bool {
        self.0.contains(&Side::Code)
    }

    /// True when the scope includes the bot side.
    pub fn has_bot(&self) -> bool {
        self.0.contains(&Side::Bot)
    }

    /// Exactly the code side — single-repo operation.
    pub fn is_code_only(&self) -> bool {
        self.has_code() && !self.has_bot()
    }

    /// Exactly the bot side — single-repo operation on the bot.
    pub fn is_bot_only(&self) -> bool {
        !self.has_code() && self.has_bot()
    }

    /// Both sides — dual-repo operation.
    pub fn is_both(&self) -> bool {
        self.has_code() && self.has_bot()
    }

    /// Neither side — empty scope, always an invalid input.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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
    fn empty() {
        let s = Scope(vec![]);
        assert!(s.is_empty());
        assert!(!s.is_code_only());
        assert!(!s.is_bot_only());
        assert!(!s.is_both());
    }
}

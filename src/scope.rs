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
//!   arm only; on `Single(_)` they all return `false` /
//!   `is_empty()` returns `false`. Callers that must
//!   distinguish modes match on the enum directly.
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
    // Staged: production code starts constructing this in 0.42.0-2
    // (parser) / -3 (sync resolver); only tests reach it in -1.
    #[allow(dead_code)]
    Single(PathBuf),
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

    /// Roles arm with an empty side list — always invalid input.
    /// `Single(_)` returns `false` (it carries a path, not a side list).
    pub fn is_empty(&self) -> bool {
        matches!(self, Scope::Roles(v) if v.is_empty())
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
        let s = Scope::Roles(vec![]);
        assert!(s.is_empty());
        assert!(!s.is_code_only());
        assert!(!s.is_bot_only());
        assert!(!s.is_both());
    }

    #[test]
    fn single_helpers_all_false() {
        // `Single(_)` is a distinct mode; the Roles-shaped helpers
        // all report false / not-empty so callers that match on
        // helpers naturally fall through to mode-aware logic.
        let s = Scope::Single(PathBuf::from("/some/repo"));
        assert!(!s.has_code());
        assert!(!s.has_bot());
        assert!(!s.is_code_only());
        assert!(!s.is_bot_only());
        assert!(!s.is_both());
        assert!(!s.is_empty());
    }
}

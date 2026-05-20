//! `--por` — create a plain single repo (no `.claude/` companion).
//! See [options_flags](README.md) for shared architecture.

use clap::Args;

/// `--por` leaf (Flag — boolean domain) — see
/// [Consuming an OF](README.md#consuming-an-of).
///
/// Workspace topology selector at creation time:
///
/// - Absent (`--por` not passed) — dual workspace (app +
///   `.claude/` session repo).
/// - Present (`--por` passed) — plain single repo, no
///   `.claude/`, no `.vc-config.toml`.
#[derive(Args, Debug, Clone, Default)]
pub struct PorFlag {
    /// Create a plain single repo (no `.claude/` companion).
    #[arg(long = "por")]
    pub value: bool,
}

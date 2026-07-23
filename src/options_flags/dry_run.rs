//! `--dry-run` — show what would be done without executing.
//! See [options_flags](README.md) for shared architecture.

use clap::Args;

/// `--dry-run` leaf (Flag — boolean domain) — see
/// [Consuming an OF](README.md#consuming-an-of).
#[derive(Args, Debug, Clone, Default)]
pub struct DryRunFlag {
    /// Dry run — show what would be done without executing.
    #[arg(id = "dry_run", long = "dry-run")]
    pub value: bool,
}

//! Reusable CLI options and flags (OFs) — see
//! [Architecture](README.md#architecture) for the leaf / bundle /
//! Pattern-A composition patterns.

/// Marker trait for OF leaves and bundles. See
/// [Marker traits](README.md#marker-traits).
#[allow(dead_code, reason = "marker trait — see README.md#marker-traits")]
pub trait FlagBundle: clap::Args {}

/// Canonical shape for a flag's typed value-parser. See
/// [Marker traits](README.md#marker-traits).
#[expect(dead_code, reason = "trait gains impls in -6.7 sub-step (6)")]
pub trait FlagParser {
    /// Typed value the parser produces.
    type Value;

    /// Parse a CLI string into the typed value.
    fn parse(s: &str) -> Result<Self::Value, String>;
}

pub mod config;
pub mod dry_run;
pub mod private;

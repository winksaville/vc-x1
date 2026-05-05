//! Reusable CLI options and flags (OFs) — see
//! [Architecture](README.md#architecture) for the leaf / bundle /
//! Pattern-A composition patterns and the Flag-vs-Option
//! domain-based classification.

/// Marker trait for pure-boolean bundles (every constituent leaf
/// is a Flag). See [Marker traits](README.md#marker-traits).
#[allow(dead_code, reason = "marker trait — see README.md#marker-traits")]
pub trait FlagBundle: clap::Args {}

/// Marker trait for pure-non-boolean bundles (every constituent
/// leaf is an Option). See [Marker traits](README.md#marker-traits).
#[allow(dead_code, reason = "marker trait — see README.md#marker-traits")]
pub trait OptionBundle: clap::Args {}

/// Marker trait for mixed bundles (constituents include both Flag
/// and Option leaves). The most common bundle marker in practice.
/// See [Marker traits](README.md#marker-traits).
#[allow(dead_code, reason = "marker trait — see README.md#marker-traits")]
pub trait OptionFlagBundle: clap::Args {}

/// Canonical shape for a Flag (boolean-domain) leaf's typed
/// value-parser. Conditional contract — implemented only when a
/// boolean leaf takes a value form (e.g. `--flag=true|false`).
/// Presence/absence flags don't need an impl; clap parses
/// directly. See [Marker traits](README.md#marker-traits).
#[allow(
    dead_code,
    reason = "conditional contract — see README.md#marker-traits"
)]
pub trait FlagParser {
    /// Typed value the parser produces (typically `bool`).
    type Value;

    /// Parse a CLI string into the typed value.
    fn parse(s: &str) -> Result<Self::Value, String>;
}

/// Canonical shape for an Option (non-boolean-domain) leaf's
/// typed value-parser. Conditional contract — implemented only
/// when a leaf has explicit parsing logic; bare `Option<String>`
/// leaves need no impl. See
/// [Marker traits](README.md#marker-traits).
#[allow(
    dead_code,
    reason = "conditional contract — see README.md#marker-traits"
)]
pub trait OptionParser {
    /// Typed value the parser produces.
    type Value;

    /// Parse a CLI string into the typed value.
    fn parse(s: &str) -> Result<Self::Value, String>;
}

pub mod account;
pub mod config;
pub mod dry_run;
pub mod private;
pub mod provision_bundle;
pub mod push_retry;
pub mod repo;
pub mod scope;
pub mod use_template;

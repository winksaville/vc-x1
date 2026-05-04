//! Per-flag option modules.
//!
//! Collects reusable typed values + value parsers for CLI flags
//! shared across multiple subcommands. Each flag lives in its own
//! submodule so its type, parser, and tests stay together.
//!
//! - `config` — `--config none|<path>` (init's `.vc-config.toml`
//!   override; reusable by future commands).

pub mod config;

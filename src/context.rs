//! Shared platform handle passed to every subcommand operation.
//!
//! `Context` holds platform state that is the same across every
//! subcommand: today, the loaded `UserConfig` and the resolved
//! `--log` path. Built once at CLI startup and threaded through
//! to the subcommand layer.
//!
//! The shape is deliberately minimal — fields like project root,
//! progress sinks, etc. are added when a real consumer surfaces.
//! See `ARCHITECTURE.md` for the CLI-args vs subcommand
//! Context+Params layering rationale.

use std::path::PathBuf;

use crate::config::{self, UserConfig};

/// Shared platform handle for subcommand operations.
///
/// - `user_config`: the loaded user config
///   (`~/.config/vc-x1/config.toml` or its discovered equivalent).
/// - `log`: the `--log` path, if given — where file-bound output
///   goes (and what a re-exec'd child is told to log to).
pub struct Context {
    pub user_config: UserConfig,
    pub log: Option<PathBuf>,
}

impl Context {
    /// Build a `Context` by loading the user config from disk and
    /// taking the CLI-resolved `--log` path.
    pub fn load(log: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            user_config: config::load()?,
            log,
        })
    }
}

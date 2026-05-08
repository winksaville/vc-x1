//! Shared platform handle passed to every subcommand operation.
//!
//! `Context` holds platform state that is the same across every
//! subcommand: today, the loaded `UserConfig`. Built once at CLI
//! startup and threaded through to the op layer.
//!
//! The shape is deliberately minimal — fields like project root,
//! progress sinks, etc. are added when a real consumer surfaces.
//! See `notes/chores-09.md > ## Ops layer architecture
//! (forward-looking)` and `## InitParams implementation (0.44.0)`.

use crate::config::{self, UserConfig};

/// Shared platform handle for subcommand operations.
///
/// - `user_config`: the loaded user config
///   (`~/.config/vc-x1/config.toml` or its discovered equivalent).
pub struct Context {
    pub user_config: UserConfig,
}

impl Context {
    /// Build a `Context` by loading the user config from disk.
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            user_config: config::load()?,
        })
    }
}

//! Shared platform handle passed to every subcommand operation.
//!
//! `Context` holds platform state that is the same across every
//! subcommand: the loaded `UserConfig`, and the repo sessions the
//! invocation has opened so far. Built once at CLI startup and
//! threaded through to the subcommand layer.
//!
//! Sessions are has-a, never is-a: an invocation touches 0..N repos
//! (push touches two, `version` touches none), so `Context` owns a
//! lazily-opened map keyed by repo path rather than being a session
//! itself. See `ARCHITECTURE.md` for the CLI-args vs subcommand
//! Context+Params layering rationale.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::{self, UserConfig};
use crate::jj::session::RepoSession;

/// Shared platform handle for subcommand operations.
///
/// - `user_config`: the loaded user config
///   (`~/.config/vc-x1/config.toml` or its discovered equivalent).
/// - `sessions`: the invocation's open `RepoSession`s, keyed by
///   canonicalized repo path; access via `session`.
///
/// The `--log` path lived here while the retired detach machinery
/// (0.69.0-2) needed to forward it to its re-exec'd child; logging
/// is now fully handled at CLI startup and the field is gone.
pub struct Context {
    pub user_config: UserConfig,
    sessions: HashMap<PathBuf, RepoSession>,
}

impl Context {
    /// Build a `Context` around an already-loaded config (tests use
    /// this with `UserConfig::default()`); no sessions open yet.
    pub fn new(user_config: UserConfig) -> Self {
        Self {
            user_config,
            sessions: HashMap::new(),
        }
    }

    /// Build a `Context` by loading the user config from disk.
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::new(config::load()?))
    }

    /// The `RepoSession` for `repo`, opened on first use and reused
    /// for the rest of the invocation.
    ///
    /// - Keyed by the canonicalized path, so two spellings of the
    ///   same repo share one session.
    /// - Reuse skips only the open (settings + workspace load);
    ///   every verb still snapshots at the op-store head and runs
    ///   its own operation, so a cached session is as fresh as a
    ///   newly opened one.
    pub fn session(&mut self, repo: &Path) -> Result<&mut RepoSession, Box<dyn std::error::Error>> {
        use std::collections::hash_map::Entry;
        let key = repo
            .canonicalize()
            .map_err(|e| format!("cannot resolve repo path '{}': {e}", repo.display()))?;
        match self.sessions.entry(key) {
            Entry::Occupied(e) => Ok(e.into_mut()),
            Entry::Vacant(e) => Ok(e.insert(RepoSession::open(repo)?)),
        }
    }
}

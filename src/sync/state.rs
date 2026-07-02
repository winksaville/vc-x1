//! Persisted pre-sync snapshot: `<repo>/.vc-x1/sync-state.toml`.
//!
//! Sync stops on error instead of auto-reverting, so the pre-sync
//! `jj op` id must outlive the failed process for `vc-x1 revert`
//! to consume. Each synced repo carries its own snapshot file —
//! sync accepts arbitrary repo lists (`-R`, scope), so there is no
//! single home for a combined file:
//!
//! - written for every repo right after the op-id snapshot, before
//!   any fetch;
//! - removed on full success (a stale file must not become a
//!   revert target long after the state moved on);
//! - left in place on failure — the error report points at it.
//!
//! The `.vc-x1/` dir is already gitignored (init writes the
//! `/.vc-x1` line; push keeps its `push-state.toml` there too).

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use log::debug;

use crate::toml_simple::toml_load;

/// State-file format version; bump on incompatible change.
const STATE_FORMAT_VERSION: u32 = 1;

/// Relative path of the sync state file under a repo root.
const STATE_REL_PATH: &str = ".vc-x1/sync-state.toml";

/// One repo's pre-sync snapshot, as persisted.
///
/// Production reader is the `revert` subcommand (0.67.0-4); until
/// it lands only tests construct/read this, hence the allow.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncState {
    /// State-file format version; must match `STATE_FORMAT_VERSION`
    /// to be considered readable.
    pub version: u32,
    /// `jj op` id of the repo captured before the sync acted.
    pub op_id: String,
    /// Bookmark the sync was converging.
    pub bookmark: String,
    /// Remote the sync was fetching from.
    pub remote: String,
    /// ISO-8601 UTC timestamp when the snapshot was taken.
    /// Informational — helps the user spot stale state.
    pub started_at: String,
}

/// Full path of the sync state file for `repo`.
pub fn state_path(repo: &Path) -> PathBuf {
    repo.join(STATE_REL_PATH)
}

/// Persist a fresh snapshot for `repo`.
///
/// Creates `.vc-x1/` as needed and overwrites any previous file —
/// a new sync supersedes an older snapshot.
pub fn save(
    repo: &Path,
    op_id: &str,
    bookmark: &str,
    remote: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = state_path(repo);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = format!(
        "# vc-x1 sync state — managed file, do not edit by hand\n\
         [sync-state]\n\
         version = {STATE_FORMAT_VERSION}\n\
         op_id = \"{op_id}\"\n\
         bookmark = \"{bookmark}\"\n\
         remote = \"{remote}\"\n\
         started_at = \"{}\"\n",
        Utc::now().to_rfc3339()
    );
    fs::write(&path, content)?;
    debug!("sync: wrote state to {}", path.display());
    Ok(())
}

/// Load `repo`'s snapshot, `Ok(None)` when absent (no failed or
/// in-flight sync), `Err` when present but unreadable / wrong
/// version.
///
/// Production caller is the `revert` subcommand (0.67.0-4); until
/// it lands only tests call this, hence the allow.
#[allow(dead_code)]
pub fn load(repo: &Path) -> Result<Option<SyncState>, Box<dyn std::error::Error>> {
    let path = state_path(repo);
    if !path.exists() {
        return Ok(None);
    }
    let map = toml_load(&path)?;
    let require = |k: &str| -> Result<String, Box<dyn std::error::Error>> {
        map.get(k)
            .cloned()
            .ok_or_else(|| format!("{}: missing key '{k}'", path.display()).into())
    };
    let version: u32 = require("sync-state.version")?.parse()?;
    if version != STATE_FORMAT_VERSION {
        return Err(format!(
            "{}: unsupported sync-state version {version} (expected {STATE_FORMAT_VERSION})",
            path.display()
        )
        .into());
    }
    Ok(Some(SyncState {
        version,
        op_id: require("sync-state.op_id")?,
        bookmark: require("sync-state.bookmark")?,
        remote: require("sync-state.remote")?,
        started_at: require("sync-state.started_at")?,
    }))
}

/// Remove `repo`'s snapshot if present (success path / post-revert).
pub fn clear(repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let path = state_path(repo);
    if path.exists() {
        fs::remove_file(&path)?;
        debug!("sync: cleared state at {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::unique_base;

    /// save → load round-trips every field; clear removes the file.
    #[test]
    fn save_load_clear_roundtrip() {
        let repo = unique_base("sync-state");
        fs::create_dir_all(&repo).expect("mkdir fixture repo");
        save(&repo, "abcdef123456", "main", "origin").expect("save");
        let st = load(&repo).expect("load").expect("state present");
        assert_eq!(st.version, STATE_FORMAT_VERSION);
        assert_eq!(st.op_id, "abcdef123456");
        assert_eq!(st.bookmark, "main");
        assert_eq!(st.remote, "origin");
        assert!(!st.started_at.is_empty());
        clear(&repo).expect("clear");
        assert!(load(&repo).expect("load after clear").is_none());
        fs::remove_dir_all(&repo).ok();
    }

    /// Absent file loads as `None`.
    #[test]
    fn load_absent_is_none() {
        let repo = unique_base("sync-state-absent");
        fs::create_dir_all(&repo).expect("mkdir fixture repo");
        assert!(load(&repo).expect("load").is_none());
        fs::remove_dir_all(&repo).ok();
    }
}

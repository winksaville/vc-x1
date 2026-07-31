//! The three versions `--version` reports, and where each comes
//! from.
//!
//! They answer different questions and have different sources:
//!
//! - ours: `CARGO_PKG_VERSION`, compile time.
//! - jj-lib's: `JJ_LIB_VERSION`, resolved from `Cargo.lock` by
//!   `build.rs`, because jj-lib exports no version constant. It is
//!   what writes ops once the mutations move in-process, so it is
//!   compile-time by nature, not by compromise.
//! - the data's: read through jj-lib's public accessors only. What
//!   comes back is backend *type* names (the `.jj/repo/<backend>/type`
//!   files `RepoLoader::init_from_file_system` reads), because that
//!   is all jj-lib exposes: as of 0.43 it has no public version
//!   constant and no public accessor for one. The commit index does
//!   carry a format version, but the constant is `pub(super)` and a
//!   mismatch self-heals by reindexing; the op store, the thing we
//!   would be co-writing, carries no stamp at all.

use std::path::Path;

use jj_lib::config::StackedConfig;
use jj_lib::settings::UserSettings;
use jj_lib::workspace::{Workspace, default_working_copy_factories};

use crate::common;

/// The jj-lib version this binary links against.
pub fn jj_lib_version() -> &'static str {
    env!("JJ_LIB_VERSION")
}

/// The backend type names one repo's jj data records about itself.
///
/// One field per `type` file jj-lib loads a backend from. Every
/// value is an identity, never a version.
#[derive(Debug, PartialEq, Eq)]
pub struct DataVersion {
    /// Commit backend, e.g. `git`.
    pub commit: String,
    /// Operation store, e.g. `simple_op_store`.
    pub op: String,
    /// Operation heads store, e.g. `simple_op_heads_store`.
    pub op_heads: String,
    /// Commit index store, e.g. `default`.
    pub index: String,
    /// Submodule store, e.g. `default`.
    pub submodule: String,
    /// Working copy, e.g. `local`.
    pub working_copy: String,
}

impl DataVersion {
    /// Render as one `key=value` line, without the repo label.
    pub fn fields(&self) -> String {
        format!(
            "commit={} op={} op-heads={} index={} submodule={} working-copy={}",
            self.commit, self.op, self.op_heads, self.index, self.submodule, self.working_copy
        )
    }
}

/// Read one repo's recorded backend types through jj-lib.
///
/// Loads the workspace only, never `load_at_head`: resolving op
/// heads can merge divergent ones, which is a write, and reporting
/// a version must not mutate the repo it reports on.
pub fn data_version(path: &Path) -> Result<DataVersion, Box<dyn std::error::Error>> {
    let config = StackedConfig::with_defaults();
    let settings = UserSettings::from_config(config)?;
    let store_factories = jj_lib::repo::StoreFactories::default();
    let working_copy_factories = default_working_copy_factories();

    let workspace = Workspace::load(&settings, path, &store_factories, &working_copy_factories)?;
    let loader = workspace.repo_loader();

    Ok(DataVersion {
        commit: loader.store().backend().name().to_string(),
        op: loader.op_store().name().to_string(),
        op_heads: loader.op_heads_store().name().to_string(),
        index: loader.index_store().name().to_string(),
        submodule: loader.submodule_store().name().to_string(),
        working_copy: workspace.working_copy().name().to_string(),
    })
}

/// Render one repo's `jj-data` line, load failure included.
fn data_line(label: &str, path: &Path) -> String {
    match data_version(path) {
        Ok(data) => format!("jj-data {label} {}", data.fields()),
        Err(e) => format!("jj-data {label} unavailable: {e}"),
    }
}

/// Build the full `--version` report, one line per fact.
///
/// The repo lines are best-effort: outside a workspace, or on a
/// repo that fails to load, the line says so rather than being
/// dropped, so a missing repo never reads as a clean report.
pub fn report(banner: &str) -> Vec<String> {
    let mut lines = vec![banner.to_string(), format!("jj-lib {}", jj_lib_version())];

    match common::find_workspace_root() {
        None => lines.push("jj-data none: not in a vc-x1 workspace".to_string()),
        Some(root) => {
            lines.push(data_line("work", &root));
            match common::bot_repo_path(&root) {
                Ok(Some(bot)) => lines.push(data_line("bot", &bot)),
                Ok(None) => lines.push("jj-data bot unavailable: no bot repo".to_string()),
                Err(e) => lines.push(format!("jj-data bot unavailable: {e}")),
            }
        }
    }

    lines.push(
        "jj-data note: backend type names only; jj-lib exposes no on-disk format version"
            .to_string(),
    );
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The compiled-in version must be jj-lib's own, not another
    /// package's. Scanned here deliberately unlike `build.rs` does
    /// it (locate the name line, take the next version line), so a
    /// parser that drifted onto the wrong `[[package]]` block
    /// fails the test rather than agreeing with itself.
    #[test]
    fn jj_lib_version_matches_the_lock() {
        let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock"))
            .expect("Cargo.lock is readable");
        let mut lines = lock
            .lines()
            .skip_while(|l| l.trim() != r#"name = "jj-lib""#);
        assert!(lines.next().is_some(), "no jj-lib package in Cargo.lock");
        let version = lines
            .find_map(|l| l.trim().strip_prefix("version = "))
            .map(|v| v.trim_matches('"').to_string());
        assert_eq!(version.as_deref(), Some(jj_lib_version()));
    }

    #[test]
    fn jj_lib_version_is_a_version_triple() {
        let v = jj_lib_version();
        let parts: Vec<&str> = v.split('.').collect();
        assert_eq!(parts.len(), 3, "not a triple: {v}");
        for part in parts {
            assert!(
                part.chars().next().is_some_and(|c| c.is_ascii_digit()),
                "not numeric: {v}"
            );
        }
    }

    #[test]
    fn report_starts_with_banner_and_jj_lib() {
        let lines = report("vc-x1-dev 0.0.0");
        assert_eq!(lines[0], "vc-x1-dev 0.0.0");
        assert_eq!(lines[1], format!("jj-lib {}", jj_lib_version()));
    }

    #[test]
    fn fields_render_every_backend() {
        let data = DataVersion {
            commit: "git".to_string(),
            op: "simple_op_store".to_string(),
            op_heads: "simple_op_heads_store".to_string(),
            index: "default".to_string(),
            submodule: "default".to_string(),
            working_copy: "local".to_string(),
        };
        assert_eq!(
            data.fields(),
            "commit=git op=simple_op_store op-heads=simple_op_heads_store index=default \
             submodule=default working-copy=local"
        );
    }
}

//! The set version: the agent-files' own version, read from the
//! workspace the command runs in.
//!
//! The set's version-of-record is an empty file,
//! `agent-data/agent-files-vX.Y.Z{-suffix}`, whose name is the
//! version (versioning.md's The set's version). It is the workspace's
//! set that is reported, never the set this binary's own repo carried
//! when it was built.
//!
//! - `versions(root)`: the `v...` names under `<root>/agent-data`,
//!   sorted, empty when the directory or the file is missing.
//! - `banner_suffix(root)`: ` (agent-files v0.1.0)` for the run
//!   banner, empty when there is nothing to say.
//! - `report_line(root)`: the `agent-files ...` line of `vc-x1
//!   version`, `none` with the reason when there is no version.
//! - `AgentFilesArgs`: the `agent-files` subcommand group, `version`
//!   printing the bare names one per line, for scripts.

use std::path::Path;
use std::process::ExitCode;

use clap::{Args, Subcommand};
use log::error;

use crate::common;

/// The file-name prefix every set version file carries.
const PREFIX: &str = "agent-files-";

/// The set versions recorded under `<work_root>/agent-data`, sorted.
///
/// One entry is the normal state. Several means a bump left both
/// names behind, which the listing shows rather than hides.
pub fn versions(work_root: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(work_root.join("agent-data")) else {
        return Vec::new();
    };
    let mut out: Vec<String> = entries
        .flatten()
        .filter_map(|e| e.file_name().to_str().map(str::to_owned))
        .filter_map(|name| name.strip_prefix(PREFIX).map(str::to_owned))
        .filter(|v| v.starts_with('v'))
        .collect();
    out.sort();
    out
}

/// The banner's tail: ` (agent-files v0.1.0)`, or empty outside a
/// workspace and when the workspace has no version file.
pub fn banner_suffix(work_root: Option<&Path>) -> String {
    match work_root.map(versions) {
        Some(vs) if !vs.is_empty() => format!(" (agent-files {})", vs.join(", ")),
        _ => String::new(),
    }
}

/// The `vc-x1 version` line for the set, a `none` with its reason
/// when there is nothing to report.
pub fn report_line(work_root: Option<&Path>) -> String {
    match work_root {
        None => "agent-files none: not in a vc-x1 workspace".to_string(),
        Some(root) => {
            let vs = versions(root);
            if vs.is_empty() {
                format!("agent-files none: no agent-data/{PREFIX}v* file")
            } else {
                format!("agent-files {}", vs.join(", "))
            }
        }
    }
}

/// CLI args for the `agent-files` group.
#[derive(Args, Debug)]
pub struct AgentFilesArgs {
    #[command(subcommand)]
    pub command: AgentFilesCommand,
}

/// The `agent-files` subcommands.
#[derive(Subcommand, Debug)]
pub enum AgentFilesCommand {
    /// Print the workspace's set version, bare, one per line
    #[command(
        long_about = "Print the workspace's agent-files set version, the name of\n\
        its agent-data/agent-files-vX.Y.Z file with the prefix\n\
        removed, one per line when a bump left several. Fails with a\n\
        message when there is none: outside a vc-x1 workspace, or in\n\
        one without the file."
    )]
    Version,
}

impl AgentFilesArgs {
    /// Run the chosen subcommand against the workspace found from
    /// the current directory.
    pub fn run(&self) -> ExitCode {
        match self.command {
            AgentFilesCommand::Version => {
                let root = common::find_workspace_root();
                let vs = root.as_deref().map(versions).unwrap_or_default(); // OK: no root, no versions
                if vs.is_empty() {
                    error!("agent-files version: {}", report_line(root.as_deref()));
                    return ExitCode::FAILURE;
                }
                for v in vs {
                    println!("{v}");
                }
                ExitCode::SUCCESS
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh work root under the test tmp root, with `agent-data`
    /// holding the given version files.
    fn root_with(tag: &str, files: &[&str]) -> std::path::PathBuf {
        let root =
            crate::test_tmp_root::resolve_tmp_root().join(format!("vc_x1_agent_files_{tag}"));
        let _ = std::fs::remove_dir_all(&root);
        let data = root.join("agent-data");
        std::fs::create_dir_all(&data).expect("mkdir agent-data");
        for f in files {
            std::fs::write(data.join(f), "").expect("write version file");
        }
        root
    }

    /// The one-file case is the whole feature: the banner's tail and
    /// the report line both carry the name with the prefix removed.
    #[test]
    fn one_version_file_is_reported_everywhere() {
        let root = root_with("one", &["agent-files-v0.1.0", "prose.md"]);
        assert_eq!(versions(&root), vec!["v0.1.0".to_string()]);
        assert_eq!(banner_suffix(Some(&root)), " (agent-files v0.1.0)");
        assert_eq!(report_line(Some(&root)), "agent-files v0.1.0");
    }

    /// Two files show as two, sorted, so a half-done bump is visible
    /// rather than resolved by a guess.
    #[test]
    fn several_version_files_are_all_shown() {
        let root = root_with("many", &["agent-files-v0.1.0-2", "agent-files-v0.1.0-1"]);
        assert_eq!(
            banner_suffix(Some(&root)),
            " (agent-files v0.1.0-1, v0.1.0-2)"
        );
        assert_eq!(report_line(Some(&root)), "agent-files v0.1.0-1, v0.1.0-2");
    }

    /// No workspace, or a workspace without the file: the banner says
    /// nothing and the report line says why.
    #[test]
    fn none_is_silent_in_the_banner_and_explained_in_the_report() {
        assert_eq!(banner_suffix(None), "");
        assert_eq!(
            report_line(None),
            "agent-files none: not in a vc-x1 workspace"
        );
        let root = root_with("empty", &["prose.md"]);
        assert_eq!(banner_suffix(Some(&root)), "");
        assert_eq!(
            report_line(Some(&root)),
            "agent-files none: no agent-data/agent-files-v* file"
        );
        let bare = root_with("no-dir", &[]);
        std::fs::remove_dir_all(bare.join("agent-data")).expect("rm agent-data");
        assert_eq!(banner_suffix(Some(&bare)), "");
    }
}

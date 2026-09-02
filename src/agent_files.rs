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
//!   printing the bare names one per line, for scripts, and `diff`
//!   ([`diff`]) naming what differs from a copy of the set.
//! - `WorkspaceAgentFiles` / `config_at(root)`: the work side's
//!   `[agent-files.diff]` and `[agent-files.copy]` tables, the
//!   per-workspace defaults for the `diff` and `copy` operands and
//!   their `--custom` choice.

use std::path::Path;
use std::process::ExitCode;

use clap::{Args, Subcommand};
use log::error;

use crate::common;

pub mod diff;

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

/// One command's table, `[agent-files.diff]` or
/// `[agent-files.copy]`: the default `DIR` operand and the
/// default `--custom` choice, each `None` when the key is absent.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CommandDefaults {
    /// `dir`: a directory holding a copy of the set, as written,
    /// relative to the config file's directory.
    pub dir: Option<String>,
    /// `custom`: compare or copy custom.md with the rest.
    pub custom: Option<bool>,
}

/// The work side's `[agent-files.*]` tables.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct WorkspaceAgentFiles {
    pub diff: CommandDefaults,
    pub copy: CommandDefaults,
}

/// Read the `[agent-files.diff]` and `[agent-files.copy]` tables
/// of the config at `root`. No config, or no tables, is the
/// default; a `custom` that is not a bare `true` or `false` is an
/// error naming the key, since a malformed config is fatal rather
/// than silently ignored.
pub fn config_at(root: &Path) -> Result<WorkspaceAgentFiles, Box<dyn std::error::Error>> {
    let Some(cfg) = crate::config_md::load(root)? else {
        return Ok(WorkspaceAgentFiles::default());
    };
    let get = |key: &str| crate::toml_simple::toml_get(&cfg.map, key);
    let parse_bool = |key: &str| -> Result<Option<bool>, Box<dyn std::error::Error>> {
        match get(key).map(String::as_str) {
            None => Ok(None),
            Some("true") => Ok(Some(true)),
            Some("false") => Ok(Some(false)),
            Some(other) => Err(format!(
                "{key}: invalid bool {other:?}: expected true or false, unquoted"
            )
            .into()),
        }
    };
    Ok(WorkspaceAgentFiles {
        diff: CommandDefaults {
            dir: get("agent-files.diff.dir").cloned(),
            custom: parse_bool("agent-files.diff.custom")?,
        },
        copy: CommandDefaults {
            dir: get("agent-files.copy.dir").cloned(),
            custom: parse_bool("agent-files.copy.custom")?,
        },
    })
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

    /// Name the set files that differ from a copy of the set in DIR
    #[command(
        long_about = "Compare this workspace's agent-files set, AGENTS.md and\n\
        agent-data/, against the copy in DIR, one line per file: same,\n\
        differs, only here, or only there. custom.md is reported as the\n\
        project layer unless -c/--custom compares it. DIR defaults to\n\
        the config's agent-files.diff.dir, else family.template, relative\n\
        to the config file's directory. Exits non-zero when anything\n\
        differs, as diff does."
    )]
    Diff(diff::DiffArgs),
}

impl AgentFilesArgs {
    /// Run the chosen subcommand against the workspace found from
    /// the current directory.
    pub fn run(&self) -> ExitCode {
        match &self.command {
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
            AgentFilesCommand::Diff(args) => args.run(),
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

    /// A config carrying both tables reads back typed, one carrying
    /// neither is the default, and a `custom` that is not a bare
    /// bool is an error naming the key.
    #[test]
    fn config_tables_read_back_typed() {
        let write = |tag: &str, fences: &str| {
            let root = root_with(tag, &[]);
            std::fs::write(
                root.join(crate::config_md::VC_CONFIG_MD),
                format!("```toml\n[repos]\nwork = \".\"\n```\n{fences}"),
            )
            .expect("write config");
            root
        };
        let both = write(
            "cfg-both",
            "```toml\n[agent-files.diff]\ndir = \"../peer\"\ncustom = true\n\n\
             [agent-files.copy]\ndir = \"../tpl/work\"\ncustom = false\n```\n",
        );
        assert_eq!(
            config_at(&both).unwrap(),
            WorkspaceAgentFiles {
                diff: CommandDefaults {
                    dir: Some("../peer".to_string()),
                    custom: Some(true),
                },
                copy: CommandDefaults {
                    dir: Some("../tpl/work".to_string()),
                    custom: Some(false),
                },
            }
        );
        let none = write("cfg-none", "");
        assert_eq!(config_at(&none).unwrap(), WorkspaceAgentFiles::default());
        assert_eq!(
            config_at(&root_with("cfg-absent", &[])).unwrap(),
            WorkspaceAgentFiles::default()
        );
        let bad = write(
            "cfg-bad",
            "```toml\n[agent-files.copy]\ncustom = \"yes\"\n```\n",
        );
        let err = config_at(&bad).unwrap_err().to_string();
        assert!(err.contains("agent-files.copy.custom"), "{err}");
        assert!(err.contains("\"yes\""), "{err}");
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

//! `agent-files diff [A] [B]`: which files of two copies of the set
//! differ, one line per file, names only.
//!
//! - `DiffArgs`: clap surface, the `A` and `B` operands and the
//!   `-c`/`--custom` / `--no-custom` pair.
//! - `resolve_set_dir`: `A` as given, else the command's `dir`
//!   key, else `family.template`, relative paths against the
//!   config file's directory. `B` as given, else the workspace
//!   around the current directory. Shared with `copy`, where the
//!   same pair is its source and destination.
//! - `resolve_custom`: the flag, else the command's `custom` key,
//!   else the built-in.
//! - `compare`: the union of both sides' set files with each one's
//!   state, and `report` renders it. The exit status is non-zero
//!   when anything differs, as `diff`'s is, so a script can ask
//!   "is a re-sync a copy?" without reading.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Args;
use log::{error, info};

use crate::common;

/// The set's fixed members beside `agent-data/`.
pub const AGENTS_MD: &str = "AGENTS.md";
/// The project layer, compared only on request.
pub const CUSTOM_MD: &str = "custom.md";
/// The directory whose files are the rest of the set.
pub const AGENT_DATA: &str = "agent-data";

/// CLI args for `agent-files diff`.
#[derive(Args, Debug)]
pub struct DiffArgs {
    /// A directory holding a copy of the set [default:
    /// agent-files.diff.dir, else family.template, else an error]
    #[arg(value_name = "A")]
    pub a: Option<PathBuf>,

    /// The other copy [default: this workspace, so one operand is A]
    #[arg(value_name = "B")]
    pub b: Option<PathBuf>,

    /// Compare custom.md like the rest of the set
    #[arg(short = 'c', long = "custom", conflicts_with = "no_custom")]
    pub custom: bool,

    /// Leave custom.md out, overriding agent-files.diff.custom
    #[arg(long = "no-custom")]
    pub no_custom: bool,
}

/// One file's state across the two sides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Same,
    Differs,
    OnlyInA,
    OnlyInB,
    /// custom.md when it is not compared: the project layer.
    ProjectLayer,
}

impl State {
    /// True when the state is a difference a copy would change.
    pub fn differs(self) -> bool {
        matches!(self, State::Differs | State::OnlyInA | State::OnlyInB)
    }

    fn label(self, a: &str, b: &str) -> String {
        match self {
            State::Same => "same".to_string(),
            State::Differs => "differs".to_string(),
            State::OnlyInA => format!("only in {a}"),
            State::OnlyInB => format!("only in {b}"),
            State::ProjectLayer => "project layer, not compared (-c compares it)".to_string(),
        }
    }
}

/// Where a set directory came from, for the header line.
#[derive(Debug, PartialEq, Eq)]
pub enum DirSource {
    Operand,
    ConfigKey(&'static str),
    FamilyTemplate,
    Workspace,
}

impl std::fmt::Display for DirSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirSource::Operand => write!(f, "the operand"),
            DirSource::ConfigKey(k) => write!(f, "{k}"),
            DirSource::FamilyTemplate => write!(f, "family.template"),
            DirSource::Workspace => write!(f, "this workspace"),
        }
    }
}

/// A resolved set directory: the path to use, the text it was
/// written as, for the report, and where it came from.
#[derive(Debug, PartialEq, Eq)]
pub struct SetDir {
    pub path: PathBuf,
    pub shown: String,
    pub source: DirSource,
}

impl SetDir {
    fn new(
        path: PathBuf,
        shown: String,
        source: DirSource,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.is_dir() {
            return Err(format!("set directory '{shown}' ({source}) is not a directory").into());
        }
        Ok(SetDir {
            path,
            shown,
            source,
        })
    }
}

/// The first set directory, `A` for diff and the source for copy:
/// the operand as given, else the command's `dir` key, else
/// `family.template`, the config values relative to `root`, the
/// config file's directory, which is only needed when the operand
/// is absent. An error names the three when none is set, and the
/// path when the directory is missing.
pub fn resolve_set_dir(
    root: Option<&Path>,
    operand: Option<&Path>,
    key: &'static str,
    key_value: Option<&str>,
    template: Option<&str>,
) -> Result<SetDir, Box<dyn std::error::Error>> {
    if let Some(p) = operand {
        return SetDir::new(p.to_path_buf(), p.display().to_string(), DirSource::Operand);
    }
    let root = root.ok_or("not in a vc-x1 workspace, and no directory given")?;
    if let Some(v) = key_value {
        SetDir::new(
            common::resolve_repo_path(root, v),
            v.to_string(),
            DirSource::ConfigKey(key),
        )
    } else if let Some(t) = template {
        SetDir::new(
            common::resolve_repo_path(root, t),
            t.to_string(),
            DirSource::FamilyTemplate,
        )
    } else {
        Err(
            format!("no set directory: give one, or set {key} or family.template in the config")
                .into(),
        )
    }
}

/// The second set directory, `B` for diff and the destination for
/// copy: the operand as given, else the workspace around the
/// current directory.
pub fn resolve_here(
    root: Option<&Path>,
    operand: Option<&Path>,
) -> Result<SetDir, Box<dyn std::error::Error>> {
    if let Some(p) = operand {
        return SetDir::new(p.to_path_buf(), p.display().to_string(), DirSource::Operand);
    }
    let root = root.ok_or("not in a vc-x1 workspace, and no second directory given")?;
    SetDir::new(
        root.to_path_buf(),
        root.display().to_string(),
        DirSource::Workspace,
    )
}

/// The `family.template` value of the config at `root`, if any.
pub fn family_template(root: &Path) -> Result<Option<String>, Box<dyn std::error::Error>> {
    Ok(crate::config_md::load(root)?
        .and_then(|c| crate::toml_simple::toml_get(&c.map, "family.template").cloned()))
}

/// The custom.md choice: the flags, else the config key, else the
/// built-in default.
pub fn resolve_custom(flag: bool, no_flag: bool, key_value: Option<bool>, default: bool) -> bool {
    if flag {
        true
    } else if no_flag {
        false
    } else {
        key_value.unwrap_or(default)
    }
}

/// The set's files under `dir`, as relative paths: AGENTS.md, the
/// plain files directly under `agent-data/`, and custom.md when
/// `custom`. A missing file is simply absent, so an empty
/// directory is an empty set.
pub fn set_files(dir: &Path, custom: bool) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if dir.join(AGENTS_MD).is_file() {
        out.insert(AGENTS_MD.to_string());
    }
    if custom && dir.join(CUSTOM_MD).is_file() {
        out.insert(CUSTOM_MD.to_string());
    }
    if let Ok(entries) = std::fs::read_dir(dir.join(AGENT_DATA)) {
        for e in entries.flatten() {
            if e.path().is_file()
                && let Some(name) = e.file_name().to_str()
            {
                out.insert(format!("{AGENT_DATA}/{name}"));
            }
        }
    }
    out
}

/// Every set file on either side with its state, sorted by path.
/// custom.md rides along as the project layer when it is not
/// compared and either side has it.
pub fn compare(
    a: &Path,
    b: &Path,
    custom: bool,
) -> Result<Vec<(String, State)>, Box<dyn std::error::Error>> {
    let in_a = set_files(a, custom);
    let in_b = set_files(b, custom);
    let mut out = Vec::new();
    for path in in_a.union(&in_b) {
        let state = match (in_a.contains(path), in_b.contains(path)) {
            (true, false) => State::OnlyInA,
            (false, true) => State::OnlyInB,
            _ => {
                if std::fs::read(a.join(path))? == std::fs::read(b.join(path))? {
                    State::Same
                } else {
                    State::Differs
                }
            }
        };
        out.push((path.clone(), state));
    }
    if !custom && (a.join(CUSTOM_MD).is_file() || b.join(CUSTOM_MD).is_file()) {
        out.push((CUSTOM_MD.to_string(), State::ProjectLayer));
    }
    Ok(out)
}

/// Render the comparison: one aligned line per file and a summary.
pub fn report(rows: &[(String, State)], a: &str, b: &str) -> String {
    let width = rows.iter().map(|(p, _)| p.len()).max().unwrap_or(0);
    let mut out = String::new();
    for (path, state) in rows {
        out.push_str(&format!("{path:<width$}  {}\n", state.label(a, b)));
    }
    let compared = rows
        .iter()
        .filter(|(_, s)| *s != State::ProjectLayer)
        .count();
    let differing = rows.iter().filter(|(_, s)| s.differs()).count();
    out.push_str(&format!("{differing} of {compared} differ\n"));
    out
}

impl DiffArgs {
    /// Run the comparison. Non-zero when the sets differ, or on an
    /// error.
    pub fn run(&self) -> ExitCode {
        match self.diff() {
            Ok(true) => ExitCode::SUCCESS,
            Ok(false) => ExitCode::FAILURE,
            Err(e) => {
                error!("agent-files diff: {e}");
                ExitCode::FAILURE
            }
        }
    }

    /// Print the report and say whether the sets are the same.
    fn diff(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let root = common::find_workspace_root();
        let cfg = match &root {
            Some(r) => super::config_at(r)?,
            None => super::WorkspaceAgentFiles::default(),
        };
        let template = match &root {
            Some(r) => family_template(r)?,
            None => None,
        };
        let a = resolve_set_dir(
            root.as_deref(),
            self.a.as_deref(),
            "agent-files.diff.dir",
            cfg.diff.dir.as_deref(),
            template.as_deref(),
        )?;
        let b = resolve_here(root.as_deref(), self.b.as_deref())?;
        let custom = resolve_custom(
            self.custom,
            self.no_custom,
            cfg.diff.custom,
            crate::config_schema::AGENT_FILES_DIFF_CUSTOM_DEFAULT,
        );
        info!(
            "agent-files diff: {} ({}) against {} ({})",
            a.shown, a.source, b.shown, b.source
        );
        let rows = compare(&a.path, &b.path, custom)?;
        info!("{}", report(&rows, &a.shown, &b.shown).trim_end());
        Ok(!rows.iter().any(|(_, s)| s.differs()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct T {
        #[command(flatten)]
        a: DiffArgs,
    }

    /// A set directory under the test tmp root with the given
    /// files, `(relative path, content)`.
    pub(crate) fn set_dir(tag: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = crate::test_tmp_root::resolve_tmp_root().join(format!("vc_x1_af_diff_{tag}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(AGENT_DATA)).expect("mkdir");
        for (path, content) in files {
            std::fs::write(dir.join(path), content).expect("write");
        }
        dir
    }

    /// The operands and flags parse, and `-c` with `--no-custom` is
    /// refused.
    #[test]
    fn flags_parse() {
        let a = T::try_parse_from(["t", "../x", "-c"]).unwrap().a;
        assert_eq!(a.a, Some(PathBuf::from("../x")));
        assert!(a.b.is_none());
        assert!(a.custom && !a.no_custom);
        let a = T::try_parse_from(["t", "../x", "../y", "--no-custom"])
            .unwrap()
            .a;
        assert_eq!(a.b, Some(PathBuf::from("../y")));
        assert!(a.no_custom);
        assert!(T::try_parse_from(["t", "-c", "--no-custom"]).is_err());
    }

    /// Flag, then key, then default, in that order.
    #[test]
    fn custom_resolution_order() {
        assert!(resolve_custom(true, false, Some(false), false));
        assert!(!resolve_custom(false, true, Some(true), true));
        assert!(resolve_custom(false, false, Some(true), false));
        assert!(!resolve_custom(false, false, None, false));
        assert!(resolve_custom(false, false, None, true));
    }

    /// A: operand, then the key, then family.template, each
    /// relative to the config's directory, needing no workspace
    /// when given. B: operand, else the workspace. Clear errors with
    /// none, with no workspace, and with a path that is not a
    /// directory.
    #[test]
    fn set_dir_resolution_order() {
        let root = set_dir("root", &[]);
        let peer = set_dir("peer", &[]);
        let tpl = set_dir("tpl", &[]);
        let key = "agent-files.diff.dir";
        let d = resolve_set_dir(None, Some(&peer), key, Some("../nope"), None).unwrap();
        assert_eq!(d.path, peer);
        assert_eq!(d.shown, peer.display().to_string());
        assert_eq!(d.source, DirSource::Operand);
        let rel = format!("../{}", peer.file_name().unwrap().to_str().unwrap());
        let d = resolve_set_dir(Some(&root), None, key, Some(&rel), Some("../x")).unwrap();
        assert_eq!(d.path.canonicalize().unwrap(), peer.canonicalize().unwrap());
        assert_eq!(d.shown, rel);
        assert_eq!(d.source, DirSource::ConfigKey(key));
        let rel = format!("../{}", tpl.file_name().unwrap().to_str().unwrap());
        let d = resolve_set_dir(Some(&root), None, key, None, Some(&rel)).unwrap();
        assert_eq!(d.path.canonicalize().unwrap(), tpl.canonicalize().unwrap());
        assert_eq!(d.source, DirSource::FamilyTemplate);
        let err = resolve_set_dir(Some(&root), None, key, None, None).unwrap_err();
        assert!(err.to_string().contains("family.template"), "{err}");
        let err = resolve_set_dir(None, None, key, Some(&rel), None).unwrap_err();
        assert!(
            err.to_string().contains("not in a vc-x1 workspace"),
            "{err}"
        );
        let err =
            resolve_set_dir(Some(&root), None, key, Some("../missing-dir"), None).unwrap_err();
        assert!(err.to_string().contains("not a directory"), "{err}");
        let h = resolve_here(Some(&root), None).unwrap();
        assert_eq!((h.path, h.source), (root.clone(), DirSource::Workspace));
        let h = resolve_here(None, Some(&peer)).unwrap();
        assert_eq!((h.path, h.source), (peer.clone(), DirSource::Operand));
        assert!(resolve_here(None, None).is_err());
    }

    /// Every state appears: same, differs, only on each side, and
    /// custom.md as the project layer until `-c` compares it. The
    /// summary counts what was compared, and the report aligns.
    #[test]
    fn compare_states_and_report() {
        let a = set_dir(
            "a",
            &[
                ("AGENTS.md", "rules\n"),
                ("custom.md", "theirs\n"),
                ("agent-data/a.md", "same\n"),
                ("agent-data/b.md", "new\n"),
                ("agent-data/c.md", "extra\n"),
            ],
        );
        let b = set_dir(
            "b",
            &[
                ("AGENTS.md", "rules\n"),
                ("custom.md", "mine\n"),
                ("agent-data/a.md", "same\n"),
                ("agent-data/b.md", "old\n"),
                ("agent-data/agent-files-v0.1.0", ""),
            ],
        );
        let rows = compare(&a, &b, false).unwrap();
        assert_eq!(
            rows,
            vec![
                ("AGENTS.md".to_string(), State::Same),
                ("agent-data/a.md".to_string(), State::Same),
                ("agent-data/agent-files-v0.1.0".to_string(), State::OnlyInB),
                ("agent-data/b.md".to_string(), State::Differs),
                ("agent-data/c.md".to_string(), State::OnlyInA),
                ("custom.md".to_string(), State::ProjectLayer),
            ]
        );
        let text = report(&rows, "../a", ".");
        let line = |start: &str| {
            text.lines()
                .find(|l| l.starts_with(start))
                .unwrap()
                .to_string()
        };
        assert!(
            line("agent-data/c.md ").ends_with("  only in ../a"),
            "{text}"
        );
        assert!(
            line("agent-data/agent-files-v0.1.0 ").ends_with("  only in ."),
            "{text}"
        );
        assert!(line("custom.md ").contains("  project layer"), "{text}");
        // Aligned: every state starts in the same column.
        let width = "agent-data/agent-files-v0.1.0".len() + 2;
        assert!(line("AGENTS.md ")[width..].starts_with("same"), "{text}");
        assert!(line("custom.md ")[width..].starts_with("project"), "{text}");
        assert!(text.ends_with("3 of 5 differ\n"), "{text}");
        let rows = compare(&a, &b, true).unwrap();
        assert!(rows.contains(&("custom.md".to_string(), State::Differs)));
        assert!(report(&rows, "x", "y").ends_with("4 of 6 differ\n"));
        let rows = compare(&a, &a, true).unwrap();
        assert!(rows.iter().all(|(_, s)| *s == State::Same));
        assert!(report(&rows, "x", "y").ends_with("0 of 5 differ\n"));
    }
}

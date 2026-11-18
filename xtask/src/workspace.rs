//! The workspace the tasks run in, as `cargo metadata` describes it:
//! its target directory, its binaries, which one is the default, and
//! the settings it gives xtask. Nothing here names a project, so
//! `xtask/` copies verbatim between workspaces.
//!
//! - The settings are `[workspace.metadata.xtask]` in the root
//!   `Cargo.toml`, every key optional:
//!   - `default-bin`: the binary a task takes when `--bin` names none.
//!   - `validate` and `validate-fast`: the validation's commands, each
//!     an array of words, `"cargo"` the cargo running the task, in
//!     place of [`FULL`] and [`FAST`].
//!   - `tasks`: the workspace's own tasks, each a name and its list of
//!     commands in the same form, run as `cargo xtask <name>`, so a
//!     project adds a task without changing `xtask/`.
//! - The default binary is `--bin`, else `default-bin`, else the root
//!   package's `default-run`, a `[package]` key a virtual workspace has
//!   no place for, else the one bin target, xtask's own left out, else
//!   none, refused naming them.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::util::{cargo, command, profile_dir};

/// The default full validation, each element one command.
pub const FULL: &[&[&str]] = &[
    &["cargo", "fmt"],
    &["cargo", "clippy", "--all-targets", "--", "-D", "warnings"],
    &["cargo", "test"],
];

/// The default fast validation, the binaries' unit tests alone.
pub const FAST: &[&[&str]] = &[&["cargo", "test", "--bins"]];

/// A bin target of a workspace member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bin {
    /// The target's name, the binary's file name.
    pub name: String,
    /// Its package's name.
    pub package: String,
    /// Whether its package is a default member, built by a plain
    /// `cargo build`.
    pub default_member: bool,
}

/// What the tasks know of the workspace.
#[derive(Debug, PartialEq, Eq)]
pub struct Workspace {
    /// Cargo's target directory.
    pub target_dir: PathBuf,
    /// The members' bin targets, xtask's own left out.
    pub bins: Vec<Bin>,
    /// `[workspace.metadata.xtask] default-bin`.
    pub default_bin: Option<String>,
    /// The root package's `default-run`, when the root is a package.
    pub default_run: Option<String>,
    /// `[workspace.metadata.xtask] validate`.
    pub validate: Option<Vec<Vec<String>>>,
    /// `[workspace.metadata.xtask] validate-fast`.
    pub validate_fast: Option<Vec<Vec<String>>>,
    /// `[workspace.metadata.xtask.tasks]`, each task's commands by its
    /// name.
    pub tasks: BTreeMap<String, Vec<Vec<String>>>,
}

impl Workspace {
    /// Read the workspace at `root` through `cargo metadata`.
    pub fn read(root: &Path) -> Result<Workspace, String> {
        let out = command(cargo())
            .current_dir(root)
            .args([
                "metadata",
                "--format-version",
                "1",
                "--no-deps",
                "--offline",
            ])
            .output()
            .map_err(|e| format!("cargo metadata did not run: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "cargo metadata failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        let meta: serde_json::Value = serde_json::from_slice(&out.stdout)
            .map_err(|e| format!("cargo metadata printed no JSON: {e}"))?;
        Workspace::from_metadata(&meta, env!("CARGO_PKG_NAME"))
    }

    /// The workspace in `cargo metadata`'s JSON, the package named
    /// `own`, xtask's, left out of the binaries.
    pub fn from_metadata(meta: &serde_json::Value, own: &str) -> Result<Workspace, String> {
        let text = |v: &serde_json::Value| v.as_str().map(str::to_string);
        let target_dir = text(&meta["target_directory"])
            .map(PathBuf::from)
            .ok_or("cargo metadata named no target_directory")?;
        let root = text(&meta["workspace_root"]).ok_or("cargo metadata named no workspace_root")?;
        let root_manifest = Path::new(&root).join("Cargo.toml");
        let defaults: Vec<&str> = match meta["workspace_default_members"].as_array() {
            Some(ids) => ids.iter().filter_map(|id| id.as_str()).collect(),
            None => Vec::new(),
        };
        let mut bins = Vec::new();
        let mut default_run = None;
        for package in meta["packages"].as_array().into_iter().flatten() {
            // OK: a package with no manifest path is no root package
            if Path::new(package["manifest_path"].as_str().unwrap_or_default()) == root_manifest {
                default_run = text(&package["default_run"]);
            }
            if package["name"] == own {
                continue;
            }
            let default_member = package["id"]
                .as_str()
                .is_some_and(|id| defaults.contains(&id));
            for target in package["targets"].as_array().into_iter().flatten() {
                let is_bin = target["kind"]
                    .as_array()
                    .is_some_and(|kinds| kinds.iter().any(|k| k == "bin"));
                if let (true, Some(name)) = (is_bin, text(&target["name"])) {
                    bins.push(Bin {
                        name,
                        // OK: cargo metadata names every package
                        package: text(&package["name"]).unwrap_or_default(),
                        default_member,
                    });
                }
            }
        }
        let settings = &meta["metadata"]["xtask"];
        Ok(Workspace {
            target_dir,
            bins,
            default_bin: text(&settings["default-bin"]),
            default_run,
            validate: commands(&settings["validate"], "validate")?,
            validate_fast: commands(&settings["validate-fast"], "validate-fast")?,
            tasks: tasks(&settings["tasks"])?,
        })
    }

    /// The binary a task takes: `requested`, else the default, refused
    /// when there is none or it names no binary of the workspace.
    pub fn bin(&self, requested: Option<&str>) -> Result<&Bin, String> {
        let names = || {
            self.bins
                .iter()
                .map(|b| format!("`{}`", b.name))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let find = |name: &str, from: &str| {
            self.bins.iter().find(|b| b.name == name).ok_or_else(|| {
                format!(
                    "{from} names `{name}`, not a binary of the workspace, whose are {}",
                    names()
                )
            })
        };
        if let Some(name) = requested {
            return find(name, "--bin");
        }
        if let Some(name) = &self.default_bin {
            return find(name, "[workspace.metadata.xtask] default-bin");
        }
        if let Some(name) = &self.default_run {
            return find(name, "the root package's default-run");
        }
        match self.bins.as_slice() {
            [one] => Ok(one),
            [] => Err("the workspace has no binary".to_string()),
            _ => Err(format!(
                "the workspace has binaries {}, and none is the default: pass --bin NAME, or name \
                 one in [workspace.metadata.xtask] default-bin",
                names()
            )),
        }
    }

    /// Where the build of `profile` leaves `bin`.
    pub fn binary(&self, profile: &str, bin: &Bin) -> PathBuf {
        self.target_dir.join(profile_dir(profile)).join(format!(
            "{}{}",
            bin.name,
            std::env::consts::EXE_SUFFIX
        ))
    }

    /// The validation's commands, the workspace's when it gives them,
    /// else the defaults.
    pub fn validation(&self, fast: bool) -> Vec<Vec<String>> {
        let (given, default) = match fast {
            true => (&self.validate_fast, FAST),
            false => (&self.validate, FULL),
        };
        match given {
            Some(lists) => lists.clone(),
            None => default
                .iter()
                .map(|words| words.iter().map(|w| w.to_string()).collect())
                .collect(),
        }
    }
}

/// The workspace's own tasks, `[workspace.metadata.xtask.tasks]`, a
/// table of names, each a list of commands. Absent is none.
fn tasks(value: &serde_json::Value) -> Result<BTreeMap<String, Vec<Vec<String>>>, String> {
    if value.is_null() {
        return Ok(BTreeMap::new());
    }
    let table = value.as_object().ok_or(
        "[workspace.metadata.xtask] tasks is not a table of tasks, each a name and its list of \
         commands, such as docs = [[\"cargo\", \"doc\"]]",
    )?;
    let mut out = BTreeMap::new();
    for (name, value) in table {
        let key = format!("tasks.{name}");
        let Some(lists) = commands(value, &key)? else {
            return Err(format!("[workspace.metadata.xtask] {key} is empty"));
        };
        out.insert(name.clone(), lists);
    }
    Ok(out)
}

/// A list of commands in the settings, `key` naming it in an error:
/// absent is `None`, and anything but a non-empty array of non-empty
/// arrays of strings is refused.
fn commands(value: &serde_json::Value, key: &str) -> Result<Option<Vec<Vec<String>>>, String> {
    if value.is_null() {
        return Ok(None);
    }
    let bad = || {
        format!(
            "[workspace.metadata.xtask] {key} is not a list of commands, each an array of words, \
             such as [[\"cargo\", \"test\"]]"
        )
    };
    let lists = value.as_array().filter(|l| !l.is_empty()).ok_or_else(bad)?;
    lists
        .iter()
        .map(|words| {
            let words = words.as_array().filter(|w| !w.is_empty()).ok_or_else(bad)?;
            words
                .iter()
                .map(|w| w.as_str().map(str::to_string).ok_or_else(bad))
                .collect()
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A package of a test workspace: its name, its bin targets, its
    /// `default-run`, and whether it is the root package.
    type Package<'a> = (&'a str, &'a [&'a str], Option<&'a str>, bool);

    /// `cargo metadata`'s JSON for a workspace at `/w` of `packages`,
    /// each a name, its bin targets, its `default-run`, and whether
    /// it is the root package, every one a default member.
    fn meta(packages: &[Package], xtask: serde_json::Value) -> serde_json::Value {
        let packages: Vec<_> = packages
            .iter()
            .map(|(name, bins, default_run, root)| {
                let manifest = match root {
                    true => "/w/Cargo.toml".to_string(),
                    false => format!("/w/{name}/Cargo.toml"),
                };
                let mut targets: Vec<_> = bins
                    .iter()
                    .map(|b| json!({"name": b, "kind": ["bin"]}))
                    .collect();
                targets.push(json!({"name": name, "kind": ["lib"]}));
                json!({
                    "name": name,
                    "id": format!("id-{name}"),
                    "manifest_path": manifest,
                    "default_run": default_run,
                    "targets": targets,
                })
            })
            .collect();
        let defaults: Vec<_> = packages_defaults(&packages);
        json!({
            "target_directory": "/w/target",
            "workspace_root": "/w",
            "workspace_default_members": defaults,
            "packages": packages,
            "metadata": {"xtask": xtask},
        })
    }

    /// Every package a default member, as a workspace with no
    /// `default-members` has it.
    fn packages_defaults(packages: &[serde_json::Value]) -> Vec<serde_json::Value> {
        packages.iter().map(|p| p["id"].clone()).collect()
    }

    fn names(ws: &Workspace) -> Vec<&str> {
        ws.bins.iter().map(|b| b.name.as_str()).collect()
    }

    /// The binaries are the members' bin targets, xtask's left out, and
    /// the default follows `--bin`, `default-bin`, `default-run`, then
    /// the one binary.
    #[test]
    fn the_default_binary_is_declared_or_the_only_one() {
        let one = Workspace::from_metadata(
            &meta(
                &[
                    ("app", &["app"], None, true),
                    ("xtask", &["xtask"], None, false),
                ],
                json!(null),
            ),
            "xtask",
        )
        .unwrap();
        assert_eq!(names(&one), ["app"]);
        assert_eq!(one.bin(None).unwrap().name, "app");
        assert_eq!(
            one.binary("dev", one.bin(None).unwrap()),
            PathBuf::from(format!(
                "/w/target/debug/app{}",
                std::env::consts::EXE_SUFFIX
            ))
        );

        let two = |xtask: serde_json::Value, default_run: Option<&str>| {
            Workspace::from_metadata(
                &meta(
                    &[
                        ("root", &[], default_run, true),
                        ("a", &["a"], None, false),
                        ("b", &["b", "b2"], None, false),
                        ("xtask", &["xtask"], None, false),
                    ],
                    xtask,
                ),
                "xtask",
            )
            .unwrap()
        };
        let none = two(json!(null), None);
        assert_eq!(names(&none), ["a", "b", "b2"]);
        let e = none.bin(None).unwrap_err();
        assert!(
            e.contains("`a`, `b`, `b2`") && e.contains("--bin NAME"),
            "{e}"
        );
        assert_eq!(none.bin(Some("b2")).unwrap().name, "b2");
        let e = none.bin(Some("xtask")).unwrap_err();
        assert!(e.contains("not a binary of the workspace"), "{e}");
        assert_eq!(
            two(json!({"default-bin": "b"}), None)
                .bin(None)
                .unwrap()
                .name,
            "b"
        );
        assert_eq!(two(json!(null), Some("a")).bin(None).unwrap().name, "a");
        assert_eq!(
            two(json!({"default-bin": "b"}), Some("a"))
                .bin(None)
                .unwrap()
                .name,
            "b",
            "default-bin outranks default-run"
        );
        let e = two(json!({"default-bin": "c"}), None)
            .bin(None)
            .unwrap_err();
        assert!(e.contains("default-bin names `c`"), "{e}");
    }

    /// The validation is the workspace's when given, else the defaults,
    /// and a malformed list is refused naming its key.
    #[test]
    fn the_validation_is_the_workspaces_or_the_defaults() {
        let ws = |xtask| {
            Workspace::from_metadata(&meta(&[("app", &["app"], None, true)], xtask), "xtask")
        };
        let plain = ws(json!(null)).unwrap();
        assert_eq!(plain.validation(false).len(), 3);
        assert_eq!(plain.validation(true), [["cargo", "test", "--bins"]]);
        let given =
            ws(json!({"validate": [["cargo", "test", "--", "--include-ignored"]]})).unwrap();
        assert_eq!(
            given.validation(false),
            [["cargo", "test", "--", "--include-ignored"]]
        );
        assert_eq!(given.validation(true), [["cargo", "test", "--bins"]]);
        for bad in [json!([]), json!([[]]), json!(["cargo test"]), json!([[1]])] {
            let e = ws(json!({ "validate-fast": bad })).unwrap_err();
            assert!(e.contains("validate-fast is not a list of commands"), "{e}");
        }
    }

    /// The workspace's own tasks are read by name, none when absent,
    /// and a malformed one is refused naming it.
    #[test]
    fn the_workspaces_tasks_are_its_commands_by_name() {
        let ws = |xtask| {
            Workspace::from_metadata(&meta(&[("app", &["app"], None, true)], xtask), "xtask")
        };
        assert!(ws(json!(null)).unwrap().tasks.is_empty());
        let given = ws(json!({"tasks": {"docs": [["cargo", "doc", "--no-deps"]]}})).unwrap();
        assert_eq!(given.tasks["docs"], [["cargo", "doc", "--no-deps"]]);
        let e = ws(json!({"tasks": {"docs": "cargo doc"}})).unwrap_err();
        assert!(e.contains("tasks.docs is not a list of commands"), "{e}");
        let e = ws(json!({"tasks": [["cargo", "doc"]]})).unwrap_err();
        assert!(e.contains("tasks is not a table of tasks"), "{e}");
    }
}

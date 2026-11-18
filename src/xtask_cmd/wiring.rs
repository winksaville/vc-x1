//! What makes `xtask/` a workspace's tasks, beside its files: the
//! member in the root `Cargo.toml` and the `cargo xtask` alias in
//! `.cargo/config.toml`, each edited by `toml_edit`, which keeps the
//! file's comments and layout and changes the one entry.
//!
//! - A directory with no `Cargo.toml`, a repo fresh from `vc-x1 init`,
//!   gets [`NEW_MANIFEST`], a virtual workspace of xtask alone.
//! - A single package gains `[workspace] members = [".", "xtask"]`,
//!   becoming a workspace whose root package it is.
//! - A workspace gains `"xtask"` in `members`, unless an entry, a glob
//!   included, already names it.
//! - A workspace whose `exclude` names `xtask`, or a package inside a
//!   parent directory's workspace, is refused, naming why.
//! - The alias is added when it is absent, and an alias of its own is
//!   kept and said.
//! - `.gitignore` gains `/target` when no line ignores it, since jj
//!   snapshots what is not ignored and the first build makes it.
//!
//! The functions take the files' text and return the new text, so the
//! caller reads and writes and these are tested without a disk.

use std::path::{Path, PathBuf};

use toml_edit::{Array, DocumentMut, Item, Table, value};

/// The member's path, and the alias's name.
pub const XTASK: &str = "xtask";

/// The alias `cargo xtask` runs, as this workspace's own
/// `.cargo/config.toml` writes it.
pub const ALIAS: &str = "run --quiet --package xtask --";

/// The `Cargo.toml` of a directory that has none: a virtual workspace
/// of xtask alone, its packages to come, and a version-of-record for
/// `cargo xtask version` and `bump`, which a member takes by
/// `version.workspace = true`.
pub const NEW_MANIFEST: &str = "# The workspace: xtask, the project's own tasks, and the packages\n\
                                # added to members. The version is the workspace's, a member\n\
                                # taking it by `version.workspace = true`.\n\
                                [workspace]\n\
                                members = [\"xtask\"]\n\
                                resolver = \"3\"\n\
                                \n\
                                [workspace.package]\n\
                                version = \"0.1.0\"\n";

/// The header a new `.cargo/config.toml` opens with.
const CONFIG_HEADER: &str = "# `cargo xtask <task>` runs the project's own tasks, the `xtask`\n\
                             # package in this workspace: `cargo xtask bump next` for one.\n";

/// The root `Cargo.toml` with `xtask` a member, `None` when it is one
/// already. `parent` is a directory above holding a workspace, if any,
/// found by [`parent_workspace`].
pub fn manifest(text: &str, parent: Option<&Path>) -> Result<Option<String>, String> {
    let mut doc: DocumentMut = text
        .parse()
        .map_err(|e| format!("Cargo.toml does not parse: {e}"))?;
    let has_package = doc.get("package").is_some_and(Item::is_table_like);
    let Some(ws) = doc.get_mut("workspace").and_then(Item::as_table_like_mut) else {
        if !has_package {
            return Err("Cargo.toml has neither [package] nor [workspace]".to_string());
        }
        if let Some(parent) = parent {
            return Err(format!(
                "the package is inside the workspace at {}, so xtask belongs to that one: copy \
                 it there",
                parent.display()
            ));
        }
        let mut table = Table::new();
        let mut members = Array::new();
        members.push(".");
        members.push(XTASK);
        table.insert("members", value(members));
        doc.insert("workspace", Item::Table(table));
        return Ok(Some(doc.to_string()));
    };
    if let Some(exclude) = ws.get("exclude").and_then(Item::as_array)
        && exclude
            .iter()
            .filter_map(|v| v.as_str())
            .any(|e| e.trim_end_matches('/') == XTASK)
    {
        return Err("the workspace's exclude names xtask: remove it there first".to_string());
    }
    match ws.get_mut("members") {
        Some(item) => {
            let members = item
                .as_array_mut()
                .ok_or("[workspace] members is not an array")?;
            if members
                .iter()
                .filter_map(|v| v.as_str())
                .any(|m| glob(m.trim_start_matches("./").trim_end_matches('/'), XTASK))
            {
                return Ok(None);
            }
            push_member(members, XTASK);
        }
        None => {
            let mut members = Array::new();
            if has_package {
                members.push(".");
            }
            members.push(XTASK);
            ws.insert("members", value(members));
        }
    }
    Ok(Some(doc.to_string()))
}

/// Push `member` onto `members` in the array's own layout: a
/// multi-line array gets it on a line of its own at the entries'
/// indent, after what trails the last one, a comment included, and a
/// one-line array gets it at the end.
fn push_member(members: &mut Array, member: &str) {
    // What trails the last entry up to its line's end, a comment
    // included, in a multi-line array, which alone has a newline there.
    let before = members
        .trailing()
        .as_str()
        .and_then(|t| t.rsplit_once('\n'))
        .map(|(before, _)| before.to_string());
    let Some(before) = before else {
        members.push(member);
        return;
    };
    let indent = members
        .iter()
        .last()
        .and_then(|v| v.decor().prefix())
        .and_then(|p| p.as_str())
        .and_then(|p| p.rsplit('\n').next())
        .filter(|i| i.chars().all(|c| c == ' ' || c == '\t'))
        .map_or_else(|| "    ".to_string(), str::to_string);
    let mut entry = toml_edit::Value::from(member);
    entry.decor_mut().set_prefix(format!("{before}\n{indent}"));
    members.push_formatted(entry);
    members.set_trailing("\n");
    members.set_trailing_comma(true);
}

/// Whether a workspace is virtual, a root `Cargo.toml` with no
/// `[package]`, and declares no `default-bin`: such a workspace of
/// several binaries has no default for `cargo xtask run`.
pub fn wants_default_bin(text: &str) -> bool {
    let Ok(doc) = text.parse::<DocumentMut>() else {
        return false;
    };
    let declared = doc
        .get("workspace")
        .and_then(|w| w.get("metadata"))
        .and_then(|m| m.get(XTASK))
        .and_then(|x| x.get("default-bin"))
        .is_some();
    doc.get("package").is_none() && !declared
}

/// What the alias needs in a `.cargo/config.toml` of `text`, `None`
/// for a file not there yet.
#[derive(Debug, PartialEq, Eq)]
pub enum Alias {
    /// The file's text with the alias added.
    Add(String),
    /// The alias is there, as [`ALIAS`].
    Present,
    /// The file has an alias of its own by that name, kept.
    Own(String),
}

/// The alias in the `.cargo/config.toml` whose text is `text`, `None`
/// when there is no file.
pub fn alias(text: Option<&str>) -> Result<Alias, String> {
    let Some(text) = text else {
        return Ok(Alias::Add(format!(
            "{CONFIG_HEADER}[alias]\n{XTASK} = \"{ALIAS}\"\n"
        )));
    };
    let mut doc: DocumentMut = text
        .parse()
        .map_err(|e| format!(".cargo/config.toml does not parse: {e}"))?;
    if !doc.contains_key("alias") {
        doc.insert("alias", Item::Table(Table::new()));
    }
    let aliases = doc["alias"]
        .as_table_like_mut()
        .ok_or(".cargo/config.toml's alias is not a table")?;
    if let Some(own) = aliases.get(XTASK) {
        let own = match own.as_str() {
            Some(s) => s.to_string(),
            None => own.to_string().trim().to_string(),
        };
        return Ok(match own == ALIAS {
            true => Alias::Present,
            false => Alias::Own(own),
        });
    }
    aliases.insert(XTASK, value(ALIAS));
    Ok(Alias::Add(doc.to_string()))
}

/// The `.gitignore` whose text is `text`, `None` for none, with
/// `/target` added, `None` when a line already ignores `target`.
pub fn gitignore(text: Option<&str>) -> Option<String> {
    let text = text.unwrap_or_default(); // OK: no file is an empty one
    let ignored = text
        .lines()
        .map(str::trim)
        .any(|l| matches!(l, "target" | "/target" | "target/" | "/target/"));
    if ignored {
        return None;
    }
    let sep = match text.is_empty() || text.ends_with('\n') {
        true => "",
        false => "\n",
    };
    Some(format!("{text}{sep}/target\n"))
}

/// The nearest directory above `root` whose `Cargo.toml` has a
/// `[workspace]`, which a package at `root` would belong to.
pub fn parent_workspace(root: &Path) -> Option<PathBuf> {
    root.ancestors().skip(1).find_map(|dir| {
        let text = std::fs::read_to_string(dir.join("Cargo.toml")).ok()?;
        let doc = text.parse::<DocumentMut>().ok()?;
        doc.contains_key("workspace").then(|| dir.to_path_buf())
    })
}

/// Cargo's glob over one path segment: `*` any run of characters, `?`
/// any one, every other character itself.
fn glob(pattern: &str, text: &str) -> bool {
    fn go(p: &[char], t: &[char]) -> bool {
        match (p.first(), t.first()) {
            (None, None) => true,
            (Some('*'), _) => go(&p[1..], t) || (!t.is_empty() && go(p, &t[1..])),
            (Some('?'), Some(_)) => go(&p[1..], &t[1..]),
            (Some(a), Some(b)) if a == b => go(&p[1..], &t[1..]),
            _ => false,
        }
    }
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    go(&p, &t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A single package becomes a workspace of it and xtask, its
    /// comments kept, and a package inside another workspace is
    /// refused naming it.
    #[test]
    fn a_single_package_becomes_a_workspace() {
        let text = "# the app\n[package]\nname = \"app\" # its name\nversion = \"0.1.0\"\n\n[dependencies]\n";
        let out = manifest(text, None).unwrap().unwrap();
        assert!(
            out.starts_with("# the app\n[package]\nname = \"app\" # its name\n"),
            "{out}"
        );
        assert!(
            out.ends_with("[workspace]\nmembers = [\".\", \"xtask\"]\n"),
            "{out}"
        );
        let e = manifest(text, Some(Path::new("/up"))).unwrap_err();
        assert!(e.contains("inside the workspace at /up"), "{e}");
    }

    /// A workspace gains the member once, a glob covering it counts,
    /// one without `members` gets them, and `exclude` refuses it.
    #[test]
    fn a_workspace_gains_the_member_once() {
        let root = "[package]\nname = \"a\"\n\n[workspace]\nmembers = [\n  \".\",\n  \"tool\", # the tool\n]\n";
        let out = manifest(root, None).unwrap().unwrap();
        assert!(
            out.contains("  \"tool\", # the tool\n  \"xtask\",\n]"),
            "a line of its own, after the comment: {out}"
        );
        assert_eq!(
            manifest(&out, None).unwrap(),
            None,
            "a second copy adds nothing"
        );
        let virtual_ws = "[workspace]\nmembers = [\"app\"]\nresolver = \"3\"\n";
        let out = manifest(virtual_ws, None).unwrap().unwrap();
        assert!(out.contains("members = [\"app\", \"xtask\"]"), "{out}");
        assert_eq!(
            manifest("[workspace]\nmembers = [\"*\"]\n", None).unwrap(),
            None
        );
        assert_eq!(
            manifest("[workspace]\nmembers = [\"./xtask/\"]\n", None).unwrap(),
            None
        );
        let bare = manifest("[package]\nname = \"a\"\n[workspace]\n", None)
            .unwrap()
            .unwrap();
        assert!(bare.contains("members = [\".\", \"xtask\"]"), "{bare}");
        let e = manifest(
            "[workspace]\nmembers = [\"a\"]\nexclude = [\"xtask\"]\n",
            None,
        )
        .unwrap_err();
        assert!(e.contains("exclude names xtask"), "{e}");
        assert!(
            manifest("[dependencies]\n", None)
                .unwrap_err()
                .contains("neither")
        );
    }

    /// Only a virtual workspace that declares no default binary is
    /// told to.
    #[test]
    fn a_virtual_workspace_is_told_of_default_bin() {
        assert!(wants_default_bin("[workspace]\nmembers = [\"a\", \"b\"]\n"));
        assert!(!wants_default_bin(
            "[workspace]\nmembers = [\"a\"]\n[workspace.metadata.xtask]\ndefault-bin = \"a\"\n"
        ));
        assert!(!wants_default_bin("[package]\nname = \"a\"\n[workspace]\n"));
    }

    /// The alias is written into a new file or added to one, and one
    /// already there is left as it is, the same or its own.
    #[test]
    fn the_alias_is_added_or_kept() {
        match alias(None).unwrap() {
            Alias::Add(text) => {
                assert!(text.ends_with("[alias]\nxtask = \"run --quiet --package xtask --\"\n"))
            }
            other => panic!("{other:?}"),
        }
        match alias(Some("# mine\n[build]\njobs = 4\n")).unwrap() {
            Alias::Add(text) => {
                assert!(text.starts_with("# mine\n[build]\njobs = 4\n"), "{text}");
                assert!(
                    text.contains("xtask = \"run --quiet --package xtask --\""),
                    "{text}"
                );
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(
            alias(Some(
                "[alias]\nxtask = \"run --quiet --package xtask --\"\n"
            ))
            .unwrap(),
            Alias::Present
        );
        assert_eq!(
            alias(Some("[alias]\nxtask = \"run -p tasks --\"\n")).unwrap(),
            Alias::Own("run -p tasks --".to_string())
        );
    }

    /// The new manifest is a virtual workspace of xtask with a version,
    /// one a second copy leaves alone.
    #[test]
    fn a_directory_without_a_manifest_gets_a_workspace_of_xtask() {
        assert_eq!(manifest(NEW_MANIFEST, None).unwrap(), None);
        assert!(wants_default_bin(NEW_MANIFEST));
        let doc: DocumentMut = NEW_MANIFEST.parse().unwrap();
        assert_eq!(
            doc["workspace"]["package"]["version"].as_str(),
            Some("0.1.0")
        );
    }

    /// `/target` is added to a missing, plain, or unterminated
    /// `.gitignore`, and never twice.
    #[test]
    fn target_is_ignored_once() {
        assert_eq!(gitignore(None).as_deref(), Some("/target\n"));
        assert_eq!(
            gitignore(Some("*.log")).as_deref(),
            Some("*.log\n/target\n")
        );
        assert_eq!(
            gitignore(Some("*.log\n")).as_deref(),
            Some("*.log\n/target\n")
        );
        for has in ["target\n", "/target\n", " target/ \n", "a\n/target/\n"] {
            assert_eq!(gitignore(Some(has)), None, "{has:?}");
        }
    }

    /// The glob is cargo's over one segment.
    #[test]
    fn the_glob_is_cargos() {
        assert!(glob("*", "xtask"));
        assert!(glob("x*", "xtask"));
        assert!(glob("?task", "xtask"));
        assert!(!glob("crates/*", "xtask"));
        assert!(!glob("tool", "xtask"));
    }
}

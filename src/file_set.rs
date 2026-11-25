//! The file set's manifest: `file-set.toml`, the one list of what a
//! set holds.
//!
//! A set is the files an adopter carries and a new workspace starts
//! from. Its manifest sits at the set's root, one `[[file]]` table
//! per entry:
//!
//! ```toml
//! [[file]]
//! path = "CONTRIBUTING.md"
//! kind = "starter"
//! from = "scripts/vc-dr-apply-base/work/CONTRIBUTING.md"
//! ```
//!
//! - `path`: where the entry is written, relative to the workspace
//!   root, a file or a directory, a directory standing for every
//!   file beneath it, at any depth.
//! - `kind`: what a copy does with it ([`Kind`]).
//! - `from`: where its text is read in the set, when that is not
//!   `path`, so a starter's template is not the adopter's own file.
//!
//! [`resolve`] turns an entry's path into the files it stands for
//! under a root.
//!
//! [`parse`] refuses what it does not know, an unknown key or kind, a
//! path that leaves the set, and a path listed twice, since a
//! manifest read loosely copies something other than what it says.

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "init --from, the cycle's next rung, is the first caller"
    )
)]

use std::collections::BTreeSet;
use std::path::Path;

use toml_edit::{DocumentMut, Item, Table};

/// The manifest's file name, at the set's root.
pub const MANIFEST: &str = "file-set.toml";

/// The one top-level key, an array of tables.
const FILE: &str = "file";

/// What a copy does with an entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Kept identical: diffed, and replaced on copy.
    Verbatim,
    /// Written only when absent and never replaced, its shape the
    /// set's and its content the project's.
    Starter,
    /// Copied only when asked for.
    Optional,
}

impl Kind {
    /// Every kind with its spelling, in the order a refusal lists them.
    const ALL: [(&'static str, Kind); 3] = [
        ("verbatim", Kind::Verbatim),
        ("starter", Kind::Starter),
        ("optional", Kind::Optional),
    ];

    /// The kind spelled `s`, `None` for any other word.
    fn from_name(s: &str) -> Option<Self> {
        Self::ALL.iter().find(|(n, _)| *n == s).map(|(_, k)| *k)
    }

    /// The kind's spelling in a manifest.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Verbatim => "verbatim",
            Kind::Starter => "starter",
            Kind::Optional => "optional",
        }
    }
}

/// One entry of a manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Where the entry is written, relative to the workspace root,
    /// with no trailing `/`.
    pub path: String,
    /// What a copy does with it.
    pub kind: Kind,
    /// Where its text is read in the set, `None` when that is `path`.
    pub from: Option<String>,
}

impl Entry {
    /// Where the entry's text is read, relative to the set's root.
    pub fn source(&self) -> &str {
        self.from.as_deref().unwrap_or(&self.path) // OK: an entry with no `from` is read from its own path
    }
}

/// A manifest's entries, in the order it lists them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileSet {
    pub entries: Vec<Entry>,
}

impl FileSet {
    /// The entries of `kind`, in the manifest's order.
    pub fn of_kind(&self, kind: Kind) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(move |e| e.kind == kind)
    }
}

/// Read the manifest at `path`. Every error names the file.
pub fn load(path: &Path) -> Result<FileSet, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Parse a manifest's text.
///
/// - The only top-level key is `file`, written as `[[file]]` tables.
/// - An entry takes `path` and `kind`, both required, and `from`.
/// - A manifest with no entry is refused, since it copies nothing.
pub fn parse(text: &str) -> Result<FileSet, String> {
    let doc: DocumentMut = text.parse().map_err(|e| format!("{e}"))?;
    let mut entries: Vec<Entry> = Vec::new();
    let mut seen = BTreeSet::new();
    for (key, item) in doc.as_table().iter() {
        if key != FILE {
            return Err(format!(
                "unknown key '{key}': a manifest holds [[{FILE}]] tables alone"
            ));
        }
        let Item::ArrayOfTables(tables) = item else {
            return Err(format!(
                "'{FILE}' is not an array of tables, write each entry as [[{FILE}]]"
            ));
        };
        for (i, table) in tables.iter().enumerate() {
            let entry = parse_entry(table).map_err(|e| format!("[[{FILE}]] {}: {e}", i + 1))?;
            if !seen.insert(entry.path.clone()) {
                return Err(format!("'{}' is listed twice", entry.path));
            }
            entries.push(entry);
        }
    }
    if entries.is_empty() {
        return Err(format!("no [[{FILE}]] entry, so the set holds nothing"));
    }
    Ok(FileSet { entries })
}

/// One `[[file]]` table as an entry.
fn parse_entry(table: &Table) -> Result<Entry, String> {
    let mut path = None;
    let mut kind = None;
    let mut from = None;
    for (key, item) in table.iter() {
        let value = item
            .as_str()
            .ok_or_else(|| format!("'{key}' is not a string"))?;
        match key {
            "path" => path = Some(relative(key, value)?),
            "from" => from = Some(relative(key, value)?),
            "kind" => {
                kind = Some(Kind::from_name(value).ok_or_else(|| {
                    let names: Vec<&str> = Kind::ALL.iter().map(|(n, _)| *n).collect();
                    format!("unknown kind '{value}', one of: {}", names.join(", "))
                })?);
            }
            _ => return Err(format!("unknown key '{key}', one of: path, kind, from")),
        }
    }
    let path = path.ok_or("no 'path'")?;
    let kind = kind.ok_or_else(|| format!("'{path}' has no 'kind'"))?;
    Ok(Entry { path, kind, from })
}

/// `value` as a path inside the set: relative, with no `.` or `..`
/// component and no empty one, a trailing `/` dropped. `key` names
/// the field in the refusal.
fn relative(key: &str, value: &str) -> Result<String, String> {
    let trimmed = value.strip_suffix('/').unwrap_or(value); // OK: a directory is written with or without its `/`
    if trimmed.is_empty() {
        return Err(format!("'{key}' is empty"));
    }
    if trimmed.starts_with('/') || trimmed.contains('\\') {
        return Err(format!(
            "{key} '{value}' is not a relative path with '/' separators"
        ));
    }
    if trimmed
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(format!(
            "{key} '{value}' has an empty, '.', or '..' component, and a path stays inside the set"
        ));
    }
    Ok(trimmed.to_string())
}

/// The files `path` stands for under `root`, each relative to `root`
/// with `/` separators, sorted.
///
/// - A file is itself, and a directory is every file beneath it, at
///   any depth, a directory holding none giving none.
/// - A path that is not there gives none, so a set missing a file is
///   a smaller set and not an error.
/// - A symlink is taken as a file and never followed, so the walk
///   stays under `root` and cannot loop.
pub fn resolve(root: &Path, path: &str) -> Result<BTreeSet<String>, String> {
    let mut out = BTreeSet::new();
    let full = root.join(path);
    match std::fs::symlink_metadata(&full) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(format!("{}: {e}", full.display())),
        Ok(meta) if meta.is_dir() => walk(&full, path, &mut out)?,
        Ok(_) => {
            out.insert(path.to_string());
        }
    }
    Ok(out)
}

/// Add every file beneath `dir` to `out`, `rel` being `dir` as the
/// set names it.
fn walk(dir: &Path, rel: &str, out: &mut BTreeSet<String>) -> Result<(), String> {
    let in_dir = |e: std::io::Error| format!("{}: {e}", dir.display());
    for entry in std::fs::read_dir(dir).map_err(in_dir)? {
        let entry = entry.map_err(in_dir)?;
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| format!("{}: a name that is not UTF-8, {name:?}", dir.display()))?;
        let child = format!("{rel}/{name}");
        if entry.file_type().map_err(in_dir)?.is_dir() {
            walk(&entry.path(), &child, out)?;
        } else {
            out.insert(child);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A manifest's entries come back in order, each with its kind,
    /// a `from` kept, a trailing `/` dropped, and the source of an
    /// entry its `from` or else its path.
    #[test]
    fn a_manifest_is_its_entries_in_order() {
        let set = parse(
            r#"
# The set.
[[file]]
path = "AGENTS.md"
kind = "verbatim"

[[file]]
path = "notes/"
kind = "starter"
from = "skel/notes"

[[file]]
path = "custom.md"
kind = "optional"
"#,
        )
        .unwrap();
        let got: Vec<(&str, Kind, &str)> = set
            .entries
            .iter()
            .map(|e| (e.path.as_str(), e.kind, e.source()))
            .collect();
        assert_eq!(
            got,
            [
                ("AGENTS.md", Kind::Verbatim, "AGENTS.md"),
                ("notes", Kind::Starter, "skel/notes"),
                ("custom.md", Kind::Optional, "custom.md"),
            ]
        );
        let starters: Vec<&str> = set
            .of_kind(Kind::Starter)
            .map(|e| e.path.as_str())
            .collect();
        assert_eq!(starters, ["notes"]);
        for (name, kind) in Kind::ALL {
            assert_eq!(kind.name(), name);
        }
    }

    /// Each thing a manifest may not hold is refused, the refusal
    /// naming it.
    #[test]
    fn a_manifest_read_loosely_is_refused() {
        let cases = [
            ("", "no [[file]] entry"),
            ("files = 1\n", "unknown key 'files'"),
            ("file = 1\n", "not an array of tables"),
            ("[[file]]\nkind = \"verbatim\"\n", "[[file]] 1: no 'path'"),
            ("[[file]]\npath = \"a\"\n", "'a' has no 'kind'"),
            (
                "[[file]]\npath = \"a\"\nkind = \"set\"\n",
                "unknown kind 'set', one of: verbatim, starter, optional",
            ),
            (
                "[[file]]\npath = \"a\"\nkind = \"starter\"\nmode = \"x\"\n",
                "unknown key 'mode'",
            ),
            (
                "[[file]]\npath = 1\nkind = \"starter\"\n",
                "'path' is not a string",
            ),
            (
                "[[file]]\npath = \"\"\nkind = \"starter\"\n",
                "'path' is empty",
            ),
            (
                "[[file]]\npath = \"/etc\"\nkind = \"starter\"\n",
                "not a relative path",
            ),
            (
                "[[file]]\npath = \"a/../b\"\nkind = \"starter\"\n",
                "stays inside the set",
            ),
            (
                "[[file]]\npath = \"a\"\nkind = \"starter\"\nfrom = \"../b\"\n",
                "from '../b'",
            ),
            (
                "[[file]]\npath = \"a\"\nkind = \"starter\"\n[[file]]\npath = \"a/\"\nkind = \"verbatim\"\n",
                "'a' is listed twice",
            ),
            ("[[file]\n", "TOML parse error"),
        ];
        for (text, want) in cases {
            let err = parse(text).unwrap_err();
            assert!(err.contains(want), "{text:?}: {err:?} lacks {want:?}");
        }
    }

    /// A file resolves to itself, a directory to every file beneath
    /// it at any depth, and a path that is not there to nothing.
    #[test]
    fn a_directory_is_every_file_beneath_it() {
        let root = crate::test_helpers::unique_base("file-set-resolve");
        for file in ["one.md", "dir/a.md", "dir/sub/b.md", "dir/sub/deep/c.md"] {
            let path = root.join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "x").unwrap();
        }
        std::fs::create_dir_all(root.join("dir/empty")).unwrap();
        let got =
            |path: &str| -> Vec<String> { resolve(&root, path).unwrap().into_iter().collect() };
        assert_eq!(got("one.md"), ["one.md"]);
        assert_eq!(
            got("dir"),
            ["dir/a.md", "dir/sub/b.md", "dir/sub/deep/c.md"]
        );
        assert_eq!(got("dir/sub"), ["dir/sub/b.md", "dir/sub/deep/c.md"]);
        assert!(got("dir/empty").is_empty());
        assert!(got("absent").is_empty());
        std::fs::remove_dir_all(&root).unwrap();
    }

    /// A manifest that cannot be read is an error naming its path.
    #[test]
    fn a_missing_manifest_names_its_path() {
        let err = load(Path::new("no-such-dir/file-set.toml")).unwrap_err();
        assert!(err.starts_with("no-such-dir/file-set.toml: "), "{err}");
    }

    /// vc-x1's own manifest parses, every entry's source is in the
    /// tree, and it lists the set's fixed members with their kinds.
    #[test]
    fn our_own_manifest_names_what_is_there() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let set = load(&root.join(MANIFEST)).unwrap();
        for e in &set.entries {
            assert!(
                root.join(e.source()).exists(),
                "{} is read from {}, which is not there",
                e.path,
                e.source()
            );
        }
        let kind_of = |path: &str| set.entries.iter().find(|e| e.path == path).map(|e| e.kind);
        assert_eq!(kind_of("AGENTS.md"), Some(Kind::Verbatim));
        assert_eq!(kind_of("agent-data"), Some(Kind::Verbatim));
        assert_eq!(kind_of("CONTRIBUTING.md"), Some(Kind::Starter));
        assert_eq!(kind_of("custom.md"), Some(Kind::Optional));
    }
}

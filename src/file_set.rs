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
//! Before the entries a manifest may hold `gitignore`, a list of the
//! lines a workspace's `.gitignore` needs for the set, `/custom.md`
//! for one, which [`add_lines`] gives a file that lacks them.
//!
//! [`resolve`] turns an entry's path into the files it stands for
//! under a root.
//!
//! [`files_of`] pairs each file of an entry with where a copy writes
//! it, and [`copy_file`] copies one, replacing or keeping what is
//! there as its caller says. What a kind does with those is the
//! caller's, so the module holds the mechanism alone.
//!
//! [`compare`] says how the files a list of paths stands for differ
//! between two roots, and [`covers`] whether a path is one of them.
//!
//! [`parse`] refuses what it does not know, an unknown key or kind, a
//! path that leaves the set, and a path listed twice, since a
//! manifest read loosely copies something other than what it says.

use std::collections::BTreeSet;
use std::path::Path;

use toml_edit::{DocumentMut, Item, Table};

/// The manifest's file name, at the set's root.
pub const MANIFEST: &str = "file-set.toml";

/// The top-level key of the entries, an array of tables.
const FILE: &str = "file";

/// The top-level key of the `.gitignore` lines, a list of strings.
const GITIGNORE: &str = "gitignore";

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

/// A manifest: its entries and its `.gitignore` lines, each in the
/// order it lists them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileSet {
    pub entries: Vec<Entry>,
    /// The lines a workspace's `.gitignore` needs for the set.
    pub gitignore: Vec<String>,
}

/// Read the manifest at `path`. Every error names the file.
pub fn load(path: &Path) -> Result<FileSet, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Parse a manifest's text.
///
/// - The top-level keys are `file`, written as `[[file]]` tables,
///   and `gitignore`, a list of lines.
/// - An entry takes `path` and `kind`, both required, and `from`.
/// - A manifest with no entry is refused, since it copies nothing.
pub fn parse(text: &str) -> Result<FileSet, String> {
    let doc: DocumentMut = text.parse().map_err(|e| format!("{e}"))?;
    let mut entries: Vec<Entry> = Vec::new();
    let mut gitignore = Vec::new();
    let mut seen = BTreeSet::new();
    for (key, item) in doc.as_table().iter() {
        if key == GITIGNORE {
            gitignore = parse_lines(item)?;
            continue;
        }
        if key != FILE {
            return Err(format!(
                "unknown key '{key}': a manifest holds '{GITIGNORE}' and [[{FILE}]] tables alone"
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
    Ok(FileSet { entries, gitignore })
}

/// The `gitignore` list: strings, each one line, none empty, and
/// none twice.
fn parse_lines(item: &Item) -> Result<Vec<String>, String> {
    let array = item
        .as_array()
        .ok_or_else(|| format!("'{GITIGNORE}' is not a list of strings"))?;
    let mut lines: Vec<String> = Vec::new();
    for value in array.iter() {
        let line = value
            .as_str()
            .ok_or_else(|| format!("'{GITIGNORE}' holds {value}, which is not a string"))?;
        if line.trim().is_empty() || line.contains('\n') {
            return Err(format!(
                "'{GITIGNORE}' holds {line:?}, and each is one line that is not empty"
            ));
        }
        if lines.iter().any(|l| l == line) {
            return Err(format!("'{GITIGNORE}' holds '{line}' twice"));
        }
        lines.push(line.to_string());
    }
    Ok(lines)
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

/// Give the file at `path` each of `lines` it lacks, appended in
/// order, a line there already, whatever its surrounding spaces,
/// kept as it is. A file that is not there is made. Returns how many
/// lines were added.
pub fn add_lines(path: &Path, lines: &[String]) -> Result<usize, String> {
    let at = |e: std::io::Error| format!("{}: {e}", path.display());
    let existing = match std::fs::read_to_string(path) {
        Ok(existing) => existing,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(at(e)),
    };
    let missing: Vec<&String> = lines
        .iter()
        .filter(|line| !existing.lines().any(|l| l.trim() == line.as_str()))
        .collect();
    if missing.is_empty() {
        return Ok(0);
    }
    let sep = if existing.is_empty() || existing.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    let added: String = missing.iter().map(|l| format!("{l}\n")).collect();
    std::fs::write(path, format!("{existing}{sep}{added}")).map_err(at)?;
    Ok(missing.len())
}

/// How one file compares across two roots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Compared {
    Same,
    Differs,
    OnlyInA,
    OnlyInB,
}

/// Every file `paths` stand for under `a` or `b`, with how the two
/// sides compare, sorted by path. Each path is resolved on both
/// sides, a directory at every depth, so a file only one side has
/// is found.
pub fn compare(a: &Path, b: &Path, paths: &[&str]) -> Result<Vec<(String, Compared)>, String> {
    let (mut in_a, mut in_b) = (BTreeSet::new(), BTreeSet::new());
    for path in paths {
        in_a.extend(resolve(a, path)?);
        in_b.extend(resolve(b, path)?);
    }
    let read = |root: &Path, file: &str| {
        std::fs::read(root.join(file)).map_err(|e| format!("{}: {e}", root.join(file).display()))
    };
    let mut out = Vec::new();
    for file in in_a.union(&in_b) {
        let state = match (in_a.contains(file), in_b.contains(file)) {
            (true, false) => Compared::OnlyInA,
            (false, true) => Compared::OnlyInB,
            _ if read(a, file)? == read(b, file)? => Compared::Same,
            _ => Compared::Differs,
        };
        out.push((file.clone(), state));
    }
    Ok(out)
}

/// Whether `file` is one `path` stands for: `path` itself, or a file
/// beneath it when `path` is a directory.
pub fn covers(path: &str, file: &str) -> bool {
    file.strip_prefix(path)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

/// One file of an entry, as a copy takes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileOf {
    /// Where it is read, relative to the set's root.
    pub from: String,
    /// Where it is written, relative to the destination's root.
    pub to: String,
}

/// The files `entry` stands for in the set at `root`, each with where
/// a copy writes it: the entry's path, and for a directory the file's
/// path beneath it. Sorted by where they are read.
pub fn files_of(root: &Path, entry: &Entry) -> Result<Vec<FileOf>, String> {
    let source = entry.source();
    resolve(root, source)?
        .into_iter()
        .map(|from| {
            let beneath = from
                .strip_prefix(source)
                .ok_or_else(|| format!("'{from}' is not under '{source}'"))?;
            Ok(FileOf {
                to: format!("{}{beneath}", entry.path),
                from,
            })
        })
        .collect()
}

/// What became of a file a copy took.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Copied {
    /// Nothing was there, and it is written.
    Written,
    /// A different file was there, and it is replaced.
    Replaced,
    /// A different file was there, and it is kept.
    Kept,
    /// The same bytes were there already.
    Same,
}

/// What a copy does to a file's text before it is compared or
/// written, a template's placeholders filled.
pub type Fill<'a> = &'a dyn Fn(&str) -> String;

/// Copy the file `from` to `to`, making the directories `to` needs.
///
/// - `replace`: whether a different file at `to` is replaced or kept.
/// - `fill`: applied to the text before it is compared or written,
///   for a template's placeholders. A file that is not UTF-8 is
///   copied as it is.
/// - The copy has the source's permissions, so a script stays one.
/// - Anything at `to` but a plain file, a directory or a symlink, is
///   refused, never written through.
pub fn copy_file(
    from: &Path,
    to: &Path,
    replace: bool,
    fill: Option<Fill>,
) -> Result<Copied, String> {
    let at = |path: &Path, e: std::io::Error| format!("{}: {e}", path.display());
    let bytes = std::fs::read(from).map_err(|e| at(from, e))?;
    let bytes = match (fill, std::str::from_utf8(&bytes)) {
        (Some(fill), Ok(text)) => fill(text).into_bytes(),
        _ => bytes,
    };
    let outcome = match std::fs::symlink_metadata(to) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Copied::Written,
        Err(e) => return Err(at(to, e)),
        Ok(meta) if !meta.is_file() => {
            return Err(format!("{} is there and is not a plain file", to.display()));
        }
        Ok(_) if std::fs::read(to).map_err(|e| at(to, e))? == bytes => return Ok(Copied::Same),
        Ok(_) if replace => Copied::Replaced,
        Ok(_) => return Ok(Copied::Kept),
    };
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).map_err(|e| at(parent, e))?;
    }
    std::fs::write(to, &bytes).map_err(|e| at(to, e))?;
    let permissions = std::fs::metadata(from)
        .map_err(|e| at(from, e))?
        .permissions();
    std::fs::set_permissions(to, permissions).map_err(|e| at(to, e))?;
    Ok(outcome)
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
gitignore = ["/tmp", "/custom.md"]

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
        for (name, kind) in Kind::ALL {
            assert_eq!(kind.name(), name);
        }
        assert_eq!(set.gitignore, ["/tmp", "/custom.md"]);
    }

    /// An entry's files are written at its path, a directory's beneath
    /// it, wherever in the set they are read.
    #[test]
    fn an_entrys_files_are_written_at_its_path() {
        let root = crate::test_helpers::unique_base("file-set-files-of");
        for file in ["skel/notes/a.md", "skel/notes/sub/b.md", "LICENSE"] {
            let path = root.join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "x").unwrap();
        }
        let entry = |path: &str, from: Option<&str>| Entry {
            path: path.to_string(),
            kind: Kind::Starter,
            from: from.map(str::to_string),
        };
        let pairs = |e: &Entry| -> Vec<(String, String)> {
            files_of(&root, e)
                .unwrap()
                .into_iter()
                .map(|f| (f.from, f.to))
                .collect()
        };
        assert_eq!(
            pairs(&entry("notes", Some("skel/notes"))),
            [
                ("skel/notes/a.md".to_string(), "notes/a.md".to_string()),
                (
                    "skel/notes/sub/b.md".to_string(),
                    "notes/sub/b.md".to_string()
                ),
            ]
        );
        assert_eq!(
            pairs(&entry("LICENSE", None)),
            [("LICENSE".to_string(), "LICENSE".to_string())]
        );
        assert_eq!(
            pairs(&entry("README.md", Some("skel/notes/a.md"))),
            [("skel/notes/a.md".to_string(), "README.md".to_string())]
        );
        assert!(pairs(&entry("absent", None)).is_empty());
        std::fs::remove_dir_all(&root).unwrap();
    }

    /// A copy writes what is absent, replaces or keeps what differs as
    /// it is told, leaves the same bytes alone, fills the text first,
    /// and refuses a directory where the file goes.
    #[test]
    fn a_copy_writes_replaces_or_keeps() {
        let root = crate::test_helpers::unique_base("file-set-copy");
        std::fs::create_dir_all(&root).unwrap();
        let from = root.join("from.md");
        std::fs::write(&from, "# {name}\n").unwrap();
        let to = root.join("deep/er/to.md");
        let read = |p: &Path| std::fs::read_to_string(p).unwrap();

        assert_eq!(copy_file(&from, &to, false, None), Ok(Copied::Written));
        assert_eq!(read(&to), "# {name}\n");
        assert_eq!(copy_file(&from, &to, true, None), Ok(Copied::Same));

        std::fs::write(&to, "theirs\n").unwrap();
        assert_eq!(copy_file(&from, &to, false, None), Ok(Copied::Kept));
        assert_eq!(read(&to), "theirs\n");
        assert_eq!(copy_file(&from, &to, true, None), Ok(Copied::Replaced));
        assert_eq!(read(&to), "# {name}\n");

        let fill = |text: &str| text.replace("{name}", "hello");
        assert_eq!(
            copy_file(&from, &to, true, Some(&fill)),
            Ok(Copied::Replaced)
        );
        assert_eq!(read(&to), "# hello\n");
        assert_eq!(copy_file(&from, &to, true, Some(&fill)), Ok(Copied::Same));

        let err = copy_file(&from, &root.join("deep"), true, None).unwrap_err();
        assert!(err.contains("not a plain file"), "{err}");
        std::fs::remove_dir_all(&root).unwrap();
    }

    /// Two roots compare by the files a list of paths stands for on
    /// either side, a directory at every depth, a file outside the
    /// list left out, and a path covers itself and what is beneath it.
    #[test]
    fn two_roots_compare_by_a_list_of_paths() {
        let base = crate::test_helpers::unique_base("file-set-compare");
        let (a, b) = (base.join("a"), base.join("b"));
        let seed = |root: &Path, files: &[(&str, &str)]| {
            for (file, text) in files {
                let path = root.join(file);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, text).unwrap();
            }
        };
        seed(
            &a,
            &[
                ("one.md", "same\n"),
                ("dir/x.md", "new\n"),
                ("dir/sub/y.md", "a's\n"),
                ("other.md", "unlisted\n"),
            ],
        );
        seed(
            &b,
            &[
                ("one.md", "same\n"),
                ("dir/x.md", "old\n"),
                ("dir/sub/z.md", "b's\n"),
            ],
        );
        let rows = compare(&a, &b, &["one.md", "dir", "absent"]).unwrap();
        assert_eq!(
            rows,
            [
                ("dir/sub/y.md".to_string(), Compared::OnlyInA),
                ("dir/sub/z.md".to_string(), Compared::OnlyInB),
                ("dir/x.md".to_string(), Compared::Differs),
                ("one.md".to_string(), Compared::Same),
            ]
        );
        assert!(covers("dir", "dir/sub/y.md"));
        assert!(covers("one.md", "one.md"));
        assert!(!covers("dir", "dirt/x.md"));
        assert!(!covers("one.md", "one.md.bak"));
        std::fs::remove_dir_all(&base).unwrap();
    }

    /// A file is given the lines it lacks and no other, after a
    /// newline it lacked, and a file that is not there is made.
    #[test]
    fn a_file_is_given_the_lines_it_lacks() {
        let root = crate::test_helpers::unique_base("file-set-add-lines");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join(".gitignore");
        let lines = ["/tmp".to_string(), "/custom.md".to_string()];
        assert_eq!(add_lines(&path, &lines), Ok(2));
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "/tmp\n/custom.md\n"
        );
        assert_eq!(add_lines(&path, &lines), Ok(0));

        std::fs::write(&path, "*.log\n  /tmp  ").unwrap();
        assert_eq!(add_lines(&path, &lines), Ok(1));
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "*.log\n  /tmp  \n/custom.md\n"
        );
        std::fs::remove_dir_all(&root).unwrap();
    }

    /// A copy has its source's permissions.
    #[cfg(unix)]
    #[test]
    fn a_copy_keeps_the_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let root = crate::test_helpers::unique_base("file-set-copy-mode");
        std::fs::create_dir_all(&root).unwrap();
        let from = root.join("run.sh");
        std::fs::write(&from, "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&from, std::fs::Permissions::from_mode(0o755)).unwrap();
        let to = root.join("copy.sh");
        copy_file(&from, &to, true, None).unwrap();
        assert_eq!(
            std::fs::metadata(&to).unwrap().permissions().mode() & 0o777,
            0o755
        );
        std::fs::remove_dir_all(&root).unwrap();
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
            (
                "gitignore = \"/tmp\"\n",
                "'gitignore' is not a list of strings",
            ),
            ("gitignore = [1]\n", "which is not a string"),
            ("gitignore = [\" \"]\n", "one line that is not empty"),
            (
                "gitignore = [\"/a\", \"/a\"]\n",
                "'gitignore' holds '/a' twice",
            ),
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
        // An optional file is local, custom.md gitignored, so a clone
        // need not hold it.
        for e in set.entries.iter().filter(|e| e.kind != Kind::Optional) {
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
        assert!(
            set.gitignore.iter().any(|l| l == "/custom.md"),
            "custom.md is local, so the set ignores it"
        );
    }
}

//! No code writes the process's `HOME` (bug 21 in notes/bugs.md).
//!
//! Tests run as threads of one process, so a test that sets `HOME`
//! changes it for every test running beside it: git looks for the
//! signing key under the wrong directory, and the jj children the
//! sync tests spawn find no config. Clippy bans `set_var` and
//! `remove_var` outright (clippy.toml), and this test names `HOME`
//! itself, so a granted `#[allow]` cannot reach it either.

use std::path::{Path, PathBuf};

/// The forbidden calls, whitespace removed. Split by `concat!` so
/// this file's own text does not match them.
const FORBIDDEN: [&str; 2] = [
    concat!("set_var(", "\"HOME\""),
    concat!("remove_var(", "\"HOME\""),
];

#[test]
fn no_test_writes_home() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for dir in ["src", "tests"] {
        rust_files(&root.join(dir), &mut files);
    }
    assert!(
        !files.is_empty(),
        "no .rs files found under {}",
        root.display()
    );

    let mut hits = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("read {}: {e}", file.display()));
        for (i, line) in text.lines().enumerate() {
            let squeezed: String = line.chars().filter(|c| !c.is_whitespace()).collect();
            if FORBIDDEN.iter().any(|f| squeezed.contains(f)) {
                let rel = file.strip_prefix(root).unwrap_or(file);
                hits.push(format!("{}:{}: {}", rel.display(), i + 1, line.trim()));
            }
        }
    }
    assert!(
        hits.is_empty(),
        "code writes HOME, which every test running beside it reads (bug 21):\n{}",
        hits.join("\n")
    );
}

/// Every `.rs` file under `dir`, recursively.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries =
        std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read dir {}: {e}", dir.display()));
    for entry in entries {
        let path = entry
            .unwrap_or_else(|e| panic!("read entry in {}: {e}", dir.display()))
            .path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

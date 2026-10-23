//! Read the commit a git directory's `HEAD` names, from the files
//! alone: no spawn and no library, since `build.rs` uses it.
//!
//! - In a jj colocated repo `HEAD` is detached at `@-`, a bare commit
//!   id, which jj moves on every operation that changes `@`'s parent.
//! - In a plain git checkout `HEAD` is `ref: refs/heads/<branch>`,
//!   resolved through the loose ref file or `packed-refs`.
//! - A `.git` that is a file, as in a git worktree, names the real
//!   directory with `gitdir: <path>`.
//!
//! `build.rs` compiles this file by `#[path]` for the fallback commit
//! a build carries when `vc-x1` did not pass one, and the crate
//! compiles it for its tests alone.

use std::path::{Path, PathBuf};

/// The git directory for a checkout at `root`: `root/.git` when it
/// is a directory, the `gitdir:` it names when it is a file, `None`
/// when there is neither.
pub fn git_dir(root: &Path) -> Option<PathBuf> {
    let dot_git = root.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git);
    }
    let text = std::fs::read_to_string(&dot_git).ok()?;
    let target = text.trim().strip_prefix("gitdir:")?.trim();
    let path = Path::new(target);
    Some(if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    })
}

/// What `HEAD`'s text says: a commit id, or a ref to resolve.
#[derive(Debug, PartialEq, Eq)]
pub enum Head {
    /// A detached `HEAD`: the commit id itself.
    Detached(String),
    /// A symbolic `HEAD`: the ref it names, `refs/heads/main`.
    Ref(String),
}

/// Parse `HEAD`'s text, `None` when it is neither form.
pub fn parse_head(text: &str) -> Option<Head> {
    let text = text.trim();
    if let Some(name) = text.strip_prefix("ref:") {
        let name = name.trim();
        return (!name.is_empty()).then(|| Head::Ref(name.to_string()));
    }
    is_commit_id(text).then(|| Head::Detached(text.to_string()))
}

/// True for a full commit id: 40 hex digits (SHA-1) or 64 (SHA-256).
pub fn is_commit_id(s: &str) -> bool {
    (s.len() == 40 || s.len() == 64) && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// The commit `name` points at in `packed-refs` text, `None` when it
/// is not listed.
pub fn packed_ref(text: &str, name: &str) -> Option<String> {
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with('^'))
        .find_map(|l| {
            let (id, rest) = l.split_once(' ')?;
            (rest.trim() == name && is_commit_id(id)).then(|| id.to_string())
        })
}

/// The commit `HEAD` names in the git directory `git_dir`, and the
/// files read to find it, so a build script can rerun when any
/// changes. The commit is `None` when it cannot be found.
pub fn head_commit(git_dir: &Path) -> (Option<String>, Vec<PathBuf>) {
    let head_path = git_dir.join("HEAD");
    let mut read = vec![head_path.clone()];
    let Some(head) = std::fs::read_to_string(&head_path)
        .ok()
        .and_then(|t| parse_head(&t))
    else {
        return (None, read);
    };
    match head {
        Head::Detached(id) => (Some(id), read),
        Head::Ref(name) => {
            let loose = git_dir.join(&name);
            if let Ok(text) = std::fs::read_to_string(&loose) {
                read.push(loose);
                let id = text.trim();
                return (is_commit_id(id).then(|| id.to_string()), read);
            }
            let packed = git_dir.join("packed-refs");
            let id = std::fs::read_to_string(&packed)
                .ok()
                .and_then(|t| packed_ref(&t, &name));
            if packed.exists() {
                read.push(packed);
            }
            (id, read)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "fcc7f222a7746dc7cf0e8303daa7c1db49199fbf";

    /// Unique tempdir.
    fn tempdir(tag: &str) -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("vc-x1-git-head-{tag}-{ts}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn parse_head_forms() {
        assert_eq!(
            parse_head(&format!("{ID}\n")),
            Some(Head::Detached(ID.to_string()))
        );
        assert_eq!(
            parse_head("ref: refs/heads/main\n"),
            Some(Head::Ref("refs/heads/main".to_string()))
        );
        assert_eq!(parse_head("ref:"), None);
        assert_eq!(parse_head("not a commit"), None);
        assert_eq!(parse_head(&ID[..12]), None);
    }

    #[test]
    fn packed_ref_finds_the_named_ref() {
        let text = format!(
            "# pack-refs with: peeled fully-peeled sorted\n\
             {ID} refs/heads/main\n\
             ^0123456789abcdef0123456789abcdef01234567\n\
             0123456789abcdef0123456789abcdef01234567 refs/tags/v1\n"
        );
        assert_eq!(packed_ref(&text, "refs/heads/main"), Some(ID.to_string()));
        assert_eq!(packed_ref(&text, "refs/heads/other"), None);
    }

    /// A detached `HEAD`, jj's colocated form, is the commit itself,
    /// and `HEAD` is the one file read.
    #[test]
    fn detached_head_is_the_commit() {
        let git = tempdir("detached");
        std::fs::write(git.join("HEAD"), format!("{ID}\n")).unwrap();
        let (id, read) = head_commit(&git);
        assert_eq!(id.as_deref(), Some(ID));
        assert_eq!(read, vec![git.join("HEAD")]);
        std::fs::remove_dir_all(&git).ok();
    }

    /// A symbolic `HEAD` resolves through the loose ref, else
    /// `packed-refs`, and every file read is reported.
    #[test]
    fn symbolic_head_resolves() {
        let git = tempdir("symbolic");
        std::fs::write(git.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        std::fs::write(git.join("packed-refs"), format!("{ID} refs/heads/main\n")).unwrap();
        let (id, read) = head_commit(&git);
        assert_eq!(id.as_deref(), Some(ID));
        assert!(read.contains(&git.join("packed-refs")), "{read:?}");
        std::fs::create_dir_all(git.join("refs/heads")).unwrap();
        let other = "0123456789abcdef0123456789abcdef01234567";
        std::fs::write(git.join("refs/heads/main"), format!("{other}\n")).unwrap();
        let (id, read) = head_commit(&git);
        assert_eq!(id.as_deref(), Some(other), "the loose ref wins");
        assert!(read.contains(&git.join("refs/heads/main")), "{read:?}");
        std::fs::remove_dir_all(&git).ok();
    }

    /// No `.git` is no git directory, and a `.git` file names one.
    #[test]
    fn git_dir_forms() {
        let root = tempdir("gitdir");
        assert_eq!(git_dir(&root), None);
        std::fs::write(root.join(".git"), "gitdir: elsewhere/.git\n").unwrap();
        assert_eq!(git_dir(&root), Some(root.join("elsewhere/.git")));
        std::fs::remove_file(root.join(".git")).unwrap();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        assert_eq!(git_dir(&root), Some(root.join(".git")));
        std::fs::remove_dir_all(&root).ok();
    }

    /// This checkout's own `HEAD` names a full commit id.
    #[test]
    fn this_checkout_has_a_head_commit() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let Some(git) = git_dir(root) else {
            return;
        };
        let (id, _) = head_commit(&git);
        assert!(id.as_deref().is_some_and(is_commit_id), "{id:?}");
    }
}

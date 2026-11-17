//! The build's provenance, `XTASK_BUILD_COMMIT` and
//! `XTASK_BUILD_STATE`, read through `jj`: `@-`'s commit, and `clean`
//! when `@` is empty, `dirty` when not.
//!
//! A workspace's `build.rs` may read them, to compile
//! `<version>+<commit>.dirty` into a banner, say, rerunning when they
//! change, so every task that runs cargo sets the same values and none
//! recompiles for another. A workspace that reads them not is
//! untouched.
//!
//! - A workspace with no jj repo, or a machine with no `jj`, has none,
//!   quietly, its `build.rs` left to read git's `HEAD` itself.
//! - A jj repo whose provenance cannot be read is a warning, and the
//!   build carries none.

use std::path::Path;
use std::process::Command;

use crate::util::command;

/// The variable carrying the build's commit to cargo.
pub const COMMIT_VAR: &str = "XTASK_BUILD_COMMIT";

/// The variable carrying the state of the source code being compiled.
pub const STATE_VAR: &str = "XTASK_BUILD_STATE";

/// The template `jj log` reads the provenance with: the state, then
/// `@`'s parents.
const TEMPLATE: &str =
    r#"if(empty, "clean", "dirty") ++ " " ++ parents.map(|c| c.commit_id()).join(" ")"#;

/// What a build is made from.
#[derive(Debug, PartialEq, Eq)]
pub struct Provenance {
    /// The full commit id of the work repo's `@-`.
    pub commit: String,
    /// `clean` when `@` is empty, else `dirty`.
    pub state: &'static str,
}

impl Provenance {
    /// Read the provenance of the work repo at `root`, none with no jj
    /// repo or no `jj`, and a warning with none when jj fails.
    pub fn read(root: &Path) -> Option<Provenance> {
        if !root.join(".jj").is_dir() {
            return None;
        }
        let out = command("jj")
            .arg("-R")
            .arg(root)
            .args([
                "--color",
                "never",
                "--no-pager",
                "log",
                "-r",
                "@",
                "--no-graph",
            ])
            .args(["-T", TEMPLATE])
            .output();
        let read = match out {
            Ok(out) if out.status.success() => parse(&String::from_utf8_lossy(&out.stdout)),
            Ok(out) => Err(format!(
                "jj failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
            Err(e) => Err(format!("jj did not run: {e}")),
        };
        match read {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("xtask: warning: the provenance: {e}, so the build carries no commit");
                None
            }
        }
    }

    /// Put the two variables in `cmd`'s environment.
    pub fn set_env(&self, cmd: &mut Command) {
        cmd.env(COMMIT_VAR, &self.commit).env(STATE_VAR, self.state);
    }
}

/// The provenance in `jj log`'s output for [`TEMPLATE`], refused
/// unless `@` has one parent, whose commit is the build's.
fn parse(out: &str) -> Result<Provenance, String> {
    let mut words = out.split_whitespace();
    let state = match words.next() {
        Some("clean") => "clean",
        Some("dirty") => "dirty",
        _ => return Err(format!("jj printed '{}', not a state", out.trim())),
    };
    let parents: Vec<&str> = words.collect();
    match parents.as_slice() {
        [commit] if commit.len() == 40 && commit.chars().all(|c| c.is_ascii_hexdigit()) => {
            Ok(Provenance {
                commit: commit.to_string(),
                state,
            })
        }
        [commit] => Err(format!("jj printed '{commit}', not a commit id")),
        _ => Err(format!("`@` has {} parents, not one", parents.len())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The provenance is the state and `@`'s one parent, and anything
    /// else is refused, saying what jj printed.
    #[test]
    fn the_provenance_is_read_from_jj() {
        let id = "a5026b80c1b2".repeat(4)[..40].to_string();
        assert_eq!(
            parse(&format!("dirty {id}\n")).unwrap(),
            Provenance {
                commit: id.clone(),
                state: "dirty"
            }
        );
        assert_eq!(parse(&format!("clean {id}")).unwrap().state, "clean");
        let merge = parse(&format!("clean {id} {id}")).unwrap_err();
        assert!(merge.contains("2 parents"), "{merge}");
        assert!(parse("clean").unwrap_err().contains("0 parents"));
        assert!(parse("clean xyz").unwrap_err().contains("not a commit id"));
        assert!(parse("").unwrap_err().contains("not a state"));
    }
}

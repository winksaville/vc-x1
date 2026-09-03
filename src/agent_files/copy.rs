//! `agent-files copy [SRC] [DST]`: make the set in DST a byte copy
//! of the one in SRC, and leave it uncommitted.
//!
//! - `CopyArgs`: clap surface, the `SRC` and `DST` operands and the
//!   `-c`/`--custom` / `--no-custom` pair, as `diff` has them.
//! - `plan`: what a copy would do, from `diff::compare`'s rows:
//!   copy what differs or is only in SRC, delete what is only in
//!   DST, and never touch custom.md unless asked, or TODO.md at
//!   all.
//! - `apply`: do it. The jj working copy around DST must be clean
//!   in the set's paths first, so the copy's changes are the only
//!   ones there and a `jj diff` reads as the re-sync.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Args;
use log::{error, info};

use super::diff::{self, AGENT_DATA, State};
use crate::common;

/// CLI args for `agent-files copy`.
#[derive(Args, Debug)]
pub struct CopyArgs {
    /// The directory to copy the set from [default:
    /// agent-files.copy.dir, else family.template, else an error]
    #[arg(value_name = "SRC")]
    pub src: Option<PathBuf>,

    /// The directory whose set becomes the copy [default: this
    /// workspace, so one operand is SRC]
    #[arg(value_name = "DST")]
    pub dst: Option<PathBuf>,

    /// Copy custom.md with the rest of the set
    #[arg(short = 'c', long = "custom", conflicts_with = "no_custom")]
    pub custom: bool,

    /// Leave custom.md alone, overriding agent-files.copy.custom
    #[arg(long = "no-custom")]
    pub no_custom: bool,
}

/// One step of a copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Copy(String),
    Delete(String),
}

impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Step::Copy(p) => write!(f, "copy   {p}"),
            Step::Delete(p) => write!(f, "delete {p}"),
        }
    }
}

/// The steps that make `dst`'s set equal to `src`'s, in path
/// order: a copy for each file that differs or exists only in
/// `src`, a delete for each that exists only in `dst`. Nothing for
/// the project layer when it is not asked for.
pub fn plan(src: &Path, dst: &Path, custom: bool) -> Result<Vec<Step>, Box<dyn std::error::Error>> {
    Ok(diff::compare(src, dst, custom)?
        .into_iter()
        .filter_map(|(path, state)| match state {
            State::Differs | State::OnlyInA => Some(Step::Copy(path)),
            State::OnlyInB => Some(Step::Delete(path)),
            State::Same | State::ProjectLayer => None,
        })
        .collect())
}

/// Run the steps: copy creates `agent-data/` when it is missing,
/// delete removes the file only.
pub fn apply(src: &Path, dst: &Path, steps: &[Step]) -> Result<(), Box<dyn std::error::Error>> {
    for step in steps {
        match step {
            Step::Copy(path) => {
                if let Some(parent) = dst.join(path).parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(src.join(path), dst.join(path))
                    .map_err(|e| format!("copy {path}: {e}"))?;
            }
            Step::Delete(path) => {
                std::fs::remove_file(dst.join(path)).map_err(|e| format!("delete {path}: {e}"))?;
            }
        }
    }
    Ok(())
}

/// True when `path`, relative to a set directory, is one the copy
/// may touch: AGENTS.md, anything under `agent-data/`, and
/// custom.md when `custom`.
fn in_set(path: &str, custom: bool) -> bool {
    path == diff::AGENTS_MD
        || path.starts_with(&format!("{AGENT_DATA}/"))
        || (custom && path == diff::CUSTOM_MD)
}

/// The changed paths inside `dst`'s set in the jj working copy
/// around it, which a copy would mix its own changes with. `None`
/// when `dst` is not inside a jj repo. `dst` may be a subdirectory
/// of the repo, the payload inside the template repo being one, so
/// the repo's paths are matched under `dst`'s prefix.
fn dirty_set_paths(
    dst: &Path,
    custom: bool,
) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
    let dst = dst.canonicalize()?;
    let Some(repo) = crate::status::nearest_jj(&dst) else {
        return Ok(None);
    };
    let repo = repo.canonicalize()?;
    let prefix = dst
        .strip_prefix(&repo)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    let prefix = if prefix.is_empty() {
        prefix
    } else {
        format!("{prefix}/")
    };
    Ok(Some(
        crate::jj::wc_status(&repo)?
            .changes
            .into_iter()
            .filter_map(|(letter, p)| {
                p.strip_prefix(&prefix)
                    .filter(|rel| in_set(rel, custom))
                    .map(|_| format!("{letter} {p}"))
            })
            .collect(),
    ))
}

impl CopyArgs {
    /// Run the copy.
    pub fn run(&self) -> ExitCode {
        match self.copy() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                error!("agent-files copy: {e}");
                ExitCode::FAILURE
            }
        }
    }

    /// Resolve, guard, print the plan, apply it.
    fn copy(&self) -> Result<(), Box<dyn std::error::Error>> {
        let root = common::find_workspace_root();
        let cfg = match &root {
            Some(r) => super::config_at(r)?,
            None => super::WorkspaceAgentFiles::default(),
        };
        let template = match &root {
            Some(r) => diff::family_template(r)?,
            None => None,
        };
        let src = diff::resolve_set_dir(
            root.as_deref(),
            self.src.as_deref(),
            "agent-files.copy.dir",
            cfg.copy.dir.as_deref(),
            template.as_deref(),
        )?;
        let dst = diff::resolve_here(root.as_deref(), self.dst.as_deref())?;
        let custom = diff::resolve_custom(
            self.custom,
            self.no_custom,
            cfg.copy.custom,
            crate::config_schema::AGENT_FILES_COPY_CUSTOM_DEFAULT,
        );
        if src.path.canonicalize()? == dst.path.canonicalize()? {
            return Err(
                format!("'{}' and '{}' are the same directory", src.shown, dst.shown).into(),
            );
        }
        info!(
            "agent-files copy: from {} ({}) into {} ({}){}",
            src.shown,
            src.source,
            dst.shown,
            dst.source,
            if custom { ", custom.md included" } else { "" }
        );
        match dirty_set_paths(&dst.path, custom)? {
            None => info!("{} is not in a jj repo: no working-copy guard", dst.shown),
            Some(dirty) if !dirty.is_empty() => {
                return Err(format!(
                    "the working copy around '{}' already changes the set, commit or restore \
                     first:\n  {}",
                    dst.shown,
                    dirty.join("\n  ")
                )
                .into());
            }
            Some(_) => {}
        }
        let steps = plan(&src.path, &dst.path, custom)?;
        if steps.is_empty() {
            info!("already the same, nothing to do");
            return Ok(());
        }
        for step in &steps {
            info!("{step}");
        }
        apply(&src.path, &dst.path, &steps)?;
        info!(
            "{} step(s) applied, left uncommitted for review",
            steps.len()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::Fixture;
    use clap::Parser;

    #[derive(Parser)]
    struct T {
        #[command(flatten)]
        a: CopyArgs,
    }

    /// Write `(relative path, content)` files under `dir`.
    fn seed(dir: &Path, files: &[(&str, &str)]) {
        std::fs::create_dir_all(dir.join(AGENT_DATA)).expect("mkdir");
        for (path, content) in files {
            std::fs::write(dir.join(path), content).expect("write");
        }
    }

    /// The operands and flags parse, and `-c` with `--no-custom` is
    /// refused.
    #[test]
    fn flags_parse() {
        let a = T::try_parse_from(["t", "../x", "--custom"]).unwrap().a;
        assert_eq!(a.src, Some(PathBuf::from("../x")));
        assert!(a.dst.is_none() && a.custom);
        let a = T::try_parse_from(["t", "../x", "../y"]).unwrap().a;
        assert_eq!(a.dst, Some(PathBuf::from("../y")));
        assert!(T::try_parse_from(["t", "-c", "--no-custom"]).is_err());
    }

    /// A copy plan copies what differs or is only in the source,
    /// deletes what is only in the destination, and leaves custom.md
    /// and TODO.md alone until `-c` brings custom.md in. Applied,
    /// the sets compare equal.
    #[test]
    fn plan_and_apply_mirror_the_set() {
        let base = crate::test_helpers::unique_base("af-copy");
        let src = base.join("src");
        let dst = base.join("dst");
        seed(
            &src,
            &[
                ("AGENTS.md", "new rules\n"),
                ("custom.md", "theirs\n"),
                ("TODO.md", "their todo\n"),
                ("agent-data/a.md", "same\n"),
                ("agent-data/b.md", "new\n"),
            ],
        );
        seed(
            &dst,
            &[
                ("AGENTS.md", "old rules\n"),
                ("custom.md", "mine\n"),
                ("TODO.md", "my todo\n"),
                ("agent-data/a.md", "same\n"),
                ("agent-data/stale.md", "gone\n"),
            ],
        );
        let steps = plan(&src, &dst, false).unwrap();
        assert_eq!(
            steps,
            vec![
                Step::Copy("AGENTS.md".to_string()),
                Step::Copy("agent-data/b.md".to_string()),
                Step::Delete("agent-data/stale.md".to_string()),
            ]
        );
        assert_eq!(steps[0].to_string(), "copy   AGENTS.md");
        assert_eq!(steps[2].to_string(), "delete agent-data/stale.md");
        apply(&src, &dst, &steps).unwrap();
        let rows = diff::compare(&src, &dst, false).unwrap();
        assert!(rows.iter().all(|(_, s)| !s.differs()), "{rows:?}");
        assert_eq!(
            std::fs::read_to_string(dst.join("custom.md")).unwrap(),
            "mine\n"
        );
        assert_eq!(
            std::fs::read_to_string(dst.join("TODO.md")).unwrap(),
            "my todo\n"
        );
        assert!(plan(&src, &dst, false).unwrap().is_empty());
        let steps = plan(&src, &dst, true).unwrap();
        assert_eq!(steps, vec![Step::Copy("custom.md".to_string())]);
        apply(&src, &dst, &steps).unwrap();
        assert_eq!(
            std::fs::read_to_string(dst.join("custom.md")).unwrap(),
            "theirs\n"
        );
        // Outside any jj repo there is no guard to consult.
        assert_eq!(dirty_set_paths(&dst, false).unwrap(), None);
        let _ = std::fs::remove_dir_all(&base);
    }

    /// The guard sees the destination's repo: a working copy that
    /// already changes a set file is refused naming the change, one
    /// that changes only other files is not, custom.md counts only
    /// when it is being copied, and a destination that is a
    /// subdirectory of its repo is matched under its prefix.
    #[test]
    fn dirty_guard_sees_the_destination_set() {
        let fx = Fixture::new("af-copy-guard");
        seed(
            &fx.work,
            &[
                ("AGENTS.md", "rules\n"),
                ("custom.md", "mine\n"),
                ("agent-data/a.md", "a\n"),
            ],
        );
        let sub = fx.work.join("payload");
        seed(
            &sub,
            &[("AGENTS.md", "rules\n"), ("agent-data/p.md", "p\n")],
        );
        crate::jj::commit(&fx.work, "test: seed the sets").unwrap();
        assert_eq!(dirty_set_paths(&fx.work, false).unwrap(), Some(vec![]));
        std::fs::write(fx.work.join("README.md"), "other\n").unwrap();
        std::fs::write(fx.work.join("custom.md"), "edited\n").unwrap();
        assert_eq!(dirty_set_paths(&fx.work, false).unwrap(), Some(vec![]));
        assert_eq!(
            dirty_set_paths(&fx.work, true).unwrap(),
            Some(vec!["M custom.md".to_string()])
        );
        std::fs::write(fx.work.join("agent-data/a.md"), "b\n").unwrap();
        assert_eq!(
            dirty_set_paths(&fx.work, false).unwrap(),
            Some(vec!["M agent-data/a.md".to_string()])
        );
        // The subdirectory's guard sees only its own set.
        assert_eq!(dirty_set_paths(&sub, false).unwrap(), Some(vec![]));
        std::fs::write(sub.join("agent-data/p.md"), "q\n").unwrap();
        assert_eq!(
            dirty_set_paths(&sub, false).unwrap(),
            Some(vec!["M payload/agent-data/p.md".to_string()])
        );
    }
}

//! `agent-files copy [SRC] [DST]`: make the set in DST a byte copy
//! of the one in SRC, and leave it uncommitted.
//!
//! - `CopyArgs`: clap surface, the `SRC` and `DST` operands and the
//!   `-c`/`--custom` / `--no-custom` pair, as `diff` has them.
//! - `plan`: what a copy would do, from `diff::compare`'s rows:
//!   copy what differs or is only in SRC, delete what is only in
//!   DST, and never touch custom.md unless asked, or TODO.md at
//!   all. The set is what SRC's `file-set.toml` says, its verbatim
//!   entries, and its optional ones on request ([`SetPaths`]).
//! - `apply`: do it. The jj working copy around DST must be clean
//!   in the set's paths first, so the copy's changes are the only
//!   ones there and a `jj diff` reads as the re-sync.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Args;
use log::{error, info};

use super::SetPaths;
use super::diff::{self, State};
use crate::common;
use crate::file_set;

/// CLI args for `agent-files copy`.
#[derive(Args, Debug)]
pub struct CopyArgs {
    /// Retired: the two directories, now `--from` and `--to`, refused
    /// naming them.
    #[arg(value_name = "RETIRED", hide = true, num_args = 0..=2)]
    pub retired: Vec<String>,

    /// The directory to copy the set from [default:
    /// agent-files.copy.dir, else family.template, else an error]
    #[arg(long = "from", value_name = "DIR")]
    pub src: Option<PathBuf>,

    /// The directory whose set becomes the copy [default: this
    /// workspace, so one side is --from]
    #[arg(long = "to", value_name = "DIR")]
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
pub fn plan(
    src: &Path,
    dst: &Path,
    paths: &SetPaths,
    custom: bool,
) -> Result<Vec<Step>, Box<dyn std::error::Error>> {
    Ok(diff::compare(src, dst, paths, custom)?
        .into_iter()
        .filter_map(|(path, state)| match state {
            State::Differs | State::OnlyInA => Some(Step::Copy(path)),
            State::OnlyInB => Some(Step::Delete(path)),
            State::Same | State::ProjectLayer => None,
        })
        .collect())
}

/// Run the steps: copy makes the directories a file needs, by
/// `file_set`'s copy, and delete removes the file only.
pub fn apply(src: &Path, dst: &Path, steps: &[Step]) -> Result<(), Box<dyn std::error::Error>> {
    for step in steps {
        match step {
            Step::Copy(path) => {
                file_set::copy_file(&src.join(path), &dst.join(path), true, None)
                    .map_err(|e| format!("copy {path}: {e}"))?;
            }
            Step::Delete(path) => {
                std::fs::remove_file(dst.join(path)).map_err(|e| format!("delete {path}: {e}"))?;
            }
        }
    }
    Ok(())
}

/// The changed paths inside `dst`'s set, the files `paths` covers,
/// in the jj working copy around it, which a copy would mix its own changes with. `None`
/// when `dst` is not inside a jj repo. `dst` may be a subdirectory
/// of the repo, the payload inside the template repo being one, so
/// the repo's paths are matched under `dst`'s prefix.
fn dirty_set_paths(
    dst: &Path,
    paths: &SetPaths,
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
                    .filter(|rel| paths.covers(rel, custom))
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
        diff::refuse_from_to("agent-files copy", &self.retired)?;
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
        let paths = SetPaths::of(&src.path)?;
        info!("agent-files copy: {}", paths.described());
        match dirty_set_paths(&dst.path, &paths, custom)? {
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
        let steps = plan(&src.path, &dst.path, &paths, custom)?;
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
    use crate::agent_files::diff::AGENT_DATA;
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
        let a = T::try_parse_from(["t", "--from", "../x", "--custom"])
            .unwrap()
            .a;
        assert_eq!(a.src, Some(PathBuf::from("../x")));
        assert!(a.dst.is_none() && a.custom);
        let a = T::try_parse_from(["t", "--from", "../x", "--to", "../y"])
            .unwrap()
            .a;
        assert_eq!(a.dst, Some(PathBuf::from("../y")));
        assert!(T::try_parse_from(["t", "-c", "--no-custom"]).is_err());
        let a = T::try_parse_from(["t", "../x", "../y"]).unwrap().a;
        let err = diff::refuse_from_to("agent-files copy", &a.retired).unwrap_err();
        assert!(err.ends_with("`--from ../x --to ../y`"), "{err}");
    }

    /// A copy plan copies what differs or is only in the source,
    /// deletes what is only in the destination, and leaves custom.md
    /// and TODO.md alone until `-c` brings custom.md in. Applied,
    /// the sets compare equal.
    #[test]
    fn plan_and_apply_mirror_the_set() {
        let base = crate::test_helpers::unique_base("af-copy");
        let set = SetPaths::builtin();
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
        let steps = plan(&src, &dst, &set, false).unwrap();
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
        let rows = diff::compare(&src, &dst, &set, false).unwrap();
        assert!(rows.iter().all(|(_, s)| !s.differs()), "{rows:?}");
        assert_eq!(
            std::fs::read_to_string(dst.join("custom.md")).unwrap(),
            "mine\n"
        );
        assert_eq!(
            std::fs::read_to_string(dst.join("TODO.md")).unwrap(),
            "my todo\n"
        );
        assert!(plan(&src, &dst, &set, false).unwrap().is_empty());
        let steps = plan(&src, &dst, &set, true).unwrap();
        assert_eq!(steps, vec![Step::Copy("custom.md".to_string())]);
        apply(&src, &dst, &steps).unwrap();
        assert_eq!(
            std::fs::read_to_string(dst.join("custom.md")).unwrap(),
            "theirs\n"
        );
        // Outside any jj repo there is no guard to consult.
        assert_eq!(dirty_set_paths(&dst, &set, false).unwrap(), None);
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
        let set = SetPaths::builtin();
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
        assert_eq!(
            dirty_set_paths(&fx.work, &set, false).unwrap(),
            Some(vec![])
        );
        std::fs::write(fx.work.join("README.md"), "other\n").unwrap();
        std::fs::write(fx.work.join("custom.md"), "edited\n").unwrap();
        assert_eq!(
            dirty_set_paths(&fx.work, &set, false).unwrap(),
            Some(vec![])
        );
        assert_eq!(
            dirty_set_paths(&fx.work, &set, true).unwrap(),
            Some(vec!["M custom.md".to_string()])
        );
        std::fs::write(fx.work.join("agent-data/a.md"), "b\n").unwrap();
        assert_eq!(
            dirty_set_paths(&fx.work, &set, false).unwrap(),
            Some(vec!["M agent-data/a.md".to_string()])
        );
        // The subdirectory's guard sees only its own set.
        assert_eq!(dirty_set_paths(&sub, &set, false).unwrap(), Some(vec![]));
        std::fs::write(sub.join("agent-data/p.md"), "q\n").unwrap();
        assert_eq!(
            dirty_set_paths(&sub, &set, false).unwrap(),
            Some(vec!["M payload/agent-data/p.md".to_string()])
        );
    }

    /// With a manifest in the source the set is what it says: its
    /// verbatim entries, a directory's files at every depth, its
    /// optional ones with `-c`, each at its own path whatever `from`
    /// it names, and never a starter. A file only the destination
    /// has beneath a verbatim directory is deleted.
    #[test]
    fn the_manifest_says_what_a_copy_takes() {
        let base = crate::test_helpers::unique_base("af-copy-manifest");
        let src = base.join("src");
        let dst = base.join("dst");
        let seed_deep = |dir: &Path, files: &[(&str, &str)]| {
            for (path, content) in files {
                let path = dir.join(path);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, content).unwrap();
            }
        };
        seed_deep(
            &src,
            &[
                (
                    "file-set.toml",
                    r#"
[[file]]
path = "rules.md"
kind = "verbatim"
[[file]]
path = "docs/"
kind = "verbatim"
[[file]]
path = "CONTRIBUTING.md"
kind = "starter"
[[file]]
path = "local.md"
kind = "optional"
from = "skel/local.md"
"#,
                ),
                ("rules.md", "new rules\n"),
                ("docs/a.md", "a\n"),
                ("docs/deep/b.md", "b\n"),
                ("CONTRIBUTING.md", "theirs\n"),
                ("local.md", "their layer\n"),
                ("skel/local.md", "a template\n"),
                ("AGENTS.md", "not in this set\n"),
            ],
        );
        seed_deep(
            &dst,
            &[
                ("rules.md", "old rules\n"),
                ("docs/deep/stale.md", "gone\n"),
                ("CONTRIBUTING.md", "ours\n"),
                ("local.md", "my layer\n"),
            ],
        );
        let set = SetPaths::of(&src).unwrap();
        assert_eq!(set.manifest, Some(src.join("file-set.toml")));
        assert_eq!(set.taken(false), ["rules.md", "docs"]);
        assert_eq!(set.taken(true), ["rules.md", "docs", "local.md"]);
        assert!(set.covers("docs/deep/b.md", false));
        assert!(!set.covers("local.md", false) && set.covers("local.md", true));
        assert!(!set.covers("CONTRIBUTING.md", true));
        assert!(set.described().ends_with("file-set.toml"));

        let steps = plan(&src, &dst, &set, false).unwrap();
        assert_eq!(
            steps,
            vec![
                Step::Copy("docs/a.md".to_string()),
                Step::Copy("docs/deep/b.md".to_string()),
                Step::Delete("docs/deep/stale.md".to_string()),
                Step::Copy("rules.md".to_string()),
            ]
        );
        apply(&src, &dst, &steps).unwrap();
        let read = |file: &str| std::fs::read_to_string(dst.join(file)).unwrap();
        assert_eq!(read("docs/deep/b.md"), "b\n");
        assert_eq!(
            read("CONTRIBUTING.md"),
            "ours\n",
            "a starter is the project's"
        );
        assert_eq!(read("local.md"), "my layer\n", "not asked for");
        assert!(!dst.join("AGENTS.md").exists(), "not in this set");
        assert!(plan(&src, &dst, &set, false).unwrap().is_empty());

        let steps = plan(&src, &dst, &set, true).unwrap();
        assert_eq!(steps, vec![Step::Copy("local.md".to_string())]);
        apply(&src, &dst, &steps).unwrap();
        assert_eq!(
            read("local.md"),
            "their layer\n",
            "its own path, not `from`"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    /// A source with no manifest is the built-in set, and one whose
    /// manifest does not parse is an error naming it, never the
    /// built-in set in its place.
    #[test]
    fn no_manifest_is_the_built_in_set_and_a_bad_one_an_error() {
        let base = crate::test_helpers::unique_base("af-copy-no-manifest");
        std::fs::create_dir_all(&base).unwrap();
        let set = SetPaths::of(&base).unwrap();
        assert_eq!(set, SetPaths::builtin());
        assert!(set.described().starts_with("the built-in set"));
        std::fs::write(base.join("file-set.toml"), "[[file]]\nkind = \"nope\"\n").unwrap();
        let err = SetPaths::of(&base).unwrap_err();
        assert!(
            err.contains("file-set.toml") && err.contains("unknown kind"),
            "{err}"
        );
        let _ = std::fs::remove_dir_all(&base);
    }
}

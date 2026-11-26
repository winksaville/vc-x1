//! `init --from`: the file set a new workspace starts from.
//!
//! A set is read by its manifest ([`crate::file_set`]), which lists
//! the files and gives each a kind. The module holds what `init` does
//! with a kind, the policy, over `file_set`'s mechanism:
//!
//! - verbatim: written, and a different file there replaced.
//! - starter: written only when no file is there, `{name}` filled
//!   with the project's name.
//! - optional: left out, and with `--custom` the source's own file
//!   is written as it is, only when no file is there.
//!
//! The manifest's `gitignore` lines are added to the work repo's
//! `.gitignore` where it lacks them, so what the set means to keep
//! local, `custom.md` for one, is out of the first commit.
//!
//! [`SetPlan::new`] is the preflight, refusing a set that cannot be
//! copied before anything is made, and [`SetPlan::copy`] the step.

use std::path::{Path, PathBuf};

use clap::{Args, ValueHint};
use log::{debug, info};

use crate::file_set::{self, Copied, Entry, FileSet, Fill, Kind};

/// The placeholder a starter's text carries for the project's name.
const NAME: &str = "{name}";

/// `--from`, `--file-set`, and `--custom`: the set `init` copies.
#[derive(Args, Clone, Debug, Default)]
pub struct SetOptions {
    /// Copy a file set from SOURCE, a directory holding one, into the
    /// work repo before its first commit.
    ///
    /// The set is read by its manifest, `SOURCE/file-set.toml`, each
    /// entry a path and a kind:
    ///
    /// - verbatim: written, replacing a different file there.
    /// - starter: written only when no file is there, `{name}` filled
    ///   with the project's name.
    /// - optional: left out unless `--custom` asks for it, and then
    ///   SOURCE's own file, as it is, written only when no file is
    ///   there.
    ///
    /// SOURCE is read as its working copy holds it.
    #[arg(
        long,
        value_name = "SOURCE",
        value_hint = ValueHint::DirPath,
        verbatim_doc_comment
    )]
    pub from: Option<PathBuf>,

    /// The manifest `--from` copies by, in place of SOURCE's own
    /// [default: SOURCE/file-set.toml].
    #[arg(
        long,
        value_name = "PATH",
        value_hint = ValueHint::FilePath,
        requires = "from"
    )]
    pub file_set: Option<PathBuf>,

    /// Copy the set's optional entries too, SOURCE's own custom.md
    /// for one, each as it is and only when no file is there.
    #[arg(short = 'c', long, requires = "from")]
    pub custom: bool,
}

/// A set `init` will copy: its root, its manifest, and what was asked.
#[derive(Debug)]
pub(crate) struct SetPlan {
    /// The set's root, absolute.
    root: PathBuf,
    /// The manifest read, as given or SOURCE's own.
    manifest: PathBuf,
    /// The manifest's entries.
    set: FileSet,
    /// Whether the optional entries are copied.
    custom: bool,
}

impl SetPlan {
    /// The preflight: `None` with no `--from`, else the set, read and
    /// checked, with nothing written.
    ///
    /// - SOURCE is a directory, and not the target itself.
    /// - The manifest parses.
    /// - Each entry the copy takes has a file in SOURCE, since a set
    ///   that lists what it lacks would make a workspace missing it.
    pub(crate) fn new(opts: &SetOptions, target: &Path) -> Result<Option<Self>, String> {
        let Some(from) = &opts.from else {
            return Ok(None);
        };
        let root = std::fs::canonicalize(from)
            .map_err(|e| format!("--from: the set's directory {}: {e}", from.display()))?;
        if !root.is_dir() {
            return Err(format!("--from: {} is not a directory", from.display()));
        }
        if std::fs::canonicalize(target).is_ok_and(|t| t == root) {
            return Err(format!(
                "--from: {} is the target itself, and a set is copied from another directory",
                from.display()
            ));
        }
        let manifest = match &opts.file_set {
            Some(path) => path.clone(),
            None => root.join(file_set::MANIFEST),
        };
        let set = file_set::load(&manifest).map_err(|e| format!("--from: {e}"))?;
        let plan = Self {
            root,
            manifest,
            set,
            custom: opts.custom,
        };
        for entry in plan.taken() {
            if file_set::files_of(&plan.root, entry)?.is_empty() {
                return Err(format!(
                    "--from: {} lists '{}', read from '{}', and {} holds no such file",
                    plan.manifest.display(),
                    entry.path,
                    entry.source(),
                    plan.root.display()
                ));
            }
        }
        Ok(Some(plan))
    }

    /// The entries the copy takes: every one but the optional, and
    /// those too with `--custom`.
    fn taken(&self) -> impl Iterator<Item = &Entry> {
        self.set
            .entries
            .iter()
            .filter(|e| self.custom || e.kind != Kind::Optional)
    }

    /// The step's title, for the dry run and the run alike.
    pub(crate) fn title(&self) -> String {
        let optional = if self.custom {
            ", its optional files included"
        } else {
            ""
        };
        format!(
            "Copy the file set of {} by {}{optional}: verbatim files replaced, starters \
             written where absent, and its .gitignore lines added",
            self.root.display(),
            self.manifest.display()
        )
    }

    /// The files the copy takes, one a line, for the dry run: the
    /// kind, where the file is written, and where it is read when
    /// that is another path.
    pub(crate) fn files(&self) -> Result<Vec<String>, String> {
        let mut lines = Vec::new();
        for entry in self.taken() {
            for file in file_set::files_of(&self.root, entry)? {
                let read = if file.from == file.to {
                    String::new()
                } else {
                    format!("  (from {})", file.from)
                };
                lines.push(format!("{:<8}  {}{read}", entry.kind.name(), file.to));
            }
        }
        Ok(lines)
    }

    /// The lines the set gives the work repo's `.gitignore`.
    pub(crate) fn gitignore(&self) -> &[String] {
        &self.set.gitignore
    }

    /// Copy the set into `dest`, the work repo's directory, whose
    /// name is the project's.
    ///
    /// Run after `init` wrote its own files, so a starter of a name
    /// `init` writes, `TODO.md` or `zTOC.md`, finds it there and is
    /// kept out, and the `.gitignore` its lines are added to is
    /// there.
    pub(crate) fn copy(&self, dest: &Path) -> Result<(), String> {
        let name = dest
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("{} has no name to fill {NAME} with", dest.display()))?;
        let fill = |text: &str| text.replace(NAME, name);
        let (mut written, mut replaced, mut kept) = (0, 0, 0);
        for entry in self.taken() {
            let (replace, fill): (bool, Option<Fill>) = match entry.kind {
                Kind::Verbatim => (true, None),
                Kind::Starter => (false, Some(&fill)),
                Kind::Optional => (false, None),
            };
            for file in file_set::files_of(&self.root, entry)? {
                let outcome = file_set::copy_file(
                    &self.root.join(&file.from),
                    &dest.join(&file.to),
                    replace,
                    fill,
                )?;
                debug!("init: set file {}: {outcome:?}", file.to);
                match outcome {
                    Copied::Written => written += 1,
                    Copied::Replaced => replaced += 1,
                    Copied::Kept | Copied::Same => kept += 1,
                }
            }
        }
        let ignored = file_set::add_lines(&dest.join(".gitignore"), &self.set.gitignore)?;
        info!(
            "  {written} written, {replaced} replaced, {kept} kept as found, {ignored} .gitignore \
             line(s) added"
        );
        Ok(())
    }
}

//! `vc-x1 xtask copy | diff`: put a project's `xtask/` in a Rust
//! workspace, or bring it up to date, as `agent-files copy` and `diff`
//! do the set, so a new workspace, an adoption, and an update are one
//! act from three starting points.
//!
//! - The source is a directory holding `xtask/`, `--from`, else the
//!   config's `xtask.dir`, read on disk. A specific version is a source
//!   tree at that commit, a jj workspace or a `git archive` of it.
//! - The target is the workspace, or `--to`. With no `xtask/` the copy
//!   brings it, and with one it replaces it, deletions included.
//! - xtask's version, `xtask/Cargo.toml`'s own, decides: a newer source
//!   replaces, an older one is refused, and files that differ at the
//!   same version are a local change, refused, `--force` taking either.
//! - The wiring, the workspace member and the `cargo xtask` alias, is
//!   edited in place ([`wiring`]), never left to the user, and a
//!   directory with no `Cargo.toml`, a repo fresh from `vc-x1 init`,
//!   becomes a workspace of xtask alone, `/target` ignored.
//!
//! Nothing is committed: the copy is left in the working copy for
//! review, and a working copy that already changes `xtask/` is refused.

pub mod wiring;

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Subcommand};
use log::{error, info, warn};

use crate::common;
use wiring::{Alias, XTASK};

/// CLI args for the `xtask` group.
#[derive(Args, Debug)]
pub struct XtaskArgs {
    #[command(subcommand)]
    pub command: XtaskCommand,
}

/// The `xtask` subcommands.
#[derive(Subcommand, Debug)]
pub enum XtaskCommand {
    /// Name what a copy of xtask/ would change, exiting non-zero when anything
    #[command(
        long_about = "Compare the source's xtask/ with the target's, one line per\n\
        file that differs, and name xtask's version on each side and the\n\
        wiring a copy would add: the workspace member in Cargo.toml and the\n\
        `cargo xtask` alias in .cargo/config.toml. The source is --from,\n\
        else the config's xtask.dir, and the target --to, else this\n\
        workspace. Exits non-zero when a copy would change anything, as\n\
        diff does."
    )]
    Diff(Ends),

    /// Copy xtask/ into a Rust workspace, or bring it up to date, uncommitted
    #[command(
        long_about = "Make the target's xtask/ a copy of the source's, deletions\n\
        included, its target/ left alone, and wire it in: `xtask` added to\n\
        the root Cargo.toml's [workspace] members, a single package\n\
        becoming a workspace of it and xtask, and the `cargo xtask` alias\n\
        added to .cargo/config.toml. Each file keeps its comments and\n\
        layout. A directory with no Cargo.toml, a repo fresh from\n\
        `vc-x1 init`, gets one, a workspace of xtask alone and a version\n\
        for bump, and .gitignore gains /target when nothing ignores it. The source is --from, a directory holding xtask/, else the\n\
        config's xtask.dir, and the target --to, else this workspace.\n\n\
        xtask's own version, xtask/Cargo.toml's, decides. A target with no\n\
        xtask/ takes the copy. A newer source replaces it. An older one is\n\
        refused, and so are files that differ at the same version, a local\n\
        change, --force taking either. A project's own task belongs in\n\
        Cargo.toml's [workspace.metadata.xtask.tasks], so xtask/ stays a\n\
        copy.\n\n\
        A specific version is a source tree at its commit, given as --from:\n\
        `jj -R SRC workspace add -r REV DIR`, or\n\
        `git -C SRC archive REV xtask | tar -x -C DIR`.\n\n\
        Prints both ends, xtask's version on each, and each step. Refuses a\n\
        target whose jj working copy already changes xtask/, so the copy's\n\
        changes are the only ones there, and leaves them uncommitted for\n\
        review. --dry-run prints the plan and writes nothing."
    )]
    Copy(CopyArgs),
}

/// The two ends, the source and the target.
#[derive(Args, Debug)]
pub struct Ends {
    /// The directory holding the xtask/ to copy [default: xtask.dir]
    #[arg(long = "from", value_name = "DIR")]
    pub from: Option<PathBuf>,

    /// The workspace whose xtask/ becomes the copy [default: this
    /// workspace]
    #[arg(long = "to", value_name = "DIR")]
    pub to: Option<PathBuf>,
}

/// CLI args for `xtask copy`.
#[derive(Args, Debug)]
pub struct CopyArgs {
    #[command(flatten)]
    pub ends: Ends,

    /// Print the plan and write nothing
    #[arg(long)]
    pub dry_run: bool,

    /// Copy an older source, or over a local change
    #[arg(long)]
    pub force: bool,
}

impl XtaskArgs {
    /// Run the chosen subcommand.
    pub fn run(&self) -> ExitCode {
        let (what, result) = match &self.command {
            XtaskCommand::Diff(ends) => ("xtask diff", diff(ends)),
            XtaskCommand::Copy(args) => ("xtask copy", copy(args).map(|()| true)),
        };
        match result {
            Ok(true) => ExitCode::SUCCESS,
            Ok(false) => ExitCode::FAILURE,
            Err(e) => {
                error!("{what}: {e}");
                ExitCode::FAILURE
            }
        }
    }
}

/// A resolved end: the path, the text it was given as, and where it
/// came from.
struct End {
    path: PathBuf,
    shown: String,
    from: &'static str,
}

/// The source: `--from`, else `xtask.dir` relative to the config's
/// directory, refused when it holds no `xtask/Cargo.toml`.
fn source(root: Option<&Path>, from: Option<&Path>) -> Result<End, String> {
    let end = match (from, root) {
        (Some(p), _) => End {
            path: p.to_path_buf(),
            shown: p.display().to_string(),
            from: "--from",
        },
        (None, Some(root)) => {
            let dir = crate::config_md::load(root)
                .map_err(|e| e.to_string())?
                .and_then(|c| crate::toml_simple::toml_get(&c.map, "xtask.dir").cloned())
                .ok_or("no source: give --from DIR, or set xtask.dir in the config")?;
            End {
                path: common::resolve_repo_path(root, &dir),
                shown: dir,
                from: "xtask.dir",
            }
        }
        (None, None) => return Err("not in a vc-x1 workspace, and no --from given".to_string()),
    };
    if !end.path.join(XTASK).join("Cargo.toml").is_file() {
        return Err(format!(
            "'{}' ({}) holds no xtask/Cargo.toml",
            end.shown, end.from
        ));
    }
    Ok(end)
}

/// The target: `--to`, else this workspace, refused when it is not a
/// directory.
fn target(root: Option<&Path>, to: Option<&Path>) -> Result<End, String> {
    let end = match (to, root) {
        (Some(p), _) => End {
            path: p.to_path_buf(),
            shown: p.display().to_string(),
            from: "--to",
        },
        (None, Some(root)) => End {
            path: root.to_path_buf(),
            shown: root.display().to_string(),
            from: "this workspace",
        },
        (None, None) => return Err("not in a vc-x1 workspace, and no --to given".to_string()),
    };
    if !end.path.is_dir() {
        return Err(format!("'{}' ({}) is not a directory", end.shown, end.from));
    }
    Ok(end)
}

/// How a file of `xtask/` stands between the two ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Same,
    Differs,
    OnlyInSource,
    OnlyInTarget,
}

/// The files of `root`'s `xtask/`, relative to `root`, its `target/`
/// left out, a missing `xtask/` none.
fn files(root: &Path) -> Result<BTreeSet<String>, String> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeSet<String>) -> Result<(), String> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Ok(());
        };
        for entry in entries {
            let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
            let path = entry.path();
            let kind = entry
                .file_type()
                .map_err(|e| format!("{}: {e}", path.display()))?;
            if kind.is_dir() {
                if entry.file_name() != "target" {
                    walk(root, &path, out)?;
                }
            } else if kind.is_file()
                && let Ok(rel) = path.strip_prefix(root)
            {
                out.insert(rel.to_string_lossy().into_owned());
            } else if kind.is_symlink() {
                return Err(format!("{} is a symlink, refused", path.display()));
            }
        }
        Ok(())
    }
    let mut out = BTreeSet::new();
    walk(root, &root.join(XTASK), &mut out)?;
    Ok(out)
}

/// Each file of either `xtask/`, in path order, and how it stands.
fn compare(src: &Path, dst: &Path) -> Result<Vec<(String, State)>, String> {
    let a = files(src)?;
    let b = files(dst)?;
    let read = |p: &Path| std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()));
    a.union(&b)
        .map(|rel| {
            let state = match (a.contains(rel), b.contains(rel)) {
                (true, false) => State::OnlyInSource,
                (false, _) => State::OnlyInTarget,
                (true, true) => match read(&src.join(rel))? == read(&dst.join(rel))? {
                    true => State::Same,
                    false => State::Differs,
                },
            };
            Ok((rel.clone(), state))
        })
        .collect()
}

/// xtask's version at `root`, `xtask/Cargo.toml`'s `[package] version`,
/// `None` when there is no `xtask/`.
fn version_at(root: &Path) -> Result<Option<String>, String> {
    let path = root.join(XTASK).join("Cargo.toml");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(None);
    };
    let doc: toml_edit::DocumentMut = text
        .parse()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    doc.get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .map(|v| Some(v.to_string()))
        .ok_or_else(|| {
            format!(
                "{} has no [package] version of its own, which xtask's copies are compared by",
                path.display()
            )
        })
}

/// The source's xtask version, which [`source`] made sure there is a
/// manifest for.
fn source_version(root: &Path) -> Result<String, String> {
    version_at(root)?.ok_or_else(|| format!("{} holds no xtask/Cargo.toml", root.display()))
}

/// Two versions by semver's order: `X.Y.Z` numerically, then a
/// version with a pre-release, `-N`, before the one without, the
/// pre-releases compared part by part, numbers as numbers.
fn cmp_versions(a: &str, b: &str) -> Ordering {
    fn split(v: &str) -> (Vec<u64>, Option<&str>) {
        let (core, pre) = match v.split_once('-') {
            Some((c, p)) => (c, Some(p)),
            None => (v, None),
        };
        // OK: a part that is no number orders as 0, so a malformed
        // version still orders, and the copy names both it compared
        let core = core.split('.').map(|p| p.parse().unwrap_or(0)).collect();
        (core, pre)
    }
    fn cmp_pre(a: &str, b: &str) -> Ordering {
        let mut a = a.split('.');
        let mut b = b.split('.');
        loop {
            return match (a.next(), b.next()) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some(x), Some(y)) => {
                    let o = match (x.parse::<u64>(), y.parse::<u64>()) {
                        (Ok(x), Ok(y)) => x.cmp(&y),
                        _ => x.cmp(y),
                    };
                    match o {
                        Ordering::Equal => continue,
                        o => o,
                    }
                }
            };
        }
    }
    let (ac, ap) = split(a);
    let (bc, bp) = split(b);
    ac.cmp(&bc).then_with(|| match (ap, bp) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(x), Some(y)) => cmp_pre(x, y),
    })
}

/// What the versions and the files say a copy does.
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    /// The target has no `xtask/`.
    New,
    /// The files are the same.
    UpToDate,
    /// The source is newer.
    Newer,
    /// The source is older than the target's, named, refused without
    /// `--force`.
    Older(String),
    /// The files differ at the same version, refused without `--force`.
    LocalChange,
}

fn verdict(src: &str, dst: Option<&str>, rows: &[(String, State)]) -> Verdict {
    let Some(dst) = dst else {
        return Verdict::New;
    };
    if rows.iter().all(|(_, s)| *s == State::Same) {
        return Verdict::UpToDate;
    }
    match cmp_versions(src, dst) {
        Ordering::Greater => Verdict::Newer,
        Ordering::Less => Verdict::Older(dst.to_string()),
        Ordering::Equal => Verdict::LocalChange,
    }
}

/// The version line, `xtask <dst> -> <src>`, or the source's alone
/// for a target that has none.
fn version_line(src: &str, dst: Option<&str>) -> String {
    match dst {
        Some(dst) if dst == src => format!("xtask {src} in both"),
        Some(dst) => format!("xtask {dst} -> {src}"),
        None => format!("xtask {src}, new to the target"),
    }
}

/// The edits the wiring needs at `dst`: the new `Cargo.toml` text,
/// the alias, the path its config is at, and the notes to print.
struct Wiring {
    manifest: Option<String>,
    /// Whether the manifest is new, the directory having none.
    new_manifest: bool,
    /// The `.gitignore` with `/target` added, `None` when it ignores it.
    gitignore: Option<String>,
    alias: Alias,
    config_path: PathBuf,
    /// The config's path as the plan names it, `.cargo/config.toml`.
    config_name: &'static str,
    notes: Vec<String>,
}

fn wiring_at(dst: &Path) -> Result<Wiring, String> {
    let manifest_path = dst.join("Cargo.toml");
    let (text, manifest, new_manifest) = match manifest_path.is_file() {
        true => {
            let text = std::fs::read_to_string(&manifest_path)
                .map_err(|e| format!("{}: {e}", manifest_path.display()))?;
            let manifest = wiring::manifest(&text, wiring::parent_workspace(dst).as_deref())?;
            (text, manifest, false)
        }
        false => {
            if let Some(parent) = wiring::parent_workspace(dst) {
                return Err(format!(
                    "{} has no Cargo.toml and is inside the workspace at {}, so xtask belongs \
                     to that one: copy it there",
                    dst.display(),
                    parent.display()
                ));
            }
            let new = wiring::NEW_MANIFEST.to_string();
            (new.clone(), Some(new), true)
        }
    };
    let gitignore_path = dst.join(".gitignore");
    let gitignore = match gitignore_path.is_file() {
        true => wiring::gitignore(Some(
            &std::fs::read_to_string(&gitignore_path)
                .map_err(|e| format!("{}: {e}", gitignore_path.display()))?,
        )),
        false => wiring::gitignore(None),
    };
    // A legacy `.cargo/config`, no extension, is the one cargo reads
    // when it is the only one, so the alias goes there.
    let cargo_dir = dst.join(".cargo");
    let config_name = match (
        cargo_dir.join("config.toml").is_file(),
        cargo_dir.join("config").is_file(),
    ) {
        (false, true) => ".cargo/config",
        _ => ".cargo/config.toml",
    };
    let config_path = dst.join(config_name);
    let config_text = match config_path.is_file() {
        true => Some(
            std::fs::read_to_string(&config_path)
                .map_err(|e| format!("{}: {e}", config_path.display()))?,
        ),
        false => None,
    };
    let alias = wiring::alias(config_text.as_deref())?;
    let mut notes = Vec::new();
    if let Alias::Own(own) = &alias {
        notes.push(format!(
            "{} keeps its own `xtask` alias, `{own}`, where xtask's is `{}`",
            config_path.display(),
            wiring::ALIAS
        ));
    }
    let edited = match &manifest {
        Some(new) => new.as_str(),
        None => text.as_str(),
    };
    if new_manifest {
        notes.push(
            "the workspace has xtask alone: add your packages to Cargo.toml's [workspace] members"
                .to_string(),
        );
    } else if wiring::wants_default_bin(edited) {
        notes.push(
            "a virtual workspace of several binaries names the one `cargo xtask run` takes in \
             Cargo.toml's [workspace.metadata.xtask] default-bin"
                .to_string(),
        );
    }
    Ok(Wiring {
        manifest,
        new_manifest,
        gitignore,
        alias,
        config_path,
        config_name,
        notes,
    })
}

/// The lines that say what the wiring adds, none when it is in place.
fn wiring_lines(w: &Wiring) -> Vec<String> {
    let mut out = Vec::new();
    match (&w.manifest, w.new_manifest) {
        (Some(_), true) => out.push("write  Cargo.toml, a workspace of xtask".to_string()),
        (Some(_), false) => out.push("edit   Cargo.toml, `xtask` a [workspace] member".to_string()),
        (None, _) => {}
    }
    if w.gitignore.is_some() {
        out.push("ignore .gitignore + /target".to_string());
    }
    if let Alias::Add(_) = w.alias {
        out.push(format!(
            "{}  {}, the `cargo xtask` alias",
            match w.config_path.is_file() {
                true => "edit ",
                false => "write",
            },
            w.config_name
        ));
    }
    out
}

/// The changed paths under `dst`'s `xtask/` in the jj working copy
/// around it, `None` when it is in no jj repo.
fn dirty_xtask(dst: &Path) -> Result<Option<Vec<String>>, String> {
    let dst = dst.canonicalize().map_err(|e| e.to_string())?;
    let Some(repo) = crate::status::nearest_jj(&dst) else {
        return Ok(None);
    };
    let repo = repo.canonicalize().map_err(|e| e.to_string())?;
    let prefix = match dst.strip_prefix(&repo) {
        Ok(p) if !p.as_os_str().is_empty() => format!("{}/{XTASK}/", p.display()),
        _ => format!("{XTASK}/"),
    };
    Ok(Some(
        crate::jj::wc_status(&repo)
            .map_err(|e| e.to_string())?
            .changes
            .into_iter()
            .filter(|(_, p)| p.starts_with(&prefix))
            .map(|(letter, p)| format!("{letter} {p}"))
            .collect(),
    ))
}

/// The two ends, refused when they are one directory.
fn ends(e: &Ends) -> Result<(End, End), String> {
    let root = common::find_workspace_root();
    let src = source(root.as_deref(), e.from.as_deref())?;
    let dst = target(root.as_deref(), e.to.as_deref())?;
    let same = |a: &Path, b: &Path| match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    };
    if same(&src.path, &dst.path) {
        return Err(format!(
            "'{}' and '{}' are the same directory",
            src.shown, dst.shown
        ));
    }
    Ok((src, dst))
}

/// Print the comparison, `Ok(true)` when a copy would change nothing.
fn diff(e: &Ends) -> Result<bool, String> {
    let (src, dst) = ends(e)?;
    info!(
        "xtask diff: {} ({}) against {} ({})",
        src.shown, src.from, dst.shown, dst.from
    );
    let src_v = source_version(&src.path)?;
    let dst_v = version_at(&dst.path)?;
    info!("xtask diff: {}", version_line(&src_v, dst_v.as_deref()));
    let rows = compare(&src.path, &dst.path)?;
    let mut same = 0;
    for (path, state) in &rows {
        match state {
            State::Same => same += 1,
            State::Differs => info!("  differs        {path}"),
            State::OnlyInSource => info!("  only in source {path}"),
            State::OnlyInTarget => info!("  only in target {path}"),
        }
    }
    let w = wiring_at(&dst.path)?;
    let lines = wiring_lines(&w);
    for line in &lines {
        info!("  wiring: {line}");
    }
    for note in &w.notes {
        info!("xtask diff: {note}");
    }
    let changed = rows.len() - same;
    info!(
        "xtask diff: {same} file(s) the same, {changed} differ, {} wiring edit(s)",
        lines.len()
    );
    Ok(changed == 0 && lines.is_empty())
}

/// Resolve, compare, judge, print the plan, apply it.
fn copy(args: &CopyArgs) -> Result<(), String> {
    let (src, dst) = ends(&args.ends)?;
    info!(
        "xtask copy: from {} ({}) into {} ({})",
        src.shown, src.from, dst.shown, dst.from
    );
    let src_v = source_version(&src.path)?;
    let dst_v = version_at(&dst.path)?;
    let rows = compare(&src.path, &dst.path)?;
    let w = wiring_at(&dst.path)?;
    info!("xtask copy: {}", version_line(&src_v, dst_v.as_deref()));
    let v = verdict(&src_v, dst_v.as_deref(), &rows);
    match (&v, args.force) {
        (Verdict::Older(target), false) => {
            return Err(format!(
                "the source's xtask {src_v} is older than the target's {target}: --force copies it"
            ));
        }
        (Verdict::LocalChange, false) => {
            let differ: Vec<&str> = rows
                .iter()
                .filter(|(_, s)| *s != State::Same)
                .map(|(p, _)| p.as_str())
                .collect();
            return Err(format!(
                "xtask {src_v} on both sides, yet {} differ: a local change, or a source changed \
                 without raising xtask's version. A project's own task belongs in Cargo.toml's \
                 [workspace.metadata.xtask.tasks]. --force copies over it",
                differ.join(", ")
            ));
        }
        (Verdict::Older(_) | Verdict::LocalChange, true) => {
            warn!("xtask copy: --force, so the target's xtask/ is replaced as it is")
        }
        _ => {}
    }
    let steps: Vec<(&String, State)> = rows
        .iter()
        .filter(|(_, s)| *s != State::Same)
        .map(|(p, s)| (p, *s))
        .collect();
    for (path, state) in &steps {
        match state {
            State::OnlyInTarget => info!("  delete {path}"),
            _ => info!("  copy   {path}"),
        }
    }
    let lines = wiring_lines(&w);
    for line in &lines {
        info!("  {line}");
    }
    for note in &w.notes {
        info!("xtask copy: {note}");
    }
    if steps.is_empty() && lines.is_empty() {
        info!("xtask copy: up to date, nothing to do");
        return Ok(());
    }
    if args.dry_run {
        info!("xtask copy: dry run, nothing written");
        return Ok(());
    }
    match dirty_xtask(&dst.path)? {
        None => info!("{} is not in a jj repo: no working-copy guard", dst.shown),
        Some(dirty) if !dirty.is_empty() => {
            return Err(format!(
                "the working copy already changes xtask/, commit or restore it first: {}",
                dirty.join(", ")
            ));
        }
        Some(_) => {}
    }
    for (path, state) in &steps {
        let to = dst.path.join(path);
        match state {
            State::OnlyInTarget => {
                std::fs::remove_file(&to).map_err(|e| format!("delete {path}: {e}"))?
            }
            _ => {
                if let Some(parent) = to.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| format!("{path}: {e}"))?;
                }
                std::fs::copy(src.path.join(path), &to).map_err(|e| format!("copy {path}: {e}"))?;
            }
        }
    }
    if let Some(text) = &w.manifest {
        std::fs::write(dst.path.join("Cargo.toml"), text)
            .map_err(|e| format!("Cargo.toml: {e}"))?;
    }
    if let Some(text) = &w.gitignore {
        std::fs::write(dst.path.join(".gitignore"), text)
            .map_err(|e| format!(".gitignore: {e}"))?;
    }
    if let Alias::Add(text) = &w.alias {
        if let Some(parent) = w.config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        std::fs::write(&w.config_path, text)
            .map_err(|e| format!("{}: {e}", w.config_path.display()))?;
    }
    info!(
        "xtask copy: done, nothing committed. `cargo xtask -h` lists the tasks, and the next \
         cargo command adds xtask's dependencies to Cargo.lock"
    );
    if v == Verdict::New {
        info!(
            "xtask copy: a shell alias shortens it, `alias cx='cargo xtask'`, and `source <(cargo \
             xtask completions cx)` completes it"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Versions order as semver: numbers numerically, a pre-release
    /// before its release, pre-releases part by part.
    #[test]
    fn versions_order_as_semver() {
        assert_eq!(cmp_versions("0.1.0", "0.0.0"), Ordering::Greater);
        assert_eq!(cmp_versions("0.10.0", "0.9.0"), Ordering::Greater);
        assert_eq!(cmp_versions("0.1.0", "0.1.0"), Ordering::Equal);
        assert_eq!(cmp_versions("0.1.0-2", "0.1.0"), Ordering::Less);
        assert_eq!(cmp_versions("0.1.0-10", "0.1.0-9"), Ordering::Greater);
        assert_eq!(cmp_versions("1.0.0", "0.99.99"), Ordering::Greater);
    }

    /// The verdict: new without a target, up to date when the files
    /// are the same whatever the versions, else by the versions.
    #[test]
    fn the_verdict_follows_the_versions() {
        let same = vec![("xtask/a".to_string(), State::Same)];
        let differs = vec![("xtask/a".to_string(), State::Differs)];
        assert_eq!(verdict("0.1.0", None, &differs), Verdict::New);
        assert_eq!(verdict("0.2.0", Some("0.1.0"), &same), Verdict::UpToDate);
        assert_eq!(verdict("0.2.0", Some("0.1.0"), &differs), Verdict::Newer);
        assert_eq!(
            verdict("0.1.0", Some("0.2.0"), &differs),
            Verdict::Older("0.2.0".to_string())
        );
        assert_eq!(
            verdict("0.1.0", Some("0.1.0"), &differs),
            Verdict::LocalChange
        );
    }

    /// The files are xtask/'s, relative to the root, its target/ left
    /// out, and the comparison names each file's state.
    #[test]
    fn the_files_compare_by_content() {
        let base = crate::test_tmp_root::resolve_tmp_root()
            .join(format!("vc-x1-xtask-compare-{}", std::process::id()));
        let (a, b) = (base.join("a"), base.join("b"));
        for (root, files) in [
            (
                &a,
                &[
                    ("xtask/Cargo.toml", "x"),
                    ("xtask/src/main.rs", "1"),
                    ("xtask/new.rs", "n"),
                ][..],
            ),
            (
                &b,
                &[
                    ("xtask/Cargo.toml", "x"),
                    ("xtask/src/main.rs", "2"),
                    ("xtask/old.rs", "o"),
                    ("xtask/target/t", "t"),
                ][..],
            ),
        ] {
            for (path, text) in files {
                let p = root.join(path);
                std::fs::create_dir_all(p.parent().expect("a parent")).expect("mkdir");
                std::fs::write(p, text).expect("write");
            }
        }
        assert_eq!(
            compare(&a, &b).unwrap(),
            [
                ("xtask/Cargo.toml".to_string(), State::Same),
                ("xtask/new.rs".to_string(), State::OnlyInSource),
                ("xtask/old.rs".to_string(), State::OnlyInTarget),
                ("xtask/src/main.rs".to_string(), State::Differs),
            ]
        );
        let _ = std::fs::remove_dir_all(&base);
    }
}

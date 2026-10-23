//! The `install` and `install-dev` subcommands: build the release or
//! the dev build, then copy it into the install directory, the two
//! commands that touch the PATH, after reporting what they copy.
//!
//! - `install` runs `[build] release` first, exactly as `vc-x1 build`
//!   does, and `install-dev` runs `[build] dev` as `vc-x1 build-dev`
//!   does. Cargo rebuilds only what changed, so with nothing changed
//!   the binary already tested is the one copied, and a missing or
//!   stale one is built rather than refused.
//! - The binary is `[build] release-bin` or `[build] dev-bin`. The
//!   destination is the one directory `[install] release-dir` or
//!   `dev-dir` names, `~` expanded, so the committed value fits every
//!   machine. The release keeps its file name there, and the dev build
//!   gets `-dev` appended, `vc-x1-dev`, so the two may share a
//!   directory and sit side by side on the PATH. `$PATH` is not
//!   searched.
//! - The report says what is being copied: the version and commit the
//!   binary's `-V` shows, whether that commit is `main` and
//!   `main@origin`, its state, and a suffixed version. A binary whose
//!   `-V` carries no commit is reported as unknown. It refuses
//!   nothing: which builds
//!   may be installed is the release process's call, a cycle's Land
//!   here, and a project releasing from a branch or a tag makes it
//!   differently.
//! - It refuses only when the copy itself would be wrong, every reason
//!   listed: no binary where the config says after the build, a binary
//!   whose `-V` will not run, or a missing install directory.
//! - The copy is a temporary file beside the destination renamed into
//!   place, so the binary installed is the one tested, a running copy
//!   of the old one is undisturbed, and a failed copy leaves the old
//!   one.
//! - `[install] release` and `dev`, optional command lists in
//!   `[build]`'s shape, replace the copy when present: the build, the
//!   report, and the refusals are the same, and the list runs instead
//!   of the copy. Its placeholders are the keys the command reads,
//!   resolved for the run: `{build.release-bin}` and
//!   `{install.release-dir}`, or `{build.dev-bin}` and
//!   `{install.dev-dir}`, `--dir` applied, so the list names every
//!   path it writes. With a list, the file name, `-dev` included, is
//!   the list's to write.
//! - `--dry-run` builds and reports, and copies nothing.
//! - `--dir <DIR>` copies into `DIR` instead of the configured
//!   directory, for one run, so a test of install leaves the PATH
//!   alone. A relative `DIR` is taken against the current directory,
//!   as any path argument is, and the workspace needs no directory key
//!   for it.
//! - A workspace with no directory key for the build has nothing to
//!   install, and the command says so before building anything.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info};

use crate::build_cmd::{BuildParams, build_params, dev_bin, release_bin};
use crate::context::Context;
use crate::options_flags::dry_run::DryRunFlag;
use crate::subcommand::SubcommandRunner;

/// The trunk the report compares the binary's commit with.
const TRUNK: &str = "main";

/// Copy the release build into the install directory.
#[derive(Args, Debug)]
pub struct InstallArgs {
    #[command(flatten)]
    pub dry_run: DryRunFlag,

    /// Workspace to install from [default: the workspace containing
    /// the current directory]
    #[arg(short = 'R', long, value_name = "DIR")]
    pub repo: Option<PathBuf>,

    /// Copy into DIR instead of `[install] release-dir`, for this run
    #[arg(long, value_name = "DIR")]
    pub dir: Option<PathBuf>,
}

/// Copy the dev build into the install directory, as `<name>-dev`.
#[derive(Args, Debug)]
pub struct InstallDevArgs {
    #[command(flatten)]
    pub dry_run: DryRunFlag,

    /// Workspace to install from [default: the workspace containing
    /// the current directory]
    #[arg(short = 'R', long, value_name = "DIR")]
    pub repo: Option<PathBuf>,

    /// Copy into DIR instead of `[install] dev-dir`, for this run
    #[arg(long, value_name = "DIR")]
    pub dir: Option<PathBuf>,
}

/// What the binary's `-V` says: its version and, from the build
/// metadata after `+`, its commit and the state after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reported {
    /// The version before any `+`.
    pub version: String,
    /// The commit prefix after `+`, when the build reports one.
    pub commit: Option<String>,
    /// The state after the commit, `dirty` or `unknown`, when not clean.
    pub state: Option<String>,
}

/// Everything the report and the refusals judge, gathered before
/// either is made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    /// The local trunk's commit, `None` when there is no such bookmark.
    pub trunk: Option<String>,
    /// The remote trunk's commit, `None` when it is not known.
    pub trunk_origin: Option<String>,
    /// The manifest's version.
    pub version: String,
    /// True when the binary is there.
    pub bin_exists: bool,
    /// What the binary's `-V` said, `None` when it could not be run or
    /// read.
    pub reported: Option<Reported>,
    /// True when the destination directory exists.
    pub dest_dir_exists: bool,
}

/// Per-invocation install inputs: the build to run first, where its
/// binary is and where it goes, and whether to copy.
#[derive(Debug)]
pub struct InstallParams {
    /// `install` or `install-dev`, prefixing the run's messages.
    pub label: &'static str,
    /// `build.release-bin` or `build.dev-bin`, the key naming the
    /// binary, for a refusal to point at.
    pub bin_key: &'static str,
    /// The work repo root.
    pub root: PathBuf,
    /// The build, `vc-x1 build`'s or `vc-x1 build-dev`'s, run before
    /// anything else.
    pub build: BuildParams,
    /// The binary, `[build] release-bin` or `[build] dev-bin`.
    pub bin: PathBuf,
    /// The destination, `--dir` or `[install] release-dir` or
    /// `dev-dir` joined with the binary's name, `suffix` appended.
    pub dest: PathBuf,
    /// `[install] dev-suffix` for the dev build, `-dev` when absent,
    /// and empty for the release.
    pub suffix: String,
    /// `[install] release` or `dev`, run in place of the copy when
    /// present.
    pub list: Option<Vec<String>>,
    /// `install.release` or `install.dev`, naming the list in its
    /// messages.
    pub list_key: &'static str,
    /// The keys whose resolved values the list's placeholders name,
    /// the binary's, the directory's, and for the dev build the
    /// suffix's.
    pub names: &'static [&'static str],
    /// Build and report, and copy nothing.
    pub dry_run: bool,
}

impl TryFrom<&InstallArgs> for InstallParams {
    type Error = String;

    /// The release: see [`install_params`].
    fn try_from(a: &InstallArgs) -> Result<Self, String> {
        install_params(false, a.repo.as_deref(), a.dir.as_deref(), a.dry_run.value)
    }
}

impl TryFrom<&InstallDevArgs> for InstallParams {
    type Error = String;

    /// The dev build: see [`install_params`].
    fn try_from(a: &InstallDevArgs) -> Result<Self, String> {
        install_params(true, a.repo.as_deref(), a.dir.as_deref(), a.dry_run.value)
    }
}

/// Resolve the workspace, the install directory, `dir` before
/// `[install] release-dir` or `dev-dir`, the build, the dev build and
/// its suffix when `dev`, and the optional install list. A missing
/// config key, and a dev copy that would replace the release, are
/// errors here, before the build runs, and every fact about the binary
/// is gathered after it.
pub fn install_params(
    dev: bool,
    repo: Option<&Path>,
    dir: Option<&Path>,
    dry_run: bool,
) -> Result<InstallParams, String> {
    let (label, dir_key, list_key) = if dev {
        ("install-dev", "install.dev-dir", "install.dev")
    } else {
        ("install", "install.release-dir", "install.release")
    };
    let (bin_key, names): (&str, &'static [&'static str]) = if dev {
        (
            "build.dev-bin",
            &["build.dev-bin", "install.dev-dir", "install.dev-suffix"],
        )
    } else {
        (
            "build.release-bin",
            &["build.release-bin", "install.release-dir"],
        )
    };
    let root = match repo {
        Some(p) => crate::common::find_workspace_root_from(p)
            .ok_or_else(|| format!("{label}: '{}' is not in a vc-x1 workspace", p.display()))?,
        None => crate::common::find_workspace_root()
            .ok_or_else(|| format!("{label}: not in a vc-x1 workspace (no config found)"))?,
    };
    let dest_dir = dest_dir(&root, dir, dir_key).map_err(|e| format!("{label}: {e}"))?;
    let build = build_params(dev, Some(&root))?;
    let bin = if dev {
        dev_bin(&root)?
    } else {
        release_bin(&root)?
    };
    let suffix = if dev {
        dev_suffix(&root).map_err(|e| format!("{label}: {e}"))?
    } else {
        String::new()
    };
    let dest = dest_dir.join(dest_name(&bin, &suffix).map_err(|e| format!("{label}: {e}"))?);
    let list = crate::command_list::load_optional(&root, list_key, label)?;
    match &list {
        Some(list) => crate::command_list::check_placeholders(list, names, list_key, label)?,
        None if dev => guard_release(&root, &dest).map_err(|e| format!("{label}: {e}"))?,
        None => {}
    }
    Ok(InstallParams {
        label,
        bin_key,
        root,
        build,
        bin,
        dest,
        suffix,
        list,
        list_key,
        names,
        dry_run,
    })
}

/// The name `bin` takes in the install directory: its own, with
/// `suffix` appended, empty for the release.
fn dest_name(bin: &Path, suffix: &str) -> Result<String, String> {
    let name = bin
        .file_name()
        .ok_or_else(|| format!("'{}' names no file", bin.display()))?
        .to_string_lossy();
    Ok(format!("{name}{suffix}"))
}

/// `[install] dev-suffix` at `root`, the schema's default, `-dev`,
/// when absent. A suffix holding a path separator is an error, since
/// it would move the copy out of the install directory.
fn dev_suffix(root: &Path) -> Result<String, String> {
    let cfg = crate::config_md::load(root)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("no config in '{}'", root.display()))?;
    let suffix = match crate::toml_simple::toml_get(&cfg.map, "install.dev-suffix") {
        Some(s) => s.clone(),
        None => crate::config_schema::INSTALL_DEV_SUFFIX_DEFAULT.to_string(),
    };
    if suffix.contains(std::path::is_separator) {
        return Err(format!(
            "{}: `install.dev-suffix` is '{suffix}', which holds a path separator",
            cfg.path.display()
        ));
    }
    Ok(suffix)
}

/// Refuse a dev copy to `dest` that is the release's own destination,
/// `[install] release-dir` joined with the release's file name, since
/// it would replace the release. A workspace with no release install
/// has none to replace.
fn guard_release(root: &Path, dest: &Path) -> Result<(), String> {
    let (Ok(dir), Ok(bin)) = (install_dir(root, "install.release-dir"), release_bin(root)) else {
        return Ok(());
    };
    let release = dir.join(dest_name(&bin, "")?);
    if !same_place(dest, &release) {
        return Ok(());
    }
    Err(format!(
        "the dev build's destination '{}' is the release's, so the copy would replace it: give \
         `install.dev-suffix` a value, or put `install.dev-dir` or --dir elsewhere",
        dest.display()
    ))
}

/// True when `a` and `b` name the same file: the same file name in the
/// same directory, each directory compared canonical when it exists.
fn same_place(a: &Path, b: &Path) -> bool {
    // OK: a directory that does not exist is compared as written
    let dir = |p: &Path| {
        p.parent()
            .map(|d| std::fs::canonicalize(d).unwrap_or_else(|_| d.to_path_buf()))
    };
    a.file_name() == b.file_name() && dir(a) == dir(b)
}

/// Gather the facts about the binary at `bin`, after the build, and
/// about the trunk of the work repo at `root`.
pub fn gather(root: &Path, bin: &Path, dest: &Path) -> Result<Facts, String> {
    Ok(Facts {
        trunk: crate::jj::cid_of(root, TRUNK).ok(),
        trunk_origin: crate::jj::cid_of(root, &format!("{TRUNK}@origin")).ok(),
        version: manifest_version(root)?,
        bin_exists: bin.is_file(),
        reported: run_version(bin).as_deref().and_then(parse_reported),
        dest_dir_exists: dest.parent().is_some_and(Path::is_dir),
    })
}

/// The directory the copy goes to: `dir`, the `--dir` value, against
/// the current directory when relative, when given, else the
/// directory `key` names at `root`. Absolute either way, since an
/// install list runs from `root` and takes it as a placeholder.
fn dest_dir(root: &Path, dir: Option<&Path>, key: &str) -> Result<PathBuf, String> {
    match dir {
        Some(d) if d.is_absolute() => Ok(d.to_path_buf()),
        Some(d) => std::env::current_dir()
            .map(|cwd| cwd.join(d))
            .map_err(|e| format!("--dir '{}': no current directory: {e}", d.display())),
        None => install_dir(root, key),
    }
}

/// The directory `key`, `install.release-dir` or `install.dev-dir`,
/// names at `root`: `~` and `~/...` against the home directory, any
/// other relative value against the config file's directory, an
/// absolute one as is. Its absence is the workspace having nothing of
/// that build to install.
pub fn install_dir(root: &Path, key: &str) -> Result<PathBuf, String> {
    let cfg = crate::config_md::load(root)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("no config in '{}'", root.display()))?;
    let value = crate::toml_simple::toml_get(&cfg.map, key).ok_or_else(|| {
        format!(
            "{}: no `{key}`, so this workspace has nothing to install. Name the \
             directory the build is copied into, such as `~/.cargo/bin` (see `vc-x1 \
             config work`), or pass --dir",
            cfg.path.display()
        )
    })?;
    // OK: a loaded config's path is a file, so it has a parent
    let cfg_dir = cfg.path.parent().unwrap_or(root);
    expand_dir(
        cfg_dir,
        key,
        value,
        crate::jj::session::home_dir().as_deref(),
    )
}

/// Expand the directory `key`'s `value`: `~` against `home`, else
/// against `cfg_dir` when relative.
fn expand_dir(
    cfg_dir: &Path,
    key: &str,
    value: &str,
    home: Option<&Path>,
) -> Result<PathBuf, String> {
    let rest = if value == "~" {
        Some("")
    } else {
        value.strip_prefix("~/")
    };
    match rest {
        Some(rest) => home
            .map(|h| h.join(rest))
            .ok_or_else(|| format!("`{key}` is '{value}' and HOME is not set")),
        None => Ok(crate::common::resolve_repo_path(cfg_dir, value)),
    }
}

/// The manifest's version, `[package] version` in the `Cargo.toml` at
/// `root`.
pub fn manifest_version(root: &Path) -> Result<String, String> {
    let path = root.join("Cargo.toml");
    let map = crate::toml_simple::toml_load(&path)
        .map_err(|e| format!("cannot read '{}': {e}", path.display()))?;
    crate::toml_simple::toml_get(&map, "package.version")
        .cloned()
        .ok_or_else(|| format!("'{}' has no `[package] version`", path.display()))
}

/// Run `bin -V` and return what it prints, `None` when it cannot be
/// run or fails.
pub(crate) fn run_version(bin: &Path) -> Option<String> {
    // Register entry 7 (clippy.toml): install runs the binary it is
    // about to copy, `-V` alone, to report it.
    #[allow(clippy::disallowed_methods)]
    let out = std::process::Command::new(bin).arg("-V").output().ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}

/// Parse `-V`'s output: the first line `<name> <version>[+<commit>[.<state>]] ...`
/// whose second word starts with a digit, so a line printed before the
/// version is passed over. `None` when no line has a version, which
/// install refuses as unreadable.
pub fn parse_reported(output: &str) -> Option<Reported> {
    let word = output.lines().find_map(|line| {
        line.split_whitespace()
            .nth(1)
            .filter(|w| w.starts_with(|c: char| c.is_ascii_digit()))
    })?;
    let (version, meta) = match word.split_once('+') {
        Some((v, m)) => (v, Some(m)),
        None => (word, None),
    };
    let (commit, state) = match meta {
        None => (None, None),
        Some(m) => match m.split_once('.') {
            Some((c, s)) => (Some(c.to_string()), Some(s.to_string())),
            None => (Some(m.to_string()), None),
        },
    };
    Some(Reported {
        version: version.to_string(),
        commit,
        state,
    })
}

/// True when the commit prefix `c` names the commit id `id`.
fn names(c: &str, id: Option<&String>) -> bool {
    id.is_some_and(|id| id.starts_with(c))
}

/// The report: what is being copied, one fact per line, for the user
/// to judge. Nothing in it refuses.
pub fn report(f: &Facts) -> Vec<String> {
    let mut out = Vec::new();
    let Some(r) = &f.reported else {
        return out;
    };
    let mut version = format!("version {}", r.version);
    if r.version.contains('-') {
        version.push_str(", a suffixed (pre-release) version");
    }
    if r.version != f.version {
        version.push_str(&format!(", the manifest's is {}", f.version));
    }
    out.push(version);
    match &r.commit {
        None => {
            out.push("commit unknown: the binary reports none".to_string());
            out.push("state unknown".to_string());
        }
        Some(c) => {
            let on_trunk = names(c, f.trunk.as_ref());
            let on_origin = names(c, f.trunk_origin.as_ref());
            let where_ = match (on_trunk, on_origin) {
                (true, true) => format!("`{TRUNK}` and `{TRUNK}@origin`"),
                (true, false) => format!("`{TRUNK}`, not yet `{TRUNK}@origin`"),
                (false, true) => format!("`{TRUNK}@origin`, not the local `{TRUNK}`"),
                (false, false) => format!("not `{TRUNK}`"),
            };
            out.push(format!("commit {c}, {where_}"));
            // A commit with no state after it is a clean build.
            let state = match &r.state {
                Some(s) => s.as_str(),
                None => "clean",
            };
            out.push(format!("state {state}"));
        }
    }
    out
}

/// Every reason the copy itself would be wrong, empty when it may go,
/// `bin_key` the key naming `bin`.
pub fn refusals(f: &Facts, bin: &Path, dest: &Path, bin_key: &str) -> Vec<String> {
    let mut out = Vec::new();
    if !f.bin_exists {
        out.push(format!(
            "no binary at '{}' after the build: check `{bin_key}`",
            bin.display()
        ));
    } else if f.reported.is_none() {
        out.push(format!("'{}' -V could not be run or read", bin.display()));
    }
    if !f.dest_dir_exists {
        // OK: dest is a directory joined with a file name, so it has a parent
        let dir = dest.parent().unwrap_or(dest);
        out.push(format!(
            "the install directory '{}' does not exist",
            dir.display()
        ));
    }
    out
}

/// Copy `bin` to `dest` as a temporary file beside it renamed into
/// place, so `dest` is never half written and a running copy of the
/// old binary is undisturbed. The temporary file is removed on failure.
pub fn deploy(bin: &Path, dest: &Path) -> Result<(), String> {
    // OK: dest is a directory joined with a file name, so both exist
    let dir = dest.parent().unwrap_or(Path::new("."));
    // OK: dest is joined with the binary's file name, so it has one
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = dir.join(format!(".{name}.vc-x1-install"));
    let result = std::fs::copy(bin, &tmp)
        .map_err(|e| format!("cannot copy to '{}': {e}", tmp.display()))
        .and_then(|_| {
            std::fs::rename(&tmp, dest).map_err(|e| {
                format!(
                    "cannot move '{}' into '{}': {e}",
                    tmp.display(),
                    dest.display()
                )
            })
        });
    if result.is_err() {
        std::fs::remove_file(&tmp).ok();
    }
    result
}

impl SubcommandRunner for InstallArgs {
    type Params = InstallParams;

    /// Delegate to the `TryFrom<&InstallArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        InstallParams::try_from(self)
    }

    /// Run the `install` op (`ctx` unused: the op is fully
    /// parameterized by `Params`).
    fn run(_ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        install(params)
    }
}

impl SubcommandRunner for InstallDevArgs {
    type Params = InstallParams;

    /// Delegate to the `TryFrom<&InstallDevArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        InstallParams::try_from(self)
    }

    /// Run the same op as `install`, its params naming the dev build.
    fn run(_ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        install(params)
    }
}

/// Run the build, as `vc-x1 build` or `vc-x1 build-dev` does, then
/// report the binary and copy it unless refused or a dry run. Cargo
/// rebuilds only what changed, so with nothing changed the binary
/// already tested is the one copied.
pub fn install(params: &InstallParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("{}: entry params={params:?}", params.label);
    crate::build_cmd::build(&params.build)?;
    let facts = gather(&params.root, &params.bin, &params.dest)
        .map_err(|e| format!("{}: {e}", params.label))?;
    copy_reported(params, &facts)
}

/// Print the report, refuse with every reason at once when the copy
/// would be wrong, then, unless it is a dry run, run the install list
/// when there is one, else copy the binary to its destination.
pub fn copy_reported(
    params: &InstallParams,
    facts: &Facts,
) -> Result<(), Box<dyn std::error::Error>> {
    let (label, bin, dest) = (params.label, &params.bin, &params.dest);
    match &params.list {
        Some(_) => info!("{label}: '{}' -> list `{}`", bin.display(), params.list_key),
        None => info!("{label}: '{}' -> '{}'", bin.display(), dest.display()),
    }
    for line in report(facts) {
        info!("{label}:   {line}");
    }
    let reasons = refusals(facts, bin, dest, params.bin_key);
    if !reasons.is_empty() {
        let list: String = reasons.iter().map(|r| format!("\n  - {r}")).collect();
        return Err(format!(
            "{label}: refused, nothing copied to '{}':{list}",
            dest.display()
        )
        .into());
    }
    if params.dry_run {
        match &params.list {
            Some(_) => info!("{label}: [dry-run] ran nothing of `{}`", params.list_key),
            None => info!("{label}: [dry-run] copied nothing"),
        }
        return Ok(());
    }
    if let Some(list) = &params.list {
        return run_list(params, list);
    }
    deploy(bin, dest).map_err(|e| format!("{label}: {e}"))?;
    info!("{label}: copied");
    Ok(())
}

/// Run the install list from the root, in place of the copy, its
/// placeholders the binary and the directory, resolved for the run.
fn run_list(params: &InstallParams, list: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let bin = params.bin.to_string_lossy();
    // OK: dest is a directory joined with a file name, so it has a parent
    let dir = params
        .dest
        .parent()
        .unwrap_or(&params.dest)
        .to_string_lossy();
    let mut vars = vec![
        (params.names[0], bin.as_ref()),
        (params.names[1], dir.as_ref()),
    ];
    if let Some(name) = params.names.get(2) {
        vars.push((name, params.suffix.as_str()));
    }
    crate::command_list::run(
        &params.root,
        params.list_key,
        list,
        &[],
        &vars,
        params.label,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cli, Commands};
    use clap::Parser;

    const C: &str = "0123456789abcdef0123456789abcdef01234567";

    /// Facts for a clean build of a pushed `main`.
    fn good() -> Facts {
        Facts {
            trunk: Some(C.to_string()),
            trunk_origin: Some(C.to_string()),
            version: "0.85.7".to_string(),
            bin_exists: true,
            reported: Some(Reported {
                version: "0.85.7".to_string(),
                commit: Some(C[..12].to_string()),
                state: None,
            }),
            dest_dir_exists: true,
        }
    }

    fn reported(f: &Facts) -> String {
        report(f).join("\n")
    }

    fn refused(f: &Facts) -> String {
        refusals(
            f,
            Path::new("target/release/vc-x1"),
            Path::new("/home/u/.cargo/bin/vc-x1"),
            "build.release-bin",
        )
        .join("\n")
    }

    /// Install params copying `bin` to `dest`, their build never run.
    fn params(bin: &Path, dest: &Path, dry_run: bool) -> InstallParams {
        InstallParams {
            label: "install",
            bin_key: "build.release-bin",
            root: PathBuf::from("."),
            build: BuildParams {
                label: "build",
                key: "build.release",
                commands: Vec::new(),
                root: PathBuf::from("."),
                provenance: None,
                bin: None,
                dry_run: false,
            },
            bin: bin.to_path_buf(),
            dest: dest.to_path_buf(),
            list: None,
            suffix: String::new(),
            list_key: "install.release",
            names: &["build.release-bin", "install.release-dir"],
            dry_run,
        }
    }

    /// Unique tempdir.
    fn tempdir(tag: &str) -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("vc-x1-install-{tag}-{ts}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn flags_parse() {
        let cli = Cli::try_parse_from(["vc-x1", "install", "--dry-run", "-R", "x"]).unwrap();
        match cli.command {
            Some(Commands::Install(a)) => {
                assert!(a.dry_run.value);
                assert_eq!(a.repo, Some(PathBuf::from("x")));
                assert_eq!(a.dir, None);
            }
            _ => panic!("expected Install"),
        }
        let cli = Cli::try_parse_from(["vc-x1", "install", "--dir", "tmp/bin"]).unwrap();
        match cli.command {
            Some(Commands::Install(a)) => assert_eq!(a.dir, Some(PathBuf::from("tmp/bin"))),
            _ => panic!("expected Install"),
        }
        let cli =
            Cli::try_parse_from(["vc-x1", "install-dev", "--dry-run", "--dir", "d", "-R", "x"])
                .unwrap();
        match cli.command {
            Some(Commands::InstallDev(a)) => {
                assert!(a.dry_run.value);
                assert_eq!(a.dir, Some(PathBuf::from("d")));
                assert_eq!(a.repo, Some(PathBuf::from("x")));
            }
            _ => panic!("expected InstallDev"),
        }
    }

    /// The file name takes the suffix, none for the release.
    #[test]
    fn dest_name_forms() {
        let bin = Path::new("target/debug/vc-x1");
        assert_eq!(dest_name(bin, "").unwrap(), "vc-x1");
        assert_eq!(dest_name(bin, "-dev").unwrap(), "vc-x1-dev");
        assert!(
            dest_name(Path::new("/"), "-dev")
                .unwrap_err()
                .contains("names no file")
        );
    }

    /// The suffix is the schema's `-dev` when absent, the config's
    /// when set, empty included, and a path separator in it is an
    /// error.
    #[test]
    fn dev_suffix_forms() {
        let dir = tempdir("suffix");
        let cfg = |extra: &str| {
            std::fs::write(
                dir.join(".vc-config.md"),
                format!("```toml\n[repos]\nwork = \".\"\n{extra}```\n"),
            )
            .unwrap()
        };
        cfg("");
        assert_eq!(dev_suffix(&dir).unwrap(), "-dev");
        cfg("[install]\ndev-suffix = \".dbg\"\n");
        assert_eq!(dev_suffix(&dir).unwrap(), ".dbg");
        cfg("[install]\ndev-suffix = \"\"\n");
        assert_eq!(dev_suffix(&dir).unwrap(), "");
        cfg("[install]\ndev-suffix = \"/x\"\n");
        assert!(dev_suffix(&dir).unwrap_err().contains("path separator"));
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A dev copy whose destination is the release's is refused, the
    /// directories compared canonical, and one beside it, or in a
    /// workspace with no release install, is not.
    #[test]
    fn a_dev_copy_onto_the_release_is_refused() {
        let dir = tempdir("guard");
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        let cfg = |extra: &str| {
            std::fs::write(
                dir.join(".vc-config.md"),
                format!(
                    "```toml\n[repos]\nwork = \".\"\n[build]\nrelease-bin = \"rel/app\"\n{extra}```\n"
                ),
            )
            .unwrap()
        };
        cfg("[install]\nrelease-dir = \"bin\"\n");
        let e = guard_release(&dir, &dir.join("bin/app")).unwrap_err();
        assert!(e.contains("is the release's"), "{e}");
        assert!(
            guard_release(&dir, &dir.join("bin/../bin/app")).is_err(),
            "canonical"
        );
        guard_release(&dir, &dir.join("bin/app-dev")).unwrap();
        guard_release(&dir, &dir.join("other/app")).unwrap();
        cfg("");
        guard_release(&dir, &dir.join("bin/app")).unwrap();
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A clean build of a pushed `main` reports so, and is not refused.
    #[test]
    fn a_clean_build_of_main_reports_so() {
        let f = good();
        assert_eq!(
            reported(&f),
            "version 0.85.7\n\
             commit 0123456789ab, `main` and `main@origin`\n\
             state clean"
        );
        assert_eq!(refused(&f), "");
    }

    /// A dirty build of a cycle's bookmark at a suffixed version is
    /// reported as such, and is not refused: the release process
    /// judges it, not install.
    #[test]
    fn a_dirty_cycle_build_is_reported_not_refused() {
        let mut f = good();
        f.trunk = Some("d".repeat(40));
        f.trunk_origin = Some("d".repeat(40));
        f.version = "0.85.7-3".to_string();
        let r = f.reported.as_mut().unwrap();
        r.version = "0.85.7-3".to_string();
        r.state = Some("dirty".to_string());
        let rep = reported(&f);
        assert!(rep.contains("a suffixed (pre-release) version"), "{rep}");
        assert!(rep.contains("commit 0123456789ab, not `main`"), "{rep}");
        assert!(rep.contains("state dirty"), "{rep}");
        assert_eq!(refused(&f), "");
    }

    /// Where the commit sits against the local and remote trunk.
    #[test]
    fn the_commit_is_placed_against_the_trunk() {
        let mut f = good();
        f.trunk_origin = Some("e".repeat(40));
        assert!(reported(&f).contains("`main`, not yet `main@origin`"));
        let mut f = good();
        f.trunk = Some("e".repeat(40));
        assert!(reported(&f).contains("`main@origin`, not the local `main`"));
        let mut f = good();
        f.trunk = None;
        f.trunk_origin = None;
        assert!(reported(&f).contains(", not `main`"));
    }

    /// The version line names a binary older or newer than the
    /// manifest.
    #[test]
    fn a_version_other_than_the_manifests_is_named() {
        let mut f = good();
        f.version = "0.85.8".to_string();
        assert!(reported(&f).contains("version 0.85.7, the manifest's is 0.85.8"));
    }

    /// A binary whose `-V` carries no commit, another adopter's, is
    /// reported as unknown, and is not refused.
    #[test]
    fn a_binary_reporting_no_commit_is_unknown() {
        let mut f = good();
        f.reported.as_mut().unwrap().commit = None;
        let rep = reported(&f);
        assert!(
            rep.contains("commit unknown: the binary reports none"),
            "{rep}"
        );
        assert!(rep.contains("state unknown"), "{rep}");
        assert_eq!(refused(&f), "");
    }

    /// The copy is refused only when it would be wrong: no binary, a
    /// binary whose `-V` will not run, or no destination directory,
    /// one reason for each fault.
    #[test]
    fn only_a_wrong_copy_is_refused() {
        let mut f = good();
        f.reported = None;
        assert!(refused(&f).contains("-V could not be run"));
        assert_eq!(reported(&f), "", "nothing to report without -V");
        let mut f = good();
        f.bin_exists = false;
        f.reported = None;
        let r = refused(&f);
        assert!(r.contains("no binary at 'target/release/vc-x1'"), "{r}");
        assert!(
            r.contains("check `build.release-bin`"),
            "names the key: {r}"
        );
        assert!(
            !r.contains("could not be run"),
            "one reason for one fault: {r}"
        );
        let mut f = good();
        f.dest_dir_exists = false;
        assert!(refused(&f).contains("'/home/u/.cargo/bin' does not exist"));
    }

    #[test]
    fn parse_reported_forms() {
        let r = parse_reported("vc-x1 0.85.7+fcc7f222a774 (agent-files v0.3.0)").unwrap();
        assert_eq!(r.version, "0.85.7");
        assert_eq!(r.commit.as_deref(), Some("fcc7f222a774"));
        assert_eq!(r.state, None);
        let r = parse_reported("vc-x1 0.85.7-3+fcc7f222a774.dirty").unwrap();
        assert_eq!(r.version, "0.85.7-3");
        assert_eq!(r.state.as_deref(), Some("dirty"));
        let r = parse_reported("iiac-perf 0.29.2").unwrap();
        assert_eq!((r.version.as_str(), r.commit), ("0.29.2", None));
        assert_eq!(parse_reported("oneword"), None);
        let r = parse_reported("wink: hi\nvc-x1 0.85.7-3+fcc7f222a774.dirty\n").unwrap();
        assert_eq!(
            (r.version.as_str(), r.state.as_deref()),
            ("0.85.7-3", Some("dirty")),
            "a line printed before the version is passed over"
        );
        assert_eq!(
            parse_reported("wink: hi\n"),
            None,
            "no version is unreadable"
        );
        assert_eq!(parse_reported(""), None);
    }

    #[test]
    fn expand_dir_forms() {
        let cfg = Path::new("/w");
        let home = Path::new("/home/u");
        assert_eq!(
            expand_dir(cfg, "install.release-dir", "~/.cargo/bin", Some(home)).unwrap(),
            PathBuf::from("/home/u/.cargo/bin")
        );
        assert_eq!(
            expand_dir(cfg, "install.release-dir", "~", Some(home)).unwrap(),
            PathBuf::from("/home/u")
        );
        assert_eq!(
            expand_dir(cfg, "install.release-dir", "/opt/bin", Some(home)).unwrap(),
            PathBuf::from("/opt/bin")
        );
        assert_eq!(
            expand_dir(cfg, "install.release-dir", "out/bin", Some(home)).unwrap(),
            PathBuf::from("/w/out/bin")
        );
        let e = expand_dir(cfg, "install.release-dir", "~/x", None).unwrap_err();
        assert!(
            e.contains("`install.release-dir`") && e.contains("HOME"),
            "{e}"
        );
        assert_eq!(
            expand_dir(cfg, "install.release-dir", "~user/x", Some(home)).unwrap(),
            PathBuf::from("/w/~user/x"),
            "only `~` and `~/` expand"
        );
    }

    /// The deploy replaces the destination whole and leaves no
    /// temporary file, keeping the binary's permissions.
    #[test]
    fn deploy_replaces_the_destination() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir("deploy");
        let bin = dir.join("release");
        std::fs::write(&bin, "the new release").unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        let dest_dir = dir.join("bin");
        std::fs::create_dir_all(&dest_dir).unwrap();
        let dest = dest_dir.join("app");
        std::fs::write(&dest, "the old release").unwrap();
        deploy(&bin, &dest).unwrap();
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "the new release");
        let mode = std::fs::metadata(&dest).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o755);
        let left: Vec<_> = std::fs::read_dir(&dest_dir).unwrap().collect();
        assert_eq!(left.len(), 1, "no temporary file left");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// The copy goes when it is not refused, whatever the report says,
    /// a dry run copies nothing, and a refusal lists every reason and
    /// copies nothing.
    #[test]
    fn copy_reported_copies_unless_refused() {
        let dir = tempdir("install");
        let bin = dir.join("release");
        std::fs::write(&bin, "the release").unwrap();
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        let dest = dir.join("bin/app");
        let mut facts = good();
        facts.version = "0.85.7-3".to_string();
        copy_reported(&params(&bin, &dest, true), &facts).unwrap();
        assert!(!dest.exists(), "a dry run copied");
        copy_reported(&params(&bin, &dest, false), &facts).unwrap();
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "the release");
        std::fs::remove_file(&dest).unwrap();
        facts.reported = None;
        facts.dest_dir_exists = false;
        let e = copy_reported(&params(&bin, &dest, false), &facts)
            .unwrap_err()
            .to_string();
        assert!(e.contains("refused, nothing copied"), "{e}");
        assert!(
            e.contains("could not be run") && e.contains("does not exist"),
            "every reason: {e}"
        );
        assert!(!dest.exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    /// The install directory is checked before the release build is
    /// resolved, so a workspace with nothing to install builds nothing.
    #[test]
    fn no_install_dir_is_an_error_before_the_build() {
        let dir = tempdir("nodir");
        std::fs::write(
            dir.join(".vc-config.md"),
            "```toml\n[repos]\nwork = \".\"\n[build]\nrelease = [\"true\"]\n\
             release-bin = \"app\"\n```\n",
        )
        .unwrap();
        let args = InstallArgs {
            dry_run: DryRunFlag { value: true },
            repo: Some(dir.clone()),
            dir: None,
        };
        let e = InstallParams::try_from(&args).unwrap_err();
        assert!(e.contains("nothing to install"), "{e}");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// `--dir` replaces the build's directory key for the run, and
    /// needs no key to do it, and each build reads its own key.
    #[test]
    fn dir_overrides_the_install_dir() {
        let dir = tempdir("dirflag");
        let cfg = "```toml\n[repos]\nwork = \".\"\n";
        std::fs::write(dir.join(".vc-config.md"), format!("{cfg}```\n")).unwrap();
        let flag = Path::new("/scratch/bin");
        let (rel, dev) = ("install.release-dir", "install.dev-dir");
        assert_eq!(dest_dir(&dir, Some(flag), rel).unwrap(), flag);
        let e = dest_dir(&dir, None, dev).unwrap_err();
        assert!(e.contains("no `install.dev-dir`"), "{e}");
        assert!(e.contains("nothing to install"), "{e}");
        std::fs::write(
            dir.join(".vc-config.md"),
            format!("{cfg}[install]\nrelease-dir = \"/opt/bin\"\ndev-dir = \"/opt/dev\"\n```\n"),
        )
        .unwrap();
        assert_eq!(dest_dir(&dir, Some(flag), dev).unwrap(), flag, "--dir wins");
        assert_eq!(
            dest_dir(&dir, None, rel).unwrap(),
            PathBuf::from("/opt/bin")
        );
        assert_eq!(
            dest_dir(&dir, None, dev).unwrap(),
            PathBuf::from("/opt/dev")
        );
        assert_eq!(
            dest_dir(&dir, Some(Path::new("tmp/bin")), rel).unwrap(),
            std::env::current_dir().unwrap().join("tmp/bin"),
            "a relative --dir is made absolute against the current directory"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// An install list runs in place of the copy, from the root, its
    /// placeholders the binary and the directory, checked by value by
    /// a script taking them as arguments, and a dry run runs nothing.
    #[test]
    fn a_list_runs_in_place_of_the_copy() {
        let dir = tempdir("list");
        let bin = dir.join("release");
        std::fs::write(&bin, "the release").unwrap();
        std::fs::create_dir_all(dir.join("bin dir")).unwrap();
        let dest = dir.join("bin dir/app");
        std::fs::write(
            dir.join("check.sh"),
            format!(
                "[ \"$1\" = '{}' ] && [ \"$2\" = '{}' ] && [ $# = 2 ] && touch list-ran\n",
                bin.display(),
                dir.join("bin dir").display(),
            ),
        )
        .unwrap();
        let mut p = params(&bin, &dest, true);
        p.root = dir.clone();
        p.list = Some(vec![
            "sh check.sh {build.release-bin} {install.release-dir}".to_string(),
        ]);
        copy_reported(&p, &good()).unwrap();
        assert!(!dir.join("list-ran").exists(), "a dry run ran the list");
        p.dry_run = false;
        copy_reported(&p, &good()).unwrap();
        assert!(dir.join("list-ran").exists(), "the list did not run");
        assert!(!dest.exists(), "the list ran and the copy was made too");
        p.list = Some(vec!["false".to_string()]);
        let e = copy_reported(&p, &good()).unwrap_err().to_string();
        assert!(
            e.contains("`false` failed") && e.contains("install.release"),
            "{e}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// The dev list takes the suffix as its third placeholder.
    #[test]
    fn a_dev_list_takes_the_suffix() {
        let dir = tempdir("devlist");
        let bin = dir.join("debug");
        std::fs::write(&bin, "the dev build").unwrap();
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        let mut p = params(&bin, &dir.join("bin/app.dbg"), false);
        p.root = dir.clone();
        p.names = &["build.dev-bin", "install.dev-dir", "install.dev-suffix"];
        p.suffix = ".dbg".to_string();
        p.list = Some(vec![
            "cp {build.dev-bin} {install.dev-dir}/app{install.dev-suffix}".to_string(),
        ]);
        copy_reported(&p, &good()).unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.join("bin/app.dbg")).unwrap(),
            "the dev build"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// The equivalent of the built-in copy, the list this workspace
    /// sets, installs the binary under the name it writes.
    #[test]
    fn the_copy_as_a_list_installs() {
        let dir = tempdir("cplist");
        let bin = dir.join("release");
        std::fs::write(&bin, "the release").unwrap();
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        let dest = dir.join("bin/app");
        std::fs::write(&dest, "the old release").unwrap();
        let mut p = params(&bin, &dest, false);
        p.root = dir.clone();
        p.list = Some(vec![
            "cp {build.release-bin} {install.release-dir}/app.new".to_string(),
            "mv {install.release-dir}/app.new {install.release-dir}/app".to_string(),
        ]);
        copy_reported(&p, &good()).unwrap();
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "the release");
        assert_eq!(std::fs::read_dir(dir.join("bin")).unwrap().count(), 1);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A failed copy leaves the destination as it was and no temporary
    /// file.
    #[test]
    fn a_failed_deploy_leaves_the_old_binary() {
        let dir = tempdir("deployfail");
        let dest = dir.join("app");
        std::fs::write(&dest, "the old release").unwrap();
        let e = deploy(&dir.join("absent"), &dest).unwrap_err();
        assert!(e.contains("cannot copy"), "{e}");
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "the old release");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        std::fs::remove_dir_all(&dir).ok();
    }
}

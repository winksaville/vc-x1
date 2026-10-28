//! The `install` subcommand: build the chosen profile, then copy its
//! binary into the install directory, the command that touches the
//! PATH, after reporting what it copies. The profile is
//! `--profile NAME`, else `[build] default-profile`, else `release`.
//! The dev install was `install-dev`, and then `install --dev`, both
//! kept hidden to be refused naming `install --profile dev`.
//!
//! - `install` runs `[build] commands` first, exactly as
//!   `vc-x1 build --profile NAME` does. Cargo rebuilds only what
//!   changed, so with nothing changed the binary already tested is the
//!   one copied, and a missing or stale one is built rather than
//!   refused.
//! - The binary is the profile's `bin`. The destination is the one
//!   directory `[install] dir` names, `~` expanded, so the committed
//!   value fits every machine, the binary keeping its file name there
//!   with the profile's `suffix` appended, `vc-x1-dev` for a `-dev`
//!   suffix, so the builds of several profiles may share a directory
//!   and sit side by side on the PATH. `$PATH` is not searched.
//! - A copy that would land on another profile's destination, the same
//!   file name and suffix in the same directory, is refused before the
//!   build, since it would replace that profile's binary.
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
//! - `[install] commands`, an optional command list in `[build]`'s
//!   shape, replaces the copy when present: the build, the report, and
//!   the refusals are the same, and the list runs instead of the copy.
//!   Its placeholders are the chosen profile's properties,
//!   `{profile.bin}` resolved and `{profile.suffix}` empty when absent,
//!   and `{install.dir}`, `--dir` applied, so the list names every path
//!   it writes. With a list, the file name, suffix included, is the
//!   list's to write, and no collision is checked.
//! - `--dry-run` builds and reports, and copies nothing.
//! - `--dir <DIR>` copies into `DIR` instead of the configured
//!   directory, for one run, so a test of install leaves the PATH
//!   alone. A relative `DIR` is taken against the current directory,
//!   as any path argument is, and the workspace needs no directory key
//!   for it.
//! - A workspace with no `[install] dir` has nothing to install, and
//!   the command says so before building anything.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info};

use crate::build_cmd::{BuildParams, Profile, build_params, defined_profiles, resolve_profile};
use crate::context::Context;
use crate::options_flags::dry_run::DryRunFlag;
use crate::subcommand::SubcommandRunner;

/// The trunk the report compares the binary's commit with.
const TRUNK: &str = "main";

/// The key naming the install directory, and its placeholder.
const INSTALL_DIR_KEY: &str = "install.dir";

/// The key of the optional install list, shared by every profile.
const INSTALL_COMMANDS_KEY: &str = "install.commands";

/// Copy the chosen profile's build into the install directory.
#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Build and install the profile NAME, a `[profile.<NAME>]` table,
    /// its `suffix` appended to the file name [default:
    /// `[build] default-profile`, else release]
    #[arg(long, value_name = "NAME")]
    pub profile: Option<String>,

    /// Retired: refused naming `--profile dev`
    #[arg(long, hide = true)]
    pub dev: bool,

    #[command(flatten)]
    pub dry_run: DryRunFlag,

    /// Workspace to install from [default: the workspace containing
    /// the current directory]
    #[arg(short = 'R', long, value_name = "DIR")]
    pub repo: Option<PathBuf>,

    /// Copy into DIR instead of `[install] dir`, for this run
    #[arg(long, value_name = "DIR")]
    pub dir: Option<PathBuf>,
}

/// Retired: `install-dev`, now `install --profile dev`, refused naming
/// it. Its flags are kept so an old invocation parses and is told the
/// fix.
#[derive(Args, Debug)]
pub struct InstallDevArgs {
    #[command(flatten)]
    pub dry_run: DryRunFlag,

    /// Workspace to install from [default: the workspace containing
    /// the current directory]
    #[arg(short = 'R', long, value_name = "DIR")]
    pub repo: Option<PathBuf>,

    /// Copy into DIR instead of `[install] dir`, for this run
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
/// binary is and where it goes, the install list and its
/// placeholders, and whether to copy.
#[derive(Debug)]
pub struct InstallParams {
    /// `install`, prefixing the run's messages.
    pub label: &'static str,
    /// `profile.<name>.bin`, the key naming the binary, for a refusal
    /// to point at.
    pub bin_key: String,
    /// The work repo root.
    pub root: PathBuf,
    /// The build, `vc-x1 build --profile <name>`'s, run before
    /// anything else.
    pub build: BuildParams,
    /// The binary, the profile's `bin`.
    pub bin: PathBuf,
    /// The destination, `--dir` or `[install] dir` joined with the
    /// binary's name, the profile's suffix appended.
    pub dest: PathBuf,
    /// `[install] commands`, run in place of the copy when present.
    pub list: Option<Vec<String>>,
    /// `install.commands`, naming the list in its messages.
    pub list_key: &'static str,
    /// The list's string placeholders, the profile's properties and
    /// `install.dir`, resolved for the run.
    pub vars: Vec<(String, String)>,
    /// The list's list-valued placeholders, the profile's.
    pub lists: Vec<(String, Vec<String>)>,
    /// Build and report, and copy nothing.
    pub dry_run: bool,
}

impl TryFrom<&InstallArgs> for InstallParams {
    type Error = String;

    /// The chosen profile: see [`install_params`]. `--dev` is refused.
    fn try_from(a: &InstallArgs) -> Result<Self, String> {
        if a.dev {
            return Err(crate::build_cmd::retired_dev_flag("install"));
        }
        install_params(
            a.profile.as_deref(),
            a.repo.as_deref(),
            a.dir.as_deref(),
            a.dry_run.value,
        )
    }
}

impl TryFrom<&InstallDevArgs> for InstallParams {
    type Error = String;

    /// Refused: `install-dev` is `install --profile dev`.
    fn try_from(_: &InstallDevArgs) -> Result<Self, String> {
        Err(crate::build_cmd::retired_dev("install"))
    }
}

/// Resolve the workspace, the install directory, `dir` before
/// `[install] dir`, the build of the profile `profile` names (see
/// [`resolve_profile`]), the destination, and the optional install
/// list, its placeholders checked. A missing config key, an unknown
/// placeholder, and a copy that would replace another profile's are
/// errors here, before the build runs, and every fact about the binary
/// is gathered after it.
pub fn install_params(
    profile: Option<&str>,
    repo: Option<&Path>,
    dir: Option<&Path>,
    dry_run: bool,
) -> Result<InstallParams, String> {
    let label = "install";
    let root = match repo {
        Some(p) => crate::common::find_workspace_root_from(p)
            .ok_or_else(|| format!("{label}: '{}' is not in a vc-x1 workspace", p.display()))?,
        None => crate::common::find_workspace_root()
            .ok_or_else(|| format!("{label}: not in a vc-x1 workspace (no config found)"))?,
    };
    let dest_dir = dest_dir(&root, dir, INSTALL_DIR_KEY).map_err(|e| format!("{label}: {e}"))?;
    let build = build_params(profile, Some(&root))?;
    let profile = &build.profile;
    let bin = profile.bin.clone();
    let dest =
        dest_dir.join(dest_name(&bin, &profile.suffix).map_err(|e| format!("{label}: {e}"))?);
    let mut vars = profile.vars.clone();
    vars.push((
        INSTALL_DIR_KEY.to_string(),
        dest_dir.to_string_lossy().into_owned(),
    ));
    let lists = profile.lists.clone();
    let list = crate::command_list::load_optional(&root, INSTALL_COMMANDS_KEY, label)?;
    match &list {
        Some(list) => {
            let names: Vec<&str> = vars.iter().map(|(n, _)| n.as_str()).collect();
            let list_names: Vec<&str> = lists.iter().map(|(n, _)| n.as_str()).collect();
            crate::command_list::check_placeholders(
                list,
                &names,
                &list_names,
                INSTALL_COMMANDS_KEY,
                label,
            )?
        }
        None => guard_others(&root, profile, &dest).map_err(|e| format!("{label}: {e}"))?,
    }
    let bin_key = format!("profile.{}.bin", profile.name);
    Ok(InstallParams {
        label,
        bin_key,
        root,
        build,
        bin,
        dest,
        list,
        list_key: INSTALL_COMMANDS_KEY,
        vars,
        lists,
        dry_run,
    })
}

/// The name `bin` takes in the install directory: its own, with
/// `suffix` appended, empty for none.
fn dest_name(bin: &Path, suffix: &str) -> Result<String, String> {
    let name = bin
        .file_name()
        .ok_or_else(|| format!("'{}' names no file", bin.display()))?
        .to_string_lossy();
    Ok(format!("{name}{suffix}"))
}

/// Refuse a copy of `profile` to `dest` whose file name is another
/// profile's in the same directory, its binary's file name with its
/// suffix appended, since the copy would replace that profile's. A
/// profile of the config's that does not resolve is passed over, since
/// it has no destination to replace.
fn guard_others(root: &Path, profile: &Profile, dest: &Path) -> Result<(), String> {
    let cfg = crate::config_md::load(root)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("no config in '{}'", root.display()))?;
    let name = &profile.name;
    for other in defined_profiles(&cfg.map) {
        if other == *name {
            continue;
        }
        let Ok(p) = resolve_profile(root, Some(&other), "install") else {
            continue;
        };
        let Ok(theirs) = dest_name(&p.bin, &p.suffix) else {
            continue;
        };
        if dest.file_name() == Some(std::ffi::OsStr::new(&theirs)) {
            return Err(format!(
                "the `{name}` profile's destination '{}' is the `{other}` profile's too, so the \
                 copy would replace it: give `profile.{name}.suffix` or `profile.{other}.suffix` \
                 a value, or pass --dir elsewhere",
                dest.display()
            ));
        }
    }
    Ok(())
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

/// The directory `key`, `install.dir`, names at `root`: `~` and
/// `~/...` against the home directory, any other relative value
/// against the config file's directory, an absolute one as is. Its
/// absence is the workspace having nothing to install.
pub fn install_dir(root: &Path, key: &str) -> Result<PathBuf, String> {
    let cfg = crate::config_md::load(root)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("no config in '{}'", root.display()))?;
    let value = crate::toml_simple::toml_get(&cfg.map, key).ok_or_else(|| {
        format!(
            "{}: no `{key}`, so this workspace has nothing to install. Name the \
             directory the build is copied into, such as `~/.cargo/bin` (see `vc-x1 \
             config -s work`), or pass --dir",
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

    /// Never reached, since the params are refused, but the same op
    /// as `install`.
    fn run(_ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        install(params)
    }
}

/// Run the build, as `vc-x1 build --profile <name>` does, then
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
    let reasons = refusals(facts, bin, dest, &params.bin_key);
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
/// placeholders the profile's properties and the directory, resolved
/// for the run.
fn run_list(params: &InstallParams, list: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let vars: Vec<(&str, &str)> = params
        .vars
        .iter()
        .map(|(n, v)| (n.as_str(), v.as_str()))
        .collect();
    let lists: Vec<(&str, &[String])> = params
        .lists
        .iter()
        .map(|(n, v)| (n.as_str(), v.as_slice()))
        .collect();
    crate::command_list::run(
        &params.root,
        params.list_key,
        list,
        &[],
        &vars,
        &lists,
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
            "profile.release.bin",
        )
        .join("\n")
    }

    /// Install params copying `bin` to `dest`, their build never run,
    /// the release profile's placeholders and the directory offered.
    fn params(bin: &Path, dest: &Path, dry_run: bool) -> InstallParams {
        let vars = vec![
            (
                "profile.bin".to_string(),
                bin.to_string_lossy().into_owned(),
            ),
            ("profile.suffix".to_string(), String::new()),
        ];
        let dir = dest.parent().unwrap().to_string_lossy().into_owned();
        InstallParams {
            label: "install",
            bin_key: "profile.release.bin".to_string(),
            root: PathBuf::from("."),
            build: BuildParams {
                label: "build",
                key: "build.commands",
                commands: Vec::new(),
                root: PathBuf::from("."),
                provenance: None,
                profile: Profile {
                    name: "release".to_string(),
                    vars: vars.clone(),
                    lists: Vec::new(),
                    bin: bin.to_path_buf(),
                    suffix: String::new(),
                },
                dry_run: false,
            },
            bin: bin.to_path_buf(),
            dest: dest.to_path_buf(),
            list: None,
            list_key: "install.commands",
            vars: [vars, vec![("install.dir".to_string(), dir)]].concat(),
            lists: Vec::new(),
            dry_run,
        }
    }

    /// A workspace at `dir` whose config is `toml`, a work repo at its
    /// root with a build list that runs nothing.
    fn write_config(dir: &Path, toml: &str) {
        std::fs::write(
            dir.join(".vc-config.md"),
            format!("```toml\n[repos]\nwork = \".\"\n[build]\ncommands = [\"true\"]\n{toml}```\n"),
        )
        .unwrap();
    }

    /// The two profiles this workspace defines, as the config writes
    /// them, both leaving `vc-x1`.
    const PROFILES: &str = "[profile.release]\ncargo = \"release\"\nbin = \"rel/vc-x1\"\n\
                            [profile.dev]\ncargo = \"dev\"\nbin = \"dbg/vc-x1\"\n\
                            suffix = \"-dev\"\n";

    /// The params for `install --profile <profile> --dir <dir>` in the
    /// workspace at `root`.
    fn resolve(root: &Path, profile: Option<&str>, dir: &Path) -> Result<InstallParams, String> {
        install_params(profile, Some(root), Some(dir), true)
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
            Cli::try_parse_from(["vc-x1", "install", "--profile", "dev", "--dir", "d"]).unwrap();
        match cli.command {
            Some(Commands::Install(a)) => {
                assert_eq!(a.profile.as_deref(), Some("dev"));
                assert_eq!(a.dir, Some(PathBuf::from("d")));
            }
            _ => panic!("expected Install"),
        }
        let cli = Cli::try_parse_from(["vc-x1", "install", "--dev", "--dir", "d"]).unwrap();
        match cli.command {
            Some(Commands::Install(a)) => assert_eq!(
                InstallParams::try_from(&a).unwrap_err(),
                "install: --dev is retired, the build is a profile: `install --profile dev`"
            ),
            _ => panic!("expected Install"),
        }
        let cli =
            Cli::try_parse_from(["vc-x1", "install-dev", "--dry-run", "--dir", "d", "-R", "x"])
                .unwrap();
        match cli.command {
            Some(Commands::InstallDev(a)) => {
                let err = InstallParams::try_from(&a).unwrap_err();
                assert!(err.ends_with("`install --profile dev`"), "{err}");
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

    /// The dev profile installs as `<dir>/vc-x1-dev`, its suffix
    /// appended, and the release under its own name, the list's
    /// placeholders offered resolved.
    #[test]
    fn the_profile_names_the_destination() {
        let dir = tempdir("dest");
        write_config(&dir, PROFILES);
        let bin = dir.join("bin");
        let p = resolve(&dir, Some("dev"), &bin).unwrap();
        assert_eq!(p.dest, bin.join("vc-x1-dev"));
        assert_eq!(p.bin, dir.join("dbg/vc-x1"));
        assert_eq!(p.bin_key, "profile.dev.bin");
        assert_eq!(p.build.profile.name, "dev");
        assert!(p.vars.contains(&(
            "install.dir".to_string(),
            bin.to_string_lossy().into_owned()
        )));
        assert!(
            p.vars
                .contains(&("profile.suffix".to_string(), "-dev".to_string()))
        );
        let p = resolve(&dir, None, &bin).unwrap();
        assert_eq!(p.dest, bin.join("vc-x1"));
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A copy whose destination is another profile's, the same file
    /// name and suffix in the same directory, is refused naming both,
    /// and one with a suffix, or with a list, is not.
    #[test]
    fn a_copy_onto_another_profiles_destination_is_refused() {
        let dir = tempdir("guard");
        let bin = dir.join("bin");
        write_config(
            &dir,
            "[profile.release]\nbin = \"rel/app\"\n[profile.dev]\nbin = \"dbg/app\"\n",
        );
        let e = resolve(&dir, Some("dev"), &bin).unwrap_err();
        assert!(e.starts_with("install: "), "{e}");
        assert!(
            e.contains("the `dev` profile's destination") && e.contains("the `release` profile's"),
            "{e}"
        );
        assert!(e.contains("`profile.dev.suffix`"), "{e}");
        let e = resolve(&dir, None, &bin).unwrap_err();
        assert!(e.contains("the `dev` profile's too"), "{e}");
        write_config(
            &dir,
            "[profile.release]\nbin = \"rel/app\"\n[profile.dev]\nbin = \"dbg/app\"\n\
             suffix = \"-dev\"\n",
        );
        resolve(&dir, Some("dev"), &bin).unwrap();
        resolve(&dir, None, &bin).unwrap();
        write_config(
            &dir,
            "[install]\ncommands = [\"true\"]\n\
             [profile.release]\nbin = \"rel/app\"\n[profile.dev]\nbin = \"dbg/app\"\n",
        );
        resolve(&dir, Some("dev"), &bin).unwrap();
        std::fs::remove_dir_all(&dir).ok();
    }

    /// An install list's placeholders are checked before the build,
    /// the profile's properties and `install.dir` offered.
    #[test]
    fn an_install_lists_placeholders_are_checked() {
        let dir = tempdir("listph");
        write_config(
            &dir,
            &format!(
                "[install]\ncommands = [\"cp {{profile.bin}} {{install.release-dir}}\"]\n{PROFILES}"
            ),
        );
        let e = resolve(&dir, None, &dir.join("bin")).unwrap_err();
        assert!(
            e.contains("`install.commands`: `{install.release-dir}` is not a placeholder"),
            "{e}"
        );
        assert!(e.contains("{install.dir}"), "{e}");
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
            r.contains("check `profile.release.bin`"),
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
            expand_dir(cfg, "install.dir", "~/.cargo/bin", Some(home)).unwrap(),
            PathBuf::from("/home/u/.cargo/bin")
        );
        assert_eq!(
            expand_dir(cfg, "install.dir", "~", Some(home)).unwrap(),
            PathBuf::from("/home/u")
        );
        assert_eq!(
            expand_dir(cfg, "install.dir", "/opt/bin", Some(home)).unwrap(),
            PathBuf::from("/opt/bin")
        );
        assert_eq!(
            expand_dir(cfg, "install.dir", "out/bin", Some(home)).unwrap(),
            PathBuf::from("/w/out/bin")
        );
        let e = expand_dir(cfg, "install.dir", "~/x", None).unwrap_err();
        assert!(e.contains("`install.dir`") && e.contains("HOME"), "{e}");
        assert_eq!(
            expand_dir(cfg, "install.dir", "~user/x", Some(home)).unwrap(),
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
            "```toml\n[repos]\nwork = \".\"\n[build]\ncommands = [\"true\"]\n\
             [profile.release]\nbin = \"app\"\n```\n",
        )
        .unwrap();
        let args = InstallArgs {
            profile: None,
            dev: false,
            dry_run: DryRunFlag { value: true },
            repo: Some(dir.clone()),
            dir: None,
        };
        let e = InstallParams::try_from(&args).unwrap_err();
        assert!(e.contains("nothing to install"), "{e}");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// `--dir` replaces the directory key for the run, and needs no
    /// key to do it.
    #[test]
    fn dir_overrides_the_install_dir() {
        let dir = tempdir("dirflag");
        let cfg = "```toml\n[repos]\nwork = \".\"\n";
        std::fs::write(dir.join(".vc-config.md"), format!("{cfg}```\n")).unwrap();
        let flag = Path::new("/scratch/bin");
        let key = INSTALL_DIR_KEY;
        assert_eq!(dest_dir(&dir, Some(flag), key).unwrap(), flag);
        let e = dest_dir(&dir, None, key).unwrap_err();
        assert!(e.contains("no `install.dir`"), "{e}");
        assert!(e.contains("nothing to install"), "{e}");
        std::fs::write(
            dir.join(".vc-config.md"),
            format!("{cfg}[install]\ndir = \"/opt/bin\"\n```\n"),
        )
        .unwrap();
        assert_eq!(dest_dir(&dir, Some(flag), key).unwrap(), flag, "--dir wins");
        assert_eq!(
            dest_dir(&dir, None, key).unwrap(),
            PathBuf::from("/opt/bin")
        );
        assert_eq!(
            dest_dir(&dir, Some(Path::new("tmp/bin")), key).unwrap(),
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
        p.list = Some(vec!["sh check.sh {profile.bin} {install.dir}".to_string()]);
        copy_reported(&p, &good()).unwrap();
        assert!(!dir.join("list-ran").exists(), "a dry run ran the list");
        p.dry_run = false;
        copy_reported(&p, &good()).unwrap();
        assert!(dir.join("list-ran").exists(), "the list did not run");
        assert!(!dest.exists(), "the list ran and the copy was made too");
        p.list = Some(vec!["false".to_string()]);
        let e = copy_reported(&p, &good()).unwrap_err().to_string();
        assert!(
            e.contains("`false` failed") && e.contains("install.commands"),
            "{e}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A list takes the profile's suffix as `{profile.suffix}`.
    #[test]
    fn a_dev_list_takes_the_suffix() {
        let dir = tempdir("devlist");
        let bin = dir.join("debug");
        std::fs::write(&bin, "the dev build").unwrap();
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        let mut p = params(&bin, &dir.join("bin/app.dbg"), false);
        p.root = dir.clone();
        for (name, value) in p.vars.iter_mut() {
            if name == "profile.suffix" {
                *value = ".dbg".to_string();
            }
        }
        p.list = Some(vec![
            "cp {profile.bin} {install.dir}/app{profile.suffix}".to_string(),
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
            "cp {profile.bin} {install.dir}/app{profile.suffix}.new".to_string(),
            "mv {install.dir}/app{profile.suffix}.new {install.dir}/app{profile.suffix}"
                .to_string(),
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

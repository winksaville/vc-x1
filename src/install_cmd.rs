//! The `install` subcommand: build the release when it is out of
//! date, then copy it into the install directory, the one command that
//! touches the PATH, after reporting what it is copying.
//!
//! - It runs `[build] release` first, exactly as `vc-x1 build` does.
//!   Cargo rebuilds only what changed, so with
//!   nothing changed the binary already tested is the one copied, and
//!   a missing or stale one is built rather than refused.
//! - The binary is `[build] release-bin`. The destination is the one directory `[install] dir` names, `~`
//!   expanded, so the committed value fits every machine, and the
//!   binary keeps its file name there. `$PATH` is not searched.
//! - The report says what is being copied: the version and commit the
//!   binary's `-V` shows, whether that commit is `main` and
//!   `main@origin`, its state, and a suffixed version. A binary whose
//!   `-V` carries no commit is reported as unknown. It refuses
//!   nothing: which builds
//!   may be installed is the release process's call, a cycle's Land
//!   here, and a project releasing from a branch or a tag makes it
//!   differently.
//! - It refuses only when the copy itself would be wrong, every reason
//!   listed: no binary at `release-bin` after the build, a binary whose
//!   `-V` will not run, or a missing install directory.
//! - The copy is a temporary file beside the destination renamed into
//!   place, so the binary installed is the one tested, a running copy
//!   of the old one is undisturbed, and a failed copy leaves the old
//!   one.
//! - `--dry-run` builds and reports, and copies nothing.
//! - A workspace with no `[install] dir` has nothing to install, and
//!   install says so before building anything.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info};

use crate::build_cmd::{BuildParams, build_params, release_bin};
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
}

/// What the release binary's `-V` says: its version and, from the
/// build metadata after `+`, its commit and the state after it.
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
    /// True when the release binary is there.
    pub bin_exists: bool,
    /// What the release binary's `-V` said, `None` when it could not be
    /// run or read.
    pub reported: Option<Reported>,
    /// True when the destination directory exists.
    pub dest_dir_exists: bool,
}

/// Per-invocation install inputs: the release build to run first,
/// where its binary is and where it goes, and whether to copy.
#[derive(Debug)]
pub struct InstallParams {
    /// The work repo root.
    pub root: PathBuf,
    /// The release build, `vc-x1 build`'s, run before anything else.
    pub release: BuildParams,
    /// The release binary, `[build] release-bin`.
    pub bin: PathBuf,
    /// The destination, `[install] dir` joined with the binary's name.
    pub dest: PathBuf,
    /// Build and report, and copy nothing.
    pub dry_run: bool,
}

impl TryFrom<&InstallArgs> for InstallParams {
    type Error = String;

    /// Resolve the workspace, the install directory, and the release
    /// build. A missing config key is an error here, before the build
    /// runs, and every fact about the binary is gathered after it.
    fn try_from(a: &InstallArgs) -> Result<Self, String> {
        let root = match &a.repo {
            Some(p) => crate::common::find_workspace_root_from(p)
                .ok_or_else(|| format!("install: '{}' is not in a vc-x1 workspace", p.display()))?,
            None => crate::common::find_workspace_root()
                .ok_or_else(|| "install: not in a vc-x1 workspace (no config found)".to_string())?,
        };
        let dest_dir = install_dir(&root)?;
        let release = build_params(false, Some(&root))?;
        let bin = release_bin(&root)?;
        let name = bin
            .file_name()
            .ok_or_else(|| format!("install: '{}' names no file", bin.display()))?;
        let dest = dest_dir.join(name);
        Ok(InstallParams {
            root,
            release,
            bin,
            dest,
            dry_run: a.dry_run.value,
        })
    }
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

/// `[install] dir` at `root`: `~` and `~/...` against the home
/// directory, any other relative value against the config file's
/// directory, an absolute one as is. Its absence is the workspace
/// having nothing to install.
pub fn install_dir(root: &Path) -> Result<PathBuf, String> {
    let cfg = crate::config_md::load(root)
        .map_err(|e| format!("install: {e}"))?
        .ok_or_else(|| format!("install: no config in '{}'", root.display()))?;
    let value = crate::toml_simple::toml_get(&cfg.map, "install.dir").ok_or_else(|| {
        format!(
            "install: {}: no `install.dir`, so this workspace has nothing to install. Name the \
             directory the release build is copied into, such as `~/.cargo/bin` (see `vc-x1 \
             config work`)",
            cfg.path.display()
        )
    })?;
    // OK: a loaded config's path is a file, so it has a parent
    let cfg_dir = cfg.path.parent().unwrap_or(root);
    expand_dir(cfg_dir, value, crate::jj::session::home_dir().as_deref())
}

/// Expand an `[install] dir` value: `~` against `home`, else against
/// `cfg_dir` when relative.
fn expand_dir(cfg_dir: &Path, value: &str, home: Option<&Path>) -> Result<PathBuf, String> {
    let rest = if value == "~" {
        Some("")
    } else {
        value.strip_prefix("~/")
    };
    match rest {
        Some(rest) => home
            .map(|h| h.join(rest))
            .ok_or_else(|| format!("install: `install.dir` is '{value}' and HOME is not set")),
        None => Ok(crate::common::resolve_repo_path(cfg_dir, value)),
    }
}

/// The manifest's version, `[package] version` in the `Cargo.toml` at
/// `root`.
pub fn manifest_version(root: &Path) -> Result<String, String> {
    let path = root.join("Cargo.toml");
    let map = crate::toml_simple::toml_load(&path)
        .map_err(|e| format!("install: cannot read '{}': {e}", path.display()))?;
    crate::toml_simple::toml_get(&map, "package.version")
        .cloned()
        .ok_or_else(|| format!("install: '{}' has no `[package] version`", path.display()))
}

/// Run `bin -V` and return what it prints, `None` when it cannot be
/// run or fails.
fn run_version(bin: &Path) -> Option<String> {
    // Register entry 7 (clippy.toml): install runs the release binary
    // itself, `-V` alone, to report the binary it is about to copy.
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

/// Every reason the copy itself would be wrong, empty when it may go.
pub fn refusals(f: &Facts, bin: &Path, dest: &Path) -> Vec<String> {
    let mut out = Vec::new();
    if !f.bin_exists {
        out.push(format!(
            "no binary at '{}': build it with `vc-x1 build`",
            bin.display()
        ));
    } else if f.reported.is_none() {
        out.push(format!("'{}' -V could not be run or read", bin.display()));
    }
    if !f.dest_dir_exists {
        // OK: dest is a directory joined with a file name, so it has a parent
        let dir = dest.parent().unwrap_or(dest);
        out.push(format!(
            "the `install.dir` directory '{}' does not exist",
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
    // OK: dest is joined with the release binary's file name, so it has one
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = dir.join(format!(".{name}.vc-x1-install"));
    let result = std::fs::copy(bin, &tmp)
        .map_err(|e| format!("install: cannot copy to '{}': {e}", tmp.display()))
        .and_then(|_| {
            std::fs::rename(&tmp, dest).map_err(|e| {
                format!(
                    "install: cannot move '{}' into '{}': {e}",
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

/// Run the release build, as `vc-x1 build` does, then report the
/// binary and copy it unless refused or a dry run. Cargo rebuilds only
/// what changed, so with nothing changed the binary already tested is
/// the one copied.
pub fn install(params: &InstallParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("install: entry params={params:?}");
    crate::build_cmd::build(&params.release)?;
    let facts = gather(&params.root, &params.bin, &params.dest)?;
    copy_reported(&params.bin, &params.dest, &facts, params.dry_run)
}

/// Print the report, refuse with every reason at once when the copy
/// would be wrong, then copy `bin` to `dest` unless it is a dry run.
pub fn copy_reported(
    bin: &Path,
    dest: &Path,
    facts: &Facts,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("install: '{}' -> '{}'", bin.display(), dest.display());
    for line in report(facts) {
        info!("install:   {line}");
    }
    let reasons = refusals(facts, bin, dest);
    if !reasons.is_empty() {
        let list: String = reasons.iter().map(|r| format!("\n  - {r}")).collect();
        return Err(format!(
            "install: refused, nothing copied to '{}':{list}",
            dest.display()
        )
        .into());
    }
    if dry_run {
        info!("install: [dry-run] copied nothing");
        return Ok(());
    }
    deploy(bin, dest)?;
    info!("install: copied");
    Ok(())
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
        )
        .join("\n")
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
            }
            _ => panic!("expected Install"),
        }
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
            expand_dir(cfg, "~/.cargo/bin", Some(home)).unwrap(),
            PathBuf::from("/home/u/.cargo/bin")
        );
        assert_eq!(
            expand_dir(cfg, "~", Some(home)).unwrap(),
            PathBuf::from("/home/u")
        );
        assert_eq!(
            expand_dir(cfg, "/opt/bin", Some(home)).unwrap(),
            PathBuf::from("/opt/bin")
        );
        assert_eq!(
            expand_dir(cfg, "out/bin", Some(home)).unwrap(),
            PathBuf::from("/w/out/bin")
        );
        assert!(expand_dir(cfg, "~/x", None).unwrap_err().contains("HOME"));
        assert_eq!(
            expand_dir(cfg, "~user/x", Some(home)).unwrap(),
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
        copy_reported(&bin, &dest, &facts, true).unwrap();
        assert!(!dest.exists(), "a dry run copied");
        copy_reported(&bin, &dest, &facts, false).unwrap();
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "the release");
        std::fs::remove_file(&dest).unwrap();
        facts.reported = None;
        facts.dest_dir_exists = false;
        let e = copy_reported(&bin, &dest, &facts, false)
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
        };
        let e = InstallParams::try_from(&args).unwrap_err();
        assert!(e.contains("nothing to install"), "{e}");
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

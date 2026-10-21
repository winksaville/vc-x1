//! The `build` and `build-dev` subcommands: build the workspace's
//! release or its dev build, each by its configured command list. A
//! command per build, named with the `-dev` suffix as `run` and
//! `run-dev` are, so each names the build it makes and no dropped flag
//! makes the other. Neither touches the PATH: that is
//! `vc-x1 install`'s, which copies the release build there.
//!
//! - `vc-x1 build` runs `[build] release`, here
//!   `cargo build --release --locked`, and writes a stamp beside
//!   `[build] release-bin` naming the commit `@-` the build was made
//!   from and the state of the source. The lists leave the binaries in
//!   `target/`, off the PATH, so nothing there is replaced: the
//!   release build is run by `vc-x1 run` and copied by
//!   `vc-x1 install`, which runs this same build first, through
//!   [`build_params`] and [`build`], and reports what the stamp says.
//! - `vc-x1 build-dev` runs `[build] dev`, the dev build, and writes
//!   no stamp, since it is never installed.
//! - Where a build leaves its binary is the config's to say,
//!   `release-bin` and `dev-bin`: vc-x1 never works it out from cargo
//!   or the package's name.
//! - Both run their commands with the build's [`Provenance`] in the
//!   environment, `VC_X1_BUILD_COMMIT` and `VC_X1_BUILD_STATE`, as
//!   `validate` does, so every build of a run sees the same values
//!   and none recompiles for another. An artifact that compiles them
//!   in can say what it was built from wherever it is copied. vc-x1
//!   does, as semver build metadata in its banner,
//!   `0.85.7+d70cf9ecdfd9`, `.dirty` for a build with edits.
//! - Neither checks the repo's state: a release may be built from a
//!   cycle's bookmark, to try the release profile mid-cycle.
//! - The stamp is removed before the build and written after it
//!   passes, so a failed build never leaves a stamp describing the
//!   binary it failed to replace.
//! - The stamp also records the binary's size and modification time,
//!   its [`BinaryId`], so `install` can tell the binary `build` made
//!   from one rebuilt by hand or replaced since, for any artifact,
//!   whether or not it reports its commit.
//! - The lists are run by [`crate::command_list`], as `validate`'s
//!   are.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info, warn};

use crate::context::Context;
use crate::subcommand::SubcommandRunner;
use crate::version::SourceState;

/// The stamp's suffix, appended to the release binary's path.
pub const STAMP_SUFFIX: &str = ".stamp";

/// The variable carrying the build's commit to its commands.
pub const BUILD_COMMIT_VAR: &str = "VC_X1_BUILD_COMMIT";

/// The variable carrying `state`, the state of the source code being
/// compiled, `clean` or `dirty`.
pub const BUILD_STATE_VAR: &str = "VC_X1_BUILD_STATE";

/// Build the release.
#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Workspace to build from [default: the workspace containing
    /// the current directory]
    #[arg(short = 'R', long, value_name = "DIR")]
    pub repo: Option<PathBuf>,
}

/// Build the dev build.
#[derive(Args, Debug)]
pub struct BuildDevArgs {
    /// Workspace to build from [default: the workspace containing
    /// the current directory]
    #[arg(short = 'R', long, value_name = "DIR")]
    pub repo: Option<PathBuf>,
}

/// What a build is made from: the work repo's `@-` and the state of
/// its source, read before the build runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    /// The full commit id of the work repo's `@-`.
    pub commit: String,
    /// The state of the source code being compiled: clean when the
    /// work repo's `@` was empty, so the compiled code is exactly
    /// `commit`'s tree, else dirty. Never unknown, since it is read
    /// through jj.
    pub state: SourceState,
}

impl Provenance {
    /// Read the provenance of the work repo at `root`, `label`
    /// prefixing an error.
    pub fn read(root: &Path, label: &str) -> Result<Self, String> {
        let commit = crate::jj::cid_of(root, "@-")
            .map_err(|e| format!("{label}: the commit `@-` in '{}': {e}", root.display()))?;
        let empty = crate::jj::is_empty(root, "@")
            .map_err(|e| format!("{label}: whether `@` is empty in '{}': {e}", root.display()))?;
        let state = if empty {
            SourceState::Clean
        } else {
            SourceState::Dirty
        };
        Ok(Provenance { commit, state })
    }

    /// Read it, or warn and carry none, for a build that runs
    /// without it: the dev build and `validate`'s.
    pub fn read_or_warn(root: &Path, label: &str) -> Option<Self> {
        match Self::read(root, label) {
            Ok(p) => Some(p),
            Err(e) => {
                warn!("{e}, so the build carries no commit");
                None
            }
        }
    }

    /// The two variables the build's commands get.
    pub fn env(&self) -> [(&'static str, &str); 2] {
        [
            (BUILD_COMMIT_VAR, self.commit.as_str()),
            (BUILD_STATE_VAR, self.state.as_str()),
        ]
    }
}

/// The release binary's identity as the stamp records it: its size in
/// bytes and its modification time, nanoseconds since the Unix epoch.
/// A rebuild or a copy over it changes the time, and almost always
/// the size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinaryId {
    /// The file's length in bytes.
    pub size: u64,
    /// The file's modification time, nanoseconds since the epoch.
    pub mtime_ns: u128,
}

impl BinaryId {
    /// Read the identity of the file at `bin`.
    pub fn of(bin: &Path) -> Result<Self, String> {
        let meta = std::fs::metadata(bin)
            .map_err(|e| format!("build: cannot read '{}': {e}", bin.display()))?;
        let mtime_ns = meta
            .modified()
            .map_err(|e| format!("build: no modification time for '{}': {e}", bin.display()))?
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("build: '{}' is dated before 1970: {e}", bin.display()))?
            .as_nanos();
        Ok(BinaryId {
            size: meta.len(),
            mtime_ns,
        })
    }
}

/// The variables for `provenance`, none when it is absent.
pub fn build_env(provenance: Option<&Provenance>) -> Vec<(&'static str, &str)> {
    // OK: no provenance is no variables, the build carrying no commit
    provenance.map(|p| p.env().to_vec()).unwrap_or_default()
}

/// Where a release's stamp goes and the binary it describes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stamp {
    /// The stamp file, `[build] release-bin` plus [`STAMP_SUFFIX`].
    pub path: PathBuf,
    /// The binary the stamp describes, `[build] release-bin`.
    pub bin: PathBuf,
}

/// Per-invocation build inputs: the command's label, the list's key,
/// its commands, the directory to run them from, the build's
/// provenance, and the stamp a release writes.
#[derive(Debug)]
pub struct BuildParams {
    /// `build` or `build-dev`, prefixing the run's messages.
    pub label: &'static str,
    /// `build.release` or `build.dev`.
    pub key: &'static str,
    /// The commands, one element per invocation.
    pub commands: Vec<String>,
    /// The work repo root the commands run from.
    pub root: PathBuf,
    /// The build's provenance, required for a release, best effort
    /// for the dev build.
    pub provenance: Option<Provenance>,
    /// The release's stamp, `None` for the dev build.
    pub stamp: Option<Stamp>,
}

impl TryFrom<&BuildArgs> for BuildParams {
    type Error = String;

    /// The release: see [`build_params`].
    fn try_from(a: &BuildArgs) -> Result<Self, String> {
        build_params(false, a.repo.as_deref())
    }
}

impl TryFrom<&BuildDevArgs> for BuildParams {
    type Error = String;

    /// The dev build: see [`build_params`].
    fn try_from(a: &BuildDevArgs) -> Result<Self, String> {
        build_params(true, a.repo.as_deref())
    }
}

/// Resolve the workspace root (from `-R` or the cwd), take the chosen
/// list, read the provenance, and for a release resolve `release-bin`.
/// A release's every miss is an error here, before anything runs, and
/// the dev build's provenance is best effort.
pub fn build_params(dev: bool, repo: Option<&Path>) -> Result<BuildParams, String> {
    let (label, key) = if dev {
        ("build-dev", "build.dev")
    } else {
        ("build", "build.release")
    };
    let root = match repo {
        Some(p) => crate::common::find_workspace_root_from(p)
            .ok_or_else(|| format!("{label}: '{}' is not in a vc-x1 workspace", p.display()))?,
        None => crate::common::find_workspace_root()
            .ok_or_else(|| format!("{label}: not in a vc-x1 workspace (no config found)"))?,
    };
    let commands = crate::command_list::load(&root, key, label)?;
    let (provenance, stamp) = if dev {
        (Provenance::read_or_warn(&root, label), None)
    } else {
        let bin = release_bin(&root)?;
        let provenance = Provenance::read(&root, label)?;
        let stamp = Stamp {
            path: stamp_path(&bin),
            bin,
        };
        (Some(provenance), Some(stamp))
    };
    Ok(BuildParams {
        label,
        key,
        commands,
        root,
        provenance,
        stamp,
    })
}

/// `[build] release-bin` at `root`, resolved against the config
/// file's directory, or as is when absolute. Required for a release,
/// since the stamp sits beside it.
pub fn release_bin(root: &Path) -> Result<PathBuf, String> {
    let cfg = crate::config_md::load(root)
        .map_err(|e| format!("build: {e}"))?
        .ok_or_else(|| format!("build: no config in '{}'", root.display()))?;
    let value = crate::toml_simple::toml_get(&cfg.map, "build.release-bin").ok_or_else(|| {
        format!(
            "build: {}: no `build.release-bin`, the binary `build.release` leaves, \
             which the stamp is written beside (see `vc-x1 config work`)",
            cfg.path.display()
        )
    })?;
    // OK: a loaded config's path is a file, so it has a parent
    let cfg_dir = cfg.path.parent().unwrap_or(root);
    Ok(crate::common::resolve_repo_path(cfg_dir, value))
}

/// The stamp file for the binary at `bin`.
pub fn stamp_path(bin: &Path) -> PathBuf {
    let mut s = bin.as_os_str().to_os_string();
    s.push(STAMP_SUFFIX);
    PathBuf::from(s)
}

/// The stamp's text: a comment saying what it is, then `commit`,
/// `state`, `size`, and `mtime-ns`, keys `toml_simple` reads back.
pub fn render_stamp(provenance: &Provenance, id: BinaryId) -> String {
    format!(
        "# vc-x1 build stamp: the commit the binary beside it was built from\n\
         commit = \"{}\"\n\
         state = \"{}\"\n\
         size = {}\n\
         mtime-ns = {}\n",
        provenance.commit,
        provenance.state.as_str(),
        id.size,
        id.mtime_ns
    )
}

impl SubcommandRunner for BuildArgs {
    type Params = BuildParams;

    /// Delegate to the `TryFrom<&BuildArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        BuildParams::try_from(self)
    }

    /// Run the `build` op (`ctx` unused: the op is fully
    /// parameterized by `Params`).
    fn run(_ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        build(params)
    }
}

impl SubcommandRunner for BuildDevArgs {
    type Params = BuildParams;

    /// Delegate to the `TryFrom<&BuildDevArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        BuildParams::try_from(self)
    }

    /// Run the same op as `build`, its params naming the dev build.
    fn run(_ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        build(params)
    }
}

/// What a release build did to its binary, from its identity before
/// and after the list ran: `up to date` when the list left it as it
/// was, as cargo does when nothing changed, `built` otherwise.
pub fn outcome(before: Option<BinaryId>, after: BinaryId) -> &'static str {
    if before == Some(after) {
        "up to date"
    } else {
        "built"
    }
}

/// Run the list from the root with the provenance in its
/// environment, and for a release remove the old stamp first and
/// write the new one once the list passes, saying whether the binary
/// was built or already up to date.
pub fn build(params: &BuildParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("{}: entry params={params:?}", params.label);
    let before = params
        .stamp
        .as_ref()
        .and_then(|stamp| BinaryId::of(&stamp.bin).ok());
    if let Some(stamp) = &params.stamp {
        match std::fs::remove_file(&stamp.path) {
            Ok(()) => debug!("build: removed the old stamp {}", stamp.path.display()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(format!(
                    "build: cannot remove the old stamp '{}': {e}",
                    stamp.path.display()
                )
                .into());
            }
        }
    }
    let env = build_env(params.provenance.as_ref());
    crate::command_list::run(
        &params.root,
        params.key,
        &params.commands,
        &env,
        params.label,
    )?;
    let (Some(stamp), Some(provenance)) = (&params.stamp, &params.provenance) else {
        return Ok(());
    };
    if !stamp.bin.is_file() {
        return Err(format!(
            "build: `{}` passed but left no binary at '{}', so no stamp is written: \
             check `build.release-bin`",
            params.key,
            stamp.bin.display()
        )
        .into());
    }
    let id = BinaryId::of(&stamp.bin)?;
    std::fs::write(&stamp.path, render_stamp(provenance, id)).map_err(|e| {
        format!(
            "build: cannot write the stamp '{}': {e}",
            stamp.path.display()
        )
    })?;
    let short = &provenance.commit[..provenance.commit.len().min(12)];
    let state = match provenance.state {
        SourceState::Clean => "state clean, `@` empty",
        _ => "state dirty, `@` held edits, so the build is not exactly that commit",
    };
    info!(
        "build: {} {}, from {short}, {state}",
        stamp.bin.display(),
        outcome(before, id)
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cli, Commands};
    use clap::Parser;

    fn parse(args: &[&str]) -> BuildArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::Build(a)) => a,
            _ => panic!("expected Build"),
        }
    }

    fn parse_dev(args: &[&str]) -> BuildDevArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::BuildDev(a)) => a,
            _ => panic!("expected BuildDev"),
        }
    }

    /// Unique tempdir for a one-file workspace.
    fn workspace(tag: &str, config: &str) -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("vc-x1-build-{tag}-{ts}"));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(".vc-config.md"), config).unwrap();
        root
    }

    const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

    fn prov(state: SourceState) -> Provenance {
        Provenance {
            commit: COMMIT.to_string(),
            state,
        }
    }

    fn stamp_in(root: &Path) -> Stamp {
        let bin = root.join("rel/bin/app");
        Stamp {
            path: stamp_path(&bin),
            bin,
        }
    }

    fn release(root: &Path, commands: Vec<String>) -> BuildParams {
        BuildParams {
            label: "build",
            key: "build.release",
            commands,
            root: root.to_path_buf(),
            provenance: Some(prov(SourceState::Clean)),
            stamp: Some(stamp_in(root)),
        }
    }

    #[test]
    fn flags_parse() {
        assert_eq!(parse(&["vc-x1", "build"]).repo, None);
        assert_eq!(
            parse(&["vc-x1", "build", "-R", "x"]).repo,
            Some(PathBuf::from("x"))
        );
        assert_eq!(
            parse_dev(&["vc-x1", "build-dev", "-R", "x"]).repo,
            Some(PathBuf::from("x"))
        );
        assert!(Cli::try_parse_from(["vc-x1", "build", "--dev"]).is_err());
    }

    /// The stamp sits beside the binary, the suffix appended.
    #[test]
    fn stamp_path_appends_the_suffix() {
        assert_eq!(
            stamp_path(Path::new("tmp/rel/bin/vc-x1")),
            PathBuf::from("tmp/rel/bin/vc-x1.stamp")
        );
    }

    /// The stamp's keys read back through toml_simple, the reader
    /// `install` uses.
    #[test]
    fn stamp_reads_back() {
        let id = BinaryId {
            size: 21394336,
            mtime_ns: 1_761_064_259_123_456_789,
        };
        let map = crate::toml_simple::toml_parse(&render_stamp(&prov(SourceState::Dirty), id));
        assert_eq!(
            crate::toml_simple::toml_get(&map, "size").map(String::as_str),
            Some("21394336")
        );
        assert_eq!(
            crate::toml_simple::toml_get(&map, "mtime-ns").map(String::as_str),
            Some("1761064259123456789")
        );
        assert_eq!(
            crate::toml_simple::toml_get(&map, "commit").map(String::as_str),
            Some(COMMIT)
        );
        assert_eq!(
            crate::toml_simple::toml_get(&map, "state").map(String::as_str),
            Some("dirty")
        );
    }

    /// The variables carry the commit and the state's word, and none
    /// without a provenance.
    #[test]
    fn build_env_forms() {
        assert_eq!(
            build_env(Some(&prov(SourceState::Clean))),
            vec![(BUILD_COMMIT_VAR, COMMIT), (BUILD_STATE_VAR, "clean")]
        );
        assert!(build_env(None).is_empty());
    }

    /// A release without `release-bin` is refused before anything
    /// runs, naming the key.
    #[test]
    fn release_needs_release_bin() {
        let root = workspace(
            "nobin",
            "```toml\n[repos]\nwork = \".\"\n[build]\nrelease = [\"true\"]\n```\n",
        );
        let e = release_bin(&root).unwrap_err();
        assert!(e.contains("no `build.release-bin`"), "{e}");
        std::fs::remove_dir_all(&root).ok();
    }

    /// `release-bin` resolves against the config file's directory,
    /// or as is when absolute.
    #[test]
    fn release_bin_resolves_against_the_config() {
        let root = workspace(
            "rel",
            "```toml\n[repos]\nwork = \".\"\n[build]\nrelease-bin = \"rel/bin/app\"\n```\n",
        );
        assert_eq!(release_bin(&root).unwrap(), root.join("rel/bin/app"));
        let root2 = workspace(
            "abs",
            "```toml\n[repos]\nwork = \".\"\n[build]\nrelease-bin = \"/opt/app\"\n```\n",
        );
        assert_eq!(release_bin(&root2).unwrap(), PathBuf::from("/opt/app"));
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&root2).ok();
    }

    /// A passing release writes the stamp beside the binary it left,
    /// replacing an old one.
    #[test]
    fn release_writes_the_stamp() {
        let root = workspace("stamp", "");
        let params = release(&root, vec!["touch rel/bin/app".to_string()]);
        let stamp = params.stamp.clone().unwrap();
        std::fs::create_dir_all(stamp.bin.parent().unwrap()).unwrap();
        std::fs::write(&stamp.path, "old").unwrap();
        build(&params).unwrap();
        let id = BinaryId::of(&stamp.bin).unwrap();
        assert_eq!(
            std::fs::read_to_string(&stamp.path).unwrap(),
            render_stamp(&prov(SourceState::Clean), id)
        );
        std::fs::remove_dir_all(&root).ok();
    }

    /// A binary rewritten after the build no longer has the identity
    /// its stamp recorded.
    #[test]
    fn a_rebuilt_binary_has_another_identity() {
        let root = workspace("rebuilt", "");
        let bin = root.join("app");
        std::fs::write(&bin, "the release build").unwrap();
        let built = BinaryId::of(&bin).unwrap();
        assert_eq!(BinaryId::of(&bin).unwrap(), built);
        std::thread::sleep(std::time::Duration::from_millis(10));
        std::fs::write(&bin, "a build made by hand afterward").unwrap();
        let rebuilt = BinaryId::of(&bin).unwrap();
        assert_ne!(rebuilt.size, built.size);
        assert_ne!(rebuilt.mtime_ns, built.mtime_ns);
        std::fs::remove_dir_all(&root).ok();
    }

    /// The binary is up to date when the list left its identity as it
    /// was, and built when it changed or was not there before.
    #[test]
    fn outcome_tells_up_to_date_from_built() {
        let id = BinaryId {
            size: 10,
            mtime_ns: 20,
        };
        let other = BinaryId {
            size: 10,
            mtime_ns: 21,
        };
        assert_eq!(outcome(Some(id), id), "up to date");
        assert_eq!(outcome(Some(other), id), "built");
        assert_eq!(outcome(None, id), "built");
    }

    /// A missing binary has no identity, an error naming the path.
    #[test]
    fn a_missing_binary_has_no_identity() {
        let e = BinaryId::of(Path::new("/nonexistent/vc-x1-test-bin")).unwrap_err();
        assert!(e.contains("/nonexistent/vc-x1-test-bin"), "{e}");
    }

    /// The commands see the provenance's values. Checked by value, by
    /// a script, so a variable inherited from a surrounding run (a
    /// `validate` that sets the same names) cannot pass for it.
    #[test]
    fn the_commands_see_the_provenance() {
        let root = workspace("env", "");
        std::fs::write(
            root.join("check.sh"),
            format!(
                "[ \"${BUILD_COMMIT_VAR}\" = \"{COMMIT}\" ] && [ \"${BUILD_STATE_VAR}\" = clean ]\n"
            ),
        )
        .unwrap();
        std::fs::create_dir_all(root.join("rel/bin")).unwrap();
        let params = release(
            &root,
            vec!["sh check.sh".to_string(), "touch rel/bin/app".to_string()],
        );
        build(&params).unwrap();
        let dev = BuildParams {
            label: "build-dev",
            key: "build.dev",
            commands: vec!["sh check.sh".to_string()],
            root: root.clone(),
            provenance: Some(prov(SourceState::Clean)),
            stamp: None,
        };
        build(&dev).unwrap();
        std::fs::remove_dir_all(&root).ok();
    }

    /// A failed release removes the old stamp and writes none, so no
    /// stamp describes the binary the build failed to replace.
    #[test]
    fn failed_release_leaves_no_stamp() {
        let root = workspace("fail", "");
        let params = release(&root, vec!["false".to_string()]);
        let stamp = params.stamp.clone().unwrap();
        std::fs::create_dir_all(stamp.bin.parent().unwrap()).unwrap();
        std::fs::write(&stamp.bin, "the previous build").unwrap();
        std::fs::write(&stamp.path, "old").unwrap();
        let e = build(&params).unwrap_err().to_string();
        assert!(e.contains("`false` failed"), "{e}");
        assert!(
            !stamp.path.exists(),
            "the old stamp survived a failed build"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    /// A release whose list passes but leaves no binary at
    /// `release-bin` is an error, and no stamp is written.
    #[test]
    fn release_without_its_binary_errors() {
        let root = workspace("nobinary", "");
        let params = release(&root, vec!["true".to_string()]);
        let stamp = params.stamp.clone().unwrap();
        let e = build(&params).unwrap_err().to_string();
        assert!(e.contains("left no binary"), "{e}");
        assert!(!stamp.path.exists());
        std::fs::remove_dir_all(&root).ok();
    }

    /// The dev build runs its list and writes no stamp.
    #[test]
    fn dev_writes_no_stamp() {
        let root = workspace("dev", "");
        let params = BuildParams {
            label: "build-dev",
            key: "build.dev",
            commands: vec!["touch dev-ran".to_string()],
            root: root.clone(),
            provenance: None,
            stamp: None,
        };
        build(&params).unwrap();
        assert!(root.join("dev-ran").exists());
        let stamps: Vec<_> = std::fs::read_dir(&root)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(STAMP_SUFFIX))
            .collect();
        assert!(stamps.is_empty());
        std::fs::remove_dir_all(&root).ok();
    }
}

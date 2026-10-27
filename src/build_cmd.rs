//! The `build` subcommand: build the workspace's release, or with
//! `--dev` its dev build, each by its configured command list. The
//! dev build was `build-dev`, a command of its own, until every
//! variant became a flag, and `build-dev` is kept hidden to be
//! refused naming `build --dev`. Neither build touches the PATH: that
//! is `vc-x1 install`'s, which copies them there.
//!
//! - `vc-x1 build` runs `[build] release`, here
//!   `cargo build --release --locked`, and checks it left its binary
//!   at `[build] release-bin`. The lists leave the binaries in
//!   `target/`, off the PATH, so nothing there is replaced: the
//!   release build is run by `vc-x1 run` and copied by
//!   `vc-x1 install`, which runs this same build first, through
//!   [`build_params`] and [`build`].
//! - `vc-x1 build --dev` runs `[build] dev`, the dev build, which
//!   `vc-x1 install --dev` runs first and copies.
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
//! - Neither tracks whether its binary is out of date: cargo does,
//!   rebuilding only what changed, so vc-x1 runs the list every time.
//! - The lists are run by [`crate::command_list`], as `validate`'s
//!   are.
//! - `--dry-run` prints the list and runs nothing, as `run` prints
//!   the build it would run first.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info, warn};

use crate::context::Context;
use crate::options_flags::dry_run::DryRunFlag;
use crate::subcommand::SubcommandRunner;
use crate::version::SourceState;

/// The variable carrying the build's commit to its commands.
pub const BUILD_COMMIT_VAR: &str = "VC_X1_BUILD_COMMIT";

/// The variable carrying `state`, the state of the source code being
/// compiled, `clean` or `dirty`.
pub const BUILD_STATE_VAR: &str = "VC_X1_BUILD_STATE";

/// Build the release, or with `--dev` the dev build.
#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Build the dev build, `[build] dev`, instead of the release
    #[arg(short = 'd', long)]
    pub dev: bool,

    #[command(flatten)]
    pub dry_run: DryRunFlag,

    /// Workspace to build from [default: the workspace containing
    /// the current directory]
    #[arg(short = 'R', long, value_name = "DIR")]
    pub repo: Option<PathBuf>,
}

/// Retired: `build-dev`, now `build --dev`, refused naming it. Its
/// flags are kept so an old invocation parses and is told the fix.
#[derive(Args, Debug)]
pub struct BuildDevArgs {
    #[command(flatten)]
    pub dry_run: DryRunFlag,

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

/// The variables for `provenance`, none when it is absent.
pub fn build_env(provenance: Option<&Provenance>) -> Vec<(&'static str, &str)> {
    // OK: no provenance is no variables, the build carrying no commit
    provenance.map(|p| p.env().to_vec()).unwrap_or_default()
}

/// Per-invocation build inputs: the command's label, the list's key,
/// its commands, the directory to run them from, the build's
/// provenance, and the binary a release leaves.
#[derive(Debug)]
pub struct BuildParams {
    /// `build` or `build --dev`, prefixing the run's messages.
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
    /// The binary a release leaves, `[build] release-bin`, `None` for
    /// the dev build.
    pub bin: Option<PathBuf>,
    /// Print the list and run nothing.
    pub dry_run: bool,
}

impl TryFrom<&BuildArgs> for BuildParams {
    type Error = String;

    /// The release, or the dev build with `--dev`: see [`build_params`].
    fn try_from(a: &BuildArgs) -> Result<Self, String> {
        let mut params = build_params(a.dev, a.repo.as_deref())?;
        params.dry_run = a.dry_run.value;
        Ok(params)
    }
}

impl TryFrom<&BuildDevArgs> for BuildParams {
    type Error = String;

    /// Refused: `build-dev` is `build --dev`.
    fn try_from(_: &BuildDevArgs) -> Result<Self, String> {
        Err(retired_dev("build"))
    }
}

/// The refusal for a retired `<command>-dev`, naming `<command> --dev`.
pub fn retired_dev(command: &str) -> String {
    format!(
        "{command}-dev: the -dev commands are retired, the dev build is a flag: \
         `{command} --dev`"
    )
}

/// Resolve the workspace root (from `-R` or the cwd), take the chosen
/// list, read the provenance, and for a release resolve `release-bin`.
/// A release's every miss is an error here, before anything runs, and
/// the dev build's provenance is best effort.
pub fn build_params(dev: bool, repo: Option<&Path>) -> Result<BuildParams, String> {
    let (label, key) = if dev {
        ("build --dev", "build.dev")
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
    let (provenance, bin) = if dev {
        (Provenance::read_or_warn(&root, label), None)
    } else {
        let bin = release_bin(&root)?;
        (Some(Provenance::read(&root, label)?), Some(bin))
    };
    Ok(BuildParams {
        label,
        key,
        commands,
        root,
        provenance,
        bin,
        dry_run: false,
    })
}

/// `[build] release-bin` at `root`: see [`bin_at`]. Required for a
/// release, whose build is checked for it and which `install` copies.
pub fn release_bin(root: &Path) -> Result<PathBuf, String> {
    bin_at(
        root,
        "build.release-bin",
        "the binary `build.release` leaves, which `install` copies",
    )
}

/// `[build] dev-bin` at `root`: see [`bin_at`]. Required by
/// `install-dev`, which copies it.
pub fn dev_bin(root: &Path) -> Result<PathBuf, String> {
    bin_at(
        root,
        "build.dev-bin",
        "the binary `build.dev` leaves, which `install-dev` copies",
    )
}

/// The binary `key` names at `root`, resolved against the config
/// file's directory, or as is when absolute, `what` saying in its
/// absence what the key is for.
fn bin_at(root: &Path, key: &str, what: &str) -> Result<PathBuf, String> {
    let cfg = crate::config_md::load(root)
        .map_err(|e| format!("build: {e}"))?
        .ok_or_else(|| format!("build: no config in '{}'", root.display()))?;
    let value = crate::toml_simple::toml_get(&cfg.map, key).ok_or_else(|| {
        format!(
            "build: {}: no `{key}`, {what} (see `vc-x1 config -s work`)",
            cfg.path.display()
        )
    })?;
    // OK: a loaded config's path is a file, so it has a parent
    let cfg_dir = cfg.path.parent().unwrap_or(root);
    Ok(crate::common::resolve_repo_path(cfg_dir, value))
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

/// Run the list from the root with the provenance in its
/// environment, and for a release check it left its binary. A dry
/// run prints the list and runs nothing.
pub fn build(params: &BuildParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("{}: entry params={params:?}", params.label);
    if params.dry_run {
        let n = params.commands.len();
        for (i, command) in params.commands.iter().enumerate() {
            info!("{}: [dry-run] [{}/{n}] {command}", params.label, i + 1);
        }
        info!("{}: [dry-run] ran nothing ({})", params.label, params.key);
        return Ok(());
    }
    let env = build_env(params.provenance.as_ref());
    crate::command_list::run(
        &params.root,
        params.key,
        &params.commands,
        &env,
        &[],
        params.label,
    )?;
    let (Some(bin), Some(provenance)) = (&params.bin, &params.provenance) else {
        return Ok(());
    };
    if !bin.is_file() {
        return Err(format!(
            "build: `{}` passed but left no binary at '{}': check `build.release-bin`",
            params.key,
            bin.display()
        )
        .into());
    }
    let short = &provenance.commit[..provenance.commit.len().min(12)];
    let state = match provenance.state {
        SourceState::Clean => "state clean, `@` empty",
        _ => "state dirty, `@` held edits, so the build is not exactly that commit",
    };
    info!("build: {}, from {short}, {state}", bin.display());
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

    fn release(root: &Path, commands: Vec<String>) -> BuildParams {
        BuildParams {
            label: "build",
            key: "build.release",
            commands,
            root: root.to_path_buf(),
            provenance: Some(prov(SourceState::Clean)),
            bin: Some(root.join("rel/bin/app")),
            dry_run: false,
        }
    }

    #[test]
    fn flags_parse() {
        assert_eq!(parse(&["vc-x1", "build"]).repo, None);
        assert_eq!(
            parse(&["vc-x1", "build", "-R", "x"]).repo,
            Some(PathBuf::from("x"))
        );
        assert!(parse(&["vc-x1", "build", "--dev"]).dev);
        assert!(parse(&["vc-x1", "build", "-d"]).dev);
        assert!(!parse(&["vc-x1", "build"]).dev);
        let err =
            BuildParams::try_from(&parse_dev(&["vc-x1", "build-dev", "-R", "x"])).unwrap_err();
        assert!(err.ends_with("`build --dev`"), "{err}");
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

    /// `dev-bin` resolves as `release-bin` does, and its absence names
    /// the key.
    #[test]
    fn dev_bin_resolves_against_the_config() {
        let root = workspace(
            "devbin",
            "```toml\n[repos]\nwork = \".\"\n[build]\ndev-bin = \"dbg/app\"\n```\n",
        );
        assert_eq!(dev_bin(&root).unwrap(), root.join("dbg/app"));
        let e = release_bin(&root).unwrap_err();
        assert!(e.contains("no `build.release-bin`"), "{e}");
        let root2 = workspace("nodevbin", "```toml\n[repos]\nwork = \".\"\n```\n");
        let e = dev_bin(&root2).unwrap_err();
        assert!(e.contains("no `build.dev-bin`"), "{e}");
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&root2).ok();
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
            label: "build --dev",
            key: "build.dev",
            commands: vec!["sh check.sh".to_string()],
            root: root.clone(),
            provenance: Some(prov(SourceState::Clean)),
            bin: None,
            dry_run: false,
        };
        build(&dev).unwrap();
        std::fs::remove_dir_all(&root).ok();
    }

    /// A passing release that left its binary passes.
    #[test]
    fn release_leaves_its_binary() {
        let root = workspace("pass", "");
        std::fs::create_dir_all(root.join("rel/bin")).unwrap();
        build(&release(&root, vec!["touch rel/bin/app".to_string()])).unwrap();
        std::fs::remove_dir_all(&root).ok();
    }

    /// A failed release is an error naming the command.
    #[test]
    fn failed_release_errors() {
        let root = workspace("fail", "");
        let e = build(&release(&root, vec!["false".to_string()]))
            .unwrap_err()
            .to_string();
        assert!(e.contains("`false` failed"), "{e}");
        std::fs::remove_dir_all(&root).ok();
    }

    /// A release whose list passes but leaves no binary at
    /// `release-bin` is an error.
    #[test]
    fn release_without_its_binary_errors() {
        let root = workspace("nobinary", "");
        let e = build(&release(&root, vec!["true".to_string()]))
            .unwrap_err()
            .to_string();
        assert!(e.contains("left no binary"), "{e}");
        std::fs::remove_dir_all(&root).ok();
    }

    /// A dry run prints the list and runs nothing, so a failing
    /// command passes and a command that writes leaves nothing.
    #[test]
    fn a_dry_run_runs_nothing() {
        let root = workspace("dry", "");
        let mut params = release(&root, vec!["touch ran".to_string(), "false".to_string()]);
        params.dry_run = true;
        build(&params).unwrap();
        assert!(!root.join("ran").exists(), "a dry run ran the list");
        assert!(
            parse(&["vc-x1", "build", "--dry-run"]).dry_run.value,
            "build takes --dry-run"
        );
        assert!(
            parse(&["vc-x1", "build", "--dev", "--dry-run"])
                .dry_run
                .value
        );
        std::fs::remove_dir_all(&root).ok();
    }

    /// The dev build runs its list and checks no binary.
    #[test]
    fn dev_runs_its_list() {
        let root = workspace("dev", "");
        let params = BuildParams {
            label: "build --dev",
            key: "build.dev",
            commands: vec!["touch dev-ran".to_string()],
            root: root.clone(),
            provenance: None,
            bin: None,
            dry_run: false,
        };
        build(&params).unwrap();
        assert!(root.join("dev-ran").exists());
        std::fs::remove_dir_all(&root).ok();
    }
}

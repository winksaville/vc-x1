//! The `build` subcommand: build the workspace's chosen profile, by
//! the configured command list. A profile is a `[profile.<name>]`
//! table, chosen by `--profile NAME`, else `[build] default-profile`,
//! else `release`. The dev build was `build-dev`, a command of its own,
//! and then `build --dev`, a flag, until every build became a profile,
//! and both are kept hidden to be refused naming `build --profile dev`.
//! No build touches the PATH: that is `vc-x1 install`'s, which copies
//! one there.
//!
//! - `vc-x1 build` runs `[build] commands`, here
//!   `cargo build --profile {profile.cargo} --locked`, each
//!   `{profile.<property>}` filled from the chosen profile, and checks
//!   it left its binary at the profile's `bin`. The binaries stay in
//!   `target/`, off the PATH, so nothing there is replaced: a build is
//!   run by `vc-x1 run` and copied by `vc-x1 install`, which run this
//!   same build first, through [`build_params`] and [`build`].
//! - A list-valued property, such as `flags`, fills a word of its own
//!   with its items, none for an empty list.
//! - Where a build leaves its binary is the profile's to say, `bin`:
//!   vc-x1 never works it out from cargo or the package's name.
//! - The commands run with the build's [`Provenance`] in the
//!   environment, `VC_X1_BUILD_COMMIT` and `VC_X1_BUILD_STATE`, as
//!   `validate` does, so every build of a run sees the same values and
//!   none recompiles for another. An artifact that compiles them in
//!   can say what it was built from wherever it is copied. vc-x1 does,
//!   as semver build metadata in its banner, `0.85.7+d70cf9ecdfd9`,
//!   `.dirty` for a build with edits. A provenance that cannot be read
//!   is a warning, and the build carries no commit.
//! - No build checks the repo's state: a release may be built from a
//!   cycle's bookmark, to try the release profile mid-cycle.
//! - No build tracks whether its binary is out of date: cargo does,
//!   rebuilding only what changed, so vc-x1 runs the list every time.
//! - The list is run by [`crate::command_list`], as `validate`'s are.
//! - `--dry-run` prints the list and runs nothing, as `run` prints the
//!   build it would run first.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info, warn};

use crate::config_md::VcConfig;
use crate::context::Context;
use crate::options_flags::dry_run::DryRunFlag;
use crate::subcommand::SubcommandRunner;
use crate::version::SourceState;

/// The variable carrying the build's commit to its commands.
pub const BUILD_COMMIT_VAR: &str = "VC_X1_BUILD_COMMIT";

/// The variable carrying `state`, the state of the source code being
/// compiled, `clean` or `dirty`.
pub const BUILD_STATE_VAR: &str = "VC_X1_BUILD_STATE";

/// The key of the build's command list, shared by every profile.
pub const BUILD_COMMANDS_KEY: &str = "build.commands";

/// The keys the profile form retired, each with the profile form that
/// replaces it, a phrase finishing "`<key>` is retired: ". The
/// `[build]` lists and binaries were one pair per build, release and
/// dev, and every build is now a `[profile.<name>]` table filling the
/// one `[build] commands`; the `[install]` lists and directories went
/// the same way, to `[install] commands` and `dir`. `validate-config`
/// reports each one found with its replacement, and `build`, `run`,
/// and `install` refuse a config that holds one in place of its
/// replacement.
pub const RETIRED_KEYS: &[(&str, &str)] = &[
    (
        "build.release",
        "the build for every profile, `[build] commands`, filled from a profile such as \
         `[profile.release] cargo = \"release\"`",
    ),
    (
        "build.release-bin",
        "a profile's `bin`, `[profile.release] bin = \"target/release/<name>\"`",
    ),
    (
        "build.dev",
        "the build for every profile, `[build] commands`, filled from a profile such as \
         `[profile.dev] cargo = \"dev\"`",
    ),
    (
        "build.dev-bin",
        "a profile's `bin`, `[profile.dev] bin = \"target/debug/<name>\"`",
    ),
    (
        "install.release",
        "the install for every profile, `[install] commands`, its placeholders \
         `{profile.bin}`, `{profile.suffix}`, and `{install.dir}`",
    ),
    (
        "install.release-dir",
        "the one install directory, `[install] dir`",
    ),
    (
        "install.dev",
        "the install for every profile, `[install] commands`, its placeholders \
         `{profile.bin}`, `{profile.suffix}`, and `{install.dir}`",
    ),
    (
        "install.dev-dir",
        "the one install directory, `[install] dir`, the dev build named apart by its \
         profile's `suffix`",
    ),
    (
        "install.dev-suffix",
        "a profile's `suffix`, `[profile.dev] suffix = \"-dev\"`",
    ),
];

/// The retirement sentence for `key`, "`<key>` is retired: <its
/// replacement>", or `None` when `key` is not one of
/// [`RETIRED_KEYS`].
pub fn retired_key(key: &str) -> Option<String> {
    RETIRED_KEYS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(k, replacement)| format!("`{k}` is retired: {replacement}"))
}

/// The retired `[build]` keys, which `[build] commands` and the
/// profiles replace.
pub const RETIRED_BUILD_KEYS: &[&str] = &[
    "build.release",
    "build.release-bin",
    "build.dev",
    "build.dev-bin",
];

/// Refuse a config that lacks `key` but still holds one of the
/// retired keys `old`, the ones `key` replaces, naming the migration
/// to the profile form: an old config would otherwise be told only
/// that `key` is missing. `Ok` when none is held, so the caller
/// reports the missing key as it always has. `path` is the config
/// file's, and the caller prefixes its label.
pub fn refuse_retired(
    map: &std::collections::HashMap<String, String>,
    path: &Path,
    key: &str,
    old: &[&str],
) -> Result<(), String> {
    let held: Vec<String> = old
        .iter()
        .filter(|k| map.contains_key(**k))
        .map(|k| format!("`{k}`"))
        .collect();
    if held.is_empty() {
        return Ok(());
    }
    let (held, verb) = match held.len() {
        1 => (held.join(""), "is"),
        _ => (held.join(", "), "are"),
    };
    Err(format!(
        "{}: no `{key}`, and {held} {verb} retired: every build is a profile now, \
         `[build] commands` filled from a `[profile.<name>]` table per build, its `cargo`, \
         `bin`, and `suffix`, and the install is `[install] dir` and, optionally, \
         `[install] commands` (`vc-x1 validate-config` names each retired key's replacement)",
        path.display()
    ))
}

/// Build the chosen profile.
#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Build the profile NAME, a `[profile.<NAME>]` table [default:
    /// `[build] default-profile`, else release]
    #[arg(long, value_name = "NAME")]
    pub profile: Option<String>,

    /// Retired: refused naming `--profile dev`
    #[arg(long, hide = true)]
    pub dev: bool,

    #[command(flatten)]
    pub dry_run: DryRunFlag,

    /// Workspace to build from [default: the workspace containing
    /// the current directory]
    #[arg(short = 'R', long, value_name = "DIR")]
    pub repo: Option<PathBuf>,
}

/// Retired: `build-dev`, now `build --profile dev`, refused naming it.
/// Its flags are kept so an old invocation parses and is told the fix.
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
    /// without it: every profile's and `validate`'s.
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

/// A build profile, a `[profile.<name>]` table, resolved: its name,
/// its properties as the placeholders the lists fill, and the two
/// properties vc-x1 reads itself, `bin` and `suffix`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    /// The profile's name, the table's `<name>`.
    pub name: String,
    /// The string-valued properties, `("profile.<property>", value)`,
    /// sorted by name. `{profile.bin}` is the resolved binary and
    /// `{profile.suffix}` is always offered, empty when absent.
    pub vars: Vec<(String, String)>,
    /// The list-valued properties, `("profile.<property>", items)`,
    /// sorted by name, each filling a word of its own per item.
    pub lists: Vec<(String, Vec<String>)>,
    /// The binary the profile's build leaves, `bin`, against the
    /// config file's directory unless absolute.
    pub bin: PathBuf,
    /// Appended to the binary's file name by `install`, `suffix`,
    /// empty when absent.
    pub suffix: String,
}

impl Profile {
    /// The string placeholders, borrowed for [`crate::command_list`].
    pub fn var_refs(&self) -> Vec<(&str, &str)> {
        self.vars
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_str()))
            .collect()
    }

    /// The list placeholders, borrowed for [`crate::command_list`].
    pub fn list_refs(&self) -> Vec<(&str, &[String])> {
        self.lists
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_slice()))
            .collect()
    }

    /// Every placeholder's name, string and list alike, as
    /// [`crate::command_list::check_placeholders`] takes them.
    pub fn names(&self) -> (Vec<&str>, Vec<&str>) {
        (
            self.vars.iter().map(|(n, _)| n.as_str()).collect(),
            self.lists.iter().map(|(n, _)| n.as_str()).collect(),
        )
    }
}

/// Load the work side's config at `root`, `label` prefixing an error.
fn load_config(root: &Path, label: &str) -> Result<VcConfig, String> {
    crate::config_md::load(root)
        .map_err(|e| format!("{label}: {e}"))?
        .ok_or_else(|| format!("{label}: no config in '{}'", root.display()))
}

/// The names of the profiles `map` defines, every `<name>` with a
/// `profile.<name>.<property>` key, sorted and each once.
pub fn defined_profiles(map: &std::collections::HashMap<String, String>) -> Vec<String> {
    let mut names: Vec<String> = map
        .keys()
        .filter_map(|k| k.strip_prefix("profile."))
        .filter_map(|rest| rest.split_once('.').map(|(name, _)| name.to_string()))
        .collect();
    names.sort();
    names.dedup();
    names
}

/// Resolve the profile to build at `root`: `requested`, the
/// `--profile` value, else `[build] default-profile`, else `release`.
/// A name the config defines no `[profile.<name>]` table for is
/// refused, naming the ones it does define, and a profile without
/// `bin` or with a path separator in `suffix` is refused naming the
/// key. `label` prefixes every error.
pub fn resolve_profile(
    root: &Path,
    requested: Option<&str>,
    label: &str,
) -> Result<Profile, String> {
    let cfg = load_config(root, label)?;
    let name = match requested {
        Some(n) => n.to_string(),
        None => crate::toml_simple::toml_get(&cfg.map, "build.default-profile")
            .cloned()
            // OK: no `build.default-profile` is the schema's default, release
            .unwrap_or_else(|| crate::config_schema::BUILD_DEFAULT_PROFILE_DEFAULT.to_string()),
    };
    profile_in(&cfg, root, &name, label)
}

/// The profile `name` in the loaded `cfg` at `root`: see
/// [`resolve_profile`].
fn profile_in(cfg: &VcConfig, root: &Path, name: &str, label: &str) -> Result<Profile, String> {
    let path = cfg.path.display();
    let prefix = format!("profile.{name}.");
    let mut props: Vec<(&str, &String)> = cfg
        .map
        .iter()
        .filter_map(|(k, v)| k.strip_prefix(&prefix).map(|p| (p, v)))
        .collect();
    props.sort();
    if props.is_empty() {
        let defined = defined_profiles(&cfg.map);
        let defined = if defined.is_empty() {
            "none".to_string()
        } else {
            defined.join(", ")
        };
        return Err(format!(
            "{label}: {path}: no profile `{name}`, a `[profile.{name}]` table, and the profiles \
             defined are: {defined} (see `vc-x1 config -s work`)"
        ));
    }
    let Some((_, bin)) = props.iter().find(|(p, _)| *p == "bin") else {
        return Err(format!(
            "{label}: {path}: no `profile.{name}.bin`, the binary the profile's build leaves, \
             which `run` runs and `install` copies (see `vc-x1 config -s work`)"
        ));
    };
    // OK: a loaded config's path is a file, so it has a parent
    let cfg_dir = cfg.path.parent().unwrap_or(root);
    let bin = crate::common::resolve_repo_path(cfg_dir, bin);
    let suffix = match props.iter().find(|(p, _)| *p == "suffix") {
        Some((_, s)) => s.to_string(),
        None => String::new(),
    };
    if suffix.contains(std::path::is_separator) {
        return Err(format!(
            "{label}: {path}: `profile.{name}.suffix` is '{suffix}', which holds a path separator"
        ));
    }
    let mut vars = Vec::new();
    let mut lists = Vec::new();
    for (prop, value) in &props {
        let key = format!("profile.{prop}");
        match *prop {
            "bin" => vars.push((key, bin.to_string_lossy().into_owned())),
            "suffix" => vars.push((key, suffix.clone())),
            _ if value.trim_start().starts_with('[') => {
                let items =
                    crate::toml_simple::parse_array(value, &format!("profile.{name}.{prop}"))
                        .map_err(|e| format!("{label}: {path}: {e}"))?;
                lists.push((key, items));
            }
            _ => vars.push((key, value.to_string())),
        }
    }
    if !props.iter().any(|(p, _)| *p == "suffix") {
        vars.push(("profile.suffix".to_string(), String::new()));
        vars.sort();
    }
    Ok(Profile {
        name: name.to_string(),
        vars,
        lists,
        bin,
        suffix,
    })
}

/// Per-invocation build inputs: the command's label, the list's key,
/// its commands, the directory to run them from, the build's
/// provenance, and the profile, which fills the list and names the
/// binary the build leaves.
#[derive(Debug)]
pub struct BuildParams {
    /// `build`, prefixing the run's messages.
    pub label: &'static str,
    /// `build.commands`, naming the list in its messages.
    pub key: &'static str,
    /// The commands, one element per invocation.
    pub commands: Vec<String>,
    /// The work repo root the commands run from.
    pub root: PathBuf,
    /// The build's provenance, best effort.
    pub provenance: Option<Provenance>,
    /// The profile being built.
    pub profile: Profile,
    /// Print the list and run nothing.
    pub dry_run: bool,
}

impl TryFrom<&BuildArgs> for BuildParams {
    type Error = String;

    /// The chosen profile: see [`build_params`]. `--dev` is refused.
    fn try_from(a: &BuildArgs) -> Result<Self, String> {
        if a.dev {
            return Err(retired_dev_flag("build"));
        }
        let mut params = build_params(a.profile.as_deref(), a.repo.as_deref())?;
        params.dry_run = a.dry_run.value;
        Ok(params)
    }
}

impl TryFrom<&BuildDevArgs> for BuildParams {
    type Error = String;

    /// Refused: `build-dev` is `build --profile dev`.
    fn try_from(_: &BuildDevArgs) -> Result<Self, String> {
        Err(retired_dev("build"))
    }
}

/// The refusal for a retired `<command>-dev`, naming
/// `<command> --profile dev`.
pub fn retired_dev(command: &str) -> String {
    format!(
        "{command}-dev: the -dev commands are retired, the build is a profile: \
         `{command} --profile dev`"
    )
}

/// The refusal for the retired `<command> --dev`, naming
/// `<command> --profile dev`.
pub fn retired_dev_flag(command: &str) -> String {
    format!("{command}: --dev is retired, the build is a profile: `{command} --profile dev`")
}

/// Resolve the workspace root (from `-R` or the cwd), take the build
/// list, resolve the profile `profile` names (see
/// [`resolve_profile`]), check the list's placeholders against it,
/// and read the provenance, best effort. Every miss but the
/// provenance is an error here, before anything runs.
pub fn build_params(profile: Option<&str>, repo: Option<&Path>) -> Result<BuildParams, String> {
    let label = "build";
    let key = BUILD_COMMANDS_KEY;
    let root = match repo {
        Some(p) => crate::common::find_workspace_root_from(p)
            .ok_or_else(|| format!("{label}: '{}' is not in a vc-x1 workspace", p.display()))?,
        None => crate::common::find_workspace_root()
            .ok_or_else(|| format!("{label}: not in a vc-x1 workspace (no config found)"))?,
    };
    let commands = match crate::command_list::load_optional(&root, key, label)? {
        Some(c) => c,
        None => {
            let cfg = load_config(&root, label)?;
            refuse_retired(&cfg.map, &cfg.path, key, RETIRED_BUILD_KEYS)
                .map_err(|e| format!("{label}: {e}"))?;
            return Err(format!(
                "{label}: {}: no `{key}`, the build for every profile. Add one, such as \
                 `commands = [\"cargo build --profile {{profile.cargo}} --locked\"]` under \
                 `[build]`, with a `[profile.<name>]` table per build (see `vc-x1 config -s work`)",
                cfg.path.display()
            ));
        }
    };
    let profile = resolve_profile(&root, profile, label)?;
    let (vars, lists) = profile.names();
    crate::command_list::check_placeholders(&commands, &vars, &lists, key, label)?;
    let provenance = Provenance::read_or_warn(&root, label);
    Ok(BuildParams {
        label,
        key,
        commands,
        root,
        provenance,
        profile,
        dry_run: false,
    })
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

    /// Never reached, since the params are refused, but the same op
    /// as `build`.
    fn run(_ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        build(params)
    }
}

/// Run the list from the root, its placeholders filled from the
/// profile and the provenance in its environment, and check it left
/// the profile's binary. A dry run prints the list and runs nothing.
pub fn build(params: &BuildParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("{}: entry params={params:?}", params.label);
    info!("{}: profile {}", params.label, params.profile.name);
    let (vars, lists) = (params.profile.var_refs(), params.profile.list_refs());
    if params.dry_run {
        let n = params.commands.len();
        for (i, command) in params.commands.iter().enumerate() {
            // The command as it would run, its placeholders filled.
            let shown = crate::command_list::expand(command, &vars, &lists)
                .map(|words| words.join(" "))
                // OK: a command that cannot be filled is shown as written
                .unwrap_or_else(|_| command.clone());
            info!("{}: [dry-run] [{}/{n}] {shown}", params.label, i + 1);
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
        &vars,
        &lists,
        params.label,
    )?;
    let bin = &params.profile.bin;
    if !bin.is_file() {
        return Err(format!(
            "{}: `{}` passed but left no binary at '{}': check `profile.{}.bin`",
            params.label,
            params.key,
            bin.display(),
            params.profile.name
        )
        .into());
    }
    let Some(provenance) = &params.provenance else {
        info!("{}: {}, carrying no commit", params.label, bin.display());
        return Ok(());
    };
    let short = &provenance.commit[..provenance.commit.len().min(12)];
    let state = match provenance.state {
        SourceState::Clean => "state clean, `@` empty",
        _ => "state dirty, `@` held edits, so the build is not exactly that commit",
    };
    info!("{}: {}, from {short}, {state}", params.label, bin.display());
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

    /// A config of `toml`, a work repo at its root.
    fn config(toml: &str) -> String {
        format!("```toml\n[repos]\nwork = \".\"\n{toml}```\n")
    }

    /// The two profiles this workspace defines, as the config writes
    /// them.
    const PROFILES: &str = "[profile.release]\ncargo = \"release\"\nbin = \"rel/app\"\n\
                            [profile.dev]\ncargo = \"dev\"\nbin = \"dbg/app\"\nsuffix = \"-dev\"\n";

    const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

    fn prov(state: SourceState) -> Provenance {
        Provenance {
            commit: COMMIT.to_string(),
            state,
        }
    }

    /// A profile `name` leaving `bin`, with no other property.
    fn profile(name: &str, bin: &Path) -> Profile {
        Profile {
            name: name.to_string(),
            vars: vec![
                (
                    "profile.bin".to_string(),
                    bin.to_string_lossy().into_owned(),
                ),
                ("profile.suffix".to_string(), String::new()),
            ],
            lists: Vec::new(),
            bin: bin.to_path_buf(),
            suffix: String::new(),
        }
    }

    fn release(root: &Path, commands: Vec<String>) -> BuildParams {
        BuildParams {
            label: "build",
            key: BUILD_COMMANDS_KEY,
            commands,
            root: root.to_path_buf(),
            provenance: Some(prov(SourceState::Clean)),
            profile: profile("release", &root.join("rel/bin/app")),
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
        assert_eq!(
            parse(&["vc-x1", "build", "--profile", "dev"])
                .profile
                .as_deref(),
            Some("dev")
        );
        assert_eq!(parse(&["vc-x1", "build"]).profile, None);
        assert!(
            Cli::try_parse_from(["vc-x1", "build", "-d"]).is_err(),
            "-d is gone"
        );
        let err =
            BuildParams::try_from(&parse_dev(&["vc-x1", "build-dev", "-R", "x"])).unwrap_err();
        assert!(err.ends_with("`build --profile dev`"), "{err}");
    }

    /// `--dev` parses, hidden, and is refused naming `--profile dev`.
    #[test]
    fn the_dev_flag_is_refused() {
        let a = parse(&["vc-x1", "build", "--dev", "-R", "x"]);
        assert!(a.dev);
        assert_eq!(
            BuildParams::try_from(&a).unwrap_err(),
            "build: --dev is retired, the build is a profile: `build --profile dev`"
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

    /// The profile is the one requested, else `build.default-profile`,
    /// else release.
    #[test]
    fn resolve_profile_picks_requested_default_or_release() {
        let root = workspace("pick", &config(PROFILES));
        assert_eq!(
            resolve_profile(&root, None, "build").unwrap().name,
            "release"
        );
        let dev = resolve_profile(&root, Some("dev"), "build").unwrap();
        assert_eq!(dev.name, "dev");
        assert_eq!(dev.suffix, "-dev");
        let root2 = workspace(
            "default",
            &config(&format!("[build]\ndefault-profile = \"dev\"\n{PROFILES}")),
        );
        assert_eq!(resolve_profile(&root2, None, "build").unwrap().name, "dev");
        assert_eq!(
            resolve_profile(&root2, Some("release"), "build")
                .unwrap()
                .name,
            "release",
            "--profile wins over the default"
        );
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&root2).ok();
    }

    /// An unknown profile is refused naming the ones defined, sorted,
    /// and the config file.
    #[test]
    fn an_unknown_profile_names_the_defined_ones() {
        let root = workspace("unknown", &config(PROFILES));
        let e = resolve_profile(&root, Some("lto"), "build").unwrap_err();
        assert!(e.starts_with("build: "), "{e}");
        assert!(e.contains("no profile `lto`"), "{e}");
        assert!(e.contains("the profiles defined are: dev, release"), "{e}");
        assert!(e.contains(".vc-config.md"), "{e}");
        let root2 = workspace("none", &config(""));
        let e = resolve_profile(&root2, None, "run").unwrap_err();
        assert!(e.contains("no profile `release`"), "{e}");
        assert!(e.contains("defined are: none"), "{e}");
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&root2).ok();
    }

    /// A profile without `bin` is refused naming the key, and a suffix
    /// holding a path separator is refused.
    #[test]
    fn a_profile_needs_bin_and_a_plain_suffix() {
        let root = workspace("nobin", &config("[profile.release]\ncargo = \"release\"\n"));
        let e = resolve_profile(&root, None, "build").unwrap_err();
        assert!(e.contains("no `profile.release.bin`"), "{e}");
        let root2 = workspace(
            "slash",
            &config("[profile.dev]\nbin = \"x\"\nsuffix = \"/x\"\n"),
        );
        let e = resolve_profile(&root2, Some("dev"), "build").unwrap_err();
        assert!(e.contains("path separator"), "{e}");
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&root2).ok();
    }

    /// `bin` resolves against the config file's directory, or as is
    /// when absolute, and fills `{profile.bin}` resolved. A missing
    /// suffix is offered empty, and a list-valued property is a list.
    #[test]
    fn a_profile_resolves_its_properties() {
        let root = workspace(
            "props",
            &config(
                "[profile.release]\nbin = \"rel/bin/app\"\nflags = [\"--release\", \"-q\"]\n\
                 [profile.abs]\nbin = \"/opt/app\"\n",
            ),
        );
        let p = resolve_profile(&root, None, "build").unwrap();
        assert_eq!(p.bin, root.join("rel/bin/app"));
        let bin = root.join("rel/bin/app").to_string_lossy().into_owned();
        assert_eq!(
            p.var_refs(),
            vec![("profile.bin", bin.as_str()), ("profile.suffix", "")]
        );
        assert_eq!(
            p.lists,
            vec![(
                "profile.flags".to_string(),
                vec!["--release".to_string(), "-q".to_string()]
            )]
        );
        let p = resolve_profile(&root, Some("abs"), "build").unwrap();
        assert_eq!(p.bin, PathBuf::from("/opt/app"));
        std::fs::remove_dir_all(&root).ok();
    }

    /// The build list is required, and its absence names the key and
    /// the profile form.
    #[test]
    fn the_build_list_is_required() {
        let root = workspace("nolist", &config(PROFILES));
        let e = build_params(None, Some(&root)).unwrap_err();
        assert!(e.contains("no `build.commands`"), "{e}");
        assert!(e.contains("[profile.<name>]"), "{e}");
        assert!(!e.contains("retired"), "no old key, no migration: {e}");
        std::fs::remove_dir_all(&root).ok();
    }

    /// A config holding only the retired `[build]` keys is refused
    /// naming them and the migration to the profile form, by `build`
    /// and by `run`, which takes the same build first.
    #[test]
    fn the_retired_build_keys_are_refused_naming_the_migration() {
        let root = workspace(
            "retired",
            &config(
                "[build]
release = [\"cargo build --release\"]
                 release-bin = \"target/release/app\"
",
            ),
        );
        let e = build_params(None, Some(&root)).unwrap_err();
        assert!(e.starts_with("build: "), "{e}");
        assert!(e.contains("no `build.commands`"), "{e}");
        assert!(
            e.contains("`build.release`, `build.release-bin` are retired"),
            "{e}"
        );
        assert!(e.contains("[profile.<name>]"), "{e}");
        assert!(e.contains("`vc-x1 validate-config`"), "{e}");
        let Err(e) = crate::run_cmd::run_params(None, Some(&root), &[], false) else {
            panic!("run refuses the retired keys");
        };
        assert!(e.contains("are retired"), "{e}");
        std::fs::remove_dir_all(&root).ok();
    }

    /// Each retired key has one sentence naming its replacement, and
    /// a key that is not retired has none.
    #[test]
    fn a_retired_key_names_its_replacement() {
        assert_eq!(RETIRED_KEYS.len(), 9);
        assert_eq!(
            retired_key("build.release-bin").unwrap(),
            "`build.release-bin` is retired: a profile's `bin`, \
             `[profile.release] bin = \"target/release/<name>\"`"
        );
        assert_eq!(
            retired_key("install.dev-suffix").unwrap(),
            "`install.dev-suffix` is retired: a profile's `suffix`, \
             `[profile.dev] suffix = \"-dev\"`"
        );
        for k in RETIRED_BUILD_KEYS {
            assert!(retired_key(k).is_some(), "{k}");
        }
        assert_eq!(retired_key("build.commands"), None);
    }

    /// A placeholder the profile does not offer is refused before
    /// anything runs.
    #[test]
    fn an_unknown_placeholder_is_refused_before_the_build() {
        let root = workspace(
            "badph",
            &config(&format!(
                "[build]\ncommands = [\"touch {{profile.nope}}\"]\n{PROFILES}"
            )),
        );
        let e = build_params(None, Some(&root)).unwrap_err();
        assert!(e.contains("`{profile.nope}` is not a placeholder"), "{e}");
        assert!(!root.join("{profile.nope}").exists());
        std::fs::remove_dir_all(&root).ok();
    }

    /// A list-valued `flags` property fills a word per item in the
    /// build list, checked by a script taking them as arguments.
    #[test]
    fn a_list_valued_flags_property_expands() {
        let root = workspace(
            "flags",
            &config(
                "[build]\ncommands = [\"sh check.sh {profile.flags} {profile.cargo}\", \
                 \"touch {profile.bin}\"]\n\
                 [profile.release]\ncargo = \"rel\"\nbin = \"app\"\n\
                 flags = [\"--release\", \"--locked\"]\n",
            ),
        );
        std::fs::write(
            root.join("check.sh"),
            "[ \"$1\" = --release ] && [ \"$2\" = --locked ] && [ \"$3\" = rel ] && [ $# = 3 ]\n",
        )
        .unwrap();
        let params = build_params(None, Some(&root)).unwrap();
        assert_eq!(params.key, "build.commands");
        build(&params).unwrap();
        assert!(root.join("app").is_file());
        std::fs::remove_dir_all(&root).ok();
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
        std::fs::remove_dir_all(&root).ok();
    }

    /// A passing build that left its binary passes.
    #[test]
    fn release_leaves_its_binary() {
        let root = workspace("pass", "");
        std::fs::create_dir_all(root.join("rel/bin")).unwrap();
        build(&release(&root, vec!["touch rel/bin/app".to_string()])).unwrap();
        std::fs::remove_dir_all(&root).ok();
    }

    /// A failed build is an error naming the command.
    #[test]
    fn failed_release_errors() {
        let root = workspace("fail", "");
        let e = build(&release(&root, vec!["false".to_string()]))
            .unwrap_err()
            .to_string();
        assert!(e.contains("`false` failed"), "{e}");
        std::fs::remove_dir_all(&root).ok();
    }

    /// A build whose list passes but leaves no binary at the
    /// profile's `bin` is an error naming the key, for every profile.
    #[test]
    fn a_build_without_its_binary_errors() {
        let root = workspace("nobinary", "");
        let e = build(&release(&root, vec!["true".to_string()]))
            .unwrap_err()
            .to_string();
        assert!(e.contains("left no binary"), "{e}");
        assert!(e.contains("`profile.release.bin`"), "{e}");
        let mut dev = release(&root, vec!["true".to_string()]);
        dev.profile = profile("dev", &root.join("dbg/app"));
        dev.provenance = None;
        let e = build(&dev).unwrap_err().to_string();
        assert!(e.contains("`profile.dev.bin`"), "{e}");
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
            parse(&["vc-x1", "build", "--profile", "dev", "--dry-run"])
                .dry_run
                .value
        );
        std::fs::remove_dir_all(&root).ok();
    }

    /// A build without a provenance runs its list and checks its
    /// binary.
    #[test]
    fn a_build_without_provenance_runs_its_list() {
        let root = workspace("dev", "");
        let mut params = release(&root, vec!["touch dev-ran".to_string()]);
        params.provenance = None;
        params.profile = profile("dev", &root.join("dev-ran"));
        build(&params).unwrap();
        assert!(root.join("dev-ran").exists());
        std::fs::remove_dir_all(&root).ok();
    }
}

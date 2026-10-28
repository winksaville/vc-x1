//! The `run` subcommand: build the chosen profile, then run its binary
//! with the given arguments, as `cargo run` builds before it runs. It
//! never touches the PATH, so a build is tried before `install` copies
//! it. The profile is `--profile NAME`, else `[build] default-profile`,
//! else `release`. The dev run was `run-dev`, and then `run --dev`,
//! both kept hidden to be refused naming `run --profile dev`.
//!
//! - `vc-x1 run` runs `[build] commands` for the profile and then the
//!   profile's `bin`, through `build`'s own code, so what runs is
//!   always the current code. `--profile NAME` comes before the
//!   arguments, which are the binary's.
//! - The arguments go to the binary untouched. One that starts with
//!   `-` comes after `--`, `vc-x1 run -- -V`, as with `cargo run`,
//!   since vc-x1's own flags, `-V` among them, are read first.
//! - The binary runs in the current directory, with the terminal, and
//!   its exit code is vc-x1's, so a failing run fails the same way.
//! - Before it runs, its `-V` is read, and a version other than the
//!   manifest's is a warning naming the key: the build passed but
//!   left an older binary where the config says. Output with no
//!   version is passed over quietly, since another project's binary
//!   may print none.
//! - `--dry-run` prints the build's list and the command, and runs
//!   nothing.
//! - Completion is forwarded: the words after `run` and its own
//!   options are completed by the binary it would run, asked in clap's
//!   completion mode, so `vc-x1 run --profile dev run <TAB>` offers
//!   the dev build's own words. The binary is asked as it stands and never built, and
//!   one that answers nothing, or is not there, offers nothing. See
//!   [`forward_completion`].

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Args;
use log::{debug, error, info, warn};

use crate::build_cmd::{BuildParams, build_params, resolve_profile};
use crate::options_flags::dry_run::DryRunFlag;

/// Build the chosen profile, then run it.
#[derive(Args, Debug)]
pub struct RunArgs {
    /// Build and run the profile NAME, a `[profile.<NAME>]` table,
    /// given before the arguments [default: `[build] default-profile`,
    /// else release]
    #[arg(long, value_name = "NAME")]
    pub profile: Option<String>,

    /// Retired: refused naming `--profile dev`
    #[arg(long, hide = true)]
    pub dev: bool,

    #[command(flatten)]
    pub dry_run: DryRunFlag,

    /// Workspace to run from [default: the workspace containing the
    /// current directory]
    #[arg(short = 'R', long, value_name = "DIR")]
    pub repo: Option<PathBuf>,

    /// Arguments for the profile's binary, after `--` when the first
    /// starts with `-`
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        value_name = "ARGS"
    )]
    pub args: Vec<OsString>,
}

/// Retired: `run-dev`, now `run --profile dev`, refused naming it. Its flags
/// are kept so an old invocation parses and is told the fix.
#[derive(Args, Debug)]
pub struct RunDevArgs {
    #[command(flatten)]
    pub dry_run: DryRunFlag,

    /// Workspace to run from [default: the workspace containing the
    /// current directory]
    #[arg(short = 'R', long, value_name = "DIR")]
    pub repo: Option<PathBuf>,

    /// Arguments for the dev build, after `--` when the first starts
    /// with `-`
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        value_name = "ARGS"
    )]
    pub args: Vec<OsString>,
}

/// Per-invocation run inputs: the build to run first, its binary, the
/// manifest's version to check it against, and the arguments.
#[derive(Debug)]
pub struct RunParams {
    /// `run`, prefixing the run's messages.
    pub label: &'static str,
    /// `profile.<name>.bin`, for a message to point at.
    pub bin_key: String,
    /// The build, `vc-x1 build --profile <name>`'s, a dry run when
    /// this one is.
    pub build: BuildParams,
    /// The binary to run.
    pub bin: PathBuf,
    /// The manifest's version, `None` when the root has no
    /// `Cargo.toml` to read it from.
    pub version: Option<String>,
    /// The arguments, passed untouched.
    pub args: Vec<OsString>,
    /// Print the build's list and the command, and run nothing.
    pub dry_run: bool,
}

impl TryFrom<&RunArgs> for RunParams {
    type Error = String;

    /// The chosen profile: see [`run_params`]. `--dev` is refused.
    fn try_from(a: &RunArgs) -> Result<Self, String> {
        if a.dev {
            return Err(crate::build_cmd::retired_dev_flag("run"));
        }
        run_params(
            a.profile.as_deref(),
            a.repo.as_deref(),
            &a.args,
            a.dry_run.value,
        )
    }
}

impl TryFrom<&RunDevArgs> for RunParams {
    type Error = String;

    /// Refused: `run-dev` is `run --profile dev`.
    fn try_from(_: &RunDevArgs) -> Result<Self, String> {
        Err(crate::build_cmd::retired_dev("run"))
    }
}

/// Resolve the build of the profile `profile` names (see
/// [`resolve_profile`]) and its binary. A missing config key is an
/// error here, before anything runs.
pub fn run_params(
    profile: Option<&str>,
    repo: Option<&Path>,
    args: &[OsString],
    dry_run: bool,
) -> Result<RunParams, String> {
    let label = "run";
    let mut build = build_params(profile, repo)?;
    build.dry_run = dry_run;
    let bin = build.profile.bin.clone();
    let bin_key = format!("profile.{}.bin", build.profile.name);
    let version = crate::install_cmd::manifest_version(&build.root).ok();
    Ok(RunParams {
        label,
        bin_key,
        build,
        bin,
        version,
        args: args.to_vec(),
        dry_run,
    })
}

/// Run `run`, for any profile, for `main`: the binary's exit code is
/// vc-x1's, so a failing run fails the same way, and any error
/// before it runs is logged and fails.
pub fn dispatch(params: Result<RunParams, String>) -> ExitCode {
    let params = match params {
        Ok(p) => p,
        Err(e) => {
            error!("{e}");
            return ExitCode::FAILURE;
        }
    };
    crate::bm_track("enter", params.label);
    let code = match run(&params) {
        Ok(code) => exit_code(code),
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    };
    crate::bm_track("exit ", params.label);
    code
}

/// The process exit code for a binary's `code`.
fn exit_code(code: i32) -> ExitCode {
    // OK: a code outside 0..=255, which no Unix exit status is, is a failure
    ExitCode::from(u8::try_from(code).unwrap_or(1))
}

/// Build, check the binary, and run it with the arguments, returning
/// its exit code. A dry run prints the build's list and the command,
/// and returns 0.
pub fn run(params: &RunParams) -> Result<i32, Box<dyn std::error::Error>> {
    debug!("{}: entry params={params:?}", params.label);
    crate::build_cmd::build(&params.build)?;
    let shown = shown_command(&params.bin, &params.args);
    if params.dry_run {
        info!("{}: [dry-run] would run {shown}", params.label);
        return Ok(0);
    }
    if !params.bin.is_file() {
        return Err(format!(
            "{}: no binary at '{}' after the build: check `{}`",
            params.label,
            params.bin.display(),
            params.bin_key
        )
        .into());
    }
    let reported = crate::install_cmd::run_version(&params.bin)
        .as_deref()
        .and_then(crate::install_cmd::parse_reported);
    if let Some(note) = version_note(
        reported.as_ref().map(|r| r.version.as_str()),
        params.version.as_deref(),
    ) {
        warn!(
            "{}: '{}' {note}: check `{}`",
            params.label,
            params.bin.display(),
            params.bin_key
        );
    }
    info!("{}: {shown}", params.label);
    // Register entry 7 (clippy.toml): run runs the workspace's build,
    // which is the whole job of the command.
    #[allow(clippy::disallowed_methods)]
    let status = std::process::Command::new(&params.bin)
        .args(&params.args)
        .status()
        .map_err(|e| {
            format!(
                "{}: cannot run '{}': {e}",
                params.label,
                params.bin.display()
            )
        })?;
    match status.code() {
        Some(code) => Ok(code),
        None => Err(format!("{}: '{}' {status}", params.label, params.bin.display()).into()),
    }
}

/// The command as the log shows it: the binary and its arguments.
fn shown_command(bin: &Path, args: &[OsString]) -> String {
    let mut out = format!("'{}'", bin.display());
    for a in args {
        out.push(' ');
        out.push_str(&a.to_string_lossy());
    }
    out
}

/// The warning for a binary reporting `reported` when the manifest
/// says `manifest`, `None` when they agree or either is unknown.
fn version_note(reported: Option<&str>, manifest: Option<&str>) -> Option<String> {
    let (reported, manifest) = (reported?, manifest?);
    (reported != manifest).then(|| format!("reports {reported}, the manifest's is {manifest}"))
}

/// What a completion forward asks: the profile whose binary answers,
/// the workspace `-R` named, the words it is given, its own name
/// first, and the cursor's index among them when the shell gave one.
#[derive(Debug, PartialEq, Eq)]
pub struct Forward {
    /// The profile `--profile` named, else the default.
    pub profile: Option<String>,
    /// The workspace `-R` named, else the current directory's.
    pub repo: Option<PathBuf>,
    /// The words for the binary, the first standing for its name.
    pub words: Vec<String>,
    /// The cursor's index in `words`, when the shell passed one.
    pub index: Option<usize>,
}

/// Plan a completion forward from the words a shell passes, the
/// program's name first, and the cursor's index among them, or
/// `None` when the words are not `run`'s arguments and clap completes
/// them itself.
///
/// - The subcommand is the first word after vc-x1's own flags, and
///   only `run` is forwarded. Any other subcommand is clap's.
/// - The subcommand's own options, `--profile NAME`,
///   `--profile=NAME`, `--dry-run`, `-R DIR`, `--repo DIR`, and
///   `--repo=DIR`, are skipped, `--profile` choosing the profile, and
///   `--` ends them. A word
///   at the cursor that starts with `-` before any argument is
///   clap's, so `run --d<TAB>` offers `--dry-run`.
/// - The cursor on the subcommand or its options is clap's too, so
///   only the arguments are forwarded, with the index shifted to
///   match.
pub fn plan_forward(words: &[String], index: Option<usize>) -> Option<Forward> {
    // OK: with no index the shell completes the last word
    let cursor = index.unwrap_or(words.len().saturating_sub(1));
    let sub = words
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, w)| !w.starts_with('-'))
        .map(|(i, _)| i)?;
    if words[sub] != "run" {
        return None;
    }
    let mut profile = None;
    let mut repo = None;
    let mut i = sub + 1;
    while i < words.len() && i < cursor {
        let w = words[i].as_str();
        if w == "--" {
            i += 1;
            break;
        }
        if w == "--dry-run" {
            i += 1;
        } else if w == "--profile" {
            profile = words.get(i + 1).cloned();
            i += 2;
        } else if let Some(name) = w.strip_prefix("--profile=") {
            profile = Some(name.to_string());
            i += 1;
        } else if w == "-R" || w == "--repo" {
            repo = words.get(i + 1).map(PathBuf::from);
            i += 2;
        } else if let Some(dir) = w.strip_prefix("--repo=") {
            repo = Some(PathBuf::from(dir));
            i += 1;
        } else {
            break;
        }
    }
    if cursor < i || (cursor == i && words[i].starts_with('-') && words[i - 1] != "--") {
        return None;
    }
    let mut forwarded = vec![words[0].clone()];
    forwarded.extend(words[i..].iter().cloned());
    Some(Forward {
        profile,
        repo,
        words: forwarded,
        index: index.map(|n| n - (i - 1)),
    })
}

/// Forward a completion request for `run`'s arguments
/// to the binary it would run, and exit with its answer. Called by
/// `main` before clap's completion, which it leaves alone for every
/// other request.
///
/// A shell asks by running `COMPLETE=<shell> vc-x1 -- <words>` with
/// the cursor in `_CLAP_COMPLETE_INDEX` for most shells. The binary
/// is asked the same way, `COMPLETE` and the shell's other variables
/// inherited and the index shifted, so it answers in the protocol
/// the shell reads. Its config is the workspace's the shell is in, or
/// `-R`'s, its profile `--profile`'s or the default, and it is never
/// built: a profile that does not resolve, no binary, or none that
/// answers, prints nothing.
pub fn forward_completion() {
    if std::env::var_os("COMPLETE").is_none_or(|v| v.is_empty() || v == "0") {
        return;
    }
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) != Some("--") {
        return;
    }
    let index = std::env::var("_CLAP_COMPLETE_INDEX")
        .ok()
        .and_then(|i| i.parse().ok());
    let Some(plan) = plan_forward(&args[2..], index) else {
        return;
    };
    let code = forward(&plan);
    std::process::exit(code);
}

/// Ask the binary `plan` names, returning its exit code, 0 when there
/// is none to ask.
fn forward(plan: &Forward) -> i32 {
    let root = match &plan.repo {
        Some(p) => crate::common::find_workspace_root_from(p),
        None => crate::common::find_workspace_root(),
    };
    let Some(root) = root else {
        return 0;
    };
    let Ok(profile) = resolve_profile(&root, plan.profile.as_deref(), "run") else {
        return 0;
    };
    let bin = profile.bin;
    if !bin.is_file() {
        return 0;
    }
    // Register entry 7 (clippy.toml): the completion forward asks the
    // workspace's build, in clap's completion mode, for its own words.
    #[allow(clippy::disallowed_methods)]
    let mut cmd = std::process::Command::new(&bin);
    cmd.arg("--").args(&plan.words);
    if let Some(index) = plan.index {
        cmd.env("_CLAP_COMPLETE_INDEX", index.to_string());
    }
    match cmd.status() {
        // OK: a binary killed by a signal answered nothing
        Ok(status) => status.code().unwrap_or(0),
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_cmd::Profile;
    use crate::{Cli, Commands};
    use clap::Parser;

    fn w(s: &str) -> Vec<String> {
        s.split(' ').map(str::to_string).collect()
    }

    /// Unique tempdir.
    fn tempdir(tag: &str) -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("vc-x1-run-{tag}-{ts}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// The arguments pass through untouched, a hyphen one after `--`
    /// or after the first argument, and vc-x1's own `-V` before them
    /// is vc-x1's.
    #[test]
    fn flags_parse() {
        let run = |args: &[&str]| match Cli::try_parse_from(args).unwrap().command {
            Some(Commands::Run(a)) => a,
            _ => panic!("expected Run"),
        };
        let a = run(&["vc-x1", "run", "--", "-V"]);
        assert_eq!(a.args, [OsString::from("-V")]);
        let a = run(&["vc-x1", "run", "bench", "--fast", "-x", "--", "y"]);
        assert_eq!(a.args, ["bench", "--fast", "-x", "--", "y"]);
        let a = run(&["vc-x1", "run", "--dry-run", "-R", "w", "x"]);
        assert!(a.dry_run.value);
        assert_eq!(
            (a.repo, a.args),
            (Some(PathBuf::from("w")), vec![OsString::from("x")])
        );
        let cli = Cli::try_parse_from(["vc-x1", "run", "-V"]).unwrap();
        assert_eq!(cli.version, 1, "-V before the arguments is vc-x1's");
        let a = run(&["vc-x1", "run", "--profile", "dev", "--", "-V"]);
        assert_eq!(a.profile.as_deref(), Some("dev"));
        assert_eq!(a.args, [OsString::from("-V")]);
        let a = run(&["vc-x1", "run", "x", "--profile", "dev"]);
        assert_eq!(
            a.profile, None,
            "--profile after an argument is the binary's"
        );
        assert_eq!(a.args, ["x", "--profile", "dev"]);
        let a = run(&["vc-x1", "run", "--dev", "--", "-V"]);
        assert!(a.dev);
        assert_eq!(
            RunParams::try_from(&a).unwrap_err(),
            "run: --dev is retired, the build is a profile: `run --profile dev`"
        );
        match Cli::try_parse_from(["vc-x1", "run-dev", "--", "-V"])
            .unwrap()
            .command
        {
            Some(Commands::RunDev(a)) => {
                let err = RunParams::try_from(&a).unwrap_err();
                assert!(err.ends_with("`run --profile dev`"), "{err}");
            }
            _ => panic!("expected RunDev"),
        }
    }

    /// A version other than the manifest's is named, and agreement or
    /// an unknown either side says nothing.
    #[test]
    fn version_note_forms() {
        assert_eq!(
            version_note(Some("0.1.0"), Some("0.2.0")).as_deref(),
            Some("reports 0.1.0, the manifest's is 0.2.0")
        );
        assert_eq!(version_note(Some("0.2.0"), Some("0.2.0")), None);
        assert_eq!(version_note(None, Some("0.2.0")), None);
        assert_eq!(version_note(Some("0.2.0"), None), None);
    }

    /// The binary runs with the arguments untouched, in the current
    /// directory, after the build, and its exit code is returned; a
    /// dry run builds nothing and runs nothing.
    #[test]
    fn the_binary_runs_with_its_arguments() {
        let dir = tempdir("bin");
        let out = dir.join("args.txt");
        let bin = dir.join("app");
        std::fs::write(
            &bin,
            format!(
                "#!/bin/sh\nprintf '%s|' \"$@\" > '{}'\nexit 3\n",
                out.display()
            ),
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut params = RunParams {
            label: "run",
            bin_key: "profile.dev.bin".to_string(),
            build: BuildParams {
                label: "build",
                key: "build.commands",
                commands: vec!["touch built".to_string()],
                root: dir.clone(),
                provenance: None,
                profile: Profile {
                    name: "dev".to_string(),
                    vars: Vec::new(),
                    lists: Vec::new(),
                    bin: bin.clone(),
                    suffix: "-dev".to_string(),
                },
                dry_run: true,
            },
            bin: bin.clone(),
            version: None,
            args: vec!["a b".into(), "-x".into()],
            dry_run: true,
        };
        assert_eq!(run(&params).unwrap(), 0);
        assert!(
            !out.exists() && !dir.join("built").exists(),
            "a dry run ran"
        );
        params.dry_run = false;
        params.build.dry_run = false;
        assert_eq!(run(&params).unwrap(), 3, "the binary's exit code");
        assert!(dir.join("built").exists(), "the build ran first");
        assert_eq!(std::fs::read_to_string(&out).unwrap(), "a b|-x|");
        params.bin = dir.join("absent");
        params.build.profile.bin = dir.join("absent");
        let e = run(&params).unwrap_err().to_string();
        assert!(
            e.contains("no binary at") && e.contains("profile.dev.bin"),
            "{e}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Only `run`'s arguments are forwarded, its
    /// own options skipped and the index shifted, and the cursor on
    /// the subcommand, its options, or a flag before any argument is
    /// clap's.
    #[test]
    fn plan_forward_forms() {
        let p = plan_forward(&w("vc-x1 run --profile dev run zcr"), Some(5)).unwrap();
        assert_eq!(
            p,
            Forward {
                profile: Some("dev".to_string()),
                repo: None,
                words: w("vc-x1 run zcr"),
                index: Some(2),
            }
        );
        let p = plan_forward(&w("vc-x1 run --profile=x run zcr"), Some(4)).unwrap();
        assert_eq!(
            (p.profile.as_deref(), p.words, p.index),
            (Some("x"), w("vc-x1 run zcr"), Some(2))
        );
        let p = plan_forward(&w("vc-x1 -v run --dry-run -R ../w -- --fast "), Some(7)).unwrap();
        assert_eq!(p.profile, None);
        assert_eq!(p.repo, Some(PathBuf::from("../w")));
        assert_eq!((p.words, p.index), (w("vc-x1 --fast "), Some(1)));
        let p = plan_forward(&w("vc-x1 run --repo=x a"), None).unwrap();
        assert_eq!(
            (p.repo, p.words, p.index),
            (Some(PathBuf::from("x")), w("vc-x1 a"), None)
        );
        assert_eq!(
            plan_forward(&w("vc-x1 run-dev run"), Some(2)),
            None,
            "the retired run-dev is clap's, which refuses it"
        );
        assert_eq!(
            plan_forward(&w("vc-x1 run --d"), Some(2)),
            None,
            "run's own flag"
        );
        assert_eq!(
            plan_forward(&w("vc-x1 run -R "), Some(3)),
            None,
            "-R's value"
        );
        assert_eq!(
            plan_forward(&w("vc-x1 run --profile "), Some(3)),
            None,
            "--profile's value"
        );
        assert_eq!(
            plan_forward(&w("vc-x1 build x"), Some(2)),
            None,
            "another subcommand"
        );
        assert_eq!(
            plan_forward(&w("vc-x1 run a --d"), Some(3)).unwrap().words,
            w("vc-x1 a --d")
        );
    }

    /// A forward with no workspace, or no binary, answers nothing.
    #[test]
    fn a_forward_with_nothing_to_ask_answers_nothing() {
        let dir = tempdir("noforward");
        let plan = Forward {
            profile: Some("dev".to_string()),
            repo: Some(dir.clone()),
            words: w("x a"),
            index: Some(1),
        };
        assert_eq!(forward(&plan), 0, "no workspace");
        std::fs::write(
            dir.join(".vc-config.md"),
            "```toml\n[repos]\nwork = \".\"\n[profile.dev]\nbin = \"absent\"\n```\n",
        )
        .unwrap();
        assert_eq!(forward(&plan), 0, "no binary");
        let unknown = Forward {
            profile: Some("lto".to_string()),
            ..plan
        };
        assert_eq!(forward(&unknown), 0, "a profile that does not resolve");
        std::fs::remove_dir_all(&dir).ok();
    }
}

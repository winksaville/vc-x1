//! `cargo xtask run [--release | --profile NAME] [--bin NAME]
//! [ARGS]...`: build a profile, dev by default, then run the
//! workspace's binary in the current directory with ARGS, its exit
//! code the task's.
//!
//! - The arguments parse as `cargo run`'s do: run's own flags first,
//!   and the binary's from the first word that is not one, a hyphen
//!   word included, so `cx run st -sb` needs no `--`. Run has no `-h`
//!   of its own, so `cx run -V` and `cx run --help` are the binary's,
//!   and `cx help run` is run's help.
//! - The words after run complete by the binary's own completion,
//!   [`forward_completion`] asking the binary it would run.

use std::path::Path;
use std::process::ExitCode;

use super::build::build;
use crate::util::{BinArgs, ProfileArgs, command, root};
use crate::workspace::Workspace;

/// The task's arguments.
#[derive(clap::Args, Debug)]
#[command(
    disable_help_flag = true,
    about = "Build a profile and run the workspace's binary, its arguments untouched",
    long_about = "\
        Build a Cargo profile, dev by default, as build does, then run the\n\
        binary, --bin's or the workspace's default, in the current directory\n\
        with ARGS, its exit code this task's.\n\n\
        The arguments are parsed as `cargo run`'s are: run's own flags come\n\
        first, and the binary's from the first word that is not one, so\n\
        `cargo xtask run st -sb` needs no `--`. Run has no -h or -V of its\n\
        own, so `cargo xtask run -V` is the binary's, and this is\n\
        `cargo xtask help run`. The words after run complete by the\n\
        binary's own completion, when it answers clap's."
)]
pub struct Args {
    #[command(flatten)]
    pub profile: ProfileArgs,

    #[command(flatten)]
    pub bin: BinArgs,

    /// The binary's arguments
    #[arg(
        value_name = "ARGS",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub args: Vec<String>,
}

/// Build, then run the binary, returning its exit code.
pub fn run(args: &Args) -> Result<ExitCode, String> {
    let root = root();
    let ws = Workspace::read(&root)?;
    let bin = ws.bin(args.bin.bin.as_deref())?;
    let path = build(&root, &ws, &args.profile.name("dev"), bin)?;
    let status = command(&path)
        .args(&args.args)
        .status()
        .map_err(|e| format!("'{}' did not run: {e}", path.display()))?;
    // OK: a binary killed by a signal has no code and fails as 1, as
    // does a code outside a u8, which no platform's exit status gives
    let code = status.code().and_then(|c| u8::try_from(c).ok());
    Ok(ExitCode::from(code.unwrap_or(1)))
}

/// A completion request run hands to the binary: the profile and the
/// binary it would run, the words for the binary, its own name first,
/// and the cursor's index among them when the shell gave one.
#[derive(Debug, PartialEq, Eq)]
pub struct Forward {
    /// The profile `--release` or `--profile` named, else `dev`.
    pub profile: String,
    /// The binary `--bin` named, else the workspace's default.
    pub bin: Option<String>,
    /// The words for the binary, the first standing for its name.
    pub words: Vec<String>,
    /// The cursor's index in `words`, when the shell passed one.
    pub index: Option<usize>,
}

/// Plan a completion forward from the words a shell passes, the
/// command's name first, and the cursor's index among them, or `None`
/// when the words are not run's arguments and clap completes them.
///
/// - Only `run` is forwarded, the first word after the name.
/// - Its own flags, `--release`, `--profile NAME`, `--profile=NAME`,
///   `--bin NAME`, and `--bin=NAME`, are skipped, and `--` ends them.
/// - The cursor on `run` or its flags is clap's, as is a word at the
///   cursor that starts with `-` before any argument, so `run --r<TAB>`
///   offers `--release`.
/// - The first word forwarded stands for the binary's name, `--bin`'s
///   or a stand-in, which [`forward_completion`] replaces with the name
///   of the binary it asks.
pub fn plan_forward(words: &[String], index: Option<usize>) -> Option<Forward> {
    // OK: with no index the shell completes the last word
    let cursor = index.unwrap_or(words.len().saturating_sub(1));
    if words.get(1).map(String::as_str) != Some("run") {
        return None;
    }
    let mut profile = "dev".to_string();
    let mut bin = None;
    let mut i = 2;
    while i < words.len() && i < cursor {
        let w = words[i].as_str();
        if w == "--" {
            i += 1;
            break;
        }
        if w == "--release" {
            profile = "release".to_string();
            i += 1;
        } else if w == "--profile" || w == "--bin" {
            let value = words.get(i + 1).cloned();
            match w {
                // OK: `--profile` with no value yet keeps the default
                "--profile" => profile = value.unwrap_or(profile),
                _ => bin = value,
            }
            i += 2;
        } else if let Some(name) = w.strip_prefix("--profile=") {
            profile = name.to_string();
            i += 1;
        } else if let Some(name) = w.strip_prefix("--bin=") {
            bin = Some(name.to_string());
            i += 1;
        } else {
            break;
        }
    }
    if cursor < i || (cursor == i && words[i].starts_with('-') && words[i - 1] != "--") {
        return None;
    }
    let name = bin.clone().unwrap_or_else(|| "bin".to_string());
    let mut forwarded = vec![name];
    forwarded.extend(words[i..].iter().cloned());
    Some(Forward {
        profile,
        bin,
        words: forwarded,
        index: index.map(|n| n - i + 1),
    })
}

/// Answer a completion request for run's arguments by asking the
/// binary it would run, in clap's completion mode, and exit with its
/// code. Anything else, and no request at all, returns, for clap to
/// complete. The binary is asked as it stands and never built, and one
/// that is not there, or that answers nothing, offers nothing.
pub fn forward_completion(root: &Path) {
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
    let found = Workspace::read(root).and_then(|ws| {
        let bin = ws.bin(plan.bin.as_deref())?;
        Ok((ws.binary(&plan.profile, bin), bin.name.clone()))
    });
    let code = match found {
        Ok((path, name)) if path.is_file() => {
            let mut cmd = command(&path);
            cmd.arg("--").arg(name).args(&plan.words[1..]);
            if let Some(index) = plan.index {
                cmd.env("_CLAP_COMPLETE_INDEX", index.to_string());
            }
            match cmd.status().map(|s| s.code()) {
                Ok(Some(code)) => code,
                _ => 0,
            }
        }
        _ => 0,
    };
    std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(s: &str) -> Vec<String> {
        s.split(' ').map(str::to_string).collect()
    }

    /// Only run's arguments are forwarded, its own flags skipped and
    /// the index shifted, and the cursor on `run`, its flags, or a flag
    /// before any argument is clap's.
    #[test]
    fn plan_forward_forms() {
        let p = plan_forward(&w("cx run st -s"), Some(3)).unwrap();
        assert_eq!(
            p,
            Forward {
                profile: "dev".to_string(),
                bin: None,
                words: w("bin st -s"),
                index: Some(2),
            }
        );
        let p = plan_forward(&w("cx run --release sy"), Some(3)).unwrap();
        assert_eq!(
            (p.profile.as_str(), p.words, p.index),
            ("release", w("bin sy"), Some(1))
        );
        let p = plan_forward(&w("cx run --profile soak st"), Some(4)).unwrap();
        assert_eq!((p.profile.as_str(), p.index), ("soak", Some(1)));
        let p = plan_forward(&w("cx run --profile=soak -- -"), Some(4)).unwrap();
        assert_eq!(
            (p.profile.as_str(), p.words, p.index),
            ("soak", w("bin -"), Some(1))
        );
        let p = plan_forward(&w("cx run --bin tool --release st"), Some(5)).unwrap();
        assert_eq!(
            (p.bin.as_deref(), p.profile.as_str(), p.words, p.index),
            (Some("tool"), "release", w("tool st"), Some(1))
        );
        let p = plan_forward(&w("cx run --bin=tool st"), Some(3)).unwrap();
        assert_eq!((p.bin.as_deref(), p.index), (Some("tool"), Some(1)));
        let p = plan_forward(&w("cx run st"), None).unwrap();
        assert_eq!((p.words, p.index), (w("bin st"), None));
        assert_eq!(
            plan_forward(&w("cx run --r"), Some(2)),
            None,
            "run's own flag"
        );
        assert_eq!(
            plan_forward(&w("cx run --profile "), Some(3)),
            None,
            "--profile's value"
        );
        assert_eq!(
            plan_forward(&w("cx run --bin "), Some(3)),
            None,
            "--bin's value"
        );
        assert_eq!(plan_forward(&w("cx ru"), Some(1)), None, "the task's name");
        assert_eq!(plan_forward(&w("cx bump n"), Some(2)), None, "another task");
    }
}

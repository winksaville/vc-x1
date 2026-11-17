//! `cargo xtask run [--release | --profile NAME] [ARGS]...`: build a
//! profile, dev by default, then run vc-x1 in the current directory
//! with ARGS, its exit code the task's.
//!
//! - The arguments parse as `cargo run`'s do: run's own flags first,
//!   and the binary's from the first word that is not one, a hyphen
//!   word included, so `cx run st -sb` needs no `--`. Run has no `-h`
//!   of its own, so `cx run -V` and `cx run --help` are vc-x1's, and
//!   `cx help run` is run's help.
//! - The words after run complete by vc-x1's own completion,
//!   [`forward_completion`] asking the binary it would run.

use std::path::Path;
use std::process::ExitCode;

use super::build::build;
use crate::util::{BIN, ProfileArgs, binary, command, root};

/// The task's arguments.
#[derive(clap::Args, Debug)]
#[command(
    disable_help_flag = true,
    about = "Build a profile and run vc-x1, its arguments untouched",
    long_about = "\
        Build a Cargo profile, dev by default, as build does, then run vc-x1\n\
        in the current directory with ARGS, its exit code this task's.\n\n\
        The arguments are parsed as `cargo run`'s are: run's own flags come\n\
        first, and the binary's from the first word that is not one, so\n\
        `cargo xtask run st -sb` needs no `--`. Run has no -h or -V of its\n\
        own, so `cargo xtask run -V` is vc-x1's, and this is\n\
        `cargo xtask help run`. The words after run complete by vc-x1's own\n\
        completion."
)]
pub struct Args {
    #[command(flatten)]
    pub profile: ProfileArgs,

    /// vc-x1's arguments
    #[arg(
        value_name = "ARGS",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub args: Vec<String>,
}

/// Build, then run the binary, returning its exit code.
pub fn run(args: &Args) -> Result<ExitCode, String> {
    let bin = build(&root(), &args.profile.name("dev"))?;
    let status = command(&bin)
        .args(&args.args)
        .status()
        .map_err(|e| format!("'{}' did not run: {e}", bin.display()))?;
    // OK: a binary killed by a signal has no code and fails as 1, as
    // does a code outside a u8, which no platform's exit status gives
    let code = status.code().and_then(|c| u8::try_from(c).ok());
    Ok(ExitCode::from(code.unwrap_or(1)))
}

/// A completion request run hands to the binary: the profile it would
/// run, the words for the binary, its own name first, and the cursor's
/// index among them when the shell gave one.
#[derive(Debug, PartialEq, Eq)]
pub struct Forward {
    /// The profile `--release` or `--profile` named, else `dev`.
    pub profile: String,
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
/// - Its own flags, `--release`, `--profile NAME`, and
///   `--profile=NAME`, are skipped, and `--` ends them.
/// - The cursor on `run` or its flags is clap's, as is a word at the
///   cursor that starts with `-` before any argument, so `run --r<TAB>`
///   offers `--release`.
pub fn plan_forward(words: &[String], index: Option<usize>) -> Option<Forward> {
    // OK: with no index the shell completes the last word
    let cursor = index.unwrap_or(words.len().saturating_sub(1));
    if words.get(1).map(String::as_str) != Some("run") {
        return None;
    }
    let mut profile = "dev".to_string();
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
        } else if w == "--profile" {
            if let Some(name) = words.get(i + 1) {
                profile = name.clone();
            }
            i += 2;
        } else if let Some(name) = w.strip_prefix("--profile=") {
            profile = name.to_string();
            i += 1;
        } else {
            break;
        }
    }
    if cursor < i || (cursor == i && words[i].starts_with('-') && words[i - 1] != "--") {
        return None;
    }
    let mut forwarded = vec![BIN.to_string()];
    forwarded.extend(words[i..].iter().cloned());
    Some(Forward {
        profile,
        words: forwarded,
        index: index.map(|n| n - i + 1),
    })
}

/// Answer a completion request for run's arguments by asking the
/// binary it would run, in clap's completion mode, and exit with its
/// code. Anything else, and no request at all, returns, for clap to
/// complete. The binary is asked as it stands and never built, and one
/// that is not there offers nothing.
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
    let code = match binary(root, &plan.profile) {
        Ok(bin) if bin.is_file() => {
            let mut cmd = command(&bin);
            cmd.arg("--").args(&plan.words);
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
                words: w("vc-x1 st -s"),
                index: Some(2),
            }
        );
        let p = plan_forward(&w("cx run --release sy"), Some(3)).unwrap();
        assert_eq!(
            (p.profile.as_str(), p.words, p.index),
            ("release", w("vc-x1 sy"), Some(1))
        );
        let p = plan_forward(&w("cx run --profile soak st"), Some(4)).unwrap();
        assert_eq!((p.profile.as_str(), p.index), ("soak", Some(1)));
        let p = plan_forward(&w("cx run --profile=soak -- -"), Some(4)).unwrap();
        assert_eq!(
            (p.profile.as_str(), p.words, p.index),
            ("soak", w("vc-x1 -"), Some(1))
        );
        let p = plan_forward(&w("cx run st"), None).unwrap();
        assert_eq!((p.words, p.index), (w("vc-x1 st"), None));
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
        assert_eq!(plan_forward(&w("cx ru"), Some(1)), None, "the task's name");
        assert_eq!(plan_forward(&w("cx bump n"), Some(2)), None, "another task");
    }
}

//! `cargo xtask`: the project's own tasks, outside `vc-x1`, run by the
//! alias in `.cargo/config.toml`.
//!
//! - [`tasks`]: the tasks, one module each, and how to add one.
//! - [`util`] and [`provenance`]: what the tasks share.
//!
//! This file parses the arguments and hands them to the task, and a
//! new task never changes it.

use std::process::ExitCode;

use clap::{CommandFactory, Parser};
use clap_complete::CompleteEnv;

mod provenance;
mod tasks;
mod util;

use tasks::{Task, dispatch};

/// The project's own tasks.
#[derive(Parser, Debug)]
#[command(name = "xtask", bin_name = "cargo xtask", max_term_width = 80)]
struct Cli {
    #[command(subcommand)]
    task: Task,
}

fn main() -> ExitCode {
    // With `COMPLETE` set these answer a completion request and exit,
    // run's arguments by the binary it runs and the rest by clap.
    tasks::run::forward_completion(&util::root());
    CompleteEnv::with_factory(Cli::command).complete();
    match dispatch(&Cli::parse().task) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("xtask: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The words the completion engine offers for `args`, the last of
    /// them the word being completed.
    fn completes(args: &[&str]) -> Vec<String> {
        let mut cmd = Cli::command();
        let words: Vec<std::ffi::OsString> = args.iter().map(Into::into).collect();
        clap_complete::engine::complete(&mut cmd, words, args.len() - 1, None)
            .expect("completes")
            .iter()
            .map(|c| c.get_value().to_string_lossy().into_owned())
            .collect()
    }

    /// The tasks complete by name, whatever the command was typed as,
    /// `bump` offers its two words, and `--profile` the profiles.
    #[test]
    fn the_tasks_and_bumps_words_complete() {
        let tasks = completes(&["cx", ""]);
        for task in [
            "version",
            "bump",
            "completions",
            "validate",
            "build",
            "run",
            "install",
        ] {
            assert!(tasks.iter().any(|t| t == task), "{task}: {tasks:?}");
        }
        assert_eq!(completes(&["cargo-xtask", "ve"]), ["version"]);
        assert_eq!(completes(&["cx", "bump", ""])[..2], ["next", "bare"]);
        assert_eq!(completes(&["cx", "bump", "n"]), ["next"]);
        assert_eq!(
            completes(&["cx", "build", "--profile", ""])[..2],
            ["dev", "release"]
        );
    }

    /// The arguments are clap's, `-h` is help, and each task's help is
    /// the one its `Args` carries.
    #[test]
    fn the_arguments_are_claps() {
        let parse = |args: &[&str]| Cli::try_parse_from(args).map(|c| c.task);
        assert!(matches!(parse(&["xtask", "version"]), Ok(Task::Version(_))));
        assert!(matches!(parse(&["xtask", "bump", "next"]), Ok(Task::Bump(a)) if a.to == "next"));
        let help = parse(&["xtask", "-h"]).unwrap_err();
        assert_eq!(help.kind(), clap::error::ErrorKind::DisplayHelp);
        let help = help.to_string();
        assert!(help.contains("cargo xtask"), "{help}");
        assert!(help.contains("Build a profile, dev by default"), "{help}");
        let help = parse(&["xtask", "help", "run"]).unwrap_err().to_string();
        assert!(help.contains("needs no `--`"), "{help}");
        assert!(parse(&["xtask", "bump"]).is_err(), "bump needs its word");
        assert!(parse(&["xtask"]).is_err(), "a task is required");
    }

    /// `run` parses as `cargo run` does: its own flags first, and the
    /// binary's from the first word that is not one, a hyphen word
    /// included, with no `-h` of its own.
    #[test]
    fn run_takes_the_binarys_arguments_as_cargo_run_does() {
        let run = |args: &[&str]| match Cli::try_parse_from(args).unwrap().task {
            Task::Run(a) => (a.profile.name("dev"), a.args),
            other => panic!("expected run, got {other:?}"),
        };
        let words = |s: &str| s.split(' ').map(str::to_string).collect::<Vec<_>>();
        assert_eq!(
            run(&["xtask", "run", "st", "-sb"]),
            ("dev".into(), words("st -sb"))
        );
        assert_eq!(
            run(&["xtask", "run", "--release", "st", "-sb"]),
            ("release".into(), words("st -sb"))
        );
        assert_eq!(
            run(&["xtask", "run", "--profile", "soak", "st"]),
            ("soak".into(), words("st"))
        );
        assert_eq!(run(&["xtask", "run", "-V"]), ("dev".into(), words("-V")));
        assert_eq!(
            run(&["xtask", "run", "--help"]),
            ("dev".into(), words("--help"))
        );
        assert_eq!(run(&["xtask", "run", "-h"]), ("dev".into(), words("-h")));
        assert_eq!(
            run(&["xtask", "run", "--", "--release"]),
            ("dev".into(), words("--release"))
        );
        assert_eq!(
            run(&["xtask", "run", "st", "--release"]),
            ("dev".into(), words("st --release"))
        );
        assert_eq!(run(&["xtask", "run"]), ("dev".into(), vec![]));
        assert!(Cli::try_parse_from(["xtask", "run", "--release", "--profile", "x"]).is_err());
    }

    /// Build and install take a profile, install's default the release.
    #[test]
    fn build_and_install_take_a_profile() {
        let parse = |args: &[&str]| Cli::try_parse_from(args).unwrap().task;
        match parse(&["xtask", "build"]) {
            Task::Build(a) => assert_eq!(a.profile.name("dev"), "dev"),
            other => panic!("{other:?}"),
        }
        match parse(&[
            "xtask",
            "install",
            "--profile",
            "dev",
            "--dry-run",
            "--dir",
            "d",
        ]) {
            Task::Install(a) => {
                assert_eq!(a.profile.name("release"), "dev");
                assert_eq!(a.dir, Some(std::path::PathBuf::from("d")));
                assert!(a.dry_run);
            }
            other => panic!("{other:?}"),
        }
        match parse(&["xtask", "install"]) {
            Task::Install(a) => assert_eq!(a.profile.name("release"), "release"),
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            parse(&["xtask", "validate", "--fast"]),
            Task::Validate(a) if a.fast
        ));
    }
}

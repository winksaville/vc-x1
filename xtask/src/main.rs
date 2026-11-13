//! `cargo xtask`: the project's own tasks, outside `vc-x1`.
//!
//! - `cargo xtask version` prints the version in `Cargo.toml`.
//! - `cargo xtask bump <next|bare|VERSION>` sets it: `next` adds one to
//!   the suffix's last number, `0.85.22-8` to `0.85.22-9`, and gives a
//!   bare version its first, `-0`, `bare` drops the suffix, a
//!   close-out's version, and anything else is the version as written.
//!   `Cargo.lock` is refreshed in the same step, since the build is
//!   `--locked`.
//! - `cargo xtask completions [NAME...]` prints a bash function that
//!   completes `cargo xtask` and each NAME, a shell alias for it, by
//!   asking the xtask of the repo the shell is in.
//!
//! The manifest is edited by `toml_edit`, which keeps its comments,
//! layout, and key order and changes the one value. The arguments are
//! clap's, whose completion engine answers when `COMPLETE` is set, as
//! `vc-x1`'s does.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::CompleteEnv;
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};
use toml_edit::{DocumentMut, value};

/// The project's own tasks.
#[derive(Parser, Debug)]
#[command(name = "xtask", bin_name = "cargo xtask", max_term_width = 80)]
struct Cli {
    #[command(subcommand)]
    task: Task,
}

/// A task.
#[derive(Subcommand, Debug)]
enum Task {
    /// Print the version in Cargo.toml
    Version,

    /// Set the version in Cargo.toml and refresh Cargo.lock
    #[command(long_about = "\
        Set the package version in Cargo.toml, its comments and layout kept,\n\
        and refresh Cargo.lock in the same step, the build being --locked.\n\n\
        `next` adds one to the suffix's last number, 0.85.22-8 to 0.85.22-9,\n\
        and gives a bare version its first, -0. `bare` drops the suffix, a\n\
        close-out's version. Anything else is the version as written, X.Y.Z\n\
        or X.Y.Z-<suffix>, with one `-` at most and no `+`.")]
    Bump {
        /// `next`, `bare`, or the version as written
        #[arg(value_name = "next|bare|VERSION", add = ArgValueCandidates::new(bump_candidates))]
        to: String,
    },

    /// Print a bash function that completes `cargo xtask` and its aliases
    #[command(long_about = "\
        Print a bash function that completes `cargo xtask`, and each NAME, a\n\
        shell alias for it such as `cx`. It holds no task names: on each tab\n\
        it asks the xtask of the repo the shell is in, so one copy in a\n\
        shell's startup file serves every repo whose xtask answers this way.\n\n\
        `cargo xtask <tab>` reaches it through cargo's own bash completion,\n\
        which hands a subcommand it does not know to the completion\n\
        registered for `cargo-<name>`, and needs bash-completion 2.12 or\n\
        later. A NAME is completed directly.\n\n\
        Save it once and source that file, the function being the same in\n\
        every repo:\n\
        \x20 cargo xtask completions cx > ~/.config/cargo-xtask-completion.bash")]
    Completions {
        /// A shell alias for `cargo xtask`, completed as it is
        #[arg(value_name = "NAME")]
        names: Vec<String>,
    },
}

/// What `bump`'s argument completes to, beside a version as written.
fn bump_candidates() -> Vec<CompletionCandidate> {
    vec![
        CompletionCandidate::new("next")
            .help(Some("one more than the suffix's last number".into())),
        CompletionCandidate::new("bare").help(Some("the version without its suffix".into())),
    ]
}

/// The bash function `completions` prints, registered for
/// `cargo-xtask`, the name cargo's completion hands `cargo xtask` to,
/// and for each of `names`.
///
/// It is clap's own registration script with one change: where that
/// calls one binary by name, this runs `cargo xtask`, so the answer is
/// the xtask's of the repo the shell is in. A repo with no xtask, or
/// one that fails, completes nothing.
fn completion_script(names: &[String]) -> String {
    let mut registered = vec!["cargo-xtask".to_string()];
    registered.extend(names.iter().cloned());
    format!(
        r#"# Completion for `cargo xtask`, printed by `cargo xtask completions`.
# It asks the xtask of the repo the shell is in, on each tab.
_cargo_xtask_complete() {{
    local IFS=$''
    local _CLAP_COMPLETE_INDEX=${{COMP_CWORD}}
    local _CLAP_COMPLETE_COMP_TYPE=${{COMP_TYPE}}
    if compopt +o nospace 2> /dev/null; then
        local _CLAP_COMPLETE_SPACE=false
    else
        local _CLAP_COMPLETE_SPACE=true
    fi
    local words=("${{COMP_WORDS[@]}}")
    if [[ "${{BASH_VERSINFO[0]}}" -ge 4 ]]; then
        words[COMP_CWORD]="$2"
    fi
    COMPREPLY=( $(         _CLAP_IFS="$IFS"         _CLAP_COMPLETE_INDEX="$_CLAP_COMPLETE_INDEX"         _CLAP_COMPLETE_COMP_TYPE="$_CLAP_COMPLETE_COMP_TYPE"         _CLAP_COMPLETE_SPACE="$_CLAP_COMPLETE_SPACE"         COMPLETE="bash"         cargo xtask -- "${{words[@]}}" 2> /dev/null     ) )
    if [[ $? != 0 ]]; then
        unset COMPREPLY
    elif [[ $_CLAP_COMPLETE_SPACE == false ]] && [[ "${{COMPREPLY-}}" =~ [=/:]$ ]]; then
        compopt -o nospace
    fi
}}
if [[ "${{BASH_VERSINFO[0]}}" -eq 4 && "${{BASH_VERSINFO[1]}}" -ge 4 || "${{BASH_VERSINFO[0]}}" -gt 4 ]]; then
    complete -o nospace -o bashdefault -o nosort -F _cargo_xtask_complete {names}
else
    complete -o nospace -o bashdefault -F _cargo_xtask_complete {names}
fi
"#,
        names = registered.join(" ")
    )
}

/// What `bump` is asked for.
#[derive(Debug, PartialEq, Eq)]
enum Bump {
    /// One more than the suffix's last number, or `-0` on a bare version.
    Next,
    /// The version without its suffix.
    Bare,
    /// The version as written.
    To(String),
}

/// The version `bump` gives `current`.
///
/// The version is `X.Y.Z` or `X.Y.Z-<suffix>`, the suffix dot-separated
/// identifiers with exactly one `-` in the whole version, as the
/// project's suffix scheme has it. Refused: `next` on a suffix whose
/// last identifier is not a number, and a version written with a `+`
/// or a second `-`.
fn bumped(current: &str, bump: &Bump) -> Result<String, String> {
    let (public, suffix) = match current.split_once('-') {
        Some((public, suffix)) => (public, Some(suffix)),
        None => (current, None),
    };
    match bump {
        Bump::Bare => Ok(public.to_string()),
        Bump::To(version) => {
            let well_formed = version.matches('-').count() <= 1
                && !version.contains('+')
                && !version.is_empty()
                && version
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || ".-".contains(c));
            match well_formed {
                true => Ok(version.clone()),
                false => Err(format!(
                    "'{version}' is not a version, `X.Y.Z` or `X.Y.Z-<suffix>` with one `-` and \
                     no `+`"
                )),
            }
        }
        Bump::Next => match suffix {
            None => Ok(format!("{public}-0")),
            Some(suffix) => {
                let (head, last) = match suffix.rsplit_once('.') {
                    Some((head, last)) => (Some(head), last),
                    None => (None, suffix),
                };
                let n: u64 = last.parse().map_err(|_| {
                    format!(
                        "the suffix of '{current}' ends '{last}', not a number, so `next` has \
                         nothing to add one to: name the version"
                    )
                })?;
                Ok(match head {
                    Some(head) => format!("{public}-{head}.{}", n + 1),
                    None => format!("{public}-{}", n + 1),
                })
            }
        },
    }
}

/// The package version in a manifest's text.
fn version_in(manifest: &str) -> Result<String, String> {
    let doc: DocumentMut = manifest
        .parse()
        .map_err(|e| format!("Cargo.toml does not parse: {e}"))?;
    doc.get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| "Cargo.toml has no `[package] version`".to_string())
}

/// A manifest's text with the package version set to `version`, the
/// rest as it was, comments and layout included.
fn with_version(manifest: &str, version: &str) -> Result<String, String> {
    let mut doc: DocumentMut = manifest
        .parse()
        .map_err(|e| format!("Cargo.toml does not parse: {e}"))?;
    let package = doc
        .get_mut("package")
        .ok_or_else(|| "Cargo.toml has no `[package]`".to_string())?;
    let slot = package
        .get_mut("version")
        .ok_or_else(|| "Cargo.toml has no `[package] version`".to_string())?;
    // The decor, a trailing comment on the line among it, stays.
    let decor = slot.as_value().map(|v| v.decor().clone());
    *slot = value(version);
    if let (Some(decor), Some(v)) = (decor, slot.as_value_mut()) {
        *v.decor_mut() = decor;
    }
    Ok(doc.to_string())
}

/// The workspace root, the directory above this package's.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// Refresh `Cargo.lock` for the workspace's own packages, offline, so
/// a `--locked` build finds the version the manifest now has.
fn refresh_lock(root: &Path) -> Result<(), String> {
    // `CARGO` is the cargo that runs this task, absent only when the
    // binary is run by hand.
    let cargo = match std::env::var_os("CARGO") {
        Some(cargo) => cargo,
        None => "cargo".into(),
    };
    // The root's clippy.toml bans spawning for `vc-x1`, which goes
    // through jj-lib. A project task running cargo is the exception.
    #[allow(clippy::disallowed_methods)]
    let status = std::process::Command::new(cargo)
        .args(["update", "--workspace", "--offline", "--quiet"])
        .current_dir(root)
        .status()
        .map_err(|e| format!("cargo update did not run: {e}"))?;
    match status.success() {
        true => Ok(()),
        false => Err(format!(
            "cargo update --workspace --offline failed: {status}"
        )),
    }
}

/// Run a task, returning what it prints.
fn run(task: &Task) -> Result<String, String> {
    let manifest = root().join("Cargo.toml");
    let read = || std::fs::read_to_string(&manifest).map_err(|e| format!("Cargo.toml: {e}"));
    match task {
        Task::Version => version_in(&read()?),
        Task::Bump { to } => {
            let bump = match to.as_str() {
                "next" => Bump::Next,
                "bare" => Bump::Bare,
                version => Bump::To(version.to_string()),
            };
            let text = read()?;
            let current = version_in(&text)?;
            let new = bumped(&current, &bump)?;
            if new == current {
                return Ok(format!("{current}, as it was"));
            }
            std::fs::write(&manifest, with_version(&text, &new)?)
                .map_err(|e| format!("Cargo.toml: {e}"))?;
            refresh_lock(&root())?;
            Ok(format!("{current} -> {new}, Cargo.lock refreshed"))
        }
        Task::Completions { names } => Ok(completion_script(names).trim_end().to_string()),
    }
}

fn main() -> ExitCode {
    // With `COMPLETE` set this answers a completion request and exits.
    CompleteEnv::with_factory(Cli::command).complete();
    match run(&Cli::parse().task) {
        Ok(said) => {
            println!("{said}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("xtask: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `next` walks the suffix, `bare` drops it, and a version named
    /// is taken as written when it is one.
    #[test]
    fn the_bump_follows_the_suffix_scheme() {
        let next = |v: &str| bumped(v, &Bump::Next);
        assert_eq!(next("0.85.22").unwrap(), "0.85.22-0");
        assert_eq!(next("0.85.22-0").unwrap(), "0.85.22-1");
        assert_eq!(next("0.85.22-9").unwrap(), "0.85.22-10");
        assert_eq!(next("0.85.22-3.1").unwrap(), "0.85.22-3.2");
        assert!(next("0.85.22-rc").unwrap_err().contains("not a number"));
        assert_eq!(bumped("0.85.22-12", &Bump::Bare).unwrap(), "0.85.22");
        assert_eq!(bumped("0.85.22", &Bump::Bare).unwrap(), "0.85.22");
        let to = |v: &str| bumped("0.85.22-1", &Bump::To(v.to_string()));
        assert_eq!(to("0.86.0-0").unwrap(), "0.86.0-0");
        for bad in ["", "1.0.0-1-2", "1.0.0+build", "1.0.0 "] {
            assert!(to(bad).is_err(), "{bad}");
        }
    }

    /// The version's value changes and nothing else does: the comments,
    /// the blank lines, the key order, and a dependency's own version.
    #[test]
    fn the_manifest_keeps_its_comments_and_layout() {
        let manifest = "\
[package]
name = \"vc-x1\"
version = \"0.85.22-8\" # the version-of-record
edition = \"2024\"

# A comment between tables.
[dependencies]
clap = { version = \"4.6\", features = [\"derive\"] }
# The rng jj-lib takes.
rand = \"0.10\"
";
        assert_eq!(version_in(manifest).unwrap(), "0.85.22-8");
        let new = with_version(manifest, "0.85.22-9").unwrap();
        assert_eq!(new, manifest.replace("0.85.22-8", "0.85.22-9"));
        assert!(with_version("[workspace]\n", "1.0.0").is_err());
    }

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
    /// and `bump` offers its two words.
    #[test]
    fn the_tasks_and_bumps_words_complete() {
        let tasks = completes(&["cx", ""]);
        for task in ["version", "bump", "completions"] {
            assert!(tasks.iter().any(|t| t == task), "{task}: {tasks:?}");
        }
        assert_eq!(completes(&["cargo-xtask", "v"]), ["version"]);
        assert_eq!(completes(&["cx", "bump", ""])[..2], ["next", "bare"]);
        assert_eq!(completes(&["cx", "bump", "n"]), ["next"]);
    }

    /// The arguments parse as the hand-written ones did, and `-h` is
    /// help and no longer a usage error.
    #[test]
    fn the_arguments_are_claps() {
        let parse = |args: &[&str]| Cli::try_parse_from(args).map(|c| c.task);
        assert!(matches!(parse(&["xtask", "version"]), Ok(Task::Version)));
        assert!(matches!(parse(&["xtask", "bump", "next"]), Ok(Task::Bump { to }) if to == "next"));
        let help = parse(&["xtask", "-h"]).unwrap_err();
        assert_eq!(help.kind(), clap::error::ErrorKind::DisplayHelp);
        assert!(help.to_string().contains("cargo xtask"), "{help}");
        assert!(parse(&["xtask", "bump"]).is_err(), "bump needs its word");
        assert!(parse(&["xtask"]).is_err(), "a task is required");
    }

    /// The function asks `cargo xtask`, not a binary by name, and is
    /// registered for cargo's hand-off name and each alias given.
    #[test]
    fn the_completion_function_asks_the_repos_xtask() {
        let script = completion_script(&["cx".to_string()]);
        assert!(
            script.contains("cargo xtask -- \"${words[@]}\" 2> /dev/null"),
            "{script}"
        );
        // Registered twice over, as clap's script is: unsorted where
        // bash can, 4.4 and later, so the tasks keep their order.
        assert!(
            script.contains("-o nosort -F _cargo_xtask_complete cargo-xtask cx\n"),
            "{script}"
        );
        assert!(
            script.ends_with("-F _cargo_xtask_complete cargo-xtask cx\nfi\n"),
            "{script}"
        );
        assert!(completion_script(&[]).ends_with("_cargo_xtask_complete cargo-xtask\nfi\n"));
    }
}

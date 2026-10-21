//! A configured command list: a TOML array in the work side's
//! config, each element one invocation, run in order from the work
//! repo root, stopping at the first failure.
//!
//! - `[validate] full` and `fast` are run by `validate`, and
//!   `[build] release` and `dev` by `build` and `build-dev`. Each caller passes
//!   its own label, which prefixes every message.
//! - Each command is printed before it runs, inherits the terminal,
//!   and runs from the root.
//! - The first non-zero exit stops the run, naming the command, its
//!   status, and where in the list it stopped.
//! - An empty or missing list is an error, not a silent pass: a
//!   list with nothing to run has done nothing.
//! - Elements are split on whitespace into a program and its
//!   arguments, with no shell in between, so a command's exit
//!   status is exactly the status the run sees.
//! - A caller may add environment variables to every command of
//!   the list, set on each child alone, never on vc-x1's own
//!   process: `build` passes the build's provenance this way.

use std::path::Path;

use log::info;

/// Read `key`'s command list from the work side's config at `root`.
///
/// Errors, each prefixed by `label` and naming the key, when the
/// config is missing or unloadable, the key is absent, the value is
/// not a proper array, or the array is empty.
pub fn load(root: &Path, key: &str, label: &str) -> Result<Vec<String>, String> {
    let cfg = crate::config_md::load(root)
        .map_err(|e| format!("{label}: {e}"))?
        .ok_or_else(|| format!("{label}: no config in '{}'", root.display()))?;
    let commands = crate::toml_simple::toml_get_list(&cfg.map, key)
        .map_err(|e| format!("{label}: {}: {e}", cfg.path.display()))?
        .ok_or_else(|| {
            format!(
                "{label}: {}: no `{key}` table. Add one, a TOML array with one command per \
                 element (see `vc-x1 config work`)",
                cfg.path.display()
            )
        })?;
    if commands.is_empty() {
        return Err(format!(
            "{label}: {}: `{key}` is empty, nothing to run is not a pass",
            cfg.path.display()
        ));
    }
    Ok(commands)
}

/// Run `commands` in order from `root`, printing each before it
/// runs, stopping at the first failure. `key` names the list in the
/// messages, `label` prefixes them, and `env` is added to each
/// command's environment.
pub fn run(
    root: &Path,
    key: &str,
    commands: &[String],
    env: &[(&str, &str)],
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let n = commands.len();
    for (i, command) in commands.iter().enumerate() {
        info!("{label}: [{}/{n}] {command}", i + 1);
        let status = run_one(root, command, env, label)?;
        if !status.success() {
            return Err(format!(
                "{label}: `{command}` failed ({}), stopping at {}/{n} of {key}",
                describe_status(status),
                i + 1,
            )
            .into());
        }
    }
    info!("{label}: {n} command(s) passed ({key})");
    Ok(())
}

/// Run one command line from `root`, inheriting stdio and adding
/// `env` to its environment, and return its exit status. The line
/// is split on whitespace, the first word the program, so no shell
/// sits between vc-x1 and the command's own status.
pub fn run_one(
    root: &Path,
    command: &str,
    env: &[(&str, &str)],
    label: &str,
) -> Result<std::process::ExitStatus, String> {
    let mut words = command.split_whitespace();
    let Some(program) = words.next() else {
        return Err(format!("{label}: an empty command element"));
    };
    // Register entry 5 (clippy.toml): a configured command list,
    // `[validate]`'s or `[build]`'s, is run as the user would
    // type it, which is the whole job of the commands that run one.
    #[allow(clippy::disallowed_methods)]
    let mut cmd = std::process::Command::new(program);
    cmd.args(words).current_dir(root).envs(env.iter().copied());
    cmd.status().map_err(|e| {
        format!(
            "{label}: cannot run `{command}` from '{}': {e}",
            root.display()
        )
    })
}

/// Render an exit status for the failure message: the code when
/// there is one, else the raw status (a signal on Unix).
fn describe_status(status: std::process::ExitStatus) -> String {
    match status.code() {
        Some(code) => format!("exit {code}"),
        None => format!("{status}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Unique tempdir for a one-file workspace.
    fn workspace(tag: &str, config: &str) -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("vc-x1-cmdlist-{tag}-{ts}"));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(".vc-config.md"), config).unwrap();
        root
    }

    /// A missing list and an empty list are errors naming the key
    /// and carrying the caller's label.
    #[test]
    fn missing_or_empty_list_errors() {
        let root = workspace("missing", "```toml\n[repos]\nwork = \".\"\n```\n");
        let e = load(&root, "validate.full", "validate").unwrap_err();
        assert!(e.starts_with("validate: "), "{e}");
        assert!(e.contains("no `validate.full` table"), "{e}");
        let root2 = workspace(
            "empty",
            "```toml\n[repos]\nwork = \".\"\n[build]\ndev = []\n```\n",
        );
        let e = load(&root2, "build.dev", "build").unwrap_err();
        assert!(e.starts_with("build: "), "{e}");
        assert!(e.contains("`build.dev` is empty"), "{e}");
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&root2).ok();
    }

    /// Commands run in order and the first failure stops the run,
    /// naming the command, its exit status, and the list.
    #[test]
    fn stops_at_first_failure() {
        let root = workspace("stop", "```toml\n[repos]\nwork = \".\"\n```\n");
        let commands = vec![
            "true".to_string(),
            "false".to_string(),
            "touch never.txt".to_string(),
        ];
        let e = run(&root, "validate.fast", &commands, &[], "validate")
            .unwrap_err()
            .to_string();
        assert!(e.contains("failed (exit 1)"), "{e}");
        assert!(e.contains("2/3 of validate.fast"), "{e}");
        assert!(!root.join("never.txt").exists(), "the run did not stop");
        std::fs::remove_dir_all(&root).ok();
    }

    /// All passing runs every command from the root.
    #[test]
    fn all_pass_runs_from_root() {
        let root = workspace("pass", "```toml\n[repos]\nwork = \".\"\n```\n");
        let commands = vec!["touch ran.txt".to_string(), "true".to_string()];
        run(&root, "validate.full", &commands, &[], "validate").unwrap();
        assert!(root.join("ran.txt").exists());
        std::fs::remove_dir_all(&root).ok();
    }

    /// The caller's variables reach each command's environment.
    #[test]
    fn env_reaches_the_commands() {
        let root = workspace("env", "```toml\n[repos]\nwork = \".\"\n```\n");
        // printenv exits 0 when the variable is set and 1 when not.
        let commands = vec!["printenv VC_X1_TEST_CMDLIST".to_string()];
        run(
            &root,
            "build.release",
            &commands,
            &[("VC_X1_TEST_CMDLIST", "x")],
            "build",
        )
        .unwrap();
        let e = run(&root, "build.release", &commands, &[], "install").unwrap_err();
        assert!(e.to_string().contains("failed (exit 1)"), "{e}");
        std::fs::remove_dir_all(&root).ok();
    }

    /// An unrunnable program is an error naming the command.
    #[test]
    fn unrunnable_program_errors() {
        let root = workspace("norun", "```toml\n[repos]\nwork = \".\"\n```\n");
        let e = run_one(&root, "no-such-program-xyz --flag", &[], "build").unwrap_err();
        assert!(e.starts_with("build: "), "{e}");
        assert!(e.contains("cannot run `no-such-program-xyz --flag`"), "{e}");
        std::fs::remove_dir_all(&root).ok();
    }
}

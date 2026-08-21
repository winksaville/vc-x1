//! The `validate` subcommand: run the workspace's configured
//! validation commands, in order, stopping at the first failure.
//!
//! - The commands come from the work side's `[validate]` table
//!   (`validate.full`, or `validate.fast` with `--fast`), each
//!   element one invocation, so the pinned checklists can name one
//!   command for every medium instead of pointing at prose.
//! - Each command is printed before it runs, inherits the terminal,
//!   and runs from the work repo root.
//! - The first non-zero exit stops the run, naming the command and
//!   its status, and the subcommand exits non-zero.
//! - An empty or missing table is an error, not a silent pass: a
//!   validator that finds nothing to run has validated nothing.
//! - Elements are split on whitespace into a program and its
//!   arguments, with no shell in between, so a command's exit
//!   status is exactly the status the run sees.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info};

use crate::context::Context;
use crate::subcommand::SubcommandRunner;

/// Run the workspace's configured validation commands.
///
/// Reads `[validate] full` (or `fast` with `--fast`) from the work
/// side's config and runs each element as one command, in order,
/// from the work repo root. Stops at the first failure.
#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// Run the `validate.fast` table instead of `validate.full`
    #[arg(long)]
    pub fast: bool,

    /// Workspace to validate [default: the workspace containing
    /// the current directory]
    #[arg(short = 'R', long, value_name = "DIR")]
    pub repo: Option<PathBuf>,
}

/// Per-invocation validate inputs: the table's key, its commands,
/// and the directory to run them from.
#[derive(Debug)]
pub struct ValidateParams {
    /// `validate.full` or `validate.fast`.
    pub key: &'static str,
    /// The commands, one element per invocation.
    pub commands: Vec<String>,
    /// The work repo root the commands run from.
    pub root: PathBuf,
}

impl TryFrom<&ValidateArgs> for ValidateParams {
    type Error = String;

    /// Resolve the workspace root (from `-R` or the cwd), load its
    /// config, and take the chosen table. An absent or empty table
    /// is the error here, before anything runs.
    fn try_from(a: &ValidateArgs) -> Result<Self, String> {
        let key = if a.fast {
            "validate.fast"
        } else {
            "validate.full"
        };
        let root = match &a.repo {
            Some(p) => crate::common::find_workspace_root_from(p).ok_or_else(|| {
                format!("validate: '{}' is not in a vc-x1 workspace", p.display())
            })?,
            None => crate::common::find_workspace_root().ok_or_else(|| {
                "validate: not in a vc-x1 workspace (no config found)".to_string()
            })?,
        };
        let commands = load_commands(&root, key)?;
        Ok(ValidateParams {
            key,
            commands,
            root,
        })
    }
}

/// Read `key`'s command list from the work side's config at `root`.
///
/// Errors, each naming the key, when the config is missing or
/// unloadable, the key is absent, the value is not a proper array,
/// or the array is empty.
fn load_commands(root: &Path, key: &str) -> Result<Vec<String>, String> {
    let cfg = crate::config_md::load(root)
        .map_err(|e| format!("validate: {e}"))?
        .ok_or_else(|| format!("validate: no config in '{}'", root.display()))?;
    let commands = crate::toml_simple::toml_get_list(&cfg.map, key)
        .map_err(|e| format!("validate: {}: {e}", cfg.path.display()))?
        .ok_or_else(|| {
            format!(
                "validate: {}: no `{key}` table. Add one, a TOML array with one command per \
                 element (see `vc-x1 config work`)",
                cfg.path.display()
            )
        })?;
    if commands.is_empty() {
        return Err(format!(
            "validate: {}: `{key}` is empty, nothing to run is not a pass",
            cfg.path.display()
        ));
    }
    Ok(commands)
}

impl SubcommandRunner for ValidateArgs {
    type Params = ValidateParams;

    /// Delegate to the `TryFrom<&ValidateArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        ValidateParams::try_from(self)
    }

    /// Run the `validate` op (`ctx` unused: the op is fully
    /// parameterized by `Params`).
    fn run(_ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        validate(params)
    }
}

/// Run the commands in order from the root, printing each before
/// it runs, stopping at the first failure.
pub fn validate(params: &ValidateParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("validate: entry params={params:?}");
    let n = params.commands.len();
    for (i, command) in params.commands.iter().enumerate() {
        info!("validate: [{}/{n}] {command}", i + 1);
        let status = run_one(&params.root, command)?;
        if !status.success() {
            return Err(format!(
                "validate: `{command}` failed ({}), stopping at {}/{n} of {}",
                describe_status(status),
                i + 1,
                params.key
            )
            .into());
        }
    }
    info!("validate: {n} command(s) passed ({})", params.key);
    Ok(())
}

/// Run one command line from `root`, inheriting stdio, and return
/// its exit status. The line is split on whitespace, the first
/// word the program, so no shell sits between vc-x1 and the
/// command's own status.
fn run_one(root: &Path, command: &str) -> Result<std::process::ExitStatus, String> {
    let mut words = command.split_whitespace();
    let Some(program) = words.next() else {
        return Err("validate: an empty command element".to_string());
    };
    // Allowlist entry 5 (clippy.toml): the validate subcommand runs
    // the workspace's configured validation commands, which is its
    // whole job, and each runs as the user would type it.
    #[allow(clippy::disallowed_methods)]
    let mut cmd = std::process::Command::new(program);
    cmd.args(words).current_dir(root);
    cmd.status().map_err(|e| {
        format!(
            "validate: cannot run `{command}` from '{}': {e}",
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
    use crate::{Cli, Commands};
    use clap::Parser;

    fn parse(args: &[&str]) -> ValidateArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::Validate(a)) => a,
            _ => panic!("expected Validate"),
        }
    }

    /// Unique tempdir for a one-file workspace.
    fn workspace(tag: &str, config: &str) -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("vc-x1-validate-{tag}-{ts}"));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(".vc-config.md"), config).unwrap();
        root
    }

    #[test]
    fn flags_parse() {
        let a = parse(&["vc-x1", "validate"]);
        assert!(!a.fast);
        let a = parse(&["vc-x1", "validate", "--fast", "-R", "x"]);
        assert!(a.fast);
        assert_eq!(a.repo, Some(PathBuf::from("x")));
    }

    /// A missing table and an empty table are errors naming the
    /// key, not silent passes.
    #[test]
    fn missing_or_empty_table_errors() {
        let root = workspace("missing", "```toml\n[repos]\nwork = \".\"\n```\n");
        let e = load_commands(&root, "validate.full").unwrap_err();
        assert!(e.contains("no `validate.full` table"), "{e}");
        let root2 = workspace(
            "empty",
            "```toml\n[repos]\nwork = \".\"\n[validate]\nfast = []\n```\n",
        );
        let e = load_commands(&root2, "validate.fast").unwrap_err();
        assert!(e.contains("`validate.fast` is empty"), "{e}");
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&root2).ok();
    }

    /// Commands run in order and the first failure stops the run,
    /// naming the command and its exit status.
    #[test]
    fn stops_at_first_failure() {
        let root = workspace("stop", "```toml\n[repos]\nwork = \".\"\n```\n");
        let params = ValidateParams {
            key: "validate.fast",
            commands: vec![
                "true".to_string(),
                "false".to_string(),
                "touch never.txt".to_string(),
            ],
            root: root.clone(),
        };
        let e = validate(&params).unwrap_err().to_string();
        assert!(e.contains("failed (exit 1)"), "{e}");
        assert!(e.contains("2/3"), "{e}");
        assert!(!root.join("never.txt").exists(), "the run did not stop");
        std::fs::remove_dir_all(&root).ok();
    }

    /// All passing runs every command from the root.
    #[test]
    fn all_pass_runs_from_root() {
        let root = workspace("pass", "```toml\n[repos]\nwork = \".\"\n```\n");
        let params = ValidateParams {
            key: "validate.full",
            commands: vec!["touch ran.txt".to_string(), "true".to_string()],
            root: root.clone(),
        };
        validate(&params).unwrap();
        assert!(root.join("ran.txt").exists());
        std::fs::remove_dir_all(&root).ok();
    }

    /// An unrunnable program is an error naming the command.
    #[test]
    fn unrunnable_program_errors() {
        let root = workspace("norun", "```toml\n[repos]\nwork = \".\"\n```\n");
        let e = run_one(&root, "no-such-program-xyz --flag").unwrap_err();
        assert!(e.contains("cannot run `no-such-program-xyz --flag`"), "{e}");
        std::fs::remove_dir_all(&root).ok();
    }
}

//! The `validate` subcommand: run the workspace's configured
//! validation commands, in order, stopping at the first failure.
//!
//! - The commands come from the work side's `[validate]` table
//!   (`validate.full`, or `validate.fast` with `--fast`), each
//!   element one invocation, so the pinned checklists can name one
//!   command for every medium instead of pointing at prose.
//! - How a list is read and run, printed, stopped at the first
//!   failure, refused when empty, and split with no shell, is
//!   [`crate::command_list`]'s, which `build` shares.
//! - The commands run with the build's provenance in their
//!   environment, as `build`'s do, so the test build compiles with the
//!   same commit and state as the dev build that follows it, which then
//!   reuses it rather than recompiling.
//! - Validation checks the source and makes no build of any profile:
//!   the dev build is `build --profile dev`'s, which
//!   `run --profile dev` runs first.

use std::path::PathBuf;

use clap::Args;
use log::debug;

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
    /// The build's provenance, best effort: the commands run without
    /// it when it cannot be read.
    pub provenance: Option<crate::build_cmd::Provenance>,
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
        let commands = crate::command_list::load(&root, key, "validate")?;
        let provenance = crate::build_cmd::Provenance::read_or_warn(&root, "validate");
        Ok(ValidateParams {
            key,
            commands,
            root,
            provenance,
        })
    }
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
    let env = crate::build_cmd::build_env(params.provenance.as_ref());
    crate::command_list::run(
        &params.root,
        params.key,
        &params.commands,
        &env,
        &[],
        &[],
        "validate",
    )
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
            provenance: None,
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
            provenance: None,
        };
        validate(&params).unwrap();
        assert!(root.join("ran.txt").exists());
        std::fs::remove_dir_all(&root).ok();
    }
}

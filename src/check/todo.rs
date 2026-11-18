//! The `check todo` subcommand: check that the `## Todo` and
//! `## Bugs` sections of a todo file are numbered `1..N` in
//! document order, with continuation-line indent matching each
//! entry's number-prefix width.
//!
//! Read-only diagnostic; exits non-zero if any entry needs
//! fixing. The check half of the `check todo` / `fix-todo`
//! pair: `fix-todo` is the rewrite.

use std::path::PathBuf;

use clap::Args;
use log::{debug, info};

use crate::context::Context;
use crate::subcommand::SubcommandRunner;
use crate::todo_helpers::{self, Section, TODO_FILE};

/// Clap-derived args for `check todo`.
#[derive(Args, Debug)]
pub struct CheckTodoArgs {
    /// Retired: the file, now `--file`, refused naming it.
    #[arg(value_name = "RETIRED", hide = true)]
    pub retired: Option<String>,

    /// Todo markdown file to check
    #[arg(long, value_name = "PATH", default_value = TODO_FILE)]
    pub file: PathBuf,
}

/// Inputs to the check todo op, flat, owned, clap-free.
///
/// Mirrors `CheckTodoArgs`: the `--file` path.
pub struct CheckTodoParams {
    pub file: PathBuf,
}

impl From<&CheckTodoArgs> for CheckTodoParams {
    /// Convert clap-derived `CheckTodoArgs` into the flat
    /// `CheckTodoParams` (total, the single field copies over).
    fn from(a: &CheckTodoArgs) -> Self {
        Self {
            file: a.file.clone(),
        }
    }
}

impl SubcommandRunner for CheckTodoArgs {
    type Params = CheckTodoParams;

    /// Delegate to the `From<&CheckTodoArgs>` impl (total).
    fn to_params(&self) -> Result<Self::Params, String> {
        if let Some(value) = &self.retired {
            return Err(crate::options_flags::retired_positional::refusal(
                "check todo",
                value,
                "--file",
            ));
        }
        Ok(CheckTodoParams::from(self))
    }

    /// Run the `check_todo` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        check_todo(ctx, params)
    }
}

/// Run the `check todo` subcommand: scan the todo file and
/// report each `## Todo` / `## Bugs` entry whose number or
/// continuation indent is off; errors if any are found.
///
/// `ctx` is unused: check todo reads a plain file and neither
/// the user config nor the `--log` path applies; it's present for
/// the uniform subcommand-layer signature.
pub fn check_todo(
    _ctx: &Context,
    params: &CheckTodoParams,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("check todo: enter");
    let content = std::fs::read_to_string(&params.file)
        .map_err(|e| format!("cannot read {}: {e}", params.file.display()))?;
    let analysis = todo_helpers::analyze(&content);

    info!("check todo: {}", params.file.display());

    let mut last_section: Option<Section> = None;
    for c in &analysis.changes {
        if last_section != Some(c.section) {
            info!("");
            info!("{}", c.section.header());
            last_section = Some(c.section);
        }
        info!("  {}  {}", c.new_first_line, todo_helpers::change_tag(c));
    }

    info!("");
    let total = analysis.todo_count + analysis.bugs_count;
    let n = analysis.changes.len();
    if n == 0 {
        info!(
            "{total} {} checked ({} Todo, {} Bugs), all sequential",
            todo_helpers::entry_word(total),
            analysis.todo_count,
            analysis.bugs_count
        );
        debug!("check todo: exit");
        Ok(())
    } else {
        info!(
            "{total} {} checked ({} Todo, {} Bugs), {n} to fix",
            todo_helpers::entry_word(total),
            analysis.todo_count,
            analysis.bugs_count
        );
        debug!("check todo: exit with issues");
        Err(format!(
            "{n} todo {} to fix, run `vc-x1 fix-todo`",
            todo_helpers::entry_word(n)
        )
        .into())
    }
}

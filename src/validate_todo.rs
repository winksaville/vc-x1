//! The `validate-todo` subcommand: check that the `## Todo` and
//! `## Bugs` sections of a todo file are numbered `1..N` in
//! document order, with continuation-line indent matching each
//! entry's number-prefix width.
//!
//! Read-only diagnostic; exits non-zero if any entry needs
//! fixing. The check half of the `validate-todo` / `fix-todo`
//! pair — `fix-todo` is the rewrite.

use std::path::PathBuf;

use clap::Args;
use log::{debug, info};

use crate::context::Context;
use crate::subcommand::SubcommandRunner;
use crate::todo_helpers::{self, Section, TODO_FILE};

/// Clap-derived args for `validate-todo`.
#[derive(Args, Debug)]
pub struct ValidateTodoArgs {
    /// Todo markdown file to check
    #[arg(value_name = "FILE", default_value = TODO_FILE)]
    pub file: PathBuf,
}

/// Inputs to the validate-todo op, flat, owned, clap-free.
///
/// Mirrors `ValidateTodoArgs`: the positional `FILE` path.
pub struct ValidateTodoParams {
    pub file: PathBuf,
}

impl From<&ValidateTodoArgs> for ValidateTodoParams {
    /// Convert clap-derived `ValidateTodoArgs` into the flat
    /// `ValidateTodoParams` (total — the single field copies over).
    fn from(a: &ValidateTodoArgs) -> Self {
        Self {
            file: a.file.clone(),
        }
    }
}

impl SubcommandRunner for ValidateTodoArgs {
    type Params = ValidateTodoParams;

    /// Delegate to the `From<&ValidateTodoArgs>` impl (total).
    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(ValidateTodoParams::from(self))
    }

    /// Run the `validate_todo` op.
    fn run(ctx: &Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        validate_todo(ctx, params)
    }
}

/// Run the `validate-todo` subcommand: scan the todo file and
/// report each `## Todo` / `## Bugs` entry whose number or
/// continuation indent is off; errors if any are found.
///
/// `ctx` is unused — validate-todo reads a plain file and neither
/// the user config nor the `--log` path applies; it's present for
/// the uniform subcommand-layer signature.
pub fn validate_todo(
    _ctx: &Context,
    params: &ValidateTodoParams,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("validate-todo: enter");
    let content = std::fs::read_to_string(&params.file)
        .map_err(|e| format!("cannot read {}: {e}", params.file.display()))?;
    let analysis = todo_helpers::analyze(&content);

    info!("validate-todo: {}", params.file.display());

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
            "{total} {} checked ({} Todo, {} Bugs) — all sequential",
            todo_helpers::entry_word(total),
            analysis.todo_count,
            analysis.bugs_count
        );
        debug!("validate-todo: exit");
        Ok(())
    } else {
        info!(
            "{total} {} checked ({} Todo, {} Bugs) — {n} to fix",
            todo_helpers::entry_word(total),
            analysis.todo_count,
            analysis.bugs_count
        );
        debug!("validate-todo: exit with issues");
        Err(format!(
            "{n} todo {} to fix — run `vc-x1 fix-todo`",
            todo_helpers::entry_word(n)
        )
        .into())
    }
}

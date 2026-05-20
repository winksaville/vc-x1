//! The `fix-todo` subcommand: renumber the `## Todo` / `## Bugs`
//! sections of a todo file to `1..N` and normalize each entry's
//! continuation-line indent.
//!
//! Dry-run by default — prints each changed entry's corrected
//! line so the output *is* the result; `--no-dry-run` writes the
//! file in place. The rewrite half of the `validate-todo` /
//! `fix-todo` pair.

use std::path::PathBuf;

use clap::Args;
use log::{debug, info};

use crate::context::Context;
use crate::subcommand::SubcommandRunner;
use crate::todo_helpers::{self, Section, TODO_FILE};

/// Clap-derived args for `fix-todo`.
#[derive(Args, Debug)]
pub struct FixTodoArgs {
    /// Todo markdown file to renumber
    #[arg(value_name = "FILE", default_value = TODO_FILE)]
    pub file: PathBuf,

    /// Write the renumbered file in place [default: dry-run]
    #[arg(long = "no-dry-run")]
    pub no_dry_run: bool,
}

/// Inputs to the fix-todo op, flat, owned, clap-free.
///
/// Mirrors `FixTodoArgs`: the positional `FILE` path and the
/// `--no-dry-run` write toggle.
pub struct FixTodoParams {
    pub file: PathBuf,
    pub no_dry_run: bool,
}

impl From<&FixTodoArgs> for FixTodoParams {
    /// Convert clap-derived `FixTodoArgs` into the flat
    /// `FixTodoParams` (total — every field copies straight over).
    fn from(a: &FixTodoArgs) -> Self {
        Self {
            file: a.file.clone(),
            no_dry_run: a.no_dry_run,
        }
    }
}

impl SubcommandRunner for FixTodoArgs {
    type Params = FixTodoParams;

    /// Delegate to the `From<&FixTodoArgs>` impl (total).
    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(FixTodoParams::from(self))
    }

    /// Run the `fix_todo` op.
    fn run(ctx: &Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        fix_todo(ctx, params)
    }
}

/// Run the `fix-todo` subcommand: renumber the todo file's
/// `## Todo` / `## Bugs` sections and normalize continuation
/// indent. Dry-run prints the corrected line of each changed
/// entry; `--no-dry-run` writes the file.
///
/// `ctx` is unused — fix-todo reads and writes a plain file and
/// neither the user config nor the `--log` path applies; it's
/// present for the uniform subcommand-layer signature.
pub fn fix_todo(_ctx: &Context, params: &FixTodoParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("fix-todo: enter");
    let content = std::fs::read_to_string(&params.file)
        .map_err(|e| format!("cannot read {}: {e}", params.file.display()))?;
    let analysis = todo_helpers::analyze(&content);

    info!("fix-todo: {}", params.file.display());

    let total = analysis.todo_count + analysis.bugs_count;
    if analysis.changes.is_empty() {
        info!(
            "{total} {} checked ({} Todo, {} Bugs) — already normalized",
            todo_helpers::entry_word(total),
            analysis.todo_count,
            analysis.bugs_count
        );
        debug!("fix-todo: exit");
        return Ok(());
    }

    // Print the corrected line of every changed entry — the
    // output is the result, not a description of it.
    let mut last_section: Option<Section> = None;
    for c in &analysis.changes {
        if last_section != Some(c.section) {
            info!("");
            info!("{}", c.section.header());
            last_section = Some(c.section);
        }
        info!("  {}  {}", c.new_first_line, todo_helpers::change_tag(c));
    }

    let n = analysis.changes.len();
    info!("");
    if params.no_dry_run {
        std::fs::write(&params.file, &analysis.fixed)
            .map_err(|e| format!("cannot write {}: {e}", params.file.display()))?;
        info!(
            "{n} {} renumbered — wrote {}",
            todo_helpers::entry_word(n),
            params.file.display()
        );
    } else {
        info!(
            "{n} {} to renumber — re-run with --no-dry-run to apply",
            todo_helpers::entry_word(n)
        );
    }
    debug!("fix-todo: exit");
    Ok(())
}

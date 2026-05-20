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

/// Truncate `line` to a single-line snippet for the report,
/// counting by `char` so a multi-byte character is never split.
fn snippet(line: &str) -> String {
    const MAX: usize = 56;
    let chars: Vec<char> = line.chars().collect();
    if chars.len() <= MAX {
        line.to_string()
    } else {
        let head: String = chars[..MAX].iter().collect();
        format!("{head}…")
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
        let mut parts: Vec<String> = Vec::new();
        if c.num_old != c.num_new {
            parts.push(format!("number {} → {}", c.num_old, c.num_new));
        }
        if let (Some(o), Some(n)) = (c.indent_old, c.indent_new)
            && o != n
        {
            parts.push(format!("indent {o} → {n}"));
        }
        info!(
            "  line {}: {} — {}",
            c.line_no,
            parts.join(", "),
            snippet(&c.first_line)
        );
    }

    info!("");
    let total = analysis.todo_count + analysis.bugs_count;
    if analysis.changes.is_empty() {
        info!(
            "{total} entries checked ({} Todo, {} Bugs) — all sequential",
            analysis.todo_count, analysis.bugs_count
        );
        debug!("validate-todo: exit");
        Ok(())
    } else {
        info!(
            "{total} entries checked ({} Todo, {} Bugs) — {} need fixing",
            analysis.todo_count,
            analysis.bugs_count,
            analysis.changes.len()
        );
        debug!("validate-todo: exit with issues");
        Err(format!(
            "{} todo entry/entries need fixing — run `vc-x1 fix-todo`",
            analysis.changes.len()
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A line within the limit is returned unchanged.
    #[test]
    fn snippet_keeps_short_lines() {
        assert_eq!(snippet("1. short entry"), "1. short entry");
    }

    /// A long line is cut on a `char` boundary and gets an
    /// ellipsis — 56 chars plus `…`.
    #[test]
    fn snippet_truncates_on_char_boundary() {
        let s = snippet(&"→".repeat(100));
        assert_eq!(s.chars().count(), 57);
        assert!(s.ends_with('…'));
    }
}

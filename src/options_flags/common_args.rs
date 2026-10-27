//! `CommonArgs`: the shared arg set for the read-only commit-query
//! subcommands (`chid` / `desc` / `list` / `show`): `-r`/`--revision`,
//! `-R`/`--repo`, `-s`/`--scope`, `-n`/`--commits`, and the `-l`/`-L`
//! inter-repo label pair, with the retired `REVISION` / `COMMITS`
//! positionals kept hidden to be refused.
//!
//! A "bundle" in the inline-fields sense (cf. `provision_bundle`'s
//! flatten-of-leaves form): the fields aren't extracted into per-flag
//! leaves because none of them is reused outside these four. If one
//! ever is (e.g. a future command wants just `--revision`), extract
//! that field into a leaf and `#[command(flatten)]` it here. Consumers
//! flatten this and add their own extra `#[arg]` fields (`list`'s
//! `-w`/`--width`, `show`'s `-f`/`--files`). See
//! [options_flags](README.md) for shared architecture.

use std::path::PathBuf;

use clap::Args;

use super::scope::{Scope, parse_scope};

/// Shared CLI args for the read-only commit-query subcommands:
/// see [Bundle](README.md#architecture).
///
/// - `retired_rev` / `retired_count`: the retired `REVISION` /
///   `COMMITS` positionals, hidden, refused by `refuse_retired`.
/// - `revision`: `-r` / `--revision` (default `@`). `..` notation is
///   parsed downstream by `common::parse_dot_rev`.
/// - `repo` / `scope`: `-R PATH` overrides the workspace root, `-s
///   work|agent|both` selects sides. They compose
///   (`common::resolve_repos`). Defaults preserve today's behavior:
///   no flag -> `[.]`, `-R foo` alone -> `[foo]`. `-s` alone resolves
///   against `find_workspace_root()`. `-R + -s` resolves against the
///   `-R` path. `-s` is keyword-only (`work|agent|both`, or the spelled-out
///   `work,agent|agent,work`).
///   Path-based single-repo operation routes through `-R`.
/// - `limit`: `-n` / `--commits`, caps the output.
/// - `label` / `no_label`: `-l` / `--label` (default `===`) and
///   `-L` / `--no-label`. `common::resolve_header` combines them.
#[derive(Args, Debug)]
pub struct CommonArgs {
    /// Retired: the revision, now `-r`, refused naming it.
    #[arg(value_name = "RETIRED", hide = true)]
    pub retired_rev: Option<String>,

    /// Retired: the count, now `-n`, refused naming it, kept as typed.
    #[arg(value_name = "RETIRED", hide = true)]
    pub retired_count: Option<String>,

    /// Revision to query, with optional .. notation
    #[arg(short, long, default_value = "@")]
    pub revision: String,

    /// Workspace root or single jj repo path [default: .]
    #[arg(short = 'R', long = "repo", value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Side(s) to query, and composes with --repo as workspace root
    #[arg(
        short = 's',
        long = "scope",
        value_name = "work|agent|both",
        value_parser = parse_scope
    )]
    pub scope: Option<Scope>,

    /// Number of commits to show
    #[arg(short = 'n', long = "commits", value_name = "COMMITS")]
    pub limit: Option<usize>,

    /// Custom label decoration between repos
    #[arg(
        short = 'l',
        long = "label",
        value_name = "TEXT",
        allow_hyphen_values = true,
        default_value = "==="
    )]
    pub label: String,

    /// Suppress label between repos
    #[arg(short = 'L', long = "no-label")]
    pub no_label: bool,
}

impl CommonArgs {
    /// Refuse the retired `[REVISION] [COMMITS]` pair on `command`,
    /// naming `-r` and `-n` with the values in place.
    pub fn refuse_retired(&self, command: &str) -> Result<(), String> {
        super::retired_positional::refuse_rev_count(
            command,
            self.retired_rev.as_deref(),
            self.retired_count.as_deref(),
        )
    }

    /// Resolve `-R` + `-s` into the concrete repo paths to iterate
    /// (delegates to [`crate::common::resolve_repos`]).
    ///
    /// Bundles the `.as_deref()` / `.as_ref()` conversion ceremony
    /// (`Option<PathBuf>` -> `Option<&Path>`,
    /// `Option<Scope>` -> `Option<&Scope>`) into one well-named place
    /// so the four subcommand bodies stay clean. See
    /// [`../../notes/rust-idioms.md`](../../notes/rust-idioms.md)
    /// for why the two fields need different conversion methods
    /// (`PathBuf: Deref<Target = Path>`, while `Scope` is a plain enum).
    pub fn resolve_repos(&self) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>> {
        crate::common::resolve_repos(self.repo.as_deref(), self.scope.as_ref())
    }
}

impl super::OptionFlagBundle for CommonArgs {}

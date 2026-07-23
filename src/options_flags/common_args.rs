//! `CommonArgs` — the shared arg set for the read-only commit-query
//! subcommands (`chid` / `desc` / `list` / `show`): a `REVISION` /
//! `COMMITS` positional pair, `-r`/`--revision`, `-R`/`--repo`,
//! `-s`/`--scope`, `-n`/`--commits`, and the `-l`/`-L` inter-repo
//! label pair.
//!
//! A "bundle" in the inline-fields sense (cf. `provision_bundle`'s
//! flatten-of-leaves form): the fields aren't extracted into per-flag
//! leaves because none of them is reused outside these four — if one
//! ever is (e.g. a future command wants just `--revision`), extract
//! that field into a leaf and `#[command(flatten)]` it here. Consumers
//! flatten this and add their own extra `#[arg]` fields (`list`'s
//! `-w`/`--width`, `show`'s `-f`/`--files`). See
//! [options_flags](README.md) for shared architecture.

use std::path::PathBuf;

use clap::Args;

use super::scope::{Scope, parse_scope};

/// Shared CLI args for the read-only commit-query subcommands —
/// see [Bundle](README.md#architecture).
///
/// - `pos_rev` / `pos_count` — the `REVISION` / `COMMITS` positionals
///   (`common::resolve_spec` reconciles them with `-r` / `-n`).
/// - `revision` — `-r` / `--revision` (default `@`); `..` notation is
///   parsed downstream by `common::parse_dot_rev`.
/// - `repo` / `scope` — `-R PATH` overrides the workspace root, `-s
///   work|bot|work,bot` selects sides; they compose
///   (`common::resolve_repos`). Defaults preserve today's behavior:
///   no flag → `[.]`, `-R foo` alone → `[foo]`. `-s` alone resolves
///   against `find_workspace_root()`; `-R + -s` resolves against the
///   `-R` path. `-s` is keyword-only (`work|bot|work,bot|bot,work`);
///   path-based single-repo operation routes through `-R`.
/// - `limit` — `-n` / `--commits`, caps the output.
/// - `label` / `no_label` — `-l` / `--label` (default `===`) and
///   `-L` / `--no-label`; `common::resolve_header` combines them.
#[derive(Args, Debug)]
pub struct CommonArgs {
    /// Revision (with optional .. notation)
    #[arg(value_name = "REVISION")]
    pub pos_rev: Option<String>,

    /// Number of commits to show (per open side)
    #[arg(value_name = "COMMITS")]
    pub pos_count: Option<usize>,

    /// Revision to query
    #[arg(short, long, default_value = "@")]
    pub revision: String,

    /// Workspace root or single jj repo path [default: .]
    #[arg(short = 'R', long = "repo", value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Side(s) to query; composes with --repo as workspace root
    #[arg(
        short = 's',
        long = "scope",
        value_name = "work|bot|work,bot",
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
    /// Resolve `-R` + `-s` into the concrete repo paths to iterate
    /// (delegates to [`crate::common::resolve_repos`]).
    ///
    /// Bundles the `.as_deref()` / `.as_ref()` conversion ceremony
    /// (`Option<PathBuf>` → `Option<&Path>`,
    /// `Option<Scope>` → `Option<&Scope>`) into one well-named place
    /// so the four subcommand bodies stay clean. See
    /// [`../../notes/rust-idioms.md`](../../notes/rust-idioms.md)
    /// for why the two fields need different conversion methods
    /// (`PathBuf: Deref<Target = Path>`; `Scope` is a plain enum).
    pub fn resolve_repos(&self) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>> {
        crate::common::resolve_repos(self.repo.as_deref(), self.scope.as_ref())
    }
}

impl super::OptionFlagBundle for CommonArgs {}

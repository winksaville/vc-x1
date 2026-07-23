//! The `revert` subcommand: restore repos to their persisted
//! pre-sync snapshots.
//!
//! Sync stops on error and leaves state in place for inspection
//! (see `sync`); each synced repo keeps its pre-sync `jj op` id in
//! `.vc-x1/sync-state.toml`. `revert` is the explicit undo: it
//! resolves the same `-R`/`--scope` invocation shape as sync,
//! `jj op restore`s every repo that has a snapshot, and clears the
//! consumed state files. Repos without a snapshot are skipped with
//! a note; finding none at all is an error ("nothing to revert").

use std::path::PathBuf;

use clap::Args;
use log::info;

use crate::context::Context;
use crate::options_flags::scope::{Scope, parse_scope};
use crate::subcommand::SubcommandRunner;
use crate::sync::state;
use crate::sync::{op_restore, resolve_repos};

/// CLI args for `vc-x1 revert`.
#[derive(Args, Debug)]
pub struct RevertArgs {
    /// Workspace root, or a single jj repo to revert on its own.
    ///
    /// - `-R PATH` alone — revert just the repo at PATH.
    /// - `-R PATH -s ROLES` — use PATH as the workspace root and
    ///   revert the named side(s).
    #[arg(short = 'R', long = "repo", value_name = "PATH", verbatim_doc_comment)]
    pub repo: Option<PathBuf>,

    /// Which repo(s) of the workspace to revert.
    ///
    /// `SCOPE=code|bot|code,bot` — same resolution as `sync`, so a
    /// failed `vc-x1 sync` and the following `vc-x1 revert` name
    /// the same repos when invoked the same way.
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = parse_scope,
        verbatim_doc_comment
    )]
    pub scope: Option<Scope>,
}

/// Inputs to the revert op, flat, owned, clap-free.
///
/// - `repo`: `-R/--repo` path (None ⇒ discover the workspace
///   root from cwd).
/// - `scope`: `--scope` parsed (None ⇒ workspace-default scope).
pub struct RevertParams {
    pub repo: Option<PathBuf>,
    pub scope: Option<Scope>,
}

impl From<&RevertArgs> for RevertParams {
    /// Convert clap-derived `RevertArgs` into the flat
    /// `RevertParams` (total — every field copies straight over).
    fn from(a: &RevertArgs) -> Self {
        Self {
            repo: a.repo.clone(),
            scope: a.scope.clone(),
        }
    }
}

impl SubcommandRunner for RevertArgs {
    type Params = RevertParams;

    /// Delegate to the existing `From<&RevertArgs>` impl above
    /// (total — never fails).
    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(RevertParams::from(self))
    }

    /// Run the revert op.
    fn run(ctx: &Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        revert(ctx, params)
    }
}

/// CLI entry point for the `revert` subcommand.
///
/// Thin wrapper over `revert_repos` that resolves `-R`/`--scope`
/// exactly the way `sync` does. `ctx` is unused today — present
/// for the uniform subcommand-layer signature.
pub fn revert(_ctx: &Context, params: &RevertParams) -> Result<(), Box<dyn std::error::Error>> {
    let repos = resolve_repos(&params.repo, &params.scope)?;
    revert_repos(&repos)
}

/// Restore every repo in `repos` that has a persisted pre-sync
/// snapshot; clear each consumed state file.
///
/// - A repo without a snapshot is skipped with a note — sync
///   clears state on success, so "no snapshot" is the normal
///   post-success condition, not an error per repo.
/// - No snapshot in *any* repo is an error: the user asked to
///   revert something that isn't there.
/// - A restore failure stops immediately (stop-on-error, like
///   sync); already-restored repos keep their cleared state,
///   remaining repos keep their snapshots for a re-run.
pub fn revert_repos(repos: &[PathBuf]) -> Result<(), Box<dyn std::error::Error>> {
    let mut reverted = 0usize;
    for repo in repos {
        match state::load(repo)? {
            Some(st) => {
                info!(
                    "{}: jj op restore {} (pre-sync snapshot of '{}' @ {}, taken {})",
                    repo.display(),
                    st.op_id,
                    st.bookmark,
                    st.remote,
                    st.started_at
                );
                op_restore(repo, &st.op_id)?;
                state::clear(repo)?;
                reverted += 1;
            }
            None => {
                info!("{}: no sync snapshot — skipping", repo.display());
            }
        }
    }
    if reverted == 0 {
        return Err("revert: no sync snapshots found — nothing to revert".into());
    }
    let noun = if reverted == 1 { "repo" } else { "repos" };
    info!("revert: {reverted} {noun} restored");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options_flags::scope::Side;
    use clap::Parser;

    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: RevertArgs,
    }

    /// Defaults: no `-R`, no `--scope` — workspace-default
    /// resolution at run time.
    #[test]
    fn parse_defaults() {
        let cli = Cli::try_parse_from(["test"]).unwrap();
        assert!(cli.args.repo.is_none());
        assert!(cli.args.scope.is_none());
    }

    /// `-R PATH` and `-s SCOPE` parse and flow through to params.
    #[test]
    fn parse_repo_and_scope() {
        let cli = Cli::try_parse_from(["test", "-R", "./solo", "-s", "code"]).unwrap();
        let params = RevertParams::from(&cli.args);
        assert_eq!(params.repo, Some(PathBuf::from("./solo")));
        assert_eq!(params.scope, Some(Scope(vec![Side::Work])));
    }
}

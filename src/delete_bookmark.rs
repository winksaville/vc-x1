//! The `delete-bookmark` subcommand: delete one bookmark locally and
//! on `origin`, the disposal a cycle's bookmark gets at Land, in one
//! invocation rather than `jj bookmark delete` and a `jj git push`.
//!
//! - The remote goes first, then the local, so a push that fails
//!   leaves the local bookmark, and a rerun finishes the job: a side
//!   already gone is no error.
//! - `--skip local` deletes the remote only, `--skip remote` the local
//!   only.
//! - The commits stay: jj keeps a commit visible after its bookmark is
//!   gone, so a deletion is undone by recreating the bookmark at the
//!   commit it prints.
//! - `-s w` deletes in the work repo, `-s a` in the agent repo, `-s b`
//!   in both, and `-s p` in the repo `-R` names, as it is.
//! - A failure restores the repo from a `jj op` snapshot when the
//!   remote was untouched.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info, warn};

use crate::context::Context;
use crate::jj;
use crate::options_flags::dry_run::DryRunFlag;
use crate::options_flags::scope::{
    ALL_REPO_SCOPES, REPO_SCOPE_FORMS, RepoScope, Side, repo_scope_parser,
};
use crate::pair::resolve_pair;
use crate::subcommand::SubcommandRunner;

/// Which side of a bookmark `--skip` leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Keep {
    /// The local bookmark is kept, only `origin`'s deleted.
    Local,
    /// The remote bookmark is kept, only the local one deleted.
    Remote,
}

/// Delete a bookmark locally and on `origin`.
#[derive(Args, Debug)]
pub struct DeleteBookmarkArgs {
    /// The bookmark to delete
    #[arg(long, value_name = "NAME", required = true)]
    pub bookmark: String,

    /// Keep one side: `local` deletes only the remote, `remote` only
    /// the local
    #[arg(long, value_enum, value_name = "SIDE")]
    pub skip: Option<Keep>,

    /// Where: a path in the workspace for `-s w|a|b`, or the repo
    /// itself for `-s p`, required there [default: the current
    /// directory]
    #[arg(short = 'R', long, value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Which repos, required
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = repo_scope_parser(ALL_REPO_SCOPES),
        required = true,
        help = format!("Which repos, required: {REPO_SCOPE_FORMS}, `p` with -R")
    )]
    pub scope: RepoScope,

    /// Print the deletions it would make, and change nothing
    #[command(flatten)]
    pub dry_run: DryRunFlag,
}

/// Inputs to the delete-bookmark op, flat, owned, clap-free.
#[derive(Debug)]
pub struct DeleteBookmarkParams {
    pub repos: Vec<PathBuf>,
    pub bookmark: String,
    pub skip: Option<Keep>,
    pub dry_run: bool,
}

impl TryFrom<&DeleteBookmarkArgs> for DeleteBookmarkParams {
    type Error = String;

    /// Resolve the scope to its repos: `-R` as it is for `-s p`, the
    /// pair's sides for `-s w|a|b`.
    fn try_from(a: &DeleteBookmarkArgs) -> Result<Self, String> {
        let repos = match &a.scope {
            RepoScope::Por => match &a.repo {
                Some(repo) => vec![repo.clone()],
                None => {
                    return Err("delete-bookmark -s p: -R is required, the repo to delete \
                                the bookmark in"
                        .into());
                }
            },
            RepoScope::Sides(sides) => {
                let pair = resolve_pair(a.repo.as_deref()).map_err(|e| e.to_string())?;
                sides
                    .0
                    .iter()
                    .map(|side| match side {
                        Side::Work => pair.work.clone(),
                        Side::Bot => pair.agent.clone(),
                    })
                    .collect()
            }
        };
        Ok(DeleteBookmarkParams {
            repos,
            bookmark: a.bookmark.clone(),
            skip: a.skip,
            dry_run: a.dry_run.value,
        })
    }
}

impl SubcommandRunner for DeleteBookmarkArgs {
    type Params = DeleteBookmarkParams;

    /// Delegate to the `TryFrom<&DeleteBookmarkArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        DeleteBookmarkParams::try_from(self)
    }

    /// Run the `delete-bookmark` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        debug!("delete-bookmark: enter {params:?}");
        for repo in &params.repos {
            delete_bookmark(ctx, repo, &params.bookmark, params.skip, params.dry_run)?;
        }
        Ok(())
    }
}

/// Delete `name` in `repo`, the remote then the local, `skip` keeping
/// one side, printing the commit it was at so the deletion can be
/// undone. A side already gone is no error, and a bookmark gone on
/// both sides is reported and left.
pub fn delete_bookmark(
    ctx: &mut Context,
    repo: &Path,
    name: &str,
    skip: Option<Keep>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let local = jj::local_bookmark_exists(repo, name)?;
    let remote = jj::remote_bookmark_targets(repo, "origin")?
        .get(name)
        .cloned();
    let at = if local {
        Some(jj::cid_of(repo, name)?)
    } else {
        remote.clone()
    };
    let (del_remote, del_local) = (
        remote.is_some() && skip != Some(Keep::Remote),
        local && skip != Some(Keep::Local),
    );
    if !del_remote && !del_local {
        warn!(
            "delete-bookmark: '{name}' has nothing to delete in {}, as asked",
            repo.display()
        );
        return Ok(());
    }
    let at = at.unwrap_or_default();
    let dry = if dry_run { "[dry-run] " } else { "" };
    if del_remote {
        info!("delete-bookmark: {dry}delete '{name}' on origin, at {at}");
    }
    if del_local {
        info!("delete-bookmark: {dry}delete '{name}' locally, at {at}");
    }
    if dry_run {
        return Ok(());
    }
    if del_remote {
        ctx.session(repo)?.git_push_delete_bookmark(name)?;
    }
    if del_local {
        ctx.session(repo)?.bookmark_delete(name)?;
    }
    info!("delete-bookmark: done, `jj bookmark set {name} -r {at}` would bring it back");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{Fixture, jj_ok, test_ctx};

    /// Where `name` is, locally and on origin, as commit ids.
    fn state(repo: &Path, name: &str) -> (bool, bool) {
        (
            jj::local_bookmark_exists(repo, name).expect("local"),
            jj::remote_bookmark_targets(repo, "origin")
                .expect("remote")
                .contains_key(name),
        )
    }

    /// A published bookmark in the fixture's work repo.
    fn published(fx: &Fixture, name: &str) {
        jj_ok(&fx.work, &["git", "push", "--named", &format!("{name}=@-")]);
        assert_eq!(state(&fx.work, name), (true, true));
    }

    /// Both sides go by default, `--skip` keeps the one it names, a
    /// side already gone is no error, and a dry run changes nothing.
    #[test]
    fn a_bookmark_is_deleted_locally_and_on_origin() {
        let fx = Fixture::new("delete-bookmark");
        published(&fx, "both");
        delete_bookmark(&mut test_ctx(), &fx.work, "both", None, true).expect("dry run");
        assert_eq!(
            state(&fx.work, "both"),
            (true, true),
            "a dry run changes nothing"
        );
        delete_bookmark(&mut test_ctx(), &fx.work, "both", None, false).expect("deleted");
        assert_eq!(state(&fx.work, "both"), (false, false));
        delete_bookmark(&mut test_ctx(), &fx.work, "both", None, false).expect("gone already");

        published(&fx, "keep-local");
        delete_bookmark(
            &mut test_ctx(),
            &fx.work,
            "keep-local",
            Some(Keep::Local),
            false,
        )
        .expect("remote deleted");
        assert_eq!(state(&fx.work, "keep-local"), (true, false));

        published(&fx, "keep-remote");
        delete_bookmark(
            &mut test_ctx(),
            &fx.work,
            "keep-remote",
            Some(Keep::Remote),
            false,
        )
        .expect("local deleted");
        assert_eq!(state(&fx.work, "keep-remote"), (false, true));
    }
}

//! `list` subcommand — list commits in a jj repo, one per line:
//! changeID + ochid (padded) + bookmarks + title, the anchor bolded.
//!
//! - `ListArgs`: clap surface; flattens
//!   `options_flags::common_args::CommonArgs` plus a `-w`/`--width`
//!   for the ochid column.
//! - `ListParams`: clap-free; embeds `common::CommonParams` + `width`.
//!   Built via `TryFrom<&ListArgs>` at the binary edge.
//! - `list(&Context, &ListParams)`: the op — `for_each_repo` +
//!   `format_commit_with_ochid`.

use clap::Args;
use jj_lib::repo::Repo;
use log::{debug, info, trace, warn};

use crate::common::{self, CommonParams};
use crate::context::Context;
use crate::options_flags::common_args::CommonArgs;

/// CLI args for `list` — the shared read-only commit-query args plus
/// the ochid column width.
#[derive(Args, Debug)]
pub struct ListArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    /// ochid column width
    #[arg(short = 'w', long = "width", default_value_t = DEFAULT_OCHID_WIDTH)]
    pub width: usize,
}

const DEFAULT_OCHID_WIDTH: usize = 21;

/// Clap-free params for `list`; embeds `CommonParams` and the ochid
/// column width.
#[derive(Debug)]
pub struct ListParams {
    pub common: CommonParams,
    pub width: usize,
}

impl TryFrom<&ListArgs> for ListParams {
    type Error = String;

    /// Resolve clap `ListArgs` into `ListParams`: delegate to
    /// `CommonParams::try_from` for the shared fields; copy `width`
    /// straight over (clap-applied default already resolved).
    fn try_from(a: &ListArgs) -> Result<Self, String> {
        Ok(ListParams {
            common: CommonParams::try_from(&a.common)?,
            width: a.width,
        })
    }
}

/// List commits in the resolved range with the ochid column.
///
/// `_ctx` is unused (list has no user-config or `--log` consumer);
/// it's present for the uniform subcommand-layer signature.
pub fn list(_ctx: &Context, params: &ListParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("list: enter");
    let c = &params.common;
    trace!(
        "list: spec rev={} desc={:?} anc={:?}",
        c.spec.rev, c.spec.desc_count, c.spec.anc_count
    );

    common::for_each_repo(&c.repos, &c.header, |workspace, repo| {
        let (ids, anchor_index) = common::collect_ids(
            workspace,
            repo,
            &c.spec.rev,
            c.spec.desc_count,
            c.spec.anc_count,
        )?;
        debug!("list: {} commits, anchor at {anchor_index}", ids.len());

        if ids.is_empty() {
            warn!("list: no commits found for revision '{}'", c.spec.rev);
        }

        for (i, commit_id) in ids.iter().enumerate() {
            let commit = repo.store().get_commit(commit_id)?;
            let bookmarks = common::format_bookmarks_at(repo, commit_id);
            let line = common::format_commit_with_ochid(&commit, params.width, &bookmarks);
            if i == anchor_index {
                info!("{}", common::bold(&line));
            } else {
                info!("{line}");
            }
        }
        Ok(())
    })?;
    debug!("list: exit");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::Parser;

    use crate::{Cli, Commands};

    fn parse(args: &[&str]) -> super::ListArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::List(a) => a,
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn defaults() {
        let args = parse(&["vc-x1", "list"]);
        assert_eq!(args.common.revision, "@");
        assert_eq!(args.common.repo, None);
        assert_eq!(args.common.scope, None);
        assert!(args.common.limit.is_none());
        assert_eq!(args.width, super::DEFAULT_OCHID_WIDTH);
    }

    #[test]
    fn with_revision() {
        let args = parse(&["vc-x1", "list", "-r", "@-"]);
        assert_eq!(args.common.revision, "@-");
    }

    #[test]
    fn with_repo() {
        let args = parse(&["vc-x1", "list", "-R", "/some/path"]);
        assert_eq!(args.common.repo, Some(PathBuf::from("/some/path")));
    }

    #[test]
    fn with_scope_bot() {
        use crate::scope::{Scope, Side};
        let args = parse(&["vc-x1", "list", "-s", "bot"]);
        assert_eq!(args.common.scope, Some(Scope::Roles(vec![Side::Bot])));
    }

    #[test]
    fn with_limit() {
        let args = parse(&["vc-x1", "list", "-n", "5"]);
        assert_eq!(args.common.limit, Some(5));
    }

    #[test]
    fn all_opts() {
        let args = parse(&["vc-x1", "list", "-r", "all()", "-R", ".claude", "-n", "10"]);
        assert_eq!(args.common.revision, "all()");
        assert_eq!(args.common.repo, Some(PathBuf::from(".claude")));
        assert_eq!(args.common.limit, Some(10));
    }

    #[test]
    fn positional_rev() {
        let args = parse(&["vc-x1", "list", "@-"]);
        assert_eq!(args.common.pos_rev, Some("@-".to_string()));
        assert_eq!(args.common.pos_count, None);
    }

    #[test]
    fn positional_rev_and_count() {
        let args = parse(&["vc-x1", "list", "@..", "5"]);
        assert_eq!(args.common.pos_rev, Some("@..".to_string()));
        assert_eq!(args.common.pos_count, Some(5));
    }

    #[test]
    fn positional_both_dots() {
        let args = parse(&["vc-x1", "list", "..abcd..", "3"]);
        assert_eq!(args.common.pos_rev, Some("..abcd..".to_string()));
        assert_eq!(args.common.pos_count, Some(3));
    }

    #[test]
    fn custom_width() {
        let args = parse(&["vc-x1", "list", "-w", "30"]);
        assert_eq!(args.width, 30);
    }

    #[test]
    fn params_from_args_defaults() {
        // ListParams::try_from goes through the binary-edge resolution
        // and copies `width` straight over from the clap default.
        use super::{DEFAULT_OCHID_WIDTH, ListParams};
        let args = parse(&["vc-x1", "list"]);
        let params = ListParams::try_from(&args).unwrap();
        assert_eq!(params.common.repos, vec![PathBuf::from(".")]);
        assert_eq!(params.common.spec.rev, "@");
        assert_eq!(params.width, DEFAULT_OCHID_WIDTH);
    }

    #[test]
    fn params_from_args_with_width() {
        use super::ListParams;
        let args = parse(&["vc-x1", "list", "-w", "30"]);
        let params = ListParams::try_from(&args).unwrap();
        assert_eq!(params.width, 30);
    }
}

//! `desc` subcommand — print each commit in a revision range with
//! its full description (changeID + commitID + bookmarks + title,
//! then the body), the anchor commit bolded.
//!
//! - `DescArgs`: clap surface; flattens
//!   `options_flags::common_args::CommonArgs`.
//! - `DescParams`: clap-free; embeds `common::CommonParams`. Built
//!   via `TryFrom<&DescArgs>` at the binary edge.
//! - `desc(&Context, &DescParams)`: the op — `for_each_repo` +
//!   `format_commit_full` + `indent_body`.

use clap::Args;
use jj_lib::repo::Repo;
use log::{debug, info};

use crate::common::{self, CommonParams};
use crate::context::Context;
use crate::options_flags::common_args::CommonArgs;
use crate::subcommand::SubcommandRunner;

/// CLI args for `desc` — just the shared read-only commit-query args.
#[derive(Args, Debug)]
pub struct DescArgs {
    #[command(flatten)]
    pub common: CommonArgs,
}

/// Clap-free params for `desc`; embeds resolved `CommonParams`.
#[derive(Debug)]
pub struct DescParams {
    pub common: CommonParams,
}

impl TryFrom<&DescArgs> for DescParams {
    type Error = String;

    /// Resolve clap `DescArgs` into `DescParams` by delegating to
    /// `CommonParams::try_from`; `desc` has no fields beyond
    /// `CommonArgs`.
    fn try_from(a: &DescArgs) -> Result<Self, String> {
        Ok(DescParams {
            common: CommonParams::try_from(&a.common)?,
        })
    }
}

impl SubcommandRunner for DescArgs {
    type Params = DescParams;

    /// Delegate to the existing `TryFrom<&DescArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        DescParams::try_from(self)
    }

    /// Run the existing `desc` op.
    fn run(ctx: &Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        desc(ctx, params)
    }
}

/// Print each commit in the resolved range with its full description.
///
/// `_ctx` is unused (desc has no user-config or `--log` consumer);
/// it's present for the uniform subcommand-layer signature.
pub fn desc(_ctx: &Context, params: &DescParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("desc: enter");
    let c = &params.common;
    common::for_each_repo(&c.repos, &c.header, |workspace, repo| {
        let (ids, anchor_index) = common::collect_ids(
            workspace,
            repo,
            &c.spec.rev,
            c.spec.desc_count,
            c.spec.anc_count,
        )?;
        for (i, commit_id) in ids.iter().enumerate() {
            let commit = repo.store().get_commit(commit_id)?;
            let bookmarks = common::format_bookmarks_at(repo, commit_id);
            let line = common::format_commit_full(&commit, &bookmarks);
            let indented = common::indent_body(&line, 4);
            if i == anchor_index {
                info!("{}", common::bold_first_line(&indented));
            } else {
                info!("{indented}");
            }
        }
        Ok(())
    })?;
    debug!("desc: exit");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::Parser;

    use crate::options_flags::common_args::CommonArgs;
    use crate::{Cli, Commands};

    fn parse(args: &[&str]) -> CommonArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::Desc(a)) => a.common,
            _ => panic!("expected Desc"),
        }
    }

    #[test]
    fn defaults() {
        let c = parse(&["vc-x1", "desc"]);
        assert_eq!(c.revision, "@");
        assert_eq!(c.repo, None);
        assert_eq!(c.scope, None);
    }

    #[test]
    fn with_revision() {
        let c = parse(&["vc-x1", "desc", "-r", "wmuxkqwu"]);
        assert_eq!(c.revision, "wmuxkqwu");
    }

    #[test]
    fn with_repo() {
        let c = parse(&["vc-x1", "desc", "-R", "/tmp"]);
        assert_eq!(c.repo, Some(PathBuf::from("/tmp")));
    }

    #[test]
    fn with_scope_code_bot() {
        use crate::options_flags::scope::{Scope, Side};
        let c = parse(&["vc-x1", "desc", "-s", "code,bot"]);
        assert_eq!(c.scope, Some(Scope(vec![Side::Code, Side::Bot])));
    }

    #[test]
    fn with_limit() {
        let c = parse(&["vc-x1", "desc", "-n", "3"]);
        assert_eq!(c.limit, Some(3));
    }

    #[test]
    fn positional_rev() {
        let c = parse(&["vc-x1", "desc", "@-"]);
        assert_eq!(c.pos_rev, Some("@-".to_string()));
        assert_eq!(c.pos_count, None);
    }

    #[test]
    fn positional_rev_and_count() {
        let c = parse(&["vc-x1", "desc", "@..", "5"]);
        assert_eq!(c.pos_rev, Some("@..".to_string()));
        assert_eq!(c.pos_count, Some(5));
    }

    #[test]
    fn positional_both_dots() {
        let c = parse(&["vc-x1", "desc", "..abcd..", "3"]);
        assert_eq!(c.pos_rev, Some("..abcd..".to_string()));
        assert_eq!(c.pos_count, Some(3));
    }

    #[test]
    fn all_opts() {
        let c = parse(&["vc-x1", "desc", "-r", "@-", "-R", ".claude", "-n", "5"]);
        assert_eq!(c.revision, "@-");
        assert_eq!(c.repo, Some(PathBuf::from(".claude")));
        assert_eq!(c.limit, Some(5));
    }

    #[test]
    fn params_from_args_defaults() {
        // DescParams::try_from goes through the binary-edge resolution:
        // resolve_spec + resolve_header + resolve_repos.
        use super::DescParams;
        let cli = Cli::try_parse_from(["vc-x1", "desc"]).unwrap();
        let args = match cli.command {
            Some(Commands::Desc(a)) => a,
            _ => panic!("expected Desc"),
        };
        let params = DescParams::try_from(&args).unwrap();
        // default: no flags → repos resolves to [.]
        assert_eq!(params.common.repos, vec![PathBuf::from(".")]);
        // default revision @
        assert_eq!(params.common.spec.rev, "@");
    }
}

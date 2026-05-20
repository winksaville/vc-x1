//! `chid` subcommand — print the short change ID of each commit in a
//! revision range, one per line (script-friendly output).
//!
//! - `ChidArgs`: clap surface; flattens
//!   `options_flags::common_args::CommonArgs`.
//! - `ChidParams`: clap-free; embeds `common::CommonParams`. Built
//!   via `TryFrom<&ChidArgs>` at the binary edge.
//! - `chid(&Context, &ChidParams)`: the op — `for_each_repo` + print
//!   `format_chid`.

use clap::Args;
use jj_lib::repo::Repo;
use log::{debug, info};

use crate::common::{self, CommonParams};
use crate::context::Context;
use crate::options_flags::common_args::CommonArgs;
use crate::subcommand::SubcommandRunner;

/// CLI args for `chid` — just the shared read-only commit-query args.
#[derive(Args, Debug)]
pub struct ChidArgs {
    #[command(flatten)]
    pub common: CommonArgs,
}

/// Clap-free params for `chid`; embeds resolved `CommonParams`.
#[derive(Debug)]
pub struct ChidParams {
    pub common: CommonParams,
}

impl TryFrom<&ChidArgs> for ChidParams {
    type Error = String;

    /// Resolve clap `ChidArgs` into `ChidParams` by delegating to
    /// `CommonParams::try_from`; `chid` has no fields beyond
    /// `CommonArgs`.
    fn try_from(a: &ChidArgs) -> Result<Self, String> {
        Ok(ChidParams {
            common: CommonParams::try_from(&a.common)?,
        })
    }
}

/// Print the short change ID of each commit in the resolved range.
///
/// `_ctx` is unused (chid has no user-config or `--log` consumer);
/// it's present for the uniform subcommand-layer signature.
pub fn chid(_ctx: &Context, params: &ChidParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("chid: enter");
    let c = &params.common;
    common::for_each_repo(&c.repos, &c.header, |workspace, repo| {
        let (ids, _) = common::collect_ids(
            workspace,
            repo,
            &c.spec.rev,
            c.spec.desc_count,
            c.spec.anc_count,
        )?;
        for commit_id in &ids {
            let commit = repo.store().get_commit(commit_id)?;
            info!("{}", common::format_chid(&commit));
        }
        Ok(())
    })?;
    debug!("chid: exit");
    Ok(())
}

impl SubcommandRunner for ChidArgs {
    type Params = ChidParams;

    /// Delegate to the existing `TryFrom<&ChidArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        ChidParams::try_from(self)
    }

    /// Run the existing `chid` op.
    fn run(ctx: &Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        chid(ctx, params)
    }
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
            Some(Commands::Chid(a)) => a.common,
            _ => panic!("expected Chid"),
        }
    }

    #[test]
    fn defaults() {
        let c = parse(&["vc-x1", "chid"]);
        assert_eq!(c.revision, "@");
        assert_eq!(c.repo, None);
        assert_eq!(c.scope, None);
        assert_eq!(c.limit, None);
    }

    #[test]
    fn with_revision() {
        let c = parse(&["vc-x1", "chid", "-r", "@-"]);
        assert_eq!(c.revision, "@-");
    }

    #[test]
    fn with_repo() {
        let c = parse(&["vc-x1", "chid", "-R", ".claude"]);
        assert_eq!(c.repo, Some(PathBuf::from(".claude")));
    }

    #[test]
    fn with_scope_code() {
        use crate::options_flags::scope::{Scope, Side};
        let c = parse(&["vc-x1", "chid", "-s", "code"]);
        assert_eq!(c.scope, Some(Scope(vec![Side::Code])));
    }

    #[test]
    fn with_scope_code_bot() {
        use crate::options_flags::scope::{Scope, Side};
        let c = parse(&["vc-x1", "chid", "-s", "code,bot"]);
        assert_eq!(c.scope, Some(Scope(vec![Side::Code, Side::Bot])));
    }

    #[test]
    fn with_repo_and_scope_compose() {
        // `-R` and `-s` compose: the path is the workspace root, the
        // roles are resolved within it. Both fields parse cleanly.
        use crate::options_flags::scope::{Scope, Side};
        let c = parse(&["vc-x1", "chid", "-R", "../foo", "-s", "bot"]);
        assert_eq!(c.repo, Some(PathBuf::from("../foo")));
        assert_eq!(c.scope, Some(Scope(vec![Side::Bot])));
    }

    #[test]
    fn scope_path_rejected_with_hint() {
        // Path-via-`-s` is a planned feature; today rejected with a
        // hint pointing at `-R`/`--repo`.
        let err = crate::Cli::try_parse_from(["vc-x1", "chid", "-s", "./foo"])
            .unwrap_err()
            .to_string();
        assert!(err.contains("path"), "got: {err}");
        assert!(err.contains("--repo"), "got: {err}");
    }

    #[test]
    fn with_limit() {
        let c = parse(&["vc-x1", "chid", "-n", "5"]);
        assert_eq!(c.limit, Some(5));
    }

    #[test]
    fn all_opts() {
        let c = parse(&["vc-x1", "chid", "-r", "@--", "-R", ".claude", "-n", "3"]);
        assert_eq!(c.revision, "@--");
        assert_eq!(c.repo, Some(PathBuf::from(".claude")));
        assert_eq!(c.limit, Some(3));
    }

    #[test]
    fn label_default() {
        let c = parse(&["vc-x1", "chid"]);
        assert_eq!(c.label, "===");
        assert!(!c.no_label);
    }

    #[test]
    fn label_custom() {
        let c = parse(&["vc-x1", "chid", "-l", "---"]);
        assert_eq!(c.label, "---");
    }

    #[test]
    fn no_label() {
        let c = parse(&["vc-x1", "chid", "-L"]);
        assert!(c.no_label);
    }

    #[test]
    fn positional_rev() {
        let c = parse(&["vc-x1", "chid", "@-"]);
        assert_eq!(c.pos_rev, Some("@-".to_string()));
        assert_eq!(c.pos_count, None);
    }

    #[test]
    fn positional_rev_and_count() {
        let c = parse(&["vc-x1", "chid", "@..", "5"]);
        assert_eq!(c.pos_rev, Some("@..".to_string()));
        assert_eq!(c.pos_count, Some(5));
    }

    #[test]
    fn positional_both_dots() {
        let c = parse(&["vc-x1", "chid", "..abcd..", "3"]);
        assert_eq!(c.pos_rev, Some("..abcd..".to_string()));
        assert_eq!(c.pos_count, Some(3));
    }

    #[test]
    fn params_from_args_defaults() {
        // ChidParams::try_from goes through the binary-edge resolution:
        // resolve_spec + resolve_header + resolve_repos.
        use super::ChidParams;
        let cli = Cli::try_parse_from(["vc-x1", "chid"]).unwrap();
        let args = match cli.command {
            Some(Commands::Chid(a)) => a,
            _ => panic!("expected Chid"),
        };
        let params = ChidParams::try_from(&args).unwrap();
        // default: no flags → repos resolves to [.]
        assert_eq!(params.common.repos, vec![PathBuf::from(".")]);
        // default revision @
        assert_eq!(params.common.spec.rev, "@");
    }
}

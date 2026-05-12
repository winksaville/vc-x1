//! `chid` subcommand — print the short change ID of each commit in a
//! revision range, one per line (script-friendly output).
//!
//! - `ChidArgs`: clap surface; flattens
//!   `options_flags::common_args::CommonArgs`.
//! - `chid(&ChidArgs)`: the op — `resolve_spec` / `resolve_header` /
//!   `for_each_repo` + print `format_chid`.

use clap::Args;
use jj_lib::repo::Repo;
use log::{debug, info};

use crate::common;
use crate::options_flags::common_args::CommonArgs;

/// CLI args for `chid` — just the shared read-only commit-query args.
#[derive(Args, Debug)]
pub struct ChidArgs {
    #[command(flatten)]
    pub common: CommonArgs,
}

/// Print the short change ID of each commit in the resolved range.
pub fn chid(args: &ChidArgs) -> Result<(), Box<dyn std::error::Error>> {
    debug!("chid: enter");
    let c = &args.common;
    let spec = common::resolve_spec(c.pos_rev.as_deref(), c.pos_count, &c.revision, c.limit, "@");
    let hdr = common::resolve_header(&c.label, c.no_label);

    common::for_each_repo(&c.repos, &hdr, |workspace, repo| {
        let (ids, _) =
            common::collect_ids(workspace, repo, &spec.rev, spec.desc_count, spec.anc_count)?;

        for commit_id in &ids {
            let commit = repo.store().get_commit(commit_id)?;
            info!("{}", common::format_chid(&commit));
        }
        Ok(())
    })?;
    debug!("chid: exit");
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
            Commands::Chid(a) => a.common,
            _ => panic!("expected Chid"),
        }
    }

    #[test]
    fn defaults() {
        let c = parse(&["vc-x1", "chid"]);
        assert_eq!(c.revision, "@");
        assert!(c.repos.is_empty());
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
        assert_eq!(c.repos, vec![PathBuf::from(".claude")]);
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
        assert_eq!(c.repos, vec![PathBuf::from(".claude")]);
        assert_eq!(c.limit, Some(3));
    }

    #[test]
    fn multi_repo() {
        let c = parse(&["vc-x1", "chid", "-R", ".", "-R", ".claude"]);
        assert_eq!(c.repos, vec![PathBuf::from("."), PathBuf::from(".claude")]);
    }

    #[test]
    fn comma_repo() {
        let c = parse(&["vc-x1", "chid", "-R", ".,.claude"]);
        assert_eq!(c.repos, vec![PathBuf::from(".,.claude")]);
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
}

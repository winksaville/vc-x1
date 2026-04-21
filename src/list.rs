use clap::Args;
use jj_lib::repo::Repo;
use log::{debug, info, trace, warn};

use crate::common;

#[derive(Args, Debug)]
pub struct ListArgs {
    #[command(flatten)]
    pub common: common::CommonArgs,

    /// ochid column width
    #[arg(short = 'w', long = "width", default_value_t = DEFAULT_OCHID_WIDTH)]
    pub width: usize,
}

const DEFAULT_OCHID_WIDTH: usize = 21;

pub fn list(args: &ListArgs) -> Result<(), Box<dyn std::error::Error>> {
    debug!("list: enter");
    let c = &args.common;
    let spec = common::resolve_spec(c.pos_rev.as_deref(), c.pos_count, &c.revision, c.limit, "@");
    trace!(
        "list: spec rev={} desc={:?} anc={:?}",
        spec.rev, spec.desc_count, spec.anc_count
    );
    let hdr = common::resolve_header(&c.label, c.no_label);

    common::for_each_repo(&c.repos, &hdr, |workspace, repo| {
        let (ids, anchor_index) =
            common::collect_ids(workspace, repo, &spec.rev, spec.desc_count, spec.anc_count)?;
        debug!("list: {} commits, anchor at {anchor_index}", ids.len());

        if ids.is_empty() {
            warn!("list: no commits found for revision '{}'", spec.rev);
        }

        for (i, commit_id) in ids.iter().enumerate() {
            let commit = repo.store().get_commit(commit_id)?;
            let bookmarks = common::format_bookmarks_at(repo, commit_id);
            let line = common::format_commit_with_ochid(&commit, args.width, &bookmarks);
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
        assert!(args.common.repos.is_empty());
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
        assert_eq!(args.common.repos, vec![PathBuf::from("/some/path")]);
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
        assert_eq!(args.common.repos, vec![PathBuf::from(".claude")]);
        assert_eq!(args.common.limit, Some(10));
    }

    #[test]
    fn multi_repo() {
        let args = parse(&["vc-x1", "list", "-R", ".", "-R", ".claude"]);
        assert_eq!(
            args.common.repos,
            vec![PathBuf::from("."), PathBuf::from(".claude")]
        );
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
}

use clap::Args;
use jj_lib::repo::Repo;
use log::info;

use crate::common;

#[derive(Args, Debug)]
pub struct DescArgs {
    #[command(flatten)]
    pub common: common::CommonArgs,
}

pub fn desc(args: &DescArgs) -> Result<(), Box<dyn std::error::Error>> {
    let c = &args.common;
    let spec = common::resolve_spec(c.pos_rev.as_deref(), c.pos_count, &c.revision, c.limit, "@");
    let hdr = common::resolve_header(&c.label, c.no_label);

    common::for_each_repo(&c.repos, &hdr, |workspace, repo| {
        let (ids, anchor_index) =
            common::collect_ids(workspace, repo, &spec.rev, spec.desc_count, spec.anc_count)?;

        for (i, commit_id) in ids.iter().enumerate() {
            let commit = repo.store().get_commit(commit_id)?;
            let line = common::format_commit_full(&commit);
            let indented = common::indent_body(&line, 4);
            if i == anchor_index {
                info!("{}", common::bold_first_line(&indented));
            } else {
                info!("{indented}");
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::Parser;

    use crate::{Cli, Commands};

    fn parse(args: &[&str]) -> crate::common::CommonArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Desc(a) => a.common,
            _ => panic!("expected Desc"),
        }
    }

    #[test]
    fn defaults() {
        let c = parse(&["vc-x1", "desc"]);
        assert_eq!(c.revision, "@");
        assert!(c.repos.is_empty());
    }

    #[test]
    fn with_revision() {
        let c = parse(&["vc-x1", "desc", "-r", "wmuxkqwu"]);
        assert_eq!(c.revision, "wmuxkqwu");
    }

    #[test]
    fn with_repo() {
        let c = parse(&["vc-x1", "desc", "-R", "/tmp"]);
        assert_eq!(c.repos, vec![PathBuf::from("/tmp")]);
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
        assert_eq!(c.repos, vec![PathBuf::from(".claude")]);
        assert_eq!(c.limit, Some(5));
    }

    #[test]
    fn multi_repo() {
        let c = parse(&["vc-x1", "desc", "-R", ".", "-R", ".claude"]);
        assert_eq!(c.repos, vec![PathBuf::from("."), PathBuf::from(".claude")]);
    }
}

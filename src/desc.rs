use std::path::PathBuf;

use clap::Args;
use jj_lib::repo::Repo;

use crate::common;

#[derive(Args, Debug)]
pub struct DescArgs {
    /// Revision (with optional .. notation)
    #[arg(value_name = "REVISION")]
    pub pos_rev: Option<String>,

    /// Number of commits to show (per dotted side)
    #[arg(value_name = "COMMITS")]
    pub pos_count: Option<usize>,

    /// Revision to describe
    #[arg(short, long, default_value = "@")]
    pub revision: String,

    /// Path to jj repo; repeatable or comma-separated [default: .]
    #[arg(short = 'R', long = "repo", value_name = "PATH")]
    pub repos: Vec<PathBuf>,

    /// Number of commits to describe
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

pub fn desc(args: &DescArgs) -> Result<(), Box<dyn std::error::Error>> {
    let spec = common::resolve_spec(
        args.pos_rev.as_deref(),
        args.pos_count,
        &args.revision,
        args.limit,
        "@",
    );
    let hdr = common::resolve_header(&args.label, args.no_label);

    common::for_each_repo(&args.repos, &hdr, |workspace, repo| {
        let (ids, anchor_index) =
            common::collect_ids(workspace, repo, &spec.rev, spec.desc_count, spec.anc_count)?;

        for (i, commit_id) in ids.iter().enumerate() {
            let commit = repo.store().get_commit(commit_id)?;
            let line = common::format_commit_full(&commit);
            let indented = common::indent_body(&line, 4);
            if i == anchor_index {
                println!("{}", common::bold_first_line(&indented));
            } else {
                println!("{indented}");
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

    fn parse(args: &[&str]) -> super::DescArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Desc(a) => a,
            _ => panic!("expected Desc"),
        }
    }

    #[test]
    fn defaults() {
        let args = parse(&["vc-x1", "desc"]);
        assert_eq!(args.revision, "@");
        assert!(args.repos.is_empty());
    }

    #[test]
    fn with_revision() {
        let args = parse(&["vc-x1", "desc", "-r", "wmuxkqwu"]);
        assert_eq!(args.revision, "wmuxkqwu");
    }

    #[test]
    fn with_repo() {
        let args = parse(&["vc-x1", "desc", "-R", "/tmp"]);
        assert_eq!(args.repos, vec![PathBuf::from("/tmp")]);
    }

    #[test]
    fn with_limit() {
        let args = parse(&["vc-x1", "desc", "-n", "3"]);
        assert_eq!(args.limit, Some(3));
    }

    #[test]
    fn positional_rev() {
        let args = parse(&["vc-x1", "desc", "@-"]);
        assert_eq!(args.pos_rev, Some("@-".to_string()));
        assert_eq!(args.pos_count, None);
    }

    #[test]
    fn positional_rev_and_count() {
        let args = parse(&["vc-x1", "desc", "@..", "5"]);
        assert_eq!(args.pos_rev, Some("@..".to_string()));
        assert_eq!(args.pos_count, Some(5));
    }

    #[test]
    fn positional_both_dots() {
        let args = parse(&["vc-x1", "desc", "..abcd..", "3"]);
        assert_eq!(args.pos_rev, Some("..abcd..".to_string()));
        assert_eq!(args.pos_count, Some(3));
    }

    #[test]
    fn all_opts() {
        let args = parse(&["vc-x1", "desc", "-r", "@-", "-R", ".claude", "-n", "5"]);
        assert_eq!(args.revision, "@-");
        assert_eq!(args.repos, vec![PathBuf::from(".claude")]);
        assert_eq!(args.limit, Some(5));
    }

    #[test]
    fn multi_repo() {
        let args = parse(&["vc-x1", "desc", "-R", ".", "-R", ".claude"]);
        assert_eq!(
            args.repos,
            vec![PathBuf::from("."), PathBuf::from(".claude")]
        );
    }
}

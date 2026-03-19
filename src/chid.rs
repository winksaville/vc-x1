use std::path::PathBuf;

use clap::Args;
use jj_lib::repo::Repo;

use crate::common;

#[derive(Args, Debug)]
pub struct ChidArgs {
    /// Revision (with optional .. notation)
    #[arg(value_name = "REVISION")]
    pub pos_rev: Option<String>,

    /// Number of commits to show (per dotted side)
    #[arg(value_name = "COMMITS")]
    pub pos_count: Option<usize>,

    /// Revision to look up (default: @)
    #[arg(short, long, default_value = "@")]
    pub revision: String,

    /// Path to jj repo; repeatable or comma-separated (default: .)
    #[arg(short = 'R', long = "repo", value_name = "PATH")]
    pub repos: Vec<PathBuf>,

    /// Number of changeIDs to show
    #[arg(short = 'n', long = "commits", value_name = "COMMITS")]
    pub limit: Option<usize>,

    /// Custom label decoration between repos (default: ===)
    #[arg(
        short = 'l',
        long = "label",
        value_name = "TEXT",
        allow_hyphen_values = true
    )]
    pub label: Option<String>,

    /// Suppress label between repos
    #[arg(short = 'L', long = "no-label")]
    pub no_label: bool,
}

pub fn chid(args: &ChidArgs) -> Result<(), Box<dyn std::error::Error>> {
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
            let line = common::format_chid(&commit);
            if i == anchor_index {
                println!("{}", common::bold(&line));
            } else {
                println!("{line}");
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

    fn parse(args: &[&str]) -> super::ChidArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Chid(a) => a,
            _ => panic!("expected Chid"),
        }
    }

    #[test]
    fn defaults() {
        let args = parse(&["vc-x1", "chid"]);
        assert_eq!(args.revision, "@");
        assert!(args.repos.is_empty());
        assert_eq!(args.limit, None);
    }

    #[test]
    fn with_revision() {
        let args = parse(&["vc-x1", "chid", "-r", "@-"]);
        assert_eq!(args.revision, "@-");
    }

    #[test]
    fn with_repo() {
        let args = parse(&["vc-x1", "chid", "-R", ".claude"]);
        assert_eq!(args.repos, vec![PathBuf::from(".claude")]);
    }

    #[test]
    fn with_limit() {
        let args = parse(&["vc-x1", "chid", "-n", "5"]);
        assert_eq!(args.limit, Some(5));
    }

    #[test]
    fn all_opts() {
        let args = parse(&["vc-x1", "chid", "-r", "@--", "-R", ".claude", "-n", "3"]);
        assert_eq!(args.revision, "@--");
        assert_eq!(args.repos, vec![PathBuf::from(".claude")]);
        assert_eq!(args.limit, Some(3));
    }

    #[test]
    fn multi_repo() {
        let args = parse(&["vc-x1", "chid", "-R", ".", "-R", ".claude"]);
        assert_eq!(
            args.repos,
            vec![PathBuf::from("."), PathBuf::from(".claude")]
        );
    }

    #[test]
    fn comma_repo() {
        let args = parse(&["vc-x1", "chid", "-R", ".,.claude"]);
        assert_eq!(args.repos, vec![PathBuf::from(".,.claude")]);
    }

    #[test]
    fn label() {
        let args = parse(&["vc-x1", "chid", "-l", "==="]);
        assert_eq!(args.label, Some("===".to_string()));
        assert!(!args.no_label);
    }

    #[test]
    fn no_label() {
        let args = parse(&["vc-x1", "chid", "-L"]);
        assert!(args.no_label);
        assert_eq!(args.label, None);
    }

    #[test]
    fn positional_rev() {
        let args = parse(&["vc-x1", "chid", "@-"]);
        assert_eq!(args.pos_rev, Some("@-".to_string()));
        assert_eq!(args.pos_count, None);
    }

    #[test]
    fn positional_rev_and_count() {
        let args = parse(&["vc-x1", "chid", "@..", "5"]);
        assert_eq!(args.pos_rev, Some("@..".to_string()));
        assert_eq!(args.pos_count, Some(5));
    }

    #[test]
    fn positional_both_dots() {
        let args = parse(&["vc-x1", "chid", "..abcd..", "3"]);
        assert_eq!(args.pos_rev, Some("..abcd..".to_string()));
        assert_eq!(args.pos_count, Some(3));
    }
}

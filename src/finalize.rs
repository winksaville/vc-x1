use std::path::PathBuf;
use std::process::Stdio;

use clap::Args;
use log::{debug, info};

use crate::common::run;

#[derive(Args, Debug)]
pub struct FinalizeArgs {
    /// Path to jj repo
    #[arg(long, default_value = ".")]
    pub repo: PathBuf,

    /// Source revision to squash
    #[arg(long, default_value = "@")]
    pub source: String,

    /// Target revision to squash into
    #[arg(long, default_value = "@-")]
    pub target: String,

    /// Seconds to wait before squashing
    #[arg(long, default_value_t = 10.0)]
    pub delay: f64,

    /// Bookmark to advance to target after squash (required)
    #[arg(long)]
    pub bookmark: String,

    /// Push after squashing
    #[arg(long)]
    pub push: bool,

    /// Log file path (off by default)
    #[arg(long)]
    pub log: Option<PathBuf>,

    /// Detach and run in the background
    #[arg(long)]
    pub detach: bool,

    /// Internal: run exec path (used by detach)
    #[arg(long, hide = true)]
    pub exec: bool,
}

#[derive(Debug)]
pub struct FinalizeOpts {
    pub repo: PathBuf,
    pub source: String,
    pub target: String,
    pub bookmark: String,
    pub delay_secs: f64,
    pub push: bool,
    pub log: Option<PathBuf>,
    pub detach: bool,
    pub exec: bool,
}

impl FinalizeArgs {
    pub fn into_opts(self) -> FinalizeOpts {
        FinalizeOpts {
            repo: self.repo,
            source: self.source,
            target: self.target,
            bookmark: self.bookmark,
            delay_secs: self.delay,
            push: self.push,
            log: self.log,
            detach: self.detach,
            exec: self.exec,
        }
    }
}

fn finalize_exec(opts: &FinalizeOpts) -> Result<(), Box<dyn std::error::Error>> {
    debug!("finalize_exec: starting opts={opts:?}");

    // Sleep to let trailing writes settle
    let delay = std::time::Duration::from_secs_f64(opts.delay_secs);
    debug!("finalize_exec: sleeping {delay:?}");
    std::thread::sleep(delay);

    // Squash source into target
    let repo_str = opts.repo.to_string_lossy();
    run(
        "jj",
        &[
            "squash",
            "--ignore-immutable",
            "--use-destination-message",
            "--from",
            &opts.source,
            "--into",
            &opts.target,
            "-R",
            &repo_str,
        ],
        &opts.repo,
    )?;

    // Advance bookmark to target
    run(
        "jj",
        &[
            "bookmark",
            "set",
            &opts.bookmark,
            "-r",
            &opts.target,
            "-R",
            &repo_str,
        ],
        &opts.repo,
    )?;

    if opts.push {
        run(
            "jj",
            &["git", "push", "--bookmark", &opts.bookmark, "-R", &repo_str],
            &opts.repo,
        )?;
    }

    debug!("finalize_exec: done");
    Ok(())
}

pub fn build_exec_args(opts: &FinalizeOpts) -> Vec<String> {
    let mut args = vec![
        "finalize".to_string(),
        "--exec".to_string(),
        "--repo".to_string(),
        opts.repo.to_string_lossy().to_string(),
        "--source".to_string(),
        opts.source.clone(),
        "--target".to_string(),
        opts.target.clone(),
        "--bookmark".to_string(),
        opts.bookmark.clone(),
        "--delay".to_string(),
        opts.delay_secs.to_string(),
    ];
    if let Some(ref log) = opts.log {
        args.push("--log".to_string());
        args.push(log.to_string_lossy().to_string());
    }
    if opts.push {
        args.push("--push".to_string());
    }
    args
}

/// Spawn a detached child process with `--exec` and return immediately.
/// The child re-enters `main()`, parses `--exec`, and `finalize()` routes
/// it to `finalize_exec()` where the actual work happens.
fn detach(opts: &FinalizeOpts) -> Result<(), Box<dyn std::error::Error>> {
    debug!("detach: parent starting opts={opts:?}");

    let exe = std::env::current_exe()?;
    let args = build_exec_args(opts);
    debug!("detach: exe={exe:?} args={args:?}");

    let mut cmd = std::process::Command::new(exe);
    cmd.args(&args).stdin(Stdio::null());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
    }

    let child = cmd.spawn()?;
    debug!("detach: spawned child pid={}", child.id());
    match &opts.log {
        Some(log) => info!(
            "finalize: detached (pid {}), log: {}",
            child.id(),
            log.display()
        ),
        None => info!("finalize: detached (pid {})", child.id()),
    }
    Ok(())
}

pub fn finalize(opts: &FinalizeOpts) -> Result<(), Box<dyn std::error::Error>> {
    debug!("finalize: entry opts={opts:?}");
    let result = if opts.detach && !opts.exec {
        detach(opts)
    } else {
        finalize_exec(opts)
    };
    match &result {
        Ok(()) => debug!("finalize: exit ok"),
        Err(e) => debug!("finalize: exit err={e}"),
    }
    result
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::{Cli, Commands};
    use clap::Parser;

    fn parse(args: &[&str]) -> FinalizeArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Finalize(a) => a,
            _ => panic!("expected Finalize"),
        }
    }

    fn parse_err(args: &[&str]) -> String {
        Cli::try_parse_from(args).unwrap_err().to_string()
    }

    #[test]
    fn defaults() {
        let args = parse(&["vc-x1", "finalize", "--bookmark", "main"]);
        assert_eq!(args.repo, PathBuf::from("."));
        assert_eq!(args.source, "@");
        assert_eq!(args.target, "@-");
        assert_eq!(args.bookmark, "main");
        assert_eq!(args.delay, 10.0);
        assert!(!args.push);
        assert!(!args.detach);
    }

    #[test]
    fn all_opts() {
        let args = parse(&[
            "vc-x1",
            "finalize",
            "--repo",
            ".claude",
            "--source",
            "@",
            "--target",
            "@-",
            "--bookmark",
            "dev-0.14.0",
            "--delay",
            "2.5",
            "--push",
            "--log",
            "/tmp/test.log",
            "--detach",
        ]);
        assert_eq!(args.repo, PathBuf::from(".claude"));
        assert_eq!(args.source, "@");
        assert_eq!(args.target, "@-");
        assert_eq!(args.bookmark, "dev-0.14.0");
        assert_eq!(args.delay, 2.5);
        assert!(args.push);
        assert_eq!(args.log, Some(PathBuf::from("/tmp/test.log")));
        assert!(args.detach);
    }

    #[test]
    fn partial_opts() {
        let args = parse(&[
            "vc-x1",
            "finalize",
            "--bookmark",
            "main",
            "--repo",
            ".claude",
            "--push",
        ]);
        assert_eq!(args.repo, PathBuf::from(".claude"));
        assert_eq!(args.bookmark, "main");
        assert!(args.push);
    }

    #[test]
    fn missing_bookmark() {
        let err = parse_err(&["vc-x1", "finalize"]);
        assert!(err.contains("--bookmark"));
    }

    #[test]
    fn bad_delay() {
        let err = parse_err(&["vc-x1", "finalize", "--delay", "abc"]);
        assert!(err.contains("invalid value"));
    }

    #[test]
    fn unknown_opt() {
        let err = parse_err(&["vc-x1", "finalize", "--bogus"]);
        assert!(err.contains("--bogus"));
    }

    #[test]
    fn build_exec_args_roundtrip() {
        let opts = FinalizeOpts {
            repo: PathBuf::from(".claude"),
            source: "@".to_string(),
            target: "@-".to_string(),
            bookmark: "dev-0.14.0".to_string(),
            delay_secs: 2.0,
            push: true,
            log: Some(PathBuf::from("/tmp/test.log")),
            detach: false,
            exec: false,
        };
        let exec_args = build_exec_args(&opts);
        let mut full_args = vec!["vc-x1".to_string()];
        full_args.extend(exec_args);
        let cli = Cli::try_parse_from(full_args).unwrap();
        if let crate::Commands::Finalize(args) = cli.command {
            let parsed = args.into_opts();
            assert_eq!(parsed.repo, opts.repo);
            assert_eq!(parsed.source, opts.source);
            assert_eq!(parsed.target, opts.target);
            assert_eq!(parsed.bookmark, opts.bookmark);
            assert_eq!(parsed.delay_secs, opts.delay_secs);
            assert_eq!(parsed.push, opts.push);
            assert_eq!(parsed.log, opts.log);
            assert!(parsed.exec);
        } else {
            panic!("expected Finalize");
        }
    }
}

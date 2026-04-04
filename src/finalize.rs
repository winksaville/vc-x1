use std::path::PathBuf;
use std::process::Stdio;

use clap::Args;
use log::{debug, info};

use crate::common::run;

/// Squash, set bookmark, and/or push a jj repo.
///
/// Designed for the bot to atomically finalize its session repo:
/// --detach exits immediately, --delay waits for trailing writes,
/// --squash folds them in, --bookmark + --push sends it upstream.
/// Every flag is opt-in. See README.md for details.
#[derive(Args, Debug)]
pub struct FinalizeArgs {
    /// Path to jj repo
    #[arg(long, default_value = ".")]
    pub repo: PathBuf,

    /// Squash SOURCE into TARGET [default: @,@-]
    #[arg(long, value_name = "SOURCE,TARGET", default_missing_value = "@,@-", num_args = 0..=1)]
    pub squash: Option<String>,

    /// Seconds to wait before squashing
    #[arg(long, default_value_t = 10.0)]
    pub delay: f64,

    /// Existing bookmark to push to remote
    #[arg(long, value_name = "BOOKMARK")]
    pub push: Option<String>,

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

/// Parsed squash spec: source and target revisions.
#[derive(Debug, Clone)]
pub struct SquashSpec {
    pub source: String,
    pub target: String,
}

impl SquashSpec {
    fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(format!(
                "invalid --squash value '{s}': expected SOURCE,TARGET (e.g. @,@-)"
            ));
        }
        Ok(SquashSpec {
            source: parts[0].to_string(),
            target: parts[1].to_string(),
        })
    }
}

#[derive(Debug)]
pub struct FinalizeOpts {
    pub repo: PathBuf,
    pub squash: Option<SquashSpec>,
    pub delay_secs: f64,
    pub bookmark: Option<String>,
    pub push: bool,
    pub log: Option<PathBuf>,
    pub detach: bool,
    pub exec: bool,
}

impl FinalizeArgs {
    pub fn into_opts(self) -> Result<FinalizeOpts, String> {
        let squash = self.squash.map(|s| SquashSpec::parse(&s)).transpose()?;
        let push = self.push.is_some();
        let bookmark = self.push;
        // Resolve to absolute path so detached child works regardless of cwd
        let repo = std::fs::canonicalize(&self.repo)
            .map_err(|e| format!("cannot resolve repo path '{}': {e}", self.repo.display()))?;
        Ok(FinalizeOpts {
            repo,
            squash,
            delay_secs: self.delay,
            bookmark,
            push,
            log: self.log,
            detach: self.detach,
            exec: self.exec,
        })
    }
}

fn finalize_exec(opts: &FinalizeOpts) -> Result<(), Box<dyn std::error::Error>> {
    debug!("finalize_exec: starting opts={opts:?}");
    let repo_str = opts.repo.to_string_lossy();
    let cwd = std::path::Path::new(".");

    // Squash if requested
    if let Some(ref sq) = opts.squash {
        let delay = std::time::Duration::from_secs_f64(opts.delay_secs);
        debug!("finalize_exec: sleeping {delay:?}");
        std::thread::sleep(delay);

        run(
            "jj",
            &[
                "squash",
                "--ignore-immutable",
                "--use-destination-message",
                "--from",
                &sq.source,
                "--into",
                &sq.target,
                "-R",
                &repo_str,
            ],
            cwd,
        )?;
    }

    // Set bookmark and push if requested
    if let Some(ref bookmark) = opts.bookmark {
        // Verify bookmark exists — don't silently create new ones
        let result = run("jj", &["bookmark", "list", bookmark, "-R", &repo_str], cwd)?;
        if result.is_empty() {
            return Err(format!("bookmark '{bookmark}' does not exist").into());
        }

        let rev = opts
            .squash
            .as_ref()
            .map(|sq| sq.target.as_str())
            .unwrap_or("@");
        run(
            "jj",
            &["bookmark", "set", bookmark, "-r", rev, "-R", &repo_str],
            cwd,
        )?;

        if opts.push {
            run(
                "jj",
                &["git", "push", "--bookmark", bookmark, "-R", &repo_str],
                &opts.repo,
            )?;
        }
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
    ];
    if let Some(ref sq) = opts.squash {
        args.push("--squash".to_string());
        args.push(format!("{},{}", sq.source, sq.target));
        args.push("--delay".to_string());
        args.push(opts.delay_secs.to_string());
    }
    if let Some(ref bookmark) = opts.bookmark {
        args.push("--push".to_string());
        args.push(bookmark.clone());
    }
    if let Some(ref log) = opts.log {
        args.push("--log".to_string());
        args.push(log.to_string_lossy().to_string());
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
    // Nothing to do — show help hint
    if opts.squash.is_none() && !opts.push {
        return Err("nothing to do (see --help for options)".into());
    }

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
    fn no_args() {
        let args = parse(&["vc-x1", "finalize"]);
        assert_eq!(args.repo, PathBuf::from("."));
        assert!(args.squash.is_none());
        assert!(args.push.is_none());
        assert!(!args.detach);
    }

    #[test]
    fn push_only() {
        let args = parse(&["vc-x1", "finalize", "--push", "main"]);
        assert!(args.squash.is_none());
        assert_eq!(args.push, Some("main".to_string()));
    }

    #[test]
    fn all_opts() {
        let args = parse(&[
            "vc-x1",
            "finalize",
            "--repo",
            ".claude",
            "--squash",
            "@,@-",
            "--push",
            "dev-0.14.0",
            "--delay",
            "2.5",
            "--log",
            "/tmp/test.log",
            "--detach",
        ]);
        assert_eq!(args.repo, PathBuf::from(".claude"));
        assert_eq!(args.squash, Some("@,@-".to_string()));
        assert_eq!(args.push, Some("dev-0.14.0".to_string()));
        assert_eq!(args.delay, 2.5);
        assert_eq!(args.log, Some(PathBuf::from("/tmp/test.log")));
        assert!(args.detach);
    }

    #[test]
    fn bare_squash() {
        let args = parse(&["vc-x1", "finalize", "--squash", "--push", "main"]);
        assert_eq!(args.squash, Some("@,@-".to_string()));
        assert_eq!(args.push, Some("main".to_string()));
    }

    #[test]
    fn squash_parse_valid() {
        let sq = SquashSpec::parse("@,@-").unwrap();
        assert_eq!(sq.source, "@");
        assert_eq!(sq.target, "@-");
    }

    #[test]
    fn squash_parse_invalid() {
        assert!(SquashSpec::parse("@").is_err());
        assert!(SquashSpec::parse(",").is_err());
        assert!(SquashSpec::parse("@,").is_err());
        assert!(SquashSpec::parse(",@-").is_err());
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
            squash: Some(SquashSpec {
                source: "@".to_string(),
                target: "@-".to_string(),
            }),
            delay_secs: 2.0,
            bookmark: Some("dev-0.14.0".to_string()),
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
            let parsed = args.into_opts().unwrap();
            // repo is canonicalized, so compare the canonical form
            assert_eq!(parsed.repo, std::fs::canonicalize(&opts.repo).unwrap());
            assert_eq!(parsed.squash.as_ref().unwrap().source, "@");
            assert_eq!(parsed.squash.as_ref().unwrap().target, "@-");
            assert_eq!(parsed.bookmark, Some("dev-0.14.0".to_string()));
            assert_eq!(parsed.delay_secs, opts.delay_secs);
            assert!(parsed.push);
            assert_eq!(parsed.log, opts.log);
            assert!(parsed.exec);
        } else {
            panic!("expected Finalize");
        }
    }
}

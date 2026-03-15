use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use clap::Args;

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
    #[arg(long, default_value_t = 1.0)]
    pub delay: f64,

    /// Push after squashing
    #[arg(long)]
    pub push: bool,

    /// Log file path (default: /tmp/vc-x1-finalize-<timestamp-millis>.log)
    #[arg(long)]
    pub log: Option<PathBuf>,

    /// Run in foreground instead of daemonizing
    #[arg(long)]
    pub foreground: bool,

    /// Internal: run exec path (used by daemonize)
    #[arg(long, hide = true)]
    pub exec: bool,
}

#[derive(Debug)]
pub struct FinalizeOpts {
    pub repo: PathBuf,
    pub source: String,
    pub target: String,
    pub delay_secs: f64,
    pub push: bool,
    pub log: PathBuf,
    pub foreground: bool,
    pub exec: bool,
}

impl FinalizeArgs {
    pub fn into_opts(self) -> FinalizeOpts {
        let log = self.log.unwrap_or_else(|| {
            PathBuf::from(format!(
                "/tmp/vc-x1-finalize-{}.log",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            ))
        });
        FinalizeOpts {
            repo: self.repo,
            source: self.source,
            target: self.target,
            delay_secs: self.delay,
            push: self.push,
            log,
            foreground: self.foreground,
            exec: self.exec,
        }
    }
}

pub fn log_msg(log: &Path, msg: &str) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    let line = format!("[{nanos}] pid={pid} {msg}\n");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)
    {
        let _ = f.write_all(line.as_bytes());
    }
}

fn run_jj(args: &[&str], log: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let args_str = args.join(" ");
    log_msg(log, &format!("run_jj: jj {args_str}"));
    let output = std::process::Command::new("jj")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run jj: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.is_empty() {
        log_msg(log, &format!("run_jj: stdout: {stdout}"));
    }
    if !stderr.is_empty() {
        log_msg(log, &format!("run_jj: stderr: {stderr}"));
    }
    if !output.status.success() {
        return Err(format!("jj {args_str} failed (exit {}): {stderr}", output.status).into());
    }
    Ok(())
}

fn finalize_exec(opts: &FinalizeOpts) -> Result<(), Box<dyn std::error::Error>> {
    log_msg(&opts.log, &format!("finalize_exec: starting opts={opts:?}"));

    // Sleep to let trailing writes settle
    let delay = std::time::Duration::from_secs_f64(opts.delay_secs);
    log_msg(&opts.log, &format!("finalize_exec: sleeping {delay:?}"));
    std::thread::sleep(delay);

    // Squash source into target
    let repo_str = opts.repo.to_string_lossy();
    run_jj(
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
        &opts.log,
    )?;

    if opts.push {
        // Advance main bookmark to target, then push
        run_jj(
            &[
                "bookmark",
                "set",
                "main",
                "-r",
                &opts.target,
                "-R",
                &repo_str,
            ],
            &opts.log,
        )?;
        run_jj(&["git", "push", "-R", &repo_str], &opts.log)?;
    }

    log_msg(&opts.log, "finalize_exec: done");
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
        "--delay".to_string(),
        opts.delay_secs.to_string(),
        "--log".to_string(),
        opts.log.to_string_lossy().to_string(),
    ];
    if opts.push {
        args.push("--push".to_string());
    }
    args
}

/// Spawn a detached child process with `--exec` and return immediately.
/// The child re-enters `main()`, parses `--exec`, and `finalize()` routes
/// it to `finalize_exec()` where the actual work happens.
fn daemonize(opts: &FinalizeOpts) -> Result<(), Box<dyn std::error::Error>> {
    log_msg(
        &opts.log,
        &format!("daemonize: parent starting opts={opts:?}"),
    );

    let exe = std::env::current_exe()?;
    let args = build_exec_args(opts);
    log_msg(&opts.log, &format!("daemonize: exe={exe:?} args={args:?}"));

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
    log_msg(
        &opts.log,
        &format!("daemonize: spawned child pid={}", child.id()),
    );
    eprintln!(
        "finalize: daemonized (pid {}), log: {}",
        child.id(),
        opts.log.display()
    );
    Ok(())
}

pub fn finalize(opts: &FinalizeOpts) -> Result<(), Box<dyn std::error::Error>> {
    log_msg(&opts.log, &format!("finalize: entry opts={opts:?}"));
    let result = if opts.foreground || opts.exec {
        finalize_exec(opts)
    } else {
        daemonize(opts)
    };
    match &result {
        Ok(()) => log_msg(&opts.log, "finalize: exit ok"),
        Err(e) => log_msg(&opts.log, &format!("finalize: exit err={e}")),
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cli;
    use clap::Parser;

    #[test]
    fn build_exec_args_roundtrip() {
        let opts = FinalizeOpts {
            repo: PathBuf::from(".claude"),
            source: "@".to_string(),
            target: "@-".to_string(),
            delay_secs: 2.0,
            push: true,
            log: PathBuf::from("/tmp/test.log"),
            foreground: false,
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
            assert_eq!(parsed.delay_secs, opts.delay_secs);
            assert_eq!(parsed.push, opts.push);
            assert_eq!(parsed.log, opts.log);
            assert!(parsed.exec);
        } else {
            panic!("expected Finalize");
        }
    }
}

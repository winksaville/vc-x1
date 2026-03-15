use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;

pub const USAGE: &str = "\
Usage: vc-x1 finalize [OPTIONS]

Squash source revision into target revision, optionally push afterward.
By default, daemonizes and returns immediately.

Options:
  --repo <path>      Path to jj repo (default: current directory)
  --source <revset>  Source revision to squash (default: @)
  --target <revset>  Target revision to squash into (default: @-)
  --delay <seconds>  Seconds to wait before squashing (default: 1)
  --push             Push after squashing
  --log <path>       Log file path (default: /tmp/vc-x1-finalize-<timestamp-millis>.log)
  --foreground       Run in foreground instead of daemonizing
  -h, --help         Print this help message";

#[derive(Debug, PartialEq)]
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

impl Default for FinalizeOpts {
    fn default() -> Self {
        Self {
            repo: PathBuf::from("."),
            source: "@".to_string(),
            target: "@-".to_string(),
            delay_secs: 1.0,
            push: false,
            log: PathBuf::from(format!(
                "/tmp/vc-x1-finalize-{}.log",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            )),
            foreground: false,
            exec: false,
        }
    }
}

/// Parse finalize subcommand args. Returns `Ok(None)` when help is requested.
pub fn parse_args<I>(args: &mut I) -> Result<Option<FinalizeOpts>, String>
where
    I: Iterator<Item = String>,
{
    let mut opts = FinalizeOpts::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--repo" => {
                opts.repo = PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--repo requires a value".to_string())?,
                );
            }
            "--source" => {
                opts.source = args
                    .next()
                    .ok_or_else(|| "--source requires a value".to_string())?;
            }
            "--target" => {
                opts.target = args
                    .next()
                    .ok_or_else(|| "--target requires a value".to_string())?;
            }
            "--delay" => {
                let val = args
                    .next()
                    .ok_or_else(|| "--delay requires a value".to_string())?;
                opts.delay_secs = val
                    .parse()
                    .map_err(|_| format!("invalid delay value: {val}"))?;
            }
            "--push" => opts.push = true,
            "--log" => {
                opts.log = PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--log requires a value".to_string())?,
                );
            }
            "--foreground" => opts.foreground = true,
            "--exec" => opts.exec = true,
            other => {
                return Err(format!("unknown finalize option: {other}\n\n{USAGE}"));
            }
        }
    }

    Ok(Some(opts))
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

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_defaults() {
        let result = parse_args(&mut std::iter::empty::<String>());
        assert_eq!(result, Ok(Some(FinalizeOpts::default())));
    }

    #[test]
    fn parse_all_opts() {
        let a = args(&[
            "--repo",
            ".claude",
            "--source",
            "@",
            "--target",
            "@-",
            "--delay",
            "2.5",
            "--push",
            "--log",
            "/tmp/test.log",
            "--foreground",
        ]);
        let result = parse_args(&mut a.into_iter()).unwrap().unwrap();
        assert_eq!(result.repo, PathBuf::from(".claude"));
        assert_eq!(result.source, "@");
        assert_eq!(result.target, "@-");
        assert_eq!(result.delay_secs, 2.5);
        assert!(result.push);
        assert_eq!(result.log, PathBuf::from("/tmp/test.log"));
        assert!(result.foreground);
        assert!(!result.exec);
    }

    #[test]
    fn parse_help() {
        let a = args(&["--help"]);
        assert_eq!(parse_args(&mut a.into_iter()), Ok(None));
    }

    #[test]
    fn parse_partial_opts() {
        let a = args(&["--repo", ".claude", "--push"]);
        let result = parse_args(&mut a.into_iter()).unwrap().unwrap();
        assert_eq!(result.repo, PathBuf::from(".claude"));
        assert!(result.push);
    }

    #[test]
    fn parse_missing_value() {
        let a = args(&["--repo"]);
        let result = parse_args(&mut a.into_iter());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--repo requires a value"));
    }

    #[test]
    fn parse_bad_delay() {
        let a = args(&["--delay", "abc"]);
        let result = parse_args(&mut a.into_iter());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid delay value"));
    }

    #[test]
    fn parse_exec_flag() {
        let a = args(&["--exec", "--repo", ".claude"]);
        let result = parse_args(&mut a.into_iter()).unwrap().unwrap();
        assert_eq!(result.repo, PathBuf::from(".claude"));
        assert!(result.exec);
    }

    #[test]
    fn parse_unknown_opt() {
        let a = args(&["--bogus"]);
        let result = parse_args(&mut a.into_iter());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown finalize option"));
    }

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
        // Skip "finalize" and "--exec" to feed back into parse_args
        let mut iter = exec_args.into_iter().skip(2);
        let parsed = parse_args(&mut iter).unwrap().unwrap();
        assert_eq!(parsed.repo, opts.repo);
        assert_eq!(parsed.source, opts.source);
        assert_eq!(parsed.target, opts.target);
        assert_eq!(parsed.delay_secs, opts.delay_secs);
        assert_eq!(parsed.push, opts.push);
        assert_eq!(parsed.log, opts.log);
    }
}

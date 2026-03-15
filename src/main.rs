mod common;
mod list;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{ExitCode, Stdio};

const USAGE: &str = "\
Usage: vc-x1 [OPTIONS] <COMMAND>

Commands:
  list [<path>]       List commits in a jj repo (defaults to current directory)
  finalize [OPTIONS]  Squash working copy into target commit (daemonizes by default)

Options:
  -V, --version  Print version
  -h, --help     Print this help message";

const FINALIZE_USAGE: &str = "\
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
enum Command {
    Version,
    Help,
    FinalizeHelp,
    List { path: PathBuf },
    Finalize(FinalizeOpts),
}

#[derive(Debug, PartialEq)]
struct FinalizeOpts {
    repo: PathBuf,
    source: String,
    target: String,
    delay_secs: f64,
    push: bool,
    log: PathBuf,
    foreground: bool,
    exec: bool,
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

fn parse_finalize_args<I>(args: &mut I) -> Result<Command, String>
where
    I: Iterator<Item = String>,
{
    let mut opts = FinalizeOpts::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(Command::FinalizeHelp),
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
                return Err(format!(
                    "unknown finalize option: {other}\n\n{FINALIZE_USAGE}"
                ));
            }
        }
    }

    Ok(Command::Finalize(opts))
}

fn parse_args<I>(args: I) -> Result<Command, String>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let _program = args.next();

    match args.next().as_deref() {
        Some("--version" | "-V") => Ok(Command::Version),
        Some("--help" | "-h" | "help") => Ok(Command::Help),
        Some("list") => {
            let path = match args.next() {
                Some(p) => PathBuf::from(p),
                None => std::env::current_dir().map_err(|e| e.to_string())?,
            };
            Ok(Command::List { path })
        }
        Some("finalize") => parse_finalize_args(&mut args),
        Some(other) => Err(format!("unknown command: {other}\n\n{USAGE}")),
        None => Ok(Command::Help),
    }
}

fn log_msg(log: &Path, msg: &str) {
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

fn build_exec_args(opts: &FinalizeOpts) -> Vec<String> {
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

fn finalize(opts: &FinalizeOpts) -> Result<(), Box<dyn std::error::Error>> {
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

fn main() -> ExitCode {
    let cmd = match parse_args(std::env::args()) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    match cmd {
        Command::Version => {
            println!("vc-x1 {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Command::Help => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Command::FinalizeHelp => {
            println!("{FINALIZE_USAGE}");
            ExitCode::SUCCESS
        }
        Command::List { path } => {
            if let Err(e) = list::list(&path) {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Command::Finalize(opts) => {
            log_msg(&opts.log, "main: finalize entry");
            match finalize(&opts) {
                Ok(()) => {
                    log_msg(&opts.log, "main: finalize exit ok");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    log_msg(&opts.log, &format!("main: finalize exit err={e}"));
                    eprintln!("error: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_version_long() {
        assert_eq!(
            parse_args(args(&["vc-x1", "--version"])),
            Ok(Command::Version)
        );
    }

    #[test]
    fn parse_version_short() {
        assert_eq!(parse_args(args(&["vc-x1", "-V"])), Ok(Command::Version));
    }

    #[test]
    fn parse_help_flag() {
        assert_eq!(parse_args(args(&["vc-x1", "--help"])), Ok(Command::Help));
    }

    #[test]
    fn parse_help_short() {
        assert_eq!(parse_args(args(&["vc-x1", "-h"])), Ok(Command::Help));
    }

    #[test]
    fn parse_help_subcommand() {
        assert_eq!(parse_args(args(&["vc-x1", "help"])), Ok(Command::Help));
    }

    #[test]
    fn parse_no_args_shows_help() {
        assert_eq!(parse_args(args(&["vc-x1"])), Ok(Command::Help));
    }

    #[test]
    fn parse_list_with_path() {
        assert_eq!(
            parse_args(args(&["vc-x1", "list", "/some/path"])),
            Ok(Command::List {
                path: PathBuf::from("/some/path")
            })
        );
    }

    #[test]
    fn parse_list_no_path_uses_cwd() {
        let result = parse_args(args(&["vc-x1", "list"]));
        assert!(result.is_ok());
        if let Ok(Command::List { path }) = result {
            assert!(path.is_absolute());
        } else {
            panic!("expected List");
        }
    }

    #[test]
    fn parse_unknown_command() {
        let result = parse_args(args(&["vc-x1", "bogus"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown command: bogus"));
    }

    #[test]
    fn parse_finalize_defaults() {
        assert_eq!(
            parse_args(args(&["vc-x1", "finalize"])),
            Ok(Command::Finalize(FinalizeOpts::default()))
        );
    }

    #[test]
    fn parse_finalize_all_opts() {
        assert_eq!(
            parse_args(args(&[
                "vc-x1",
                "finalize",
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
            ])),
            Ok(Command::Finalize(FinalizeOpts {
                repo: PathBuf::from(".claude"),
                source: "@".to_string(),
                target: "@-".to_string(),
                delay_secs: 2.5,
                push: true,
                log: PathBuf::from("/tmp/test.log"),
                foreground: true,
                exec: false,
            }))
        );
    }

    #[test]
    fn parse_finalize_partial_opts() {
        assert_eq!(
            parse_args(args(&["vc-x1", "finalize", "--repo", ".claude", "--push"])),
            Ok(Command::Finalize(FinalizeOpts {
                repo: PathBuf::from(".claude"),
                push: true,
                ..FinalizeOpts::default()
            }))
        );
    }

    #[test]
    fn parse_finalize_missing_value() {
        let result = parse_args(args(&["vc-x1", "finalize", "--repo"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--repo requires a value"));
    }

    #[test]
    fn parse_finalize_bad_delay() {
        let result = parse_args(args(&["vc-x1", "finalize", "--delay", "abc"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid delay value"));
    }

    #[test]
    fn parse_finalize_exec_flag() {
        assert_eq!(
            parse_args(args(&["vc-x1", "finalize", "--exec", "--repo", ".claude"])),
            Ok(Command::Finalize(FinalizeOpts {
                repo: PathBuf::from(".claude"),
                exec: true,
                ..FinalizeOpts::default()
            }))
        );
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
        // Parse them back (prepend program name)
        let mut full_args = vec!["vc-x1".to_string()];
        full_args.extend(exec_args);
        let result = parse_args(full_args).unwrap();
        if let Command::Finalize(parsed) = result {
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

    #[test]
    fn parse_finalize_unknown_opt() {
        let result = parse_args(args(&["vc-x1", "finalize", "--bogus"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown finalize option"));
    }
}

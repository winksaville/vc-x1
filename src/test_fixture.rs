use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Args;
use log::{debug, info};

use crate::common::{mkdir_p, run, write_file};

/// Create a throwaway dual-repo jj fixture for testing.
///
/// Mirrors the real dual-repo layout created by `vc-x1 init` minus the
/// GitHub side and the `~/.claude/projects/` symlink: app repo at
/// `<base>/work/`, bot session repo at `<base>/work/.claude/`, and a local
/// bare-git remote per repo under `<base>/remote-*.git/`. Both repos carry
/// `.vc-config.toml`, `.gitignore`, an initial described commit with
/// matching `ochid:` trailers, and a tracked `main` bookmark.
#[derive(Args, Debug)]
pub struct TestFixtureArgs {
    /// Fixture root directory [default: $TMPDIR/vc-x1-test-<timestamp>]
    #[arg(long, value_name = "PATH")]
    pub path: Option<PathBuf>,
}

const VC_CONFIG_CODE: &str = r#"# vc-config: Vibe Coding workspace configuration
[workspace]
path = "/"
other-repo = ".claude"
"#;

const VC_CONFIG_SESSION: &str = r#"# vc-config: Vibe Coding workspace configuration
[workspace]
path = "/.claude"
other-repo = ".."
"#;

const GITIGNORE_CODE: &str = "/target
/.claude
/.git
/.jj
";

const GITIGNORE_SESSION: &str = ".git
.jj
";

fn jj_chid(rev: &str, cwd: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let full = run(
        "jj",
        &[
            "log",
            "-r",
            rev,
            "--no-graph",
            "-T",
            "change_id",
            "--limit",
            "1",
        ],
        cwd,
    )?;
    Ok(full[..full.len().min(12)].to_string())
}

pub fn test_fixture(args: &TestFixtureArgs) -> Result<(), Box<dyn std::error::Error>> {
    debug!("test_fixture: enter");

    let base = match &args.path {
        Some(p) => p.clone(),
        None => {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| format!("time error: {e}"))?
                .as_nanos();
            std::env::temp_dir().join(format!("vc-x1-test-{ts}"))
        }
    };

    if base.exists() {
        return Err(format!("'{}' already exists", base.display()).into());
    }

    info!("Creating test fixture at {}", base.display());

    let remote_code = base.join("remote-code.git");
    let remote_claude = base.join("remote-claude.git");
    let work_dir = base.join("work");
    let session_dir = work_dir.join(".claude");

    mkdir_p(&base)?;
    mkdir_p(&remote_code)?;
    mkdir_p(&remote_claude)?;
    mkdir_p(&work_dir)?;
    mkdir_p(&session_dir)?;

    info!("Step 1: Initializing bare git remotes...");
    run("git", &["init", "--bare"], &remote_code)?;
    run("git", &["init", "--bare"], &remote_claude)?;

    info!("Step 2: Initializing work repo (jj colocated)...");
    run("jj", &["git", "init", "--colocate"], &work_dir)?;
    write_file(&work_dir.join(".vc-config.toml"), VC_CONFIG_CODE)?;
    write_file(&work_dir.join(".gitignore"), GITIGNORE_CODE)?;

    info!("Step 3: Initializing .claude session repo (jj colocated)...");
    run("jj", &["git", "init", "--colocate"], &session_dir)?;
    write_file(&session_dir.join(".vc-config.toml"), VC_CONFIG_SESSION)?;
    write_file(&session_dir.join(".gitignore"), GITIGNORE_SESSION)?;

    info!("Step 4: Initial commits with placeholder ochids...");
    run(
        "jj",
        &["commit", "-m", "initial commit\n\nochid: /none"],
        &work_dir,
    )?;
    run(
        "jj",
        &["commit", "-m", "initial commit\n\nochid: /none"],
        &session_dir,
    )?;

    info!("Step 5: Setting ochid cross-references...");
    let code_chid = jj_chid("@-", &work_dir)?;
    let session_chid = jj_chid("@-", &session_dir)?;
    let code_desc = format!("initial commit\n\nochid: /.claude/{session_chid}");
    let session_desc = format!("initial commit\n\nochid: /{code_chid}");
    run("jj", &["describe", "@-", "-m", &code_desc], &work_dir)?;
    run("jj", &["describe", "@-", "-m", &session_desc], &session_dir)?;

    info!("Step 6: Setting bookmarks and wiring remotes...");
    run("jj", &["bookmark", "set", "main", "-r", "@-"], &work_dir)?;
    run("jj", &["bookmark", "set", "main", "-r", "@-"], &session_dir)?;
    let code_url = remote_code.canonicalize()?.to_string_lossy().into_owned();
    let claude_url = remote_claude.canonicalize()?.to_string_lossy().into_owned();
    run("git", &["remote", "add", "origin", &code_url], &work_dir)?;
    run(
        "git",
        &["remote", "add", "origin", &claude_url],
        &session_dir,
    )?;

    info!("Step 7: Pushing main to both remotes...");
    run("jj", &["git", "push", "--bookmark", "main"], &work_dir)?;
    run("jj", &["git", "push", "--bookmark", "main"], &session_dir)?;

    info!("");
    info!("Fixture ready:");
    info!("  Code repo:     {}", work_dir.display());
    info!("  Session repo:  {}", session_dir.display());
    info!("  Code remote:   {}", remote_code.display());
    info!("  Claude remote: {}", remote_claude.display());
    info!("");
    info!("Try:");
    info!(
        "  vc-x1 finalize --repo {} --push main --detach",
        work_dir.display()
    );
    info!(
        "  vc-x1 finalize --repo {} --squash --push main --detach",
        session_dir.display()
    );

    debug!("test_fixture: exit");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cli, Commands};
    use clap::Parser;

    fn parse(args: &[&str]) -> TestFixtureArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::TestFixture(a) => a,
            _ => panic!("expected TestFixture"),
        }
    }

    #[test]
    fn defaults() {
        let args = parse(&["vc-x1", "test-fixture"]);
        assert!(args.path.is_none());
    }

    #[test]
    fn with_path() {
        let args = parse(&["vc-x1", "test-fixture", "--path", "/tmp/foo"]);
        assert_eq!(args.path, Some(PathBuf::from("/tmp/foo")));
    }

    #[test]
    fn unknown_opt() {
        let err = Cli::try_parse_from(["vc-x1", "test-fixture", "--bogus"])
            .unwrap_err()
            .to_string();
        assert!(err.contains("--bogus"));
    }

    #[test]
    fn vc_config_contents() {
        assert!(VC_CONFIG_CODE.contains("path = \"/\""));
        assert!(VC_CONFIG_CODE.contains("other-repo = \".claude\""));
        assert!(VC_CONFIG_SESSION.contains("path = \"/.claude\""));
        assert!(VC_CONFIG_SESSION.contains("other-repo = \"..\""));
    }

    #[test]
    fn gitignore_contents() {
        assert!(GITIGNORE_CODE.contains("/.claude"));
        assert!(GITIGNORE_CODE.contains("/.jj"));
        assert!(GITIGNORE_SESSION.contains(".jj"));
    }
}

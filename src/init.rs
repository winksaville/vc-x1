use std::path::{Path, PathBuf};

use clap::Args;

use crate::symlink;

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Project name (directory and GitHub repo name)
    #[arg(value_name = "NAME")]
    pub name: String,

    /// GitHub user or organization
    #[arg(long)]
    pub owner: Option<String>,

    /// Parent directory to create project in [default: cwd]
    #[arg(long)]
    pub dir: Option<PathBuf>,

    /// Create private GitHub repos (default: public)
    #[arg(long)]
    pub private: bool,

    /// Dry run — show what would be done without executing
    #[arg(long)]
    pub dry_run: bool,

    /// Max push retries after repo creation [default: 5]
    #[arg(long, default_value_t = 5)]
    pub push_retries: u32,

    /// Seconds between push retries [default: 3]
    #[arg(long, default_value_t = 3)]
    pub push_retry_delay: u64,

    /// Verbose output (show retry details)
    #[arg(short, long)]
    pub verbose: bool,
}

/// Run a command with retries, sleeping between attempts.
fn run_retry(
    cmd: &str,
    args: &[&str],
    cwd: &Path,
    retries: u32,
    delay_secs: u64,
    verbose: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut last_err = String::new();
    for attempt in 1..=retries {
        match run(cmd, args, cwd) {
            Ok(out) => {
                if attempt > 1 {
                    eprintln!("  succeeded after {attempt} attempts");
                }
                return Ok(out);
            }
            Err(e) => {
                last_err = e.to_string();
                if attempt < retries {
                    if verbose {
                        eprintln!("  attempt {attempt}/{retries} failed: {last_err}");
                    }
                    eprintln!("  retrying in {delay_secs}s...");
                    std::thread::sleep(std::time::Duration::from_secs(delay_secs));
                }
            }
        }
    }
    Err(format!("failed after {retries} attempts: {last_err}").into())
}

/// Run a command, printing it first. Returns stdout on success.
fn run(cmd: &str, args: &[&str], cwd: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let args_str = args.join(" ");
    eprintln!("  $ {cmd} {args_str}");
    let output = std::process::Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("failed to run {cmd}: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{cmd} {args_str} failed: {stderr}").into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get the current GitHub user via `gh api user`.
fn gh_whoami() -> Result<String, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("gh")
        .args(["api", "user", "--jq", ".login"])
        .output()
        .map_err(|e| format!("failed to run gh: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh api user failed (is gh authenticated?): {stderr}").into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Check if a GitHub repo exists.
fn gh_repo_exists(owner: &str, name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let full = format!("{owner}/{name}");
    let output = std::process::Command::new("gh")
        .args(["repo", "view", &full])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("failed to run gh: {e}"))?;
    Ok(output.success())
}

const VC_CONFIG_CODE: &str = r#"# vc-config: Vibe Coding workspace configuration
#
# workspace-path is this repo's path relative to the workspace root.
# Used to resolve changeID paths in git trailers (e.g. ochid: /changeID).
# other-repo is the relative path to the counterpart repo.

[workspace]
path = "/"
other-repo = ".claude"
"#;

const VC_CONFIG_SESSION: &str = r#"# vc-config: Vibe Coding workspace configuration
#
# workspace-path is this repo's path relative to the workspace root.
# Used to resolve changeID paths in git trailers (e.g. ochid: /.claude/changeID).
# other-repo is the relative path to the counterpart repo.

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

pub fn init(args: &InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    let parent_dir = match &args.dir {
        Some(d) => d.clone(),
        None => std::env::current_dir()?,
    };
    let project_dir = parent_dir.join(&args.name);
    let session_dir = project_dir.join(".claude");

    // Preflight checks
    eprintln!("Preflight checks...");

    // Check tools
    std::process::Command::new("gh")
        .arg("auth")
        .arg("status")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|_| "gh is not installed")?
        .success()
        .then_some(())
        .ok_or("gh is not authenticated (run: gh auth login)")?;

    std::process::Command::new("jj")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .status()
        .map_err(|_| "jj is not installed")?;

    let owner = match &args.owner {
        Some(o) => o.clone(),
        None => gh_whoami()?,
    };

    if project_dir.exists() {
        return Err(format!("'{}' already exists", project_dir.display()).into());
    }

    let code_repo = format!("{owner}/{}", args.name);
    let session_repo = format!("{owner}/{}.claude", args.name);

    if gh_repo_exists(&owner, &args.name)? {
        return Err(format!("GitHub repo '{code_repo}' already exists").into());
    }
    if gh_repo_exists(&owner, &format!("{}.claude", args.name))? {
        return Err(format!("GitHub repo '{session_repo}' already exists").into());
    }

    let visibility = if args.private {
        "--private"
    } else {
        "--public"
    };

    if args.dry_run {
        eprintln!("\nDry run — would execute:");
        eprintln!("  1. Create directories: {}", project_dir.display());
        eprintln!("  2. git init both repos, jj git init --colocate session repo");
        eprintln!("  3. Add .vc-config.toml and .gitignore to both");
        eprintln!("  4. jj commit .claude repo (ochid: /none placeholder)");
        eprintln!("  5. gh repo create {session_repo} {visibility}, push");
        eprintln!("  6. Clean .claude, git submodule add");
        eprintln!("  7. git add + git commit code repo with ochid + .gitmodules");
        eprintln!("  8. gh repo create {code_repo} {visibility}, git push");
        eprintln!("  9. jj git init --colocate code repo, set bookmark");
        eprintln!("  10. Fix .claude ochid via git commit --amend, force push");
        eprintln!("  11. jj git init --colocate session repo");
        eprintln!("  12. Create Claude Code symlink");
        return Ok(());
    }

    // Step 1: Create directories
    eprintln!("\nCreating project directories...");
    std::fs::create_dir_all(&session_dir)?;

    // Step 2: git init + jj init
    // Code repo: git only until the end (jj doesn't understand submodules)
    // Session repo: git + jj from the start (no submodule issues)
    eprintln!("\nInitializing repos...");
    run("git", &["init"], &project_dir)?;
    run("git", &["init"], &session_dir)?;
    run("jj", &["git", "init", "--colocate"], &session_dir)?;

    // Step 3: Write config files
    eprintln!("\nWriting config files...");
    std::fs::write(project_dir.join(".vc-config.toml"), VC_CONFIG_CODE)?;
    std::fs::write(project_dir.join(".gitignore"), GITIGNORE_CODE)?;
    std::fs::write(session_dir.join(".vc-config.toml"), VC_CONFIG_SESSION)?;
    std::fs::write(session_dir.join(".gitignore"), GITIGNORE_SESSION)?;

    // Step 4: Commit .claude repo first (need its changeID for code repo ochid)
    eprintln!("\nCommitting session repo...");
    // Code repo has no jj yet, so no changeID — use "none" placeholder.
    // The ochid can be fixed later with fix-desc once jj is initialized.
    let session_commit_body = "Initial commit\n\nochid: /none";
    run("jj", &["commit", "-m", session_commit_body], &session_dir)?;

    // Get the .claude repo's committed changeID for the code repo ochid
    // After jj commit, the committed revision is @-
    let session_chid_output = std::process::Command::new("jj")
        .args([
            "log",
            "-r",
            "@-",
            "--no-graph",
            "-T",
            "change_id",
            "--limit",
            "1",
        ])
        .current_dir(&session_dir)
        .output()?;
    let session_chid_full = String::from_utf8_lossy(&session_chid_output.stdout)
        .trim()
        .to_string();
    let session_chid = &session_chid_full[..session_chid_full.len().min(12)];

    // Step 5: Create .claude GitHub repo and push
    eprintln!("\nCreating GitHub repo {session_repo}...");
    run(
        "gh",
        &["repo", "create", &session_repo, visibility],
        &project_dir,
    )?;

    let session_url = format!("git@github.com:{session_repo}.git");
    run(
        "git",
        &["remote", "add", "origin", &session_url],
        &session_dir,
    )?;
    // Set the bookmark before pushing
    run("jj", &["bookmark", "set", "main", "-r", "@-"], &session_dir)?;
    run_retry(
        "jj",
        &["git", "push", "--bookmark", "main"],
        &session_dir,
        args.push_retries,
        args.push_retry_delay,
        args.verbose,
    )?;

    // Step 6: Clean .claude contents and add as submodule
    eprintln!("\nAdding .claude as submodule...");
    // Remove the .claude directory contents so git submodule add can work
    std::fs::remove_dir_all(&session_dir)?;

    run(
        "git",
        &["submodule", "add", "--force", &session_url, ".claude"],
        &project_dir,
    )?;

    // Step 7: Commit code repo using git (jj doesn't understand submodules)
    eprintln!("\nCommitting code repo...");
    let code_commit_body = format!("Initial commit\n\nochid: /.claude/{session_chid}");
    run("git", &["add", "."], &project_dir)?;
    run("git", &["commit", "-m", &code_commit_body], &project_dir)?;

    // Step 8: Create code GitHub repo and push
    eprintln!("\nCreating GitHub repo {code_repo}...");
    run(
        "gh",
        &["repo", "create", &code_repo, visibility],
        &project_dir,
    )?;

    let code_url = format!("git@github.com:{code_repo}.git");
    run("git", &["remote", "add", "origin", &code_url], &project_dir)?;
    run_retry(
        "git",
        &["push", "origin", "main"],
        &project_dir,
        args.push_retries,
        args.push_retry_delay,
        args.verbose,
    )?;

    // Step 9: Init jj on code repo and get its changeID
    eprintln!("\nInitializing jj...");
    run("jj", &["git", "init", "--colocate"], &project_dir)?;
    run("jj", &["bookmark", "set", "main", "-r", "@-"], &project_dir)?;

    // Step 10: Fix session repo ochid using pure git (before jj init to avoid divergence)
    eprintln!("\nFixing session repo ochid...");
    let code_chid_output = std::process::Command::new("jj")
        .args([
            "log",
            "-r",
            "@-",
            "--no-graph",
            "-T",
            "change_id",
            "--limit",
            "1",
        ])
        .current_dir(&project_dir)
        .output()?;
    let code_chid_full = String::from_utf8_lossy(&code_chid_output.stdout)
        .trim()
        .to_string();
    let code_chid = &code_chid_full[..code_chid_full.len().min(12)];
    let fixed_session_body = format!("Initial commit\n\nochid: /{code_chid}");
    // Use git commit --amend before jj init so jj only ever sees one version
    run(
        "git",
        &["commit", "--amend", "-m", &fixed_session_body],
        &session_dir,
    )?;
    run("git", &["push", "--force", "origin", "main"], &session_dir)?;

    // Now init jj on the session repo (clean state, no divergence)
    run("jj", &["git", "init", "--colocate"], &session_dir)?;

    // Step 12: Create Claude Code symlink
    eprintln!("\nCreating Claude Code symlink...");
    let symlink_dir = {
        let home =
            std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
        PathBuf::from(home).join(".claude").join("projects")
    };

    let meta = symlink::probe_symlink(
        &symlink_dir.join(symlink::encode_path(
            project_dir
                .to_str()
                .ok_or("project path is not valid UTF-8")?,
        )),
    );
    let plan = symlink::compute_plan(&project_dir, Path::new(".claude"), &symlink_dir, meta)?;
    symlink::execute_plan(&plan, false)?;
    eprintln!(
        "  Symlink: {} -> {}",
        plan.symlink_path.display(),
        plan.abs_target.display()
    );

    eprintln!("\nDone! Project created at {}", project_dir.display());
    eprintln!("  Code repo:    {code_repo}");
    eprintln!("  Session repo: {session_repo}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cli, Commands};
    use clap::Parser;

    fn parse(args: &[&str]) -> InitArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Init(a) => a,
            _ => panic!("expected New"),
        }
    }

    fn parse_err(args: &[&str]) -> String {
        Cli::try_parse_from(args).unwrap_err().to_string()
    }

    #[test]
    fn defaults() {
        let args = parse(&["vc-x1", "init", "my-project"]);
        assert_eq!(args.name, "my-project");
        assert!(args.owner.is_none());
        assert!(args.dir.is_none());
        assert!(!args.private);
        assert!(!args.dry_run);
        assert_eq!(args.push_retries, 5);
        assert_eq!(args.push_retry_delay, 3);
        assert!(!args.verbose);
    }

    #[test]
    fn all_opts() {
        let args = parse(&[
            "vc-x1",
            "init",
            "my-project",
            "--owner",
            "myorg",
            "--dir",
            "/tmp/projects",
            "--private",
            "--dry-run",
            "--push-retries",
            "10",
            "--push-retry-delay",
            "5",
            "--verbose",
        ]);
        assert_eq!(args.name, "my-project");
        assert_eq!(args.owner.as_deref(), Some("myorg"));
        assert_eq!(args.dir, Some(PathBuf::from("/tmp/projects")));
        assert!(args.private);
        assert!(args.dry_run);
        assert!(args.verbose);
        assert_eq!(args.push_retries, 10);
        assert_eq!(args.push_retry_delay, 5);
    }

    #[test]
    fn missing_name() {
        let err = parse_err(&["vc-x1", "init"]);
        assert!(err.contains("NAME"));
    }

    #[test]
    fn config_content_code() {
        assert!(VC_CONFIG_CODE.contains("path = \"/\""));
        assert!(VC_CONFIG_CODE.contains("other-repo = \".claude\""));
    }

    #[test]
    fn config_content_session() {
        assert!(VC_CONFIG_SESSION.contains("path = \"/.claude\""));
        assert!(VC_CONFIG_SESSION.contains("other-repo = \"..\""));
    }

    #[test]
    fn gitignore_code_excludes_claude() {
        assert!(GITIGNORE_CODE.contains("/.claude"));
        assert!(GITIGNORE_CODE.contains("/.git"));
        assert!(GITIGNORE_CODE.contains("/.jj"));
    }

    #[test]
    fn gitignore_session_excludes_git() {
        assert!(GITIGNORE_SESSION.contains(".git"));
        assert!(GITIGNORE_SESSION.contains(".jj"));
    }
}

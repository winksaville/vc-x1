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

    /// Verbose output (diagnostic detail on stderr)
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
        match run(cmd, args, cwd, verbose) {
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

use crate::common::run;

/// Get the short (12-char) jj change ID for a revision, without printing.
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
        false,
    )?;
    Ok(full[..full.len().min(12)].to_string())
}

/// Get the current GitHub user via `gh api user`.
fn gh_whoami() -> Result<String, Box<dyn std::error::Error>> {
    run(
        "gh",
        &["api", "user", "--jq", ".login"],
        Path::new("."),
        false,
    )
}

/// Check if a GitHub repo exists.
fn gh_repo_exists(owner: &str, name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let full = format!("{owner}/{name}");
    Ok(run("gh", &["repo", "view", &full], Path::new("."), false).is_ok())
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
    println!("Preflight checks...");

    // Check tools
    run("gh", &["auth", "status"], Path::new("."), false)
        .map_err(|_| "gh is not installed or not authenticated (run: gh auth login)")?;
    run("jj", &["--version"], Path::new("."), false).map_err(|_| "jj is not installed")?;

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
        println!("Dry run — would execute:");
        println!("  1. Create directories: {}", project_dir.display());
        println!("  2. git init + jj git init --colocate on both repos");
        println!("  3. Write .vc-config.toml and .gitignore to both");
        println!("  4. jj commit both with placeholder ochids");
        println!("  5. Get both chids, jj describe both with correct ochids");
        println!("  6. Remove jj from both (git clean -xdf)");
        println!("  7. gh repo create {session_repo} {visibility}, push");
        println!("  8. git submodule add .claude, second commit in code repo");
        println!("  9. gh repo create {code_repo} {visibility}, push");
        println!("  10. jj git init --colocate on both repos");
        println!("  11. Create Claude Code symlink");
        return Ok(());
    }

    let v = args.verbose;

    // Step 1: Create directories
    println!("Step 1: Creating project directories...");
    std::fs::create_dir_all(&session_dir)?;

    // Step 2: git init + jj init on both repos
    println!("Step 2: Initializing repos...");
    run("git", &["init"], &project_dir, v)?;
    run("jj", &["git", "init", "--colocate"], &project_dir, v)?;
    run("git", &["init"], &session_dir, v)?;
    run("jj", &["git", "init", "--colocate"], &session_dir, v)?;

    // Step 3: Write config files
    println!("Step 3: Writing config files...");
    std::fs::write(project_dir.join(".vc-config.toml"), VC_CONFIG_CODE)?;
    std::fs::write(project_dir.join(".gitignore"), GITIGNORE_CODE)?;
    std::fs::write(session_dir.join(".vc-config.toml"), VC_CONFIG_SESSION)?;
    std::fs::write(session_dir.join(".gitignore"), GITIGNORE_SESSION)?;

    // Step 4: jj commit both with placeholder ochids
    println!("Step 4: Committing both repos with placeholder ochids...");
    run(
        "jj",
        &["commit", "-m", "Initial commit\n\nochid: /none"],
        &project_dir,
        v,
    )?;
    run(
        "jj",
        &["commit", "-m", "Initial commit\n\nochid: /none"],
        &session_dir,
        v,
    )?;

    // Step 5: Get both chids, then describe both with correct ochids
    println!("Step 5: Setting ochid cross-references...");
    let code_chid = jj_chid("@-", &project_dir)?;
    let session_chid = jj_chid("@-", &session_dir)?;

    let code_desc = format!("Initial commit\n\nochid: /.claude/{session_chid}");
    let session_desc = format!("Initial commit\n\nochid: /{code_chid}");
    run("jj", &["describe", "@-", "-m", &code_desc], &project_dir, v)?;
    run(
        "jj",
        &["describe", "@-", "-m", &session_desc],
        &session_dir,
        v,
    )?;

    if v {
        let hash = run("git", &["rev-parse", "HEAD"], &project_dir, v)?;
        eprintln!("  code repo: chid={code_chid} hash={hash}");
        let hash = run("git", &["rev-parse", "HEAD"], &session_dir, v)?;
        eprintln!("  .claude:   chid={session_chid} hash={hash}");
    }

    // Step 6: Set bookmarks (creates git branches), then remove jj
    // Bookmarks must be set before removing .jj/ so git has a 'main' branch to push
    println!("Step 6: Setting bookmarks and removing jj...");
    run(
        "jj",
        &["bookmark", "set", "main", "-r", "@-"],
        &project_dir,
        v,
    )?;
    run(
        "jj",
        &["bookmark", "set", "main", "-r", "@-"],
        &session_dir,
        v,
    )?;
    // Clean .claude first, then code repo with --exclude to preserve .claude/
    run("git", &["clean", "-xdf"], &session_dir, v)?;
    run(
        "git",
        &["clean", "-xdf", "--exclude", ".claude"],
        &project_dir,
        v,
    )?;
    // After removing .jj/, git HEAD is detached — reattach to main
    run("git", &["checkout", "main"], &session_dir, v)?;
    run("git", &["checkout", "main"], &project_dir, v)?;

    // Step 7: Create .claude GitHub repo and push
    let session_url = format!("git@github.com:{session_repo}.git");
    println!("Step 7: Creating GitHub repo {session_repo}...");
    run(
        "gh",
        &["repo", "create", &session_repo, visibility],
        &project_dir,
        v,
    )?;
    run(
        "git",
        &["remote", "add", "origin", &session_url],
        &session_dir,
        v,
    )?;
    run_retry(
        "git",
        &["push", "-u", "origin", "main"],
        &session_dir,
        args.push_retries,
        args.push_retry_delay,
        v,
    )?;

    // Step 8: Add .claude as submodule — second commit in code repo
    println!("Step 8: Adding .claude as submodule...");
    // Remove .claude directory so git submodule add can re-clone it
    std::fs::remove_dir_all(&session_dir)?;
    run(
        "git",
        &["submodule", "add", "--force", &session_url, ".claude"],
        &project_dir,
        v,
    )?;
    let submodule_body = format!("Add .claude submodule\n\nochid: /.claude/{session_chid}");
    run("git", &["add", "."], &project_dir, v)?;
    run("git", &["commit", "-m", &submodule_body], &project_dir, v)?;

    // Step 9: Create code GitHub repo and push
    let code_url = format!("git@github.com:{code_repo}.git");
    println!("Step 9: Creating GitHub repo {code_repo}...");
    run(
        "gh",
        &["repo", "create", &code_repo, visibility],
        &project_dir,
        v,
    )?;
    run(
        "git",
        &["remote", "add", "origin", &code_url],
        &project_dir,
        v,
    )?;
    run_retry(
        "git",
        &["push", "-u", "origin", "main"],
        &project_dir,
        args.push_retries,
        args.push_retry_delay,
        v,
    )?;

    // Step 10: Re-initialize jj on both repos
    println!("Step 10: Re-initializing jj on both repos...");
    run("jj", &["git", "init", "--colocate"], &project_dir, v)?;
    run(
        "jj",
        &["bookmark", "set", "main", "-r", "@-"],
        &project_dir,
        v,
    )?;
    run("jj", &["bookmark", "track", "main@origin"], &project_dir, v)?;
    let code_chid_final = jj_chid("@-", &project_dir)?;

    run("jj", &["git", "init", "--colocate"], &session_dir, v)?;
    run(
        "jj",
        &["bookmark", "set", "main", "-r", "@-"],
        &session_dir,
        v,
    )?;
    run("jj", &["bookmark", "track", "main@origin"], &session_dir, v)?;
    let session_chid_final = jj_chid("@-", &session_dir)?;

    // Step 11: Create Claude Code symlink
    println!("Step 11: Creating Claude Code symlink...");
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

    println!();
    println!("Done! Project created at {}", project_dir.display());
    println!("  Code repo:    {code_repo}  (chid={code_chid_final})");
    println!("  Session repo: {session_repo}  (chid={session_chid_final})");
    println!(
        "  Symlink:      {} -> {}",
        plan.symlink_path.display(),
        plan.abs_target.display()
    );

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

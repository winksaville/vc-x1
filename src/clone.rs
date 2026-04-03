use std::path::{Path, PathBuf};

use clap::Args;

use crate::symlink;

#[derive(Args, Debug)]
pub struct CloneArgs {
    /// GitHub repo (owner/name) or git URL
    #[arg(value_name = "REPO")]
    pub repo: String,

    /// Parent directory to clone into [default: cwd]
    #[arg(long)]
    pub dir: Option<PathBuf>,

    /// Dry run — show what would be done without executing
    #[arg(long)]
    pub dry_run: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
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

/// Derive project name from a repo argument.
///
/// Handles: `owner/name`, `git@github.com:owner/name.git`,
/// `https://github.com/owner/name.git`, `https://github.com/owner/name`
fn derive_name(repo: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Strip trailing .git
    let repo = repo.strip_suffix(".git").unwrap_or(repo);

    // Take everything after the last `/` or `:`
    let name = repo
        .rsplit_once('/')
        .or_else(|| repo.rsplit_once(':'))
        .map(|(_, name)| name)
        .unwrap_or(repo);

    if name.is_empty() {
        return Err(format!("cannot derive project name from '{repo}'").into());
    }
    Ok(name.to_string())
}

/// Resolve a repo argument to a git clone URL.
///
/// `owner/name` (no scheme, no `:` before `/`) becomes `git@github.com:owner/name.git`.
/// Anything else is passed through as-is.
fn resolve_url(repo: &str) -> String {
    // Already a URL (has scheme or SSH-style colon)
    if repo.contains("://") || repo.contains('@') {
        return repo.to_string();
    }
    // owner/name shorthand — must have exactly one `/` and no other URL indicators
    if repo.matches('/').count() == 1 && !repo.contains(':') {
        return format!("git@github.com:{repo}.git");
    }
    repo.to_string()
}

pub fn clone_repo(args: &CloneArgs) -> Result<(), Box<dyn std::error::Error>> {
    let parent_dir = match &args.dir {
        Some(d) => d.clone(),
        None => std::env::current_dir()?,
    };

    let name = derive_name(&args.repo)?;
    let project_dir = parent_dir.join(&name);
    let session_dir = project_dir.join(".claude");
    let url = resolve_url(&args.repo);

    if args.dry_run {
        eprintln!("Dry run — would execute:");
        eprintln!("  1. git clone --recursive {url} {name}");
        eprintln!("  2. jj git init --colocate in {name}/");
        eprintln!("  3. jj git init --colocate in {name}/.claude/");
        eprintln!("  4. Create Claude Code symlink");
        return Ok(());
    }

    // Preflight
    if project_dir.exists() {
        return Err(format!("'{}' already exists", project_dir.display()).into());
    }

    std::process::Command::new("jj")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .status()
        .map_err(|_| "jj is not installed")?;

    // Step 1: git clone --recursive
    eprintln!("Cloning {url}...");
    let project_str = project_dir
        .to_str()
        .ok_or("project path is not valid UTF-8")?;
    run(
        "git",
        &["clone", "--recursive", &url, project_str],
        &parent_dir,
    )?;

    // Step 2: jj git init --colocate in code repo
    eprintln!("\nInitializing jj in code repo...");
    run("jj", &["git", "init", "--colocate"], &project_dir)?;

    // Step 3: jj git init --colocate in session repo (if submodule exists)
    if session_dir.exists() {
        // Align submodule to origin/main before jj init.
        // init's ochid fixup amends the .claude commit after the code repo
        // records the pre-amend hash, so the submodule checkout and
        // origin/main can point to different commits with the same change ID.
        // Without this, jj sees both and reports "divergent".
        eprintln!("\nAligning session repo to origin/main...");
        run(
            "git",
            &["checkout", "-B", "main", "origin/main"],
            &session_dir,
        )?;

        eprintln!("\nInitializing jj in session repo...");
        run("jj", &["git", "init", "--colocate"], &session_dir)?;
    } else {
        eprintln!("\nNote: no .claude submodule found — skipping session repo jj init");
    }

    // Step 4: Create Claude Code symlink
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

    eprintln!("\nDone! Project cloned to {}", project_dir.display());
    if args.verbose {
        eprintln!("  Code repo:    {project_str}");
        eprintln!("  Session repo: {}", session_dir.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cli, Commands};
    use clap::Parser;

    fn parse(args: &[&str]) -> CloneArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Clone(a) => a,
            _ => panic!("expected Clone"),
        }
    }

    fn parse_err(args: &[&str]) -> String {
        Cli::try_parse_from(args).unwrap_err().to_string()
    }

    #[test]
    fn defaults() {
        let args = parse(&["vc-x1", "clone", "owner/repo"]);
        assert_eq!(args.repo, "owner/repo");
        assert!(args.dir.is_none());
        assert!(!args.dry_run);
        assert!(!args.verbose);
    }

    #[test]
    fn all_opts() {
        let args = parse(&[
            "vc-x1",
            "clone",
            "owner/repo",
            "--dir",
            "/tmp/projects",
            "--dry-run",
            "--verbose",
        ]);
        assert_eq!(args.repo, "owner/repo");
        assert_eq!(args.dir, Some(PathBuf::from("/tmp/projects")));
        assert!(args.dry_run);
        assert!(args.verbose);
    }

    #[test]
    fn missing_repo() {
        let err = parse_err(&["vc-x1", "clone"]);
        assert!(err.contains("REPO"));
    }

    #[test]
    fn derive_name_owner_slash_name() {
        assert_eq!(derive_name("owner/my-project").unwrap(), "my-project");
    }

    #[test]
    fn derive_name_ssh_url() {
        assert_eq!(
            derive_name("git@github.com:owner/my-project.git").unwrap(),
            "my-project"
        );
    }

    #[test]
    fn derive_name_https_url() {
        assert_eq!(
            derive_name("https://github.com/owner/my-project.git").unwrap(),
            "my-project"
        );
    }

    #[test]
    fn derive_name_https_no_suffix() {
        assert_eq!(
            derive_name("https://github.com/owner/my-project").unwrap(),
            "my-project"
        );
    }

    #[test]
    fn derive_name_bare() {
        assert_eq!(derive_name("my-project").unwrap(), "my-project");
    }

    #[test]
    fn resolve_url_shorthand() {
        assert_eq!(resolve_url("owner/repo"), "git@github.com:owner/repo.git");
    }

    #[test]
    fn resolve_url_ssh_passthrough() {
        let url = "git@github.com:owner/repo.git";
        assert_eq!(resolve_url(url), url);
    }

    #[test]
    fn resolve_url_https_passthrough() {
        let url = "https://github.com/owner/repo.git";
        assert_eq!(resolve_url(url), url);
    }
}

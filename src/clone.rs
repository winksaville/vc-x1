use std::path::{Path, PathBuf};

use clap::Args;
use log::info;

use crate::common::run;
use crate::symlink;

#[derive(Args, Debug)]
pub struct CloneArgs {
    /// GitHub repo (owner/name) or git URL
    #[arg(value_name = "REPO")]
    pub repo: String,

    /// Local directory name [default: derived from REPO]
    #[arg(value_name = "NAME")]
    pub name: Option<String>,

    /// Parent directory to clone into [default: cwd]
    #[arg(long)]
    pub dir: Option<PathBuf>,

    /// Dry run — show what would be done without executing
    #[arg(long)]
    pub dry_run: bool,
}

/// Derive project name from a repo argument.
///
/// Handles: `owner/name`, `git@github.com:owner/name.git`,
/// `https://github.com/owner/name.git`, `https://github.com/owner/name`
fn derive_name(repo: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Strip trailing .git
    let repo = repo.strip_suffix(".git").unwrap_or(repo); // OK: repo arg may not end in .git

    // Take everything after the last `/` or `:`
    let name = repo
        .rsplit_once('/')
        .or_else(|| repo.rsplit_once(':'))
        .map(|(_, name)| name)
        .unwrap_or(repo); // OK: no separator → whole string is the name

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

/// Derive the session repo URL from the code repo URL.
///
/// Appends `.claude` before `.git` (or at the end):
/// - `git@github.com:owner/name.git` → `git@github.com:owner/name.claude.git`
/// - `https://github.com/owner/name.git` → `https://github.com/owner/name.claude.git`
/// - `https://github.com/owner/name` → `https://github.com/owner/name.claude`
fn derive_session_url(url: &str) -> String {
    if let Some(base) = url.strip_suffix(".git") {
        format!("{base}.claude.git")
    } else {
        format!("{url}.claude")
    }
}

pub fn clone_repo(args: &CloneArgs) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("clone: enter");
    let parent_dir = match &args.dir {
        Some(d) => d.clone(),
        None => std::env::current_dir()?,
    };

    let name = match &args.name {
        Some(n) => n.clone(),
        None => derive_name(&args.repo)?,
    };
    let project_dir = parent_dir.join(&name);
    let session_dir = project_dir.join(".claude");
    let url = resolve_url(&args.repo);

    let session_url = derive_session_url(&url);

    if args.dry_run {
        info!("Dry run — would execute:");
        info!("  1. git clone {url} {name}");
        info!("  2. git clone {session_url} {name}/.claude");
        info!("  3. jj git init --colocate in {name}/");
        info!("  4. jj git init --colocate in {name}/.claude/");
        info!("  5. Create Claude Code symlink");
        return Ok(());
    }

    // Preflight
    if project_dir.exists() {
        return Err(format!("'{}' already exists", project_dir.display()).into());
    }

    run("jj", &["--version"], Path::new(".")).map_err(|_| "jj is not installed")?;

    // Step 1: Clone code repo
    let project_str = project_dir
        .to_str()
        .ok_or("project path is not valid UTF-8")?;
    info!("Step 1: Cloning {url}...");
    run("git", &["clone", &url, project_str], &parent_dir)?;

    // Step 2: Clone session repo
    let session_str = session_dir
        .to_str()
        .ok_or("session path is not valid UTF-8")?;
    info!("Step 2: Cloning {session_url}...");
    match run("git", &["clone", &session_url, session_str], &parent_dir) {
        Ok(_) => {}
        Err(e) => {
            info!("Step 2: No session repo found ({e}) — skipping");
        }
    }

    // Step 3: jj git init --colocate in code repo. Note: colocate after a
    // plain `git clone` does NOT auto-establish bookmark tracking — jj
    // emits a hint to run `jj bookmark track …`. We do that explicitly,
    // then assert via verify_tracking.
    info!("Step 3: Initializing jj in code repo...");
    run("jj", &["git", "init", "--colocate"], &project_dir)?;
    run(
        "jj",
        &["bookmark", "track", "main", "--remote=origin"],
        &project_dir,
    )?;
    crate::common::verify_tracking(&project_dir, "main")?;

    // Step 4: jj git init --colocate in session repo (if cloned)
    if session_dir.exists() {
        info!("Step 4: Initializing jj in session repo...");
        run("jj", &["git", "init", "--colocate"], &session_dir)?;
        run(
            "jj",
            &["bookmark", "track", "main", "--remote=origin"],
            &session_dir,
        )?;
        crate::common::verify_tracking(&session_dir, "main")?;
    } else {
        info!("Step 4: No session repo — skipping jj init");
    }

    // Step 5: Create Claude Code symlink
    info!("Step 5: Creating Claude Code symlink...");
    let symlink_dir = {
        let home =
            std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
        PathBuf::from(home).join(".claude").join("projects")
    };

    let sl = symlink::SymLink::new(&project_dir, Path::new(".claude"), &symlink_dir)?;
    sl.create(false)?;

    info!("");
    info!("Done! Project cloned to {}", project_dir.display());
    info!("  Code repo:    {project_str}");
    info!("  Session repo: {}", session_dir.display());
    info!(
        "  Symlink:      {} -> {}",
        sl.symlink_path.display(),
        sl.abs_target.display()
    );

    log::debug!("clone: exit");
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
        assert!(args.name.is_none());
        assert!(args.dir.is_none());
        assert!(!args.dry_run);
    }

    #[test]
    fn with_name() {
        let args = parse(&["vc-x1", "clone", "owner/repo", "my-dir"]);
        assert_eq!(args.repo, "owner/repo");
        assert_eq!(args.name.as_deref(), Some("my-dir"));
    }

    #[test]
    fn all_opts() {
        let args = parse(&[
            "vc-x1",
            "clone",
            "owner/repo",
            "my-dir",
            "--dir",
            "/tmp/projects",
            "--dry-run",
        ]);
        assert_eq!(args.repo, "owner/repo");
        assert_eq!(args.name.as_deref(), Some("my-dir"));
        assert_eq!(args.dir, Some(PathBuf::from("/tmp/projects")));
        assert!(args.dry_run);
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

    #[test]
    fn session_url_ssh() {
        assert_eq!(
            derive_session_url("git@github.com:owner/repo.git"),
            "git@github.com:owner/repo.claude.git"
        );
    }

    #[test]
    fn session_url_https_with_git() {
        assert_eq!(
            derive_session_url("https://github.com/owner/repo.git"),
            "https://github.com/owner/repo.claude.git"
        );
    }

    #[test]
    fn session_url_https_no_suffix() {
        assert_eq!(
            derive_session_url("https://github.com/owner/repo"),
            "https://github.com/owner/repo.claude"
        );
    }
}

use std::path::{Path, PathBuf};

use clap::Args;
use log::info;

use crate::common::run;
use crate::repo_url::{derive_name, derive_session_url, resolve_url};
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

    // Unit tests for derive_name / resolve_url / derive_session_url
    // live in src/repo_url.rs alongside the lifted functions.
}

use std::path::{Path, PathBuf};

use clap::Args;

/// What action the symlink operation needs to take.
#[derive(Debug, PartialEq, Eq)]
pub enum SymlinkAction {
    /// Nothing exists at the symlink path — create it.
    Create,
    /// An existing symlink points somewhere else — caller must decide whether to replace.
    Replace { current_target: PathBuf },
    /// The symlink already points to the correct target — nothing to do.
    AlreadyCorrect,
}

/// The result of planning a symlink operation.
#[derive(Debug)]
pub struct SymlinkPlan {
    /// Absolute path to the target directory (what the symlink points to).
    pub abs_target: PathBuf,
    /// Full path of the symlink to create/replace.
    pub symlink_path: PathBuf,
    /// What action is needed.
    pub action: SymlinkAction,
}

/// Encode a path the way Claude Code does: replace `/` and `.` with `-`.
pub fn encode_path(path: &str) -> String {
    path.replace(['/', '.'], "-")
}

/// Compute the symlink plan without performing any I/O.
///
/// # Arguments
/// * `cwd` — current working directory (used to derive the symlink name)
/// * `target` — the directory the symlink should point to (resolved to absolute)
/// * `symlink_dir` — parent directory for the symlink (e.g. `~/.claude/projects`)
/// * `target_exists` — whether the target directory exists on disk
/// * `symlink_meta` — what currently exists at the symlink path:
///   - `None` — nothing exists
///   - `Some(None)` — exists but is not a symlink (file or directory)
///   - `Some(Some(path))` — existing symlink pointing to `path`
pub fn compute_plan(
    cwd: &Path,
    target: &Path,
    symlink_dir: &Path,
    symlink_meta: Option<Option<PathBuf>>,
) -> Result<SymlinkPlan, String> {
    // Resolve target to absolute path
    let abs_target = if target.is_absolute() {
        target.to_path_buf()
    } else {
        cwd.join(target)
    };

    // Derive symlink name from cwd
    let cwd_str = cwd
        .to_str()
        .ok_or_else(|| "current directory path is not valid UTF-8".to_string())?;
    let symlink_name = encode_path(cwd_str);
    let symlink_path = symlink_dir.join(symlink_name);

    let action = match symlink_meta {
        None => SymlinkAction::Create,
        Some(None) => {
            return Err(format!(
                "'{}' exists and is not a symlink",
                symlink_path.display()
            ));
        }
        Some(Some(current_target)) => {
            if current_target == abs_target {
                SymlinkAction::AlreadyCorrect
            } else {
                SymlinkAction::Replace { current_target }
            }
        }
    };

    Ok(SymlinkPlan {
        abs_target,
        symlink_path,
        action,
    })
}

/// Read what exists at a path without following symlinks.
///
/// Returns:
/// - `None` — nothing exists
/// - `Some(None)` — exists but is not a symlink
/// - `Some(Some(target))` — symlink pointing to target
pub fn probe_symlink(path: &Path) -> Option<Option<PathBuf>> {
    match path.symlink_metadata() {
        Err(_) => None,
        Ok(meta) => {
            if meta.is_symlink() {
                Some(Some(std::fs::read_link(path).unwrap_or_default()))
            } else {
                Some(None)
            }
        }
    }
}

/// Execute a symlink plan: create the target dir if needed, create/replace the symlink.
pub fn execute_plan(plan: &SymlinkPlan, create_target: bool) -> Result<(), String> {
    if create_target && !plan.abs_target.exists() {
        std::fs::create_dir_all(&plan.abs_target)
            .map_err(|e| format!("cannot create target '{}': {e}", plan.abs_target.display()))?;
    }

    // Ensure symlink parent directory exists
    if let Some(parent) = plan.symlink_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!(
                "cannot create symlink directory '{}': {e}",
                parent.display()
            )
        })?;
    }

    match &plan.action {
        SymlinkAction::Create => {}
        SymlinkAction::Replace { .. } => {
            std::fs::remove_file(&plan.symlink_path).map_err(|e| {
                format!(
                    "cannot remove existing symlink '{}': {e}",
                    plan.symlink_path.display()
                )
            })?;
        }
        SymlinkAction::AlreadyCorrect => return Ok(()),
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(&plan.abs_target, &plan.symlink_path).map_err(|e| {
        format!(
            "cannot create symlink '{}' -> '{}': {e}",
            plan.symlink_path.display(),
            plan.abs_target.display()
        )
    })?;

    #[cfg(not(unix))]
    return Err("symlink creation is only supported on Unix".to_string());

    Ok(())
}

// -- Subcommand --

#[derive(Args, Debug)]
pub struct SymlinkArgs {
    /// Directory to link to [default: .claude]
    #[arg(value_name = "TARGET")]
    pub target: Option<String>,

    /// Directory for symlink [default: ~/.claude/projects]
    #[arg(long, value_name = "PATH")]
    pub symlink_dir: Option<PathBuf>,

    /// List contents of symlinked directory after creation
    #[arg(short, long)]
    pub list: bool,

    /// Replace existing symlink without prompting
    #[arg(short, long)]
    pub yes: bool,

    /// Verbose output (diagnostic detail on stderr)
    #[arg(short, long)]
    pub verbose: bool,
}

pub fn symlink(args: &SymlinkArgs) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;

    let target = match &args.target {
        Some(t) => PathBuf::from(t),
        None => PathBuf::from(".claude"),
    };

    let symlink_dir = match &args.symlink_dir {
        Some(d) => d.clone(),
        None => {
            let home = std::env::var("HOME")
                .map_err(|_| "HOME environment variable not set".to_string())?;
            PathBuf::from(home).join(".claude").join("projects")
        }
    };

    let abs_target = if target.is_absolute() {
        target.clone()
    } else {
        cwd.join(&target)
    };

    let symlink_name = encode_path(
        cwd.to_str()
            .ok_or("current directory path is not valid UTF-8")?,
    );
    let symlink_path = symlink_dir.join(&symlink_name);

    let meta = probe_symlink(&symlink_path);
    let plan = compute_plan(&cwd, &target, &symlink_dir, meta)?;

    // Handle interactive prompt for replacement
    if let SymlinkAction::Replace { ref current_target } = plan.action {
        log::info!(
            "Existing symlink: {} -> {}",
            plan.symlink_path.display(),
            current_target.display()
        );
        if !args.yes {
            let response = crate::common::prompt("Replace with new target? [y/N] ")?;
            if !response.eq_ignore_ascii_case("y") {
                return Err("aborted".into());
            }
        }
    }

    if plan.action == SymlinkAction::AlreadyCorrect {
        log::info!(
            "Already correct: {} -> {}",
            plan.symlink_path.display(),
            plan.abs_target.display()
        );
    } else {
        execute_plan(&plan, true)?;
        log::info!(
            "Created: {} -> {}",
            plan.symlink_path.display(),
            plan.abs_target.display()
        );
    }

    if args.list {
        log::info!("");
        log::info!("Contents of {}:", abs_target.display());
        for entry in std::fs::read_dir(&abs_target)? {
            let entry = entry?;
            log::info!("  {}", entry.file_name().to_string_lossy());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_path_basic() {
        assert_eq!(
            encode_path("/home/wink/data/prgs/vc-x1"),
            "-home-wink-data-prgs-vc-x1"
        );
    }

    #[test]
    fn encode_path_with_dots() {
        assert_eq!(
            encode_path("/home/wink/.config/test"),
            "-home-wink--config-test"
        );
    }

    #[test]
    fn plan_create_when_nothing_exists() {
        let cwd = Path::new("/home/user/project");
        let target = Path::new(".claude");
        let symlink_dir = Path::new("/home/user/.claude/projects");

        let plan = compute_plan(cwd, target, symlink_dir, None).unwrap();

        assert_eq!(plan.abs_target, PathBuf::from("/home/user/project/.claude"));
        assert_eq!(
            plan.symlink_path,
            PathBuf::from("/home/user/.claude/projects/-home-user-project")
        );
        assert_eq!(plan.action, SymlinkAction::Create);
    }

    #[test]
    fn plan_already_correct() {
        let cwd = Path::new("/home/user/project");
        let target = Path::new(".claude");
        let symlink_dir = Path::new("/home/user/.claude/projects");
        let current = PathBuf::from("/home/user/project/.claude");

        let plan = compute_plan(cwd, target, symlink_dir, Some(Some(current))).unwrap();
        assert_eq!(plan.action, SymlinkAction::AlreadyCorrect);
    }

    #[test]
    fn plan_replace_different_target() {
        let cwd = Path::new("/home/user/project");
        let target = Path::new(".claude");
        let symlink_dir = Path::new("/home/user/.claude/projects");
        let current = PathBuf::from("/home/user/other/.claude");

        let plan = compute_plan(cwd, target, symlink_dir, Some(Some(current.clone()))).unwrap();
        assert_eq!(
            plan.action,
            SymlinkAction::Replace {
                current_target: current
            }
        );
    }

    #[test]
    fn plan_error_not_a_symlink() {
        let cwd = Path::new("/home/user/project");
        let target = Path::new(".claude");
        let symlink_dir = Path::new("/home/user/.claude/projects");

        let err = compute_plan(cwd, target, symlink_dir, Some(None)).unwrap_err();
        assert!(err.contains("not a symlink"));
    }

    #[test]
    fn plan_absolute_target() {
        let cwd = Path::new("/home/user/project");
        let target = Path::new("/tmp/my-claude");
        let symlink_dir = Path::new("/home/user/.claude/projects");

        let plan = compute_plan(cwd, target, symlink_dir, None).unwrap();
        assert_eq!(plan.abs_target, PathBuf::from("/tmp/my-claude"));
    }

    #[test]
    fn probe_nothing() {
        let path = Path::new("/nonexistent/path/that/does/not/exist");
        assert!(probe_symlink(path).is_none());
    }

    #[test]
    fn probe_regular_file() {
        let dir = std::env::temp_dir().join("symlink_test_probe_file");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("regular");
        std::fs::write(&file, "hello").unwrap();

        let result = probe_symlink(&file);
        assert_eq!(result, Some(None)); // exists, not a symlink

        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn probe_existing_symlink() {
        let dir = std::env::temp_dir().join("symlink_test_probe_link");
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("target_dir");
        std::fs::create_dir_all(&target).unwrap();
        let link = dir.join("the_link");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let result = probe_symlink(&link);
        assert_eq!(result, Some(Some(target.clone())));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn execute_create_symlink() {
        let dir = std::env::temp_dir().join("symlink_test_execute_create");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("my_target");
        let symlink_path = dir.join("my_link");

        let plan = SymlinkPlan {
            abs_target: target.clone(),
            symlink_path: symlink_path.clone(),
            action: SymlinkAction::Create,
        };

        execute_plan(&plan, true).unwrap();

        assert!(target.exists()); // create_target=true created it
        assert!(symlink_path.symlink_metadata().unwrap().is_symlink());
        assert_eq!(std::fs::read_link(&symlink_path).unwrap(), target);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn execute_replace_symlink() {
        let dir = std::env::temp_dir().join("symlink_test_execute_replace");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let old_target = dir.join("old_target");
        std::fs::create_dir_all(&old_target).unwrap();
        let new_target = dir.join("new_target");
        std::fs::create_dir_all(&new_target).unwrap();
        let symlink_path = dir.join("the_link");
        std::os::unix::fs::symlink(&old_target, &symlink_path).unwrap();

        let plan = SymlinkPlan {
            abs_target: new_target.clone(),
            symlink_path: symlink_path.clone(),
            action: SymlinkAction::Replace {
                current_target: old_target,
            },
        };

        execute_plan(&plan, false).unwrap();

        assert_eq!(std::fs::read_link(&symlink_path).unwrap(), new_target);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn execute_already_correct_is_noop() {
        let dir = std::env::temp_dir().join("symlink_test_execute_noop");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("target");
        std::fs::create_dir_all(&target).unwrap();

        let plan = SymlinkPlan {
            abs_target: target,
            symlink_path: dir.join("nonexistent_link"),
            action: SymlinkAction::AlreadyCorrect,
        };

        // Should succeed without creating anything
        execute_plan(&plan, false).unwrap();
        assert!(!dir.join("nonexistent_link").exists());

        std::fs::remove_dir_all(&dir).ok();
    }
}

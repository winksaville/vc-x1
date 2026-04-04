use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info};

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

/// A planned symlink operation.
#[derive(Debug)]
pub struct SymLink {
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

/// Read what exists at a path without following symlinks.
///
/// Returns:
/// - `None` — nothing exists
/// - `Some(None)` — exists but is not a symlink
/// - `Some(Some(target))` — symlink pointing to target
fn probe(path: &Path) -> Option<Option<PathBuf>> {
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

impl SymLink {
    /// Plan a symlink operation by probing the filesystem.
    ///
    /// # Arguments
    /// * `cwd` — current working directory (used to derive the symlink name)
    /// * `target` — the directory the symlink should point to (resolved to absolute)
    /// * `symlink_dir` — parent directory for the symlink (e.g. `~/.claude/projects`)
    pub fn new(cwd: &Path, target: &Path, symlink_dir: &Path) -> Result<Self, String> {
        let abs_target = if target.is_absolute() {
            target.to_path_buf()
        } else {
            cwd.join(target)
        };

        let cwd_str = cwd
            .to_str()
            .ok_or_else(|| "current directory path is not valid UTF-8".to_string())?;
        let symlink_name = encode_path(cwd_str);
        let symlink_path = symlink_dir.join(symlink_name);

        let action = match probe(&symlink_path) {
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

        Ok(SymLink {
            abs_target,
            symlink_path,
            action,
        })
    }

    /// Plan a symlink operation from pre-probed metadata (for testing).
    #[cfg(test)]
    pub fn with_meta(
        cwd: &Path,
        target: &Path,
        symlink_dir: &Path,
        symlink_meta: Option<Option<PathBuf>>,
    ) -> Result<Self, String> {
        let abs_target = if target.is_absolute() {
            target.to_path_buf()
        } else {
            cwd.join(target)
        };

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

        Ok(SymLink {
            abs_target,
            symlink_path,
            action,
        })
    }

    /// Execute the symlink operation: create the target dir if needed, create/replace the symlink.
    pub fn create(&self, create_target: bool) -> Result<(), String> {
        debug!(
            "symlink {} -> {}",
            self.symlink_path.display(),
            self.abs_target.display()
        );

        if create_target && !self.abs_target.exists() {
            std::fs::create_dir_all(&self.abs_target).map_err(|e| {
                format!("cannot create target '{}': {e}", self.abs_target.display())
            })?;
        }

        if let Some(parent) = self.symlink_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "cannot create symlink directory '{}': {e}",
                    parent.display()
                )
            })?;
        }

        match &self.action {
            SymlinkAction::Create => {}
            SymlinkAction::Replace { .. } => {
                std::fs::remove_file(&self.symlink_path).map_err(|e| {
                    format!(
                        "cannot remove existing symlink '{}': {e}",
                        self.symlink_path.display()
                    )
                })?;
            }
            SymlinkAction::AlreadyCorrect => return Ok(()),
        }

        #[cfg(unix)]
        std::os::unix::fs::symlink(&self.abs_target, &self.symlink_path).map_err(|e| {
            format!(
                "cannot create symlink '{}' -> '{}': {e}",
                self.symlink_path.display(),
                self.abs_target.display()
            )
        })?;

        #[cfg(not(unix))]
        return Err("symlink creation is only supported on Unix".to_string());

        Ok(())
    }
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
}

pub fn symlink(args: &SymlinkArgs) -> Result<(), Box<dyn std::error::Error>> {
    debug!("symlink: enter");
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

    let sl = SymLink::new(&cwd, &target, &symlink_dir)?;

    // Handle interactive prompt for replacement
    if let SymlinkAction::Replace { ref current_target } = sl.action {
        info!(
            "Existing symlink: {} -> {}",
            sl.symlink_path.display(),
            current_target.display()
        );
        if !args.yes {
            let response = crate::common::prompt("Replace with new target? [y/N] ")?;
            if !response.eq_ignore_ascii_case("y") {
                return Err("aborted".into());
            }
        }
    }

    if sl.action == SymlinkAction::AlreadyCorrect {
        info!(
            "Already correct: {} -> {}",
            sl.symlink_path.display(),
            sl.abs_target.display()
        );
    } else {
        sl.create(true)?;
        info!(
            "Created: {} -> {}",
            sl.symlink_path.display(),
            sl.abs_target.display()
        );
    }

    if args.list {
        let abs_target = if target.is_absolute() {
            target
        } else {
            cwd.join(&target)
        };
        info!("");
        info!("Contents of {}:", abs_target.display());
        for entry in std::fs::read_dir(&abs_target)? {
            let entry = entry?;
            info!("  {}", entry.file_name().to_string_lossy());
        }
    }

    debug!("symlink: exit");
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
    fn new_create_when_nothing_exists() {
        let cwd = Path::new("/home/user/project");
        let target = Path::new(".claude");
        let symlink_dir = Path::new("/home/user/.claude/projects");

        let sl = SymLink::with_meta(cwd, target, symlink_dir, None).unwrap();

        assert_eq!(sl.abs_target, PathBuf::from("/home/user/project/.claude"));
        assert_eq!(
            sl.symlink_path,
            PathBuf::from("/home/user/.claude/projects/-home-user-project")
        );
        assert_eq!(sl.action, SymlinkAction::Create);
    }

    #[test]
    fn new_already_correct() {
        let cwd = Path::new("/home/user/project");
        let target = Path::new(".claude");
        let symlink_dir = Path::new("/home/user/.claude/projects");
        let current = PathBuf::from("/home/user/project/.claude");

        let sl = SymLink::with_meta(cwd, target, symlink_dir, Some(Some(current))).unwrap();
        assert_eq!(sl.action, SymlinkAction::AlreadyCorrect);
    }

    #[test]
    fn new_replace_different_target() {
        let cwd = Path::new("/home/user/project");
        let target = Path::new(".claude");
        let symlink_dir = Path::new("/home/user/.claude/projects");
        let current = PathBuf::from("/home/user/other/.claude");

        let sl = SymLink::with_meta(cwd, target, symlink_dir, Some(Some(current.clone()))).unwrap();
        assert_eq!(
            sl.action,
            SymlinkAction::Replace {
                current_target: current
            }
        );
    }

    #[test]
    fn new_error_not_a_symlink() {
        let cwd = Path::new("/home/user/project");
        let target = Path::new(".claude");
        let symlink_dir = Path::new("/home/user/.claude/projects");

        let err = SymLink::with_meta(cwd, target, symlink_dir, Some(None)).unwrap_err();
        assert!(err.contains("not a symlink"));
    }

    #[test]
    fn new_absolute_target() {
        let cwd = Path::new("/home/user/project");
        let target = Path::new("/tmp/my-claude");
        let symlink_dir = Path::new("/home/user/.claude/projects");

        let sl = SymLink::with_meta(cwd, target, symlink_dir, None).unwrap();
        assert_eq!(sl.abs_target, PathBuf::from("/tmp/my-claude"));
    }

    #[test]
    fn probe_nothing() {
        let path = Path::new("/nonexistent/path/that/does/not/exist");
        assert!(probe(path).is_none());
    }

    #[test]
    fn probe_regular_file() {
        let dir = std::env::temp_dir().join("symlink_test_probe_file");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("regular");
        std::fs::write(&file, "hello").unwrap();

        let result = probe(&file);
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

        let result = probe(&link);
        assert_eq!(result, Some(Some(target.clone())));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn create_symlink() {
        let dir = std::env::temp_dir().join("symlink_test_create");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("my_target");
        let symlink_path = dir.join("my_link");

        let sl = SymLink {
            abs_target: target.clone(),
            symlink_path: symlink_path.clone(),
            action: SymlinkAction::Create,
        };

        sl.create(true).unwrap();

        assert!(target.exists());
        assert!(symlink_path.symlink_metadata().unwrap().is_symlink());
        assert_eq!(std::fs::read_link(&symlink_path).unwrap(), target);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn replace_symlink() {
        let dir = std::env::temp_dir().join("symlink_test_replace");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let old_target = dir.join("old_target");
        std::fs::create_dir_all(&old_target).unwrap();
        let new_target = dir.join("new_target");
        std::fs::create_dir_all(&new_target).unwrap();
        let symlink_path = dir.join("the_link");
        std::os::unix::fs::symlink(&old_target, &symlink_path).unwrap();

        let sl = SymLink {
            abs_target: new_target.clone(),
            symlink_path: symlink_path.clone(),
            action: SymlinkAction::Replace {
                current_target: old_target,
            },
        };

        sl.create(false).unwrap();

        assert_eq!(std::fs::read_link(&symlink_path).unwrap(), new_target);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn already_correct_is_noop() {
        let dir = std::env::temp_dir().join("symlink_test_noop");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("target");
        std::fs::create_dir_all(&target).unwrap();

        let sl = SymLink {
            abs_target: target,
            symlink_path: dir.join("nonexistent_link"),
            action: SymlinkAction::AlreadyCorrect,
        };

        sl.create(false).unwrap();
        assert!(!dir.join("nonexistent_link").exists());

        std::fs::remove_dir_all(&dir).ok();
    }
}

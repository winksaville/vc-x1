//! Repo lifecycle helpers — creation, finalization, cross-references.
//!
//! Sibling of `url` (URL/target parsing); this module hosts
//! the local-state mechanics that don't depend on URL form. As of
//! 0.41.1-6.3 it carries `create_local_repo` (steps 1-5 of the
//! init lifecycle) and the `OchidStrategy` policy enum;
//! `cross_ref_ochids` is scheduled to land in -6.4.

use std::path::Path;

use log::info;

use crate::common::{mkdir_p, run, write_file};
use crate::init::{copy_template_recursive, jj_chid, rewrite_readme_first_line};

/// Initial-commit ochid policy used by `create_local_repo`.
///
/// - `None` — POR: plain `Initial commit` message.
/// - `Placeholder` — Dual: `Initial commit\n\nochid: /none`,
///   rewritten via cross-ref `jj describe` once both sides have
///   committed and their chids are known.
#[derive(Clone, Copy, Debug)]
pub enum OchidStrategy {
    None,
    Placeholder,
}

/// Create a new local repo at `target`. Returns the chid of the
/// new initial commit (`jj @-`).
///
/// Performs:
/// - Creates `target` (and its parent if needed).
/// - `git init` + `jj git init --colocate`.
/// - Optionally writes `.vc-config.toml` and/or `.gitignore`.
/// - Optionally copies a template tree, rewriting any
///   `README.md`'s first line to `# <name>`.
/// - Commits `Initial commit` (with an `ochid: /none` trailer
///   when `ochid_strategy` is `Placeholder`).
///
/// Parameters:
/// - `target` — destination directory for the new repo. Created
///   (along with its parent if needed); must not already exist
///   as a populated repo.
/// - `info_label` — narration tag (`"code"`, `"bot"`, `"scratch"`,
///   etc.); appears in `info!()` lines.
/// - `config` — optional `.vc-config.toml` contents. `Some(s)`
///   writes the file; `None` skips. Useful for upgrade paths
///   that leave config in place, or for scratch repos that want
///   a bare git+jj init without project-specific files.
/// - `gitignore` — optional `.gitignore` contents; same
///   `Some`/`None` semantics as `config`.
/// - `template` — optional source dir. When present, copied
///   recursively (non-hidden only) and any `README.md`'s first
///   line is rewritten to `# <name>`.
/// - `name` — repo name used by the README rewrite.
/// - `ochid_strategy` — initial-commit message policy.
#[allow(clippy::too_many_arguments)]
pub fn create_local_repo(
    target: &Path,
    info_label: &str,
    config: Option<&str>,
    gitignore: Option<&str>,
    template: Option<&Path>,
    name: &str,
    ochid_strategy: OchidStrategy,
) -> Result<String, Box<dyn std::error::Error>> {
    info!(
        "Creating local repo {info_label} directory at {}",
        target.display()
    );

    if let Some(parent) = target.parent() {
        mkdir_p(parent)?;
    }
    mkdir_p(target)?;

    info!("Initializing {info_label} repo (git + jj)...");
    run("git", &["init"], target)?;
    info!("colocate jj atop the {info_label} git repo");
    run("jj", &["git", "init", "--colocate"], target)?;

    if config.is_some() || gitignore.is_some() {
        info!("Writing {info_label} config files...");
        if let Some(c) = config {
            write_file(&target.join(".vc-config.toml"), c)?;
        }
        if let Some(g) = gitignore {
            write_file(&target.join(".gitignore"), g)?;
        }
    }

    if let Some(t) = template {
        info!(
            "Copying {info_label} template: {} -> {}",
            t.display(),
            target.display()
        );
        copy_template_recursive(t, target)?;
        rewrite_readme_first_line(target, name)?;
    }

    let msg = match ochid_strategy {
        OchidStrategy::None => "Initial commit",
        OchidStrategy::Placeholder => "Initial commit\n\nochid: /none",
    };
    info!("Committing {info_label}...");
    run("jj", &["commit", "-m", msg], target)?;

    let chid = jj_chid("@-", target)?;

    info!(
        "Created local repo {info_label} directory at {} chid = {chid}",
        target.display()
    );
    Ok(chid)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `OchidStrategy::None` produces a plain `Initial commit`
    /// message (POR-shape).
    #[test]
    fn strategy_none_writes_plain_commit() {
        let base = crate::test_helpers::unique_base("create-repo-strategy-none");
        let target = base.join("work");
        std::fs::create_dir_all(&base).expect("mkdir base");

        let chid = create_local_repo(
            &target,
            "code",
            Some(crate::init::VC_CONFIG_APP_ONLY),
            Some(crate::init::GITIGNORE_APP_ONLY),
            None,
            "scratch",
            OchidStrategy::None,
        )
        .expect("create_local_repo with strategy None");

        assert!(!chid.is_empty(), "chid returned");
        assert!(target.join(".jj").exists(), "jj initialized");
        assert!(target.join(".git").exists(), "git initialized");

        let cfg = std::fs::read_to_string(target.join(".vc-config.toml")).expect("read config");
        assert!(cfg.contains("path = \"/\""));
        assert!(!cfg.contains("other-repo"));

        let log = run("git", &["log", "-1", "--format=%B"], &target).expect("git log");
        assert_eq!(
            log.trim(),
            "Initial commit",
            "OchidStrategy::None uses a plain message"
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    /// `OchidStrategy::Placeholder` writes the `ochid: /none`
    /// placeholder in the initial commit message — what dual mode
    /// lands before the cross-ref rewrite in step 6.
    #[test]
    fn strategy_placeholder_writes_ochid_none() {
        let base = crate::test_helpers::unique_base("create-repo-strategy-placeholder");
        let target = base.join("work");
        std::fs::create_dir_all(&base).expect("mkdir base");

        let _chid = create_local_repo(
            &target,
            "code",
            Some(crate::init::VC_CONFIG_CODE),
            Some(crate::init::GITIGNORE_CODE),
            None,
            "scratch",
            OchidStrategy::Placeholder,
        )
        .expect("create_local_repo with strategy Placeholder");

        let cfg = std::fs::read_to_string(target.join(".vc-config.toml")).expect("read config");
        assert!(cfg.contains("other-repo = \".claude\""));

        let log = run("git", &["log", "-1", "--format=%B"], &target).expect("git log");
        assert!(log.contains("Initial commit"));
        assert!(
            log.contains("ochid: /none"),
            "OchidStrategy::Placeholder leaves a placeholder ochid trailer"
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    /// Both `config` and `gitignore` set to `None` skips the file
    /// writes entirely — no `.vc-config.toml`, no `.gitignore` in
    /// the resulting tree (only the `.jj/` and `.git/` from init).
    #[test]
    fn no_config_or_gitignore_writes_neither_file() {
        let base = crate::test_helpers::unique_base("create-repo-no-files");
        let target = base.join("work");
        std::fs::create_dir_all(&base).expect("mkdir base");

        let _chid = create_local_repo(
            &target,
            "scratch",
            None,
            None,
            None,
            "scratch",
            OchidStrategy::None,
        )
        .expect("create_local_repo with no config files");

        assert!(target.join(".jj").exists(), "jj still initialized");
        assert!(target.join(".git").exists(), "git still initialized");
        assert!(
            !target.join(".vc-config.toml").exists(),
            "no .vc-config.toml when config=None"
        );
        assert!(
            !target.join(".gitignore").exists(),
            "no .gitignore when gitignore=None"
        );

        let _ = std::fs::remove_dir_all(&base);
    }
}

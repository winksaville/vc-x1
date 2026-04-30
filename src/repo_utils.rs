//! Repo lifecycle helpers — creation, finalization, cross-references.
//!
//! Sibling of `url` (URL/target parsing); this module hosts the
//! local-state mechanics that don't depend on URL form. As of
//! 0.41.1-6.5:
//!
//! - `prepare_local_repo` — mkdir, git init, jj init, optional
//!   template copy. Leaves the working copy uncommitted.
//! - `commit_initial` — jj commit of the prepared tree; returns
//!   its chid.
//! - `cross_ref_ochids` — rewrite both initial commits' placeholder
//!   `ochid: /none` trailers once each side's chid is known.
//! - `OchidStrategy` — initial-commit message policy.
//!
//! Splitting prepare from commit lets callers write role-specific
//! files (`.vc-config.toml`, `.gitignore`) into the prepared tree
//! before the initial commit captures them.

use std::path::Path;

use log::{debug, info};

use crate::common::{mkdir_p, run};
use crate::init::{copy_template_recursive, jj_chid, rewrite_readme_first_line};

/// Initial-commit ochid policy used by `commit_initial`.
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

/// Prepare a fresh local repo at `target`: directory + git/jj
/// init + optional template copy. The working copy is left
/// uncommitted so callers can drop role-specific files (e.g.
/// `.vc-config.toml`, `.gitignore`) into the tree before
/// `commit_initial` captures them in the initial commit.
///
/// Performs:
/// - Creates `target` (and its parent if needed).
/// - `git init` + `jj git init --colocate`.
/// - Optionally copies a template tree, rewriting any
///   `README.md`'s first line to `# <name>`.
///
/// Parameters:
/// - `target` — destination directory for the new repo. Created
///   (along with its parent if needed); must not already exist
///   as a populated repo.
/// - `info_label` — narration tag (`"code"`, `"bot"`, `"scratch"`,
///   etc.); appears in `info!()` lines.
/// - `template` — optional source dir. When present, copied
///   recursively (non-hidden only) and any `README.md`'s first
///   line is rewritten to `# <name>`.
/// - `name` — repo name used by the README rewrite.
pub fn prepare_local_repo(
    target: &Path,
    info_label: &str,
    template: Option<&Path>,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Preparing local repo {info_label} directory at {}",
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

    if let Some(t) = template {
        info!(
            "Copying {info_label} template: {} -> {}",
            t.display(),
            target.display()
        );
        copy_template_recursive(t, target)?;
        rewrite_readme_first_line(target, name)?;
    }

    Ok(())
}

/// Commit the prepared working copy as an initial commit and
/// return its chid (`jj @-`).
///
/// Pairs with `prepare_local_repo`: caller calls `prepare_local_repo`,
/// optionally writes role-specific files (e.g. `.vc-config.toml`,
/// `.gitignore`) into the tree, then calls this to capture the
/// whole snapshot in the initial commit.
///
/// Parameters:
/// - `target` — repo working dir, already prepared.
/// - `info_label` — narration tag, mirroring `prepare_local_repo`.
/// - `ochid_strategy` — message policy: `None` writes a plain
///   `Initial commit`; `Placeholder` writes
///   `Initial commit\n\nochid: /none` for later rewrite by
///   `cross_ref_ochids`.
pub fn commit_initial(
    target: &Path,
    info_label: &str,
    ochid_strategy: OchidStrategy,
) -> Result<String, Box<dyn std::error::Error>> {
    let msg = match ochid_strategy {
        OchidStrategy::None => "Initial commit",
        OchidStrategy::Placeholder => "Initial commit\n\nochid: /none",
    };
    info!("Committing {info_label}...");
    run("jj", &["commit", "-m", msg], target)?;

    let chid = jj_chid("@-", target)?;
    info!(
        "Committed {info_label} initial at {} chid = {chid}",
        target.display()
    );
    Ok(chid)
}

/// Rewrite the placeholder `ochid: /none` trailers on both sides
/// of a dual-repo workspace so each initial commit points at its
/// counterpart's chid (init step 6).
///
/// Both sides must already have an initial commit shaped by
/// `OchidStrategy::Placeholder`. After the rewrite, each commit's
/// ochid trailer is a workspace-root-relative path: the code side
/// points at `/.claude/<session_chid>`; the session side points at
/// `/<code_chid>`.
///
/// Parameters:
/// - `code_dir` — code repo on disk; receives `/.claude/<chid>`.
/// - `code_chid` — code-side initial-commit chid; embedded into
///   the session-side trailer.
/// - `session_dir` — session repo on disk; receives `/<chid>`.
/// - `session_chid` — session-side initial-commit chid; embedded
///   into the code-side trailer.
pub fn cross_ref_ochids(
    code_dir: &Path,
    code_chid: &str,
    session_dir: &Path,
    session_chid: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting ochid cross-references...");
    let code_desc = format!("Initial commit\n\nochid: /.claude/{session_chid}");
    let session_desc = format!("Initial commit\n\nochid: /{code_chid}");

    debug!("code side: rewrite initial commit's ochid to point at session chid");
    run("jj", &["describe", "@-", "-m", &code_desc], code_dir)?;
    debug!("session side: rewrite initial commit's ochid to point at code chid");
    run("jj", &["describe", "@-", "-m", &session_desc], session_dir)?;

    debug!("surface post-describe git hashes for the debug log");
    let hash = run("git", &["rev-parse", "HEAD"], code_dir)?;
    debug!("code repo: chid={code_chid} hash={hash}");
    let hash = run("git", &["rev-parse", "HEAD"], session_dir)?;
    debug!(".claude:   chid={session_chid} hash={hash}");
    Ok(())
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

        prepare_local_repo(&target, "code", None, "scratch").expect("prepare_local_repo");
        let chid = commit_initial(&target, "code", OchidStrategy::None).expect("commit_initial");

        assert!(!chid.is_empty(), "chid returned");
        assert!(target.join(".jj").exists(), "jj initialized");
        assert!(target.join(".git").exists(), "git initialized");

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

        prepare_local_repo(&target, "code", None, "scratch").expect("prepare_local_repo");
        let _chid =
            commit_initial(&target, "code", OchidStrategy::Placeholder).expect("commit_initial");

        let log = run("git", &["log", "-1", "--format=%B"], &target).expect("git log");
        assert!(log.contains("Initial commit"));
        assert!(
            log.contains("ochid: /none"),
            "OchidStrategy::Placeholder leaves a placeholder ochid trailer"
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    /// Neither `prepare_local_repo` nor `commit_initial` writes
    /// `.vc-config.toml` or `.gitignore` — those are role-specific
    /// and the caller drops them between prepare and commit (POR
    /// branch of `init_with_symlink` / `create_dual` in `init.rs`).
    /// Verifies the tree contains only `.jj/` and `.git/`.
    #[test]
    fn lifecycle_does_not_write_vc_config_or_gitignore() {
        let base = crate::test_helpers::unique_base("create-repo-no-files");
        let target = base.join("work");
        std::fs::create_dir_all(&base).expect("mkdir base");

        prepare_local_repo(&target, "scratch", None, "scratch").expect("prepare_local_repo");
        let _chid =
            commit_initial(&target, "scratch", OchidStrategy::None).expect("commit_initial");

        assert!(target.join(".jj").exists(), "jj still initialized");
        assert!(target.join(".git").exists(), "git still initialized");
        assert!(
            !target.join(".vc-config.toml").exists(),
            "lifecycle does not write .vc-config.toml"
        );
        assert!(
            !target.join(".gitignore").exists(),
            "lifecycle does not write .gitignore"
        );

        let _ = std::fs::remove_dir_all(&base);
    }
}

//! Repo lifecycle helpers: creation, finalization, cross-references.
//!
//! Sibling of `url` (URL/target parsing): this module hosts the
//! local-state mechanics that don't depend on URL form. As of
//! 0.41.1-6.5:
//!
//! - `prepare_local_repo`: mkdir and colocated jj init. Leaves the
//!   working copy uncommitted.
//! - `commit_initial`: jj commit of the prepared tree, returning
//!   its chid.
//! - `cross_ref_ochids`: rewrite both initial commits' placeholder
//!   `ochid: /none` trailers once each side's chid is known.
//! - `OchidStrategy`: initial-commit message policy.
//!
//! Splitting prepare from commit lets callers write role-specific
//! files (`.vc-config.toml`, `.gitignore`) into the prepared tree
//! before the initial commit captures them.

use std::path::Path;

use log::{debug, info};

use crate::common::mkdir_p;
use crate::desc_helpers::{Trailers, commit_url, with_trailer, with_trailers};
use crate::jj;

/// The title of a new repo's first commit.
pub const INITIAL_TITLE: &str = "Initial commit";

/// Initial-commit ochid policy used by `commit_initial`.
///
/// - `None` (POR): plain `Initial commit` message.
/// - `Placeholder` (Dual): `Initial commit\n\nochid: /none`,
///   rewritten via cross-ref `jj describe` once both sides have
///   committed and their chids are known.
#[derive(Clone, Copy, Debug)]
pub enum OchidStrategy {
    None,
    Placeholder,
}

/// Prepare a fresh local repo at `target`: directory + git/jj
/// init. The working copy is left
/// uncommitted so callers can drop role-specific files (e.g.
/// `.vc-config.toml`, `.gitignore`) into the tree before
/// `commit_initial` captures them in the initial commit.
///
/// Performs:
/// - Creates `target` (and its parent if needed).
/// - `jj git init --colocate` (which creates the git repo too).
///
/// Parameters:
/// - `target`: destination directory for the new repo. Created
///   (along with its parent if needed), and must not already exist
///   as a populated repo.
/// - `info_label`: narration tag (`"work"`, `"agent"`, `"scratch"`,
///   etc.), appearing in `info!()` lines.
pub fn prepare_local_repo(
    target: &Path,
    info_label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!(
        "prepare local repo {info_label} directory at {}",
        target.display()
    );

    if let Some(parent) = target.parent() {
        mkdir_p(parent)?;
    }
    mkdir_p(target)?;

    // The colocated init creates the git repo itself: the former
    // explicit `git init` was redundant (0.76.0-5).
    debug!("init {info_label} repo (jj, colocated)");
    jj::git_init_colocated(target)?;

    Ok(())
}

/// Commit the prepared working copy as an initial commit and
/// return its chid (`jj @-`).
///
/// Every file not ignored goes in, whatever its size: an adopted
/// directory's content is what the commit is for, and a `.gitignore`
/// is how to keep a file out. Files over jj's new-file size limit are
/// named, since a later snapshot would not have taken them.
///
/// Pairs with `prepare_local_repo`: caller calls `prepare_local_repo`,
/// optionally writes role-specific files (e.g. `.vc-config.toml`,
/// `.gitignore`) into the tree, then calls this to capture the
/// whole snapshot in the initial commit.
///
/// Parameters:
/// - `target`: repo working dir, already prepared.
/// - `info_label`: narration tag, mirroring `prepare_local_repo`.
/// - `title`: the commit's title, [`INITIAL_TITLE`] for a new
///   repo's first commit.
/// - `ochid_strategy`: message policy: `None` writes the bare title,
///   while `Placeholder` adds an `ochid: /none` trailer for later
///   rewrite by `cross_ref_ochids`.
pub fn commit_initial(
    target: &Path,
    info_label: &str,
    title: &str,
    ochid_strategy: OchidStrategy,
) -> Result<String, Box<dyn std::error::Error>> {
    let msg = match ochid_strategy {
        OchidStrategy::None => title.to_string(),
        OchidStrategy::Placeholder => format!("{title}\n\nochid: /none"),
    };
    debug!("commit {info_label}");
    let large = jj::commit_any_size(target, &msg)?;
    if !large.is_empty() {
        info!(
            "  tracked {} file(s) over jj's new-file size limit:",
            large.len()
        );
        for (path, size) in &large {
            info!("    {path} ({size} bytes)");
        }
    }

    let chid = jj::chid_of(target, "@-")?;
    info!("  {info_label} commit: chid {chid}");
    Ok(chid)
}

/// One side of a dual-repo workspace as [`cross_ref_ochids`] takes it.
pub struct CrossRefSide<'a> {
    /// The repo on disk.
    pub dir: &'a Path,
    /// Its remote, passed in since it is not yet the repo's `origin`,
    /// `None` for no links.
    pub url: Option<&'a str>,
    /// Its initial commit's chid, embedded into the other's trailer.
    pub chid: &'a str,
}

/// Rewrite the placeholder `ochid: /none` trailers on both sides
/// of a dual-repo workspace so each initial commit points at its
/// counterpart's chid (init step 6).
///
/// Both sides must already have an initial commit shaped by
/// `OchidStrategy::Placeholder`. After the rewrite, each commit's
/// ochid trailer is the other's change id, and the work side also links
/// by URL: `ocid:` the bot commit,
/// rewritten first so its SHA is final, and `pcid:` each parent.
///
/// Parameters:
/// - `work`: the work side, receiving the bot side's value.
/// - `work_title`: the work commit's title, kept as it is: an adopted
///   repo's commit is not its first.
/// - `bot`: the bot side, receiving the work side's value.
pub fn cross_ref_ochids(
    work: CrossRefSide,
    work_title: &str,
    bot: CrossRefSide,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("set ochid cross-references");
    let (work_dir, work_chid) = (work.dir, work.chid);
    let (bot_dir, bot_chid) = (bot.dir, bot.chid);

    debug!("bot side: rewrite initial commit's ochid to point at work chid");
    jj::describe(bot_dir, "@-", &with_trailer(INITIAL_TITLE, work_chid))?;
    let cwd = std::env::current_dir()?;
    let canonical = |url: Option<&str>| url.and_then(|url| crate::url::link_repo_url(url, &cwd));
    let trailers = Trailers {
        ochids: vec![bot_chid.to_string()],
        ocid: match canonical(bot.url) {
            Some(url) => Some(commit_url(&url, &jj::cid_of(bot_dir, "@-")?)),
            None => None,
        },
        pcids: match canonical(work.url) {
            Some(url) => {
                let root = jj::cid_of(work_dir, "root()")?;
                jj::parent_cids_of(work_dir, "@-")?
                    .iter()
                    .filter(|parent| **parent != root)
                    .map(|parent| commit_url(&url, parent))
                    .collect()
            }
            None => Vec::new(),
        },
    };
    debug!("work side: rewrite initial commit's trailers to point at bot commit");
    jj::describe(work_dir, "@-", &with_trailers(work_title, &trailers))?;

    debug!("surface post-describe commit ids for the debug log");
    // Colocated, so the jj commit id is the git hash.
    let hash = jj::cid_of(work_dir, "@-")?;
    debug!("work repo: chid={work_chid} hash={hash}");
    let hash = jj::cid_of(bot_dir, "@-")?;
    debug!(".claude:   chid={bot_chid} hash={hash}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::git_ok;

    /// Each side holds the other's change id, and with hosted remotes the
    /// work side links the bot commit's final SHA, an initial commit having
    /// no parent to link.
    #[test]
    fn cross_ref_stamps_change_ids_and_links_the_work_side() {
        let base = crate::test_helpers::unique_base("create-repo-cross-ref");
        let (work, bot) = (base.join("work"), base.join("bot"));
        std::fs::create_dir_all(&base).expect("mkdir base");
        prepare_local_repo(&work, "work").expect("prepare work");
        prepare_local_repo(&bot, "bot").expect("prepare bot");
        let wc = commit_initial(&work, "work", INITIAL_TITLE, OchidStrategy::Placeholder)
            .expect("commit work");
        let bc = commit_initial(&bot, "bot", INITIAL_TITLE, OchidStrategy::Placeholder)
            .expect("commit bot");

        let cross_ref = |work_url: Option<&str>, bot_url: Option<&str>| {
            cross_ref_ochids(
                CrossRefSide {
                    dir: &work,
                    url: work_url,
                    chid: &wc,
                },
                INITIAL_TITLE,
                CrossRefSide {
                    dir: &bot,
                    url: bot_url,
                    chid: &bc,
                },
            )
            .expect("cross_ref_ochids");
        };
        let desc = |repo: &Path| jj::desc_of(repo, "@-").expect("desc");

        cross_ref(
            Some("git@github.com:o/r.git"),
            Some("https://github.com/o/r.agent-session.git"),
        );
        let sha = jj::cid_of(&bot, "@-").expect("cid");
        assert!(
            desc(&work).ends_with(&format!(
                "\n\nochid: {bc}\nocid: https://github.com/o/r.agent-session/commit/{sha}"
            )),
            "{}",
            desc(&work)
        );
        assert!(
            desc(&bot).ends_with(&format!("\n\nochid: {wc}")),
            "{}",
            desc(&bot)
        );

        cross_ref(None, None);
        assert!(
            desc(&work).ends_with(&format!("\n\nochid: {bc}")),
            "{}",
            desc(&work)
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    /// `OchidStrategy::None` produces a plain `Initial commit`
    /// message (POR-shape).
    #[test]
    fn strategy_none_writes_plain_commit() {
        let base = crate::test_helpers::unique_base("create-repo-strategy-none");
        let target = base.join("work");
        std::fs::create_dir_all(&base).expect("mkdir base");

        prepare_local_repo(&target, "work").expect("prepare_local_repo");
        let chid = commit_initial(&target, "work", INITIAL_TITLE, OchidStrategy::None)
            .expect("commit_initial");

        assert!(!chid.is_empty(), "chid returned");
        assert!(target.join(".jj").exists(), "jj initialized");
        assert!(target.join(".git").exists(), "git initialized");

        let log = git_ok(&target, &["log", "-1", "--format=%B"]);
        assert_eq!(
            log.trim(),
            "Initial commit",
            "OchidStrategy::None uses a plain message"
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    /// `OchidStrategy::Placeholder` writes the `ochid: /none`
    /// placeholder in the initial commit message: what dual mode
    /// lands before the cross-ref rewrite in step 6.
    #[test]
    fn strategy_placeholder_writes_ochid_none() {
        let base = crate::test_helpers::unique_base("create-repo-strategy-placeholder");
        let target = base.join("work");
        std::fs::create_dir_all(&base).expect("mkdir base");

        prepare_local_repo(&target, "work").expect("prepare_local_repo");
        let _chid = commit_initial(&target, "work", INITIAL_TITLE, OchidStrategy::Placeholder)
            .expect("commit_initial");

        let log = git_ok(&target, &["log", "-1", "--format=%B"]);
        assert!(log.contains("Initial commit"));
        assert!(
            log.contains("ochid: /none"),
            "OchidStrategy::Placeholder leaves a placeholder ochid trailer"
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    /// Neither `prepare_local_repo` nor `commit_initial` writes
    /// `.vc-config.toml` or `.gitignore`: those are role-specific
    /// and the caller drops them between prepare and commit.
    /// Verifies the tree contains only `.jj/` and `.git/`.
    #[test]
    fn lifecycle_does_not_write_vc_config_or_gitignore() {
        let base = crate::test_helpers::unique_base("create-repo-no-files");
        let target = base.join("work");
        std::fs::create_dir_all(&base).expect("mkdir base");

        prepare_local_repo(&target, "scratch").expect("prepare_local_repo");
        let _chid = commit_initial(&target, "scratch", INITIAL_TITLE, OchidStrategy::None)
            .expect("commit_initial");

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

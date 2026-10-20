//! The pair: a dual workspace's two repos as the commands that act
//! on both see them, and the two-repo moves those commands share.
//!
//! `commit` and `push` resolve the same two paths from `-s` and
//! `-R`, whatever side the scope names, since a half still needs the
//! other repo to find or name its partner, and both take the same
//! `jj op` snapshots before their mutation window and restore both
//! repos from them on a failure inside it. `squash` will too. The
//! description helpers they share are in `desc_helpers`.

use std::path::{Path, PathBuf};

use log::{info, warn};

use crate::jj::{current_op_id, op_restore};
use crate::options_flags::scope::{RepoScope, Scope, Side};

/// A dual workspace's two repos, by absolute path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pair {
    pub work: PathBuf,
    pub agent: PathBuf,
}

/// The pair `-R`, or the cwd, is in, by its root or either repo,
/// resolved as `-s b` whatever side the caller's scope names.
pub fn resolve_pair(repo: Option<&Path>) -> Result<Pair, Box<dyn std::error::Error>> {
    let both = RepoScope::Sides(Scope(vec![Side::Work, Side::Bot]));
    let repos = crate::common::resolve_scoped_repos(&both, repo)?;
    match repos.as_slice() {
        [work, agent] => Ok(Pair {
            work: work.clone(),
            agent: agent.clone(),
        }),
        other => Err(format!("expected the pair's two repos, got {}", other.len()).into()),
    }
}

/// `jj op` ids of both repos, captured immediately before a
/// command's first mutation, to restore both from if a stage fails
/// before anything reaches a remote.
#[derive(Debug)]
pub struct Snapshots {
    pub work: String,
    pub agent: String,
}

impl Snapshots {
    /// Capture both repos' current op ids.
    pub fn take(pair: &Pair) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            work: current_op_id(&pair.work)?,
            agent: current_op_id(&pair.agent)?,
        })
    }

    /// Restore both repos to the snapshots, agent first since it is
    /// the side mutated last. Best-effort: a failed restore is
    /// warned about and never shadows the original error the caller
    /// propagates, which `original` names in the first line.
    pub fn restore(&self, pair: &Pair, who: &str, original: &dyn std::error::Error) {
        warn!("{who}: rolling back both repos after: {original}");
        match op_restore(&pair.agent, &self.agent) {
            Ok(()) => info!("{who}: restored agent repo to op {}", self.agent),
            Err(e) => warn!("{who}: agent repo restore failed: {e}"),
        }
        match op_restore(&pair.work, &self.work) {
            Ok(()) => info!("{who}: restored work repo to op {}", self.work),
            Err(e) => warn!("{who}: work repo restore failed: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{Fixture, cid, jj_ok};
    use std::fs;

    /// The pair resolves from either repo, and `restore` rewinds
    /// both repos' bookmarks to where the snapshots were taken.
    #[test]
    fn resolve_from_either_repo_and_restore_both() {
        let fx = Fixture::new("pair");
        let canon = |p: &PathBuf| p.canonicalize().expect("canonical");
        for start in [&fx.work, &fx.bot] {
            let pair = resolve_pair(Some(start)).expect("pair");
            assert_eq!(pair.work, canon(&fx.work), "from {start:?}");
            assert_eq!(pair.agent, canon(&fx.bot), "from {start:?}");
        }
        let pair = resolve_pair(Some(&fx.work)).expect("pair");

        let snapshots = Snapshots::take(&pair).expect("snapshots");
        let main_work_start = cid(&fx.work, "main");
        let main_bot_start = cid(&fx.bot, "main");
        for (repo, file) in [(&fx.work, "work.txt"), (&fx.bot, "session.jsonl")] {
            fs::write(repo.join(file), "x\n").expect("write");
            jj_ok(repo, &["describe", "-m", "moved"]);
            jj_ok(repo, &["bookmark", "set", "main", "-r", "@"]);
            jj_ok(repo, &["new"]);
        }
        assert_ne!(cid(&fx.work, "main"), main_work_start);
        assert_ne!(cid(&fx.bot, "main"), main_bot_start);

        let err: Box<dyn std::error::Error> = "forced for test".into();
        snapshots.restore(&pair, "test", err.as_ref());
        assert_eq!(cid(&fx.work, "main"), main_work_start);
        assert_eq!(cid(&fx.bot, "main"), main_bot_start);
    }
}

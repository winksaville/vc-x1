//! The pair: a dual workspace's two repos as the commands that act
//! on both see them, and the two-repo moves those commands share.
//!
//! `commit`, `push`, and `squash` resolve the same two paths from
//! `-s` and `-R`, whatever side the scope names, since a half still
//! needs the other repo to find or name its partner, take the same
//! `jj op` snapshots before their mutation window and restore both
//! repos from them on a failure inside it, and read the pair's
//! state the same way: the work commit's trailer, and where its
//! partner stands. The description helpers they share are in
//! `desc_helpers`.

use std::path::{Path, PathBuf};

use log::{info, warn};

use crate::desc_helpers::{extract_bare_id, extract_ochid_from_desc};
use crate::jj;
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

/// Refuse a conflicted repo, naming it by `label` under `who`.
pub fn refuse_conflicts(
    repo: &Path,
    who: &str,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if jj::matches(repo, "conflicts()")? {
        return Err(format!(
            "{who}: the {label} repo '{}' has conflicts: resolve them first",
            repo.display()
        )
        .into());
    }
    Ok(())
}

/// Whether `rev` is published: an ancestor of some remote bookmark,
/// so rewriting it is a remote rewrite.
pub fn is_published(repo: &Path, rev: &str) -> Result<bool, Box<dyn std::error::Error>> {
    jj::matches(repo, &format!("({rev}) & ::remote_bookmarks()"))
}

/// The work commit of a pair: the work `@-`, its change id, and the
/// trailer it carries, the full value and the bare change id it
/// names. `None` when `@-` carries no trailer, so is not a pair's.
#[derive(Debug, Clone)]
pub struct WorkCommit {
    pub chid: String,
    pub trailer: String,
    pub stamped: String,
}

/// Read the work commit of the pair, `None` when it has no trailer.
pub fn work_commit(pair: &Pair) -> Result<Option<WorkCommit>, Box<dyn std::error::Error>> {
    let desc = jj::desc_of(&pair.work, "@-")?;
    let Some(trailer) = extract_ochid_from_desc(&desc) else {
        return Ok(None);
    };
    Ok(Some(WorkCommit {
        chid: jj::chid_of(&pair.work, "@-")?,
        stamped: extract_bare_id(&trailer).to_string(),
        trailer,
    }))
}

/// Where the agent half of a pair stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Partner {
    /// The agent `@-` names the work commit, the pair made at once
    /// (`commit -s b`): the journal in `@` folds into it. Its id.
    Committed(String),
    /// The agent `@` is the change the work commit names (`commit
    /// -s w`): the journal is the partner, for push to commit. Its id.
    Pending(String),
    /// Neither: the agent repo changed since the commit, so the work
    /// commit names a change that is no longer the partner.
    Changed { stamped: String, agent_at: String },
}

/// Read where the agent half stands for `work`. Reading the agent
/// `@` snapshots it, so a file the size limit refuses stops the
/// caller here, before any mutation.
pub fn partner(pair: &Pair, work: &WorkCommit) -> Result<Partner, Box<dyn std::error::Error>> {
    let agent_at = jj::chid_of(&pair.agent, "@")?;
    let names_work = extract_ochid_from_desc(&jj::desc_of(&pair.agent, "@-")?)
        .is_some_and(|o| extract_bare_id(&o) == work.chid);
    Ok(if names_work {
        Partner::Committed(jj::chid_of(&pair.agent, "@-")?)
    } else if agent_at == work.stamped {
        Partner::Pending(agent_at)
    } else {
        Partner::Changed {
            stamped: work.stamped.clone(),
            agent_at,
        }
    })
}

/// One repo's `jj op` id, captured before a half's first mutation,
/// to restore that repo alone if a stage fails before its remote
/// leg. A half restores only the repo it touched, since the other
/// may have moved on, a session journal writing to it.
#[derive(Debug)]
pub struct Snapshot {
    repo: PathBuf,
    op: String,
}

impl Snapshot {
    /// Capture the repo's current op id.
    pub fn take(repo: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            repo: repo.to_path_buf(),
            op: current_op_id(repo)?,
        })
    }

    /// Restore the repo to the snapshot, best-effort, as
    /// [`Snapshots::restore`] does for both.
    pub fn restore(&self, who: &str, original: &dyn std::error::Error) {
        warn!(
            "{who}: rolling back '{}' after: {original}",
            self.repo.display()
        );
        match op_restore(&self.repo, &self.op) {
            Ok(()) => info!(
                "{who}: restored '{}' to op {}",
                self.repo.display(),
                self.op
            ),
            Err(e) => warn!("{who}: restore of '{}' failed: {e}", self.repo.display()),
        }
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

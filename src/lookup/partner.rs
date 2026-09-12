//! Partner resolution: the commit on the other side of a dual
//! workspace that an `ochid:` trailer names, the candidates by time
//! when a commit carries none, and the predecessors a partner was
//! rewritten from.
//!
//! The trailer is load-bearing: it names a change id, which is what
//! survives every rewrite, so it finds the current partner through
//! a re-describe, a rebase, an amend, and a trapezoid reshape. Time
//! is the degraded key, and it gives candidates rather than an
//! answer, all of them named.
//!
//! - `Partner`: linked commits, one per trailer, or the candidates
//!   inside a tolerance of the commit's committer time.
//! - `partners`: the resolution against the other side's repo.
//! - `predecessors` and `pushed_predecessors`: the evolution log
//!   walked back from a commit, whole, or only the versions that
//!   carried its title, which are the ones a push published.

use std::sync::Arc;

use futures::StreamExt;
use jj_lib::commit::Commit;
use jj_lib::evolution::walk_predecessors;
use jj_lib::repo::{ReadonlyRepo, Repo};
use jj_lib::workspace::Workspace;
use pollster::FutureExt;

use crate::common;
use crate::desc_helpers::{extract_bare_id, extract_ochids};

/// The other side's answer for one commit.
#[derive(Debug)]
pub enum Partner {
    /// The commits the trailers name, in trailer order, each the
    /// change id's current commit.
    Linked(Vec<Commit>),
    /// No trailer: the commits whose committer time is within the
    /// tolerance, nearest first, which may be none.
    Candidates {
        /// The tolerance the candidates were gathered with.
        tolerance_secs: i64,
        /// The candidates, nearest in time first.
        commits: Vec<Commit>,
    },
}

/// The commit's committer time in milliseconds since the epoch.
fn committed_at(commit: &Commit) -> i64 {
    commit.committer().timestamp.timestamp.0
}

/// Resolve `commit`'s partners on the other side, `there`.
///
/// - With trailers, each names a change id that must resolve there,
///   and a trailer that resolves to nothing is an error, since a
///   dangling trailer is a broken link and not a missing one.
/// - Without, every commit there within `tolerance_secs` of the
///   commit's committer time is a candidate.
pub fn partners(
    commit: &Commit,
    there: (&Workspace, &Arc<ReadonlyRepo>),
    tolerance_secs: i64,
) -> Result<Partner, Box<dyn std::error::Error>> {
    let (workspace, repo) = there;
    let trailers = extract_ochids(commit.description());
    if trailers.is_empty() {
        return candidates(commit, there, tolerance_secs);
    }
    let mut linked = Vec::new();
    for trailer in &trailers {
        let bare = extract_bare_id(trailer);
        let dangling = || {
            format!(
                "ochid: {trailer} names no commit on the other side, in '{}'",
                workspace.workspace_root().display()
            )
        };
        let ids = match common::resolve_revset(workspace, repo, bare) {
            Ok(ids) if ids.is_empty() => return Err(dangling().into()),
            Ok(ids) => ids,
            Err(e) if crate::jj::is_no_such_revision(e.as_ref()) => return Err(dangling().into()),
            Err(e) => return Err(e),
        };
        for id in &ids {
            linked.push(repo.store().get_commit(id)?);
        }
    }
    Ok(Partner::Linked(linked))
}

/// The commits on the other side within the tolerance of `commit`'s
/// committer time, nearest first, the root excluded.
fn candidates(
    commit: &Commit,
    there: (&Workspace, &Arc<ReadonlyRepo>),
    tolerance_secs: i64,
) -> Result<Partner, Box<dyn std::error::Error>> {
    let (workspace, repo) = there;
    let at = committed_at(commit);
    let root = repo.store().root_commit_id().clone();
    let mut commits = Vec::new();
    for id in common::resolve_revset(workspace, repo, "all()")? {
        if id == root {
            continue;
        }
        let other = repo.store().get_commit(&id)?;
        if (committed_at(&other) - at).abs() <= tolerance_secs * 1000 {
            commits.push(other);
        }
    }
    commits.sort_by_key(|c| (committed_at(c) - at).abs());
    Ok(Partner::Candidates {
        tolerance_secs,
        commits,
    })
}

/// Every commit `commit` was rewritten from, nearest first, from the
/// operation log's predecessor records.
///
/// The working copy's snapshots are in the chain too, since jj
/// records each as a rewrite, so most commits have predecessors and
/// the interesting ones are what `pushed_predecessors` keeps.
#[allow(
    dead_code,
    reason = "the work-to-transcript window reads an amended partner's"
)]
pub fn predecessors(
    repo: &Arc<ReadonlyRepo>,
    commit: &Commit,
) -> Result<Vec<Commit>, Box<dyn std::error::Error>> {
    let mut stream = std::pin::pin!(walk_predecessors(repo, &[commit.id().clone()]));
    let mut out = Vec::new();
    while let Some(entry) = stream.next().block_on() {
        let entry = entry?;
        if entry.commit.id() != commit.id() {
            out.push(entry.commit);
        }
    }
    Ok(out)
}

/// The predecessors that carried `commit`'s title: the versions a
/// push published before the rewrite, which is what an amended
/// partner's push-time window is read from.
#[allow(
    dead_code,
    reason = "the work-to-transcript window reads an amended partner's"
)]
pub fn pushed_predecessors(
    repo: &Arc<ReadonlyRepo>,
    commit: &Commit,
) -> Result<Vec<Commit>, Box<dyn std::error::Error>> {
    let title = commit.description().lines().next().unwrap_or(""); // OK: obvious
    Ok(predecessors(repo, commit)?
        .into_iter()
        .filter(|p| p.description().lines().next() == Some(title))
        .collect())
}

/// A commit's change id in jj's spelling, whole.
pub fn chid(c: &Commit) -> String {
    jj_lib::hex_util::encode_reverse_hex(jj_lib::object_id::ObjectId::as_bytes(c.change_id()))
}

/// A commit's change id as the trailers and the logs spell it, the
/// first twelve characters.
pub fn short_chid(c: &Commit) -> String {
    chid(c)[..12].to_string()
}

/// A commit's title, its description's first line.
pub fn title(c: &Commit) -> &str {
    c.description().lines().next().unwrap_or("") // OK: obvious
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::lookup::tests::dr1;

    /// One commit by a revset naming exactly one.
    pub(crate) fn by_chid(ws: &Workspace, repo: &Arc<ReadonlyRepo>, id: &str) -> Commit {
        let ids = common::resolve_revset(ws, repo, id).unwrap();
        assert_eq!(ids.len(), 1, "{id}");
        repo.store().get_commit(&ids[0]).unwrap()
    }

    /// The trailer finds the current partner in both directions for
    /// every push, the reshaped closing and the amended rung included.
    #[test]
    fn dr1_trailer_finds_every_partner_both_ways() {
        let Some((root, rel)) = dr1() else { return };
        let (ww, wr) = common::load_repo(&root).unwrap();
        let (aw, ar) = common::load_repo(&root.join(".claude")).unwrap();
        for p in rel["pushes"].as_array().unwrap() {
            if p["case"] == "no-trailer" {
                continue;
            }
            let work = by_chid(&ww, &wr, p["work"].as_str().unwrap());
            let agent = by_chid(&aw, &ar, p["agent"].as_str().unwrap());
            let Partner::Linked(found) = partners(&work, (&aw, &ar), 0).unwrap() else {
                panic!("{}: work has a trailer", p["title"]);
            };
            assert_eq!(found.len(), 1, "{}", p["title"]);
            assert_eq!(chid(&found[0]), chid(&agent), "{}", p["title"]);
            let Partner::Linked(found) = partners(&agent, (&ww, &wr), 0).unwrap() else {
                panic!("{}: agent has a trailer", p["title"]);
            };
            assert_eq!(found.len(), 1, "{}", p["title"]);
            assert_eq!(chid(&found[0]), chid(&work), "{}", p["title"]);
        }
    }

    /// The pair made by hand resolves to candidates by time: the
    /// partner alone inside a minute, and nothing inside ten seconds,
    /// since the fixture committed the agent side twenty seconds after
    /// the work side.
    #[test]
    fn dr1_no_trailer_gives_candidates_by_time() {
        let Some((root, rel)) = dr1() else { return };
        let (ww, wr) = common::load_repo(&root).unwrap();
        let (aw, ar) = common::load_repo(&root.join(".claude")).unwrap();
        let p = rel["pushes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["case"] == "no-trailer")
            .unwrap();
        let work = by_chid(&ww, &wr, p["work"].as_str().unwrap());
        let Partner::Candidates { commits, .. } = partners(&work, (&aw, &ar), 60).unwrap() else {
            panic!("no trailer, so candidates");
        };
        let found: Vec<String> = commits.iter().map(chid).collect();
        assert_eq!(found, vec![p["agent"].as_str().unwrap().to_string()]);
        let Partner::Candidates { commits, .. } = partners(&work, (&aw, &ar), 10).unwrap() else {
            panic!("no trailer, so candidates");
        };
        assert!(commits.is_empty());
    }

    /// A trailer that names nothing on the other side is an error
    /// naming the trailer, never an empty answer.
    #[test]
    fn dr1_dangling_trailer_is_an_error() {
        let Some((root, rel)) = dr1() else { return };
        let (ww, wr) = common::load_repo(&root).unwrap();
        let (aw, ar) = common::load_repo(&root.join(".claude")).unwrap();
        let p = &rel["pushes"][0];
        let agent = by_chid(&aw, &ar, p["agent"].as_str().unwrap());
        let err = partners(&agent, (&aw, &ar), 0).unwrap_err().to_string();
        assert!(err.contains("names no commit"), "{err}");
        let work = by_chid(&ww, &wr, p["work"].as_str().unwrap());
        assert!(partners(&work, (&ww, &wr), 0).is_err());
    }

    /// The amended rung and its partner each have a pushed
    /// predecessor, the version the first push published, and a rung
    /// pushed once has none.
    #[test]
    fn dr1_amended_rung_has_a_pushed_predecessor_on_both_sides() {
        let Some((root, rel)) = dr1() else { return };
        let (ww, wr) = common::load_repo(&root).unwrap();
        let (aw, ar) = common::load_repo(&root.join(".claude")).unwrap();
        for p in rel["pushes"].as_array().unwrap() {
            let work = by_chid(&ww, &wr, p["work"].as_str().unwrap());
            let agent = by_chid(&aw, &ar, p["agent"].as_str().unwrap());
            let w_prev = pushed_predecessors(&wr, &work).unwrap();
            let a_prev = pushed_predecessors(&ar, &agent).unwrap();
            if p["predecessor"] == true {
                assert!(!w_prev.is_empty(), "{}: work", p["title"]);
                assert!(!a_prev.is_empty(), "{}: agent", p["title"]);
                assert_ne!(w_prev[0].tree_ids(), work.tree_ids(), "{}", p["title"]);
                assert_eq!(chid(&w_prev[0]), chid(&work));
            } else if p["case"] == "rung" || p["case"] == "opening" {
                assert!(w_prev.is_empty(), "{}: work", p["title"]);
            }
            assert!(!predecessors(&wr, &work).unwrap().is_empty());
        }
    }
}

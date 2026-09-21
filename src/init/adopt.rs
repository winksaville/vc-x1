//! What an init target already holds, read before init touches it.
//!
//! A fresh init creates everything and so wants an absent target.
//! `--adopt` grows what an existing target lacks, and what it lacks
//! depends on which of these states the target is in.

use std::path::Path;

use crate::toml_simple;

/// The state of an init target directory.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TargetState {
    /// Nothing there: the fresh-create path.
    Absent,
    /// A directory that is no repo: both repos are to be grown, its
    /// content the work repo's first commit.
    PlainDir,
    /// A repo with no workspace config: the agent side is to be grown
    /// beside it, its history left alone.
    Por,
    /// A repo whose config declares the work side only: the agent
    /// side is to be grown and the config given its entry.
    SingleRepo,
    /// A repo whose config already declares an agent side: nothing to
    /// grow, so init refuses it.
    Dual,
}

impl TargetState {
    /// How a message names the state.
    pub(crate) fn describe(&self) -> &'static str {
        match self {
            TargetState::Absent => "absent",
            TargetState::PlainDir => "a plain directory with no repo",
            TargetState::Por => "a repo with no workspace config",
            TargetState::SingleRepo => "a single-repo workspace",
            TargetState::Dual => "a dual-repo workspace",
        }
    }
}

/// Read what `dir` holds.
///
/// A repo is a `.git` or a `.jj` in `dir` itself. The workspace
/// config is whichever carrier `dir` holds, and its `repos.agent`
/// decides single-repo against dual, so a dual workspace's agent
/// repo, whose config declares `agent = "."`, reads as dual too.
///
/// Errors, rather than a state, for what no state describes and init
/// must not guess at:
///
/// - `dir` is a file.
/// - a config with no repo beside it, or one whose `[repos]` registry
///   declares no work side, which is a legacy config or a broken one.
pub(crate) fn detect_target_state(dir: &Path) -> Result<TargetState, Box<dyn std::error::Error>> {
    if !dir.exists() {
        return Ok(TargetState::Absent);
    }
    if !dir.is_dir() {
        return Err(format!("'{}' exists and is not a directory", dir.display()).into());
    }
    let is_repo = dir.join(".git").exists() || dir.join(".jj").exists();
    let Some(cfg) = crate::config_md::load(dir)? else {
        return Ok(if is_repo {
            TargetState::Por
        } else {
            TargetState::PlainDir
        });
    };
    if !is_repo {
        return Err(format!(
            "'{}' holds a workspace config but no repo",
            cfg.path.display()
        )
        .into());
    }
    crate::common::reject_legacy_config(dir)?;
    if toml_simple::toml_get(&cfg.map, "repos.work").is_none() {
        return Err(format!(
            "{}: declares no repos.work, so it is no workspace config init can read",
            cfg.path.display()
        )
        .into());
    }
    Ok(
        match toml_simple::toml_get(&cfg.map, "repos.agent").filter(|v| !v.is_empty()) {
            Some(_) => TargetState::Dual,
            None => TargetState::SingleRepo,
        },
    )
}

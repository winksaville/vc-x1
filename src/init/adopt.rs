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

/// Check that a repo adopt is to grow is one it can: colocated with
/// git, since vc-x1 reads both halves, and with a clean working copy,
/// since adopt's commit would otherwise take the uncommitted work in.
///
/// A git-only repo is pointed at `jj git init --colocate`, which
/// colocates it without touching its history, and a jj repo whose git
/// store is internal is refused, having no `.git` for git to use.
pub(crate) fn check_repo(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let (jj, git) = (dir.join(".jj").exists(), dir.join(".git").exists());
    if git && !jj {
        return Err(format!(
            "--adopt: '{}' is a git repo with no jj, which adopt does not take yet: run \
             `jj git init --colocate` in it, then adopt it",
            dir.display()
        )
        .into());
    }
    if jj && !git {
        return Err(format!(
            "--adopt: '{}' is a jj repo not colocated with git: vc-x1 needs a colocated repo",
            dir.display()
        )
        .into());
    }
    let clean = crate::jj::is_empty(dir, "@")? && crate::jj::desc_of(dir, "@")?.trim().is_empty();
    if !clean {
        return Err(format!(
            "--adopt: '{}' has uncommitted work in @: commit it first, since adopt adds a \
             commit of its own on top",
            dir.display()
        )
        .into());
    }
    Ok(())
}

/// Add the agent side to a single-repo workspace's config, in place.
///
/// Only lines are added: `agent = "<agent_dir>"` after the last key of
/// `[repos]`, and `agent-repo = "<agent_repo>"` under `[remote]`, a
/// header added after the `agent` line when the file has none. The
/// prose, the comments, and every other key stay as they are, since
/// the file is the user's. The result is read back, and a file that
/// does not then declare both keys is restored and refused.
pub(crate) fn add_agent_to_config(
    dir: &Path,
    agent_dir: &str,
    agent_repo: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = crate::config_md::vc_config_path(dir)?
        .ok_or_else(|| format!("'{}' holds no workspace config to edit", dir.display()))?;
    let original = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read '{}': {e}", path.display()))?;
    let fenced = path.extension().is_some_and(|e| e == "md");
    let edited = add_agent_keys(&original, fenced, agent_dir, agent_repo)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    crate::common::write_file(&path, &edited)?;

    let map = crate::config_md::load_file(&path)?;
    let reads = |key: &str| toml_simple::toml_get(&map, key).map(String::as_str);
    if reads("repos.agent") != Some(agent_dir) || reads("remote.agent-repo") != Some(agent_repo) {
        crate::common::write_file(&path, &original)?;
        return Err(format!(
            "{}: the edited config did not read back with repos.agent and remote.agent-repo, so \
             it was restored: add them by hand, then adopt",
            path.display()
        )
        .into());
    }
    Ok(())
}

/// The lines-only edit behind [`add_agent_to_config`].
///
/// `fenced` is a markdown carrier, whose TOML is its ```` ```toml ````
/// fences read as one document, so a table runs on across fences.
/// Otherwise the whole text is TOML.
pub(crate) fn add_agent_keys(
    text: &str,
    fenced: bool,
    agent_dir: &str,
    agent_repo: &str,
) -> Result<String, String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut in_toml = vec![!fenced; lines.len()];
    if fenced {
        let mut open = false;
        for (i, line) in lines.iter().enumerate() {
            let t = line.trim();
            if !open && t == "```toml" {
                open = true;
            } else if open && t.starts_with("```") {
                open = false;
            } else {
                in_toml[i] = open;
            }
        }
    }

    let mut table: Option<&str> = None;
    let mut repos_last_key = None;
    let mut remote_header = None;
    for (i, line) in lines.iter().enumerate() {
        if !in_toml[i] {
            continue;
        }
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') && !t.starts_with("[[") {
            let name = t[1..t.len() - 1].trim();
            if name == "remote" {
                remote_header = Some(i);
            }
            table = Some(name);
        } else if !t.is_empty() && !t.starts_with('#') && table == Some("repos") {
            repos_last_key = Some(i);
        }
    }
    let last = repos_last_key.ok_or("no [repos] table with a key under it")?;

    let indent: String = lines[last]
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect();
    let agent_line = format!("{indent}agent = \"{agent_dir}\"");
    let repo_line = format!("agent-repo = \"{agent_repo}\"");
    let mut out: Vec<String> = Vec::with_capacity(lines.len() + 4);
    for (i, line) in lines.iter().enumerate() {
        out.push((*line).to_string());
        if i == last {
            out.push(agent_line.clone());
            if remote_header.is_none() {
                out.push(String::new());
                out.push("[remote]".to_string());
                out.push(repo_line.clone());
            }
        }
        if Some(i) == remote_header {
            out.push(repo_line.clone());
        }
    }
    let mut edited = out.join("\n");
    if text.ends_with('\n') {
        edited.push('\n');
    }
    Ok(edited)
}

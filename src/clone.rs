//! `vc-x1 clone` — clone a repo (URL or local path) into a workspace.
//!
//! - `--scope=code,bot` (default): dual-repo layout — clones code,
//!   derives bot source (`<source>.claude`), clones bot into
//!   `<target>/.claude`, creates the Claude Code symlink. Both
//!   sides must succeed.
//! - `--scope=por`: single repo into `<target>`. No `.claude/`,
//!   no symlink.
//!
//! TARGET shapes (all routed through `parse_target`): URL,
//! `owner/name` shorthand, or a local path (`./X`, `/X`, `~/X`,
//! `.`, `..`). Path-form is symmetric with `git clone /local/bare.git`
//! — useful for fixtures and CI scratch dirs.
//!
//! `clone_one` and `clone_dual` are `pub(crate)` so init's `-3`
//! reshape can reuse them for the "URL exists → clone" preflight
//! path (per `notes/chores-08.md > init + clone redesign`).

use std::path::Path;

use clap::Args;
use log::info;

use crate::common::run;
use crate::repo_url::{Target, derive_name, derive_session_url, parse_target, resolve_url};
use crate::symlink;

/// Topology choice for `vc-x1 clone --scope`.
///
/// - `CodeBot` (default) — clone the dual-repo layout: code into
///   target dir, bot derived source into `target/.claude`, plus
///   the Claude Code symlink. Both sides must succeed.
/// - `Por` — clone a single repo (Plain Old Repo) into target dir;
///   no `.claude/`, no symlink.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloneScope {
    CodeBot,
    Por,
}

/// Parse the `--scope` value for clone.
///
/// - Accepts `code,bot` / `bot,code` (commutative) and `por`.
/// - Standalone `code` or `bot` errors — clone has no
///   config-driven defaults to look up against; the manual
///   decomposition (two `--scope=por` clones + `vc-x1 symlink`)
///   covers the rare cases where a user wants the dual layout
///   composed by hand.
pub fn parse_clone_scope(s: &str) -> Result<CloneScope, String> {
    match s {
        "code,bot" | "bot,code" => Ok(CloneScope::CodeBot),
        "por" => Ok(CloneScope::Por),
        "code" | "bot" => Err(format!(
            "'--scope={s}' is not valid for clone — use 'code,bot' (dual) or 'por' (single)"
        )),
        _ => Err(format!(
            "'--scope={s}' is not recognized — expected 'code,bot' or 'por'"
        )),
    }
}

/// CLI args for `vc-x1 clone`.
#[derive(Args, Debug)]
pub struct CloneArgs {
    /// Source to clone — URL, owner/name shorthand, or local path.
    ///
    /// - URL: `git@host:owner/name(.git)?`, `https://...(.git)?`
    /// - owner/name shorthand: resolves to
    ///   `git@github.com:owner/name.git`
    /// - Local path: `./X`, `../X`, `/X`, `~/X`, `~`, `.`, `..`
    ///   (passed directly to `git clone`)
    #[arg(value_name = "TARGET", verbatim_doc_comment)]
    pub target: String,

    /// Destination dir name in cwd [default: derived from TARGET]
    #[arg(value_name = "NAME")]
    pub name: Option<String>,

    /// What to clone: `code,bot` (dual, default) or `por` (single).
    #[arg(
        long,
        short,
        value_name = "SCOPE",
        value_parser = parse_clone_scope,
        default_value = "code,bot"
    )]
    pub scope: CloneScope,

    /// Dry run — show what would be done without executing.
    #[arg(long)]
    pub dry_run: bool,
}

/// Top-level clone driver.
///
/// - Resolves TARGET to a concrete clone source (URL or path).
/// - Determines destination dir name (`[NAME]` override or
///   `derive_name(TARGET)`).
/// - Pre-checks target dir doesn't exist.
/// - Dispatches to `clone_one` (POR) or `clone_dual` (code,bot).
pub fn clone_repo(args: &CloneArgs) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("clone: enter");

    let parsed = parse_target(&args.target)?;
    let source = match parsed {
        Target::Url(u) => u,
        Target::OwnerName(o, n) => resolve_url(&format!("{o}/{n}")),
        Target::Path(p) => p.to_str().ok_or("path is not valid UTF-8")?.to_string(),
    };

    let name = match &args.name {
        Some(n) => n.clone(),
        None => derive_name(&args.target)?,
    };
    let parent_dir = std::env::current_dir()?;
    let project_dir = parent_dir.join(&name);

    if project_dir.exists() {
        return Err(format!("'{}' already exists", project_dir.display()).into());
    }

    if args.dry_run {
        info!("Dry run — would execute:");
        match args.scope {
            CloneScope::Por => {
                info!("  1. jj git clone --colocate {source} {name}");
            }
            CloneScope::CodeBot => {
                let session_source = derive_session_url(&source);
                info!("  1. jj git clone --colocate {source} {name}");
                info!("  2. jj git clone --colocate {session_source} {name}/.claude");
                info!("  3. Create Claude Code symlink");
            }
        }
        return Ok(());
    }

    run("jj", &["--version"], Path::new(".")).map_err(|_| "jj is not installed")?;

    match args.scope {
        CloneScope::Por => {
            clone_one(&source, &project_dir, &parent_dir)?;
            info!("");
            info!("Done! Project cloned to {}", project_dir.display());
            info!("  Code repo: {}", project_dir.display());
        }
        CloneScope::CodeBot => clone_dual(&source, &project_dir, &parent_dir)?,
    }

    log::debug!("clone: exit");
    Ok(())
}

/// Clone a single repo via `jj git clone --colocate` and verify
/// bookmark tracking.
///
/// - `source` is the clone source (URL or local path).
/// - `target_dir` is the destination working repo location.
/// - `parent_dir` is the cwd for the `jj git clone` subprocess.
///
/// `jj git clone --colocate` does git clone + jj init + automatic
/// bookmark tracking in one step (unlike colocate-after-bare-git-clone,
/// which needs explicit `jj bookmark track`). The `verify_tracking`
/// call asserts the post-clone state matches expectations.
pub(crate) fn clone_one(
    source: &str,
    target_dir: &Path,
    parent_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let target_str = target_dir
        .to_str()
        .ok_or("target path is not valid UTF-8")?;
    info!("Cloning {source} → {target_str}...");
    run(
        "jj",
        &["git", "clone", "--colocate", source, target_str],
        parent_dir,
    )?;
    crate::common::verify_tracking(target_dir, "main")?;
    Ok(())
}

/// Orchestrate a dual-repo clone: code via `clone_one`, bot via
/// `clone_one` (no graceful skip — `--scope=code,bot` requires
/// both sides; users who want code-only should use `--scope=por`),
/// then create the Claude Code symlink.
pub(crate) fn clone_dual(
    code_source: &str,
    target_dir: &Path,
    parent_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let session_source = derive_session_url(code_source);
    let session_dir = target_dir.join(".claude");

    clone_one(code_source, target_dir, parent_dir)?;
    clone_one(&session_source, &session_dir, target_dir)?;
    info!("Creating Claude Code symlink...");
    let sl = symlink::install(target_dir)?;

    info!("");
    info!("Done! Project cloned to {}", target_dir.display());
    info!("  Code repo:    {}", target_dir.display());
    info!("  Session repo: {}", session_dir.display());
    info!(
        "  Symlink:      {} -> {}",
        sl.symlink_path.display(),
        sl.abs_target.display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cli, Commands};
    use clap::Parser;

    fn parse(args: &[&str]) -> CloneArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Clone(a) => a,
            _ => panic!("expected Clone"),
        }
    }

    fn parse_err(args: &[&str]) -> String {
        Cli::try_parse_from(args).unwrap_err().to_string()
    }

    #[test]
    fn defaults() {
        let args = parse(&["vc-x1", "clone", "owner/repo"]);
        assert_eq!(args.target, "owner/repo");
        assert!(args.name.is_none());
        assert_eq!(args.scope, CloneScope::CodeBot);
        assert!(!args.dry_run);
    }

    #[test]
    fn with_name() {
        let args = parse(&["vc-x1", "clone", "owner/repo", "my-dir"]);
        assert_eq!(args.target, "owner/repo");
        assert_eq!(args.name.as_deref(), Some("my-dir"));
    }

    #[test]
    fn all_opts() {
        let args = parse(&[
            "vc-x1",
            "clone",
            "owner/repo",
            "my-dir",
            "--scope",
            "por",
            "--dry-run",
        ]);
        assert_eq!(args.target, "owner/repo");
        assert_eq!(args.name.as_deref(), Some("my-dir"));
        assert_eq!(args.scope, CloneScope::Por);
        assert!(args.dry_run);
    }

    #[test]
    fn missing_target() {
        let err = parse_err(&["vc-x1", "clone"]);
        assert!(err.contains("TARGET"));
    }

    #[test]
    fn scope_short_flag() {
        let args = parse(&["vc-x1", "clone", "owner/repo", "-s", "por"]);
        assert_eq!(args.scope, CloneScope::Por);
    }

    #[test]
    fn scope_bot_code_commutative() {
        let args = parse(&["vc-x1", "clone", "owner/repo", "--scope", "bot,code"]);
        assert_eq!(args.scope, CloneScope::CodeBot);
    }

    #[test]
    fn scope_code_alone_errors() {
        let err = parse_err(&["vc-x1", "clone", "owner/repo", "--scope", "code"]);
        assert!(err.contains("not valid for clone"), "got: {err}");
    }

    #[test]
    fn scope_bot_alone_errors() {
        let err = parse_err(&["vc-x1", "clone", "owner/repo", "--scope", "bot"]);
        assert!(err.contains("not valid for clone"), "got: {err}");
    }

    #[test]
    fn scope_unknown_errors() {
        let err = parse_err(&["vc-x1", "clone", "owner/repo", "--scope", "xyz"]);
        assert!(err.contains("not recognized"), "got: {err}");
    }

    #[test]
    fn target_path_form_accepted() {
        let args = parse(&["vc-x1", "clone", "/tmp/foo.git"]);
        assert_eq!(args.target, "/tmp/foo.git");
    }

    #[test]
    fn target_relative_path_form_accepted() {
        let args = parse(&["vc-x1", "clone", "./bare.git"]);
        assert_eq!(args.target, "./bare.git");
    }

    // Unit tests for derive_name / resolve_url / derive_session_url
    // live in src/repo_url.rs alongside the lifted functions.
}

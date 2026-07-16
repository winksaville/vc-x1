//! `vc-x1 clone` — clone a repo (URL or local path) into a workspace.
//!
//! - Default (no `--por`): dual-repo layout — clones code, derives
//!   bot source (`<source>.claude`), clones bot into
//!   `<target>/.claude`, creates the Claude Code symlink. Both
//!   sides must succeed.
//! - `--por`: single repo into `<target>`. No `.claude/`, no
//!   symlink.
//!
//! TARGET shapes (all routed through `parse_target`): URL,
//! `owner/name` shorthand, or a local path (`./X`, `/X`, `~/X`,
//! `.`, `..`). Path-form is symmetric with `git clone /local/bare.git`
//! — useful for fixtures and CI scratch dirs.
//!
//! `clone_one` and `clone_dual` are `pub(crate)` so init's `-3`
//! reshape can reuse them for the "URL exists → clone" preflight
//! path (per `notes/chores/chores-08.md > init + clone redesign`).

use std::path::Path;

use clap::Args;
use log::info;

use crate::common::run;
use crate::context::Context;
use crate::options_flags::por::PorFlag;
use crate::subcommand::SubcommandRunner;
use crate::symlink;
use crate::url::{Target, derive_name, derive_session_url, parse_target, resolve_url};

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

    /// Flatten of the shared [`PorFlag`] leaf — `--por` switches
    /// the clone shape from dual (default) to single repo.
    #[command(flatten)]
    pub por: PorFlag,

    /// Dry run — show what would be done without executing.
    #[arg(long)]
    pub dry_run: bool,
}

/// Inputs to the clone op, flat, owned, clap-free.
///
/// - `target`: the `TARGET` positional (URL, `owner/name`, or
///   local path).
/// - `name`: optional `NAME` positional override for the
///   destination dir.
/// - `por`: `--por` resolved — `true` for plain single repo,
///   `false` (default) for dual.
/// - `dry_run`: `--dry-run`.
pub struct CloneParams {
    pub target: String,
    pub name: Option<String>,
    pub por: bool,
    pub dry_run: bool,
}

impl From<&CloneArgs> for CloneParams {
    /// Convert clap-derived `CloneArgs` into the flat `CloneParams`
    /// (total — every field copies straight over).
    fn from(a: &CloneArgs) -> Self {
        Self {
            target: a.target.clone(),
            name: a.name.clone(),
            por: a.por.value,
            dry_run: a.dry_run,
        }
    }
}

impl SubcommandRunner for CloneArgs {
    type Params = CloneParams;

    /// Delegate to the existing `From<&CloneArgs>` impl above
    /// (total — never fails).
    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(CloneParams::from(self))
    }

    /// Run the existing `clone_repo` op.
    fn run(ctx: &Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        clone_repo(ctx, params)
    }
}

/// Top-level clone driver.
///
/// - Resolves TARGET to a concrete clone source (URL or path).
/// - Determines destination dir name (`[NAME]` override or
///   `derive_name(TARGET)`).
/// - Pre-checks target dir doesn't exist.
/// - Dispatches to `clone_one` (POR) or `clone_dual` (code,bot).
///
/// `ctx` is unused today (clone reads neither user config nor the
/// `--log` path); it's present for the uniform subcommand-layer
/// signature.
pub fn clone_repo(_ctx: &Context, params: &CloneParams) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("clone: enter");

    let parsed = parse_target(&params.target)?;
    let source = match parsed {
        Target::Url(u) => u,
        Target::OwnerName(o, n) => resolve_url(&format!("{o}/{n}")),
        Target::Path(p) => p.to_str().ok_or("path is not valid UTF-8")?.to_string(),
        Target::BareName(n) => {
            return Err(format!(
                "'{n}' is a bare name — clone has no config-driven defaults; \
                 use 'owner/{n}', a full URL, or './{n}' for a local path"
            )
            .into());
        }
    };

    let name = match &params.name {
        Some(n) => n.clone(),
        None => derive_name(&params.target)?,
    };
    let parent_dir = std::env::current_dir()?;
    let project_dir = parent_dir.join(&name);

    if project_dir.exists() {
        return Err(format!("'{}' already exists", project_dir.display()).into());
    }

    if params.dry_run {
        info!("Dry run — would execute:");
        if params.por {
            info!("  1. jj git clone --colocate {source} {name}");
        } else {
            let session_source = derive_session_url(&source);
            info!("  1. jj git clone --colocate {source} {name}");
            info!("  2. jj git clone --colocate {session_source} {name}/.claude");
            info!("  3. Create Claude Code symlink");
        }
        return Ok(());
    }

    run("jj", &["--version"], Path::new(".")).map_err(|_| "jj is not installed")?;

    if params.por {
        clone_one(&source, &project_dir, &parent_dir)?;
        info!("");
        info!("Done! Project cloned to {}", project_dir.display());
        info!("  Work repo: {}", project_dir.display());
    } else {
        clone_dual(&source, &project_dir, &parent_dir)?;
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
/// `clone_one` (no graceful skip — both sides required by the
/// default dual shape; users who want code-only pass `--por`),
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
    info!("  Work repo:    {}", target_dir.display());
    info!("  Bot repo: {}", session_dir.display());
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
            Some(Commands::Clone(a)) => a,
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
        assert!(!args.por.value);
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
            "--por",
            "--dry-run",
        ]);
        assert_eq!(args.target, "owner/repo");
        assert_eq!(args.name.as_deref(), Some("my-dir"));
        assert!(args.por.value);
        assert!(args.dry_run);
    }

    #[test]
    fn missing_target() {
        let err = parse_err(&["vc-x1", "clone"]);
        assert!(err.contains("TARGET"));
    }

    #[test]
    fn por_flag() {
        let args = parse(&["vc-x1", "clone", "owner/repo", "--por"]);
        assert!(args.por.value);
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
    // live in src/url.rs alongside the lifted functions.
}

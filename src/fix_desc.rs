//! The `fix-desc` subcommand: scan a revision range and repair
//! each commit's `ochid:` trailer against the cross-referenced
//! repo — fixing wrong prefixes/lengths, adding missing trailers
//! (`--add-missing`), or substituting a fallback value.
//!
//! Dry-run by default; `--no-dry-run` actually rewrites
//! descriptions via `jj describe --ignore-immutable`.

use std::path::PathBuf;

use clap::Args;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::Repo;
use log::{debug, info};

use crate::common;
use crate::context::Context;
use crate::desc_helpers::{
    DEFAULT_ID_LEN, OchidIssues, TitleMatch, VC_CONFIG_FILE, append_ochid_trailer, extract_bare_id,
    extract_ochid_from_desc, find_matching_commit, fix_ochid_in_description,
    ochid_prefix_from_config, resolve_full_change_id, validate_ochid,
};
use crate::subcommand::SubcommandRunner;
use crate::toml_simple;

/// Fix commit descriptions against the other repo.
#[derive(Args, Debug)]
pub struct FixDescArgs {
    /// Revision (with optional .. notation)
    #[arg(value_name = "REVISION")]
    pub pos_rev: Option<String>,

    /// Number of commits (per open side)
    #[arg(value_name = "COMMITS")]
    pub pos_count: Option<usize>,

    /// Revision(s) to scan
    #[arg(short, long, default_value = "@")]
    pub revision: String,

    /// Number of commits to scan
    #[arg(short = 'n', long = "commits", value_name = "COMMITS")]
    pub limit: Option<usize>,

    /// Maximum number of commits to actually fix/change [default: all]
    #[arg(short = 'm', long = "max-fixes")]
    pub max_fixes: Option<usize>,

    /// Path to jj repo
    #[arg(short = 'R', long, default_value = ".")]
    pub repo: PathBuf,

    /// Path to the other repo [default: from .vc-config.toml]
    #[arg(long = "other-repo")]
    pub other_repo: Option<PathBuf>,

    /// Expected changeID length
    #[arg(long = "id-len", default_value_t = DEFAULT_ID_LEN)]
    pub id_len: usize,

    /// New title to replace the first line (optional)
    #[arg(long)]
    pub title: Option<String>,

    /// Fallback ochid value for IDs not found in other repo (e.g. /.claude/lost)
    #[arg(long)]
    pub fallback: Option<String>,

    /// Actually write changes [default: dry-run]
    #[arg(long = "no-dry-run")]
    pub no_dry_run: bool,

    /// Add missing ochid trailers by matching title and timestamp
    #[arg(long = "add-missing")]
    pub add_missing: bool,
}

/// Inputs to the fix-desc op, flat, owned, clap-free.
///
/// Mirrors `FixDescArgs` field-for-field: positional `REVISION` /
/// `COMMITS` (`pos_rev` / `pos_count`), `--revision` / `--commits`
/// (`revision` / `limit`), `--max-fixes`, `--repo`, `--other-repo`,
/// `--id-len`, `--title`, `--fallback`, `--no-dry-run`,
/// `--add-missing`.
pub struct FixDescParams {
    pub pos_rev: Option<String>,
    pub pos_count: Option<usize>,
    pub revision: String,
    pub limit: Option<usize>,
    pub max_fixes: Option<usize>,
    pub repo: PathBuf,
    pub other_repo: Option<PathBuf>,
    pub id_len: usize,
    pub title: Option<String>,
    pub fallback: Option<String>,
    pub no_dry_run: bool,
    pub add_missing: bool,
}

impl From<&FixDescArgs> for FixDescParams {
    /// Convert clap-derived `FixDescArgs` into the flat
    /// `FixDescParams` (total — every field copies straight over).
    fn from(a: &FixDescArgs) -> Self {
        Self {
            pos_rev: a.pos_rev.clone(),
            pos_count: a.pos_count,
            revision: a.revision.clone(),
            limit: a.limit,
            max_fixes: a.max_fixes,
            repo: a.repo.clone(),
            other_repo: a.other_repo.clone(),
            id_len: a.id_len,
            title: a.title.clone(),
            fallback: a.fallback.clone(),
            no_dry_run: a.no_dry_run,
            add_missing: a.add_missing,
        }
    }
}

impl SubcommandRunner for FixDescArgs {
    type Params = FixDescParams;

    /// Delegate to the existing `From<&FixDescArgs>` impl above
    /// (total — never fails).
    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(FixDescParams::from(self))
    }

    /// Run the existing `fix_desc` op.
    fn run(ctx: &Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        fix_desc(ctx, params)
    }
}

/// Run the `fix-desc` subcommand: scan the resolved revision range
/// and repair each commit's ochid trailer against the other repo;
/// errors if any commit could not be fixed.
///
/// `ctx` is unused today (fix-desc reads `.vc-config.toml`, not the
/// user config, and doesn't touch the `--log` path); it's present
/// for the uniform subcommand-layer signature.
pub fn fix_desc(_ctx: &Context, params: &FixDescParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("fix-desc: enter");
    let (workspace, repo) = common::load_repo(&params.repo)?;

    // Resolve other repo: --other-repo flag, or scope-aware
    // resolution from the workspace config. A single-repo / POR
    // workspace has no bot side — nothing to repair against, so
    // the command no-ops instead of erroring (por equalization;
    // topology from `default_scope`, not a flag).
    let other_repo_path = match &params.other_repo {
        Some(p) => p.clone(),
        None => match common::bot_repo_path(&params.repo)? {
            Some(p) => p,
            None => {
                info!("fix-desc: single-repo workspace (no bot side) — nothing to fix");
                return Ok(());
            }
        },
    };

    let (other_workspace, other_repo) = common::load_repo(&other_repo_path)?;
    let other_config = toml_simple::toml_load(&other_repo_path.join(VC_CONFIG_FILE))?;
    let other_prefix = ochid_prefix_from_config(&other_config)?;

    let spec = common::resolve_spec(
        params.pos_rev.as_deref(),
        params.pos_count,
        &params.revision,
        params.limit,
        "@",
    );

    let (ids, _anchor_index) = common::collect_ids(
        &workspace,
        &repo,
        &spec.rev,
        spec.desc_count,
        spec.anc_count,
    )?;
    if ids.is_empty() {
        return Err(format!("no commits found for revision '{}'", params.revision).into());
    }

    let root_id = repo.store().root_commit_id().clone();
    let mut fixed = 0;
    let mut skipped = 0;
    let mut errors = 0;

    for commit_id in &ids {
        if *commit_id == root_id {
            continue;
        }
        let commit = repo.store().get_commit(commit_id)?;
        let desc = commit.description();
        let change_hex = jj_lib::hex_util::encode_reverse_hex(commit.change_id().as_bytes());
        let change_short = &change_hex[..change_hex.len().min(12)];
        let first_line = desc.lines().next().unwrap_or(""); // OK: obvious
        let display_title = if first_line.is_empty() {
            "(no description set)"
        } else {
            first_line
        };

        // Extract current ochid value
        let current_ochid = common::extract_ochid(&commit);

        // Validate current ochid
        let issues = if let Some(ref ochid_val) = current_ochid {
            validate_ochid(
                ochid_val,
                &other_prefix,
                params.id_len,
                &other_workspace,
                &other_repo,
            )
        } else if params.add_missing {
            // Stop if we've hit the max-fixes limit
            if let Some(max) = params.max_fixes
                && fixed >= max
            {
                skipped += 1;
                if !params.no_dry_run {
                    info!("skip {change_short}  {display_title}  (max-fixes reached)");
                }
                continue;
            }
            // No ochid trailer — try to infer from the other repo
            match find_matching_commit(&commit, &other_workspace, &other_repo)? {
                TitleMatch::NoTitle => {
                    skipped += 1;
                    if !params.no_dry_run {
                        info!("skip {change_short}  {display_title}");
                    }
                    continue;
                }
                TitleMatch::One(matched_id) => {
                    let new_desc =
                        append_ochid_trailer(desc, &other_prefix, &matched_id, params.id_len);
                    let short_matched = &matched_id[..matched_id.len().min(params.id_len)];
                    if !params.no_dry_run {
                        info!("add  {change_short}  {display_title}  [missing]");
                        info!("     -> ochid: {other_prefix}{short_matched}");
                    } else {
                        jj_describe(commit_id, &new_desc, &params.repo, change_short)?;
                        info!("added {change_short}  {display_title}");
                    }
                    fixed += 1;
                    continue;
                }
                TitleMatch::Ambiguous(n) => {
                    skipped += 1;
                    if !params.no_dry_run {
                        info!(
                            "skip {change_short}  {display_title}  ({n} title matches, ambiguous)"
                        );
                    }
                    continue;
                }
                TitleMatch::None => {
                    skipped += 1;
                    if !params.no_dry_run {
                        info!(
                            "skip {change_short}  {display_title}  (no matching title in other repo)"
                        );
                    }
                    continue;
                }
            }
        } else {
            // No ochid trailer — nothing to fix
            skipped += 1;
            if !params.no_dry_run {
                info!("skip {change_short}  {display_title}  (no ochid trailer)");
            }
            continue;
        };

        if !issues.any() {
            skipped += 1;
            if !params.no_dry_run {
                info!("ok   {change_short}  {display_title}");
            }
            continue;
        }

        // Stop if we've hit the max-fixes limit
        if let Some(max) = params.max_fixes
            && fixed >= max
        {
            skipped += 1;
            if !params.no_dry_run {
                info!("skip {change_short}  {display_title}  (max-fixes reached)");
            }
            continue;
        }

        // Resolve the full change ID from the other repo when length is wrong
        let resolved_id = if issues.wrong_length.is_some() {
            let ochid_val = current_ochid.as_deref().unwrap_or(""); // OK: obvious
            let bare_id = extract_bare_id(ochid_val);
            resolve_full_change_id(bare_id, &other_workspace, &other_repo)?
        } else {
            None
        };

        // Build the fixed description
        let mut new_desc = fix_ochid_in_description(
            desc,
            &other_prefix,
            params.id_len,
            params.title.as_deref(),
            resolved_id.as_deref(),
        );

        // Re-extract the fixed ochid value for post-validation
        let mut fixed_ochid = extract_ochid_from_desc(&new_desc);

        let post_issues = if let Some(ref v) = fixed_ochid {
            validate_ochid(
                v,
                &other_prefix,
                params.id_len,
                &other_workspace,
                &other_repo,
            )
        } else {
            OchidIssues {
                wrong_prefix: None,
                wrong_length: None,
                not_found: true,
                bare_id: String::new(),
            }
        };

        if post_issues.not_found {
            if let Some(ref fallback) = params.fallback {
                // Replace the ochid line with the fallback value
                let mut fb_lines: Vec<String> = new_desc.lines().map(|l| l.to_string()).collect();
                for line in &mut fb_lines {
                    if line.trim().starts_with("ochid:") {
                        *line = format!("ochid: {fallback}");
                    }
                }
                let fb_desc = if new_desc.ends_with('\n') {
                    format!("{}\n", fb_lines.join("\n"))
                } else {
                    fb_lines.join("\n")
                };
                new_desc = fb_desc;
                fixed_ochid = Some(fallback.clone());
            } else {
                errors += 1;
                info!(
                    "err  {change_short}  {display_title}  (ID not found, ochid: {})",
                    fixed_ochid.as_deref().unwrap_or("?") // OK: "?" placeholder when ochid unresolved
                );
                continue;
            }
        }

        if !params.no_dry_run {
            info!(
                "fix  {change_short}  {display_title}  [{}]",
                issues.summary()
            );
            if let Some(ref v) = fixed_ochid {
                info!("     -> ochid: {v}");
            }
        } else {
            jj_describe(commit_id, &new_desc, &params.repo, change_short)?;
            let fixed_title = new_desc.lines().next().unwrap_or(""); // OK: obvious
            info!("fixed {change_short}  {fixed_title}");
        }
        fixed += 1;
    }

    info!("");
    info!(
        "{fixed} fixed, {skipped} skipped, {errors} errors (of {} total)",
        ids.len()
    );
    if errors > 0 {
        debug!("fix-desc: exit with errors");
        Err(format!("{errors} commit(s) could not be fixed").into())
    } else {
        debug!("fix-desc: exit");
        Ok(())
    }
}

/// Run `jj describe` to rewrite a commit's description.
fn jj_describe(
    commit_id: &jj_lib::backend::CommitId,
    new_desc: &str,
    repo_path: &std::path::Path,
    _change_short: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::common::run(
        "jj",
        &[
            "describe",
            "-m",
            new_desc,
            "-r",
            &commit_id.hex(),
            "-R",
            &repo_path.to_string_lossy(),
            "--ignore-immutable",
        ],
        repo_path,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::FixturePor;

    /// A POR workspace has no bot side: fix-desc no-ops with
    /// `Ok(())` instead of erroring (por equalization).
    #[test]
    fn por_workspace_noops() {
        let fx = FixturePor::new("fdesc-por-noop");
        let ctx = Context::load().expect("load user config");
        let params = FixDescParams {
            pos_rev: None,
            pos_count: None,
            revision: "@".to_string(),
            limit: None,
            max_fixes: None,
            repo: fx.work.clone(),
            other_repo: None,
            id_len: DEFAULT_ID_LEN,
            title: None,
            fallback: None,
            no_dry_run: false,
            add_missing: false,
        };
        fix_desc(&ctx, &params).expect("por workspace should no-op, not error");
    }
}

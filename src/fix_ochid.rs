use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use clap::Args;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::{ReadonlyRepo, Repo};
use jj_lib::workspace::Workspace;

use crate::common;
use crate::toml_simple;

const DEFAULT_ID_LEN: usize = 12;
const VC_CONFIG_FILE: &str = ".vc-config.toml";

#[derive(Args, Debug)]
pub struct FixOchidArgs {
    /// Revision (with optional .. notation)
    #[arg(value_name = "REVISION")]
    pub pos_rev: Option<String>,

    /// Number of commits (per dotted side)
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

    /// Path to the other repo (e.g. .claude or .)
    #[arg(long = "other-repo")]
    pub other_repo: PathBuf,

    /// Expected changeID length
    #[arg(long = "id-len", default_value_t = DEFAULT_ID_LEN)]
    pub id_len: usize,

    /// New title to replace the first line (optional)
    #[arg(long)]
    pub title: Option<String>,

    /// Fallback ochid value for IDs not found in other repo (e.g. /.claude/lost)
    #[arg(long)]
    pub fallback: Option<String>,

    /// Actually write changes (default is dry-run)
    #[arg(long = "no-dry-run")]
    pub no_dry_run: bool,

    /// Add missing ochid trailers by matching title and timestamp
    #[arg(long = "add-missing")]
    pub add_missing: bool,
}

/// Derive the ochid prefix from the other repo's `.vc-config.toml`.
///
/// The `workspace.path` value is the repo's path relative to the workspace
/// root, which is exactly the ochid prefix (with a trailing `/` when the
/// path isn't just `/`).
fn ochid_prefix_from_config(
    config: &std::collections::HashMap<String, String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let ws_path = toml_simple::toml_get(config, "workspace.path")
        .ok_or("missing workspace.path in .vc-config.toml")?;
    let trimmed = ws_path.trim_end_matches('/');
    if trimmed.is_empty() {
        Ok("/".to_string())
    } else {
        Ok(format!("{trimmed}/"))
    }
}

/// Problems found with an ochid trailer.
#[derive(Debug)]
struct OchidIssues {
    wrong_prefix: bool,
    wrong_length: bool,
    not_found: bool,
}

impl OchidIssues {
    fn any(&self) -> bool {
        self.wrong_prefix || self.wrong_length || self.not_found
    }

    fn summary(&self) -> String {
        let mut parts = Vec::new();
        if self.wrong_prefix {
            parts.push("wrong prefix");
        }
        if self.wrong_length {
            parts.push("wrong ID length");
        }
        if self.not_found {
            parts.push("ID not found in other repo");
        }
        parts.join(", ")
    }
}

/// Validate an ochid trailer value against expected prefix, length, and other repo.
fn validate_ochid(
    value: &str,
    other_prefix: &str,
    id_len: usize,
    other_workspace: &Workspace,
    other_repo: &Arc<ReadonlyRepo>,
) -> OchidIssues {
    let wrong_prefix = !value.starts_with(other_prefix);

    // Extract the bare ID
    let bare_id = if let Some(pos) = value.rfind('/') {
        &value[pos + 1..]
    } else {
        value
    };

    let wrong_length = bare_id.len() != id_len;

    // Check if the ID resolves in the other repo
    let not_found = common::resolve_revset(other_workspace, other_repo, bare_id)
        .map(|ids| ids.is_empty())
        .unwrap_or(true);

    OchidIssues {
        wrong_prefix,
        wrong_length,
        not_found,
    }
}

/// Fix the ochid trailer in a commit description.
///
/// If `resolved_id` is provided, it replaces the bare changeID entirely
/// (used when the existing ID is too short to extend by truncation alone).
fn fix_ochid_in_description(
    desc: &str,
    other_prefix: &str,
    id_len: usize,
    new_title: Option<&str>,
    resolved_id: Option<&str>,
) -> String {
    let mut lines: Vec<String> = desc.lines().map(|l| l.to_string()).collect();

    // Fix title if requested
    if let Some(title) = new_title
        && !lines.is_empty()
    {
        lines[0] = title.to_string();
    }

    // Fix ochid trailer
    for line in &mut lines {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("ochid:") {
            let value = value.trim();
            let id = if let Some(rid) = resolved_id {
                &rid[..rid.len().min(id_len)]
            } else {
                // Extract the bare changeID (strip any existing path prefix)
                let bare_id = if let Some(pos) = value.rfind('/') {
                    &value[pos + 1..]
                } else {
                    value
                };
                // Normalize to id_len chars
                &bare_id[..bare_id.len().min(id_len)]
            };
            *line = format!("ochid: {other_prefix}{id}");
        }
    }

    // Preserve trailing newline if original had one
    let mut result = lines.join("\n");
    if desc.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// Maximum timestamp difference (in milliseconds) for title-based matching.
const TIMESTAMP_TOLERANCE_MS: i64 = 60_000;

/// Find a unique matching commit in the other repo by title and timestamp.
///
/// Returns the other commit's change ID (full hex) if exactly one commit
/// in the other repo has the same first description line and a committer
/// timestamp within `TIMESTAMP_TOLERANCE_MS` of the source commit.
fn find_matching_commit(
    commit: &jj_lib::commit::Commit,
    other_workspace: &Workspace,
    other_repo: &Arc<ReadonlyRepo>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let title = commit.description().lines().next().unwrap_or("");
    if title.is_empty() {
        return Ok(None);
    }
    let src_millis = commit.committer().timestamp.timestamp.0;

    // Search all commits in the other repo
    let all_ids = common::resolve_revset(other_workspace, other_repo, "all()")?;
    let root_id = other_repo.store().root_commit_id().clone();

    let mut matches = Vec::new();
    for cid in &all_ids {
        if *cid == root_id {
            continue;
        }
        let other_commit = other_repo.store().get_commit(cid)?;
        let other_title = other_commit.description().lines().next().unwrap_or("");
        if other_title != title {
            continue;
        }
        let other_millis = other_commit.committer().timestamp.timestamp.0;
        if (src_millis - other_millis).abs() <= TIMESTAMP_TOLERANCE_MS {
            let full_hex =
                jj_lib::hex_util::encode_reverse_hex(other_commit.change_id().as_bytes());
            matches.push(full_hex);
        }
    }

    if matches.len() == 1 {
        Ok(Some(matches.into_iter().next().unwrap()))
    } else {
        Ok(None)
    }
}

/// Append an ochid trailer to a commit description.
fn append_ochid_trailer(desc: &str, other_prefix: &str, change_id: &str, id_len: usize) -> String {
    let short_id = &change_id[..change_id.len().min(id_len)];
    let trailer = format!("ochid: {other_prefix}{short_id}");

    let mut result = desc.trim_end().to_string();
    // Add blank line before trailer if body exists, otherwise just newline
    if result.lines().count() > 1 {
        result.push('\n');
    } else {
        result.push_str("\n\n");
    }
    result.push_str(&trailer);
    result.push('\n');
    result
}

pub fn fix_ochid(args: &FixOchidArgs) -> Result<(), Box<dyn std::error::Error>> {
    let (workspace, repo) = common::load_repo(&args.repo)?;
    let (other_workspace, other_repo) = common::load_repo(&args.other_repo)?;
    let other_config = toml_simple::toml_load(&args.other_repo.join(VC_CONFIG_FILE))?;
    let other_prefix = ochid_prefix_from_config(&other_config)?;

    let spec = common::resolve_spec(
        args.pos_rev.as_deref(),
        args.pos_count,
        &args.revision,
        args.limit,
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
        return Err(format!("no commits found for revision '{}'", args.revision).into());
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

        // Extract current ochid value
        let current_ochid = common::extract_ochid(&commit);

        // Validate current ochid
        let issues = if let Some(ref ochid_val) = current_ochid {
            validate_ochid(
                ochid_val,
                &other_prefix,
                args.id_len,
                &other_workspace,
                &other_repo,
            )
        } else if args.add_missing {
            // Stop if we've hit the max-fixes limit
            if let Some(max) = args.max_fixes
                && fixed >= max
            {
                skipped += 1;
                if !args.no_dry_run {
                    println!("skip {change_short}  (max-fixes reached)");
                }
                continue;
            }
            // No ochid trailer — try to infer from the other repo
            if let Some(matched_id) = find_matching_commit(&commit, &other_workspace, &other_repo)?
            {
                let new_desc = append_ochid_trailer(desc, &other_prefix, &matched_id, args.id_len);
                let first_line = desc.lines().next().unwrap_or("");
                let short_matched = &matched_id[..matched_id.len().min(args.id_len)];
                if !args.no_dry_run {
                    println!("add  {change_short}  {first_line}  [missing]");
                    println!("     -> ochid: {other_prefix}{short_matched}");
                } else {
                    let status = Command::new("jj")
                        .arg("describe")
                        .arg("-m")
                        .arg(&new_desc)
                        .arg("-r")
                        .arg(commit_id.hex())
                        .arg("-R")
                        .arg(&args.repo)
                        .arg("--ignore-immutable")
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::inherit())
                        .status()?;
                    if !status.success() {
                        return Err(format!(
                            "jj describe failed for {} (exit {})",
                            change_short,
                            status.code().unwrap_or(-1)
                        )
                        .into());
                    }
                    println!("added {change_short}  {first_line}");
                }
                fixed += 1;
                continue;
            } else {
                skipped += 1;
                if !args.no_dry_run {
                    println!("skip {change_short}  (no ochid, no match in other repo)");
                }
                continue;
            }
        } else {
            // No ochid trailer — nothing to fix
            skipped += 1;
            if !args.no_dry_run {
                println!("skip {change_short}  (no ochid trailer)");
            }
            continue;
        };

        if !issues.any() {
            skipped += 1;
            if !args.no_dry_run {
                println!("ok   {change_short}  (valid)");
            }
            continue;
        }

        // Stop if we've hit the max-fixes limit
        if let Some(max) = args.max_fixes
            && fixed >= max
        {
            skipped += 1;
            if !args.no_dry_run {
                println!("skip {change_short}  (max-fixes reached)");
            }
            continue;
        }

        // Resolve the full change ID from the other repo when length is wrong
        let resolved_id = if issues.wrong_length {
            let ochid_val = current_ochid.as_deref().unwrap_or("");
            let bare_id = if let Some(pos) = ochid_val.rfind('/') {
                &ochid_val[pos + 1..]
            } else {
                ochid_val
            };
            if let Ok(commit_ids) = common::resolve_revset(&other_workspace, &other_repo, bare_id)
                && let Some(cid) = commit_ids.first()
            {
                let other_commit = other_repo.store().get_commit(cid)?;
                let full_hex =
                    jj_lib::hex_util::encode_reverse_hex(other_commit.change_id().as_bytes());
                Some(full_hex)
            } else {
                None
            }
        } else {
            None
        };

        // Build the fixed description
        let mut new_desc = fix_ochid_in_description(
            desc,
            &other_prefix,
            args.id_len,
            args.title.as_deref(),
            resolved_id.as_deref(),
        );

        // Re-extract the fixed ochid value for post-validation
        let mut fixed_ochid = {
            let mut val = None;
            for line in new_desc.lines() {
                let trimmed = line.trim();
                if let Some(v) = trimmed.strip_prefix("ochid:") {
                    val = Some(v.trim().to_string());
                }
            }
            val
        };

        let post_issues = if let Some(ref v) = fixed_ochid {
            validate_ochid(v, &other_prefix, args.id_len, &other_workspace, &other_repo)
        } else {
            OchidIssues {
                wrong_prefix: false,
                wrong_length: false,
                not_found: true,
            }
        };

        if post_issues.not_found {
            if let Some(ref fallback) = args.fallback {
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
                eprintln!(
                    "err  {change_short}  ID not found in other repo (ochid: {})",
                    fixed_ochid.as_deref().unwrap_or("?")
                );
                continue;
            }
        }

        if !args.no_dry_run {
            let first_line = new_desc.lines().next().unwrap_or("");
            println!("fix  {change_short}  {first_line}  [{}]", issues.summary());
            if let Some(ref v) = fixed_ochid {
                println!("     -> ochid: {v}");
            }
        } else {
            let status = Command::new("jj")
                .arg("describe")
                .arg("-m")
                .arg(&new_desc)
                .arg("-r")
                .arg(commit_id.hex())
                .arg("-R")
                .arg(&args.repo)
                .arg("--ignore-immutable")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::inherit())
                .status()?;
            if !status.success() {
                return Err(format!(
                    "jj describe failed for {} (exit {})",
                    change_short,
                    status.code().unwrap_or(-1)
                )
                .into());
            }
            let first_line = new_desc.lines().next().unwrap_or("");
            println!("fixed {change_short}  {first_line}");
        }
        fixed += 1;
    }

    println!(
        "\n{fixed} fixed, {skipped} skipped, {errors} errors (of {} total)",
        ids.len()
    );
    if errors > 0 {
        Err(format!("{errors} commit(s) could not be fixed").into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(ws_path: &str) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        map.insert("workspace.path".to_string(), ws_path.to_string());
        map
    }

    #[test]
    fn prefix_from_root() {
        let config = make_config("/");
        assert_eq!(ochid_prefix_from_config(&config).unwrap(), "/");
    }

    #[test]
    fn prefix_from_claude() {
        let config = make_config("/.claude");
        assert_eq!(ochid_prefix_from_config(&config).unwrap(), "/.claude/");
    }

    #[test]
    fn prefix_from_nested_path() {
        let config = make_config("/some/path");
        assert_eq!(ochid_prefix_from_config(&config).unwrap(), "/some/path/");
    }

    #[test]
    fn prefix_missing_config_key() {
        let config = std::collections::HashMap::new();
        assert!(ochid_prefix_from_config(&config).is_err());
    }

    #[test]
    fn fix_bare_id() {
        let desc = "Some title\n\nBody text.\n\nochid: tzupykyyvnrp\n";
        let result = fix_ochid_in_description(desc, "/.claude/", DEFAULT_ID_LEN, None, None);
        assert!(result.contains("ochid: /.claude/tzupykyyvnrp"));
        assert!(result.starts_with("Some title\n"));
    }

    #[test]
    fn fix_wrong_prefix() {
        let desc = "Title\n\nochid: /wrong/tzupykyyvnrp\n";
        let result = fix_ochid_in_description(desc, "/.claude/", DEFAULT_ID_LEN, None, None);
        assert!(result.contains("ochid: /.claude/tzupykyyvnrp"));
    }

    #[test]
    fn already_correct() {
        let desc = "Title\n\nochid: /.claude/tzupykyyvnrp\n";
        let result = fix_ochid_in_description(desc, "/.claude/", DEFAULT_ID_LEN, None, None);
        assert_eq!(result, desc);
    }

    #[test]
    fn truncate_long_id() {
        let desc = "Title\n\nochid: abcdefghijklmnop\n";
        let result = fix_ochid_in_description(desc, "/", DEFAULT_ID_LEN, None, None);
        assert!(result.contains("ochid: /abcdefghijkl"));
        assert!(!result.contains("mnop"));
    }

    #[test]
    fn fix_title_and_ochid() {
        let desc = "Old title\n\nochid: bare12345678\n";
        let result =
            fix_ochid_in_description(desc, "/.claude/", DEFAULT_ID_LEN, Some("New title"), None);
        assert!(result.starts_with("New title\n"));
        assert!(result.contains("ochid: /.claude/bare12345678"));
    }

    #[test]
    fn no_ochid_no_change() {
        let desc = "Title\n\nNo trailer here.\n";
        let result = fix_ochid_in_description(desc, "/.claude/", DEFAULT_ID_LEN, None, None);
        assert_eq!(result, desc);
    }

    #[test]
    fn other_path_without_trailing_slash() {
        let desc = "Title\n\nochid: abcdefghijkl\n";
        let result = fix_ochid_in_description(desc, "/", DEFAULT_ID_LEN, None, None);
        assert!(result.contains("ochid: /abcdefghijkl"));
    }

    #[test]
    fn custom_id_len() {
        let desc = "Title\n\nochid: abcdefghijklmnop\n";
        let result = fix_ochid_in_description(desc, "/", 8, None, None);
        assert!(result.contains("ochid: /abcdefgh"));
        assert!(!result.contains("ijkl"));
    }

    #[test]
    fn extend_short_id_with_resolved() {
        let desc = "Title\n\nochid: /abcdefgh\n";
        let result =
            fix_ochid_in_description(desc, "/", DEFAULT_ID_LEN, None, Some("abcdefghijkl"));
        assert!(result.contains("ochid: /abcdefghijkl"));
    }

    #[test]
    fn resolved_id_with_prefix_fix() {
        let desc = "Title\n\nochid: /wrong/abcdefgh\n";
        let result = fix_ochid_in_description(
            desc,
            "/.claude/",
            DEFAULT_ID_LEN,
            None,
            Some("abcdefghijklmnop"),
        );
        assert!(result.contains("ochid: /.claude/abcdefghijkl"));
    }

    #[test]
    fn append_ochid_title_only() {
        let desc = "Title\n";
        let result = append_ochid_trailer(desc, "/.claude/", "abcdefghijklmnop", DEFAULT_ID_LEN);
        assert_eq!(result, "Title\n\nochid: /.claude/abcdefghijkl\n");
    }

    #[test]
    fn append_ochid_with_body() {
        let desc = "Title\n\nSome body text.\n";
        let result = append_ochid_trailer(desc, "/", "abcdefghijklmnop", DEFAULT_ID_LEN);
        assert_eq!(result, "Title\n\nSome body text.\nochid: /abcdefghijkl\n");
    }

    #[test]
    fn append_ochid_root_prefix() {
        let desc = "Title\n";
        let result = append_ochid_trailer(desc, "/", "xyzxyzxyzxyz", DEFAULT_ID_LEN);
        assert_eq!(result, "Title\n\nochid: /xyzxyzxyzxyz\n");
    }
}

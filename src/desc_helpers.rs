#![allow(dead_code)] // Some helpers are unused until fix-desc lands in dev2.

use std::collections::HashMap;
use std::sync::Arc;

use jj_lib::commit::Commit;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::{ReadonlyRepo, Repo};
use jj_lib::workspace::Workspace;

use crate::common;
use crate::toml_simple;

pub const DEFAULT_ID_LEN: usize = 12;
pub const VC_CONFIG_FILE: &str = ".vc-config.toml";

/// Maximum timestamp difference (in milliseconds) for title-based matching.
pub const TIMESTAMP_TOLERANCE_MS: i64 = 60_000;

/// Derive the ochid prefix from a repo's `.vc-config.toml`.
///
/// The `workspace.path` value is the repo's path relative to the workspace
/// root, which is exactly the ochid prefix (with a trailing `/` when the
/// path isn't just `/`).
pub fn ochid_prefix_from_config(
    config: &HashMap<String, String>,
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
pub struct OchidIssues {
    pub wrong_prefix: bool,
    pub wrong_length: bool,
    pub not_found: bool,
}

impl OchidIssues {
    pub fn any(&self) -> bool {
        self.wrong_prefix || self.wrong_length || self.not_found
    }

    pub fn summary(&self) -> String {
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
pub fn validate_ochid(
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

/// Find a unique matching commit in the other repo by title and timestamp.
///
/// Returns the other commit's change ID (full hex) if exactly one commit
/// in the other repo has the same first description line and a committer
/// timestamp within `TIMESTAMP_TOLERANCE_MS` of the source commit.
pub fn find_matching_commit(
    commit: &Commit,
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

/// Resolve a short change ID to its full hex via the other repo.
pub fn resolve_full_change_id(
    bare_id: &str,
    other_workspace: &Workspace,
    other_repo: &Arc<ReadonlyRepo>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if let Ok(commit_ids) = common::resolve_revset(other_workspace, other_repo, bare_id)
        && let Some(cid) = commit_ids.first()
    {
        let other_commit = other_repo.store().get_commit(cid)?;
        let full_hex = jj_lib::hex_util::encode_reverse_hex(other_commit.change_id().as_bytes());
        Ok(Some(full_hex))
    } else {
        Ok(None)
    }
}

/// Extract the bare change ID from an ochid value (strip path prefix).
pub fn extract_bare_id(ochid_value: &str) -> &str {
    if let Some(pos) = ochid_value.rfind('/') {
        &ochid_value[pos + 1..]
    } else {
        ochid_value
    }
}

/// Fix the ochid trailer in a commit description.
///
/// If `resolved_id` is provided, it replaces the bare changeID entirely
/// (used when the existing ID is too short to extend by truncation alone).
pub fn fix_ochid_in_description(
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
                let bare_id = extract_bare_id(value);
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

/// Append an ochid trailer to a commit description.
pub fn append_ochid_trailer(
    desc: &str,
    other_prefix: &str,
    change_id: &str,
    id_len: usize,
) -> String {
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

/// Extract the ochid value from a description string (without needing a Commit).
pub fn extract_ochid_from_desc(desc: &str) -> Option<String> {
    for line in desc.lines() {
        let trimmed = line.trim();
        if let Some(v) = trimmed.strip_prefix("ochid:") {
            return Some(v.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(ws_path: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
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
        let config = HashMap::new();
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

    #[test]
    fn extract_bare_id_with_prefix() {
        assert_eq!(extract_bare_id("/.claude/abcdefghijkl"), "abcdefghijkl");
    }

    #[test]
    fn extract_bare_id_root_prefix() {
        assert_eq!(extract_bare_id("/abcdefghijkl"), "abcdefghijkl");
    }

    #[test]
    fn extract_bare_id_no_prefix() {
        assert_eq!(extract_bare_id("abcdefghijkl"), "abcdefghijkl");
    }

    #[test]
    fn extract_ochid_from_desc_found() {
        let desc = "Title\n\nBody.\n\nochid: /.claude/abcdefghijkl\n";
        assert_eq!(
            extract_ochid_from_desc(desc),
            Some("/.claude/abcdefghijkl".to_string())
        );
    }

    #[test]
    fn extract_ochid_from_desc_missing() {
        let desc = "Title\n\nNo trailer.\n";
        assert_eq!(extract_ochid_from_desc(desc), None);
    }
}

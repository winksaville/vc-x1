use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::Args;

use crate::desc_helpers::VC_CONFIG_FILE;
use crate::scope::{Scope, Side};
use crate::toml_simple;
use jj_lib::backend::CommitId;
use jj_lib::commit::Commit;
use jj_lib::config::StackedConfig;
use jj_lib::fileset::FilesetAliasesMap;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::{ReadonlyRepo, Repo, StoreFactories};
use jj_lib::repo_path::RepoPathUiConverter;
use jj_lib::revset::{
    RevsetAliasesMap, RevsetDiagnostics, RevsetExtensions, RevsetParseContext,
    RevsetWorkspaceContext, SymbolResolver,
};
use jj_lib::settings::UserSettings;
use jj_lib::workspace::{Workspace, default_working_copy_factories};
use log::{debug, error, info, trace};
use pollster::FutureExt;

/// Common CLI args shared by read-only subcommands (chid, desc, list, show).
#[derive(Args, Debug)]
pub struct CommonArgs {
    /// Revision (with optional .. notation)
    #[arg(value_name = "REVISION")]
    pub pos_rev: Option<String>,

    /// Number of commits to show (per dotted side)
    #[arg(value_name = "COMMITS")]
    pub pos_count: Option<usize>,

    /// Revision to query
    #[arg(short, long, default_value = "@")]
    pub revision: String,

    /// Path to jj repo; repeatable or comma-separated [default: .]
    #[arg(short = 'R', long = "repo", value_name = "PATH")]
    pub repos: Vec<PathBuf>,

    /// Number of commits to show
    #[arg(short = 'n', long = "commits", value_name = "COMMITS")]
    pub limit: Option<usize>,

    /// Custom label decoration between repos
    #[arg(
        short = 'l',
        long = "label",
        value_name = "TEXT",
        allow_hyphen_values = true,
        default_value = "==="
    )]
    pub label: String,

    /// Suppress label between repos
    #[arg(short = 'L', long = "no-label")]
    pub no_label: bool,
}

/// Result of parsing positional `..` notation.
///
/// Counts are `Some(0)` for closed sides and `None` for open (dotted) sides.
/// Open sides become unlimited unless a positional count is applied later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DotSpec {
    /// The bare revision (without `..`).
    pub rev: String,
    /// Descendant count: `None` = open (dotted), `Some(0)` = closed.
    pub desc_count: Option<usize>,
    /// Ancestor count: `None` = open (dotted), `Some(0)` = closed.
    pub anc_count: Option<usize>,
}

/// Parse a positional REV string for `..` notation.
///
/// Returns the bare revision with open/closed counts per side.
/// `None` means the side had dots (open for expansion);
/// `Some(0)` means no dots on that side (closed).
/// Actual counts are applied later from positional args.
pub fn parse_dot_rev(rev: &str) -> DotSpec {
    if let Some(inner) = rev.strip_prefix("..").and_then(|s| s.strip_suffix("..")) {
        DotSpec {
            rev: inner.to_string(),
            desc_count: None,
            anc_count: None,
        }
    } else if let Some(inner) = rev.strip_prefix("..") {
        DotSpec {
            rev: inner.to_string(),
            desc_count: None,
            anc_count: Some(0),
        }
    } else if let Some(inner) = rev.strip_suffix("..") {
        DotSpec {
            rev: inner.to_string(),
            desc_count: Some(0),
            anc_count: None,
        }
    } else {
        DotSpec {
            rev: rev.to_string(),
            desc_count: Some(0),
            anc_count: Some(0),
        }
    }
}

/// Resolve positional and flag args into a `DotSpec` ready for `collect_ids`.
///
/// Handles: dot notation parsing, flag vs positional precedence,
/// count-means-total semantics (subtracts 1 for anchor).
pub fn resolve_spec(
    pos_rev: Option<&str>,
    pos_count: Option<usize>,
    flag_rev: &str,
    flag_limit: Option<usize>,
    default_rev: &str,
) -> DotSpec {
    let flag_rev_set = flag_rev != default_rev;
    let rev_str = if flag_rev_set {
        flag_rev
    } else {
        pos_rev.unwrap_or(default_rev) // OK: positional absent → fall back to CLI default
    };
    let mut spec = parse_dot_rev(rev_str);

    let count = if flag_limit.is_some() {
        flag_limit
    } else {
        pos_count
    };

    if let Some(c) = count {
        let n = c.saturating_sub(1);
        let both_open = spec.desc_count.is_none() && spec.anc_count.is_none();
        if both_open {
            // Split budget across both sides: ancestors get the extra
            let desc_n = n / 2;
            let anc_n = n - desc_n;
            spec.desc_count = Some(desc_n);
            spec.anc_count = Some(anc_n);
        } else {
            if spec.desc_count.is_none() {
                spec.desc_count = Some(n);
            }
            if spec.anc_count.is_none() {
                spec.anc_count = Some(n);
            }
            // bare `x 5` → treat as `x.. 5` (ancestors)
            if spec.desc_count == Some(0) && spec.anc_count == Some(0) {
                spec.anc_count = Some(n);
            }
        }
    }

    spec
}

/// Run a shell command. Returns stdout on success.
///
/// Logs the command at `debug!` and the process streams as follows:
/// - **stderr at `debug!`** on success — jj's chatter (`Moved 1 bookmarks
///   to …`, `Rebased N commits`, `Nothing changed.`) is hidden by default
///   and surfaced with `-v`. The debug formatter indents each line two
///   spaces so subprocess output sits visually under the caller's own
///   `info!` line in `-v` mode.
/// - **stdout at `debug!`** — callers usually consume stdout as data
///   (bookmark lists, commit IDs, etc.), and `info!` would flood the user
///   with machine-readable output they didn't ask for.
/// - **Failures propagate as `Err`** carrying the stderr; the caller's
///   error handler (`main::run_command`) surfaces it at `error!`.
pub fn run(cmd: &str, args: &[&str], cwd: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let args_str = args.join(" ");
    debug!("$ {cmd} {args_str}");
    trace!("cwd: {}", cwd.display());
    let output = std::process::Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("failed to run {cmd}: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stdout.is_empty() {
        debug!("{stdout}");
    }
    if !output.status.success() {
        return Err(format!("{cmd} {args_str} failed: {stderr}").into());
    }
    if !stderr.is_empty() {
        debug!("{stderr}");
    }
    Ok(stdout)
}

/// Create a directory (and parents). Logs at debug level.
pub fn mkdir_p(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    debug!("mkdir -p {}", path.display());
    std::fs::create_dir_all(path)?;
    Ok(())
}

/// Write content to a file. Logs at debug level.
pub fn write_file(path: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    debug!("write {}", path.display());
    std::fs::write(path, content)?;
    Ok(())
}

/// Display a prompt, read a line from stdin, and log both together.
/// Returns the trimmed response.
pub fn prompt(msg: &str) -> Result<String, Box<dyn std::error::Error>> {
    use std::io::Write;
    eprint!("{msg}");
    std::io::stderr().flush()?;
    let mut response = String::new();
    std::io::stdin().read_line(&mut response)?;
    let trimmed = response.trim().to_string();
    info!("{msg}{trimmed}");
    Ok(trimmed)
}

/// Wrap text in ANSI bold escape codes.
pub fn bold(s: &str) -> String {
    format!("\x1b[1m{s}\x1b[0m")
}

/// Bold only the first line of a multi-line string.
pub fn bold_first_line(s: &str) -> String {
    if let Some(pos) = s.find('\n') {
        format!("\x1b[1m{}\x1b[0m{}", &s[..pos], &s[pos..])
    } else {
        bold(s)
    }
}

/// Indent all lines after the first by `n` spaces.
///
/// Returns the string unchanged if `n == 0` or there is only one line.
pub fn indent_body(s: &str, n: usize) -> String {
    if n == 0 {
        return s.to_string();
    }
    let pad = " ".repeat(n);
    let mut lines = s.lines();
    let mut result = match lines.next() {
        Some(first) => first.to_string(),
        None => return String::new(),
    };
    for line in lines {
        result.push('\n');
        if !line.is_empty() {
            result.push_str(&pad);
        }
        result.push_str(line);
    }
    result
}

/// Extract the ochid trailer value from a commit description, if present.
pub fn extract_ochid(commit: &Commit) -> Option<String> {
    for line in commit.description().lines().rev() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("ochid:") {
            return Some(value.trim().to_string());
        }
    }
    None
}

/// Local bookmark names pointing exactly at `commit_id`, space-separated.
///
/// Returns an empty string when no local bookmark targets the commit. Remote
/// bookmarks are not included — that's a separate display concern.
pub fn format_bookmarks_at(repo: &Arc<ReadonlyRepo>, commit_id: &CommitId) -> String {
    let view = repo.view();
    let names: Vec<String> = view
        .local_bookmarks_for_commit(commit_id)
        .map(|(name, _)| name.as_str().to_string())
        .collect();
    names.join(" ")
}

/// Format: changeID ochid(padded) [bookmarks |] first-line.
///
/// ochid column is left-padded to `width` characters. If no ochid, a blank
/// placeholder of that width is used. When `bookmarks` is non-empty it is
/// inserted before the title with a `|` separator (mirroring jj's
/// `Parent commit (@-): <chid> <hash> <bookmark> | <title>` style).
pub fn format_commit_with_ochid(commit: &Commit, width: usize, bookmarks: &str) -> String {
    let change_hex = encode_reverse_hex(commit.change_id().as_bytes());
    let change_short = &change_hex[..change_hex.len().min(12)];
    let ochid = extract_ochid(commit).unwrap_or_default(); // OK: no ochid trailer → empty string
    let first_line = commit.description().lines().next().unwrap_or(""); // OK: obvious
    let title = if first_line.is_empty() {
        "(no description set)"
    } else {
        first_line
    };
    if bookmarks.is_empty() {
        format!("{change_short}  {ochid:<width$}  {title}")
    } else {
        format!("{change_short}  {ochid:<width$}  {bookmarks} | {title}")
    }
}

/// Format: just the short changeID.
pub fn format_chid(commit: &Commit) -> String {
    let change_hex = encode_reverse_hex(commit.change_id().as_bytes());
    change_hex[..change_hex.len().min(12)].to_string()
}

/// Format: changeID commitID [bookmarks |] first-line (single line).
///
/// When `bookmarks` is non-empty it is inserted before the title with a `|`
/// separator (mirroring jj's `<chid> <hash> <bookmark> | <title>` style).
pub fn format_commit_short(commit: &Commit, bookmarks: &str) -> String {
    let change_hex = encode_reverse_hex(commit.change_id().as_bytes());
    let change_short = &change_hex[..change_hex.len().min(12)];
    let commit_hex = commit.id().hex();
    let commit_short = &commit_hex[..commit_hex.len().min(12)];
    let first_line = commit.description().lines().next().unwrap_or(""); // OK: obvious
    let title = if first_line.is_empty() {
        "(no description set)"
    } else {
        first_line
    };
    if bookmarks.is_empty() {
        format!("{change_short} {commit_short} {title}")
    } else {
        format!("{change_short} {commit_short} {bookmarks} | {title}")
    }
}

/// Format: changeID commitID [bookmarks |] first-line, then remaining description lines.
///
/// Same header rules as `format_commit_short`; body lines follow unchanged.
pub fn format_commit_full(commit: &Commit, bookmarks: &str) -> String {
    let change_hex = encode_reverse_hex(commit.change_id().as_bytes());
    let change_short = &change_hex[..change_hex.len().min(12)];
    let commit_hex = commit.id().hex();
    let commit_short = &commit_hex[..commit_hex.len().min(12)];
    let desc = commit.description();
    if desc.is_empty() {
        return if bookmarks.is_empty() {
            format!("{change_short} {commit_short} (no description set)")
        } else {
            format!("{change_short} {commit_short} {bookmarks} | (no description set)")
        };
    }
    let mut lines = desc.lines();
    let first_line = lines.next().unwrap_or(""); // OK: obvious
    let mut result = if bookmarks.is_empty() {
        format!("{change_short} {commit_short} {first_line}")
    } else {
        format!("{change_short} {commit_short} {bookmarks} | {first_line}")
    };
    for line in lines {
        result.push('\n');
        result.push_str(line);
    }
    result
}

/// Collect commit IDs for `..x..` (both directions) or any dot notation.
///
/// Returns `(commit_ids, anchor_index)` — the IDs in display order
/// (descendants closest to anchor, anchor, ancestors closest to anchor)
/// and the index of the anchor commit within the list.
///
/// - `desc_count`: `Some(n)` = n closest descendants, `None` = all, `Some(0)` = none
/// - `anc_count`: `Some(n)` = n closest ancestors, `None` = all, `Some(0)` = none
pub fn collect_ids(
    workspace: &Workspace,
    repo: &Arc<ReadonlyRepo>,
    rev: &str,
    desc_count: Option<usize>,
    anc_count: Option<usize>,
) -> Result<(Vec<CommitId>, usize), Box<dyn std::error::Error>> {
    let anchor_ids = resolve_revset(workspace, repo, rev)?;
    if anchor_ids.is_empty() {
        return Err(format!("no commit found for revision '{rev}'").into());
    }
    let anchor_id = &anchor_ids[0];
    let root_commit_id = repo.store().root_commit_id().clone();

    // Descendants (closest to anchor)
    let mut desc_ids: Vec<CommitId> = Vec::new();
    if desc_count != Some(0) {
        let descendant_ids = resolve_revset(workspace, repo, &format!("{rev}::"))?;
        desc_ids = descendant_ids
            .into_iter()
            .filter(|id| *id != root_commit_id && *id != *anchor_id)
            .collect();
        if let Some(n) = desc_count {
            let start = desc_ids.len().saturating_sub(n);
            desc_ids = desc_ids[start..].to_vec();
        }
    }

    // Ancestors (closest to anchor)
    let mut anc_ids: Vec<CommitId> = Vec::new();
    if anc_count != Some(0) {
        let ancestor_ids = resolve_revset(workspace, repo, &format!("::{rev}"))?;
        let limit = anc_count.unwrap_or(usize::MAX); // OK: no --ancestors limit → unbounded
        let mut count = 0;
        for commit_id in ancestor_ids {
            if commit_id == root_commit_id || commit_id == *anchor_id {
                continue;
            }
            anc_ids.push(commit_id);
            count += 1;
            if count >= limit {
                break;
            }
        }
    }

    let anchor_index = desc_ids.len();
    let mut result = desc_ids;
    result.push(anchor_id.clone());
    result.extend(anc_ids);
    Ok((result, anchor_index))
}

/// Header style between repos in multi-repo output.
#[derive(Debug, Clone)]
pub enum Header {
    /// Show `deco path deco` header per repo.
    Label(String),
    /// No header at all.
    None,
}

/// Resolve `-l` / `-L` flags into a `Header`.
///
/// `-L` (no header) takes precedence over `-l` (label decoration).
pub fn resolve_header(label: &str, suppress: bool) -> Header {
    if suppress {
        Header::None
    } else {
        Header::Label(label.to_string())
    }
}

/// Expand comma-separated repo paths, default to `["."]`, and run a closure for each.
///
/// Header style controls output between repos.
/// Continues past errors, printing them to stderr, and returns an error if any failed.
pub fn for_each_repo<F>(
    raw_repos: &[PathBuf],
    header: &Header,
    mut body: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(&Workspace, &Arc<ReadonlyRepo>) -> Result<(), Box<dyn std::error::Error>>,
{
    let repos: Vec<PathBuf> = if raw_repos.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        raw_repos
            .iter()
            .flat_map(|p| {
                p.to_string_lossy()
                    .split(',')
                    .map(|s| PathBuf::from(s.trim()))
                    .collect::<Vec<_>>()
            })
            .collect()
    };
    let multi = repos.len() > 1;
    let mut first = true;
    let mut errors: Vec<String> = Vec::new();

    for repo_path in &repos {
        if multi {
            match header {
                Header::Label(deco) => {
                    if !first {
                        info!("");
                    }
                    info!(
                        "{}",
                        bold(&format!("{deco} {} {deco}", repo_path.display()))
                    );
                }
                Header::None => {}
            }
        }
        first = false;

        match load_repo(repo_path) {
            Ok((workspace, repo)) => {
                if let Err(e) = body(&workspace, &repo) {
                    error!("({}): {e}", repo_path.display());
                    errors.push(format!("{}: {e}", repo_path.display()));
                }
            }
            Err(e) => {
                error!("({}): {e}", repo_path.display());
                errors.push(format!("{}: {e}", repo_path.display()));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("{} repo(s) had errors", errors.len()).into())
    }
}

pub fn load_repo(
    path: &Path,
) -> Result<(Workspace, Arc<ReadonlyRepo>), Box<dyn std::error::Error>> {
    let config = StackedConfig::with_defaults();
    let settings = UserSettings::from_config(config)?;
    let store_factories = StoreFactories::default();
    let working_copy_factories = default_working_copy_factories();

    let workspace = Workspace::load(&settings, path, &store_factories, &working_copy_factories)?;
    let repo = workspace.repo_loader().load_at_head().block_on()?;

    Ok((workspace, repo))
}

/// Parse and evaluate a revset string, returning commit IDs.
pub fn resolve_revset(
    workspace: &Workspace,
    repo: &Arc<ReadonlyRepo>,
    revset_str: &str,
) -> Result<Vec<CommitId>, Box<dyn std::error::Error>> {
    let aliases_map = RevsetAliasesMap::new();
    let fileset_aliases_map = FilesetAliasesMap::new();
    let extensions = RevsetExtensions::default();
    let workspace_root = workspace.workspace_root().to_path_buf();
    let path_converter = RepoPathUiConverter::Fs {
        cwd: workspace_root.clone(),
        base: workspace_root,
    };
    let workspace_name = workspace.workspace_name();
    let workspace_ctx = RevsetWorkspaceContext {
        path_converter: &path_converter,
        workspace_name,
    };
    let context = RevsetParseContext {
        aliases_map: &aliases_map,
        local_variables: HashMap::new(),
        user_email: "",
        date_pattern_context: chrono::Utc::now().fixed_offset().into(),
        default_ignored_remote: None,
        fileset_aliases_map: &fileset_aliases_map,
        use_glob_by_default: false,
        extensions: &extensions,
        workspace: Some(workspace_ctx),
    };

    let mut diagnostics = RevsetDiagnostics::new();
    let expression = jj_lib::revset::parse(&mut diagnostics, revset_str, &context)?;

    let no_extensions: &[Box<dyn jj_lib::revset::SymbolResolverExtension>] = &[];
    let symbol_resolver = SymbolResolver::new(repo.as_ref(), no_extensions);
    let resolved = expression.resolve_user_expression(repo.as_ref(), &symbol_resolver)?;
    let revset = resolved.evaluate(repo.as_ref())?;

    let mut commit_ids = Vec::new();
    for result in revset.iter() {
        commit_ids.push(result?);
    }
    Ok(commit_ids)
}

/// Scan `jj bookmark list -a <name>` output for non-tracking remote refs.
///
/// Tracked remotes appear indented (`  @origin: ...`); non-tracking remotes
/// appear at column 0 as `<bookmark>@<remote>: ...`. Returns the
/// non-tracking remote name if any is found.
pub fn find_non_tracking_remote(list_output: &str, bookmark: &str) -> Option<String> {
    let prefix = format!("{bookmark}@");
    for line in list_output.lines() {
        if line.starts_with(&prefix)
            && let Some(rest) = line.strip_prefix(&prefix)
            && let Some((remote, _)) = rest.split_once(':')
        {
            return Some(remote.to_string());
        }
    }
    None
}

/// Verify all remote refs for `bookmark` in `repo` are tracked.
///
/// Returns `Err` with the exact `jj bookmark track …` remediation command if
/// any non-tracking remote ref is found. Used as a preflight by repo-modifying
/// commands (sync, push, finalize) and as a post-condition assertion by setup
/// commands (init, clone, test-fixture).
pub fn verify_tracking(repo: &Path, bookmark: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repo_str = repo.to_string_lossy();
    let cwd = Path::new(".");
    let all = run(
        "jj",
        &["bookmark", "list", "-a", bookmark, "-R", &repo_str],
        cwd,
    )?;
    if let Some(remote) = find_non_tracking_remote(&all, bookmark) {
        return Err(format!(
            "bookmark '{bookmark}' has non-tracking remote '{bookmark}@{remote}' — \
             run `jj bookmark track {bookmark} --remote={remote} -R {repo_str}` to fix"
        )
        .into());
    }
    Ok(())
}

/// Walk up from `start` to find the workspace root.
///
/// The workspace root is the directory whose `.vc-config.toml` has
/// `path = "/"`. Returns `None` if no such config is found up to the
/// filesystem root — the caller is in a "plain old repo" (POR) with
/// no vc-x1 workspace metadata.
pub fn find_workspace_root_from(start: &Path) -> Option<PathBuf> {
    let mut cur = start.to_path_buf();
    loop {
        let cfg = cur.join(VC_CONFIG_FILE);
        if let Ok(map) = toml_simple::toml_load(&cfg)
            && toml_simple::toml_get(&map, "workspace.path").map(String::as_str) == Some("/")
        {
            return Some(cur);
        }
        cur = cur.parent()?.to_path_buf();
    }
}

/// Cwd-anchored wrapper over [`find_workspace_root_from`].
pub fn find_workspace_root() -> Option<PathBuf> {
    let cur = std::env::current_dir().ok()?;
    find_workspace_root_from(&cur)
}

/// Resolve the workspace's default `--scope`.
///
/// Reads `<workspace_root>/.vc-config.toml > [workspace] other-repo`:
///
/// - non-empty `other-repo` → `Roles([Code, Bot])`
/// - missing / empty `other-repo` → `Roles([Code])`
/// - `workspace_root = None` (POR) → `Roles([Code])`
///
/// Single-repo / POR refinement to `Single(_)` lands in 0.42.0-3.
pub fn default_scope(workspace_root: Option<&Path>) -> Scope {
    let Some(root) = workspace_root else {
        return Scope::Roles(vec![Side::Code]);
    };
    let cfg = match toml_simple::toml_load(&root.join(VC_CONFIG_FILE)) {
        Ok(c) => c,
        Err(_) => return Scope::Roles(vec![Side::Code]),
    };
    match toml_simple::toml_get(&cfg, "workspace.other-repo") {
        Some(v) if !v.is_empty() => Scope::Roles(vec![Side::Code, Side::Bot]),
        _ => Scope::Roles(vec![Side::Code]),
    }
}

/// Resolve a `Scope` to concrete repo paths.
///
/// `Roles(...)` arm:
///
/// - `Side::Code` → `workspace_root` (or cwd's `.` when `workspace_root` is None).
/// - `Side::Bot` → `workspace_root.join(other-repo)`.
///
/// Errors when `Side::Bot` is requested but the workspace doesn't define
/// one (POR or single-repo workspace).
///
/// `Single(_)` arm currently errors — single-repo handling lands in
/// 0.42.0-3 alongside the sync retrofit.
pub fn scope_to_repos(
    scope: &Scope,
    workspace_root: Option<&Path>,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let sides = match scope {
        Scope::Roles(v) => v,
        Scope::Single(_) => {
            return Err("Scope::Single(_) handling lands in 0.42.0-3 (sync retrofit)".into());
        }
    };
    let mut repos = Vec::new();
    for side in sides {
        match side {
            Side::Code => repos.push(
                workspace_root
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| PathBuf::from(".")),
            ),
            Side::Bot => {
                let root = workspace_root.ok_or(
                    "--scope=bot: not in a vc-x1 workspace (no .vc-config.toml with path = \"/\") — drop --scope or use --scope=code",
                )?;
                let cfg = toml_simple::toml_load(&root.join(VC_CONFIG_FILE))?;
                let other = toml_simple::toml_get(&cfg, "workspace.other-repo")
                    .filter(|v| !v.is_empty())
                    .ok_or(
                        "--scope=bot: no other-repo configured. Add `other-repo = \"…\"` to .vc-config.toml to enable dual-repo operations",
                    )?;
                repos.push(root.join(other));
            }
        }
    }
    Ok(repos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_non_tracking_remote_tracked() {
        let output = "\
main: zzwozmkn 40a8309a title here
  @git: zzwozmkn 40a8309a title here
  @origin: zzwozmkn 40a8309a title here";
        assert_eq!(find_non_tracking_remote(output, "main"), None);
    }

    #[test]
    fn find_non_tracking_remote_untracked() {
        let output = "\
main: zzwozmkn 40a8309a title here
main@origin: zzwozmkn 40a8309a title here";
        assert_eq!(
            find_non_tracking_remote(output, "main"),
            Some("origin".to_string())
        );
    }

    #[test]
    fn find_non_tracking_remote_no_remote() {
        let output = "main: zzwozmkn 40a8309a title here";
        assert_eq!(find_non_tracking_remote(output, "main"), None);
    }

    #[test]
    fn find_non_tracking_remote_other_bookmark() {
        let output = "\
main: zzwozmkn 40a8309a title here
  @origin: zzwozmkn 40a8309a title here
other@origin: abcd1234 5678efgh other stuff";
        assert_eq!(find_non_tracking_remote(output, "main"), None);
    }

    #[test]
    fn parse_dot_rev_bare() {
        let spec = parse_dot_rev("@");
        assert_eq!(spec.rev, "@");
        assert_eq!(spec.desc_count, Some(0));
        assert_eq!(spec.anc_count, Some(0));
    }

    #[test]
    fn parse_dot_rev_ancestors() {
        let spec = parse_dot_rev("@..");
        assert_eq!(spec.rev, "@");
        assert_eq!(spec.desc_count, Some(0));
        assert_eq!(spec.anc_count, None);
    }

    #[test]
    fn parse_dot_rev_descendants() {
        let spec = parse_dot_rev("..@");
        assert_eq!(spec.rev, "@");
        assert_eq!(spec.desc_count, None);
        assert_eq!(spec.anc_count, Some(0));
    }

    #[test]
    fn parse_dot_rev_both() {
        let spec = parse_dot_rev("..@..");
        assert_eq!(spec.rev, "@");
        assert_eq!(spec.desc_count, None);
        assert_eq!(spec.anc_count, None);
    }

    #[test]
    fn parse_dot_rev_changeid() {
        let spec = parse_dot_rev("abcd..");
        assert_eq!(spec.rev, "abcd");
        assert_eq!(spec.desc_count, Some(0));
        assert_eq!(spec.anc_count, None);
    }

    #[test]
    fn parse_dot_rev_both_changeid() {
        let spec = parse_dot_rev("..abcd..");
        assert_eq!(spec.rev, "abcd");
        assert_eq!(spec.desc_count, None);
        assert_eq!(spec.anc_count, None);
    }

    #[test]
    fn resolve_spec_defaults() {
        let s = resolve_spec(None, None, "@", None, "@");
        assert_eq!(s.rev, "@");
        assert_eq!(s.desc_count, Some(0));
        assert_eq!(s.anc_count, Some(0));
    }

    #[test]
    fn resolve_spec_bare_with_count() {
        let s = resolve_spec(Some("@"), Some(5), "@", None, "@");
        assert_eq!(s.rev, "@");
        assert_eq!(s.desc_count, Some(0));
        assert_eq!(s.anc_count, Some(4)); // 5 - 1 = 4 ancestors
    }

    #[test]
    fn resolve_spec_ancestors() {
        let s = resolve_spec(Some("@.."), Some(3), "@", None, "@");
        assert_eq!(s.rev, "@");
        assert_eq!(s.desc_count, Some(0));
        assert_eq!(s.anc_count, Some(2)); // 3 - 1 = 2 ancestors
    }

    #[test]
    fn resolve_spec_descendants() {
        let s = resolve_spec(Some("..@"), Some(3), "@", None, "@");
        assert_eq!(s.rev, "@");
        assert_eq!(s.desc_count, Some(2)); // 3 - 1 = 2 descendants
        assert_eq!(s.anc_count, Some(0));
    }

    #[test]
    fn resolve_spec_both() {
        let s = resolve_spec(Some("..@.."), Some(5), "@", None, "@");
        assert_eq!(s.rev, "@");
        assert_eq!(s.desc_count, Some(2)); // 4/2 = 2 descendants
        assert_eq!(s.anc_count, Some(2)); // 4-2 = 2 ancestors
    }

    #[test]
    fn resolve_spec_both_odd() {
        let s = resolve_spec(Some("..@.."), Some(4), "@", None, "@");
        assert_eq!(s.rev, "@");
        assert_eq!(s.desc_count, Some(1)); // 3/2 = 1 descendant
        assert_eq!(s.anc_count, Some(2)); // 3-1 = 2 ancestors (extra goes to ancestors)
    }

    #[test]
    fn resolve_spec_flag_overrides_positional() {
        let s = resolve_spec(Some("@.."), Some(5), "@-", None, "@");
        assert_eq!(s.rev, "@-"); // flag_rev takes precedence
        assert_eq!(s.desc_count, Some(0));
        assert_eq!(s.anc_count, Some(4)); // pos_count 5 - 1
    }

    #[test]
    fn resolve_spec_ancestors_no_count() {
        let s = resolve_spec(Some("@.."), None, "@", None, "@");
        assert_eq!(s.rev, "@");
        assert_eq!(s.desc_count, Some(0));
        assert_eq!(s.anc_count, None); // unlimited
    }

    #[test]
    fn resolve_spec_flag_limit_overrides_pos_count() {
        let s = resolve_spec(Some("@.."), Some(5), "@", Some(3), "@");
        assert_eq!(s.rev, "@");
        assert_eq!(s.desc_count, Some(0));
        assert_eq!(s.anc_count, Some(2)); // flag 3 - 1 = 2
    }

    #[test]
    fn indent_body_zero() {
        let s = "first\nsecond\nthird";
        assert_eq!(indent_body(s, 0), s);
    }

    #[test]
    fn indent_body_single_line() {
        assert_eq!(indent_body("only", 3), "only");
    }

    #[test]
    fn indent_body_multi_line() {
        let s = "first\nsecond\nthird";
        assert_eq!(indent_body(s, 3), "first\n   second\n   third");
    }

    #[test]
    fn indent_body_empty_lines_preserved() {
        let s = "first\n\nthird";
        assert_eq!(indent_body(s, 3), "first\n\n   third");
    }

    #[test]
    fn indent_body_empty_string() {
        assert_eq!(indent_body("", 3), "");
    }

    /// Build a unique tempdir for the workspace-helper tests.
    fn ws_tempdir(tag: &str) -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("vc-x1-ws-{tag}-{ts}"));
        std::fs::create_dir_all(&dir).expect("mkdir tempdir");
        dir
    }

    /// Workspace root walk finds the dir whose `.vc-config.toml`
    /// has `path = "/"`, even when starting from a deep subdir.
    #[test]
    fn find_workspace_root_walks_up() {
        let base = ws_tempdir("walk-up");
        let root = base.join("ws");
        let nested = root.join("a").join("b").join("c");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(
            root.join(VC_CONFIG_FILE),
            "[workspace]\npath = \"/\"\nother-repo = \".claude\"\n",
        )
        .unwrap();
        assert_eq!(find_workspace_root_from(&nested).as_deref(), Some(&*root));
        std::fs::remove_dir_all(&base).ok();
    }

    /// Walking from a directory with no enclosing workspace yields None.
    #[test]
    fn find_workspace_root_none_outside() {
        let base = ws_tempdir("none-outside");
        let nested = base.join("nested");
        std::fs::create_dir_all(&nested).unwrap();
        assert!(find_workspace_root_from(&nested).is_none());
        std::fs::remove_dir_all(&base).ok();
    }

    /// `path` set to something other than `/` keeps walking — only the
    /// `path = "/"` config marks the workspace root.
    #[test]
    fn find_workspace_root_skips_non_root_path() {
        let base = ws_tempdir("skip-non-root");
        let root = base.join("ws");
        let claude = root.join(".claude");
        std::fs::create_dir_all(&claude).unwrap();
        std::fs::write(
            root.join(VC_CONFIG_FILE),
            "[workspace]\npath = \"/\"\nother-repo = \".claude\"\n",
        )
        .unwrap();
        std::fs::write(
            claude.join(VC_CONFIG_FILE),
            "[workspace]\npath = \"/.claude\"\nother-repo = \"..\"\n",
        )
        .unwrap();
        // From inside .claude, the root walker still resolves to the
        // app root (path = "/"), not to .claude itself.
        assert_eq!(find_workspace_root_from(&claude).as_deref(), Some(&*root));
        std::fs::remove_dir_all(&base).ok();
    }

    /// Default scope: workspace with non-empty `other-repo` → dual.
    #[test]
    fn default_scope_dual_workspace() {
        let base = ws_tempdir("default-dual");
        let root = base.join("ws");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join(VC_CONFIG_FILE),
            "[workspace]\npath = \"/\"\nother-repo = \".claude\"\n",
        )
        .unwrap();
        assert_eq!(
            default_scope(Some(&root)),
            Scope::Roles(vec![Side::Code, Side::Bot])
        );
        std::fs::remove_dir_all(&base).ok();
    }

    /// Default scope: workspace with no `other-repo` → code-only.
    #[test]
    fn default_scope_single_repo_workspace() {
        let base = ws_tempdir("default-single");
        let root = base.join("ws");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(VC_CONFIG_FILE), "[workspace]\npath = \"/\"\n").unwrap();
        assert_eq!(default_scope(Some(&root)), Scope::Roles(vec![Side::Code]));
        std::fs::remove_dir_all(&base).ok();
    }

    /// Default scope: empty `other-repo` value treated like missing.
    #[test]
    fn default_scope_empty_other_repo() {
        let base = ws_tempdir("default-empty");
        let root = base.join("ws");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join(VC_CONFIG_FILE),
            "[workspace]\npath = \"/\"\nother-repo = \"\"\n",
        )
        .unwrap();
        assert_eq!(default_scope(Some(&root)), Scope::Roles(vec![Side::Code]));
        std::fs::remove_dir_all(&base).ok();
    }

    /// Default scope: POR (None workspace_root) → code-only.
    #[test]
    fn default_scope_por() {
        assert_eq!(default_scope(None), Scope::Roles(vec![Side::Code]));
    }

    /// `scope_to_repos`: dual workspace resolves to root + root/other-repo.
    #[test]
    fn scope_to_repos_dual() {
        let base = ws_tempdir("repos-dual");
        let root = base.join("ws");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join(VC_CONFIG_FILE),
            "[workspace]\npath = \"/\"\nother-repo = \".claude\"\n",
        )
        .unwrap();
        let repos =
            scope_to_repos(&Scope::Roles(vec![Side::Code, Side::Bot]), Some(&root)).unwrap();
        assert_eq!(repos, vec![root.clone(), root.join(".claude")]);
        std::fs::remove_dir_all(&base).ok();
    }

    /// `scope_to_repos`: code-only inside a workspace yields just root.
    #[test]
    fn scope_to_repos_code_only() {
        let base = ws_tempdir("repos-code");
        let root = base.join("ws");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join(VC_CONFIG_FILE),
            "[workspace]\npath = \"/\"\nother-repo = \".claude\"\n",
        )
        .unwrap();
        let repos = scope_to_repos(&Scope::Roles(vec![Side::Code]), Some(&root)).unwrap();
        assert_eq!(repos, vec![root.clone()]);
        std::fs::remove_dir_all(&base).ok();
    }

    /// `scope_to_repos`: code-only with POR → cwd `.`.
    #[test]
    fn scope_to_repos_code_por() {
        let repos = scope_to_repos(&Scope::Roles(vec![Side::Code]), None).unwrap();
        assert_eq!(repos, vec![PathBuf::from(".")]);
    }

    /// `scope_to_repos`: bot in POR errors with the documented message.
    #[test]
    fn scope_to_repos_bot_por_errors() {
        let err = scope_to_repos(&Scope::Roles(vec![Side::Bot]), None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("not in a vc-x1 workspace"), "got: {err}");
    }

    /// `scope_to_repos`: bot in single-repo workspace errors.
    #[test]
    fn scope_to_repos_bot_single_repo_errors() {
        let base = ws_tempdir("repos-bot-single");
        let root = base.join("ws");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(VC_CONFIG_FILE), "[workspace]\npath = \"/\"\n").unwrap();
        let err = scope_to_repos(&Scope::Roles(vec![Side::Bot]), Some(&root))
            .unwrap_err()
            .to_string();
        assert!(err.contains("no other-repo configured"), "got: {err}");
        std::fs::remove_dir_all(&base).ok();
    }

    /// `scope_to_repos`: `Single(_)` errors until 0.42.0-3 lands its
    /// handling. Locks the staging contract so the migration step is
    /// noticed if it regresses.
    #[test]
    fn scope_to_repos_single_errors_until_0420_3() {
        let err = scope_to_repos(&Scope::Single(PathBuf::from(".")), None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("0.42.0-3"), "got: {err}");
    }
}

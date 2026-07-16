use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::desc_helpers::VC_CONFIG_FILE;
use crate::options_flags::common_args::CommonArgs;
use crate::options_flags::scope::{Scope, Side};
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

/// Run a closure once per resolved repo, with optional header decoration between repos.
///
/// Takes an already-resolved `&[PathBuf]` — the caller is responsible
/// for any flag-driven expansion (see `resolve_repos`). Continues past
/// errors, printing them to stderr, and returns an error if any
/// repo's body failed.
pub fn for_each_repo<F>(
    repos: &[PathBuf],
    header: &Header,
    mut body: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(&Workspace, &Arc<ReadonlyRepo>) -> Result<(), Box<dyn std::error::Error>>,
{
    let multi = repos.len() > 1;
    let mut first = true;
    let mut errors: Vec<String> = Vec::new();

    for repo_path in repos {
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
    for result in revset.commit_change_ids() {
        let (commit_id, _change_id) = result?;
        commit_ids.push(commit_id);
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
/// commands (sync, push, squash-push) and as a post-condition assertion by setup
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

/// At-rest publish state of a bookmark vs its `@origin` counterpart.
#[derive(Debug, PartialEq, Eq)]
pub enum PublishState {
    /// Local bookmark and `bookmark@origin` point at the same commit.
    InSync,
    /// `bookmark@origin` does not exist — never pushed.
    NeverPushed,
    /// Local and origin point at different commits (full commit ids).
    Mismatch { local: String, remote: String },
}

/// Resolve the at-rest publish state of `bookmark` in `repo`.
///
/// The bookmark only moves inside a push / squash-push run, which
/// publishes it in the same invocation — so between commands
/// `bookmark == bookmark@origin` is the invariant. A mismatch means
/// an earlier publish was lost (or origin moved out from under us).
///
/// - Read-only: two `jj log` commit-id lookups, nothing else.
/// - Errors when the local bookmark itself does not resolve.
pub fn bookmark_publish_state(
    repo: &Path,
    bookmark: &str,
) -> Result<PublishState, Box<dyn std::error::Error>> {
    let repo_str = repo.to_string_lossy();
    let cwd = Path::new(".");
    let commit_id = |rev: &str| -> Result<String, Box<dyn std::error::Error>> {
        Ok(run(
            "jj",
            &[
                "log",
                "-r",
                rev,
                "--no-graph",
                "-T",
                "commit_id",
                "-R",
                &repo_str,
            ],
            cwd,
        )?
        .trim()
        .to_string())
    };
    let local = commit_id(bookmark)
        .map_err(|e| format!("bookmark '{bookmark}' does not resolve in '{repo_str}': {e}"))?;
    // `present(...)` maps a nonexistent remote bookmark to an empty
    // result instead of a revset error.
    let remote = commit_id(&format!("present({bookmark}@origin)"))?;
    Ok(if remote.is_empty() {
        PublishState::NeverPushed
    } else if local == remote {
        PublishState::InSync
    } else {
        PublishState::Mismatch { local, remote }
    })
}

/// Verify the bot repo's `bookmark` is published at origin.
///
/// Error-raising wrapper over [`bookmark_publish_state`], used by
/// `validate-bot` and push preflight. Fixes nothing (decided
/// 2026-07-15: no automatic fixing) — the error names the
/// `vc-x1 squash-push` resolution and the caller stops.
pub fn verify_bot_published(
    bot_repo: &Path,
    bookmark: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_str = bot_repo.to_string_lossy();
    match bookmark_publish_state(bot_repo, bookmark)? {
        PublishState::InSync => Ok(()),
        PublishState::NeverPushed => Err(format!(
            "bot repo '{repo_str}': bookmark '{bookmark}' has never been pushed to origin — \
             publish it with `vc-x1 squash-push -R {repo_str}`"
        )
        .into()),
        PublishState::Mismatch { local, remote } => Err(format!(
            "bot repo '{repo_str}': '{bookmark}' ({}) does not match '{bookmark}@origin' ({}) — \
             an earlier publish was likely lost; publish it with \
             `vc-x1 squash-push -R {repo_str}`, then retry",
            &local[..local.len().min(12)],
            &remote[..remote.len().min(12)]
        )
        .into()),
    }
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
/// - dual workspace (non-empty `other-repo`) → `Scope([Code, Bot])`
/// - single-repo workspace (missing / empty `other-repo`) →
///   `Scope([Code])`
/// - POR (no `workspace_root`) → `Scope([Code])` — `scope_to_repos`
///   resolves `Side::Code` to cwd's `.` when no root is given, so a
///   POR run still operates on the current directory.
pub fn default_scope(workspace_root: Option<&Path>) -> Scope {
    let Some(root) = workspace_root else {
        return Scope(vec![Side::Code]);
    };
    let cfg = match toml_simple::toml_load(&root.join(VC_CONFIG_FILE)) {
        Ok(c) => c,
        Err(_) => return Scope(vec![Side::Code]),
    };
    match toml_simple::toml_get(&cfg, "workspace.other-repo") {
        Some(v) if !v.is_empty() => Scope(vec![Side::Code, Side::Bot]),
        _ => Scope(vec![Side::Code]),
    }
}

/// Resolve a `Scope` to concrete repo paths.
///
/// - `Side::Code` → `workspace_root` (or cwd's `.` when
///   `workspace_root` is None).
/// - `Side::Bot` → `workspace_root.join(other-repo)`.
///
/// Errors when `Side::Bot` is requested but the workspace doesn't
/// define one (POR or single-repo workspace).
pub fn scope_to_repos(
    scope: &Scope,
    workspace_root: Option<&Path>,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut repos = Vec::new();
    for side in &scope.0 {
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

/// Resolve `CommonArgs.repo` + `CommonArgs.scope` into concrete repo paths.
///
/// The four `CommonArgs` consumers (`chid` / `desc` / `list` / `show`)
/// call this at the start of their body to translate the clap surface
/// into the `&[PathBuf]` that `for_each_repo` iterates. Behavior matches
/// today's defaults plus a composing rule for the new `--scope` flag:
///
/// - neither flag → `[.]` (today's no-arg default).
/// - `-R PATH` alone → `[PATH]` (today's `-R <path>` behavior;
///   workspace context not consulted).
/// - `-s ROLES` alone → resolves against `find_workspace_root()` —
///   the surrounding workspace if there is one, else POR
///   (`Side::Code` → `.`, `Side::Bot` → error).
/// - `-R PATH -s ROLES` → resolves with `PATH` as the workspace root
///   (overrides `find_workspace_root`). This is the composing case;
///   e.g. `chid -R ../foo -s bot` queries `../foo/.claude`.
///
/// `--scope` is keyword-only (`code|bot|code,bot|bot,code`);
/// path-based single-repo operation routes through `-R/--repo`.
pub fn resolve_repos(
    repo: Option<&Path>,
    scope: Option<&Scope>,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    match (repo, scope) {
        (None, None) => Ok(vec![PathBuf::from(".")]),
        (Some(p), None) => Ok(vec![p.to_path_buf()]),
        (Some(p), Some(s)) => scope_to_repos(s, Some(p)),
        (None, Some(s)) => scope_to_repos(s, find_workspace_root().as_deref()),
    }
}

/// Resolved shared params for the read-only commit-query subcommands
/// (`chid` / `desc` / `list` / `show`). Built once at the binary edge
/// from `CommonArgs` via `TryFrom`; each subcommand's `XxxParams`
/// embeds this.
///
/// - `spec` — resolved `DotSpec` (parsed `..` notation + per-side
///   count budget).
/// - `header` — resolved inter-repo `Header` (label vs none).
/// - `repos` — resolved repo paths (`-R` + `-s` → `Vec<PathBuf>`).
#[derive(Debug)]
pub struct CommonParams {
    pub spec: DotSpec,
    pub header: Header,
    pub repos: Vec<PathBuf>,
}

impl TryFrom<&CommonArgs> for CommonParams {
    type Error = String;

    /// Resolve clap `CommonArgs` into clap-free `CommonParams`: run
    /// `resolve_spec` / `resolve_header` / `resolve_repos` at the
    /// binary edge. Fallible because `resolve_repos` can fail
    /// (workspace lookup, path issues).
    fn try_from(a: &CommonArgs) -> Result<Self, String> {
        let spec = resolve_spec(a.pos_rev.as_deref(), a.pos_count, &a.revision, a.limit, "@");
        let header = resolve_header(&a.label, a.no_label);
        let repos = a.resolve_repos().map_err(|e| e.to_string())?;
        Ok(CommonParams {
            spec,
            header,
            repos,
        })
    }
}

#[cfg(test)]
mod tests;

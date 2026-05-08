mod params;
pub use params::InitParams;

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info};

use crate::config::{self, UserConfig};
use crate::context::Context;
use crate::options_flags::account::AccountOption;
use crate::options_flags::config::{ConfigKind, ConfigOption};
use crate::options_flags::provision_bundle::ProvisionOptionFlagBundle;
use crate::options_flags::push_retry::PushRetryOptions;
use crate::options_flags::repo::RepoOption;
use crate::options_flags::scope::{ScopeKind, ScopeOption};
use crate::options_flags::use_template::UseTemplateOption;
use crate::repo_utils::{OchidStrategy, commit_initial, cross_ref_ochids, prepare_local_repo};
use crate::scope::{Scope, Side};
use crate::symlink;
use crate::url::{Target, derive_name, derive_session_url, parse_target};

/// CLI args for `vc-x1 init`.
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Target — URL, owner/name shorthand, path, or bare NAME.
    ///
    /// - URL: `git@host:owner/name(.git)?`, `https://...(.git)?`
    ///   — used as-is; config not consulted.
    /// - owner/name shorthand: resolves to
    ///   `git@github.com:owner/name.git`; config not consulted.
    /// - Path: `./X`, `../X`, `/X`, `~/X`, `~`, `.`, `..` — is the
    ///   directory path; remote resolved via `--repo` chain.
    /// - Bare NAME: becomes NAME.git; remote resolved via
    ///   `--repo` chain.
    #[arg(value_name = "TARGET", verbatim_doc_comment)]
    pub target: String,

    /// Repo directory name override (URL / owner/name forms only).
    ///
    /// - URL / owner/name forms: repo created at `cwd/<NAME>`
    ///   instead of the URL-derived name.
    /// - Path / bare-NAME forms: error if given (TARGET already
    ///   names the repo).
    #[arg(value_name = "NAME", verbatim_doc_comment)]
    pub name: Option<String>,

    /// `--account` — flatten of the shared [`AccountOption`] leaf.
    #[command(flatten)]
    pub account: AccountOption,

    /// `--repo` — flatten of the shared [`RepoOption`] leaf.
    /// Init's built-in categories: `remote` (URL prefix; init
    /// appends `/<NAME>.git`) and `local` (parent dir for
    /// fixture bare repos). Meaningful only with Path or
    /// bare-NAME targets.
    #[command(flatten)]
    pub repo: RepoOption,

    /// `--scope` — flatten of the shared [`ScopeOption`] leaf.
    #[command(flatten)]
    pub scope: ScopeOption,

    /// `--dry-run` + `--private` + `--push-retries` /
    /// `--push-retry-delay` — flatten of the shared
    /// [`ProvisionOptionFlagBundle`] bundle.
    #[command(flatten)]
    pub provision: ProvisionOptionFlagBundle,

    /// `--use-template` — flatten of the shared
    /// [`UseTemplateOption`] leaf.
    #[command(flatten)]
    pub use_template: UseTemplateOption,

    /// `--config none|<path>` — flatten of the shared
    /// [`ConfigOption`] leaf. Only meaningful with `--scope=por`;
    /// rejected at preflight when paired with `--scope=code,bot`.
    /// `.gitignore` is always written regardless of `--config`.
    #[command(flatten)]
    pub config: ConfigOption,
}

/// Run a command with retries, sleeping between attempts.
fn run_retry(
    cmd: &str,
    args: &[&str],
    cwd: &Path,
    retry: &PushRetryOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut last_err = String::new();
    for attempt in 1..=retry.push_retries {
        match run(cmd, args, cwd) {
            Ok(out) => {
                if attempt > 1 {
                    debug!("succeeded after {attempt} attempts");
                }
                return Ok(out);
            }
            Err(e) => {
                last_err = e.to_string();
                if attempt < retry.push_retries {
                    debug!(
                        "attempt {attempt}/{} failed: {last_err}",
                        retry.push_retries
                    );
                    debug!("retrying in {}s...", retry.push_retry_delay);
                    std::thread::sleep(std::time::Duration::from_secs(retry.push_retry_delay));
                }
            }
        }
    }
    Err(format!("failed after {} attempts: {last_err}", retry.push_retries).into())
}

use crate::common::{mkdir_p, run, write_file};

/// Parse the `--use-template` value into `(code, bot)` template paths.
///
/// Format: `CODE[,BOT]`. If `BOT` is omitted, the default is the sibling
/// directory whose name is `<CODE-basename>.claude` (via
/// `Path::with_file_name`, so a trailing slash on `CODE` does not produce
/// a different result).
pub(crate) fn parse_use_template(
    s: &str,
) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let mut parts = s.splitn(2, ',');
    let code_raw = parts.next().unwrap_or(""); // OK: splitn always yields at least one element
    let bot_raw = parts.next();
    let code_trim = code_raw.trim();
    if code_trim.is_empty() {
        return Err("--use-template: code template path is empty".into());
    }
    let code = PathBuf::from(code_trim);
    let bot = match bot_raw.map(str::trim) {
        Some(b) if !b.is_empty() => PathBuf::from(b),
        _ => {
            let file_name = code.file_name().ok_or_else(|| {
                format!(
                    "--use-template: cannot derive default bot path from '{}' (no file name component)",
                    code.display()
                )
            })?;
            let new_name = format!("{}.claude", file_name.to_string_lossy());
            code.with_file_name(new_name)
        }
    };
    Ok((code, bot))
}

/// Top-level non-hidden files init writes. Kept here so that if init is
/// ever extended to write non-hidden top-level files, the pre-flight
/// conflict scan flags any template that would clash. Currently empty
/// because init only writes hidden files (`.vc-config.toml`, `.gitignore`),
/// and the template copy skips hidden entries.
const RESERVED_TEMPLATE_ENTRIES: &[&str] = &[];

/// Validate one template directory: exists, is a directory, and has no
/// top-level non-hidden entry that would collide with init's own writes.
///
/// - `label` prefixes every error message (`code` / `bot`).
pub(crate) fn validate_template_one(
    label: &str,
    p: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !p.exists() {
        return Err(format!(
            "--use-template: {label} template '{}' does not exist",
            p.display()
        )
        .into());
    }
    if !p.is_dir() {
        return Err(format!(
            "--use-template: {label} template '{}' is not a directory",
            p.display()
        )
        .into());
    }
    for entry in std::fs::read_dir(p)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.') {
            continue;
        }
        if RESERVED_TEMPLATE_ENTRIES.contains(&name_str.as_ref()) {
            return Err(format!(
                "--use-template: {label} template '{}' contains reserved entry '{}'",
                p.display(),
                name_str
            )
            .into());
        }
    }
    Ok(())
}

/// Validate both template paths (dual-repo mode). Thin wrapper over
/// `validate_template_one` kept for single-call ergonomics.
pub(crate) fn validate_templates(
    code: &Path,
    bot: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    validate_template_one("code", code)?;
    validate_template_one("bot", bot)?;
    Ok(())
}

/// Recursively copy non-hidden entries from `src` to `dst`. Any entry whose
/// file name starts with `.` is skipped. Symlinks are skipped with a debug
/// log — templates don't need them, and following them risks escaping the
/// template tree.
pub(crate) fn copy_template_recursive(
    src: &Path,
    dst: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.') {
            continue;
        }
        let src_path = entry.path();
        let dst_path = dst.join(&name);
        let ft = entry.file_type()?;
        if ft.is_symlink() {
            debug!("skip symlink {}", src_path.display());
            continue;
        }
        if ft.is_dir() {
            mkdir_p(&dst_path)?;
            copy_template_recursive(&src_path, &dst_path)?;
        } else if ft.is_file() {
            debug!("copy {} -> {}", src_path.display(), dst_path.display());
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Replace the first line of `<dir>/README.md` with `# <name>`. If
/// `README.md` is absent, this is a no-op. Trailing content after the
/// first newline is preserved verbatim; a file with no newline becomes
/// just `# <name>`.
pub(crate) fn rewrite_readme_first_line(
    dir: &Path,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let readme = dir.join("README.md");
    if !readme.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&readme)?;
    let rest = match content.find('\n') {
        Some(pos) => &content[pos..],
        None => "",
    };
    let new_content = format!("# {name}{rest}");
    std::fs::write(&readme, new_content)?;
    debug!("rewrote first line of {}", readme.display());
    Ok(())
}

/// Get the short (12-char) jj change ID for a revision, without printing.
pub(crate) fn jj_chid(rev: &str, cwd: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let full = run(
        "jj",
        &[
            "log",
            "-r",
            rev,
            "--no-graph",
            "-T",
            "change_id",
            "--limit",
            "1",
        ],
        cwd,
    )?;
    Ok(full[..full.len().min(12)].to_string())
}

/// Check if a GitHub repo exists.
fn gh_repo_exists(owner: &str, name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let full = format!("{owner}/{name}");
    Ok(run("gh", &["repo", "view", &full], Path::new(".")).is_ok())
}

pub(crate) const VC_CONFIG_CODE: &str = r#"# vc-config: Vibe Coding workspace configuration
#
# workspace-path is this repo's path relative to the workspace root.
# Used to resolve changeID paths in git trailers (e.g. ochid: /changeID).
# other-repo is the relative path to the counterpart repo.

[workspace]
path = "/"
other-repo = ".claude"
"#;

const VC_CONFIG_SESSION: &str = r#"# vc-config: Vibe Coding workspace configuration
#
# workspace-path is this repo's path relative to the workspace root.
# Used to resolve changeID paths in git trailers (e.g. ochid: /.claude/changeID).
# other-repo is the relative path to the counterpart repo.

[workspace]
path = "/.claude"
other-repo = ".."
"#;

pub(crate) const GITIGNORE_CODE: &str = "/target
/.claude
/.git
/.jj
/.vc-x1
";

const GITIGNORE_SESSION: &str = ".git
.jj
";

/// Config for a single-repo workspace (`--scope=code`).
///
/// - No `other-repo` field — its absence is what downstream
///   commands use to detect the single-repo state.
pub(crate) const VC_CONFIG_APP_ONLY: &str = r#"# vc-config: Vibe Coding workspace configuration
#
# workspace-path is this repo's path relative to the workspace root.
# Used to resolve changeID paths in git trailers (e.g. ochid: /changeID).

[workspace]
path = "/"
"#;

/// Gitignore for a single-repo workspace.
///
/// - Same as `GITIGNORE_CODE` minus the `/.claude` entry — there's
///   no session subdir in single-repo mode.
pub(crate) const GITIGNORE_APP_ONLY: &str = "/target
/.git
/.jj
/.vc-x1
";

/// Write the POR (single-repo) canned `.vc-config.toml` into `dir`.
///
/// Split from the legacy `write_por_config` so the `.vc-config.toml`
/// write is gateable by `--config` while the `.gitignore` write
/// stays unconditional.
fn write_por_vc_config(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_file(&dir.join(".vc-config.toml"), VC_CONFIG_APP_ONLY)
}

/// Write the POR (single-repo) `.gitignore` into `dir`. Always
/// unconditional — `--config` controls only `.vc-config.toml`.
fn write_por_gitignore(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_file(&dir.join(".gitignore"), GITIGNORE_APP_ONLY)
}

/// Copy a user-supplied `.vc-config.toml` from `src` to `dir`.
///
/// Bytewise copy — no TOML parse, no schema validation. If the
/// content is malformed the project will surface the problem on
/// first `find_workspace_root` / config-reader call. Caller is
/// responsible for path-existence preflight.
fn copy_user_config(src: &Path, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::copy(src, dir.join(".vc-config.toml"))
        .map_err(|e| format!("--config: failed to copy {}: {e}", src.display()))?;
    Ok(())
}

/// Write the dual-mode code-side `.vc-config.toml` and `.gitignore`
/// into `dir`. Used by `create_dual` for the code repo.
fn write_code_config(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_file(&dir.join(".vc-config.toml"), VC_CONFIG_CODE)?;
    write_file(&dir.join(".gitignore"), GITIGNORE_CODE)?;
    Ok(())
}

/// Write the dual-mode session-side `.vc-config.toml` and
/// `.gitignore` into `dir`. Used by `create_dual` for the bot repo.
fn write_session_config(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_file(&dir.join(".vc-config.toml"), VC_CONFIG_SESSION)?;
    write_file(&dir.join(".gitignore"), GITIGNORE_SESSION)?;
    Ok(())
}

/// Expand `~` and `$VAR` / `${VAR}` substitutions in a user string.
///
/// - `~` and `~/…` resolve via `HOME`.
/// - Unset env vars are a fatal error (silent empty substitution
///   would mask typos).
/// - Bare `$` with no identifier after stays literal.
pub(crate) fn expand_vars(s: &str) -> Result<String, Box<dyn std::error::Error>> {
    let home_expanded = if s == "~" {
        std::env::var("HOME").map_err(|_| "HOME is not set; cannot expand '~'")?
    } else if let Some(rest) = s.strip_prefix("~/") {
        let home = std::env::var("HOME").map_err(|_| "HOME is not set; cannot expand '~/'")?;
        format!("{home}/{rest}")
    } else {
        s.to_string()
    };

    let mut out = String::with_capacity(home_expanded.len());
    let mut chars = home_expanded.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '$' {
            out.push(c);
            continue;
        }
        let (name, braced) = if chars.peek() == Some(&'{') {
            chars.next(); // OK: peeked '{' above, next() must yield it
            let mut name = String::new();
            let mut closed = false;
            for c2 in chars.by_ref() {
                if c2 == '}' {
                    closed = true;
                    break;
                }
                name.push(c2);
            }
            if !closed {
                return Err(format!("unterminated '${{…}}' in '{s}'").into());
            }
            (name, true)
        } else {
            let mut name = String::new();
            while let Some(&c2) = chars.peek() {
                if c2.is_ascii_alphanumeric() || c2 == '_' {
                    name.push(c2);
                    chars.next();
                } else {
                    break;
                }
            }
            (name, false)
        };
        if name.is_empty() {
            out.push('$');
            if braced {
                out.push_str("{}");
            }
            continue;
        }
        let val =
            std::env::var(&name).map_err(|_| format!("env var '${name}' is not set (in '{s}')"))?;
        out.push_str(&val);
    }
    Ok(out)
}

/// True when `s` looks like a git remote URL rather than a path.
///
/// - Scheme-qualified (`scheme://…`).
/// - scp-like SSH form (`user@host:path`).
/// - Everything else is treated as a path.
pub(crate) fn is_remote_url(s: &str) -> bool {
    if s.contains("://") {
        return true;
    }
    if let Some(at) = s.find('@')
        && let Some(colon) = s.find(':')
        && at < colon
        && !s[..at].contains('/')
    {
        return true;
    }
    false
}

/// True when `s` is a GitHub URL (host-based detection).
///
/// - SSH scp-like (`git@github.com:…`).
/// - `ssh://git@github.com/…`.
/// - `https://github.com/…` and `http://github.com/…`.
/// - GitHub URLs route to the `gh repo create` provisioner.
pub(crate) fn is_github_url(s: &str) -> bool {
    s.starts_with("git@github.com:")
        || s.starts_with("ssh://git@github.com/")
        || s.starts_with("https://github.com/")
        || s.starts_with("http://github.com/")
}

/// Normalize a `--repo local=<parent>` value: expand variables,
/// make absolute. Parent doesn't need to exist yet — init creates
/// it.
pub(crate) fn normalize_local_parent(s: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let s = s.trim();
    if s.is_empty() {
        return Err("--repo local: parent dir is empty".into());
    }
    let expanded = expand_vars(s)?;
    let p = PathBuf::from(&expanded);
    if p.is_absolute() {
        Ok(p)
    } else {
        let cwd = std::env::current_dir()?;
        Ok(cwd.join(p))
    }
}

/// Append `.git` to a URL-or-path stem if not already present.
///
/// - Lets `--repo-remote git@github.com:u/tf1` normalize to the
///   same URL the default-mode derivation produces.
pub(crate) fn ensure_git_suffix(s: &str) -> String {
    if s.ends_with(".git") {
        s.to_string()
    } else {
        format!("{s}.git")
    }
}

/// How init should provision the remote repositories it pushes to.
/// See `notes/chores-06.md > Generalize --scope across commands
/// (0.40.0) > 0.40.0-1` for the dispatch rules.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Provisioner {
    /// `git init --bare` on local paths under `--repo-local`.
    LocalBareInit,
    /// `gh repo create` for GitHub URLs (default mode or
    /// `--repo-remote <github-url>`).
    GhCreate,
    /// Skip any create step; the caller pre-created the remote
    /// (non-GitHub URLs under `--repo-remote`).
    ExternalPreExisting,
}

/// Fully-resolved inputs to the init execution phase.
///
/// - Dispatch and ambiguity rules live in `plan_init`.
/// - Downstream code operates on an `InitPlan` — execution stays
///   linear.
#[derive(Debug)]
pub(crate) struct InitPlan {
    /// Which side(s) this plan creates. `scope.is_code_only()` means
    /// single-repo; `scope.is_both()` means dual.
    pub scope: Scope,
    pub project_dir: PathBuf,
    pub name: String,
    pub code_url: String,
    pub provisioner: Provisioner,
    /// Set only when `provisioner == LocalBareInit`.
    pub code_bare_path: Option<PathBuf>,
    /// GitHub `owner/name` for the code side; only populated for
    /// the `GhCreate` path (`gh repo create` needs it).
    pub gh_code_slug: Option<String>,
    /// Session-side dir. None when `scope.is_code_only()`.
    pub session_dir: Option<PathBuf>,
    /// Session-side project name (e.g. `<name>.claude`). None when
    /// `scope.is_code_only()`.
    pub session_name: Option<String>,
    /// Session-side origin URL. None when `scope.is_code_only()`.
    pub session_url: Option<String>,
    /// Session-side bare-repo path. `Some` only under `LocalBareInit`
    /// with `scope.is_both()`.
    pub session_bare_path: Option<PathBuf>,
    /// Session-side GitHub slug. `Some` only under `GhCreate` with
    /// `scope.is_both()`.
    pub gh_session_slug: Option<String>,
}

/// Build an `InitPlan` from CLI args + user config.
///
/// Dispatches on the parsed `<TARGET>` form:
///
/// - `Url(u)` / `OwnerName(o, n)` → URL is explicit; config not
///   consulted; `--account` and `--repo` are rejected as
///   meaningless.
/// - `Path(p)` → `p` is the destination; basename names the repo;
///   remote URL resolved from config via the `--repo` chain.
/// - `BareName(n)` → destination at `cwd/<n>`; remote resolved
///   from config.
///
/// The `cfg` parameter is the loaded user config; `init` loads it
/// once at entry and passes it through so tests can supply a
/// synthetic config without touching disk.
pub(crate) fn plan_init(
    params: &InitParams,
    cfg: &UserConfig,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    debug!(
        "init args: target={:?}, name={:?}, account={:?}, repo={:?}, scope={:?}, private={}",
        params.target, params.name, params.account, params.repo, params.scope, params.private
    );

    let scope = match params.scope {
        ScopeKind::CodeBot => Scope::Roles(vec![Side::Code, Side::Bot]),
        ScopeKind::Por => Scope::Roles(vec![Side::Code]),
    };

    if scope.is_code_only()
        && let Some(t) = &params.use_template
        && t.contains(',')
    {
        return Err(format!(
            "--scope=por takes a single template path; got '{t}' (drop the `,BOT` half)"
        )
        .into());
    }

    if params.config.is_some() && params.scope == ScopeKind::CodeBot {
        return Err(
            "--config is only valid with --scope=por (dual-mode configs are per-side and unconditional)".into(),
        );
    }
    if let Some(ConfigKind::Path(p)) = &params.config
        && !p.exists()
    {
        return Err(format!("--config: path does not exist: {}", p.display()).into());
    }

    let parsed = parse_target(&params.target)?;
    debug!("parse_target: {:?} → {:?}", params.target, parsed);
    let plan = match parsed {
        Target::Url(url) => plan_from_url(params, scope, url),
        Target::OwnerName(o, n) => {
            plan_from_url(params, scope, format!("git@github.com:{o}/{n}.git"))
        }
        Target::Path(p) => plan_from_path(params, scope, p, cfg),
        Target::BareName(n) => plan_from_bare_name(params, scope, n, cfg),
    }?;
    debug!(
        "plan_init: project_dir={}, name={}, code_url={}, provisioner={:?}, gh_code_slug={:?}, code_bare_path={:?}, session_url={:?}",
        plan.project_dir.display(),
        plan.name,
        plan.code_url,
        plan.provisioner,
        plan.gh_code_slug,
        plan.code_bare_path,
        plan.session_url,
    );
    Ok(plan)
}

/// Plan when TARGET is a URL or `owner/name` shorthand.
///
/// - URL is explicit; config not consulted.
/// - `--account` / `--repo` are rejected (would have no effect).
/// - `[NAME]` overrides the URL-derived directory name.
fn plan_from_url(
    params: &InitParams,
    scope: Scope,
    url: String,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    if params.account.is_some() {
        return Err(
            "--account is meaningless with a URL or owner/name TARGET (config not consulted)"
                .into(),
        );
    }
    if params.repo.is_some() {
        return Err(
            "--repo is meaningless with a URL or owner/name TARGET (config not consulted)".into(),
        );
    }
    let code_url = ensure_git_suffix(&url);
    let derived_name = derive_name(&code_url)?;
    let name = params.name.clone().unwrap_or(derived_name);
    let cwd = std::env::current_dir()?;
    let project_dir = cwd.join(&name);

    let provisioner = if is_github_url(&code_url) {
        Provisioner::GhCreate
    } else {
        Provisioner::ExternalPreExisting
    };
    let gh_code_slug = if provisioner == Provisioner::GhCreate {
        Some(github_slug_from_url(&code_url)?)
    } else {
        None
    };

    build_plan(
        scope,
        name,
        project_dir,
        code_url,
        provisioner,
        gh_code_slug,
    )
}

/// Plan when TARGET is a path. Path IS the destination; basename
/// names the repo. Remote resolved via the `--repo` chain.
fn plan_from_path(
    params: &InitParams,
    scope: Scope,
    path: PathBuf,
    cfg: &UserConfig,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    if params.name.is_some() {
        return Err(
            "[NAME] is meaningless with a path TARGET (path already names the destination)".into(),
        );
    }
    let project_dir = resolve_path_target(&path)?;
    let name = project_dir
        .file_name()
        .ok_or_else(|| {
            format!(
                "path TARGET '{}' has no last component; cannot derive a repo name",
                path.display()
            )
        })?
        .to_str()
        .ok_or("path TARGET is not valid UTF-8")?
        .to_string();
    let (cat, val) = config::resolve_repo(cfg, params.account.as_deref(), params.repo.as_ref())?;
    plan_from_resolved(scope, name, project_dir, &cat, &val)
}

/// Plan when TARGET is a bare alphanumeric NAME. Destination at
/// `cwd/<NAME>`; remote resolved via the `--repo` chain.
fn plan_from_bare_name(
    params: &InitParams,
    scope: Scope,
    name: String,
    cfg: &UserConfig,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    if params.name.is_some() {
        return Err(
            "[NAME] is meaningless with a bare-NAME TARGET (TARGET already names the repo)".into(),
        );
    }
    let cwd = std::env::current_dir()?;
    let project_dir = cwd.join(&name);
    let (cat, val) = config::resolve_repo(cfg, params.account.as_deref(), params.repo.as_ref())?;
    plan_from_resolved(scope, name, project_dir, &cat, &val)
}

/// Build an `InitPlan` for a Path or BareName target after the
/// `--repo` chain has resolved to `(category, value)`. Dispatches
/// to `plan_remote` (URL prefix) or `plan_local` (fixture parent).
fn plan_from_resolved(
    scope: Scope,
    name: String,
    project_dir: PathBuf,
    category: &str,
    value: &str,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    match category {
        "remote" => plan_remote(scope, name, project_dir, value),
        "local" => plan_local(scope, name, project_dir, value),
        other => Err(format!(
            "--repo category '{other}' is not recognized — built-ins are 'remote' and 'local'"
        )
        .into()),
    }
}

/// Plan for `category = "remote"`: URL prefix + `/<NAME>.git`.
fn plan_remote(
    scope: Scope,
    name: String,
    project_dir: PathBuf,
    url_prefix: &str,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    let prefix = url_prefix.trim_end_matches('/');
    let code_url = format!("{prefix}/{name}.git");
    let provisioner = if is_github_url(&code_url) {
        Provisioner::GhCreate
    } else {
        Provisioner::ExternalPreExisting
    };
    let gh_code_slug = if provisioner == Provisioner::GhCreate {
        Some(github_slug_from_url(&code_url)?)
    } else {
        None
    };
    build_plan(
        scope,
        name,
        project_dir,
        code_url,
        provisioner,
        gh_code_slug,
    )
}

/// Plan for `category = "local"`: bare repos under a parent dir.
///
/// - Dual: `<parent>/remote-code.git` + `<parent>/remote-claude.git`.
/// - POR: `<parent>/remote.git`.
fn plan_local(
    scope: Scope,
    name: String,
    project_dir: PathBuf,
    parent_spec: &str,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    let parent = normalize_local_parent(parent_spec)?;

    if scope.is_code_only() {
        let code_bare = parent.join("remote.git");
        return Ok(InitPlan {
            scope,
            code_url: code_bare.to_string_lossy().to_string(),
            code_bare_path: Some(code_bare),
            project_dir,
            name,
            provisioner: Provisioner::LocalBareInit,
            gh_code_slug: None,
            session_dir: None,
            session_name: None,
            session_url: None,
            session_bare_path: None,
            gh_session_slug: None,
        });
    }

    let session_dir = project_dir.join(".claude");
    let code_bare = parent.join("remote-code.git");
    let session_bare = parent.join("remote-claude.git");
    let session_name = format!("{name}.claude");
    Ok(InitPlan {
        scope,
        code_url: code_bare.to_string_lossy().to_string(),
        code_bare_path: Some(code_bare),
        project_dir,
        name,
        provisioner: Provisioner::LocalBareInit,
        gh_code_slug: None,
        session_url: Some(session_bare.to_string_lossy().to_string()),
        session_bare_path: Some(session_bare),
        session_dir: Some(session_dir),
        session_name: Some(session_name),
        gh_session_slug: None,
    })
}

/// Compose an `InitPlan` for the URL-style provisioners (`GhCreate`
/// and `ExternalPreExisting`) — both `plan_from_url` and
/// `plan_remote` share this tail because the only difference between
/// them is how they construct the URL.
fn build_plan(
    scope: Scope,
    name: String,
    project_dir: PathBuf,
    code_url: String,
    provisioner: Provisioner,
    gh_code_slug: Option<String>,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    if scope.is_code_only() {
        return Ok(InitPlan {
            scope,
            project_dir,
            name,
            code_url,
            provisioner,
            code_bare_path: None,
            gh_code_slug,
            session_dir: None,
            session_name: None,
            session_url: None,
            session_bare_path: None,
            gh_session_slug: None,
        });
    }
    let session_url = derive_session_url(&code_url);
    let session_name = format!("{name}.claude");
    let session_dir = project_dir.join(".claude");
    let gh_session_slug = if provisioner == Provisioner::GhCreate {
        Some(github_slug_from_url(&session_url)?)
    } else {
        None
    };
    Ok(InitPlan {
        scope,
        project_dir,
        name,
        code_url,
        provisioner,
        code_bare_path: None,
        gh_code_slug,
        session_url: Some(session_url),
        session_name: Some(session_name),
        session_dir: Some(session_dir),
        session_bare_path: None,
        gh_session_slug,
    })
}

/// Resolve a Path target to an absolute `PathBuf`. Handles `~`,
/// `~/X`, `$VAR`, `${VAR}` expansion; makes relative paths
/// absolute against cwd; lexically normalizes `.` / `..`. The
/// resulting path is **not** required to exist (init creates it).
fn resolve_path_target(p: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let s = p.to_str().ok_or("path TARGET is not valid UTF-8")?;
    let expanded = expand_vars(s)?;
    let pb = PathBuf::from(&expanded);
    let abs = if pb.is_absolute() {
        pb
    } else {
        std::env::current_dir()?.join(pb)
    };
    Ok(normalize_path(&abs))
}

/// Lexically normalize a path: collapse `.` / `..` components
/// without touching disk. `std::fs::canonicalize` requires the
/// path to exist, but init's destination doesn't yet.
fn normalize_path(p: &Path) -> PathBuf {
    let mut out: Vec<std::path::Component> = Vec::new();
    for comp in p.components() {
        match comp {
            std::path::Component::ParentDir => {
                let pop = matches!(
                    out.last(),
                    Some(std::path::Component::Normal(_)) | Some(std::path::Component::CurDir)
                );
                if pop {
                    out.pop();
                } else {
                    out.push(comp);
                }
            }
            std::path::Component::CurDir => {}
            other => out.push(other),
        }
    }
    out.iter().collect()
}

/// Extract the `owner/name` slug from a GitHub URL (any of the
/// supported forms: SSH scp-like, `ssh://`, `https://`).
fn github_slug_from_url(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let stem = url.strip_suffix(".git").unwrap_or(url);
    // The slug is the last two path components, `owner/name`.
    if let Some(path) = stem
        .strip_prefix("git@github.com:")
        .or_else(|| stem.strip_prefix("ssh://git@github.com/"))
        .or_else(|| stem.strip_prefix("https://github.com/"))
        .or_else(|| stem.strip_prefix("http://github.com/"))
    {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return Ok(format!(
                "{}/{}",
                parts[parts.len() - 2],
                parts[parts.len() - 1]
            ));
        }
    }
    Err(format!("cannot extract owner/name slug from GitHub URL '{url}'").into())
}

/// Init entry point — runs the dual or POR provisioning flow.
///
/// Takes the shared `Context` (loaded user config) and the flat
/// `InitParams`. CLI builds both at the binary edge in main.rs;
/// tests construct `InitParams` directly.
///
/// `params.create_symlink` controls the `~/.claude/projects/`
/// symlink side effect:
///
/// - `true` — CLI behavior; step 11 creates the symlink for
///   dual-scope runs.
/// - `false` — suppresses that side effect; used by test
///   harnesses (`test_helpers::Fixture`, `test_helpers::FixturePor`)
///   so parallel fixtures don't collide on the user's home dir.
pub fn init(ctx: &Context, params: &InitParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("init: enter");

    let create_symlink = params.create_symlink;
    let cfg = &ctx.user_config;
    let plan = plan_init(params, cfg)?;
    let is_dual = plan.scope.is_both();

    // --- Preflight ---
    info!("Preflight checks...");

    debug!("verify jj is installed");
    run("jj", &["--version"], Path::new(".")).map_err(|_| "jj is not installed")?;

    if plan.provisioner == Provisioner::GhCreate {
        debug!("verify gh CLI is installed + authenticated");
        run("gh", &["auth", "status"], Path::new("."))
            .map_err(|_| "gh is not installed or not authenticated (run: gh auth login)")?;
    }

    if plan.project_dir.exists() {
        return Err(format!("'{}' already exists", plan.project_dir.display()).into());
    }

    match &plan.provisioner {
        Provisioner::GhCreate => {
            #[allow(clippy::unwrap_used)]
            let code_slug = plan.gh_code_slug.as_ref().unwrap(); // OK: GhCreate path always sets gh_code_slug
            let (code_owner, code_name) = split_slug(code_slug)?;
            if gh_repo_exists(code_owner, code_name)? {
                return Err(format!("GitHub repo '{code_slug}' already exists").into());
            }
            if let Some(session_slug) = plan.gh_session_slug.as_ref() {
                let (session_owner, session_name) = split_slug(session_slug)?;
                if gh_repo_exists(session_owner, session_name)? {
                    return Err(format!("GitHub repo '{session_slug}' already exists").into());
                }
            }
        }
        Provisioner::LocalBareInit => {
            #[allow(clippy::unwrap_used)]
            let code_bare = plan.code_bare_path.as_ref().unwrap(); // OK: LocalBareInit path always sets code_bare_path
            if code_bare.exists() {
                return Err(format!(
                    "bare repo '{}' already exists; refusing to clobber",
                    code_bare.display()
                )
                .into());
            }
            if let Some(session_bare) = plan.session_bare_path.as_ref()
                && session_bare.exists()
            {
                return Err(format!(
                    "bare repo '{}' already exists; refusing to clobber",
                    session_bare.display()
                )
                .into());
            }
        }
        Provisioner::ExternalPreExisting => {
            // For path-shaped URLs: the path must exist (caller
            // pre-created with `git init --bare`). For scheme-URL
            // values we can't preflight cheaply — let git push
            // surface failures.
            if !is_remote_url(&plan.code_url) {
                let p = PathBuf::from(&plan.code_url);
                if !p.exists() {
                    return Err(format!(
                        "--repo-remote: code path '{}' does not exist — \
                         pre-create with `git init --bare`, or use --repo-local for fixture creation",
                        p.display()
                    )
                    .into());
                }
            }
            if let Some(session_url) = plan.session_url.as_ref()
                && !is_remote_url(session_url)
            {
                let p = PathBuf::from(session_url);
                if !p.exists() {
                    return Err(format!(
                        "--repo-remote: session path '{}' does not exist — \
                         pre-create with `git init --bare`, or use --repo-local for fixture creation",
                        p.display()
                    )
                    .into());
                }
            }
        }
    }

    let templates = match &params.use_template {
        Some(s) => {
            let (code_t, bot_t) = parse_use_template(s)?;
            if is_dual {
                validate_templates(&code_t, &bot_t)?;
                Some((code_t, Some(bot_t)))
            } else {
                validate_template_one("code", &code_t)?;
                Some((code_t, None))
            }
        }
        None => None,
    };

    let visibility = if params.private {
        "--private"
    } else {
        "--public"
    };

    if params.dry_run {
        info!("Dry run — would execute:");
        info!("  1. Create directories: {}", plan.project_dir.display());
        info!(
            "  2. git init + jj git init --colocate on {}",
            if is_dual {
                "both repos"
            } else {
                "the code repo"
            }
        );
        info!(
            "  3. Write .vc-config.toml and .gitignore{}",
            if is_dual {
                " to both repos"
            } else {
                " (code-only layout)"
            }
        );
        match &templates {
            Some((c, b)) => {
                info!("  4. Copy templates (non-hidden) + rewrite README.md first line");
                info!("       code: {}", c.display());
                if let Some(b) = b {
                    info!("       bot:  {}", b.display());
                }
            }
            None => info!("  4. (skipped — no --use-template)"),
        }
        if is_dual {
            info!("  5. jj commit both with placeholder ochids");
            info!("  6. Get both chids, jj describe both with correct ochids");
            info!("  7. Remove jj from both (git clean -xdf)");
        } else {
            info!("  5. jj commit code with 'Initial commit'");
            info!("  6. (skipped — no cross-reference in single-repo)");
            info!("  7. Remove jj (git clean -xdf)");
        }
        match &plan.provisioner {
            Provisioner::GhCreate => {
                if is_dual {
                    info!(
                        "  8. gh repo create {} {visibility}; push to {}",
                        plan.gh_session_slug.as_deref().unwrap_or(""), // OK: dry-run display only
                        plan.session_url.as_deref().unwrap_or(""),     // OK: dry-run display only
                    );
                } else {
                    info!("  8. (skipped — no session side in single-repo)");
                }
                info!(
                    "  9. gh repo create {} {visibility}; push to {}",
                    plan.gh_code_slug.as_deref().unwrap_or(""), // OK: dry-run display only
                    plan.code_url
                );
            }
            Provisioner::LocalBareInit => {
                if is_dual {
                    info!(
                        "  8. git init --bare {}; push to {}",
                        plan.session_bare_path
                            .as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default(), // OK: dry-run display only
                        plan.session_url.as_deref().unwrap_or(""), // OK: dry-run display only
                    );
                } else {
                    info!("  8. (skipped — no session side in single-repo)");
                }
                info!(
                    "  9. git init --bare {}; push to {}",
                    plan.code_bare_path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default(), // OK: dry-run display only
                    plan.code_url
                );
            }
            Provisioner::ExternalPreExisting => {
                if is_dual {
                    info!(
                        "  8. skip create; push to pre-existing {}",
                        plan.session_url.as_deref().unwrap_or(""), // OK: dry-run display only
                    );
                } else {
                    info!("  8. (skipped — no session side in single-repo)");
                }
                info!("  9. skip create; push to pre-existing {}", plan.code_url);
            }
        }
        info!(
            "  10. jj git init --colocate on {}",
            if is_dual {
                "both repos"
            } else {
                "the code repo"
            }
        );
        if is_dual {
            info!("  11. Create Claude Code symlink");
        } else {
            info!("  11. (skipped — no .claude symlink in single-repo)");
        }
        return Ok(());
    }

    match params.scope {
        ScopeKind::CodeBot => create_dual(params, &plan, templates, visibility, create_symlink),
        ScopeKind::Por => create_por(params, &plan, templates, visibility, create_symlink),
    }
}

/// Create-from-empty orchestrator for `--scope=por` (single repo).
///
/// Composes (in order):
/// - `prepare_local_repo` → conditional `.vc-config.toml` write
///   (gated by `--config`) → `write_por_gitignore` (always) →
///   `commit_initial` (`OchidStrategy::None`).
/// - `push_repo` for code side (no `clean_exclude`).
/// - No cross-reference (no session), no session push, no symlink.
///
/// `_create_symlink` is unused (no symlink in single-repo); kept in
/// the signature for shape-symmetry with `create_dual`.
fn create_por(
    params: &InitParams,
    plan: &InitPlan,
    templates: Option<(PathBuf, Option<PathBuf>)>,
    visibility: &str,
    _create_symlink: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let code_template = templates.as_ref().map(|(c, _)| c.as_path());

    prepare_local_repo(&plan.project_dir, "code", code_template, &plan.name)?;
    match &params.config {
        None => write_por_vc_config(&plan.project_dir)?,
        Some(ConfigKind::None) => {} // skip — user asked not to write
        Some(ConfigKind::Path(p)) => copy_user_config(p, &plan.project_dir)?,
    }
    write_por_gitignore(&plan.project_dir)?;
    let code_chid = commit_initial(&plan.project_dir, "code", OchidStrategy::None)?;

    info!("Step 6: (skipped — no cross-reference in single-repo)");
    let hash = run("git", &["rev-parse", "HEAD"], &plan.project_dir)?;
    debug!("code repo: chid={code_chid} hash={hash}");

    info!("Step 8: (skipped — no session side in single-repo)");

    let code_chid_final = push_repo(
        &plan.project_dir,
        "code",
        "Step 9",
        None,
        plan,
        params,
        visibility,
        &plan.code_url,
        plan.gh_code_slug.as_deref(),
        plan.code_bare_path.as_deref(),
    )?;

    info!("Step 11: (skipped — no .claude symlink in single-repo)");

    info!("");
    info!("Done! Project created at {}", plan.project_dir.display());
    info!(
        "  Code repo:    {}  (chid={code_chid_final})",
        plan.code_url
    );

    debug!("init: exit");
    Ok(())
}

/// Create-from-empty orchestrator for `--scope=code,bot` (dual).
///
/// Composes (in order):
/// - Code side: `prepare_local_repo` → `write_code_config` →
///   `commit_initial` (`OchidStrategy::Placeholder`).
/// - Session side: `prepare_local_repo` → `write_session_config`
///   → `commit_initial` (`OchidStrategy::Placeholder`).
/// - `cross_ref_ochids` — rewrite both initial commits' placeholder
///   trailers once each side's chid is known.
/// - `push_repo` for session side (no `clean_exclude`).
/// - `push_repo` for code side with `clean_exclude = Some(".claude")`
///   so the nested session repo survives the code-side clean.
/// - `symlink::install` (when `create_symlink`).
fn create_dual(
    params: &InitParams,
    plan: &InitPlan,
    templates: Option<(PathBuf, Option<PathBuf>)>,
    visibility: &str,
    create_symlink: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    #[allow(clippy::unwrap_used)]
    let session_dir = plan.session_dir.as_ref().unwrap(); // OK: scope=code,bot ⇒ session_dir set
    #[allow(clippy::unwrap_used)]
    let session_name = plan.session_name.as_ref().unwrap(); // OK: scope=code,bot ⇒ session_name set
    #[allow(clippy::unwrap_used)]
    let session_url = plan.session_url.as_deref().unwrap(); // OK: scope=code,bot ⇒ session_url set

    let (code_template, bot_template) = match templates.as_ref() {
        Some((c, b)) => (Some(c.as_path()), b.as_deref()),
        None => (None, None),
    };

    prepare_local_repo(&plan.project_dir, "code", code_template, &plan.name)?;
    write_code_config(&plan.project_dir)?;
    let code_chid = commit_initial(&plan.project_dir, "code", OchidStrategy::Placeholder)?;

    prepare_local_repo(session_dir, "bot", bot_template, session_name)?;
    write_session_config(session_dir)?;
    let session_chid = commit_initial(session_dir, "bot", OchidStrategy::Placeholder)?;

    cross_ref_ochids(&plan.project_dir, &code_chid, session_dir, &session_chid)?;

    let session_chid_final = push_repo(
        session_dir,
        "session",
        "Step 8",
        None,
        plan,
        params,
        visibility,
        session_url,
        plan.gh_session_slug.as_deref(),
        plan.session_bare_path.as_deref(),
    )?;
    // `Some(".claude")` clean_exclude preserves the nested session
    // repo through the code-side `git clean -xdf`.
    let code_chid_final = push_repo(
        &plan.project_dir,
        "code",
        "Step 9",
        Some(".claude"),
        plan,
        params,
        visibility,
        &plan.code_url,
        plan.gh_code_slug.as_deref(),
        plan.code_bare_path.as_deref(),
    )?;

    let sl_opt = if create_symlink {
        info!("Step 11: Creating Claude Code symlink...");
        Some(symlink::install(&plan.project_dir)?)
    } else {
        info!("Step 11: (skipped — symlink disabled by caller)");
        None
    };

    info!("");
    info!("Done! Project created at {}", plan.project_dir.display());
    info!(
        "  Code repo:    {}  (chid={code_chid_final})",
        plan.code_url
    );
    info!("  Session repo: {session_url}  (chid={session_chid_final})");
    if let Some(sl) = sl_opt.as_ref() {
        info!(
            "  Symlink:      {} -> {}",
            sl.symlink_path.display(),
            sl.abs_target.display()
        );
    }

    debug!("init: exit");
    Ok(())
}

/// Push primitive: steps 7, 8|9, 10 on `target`. Returns the
/// final chid (`jj @-`) after re-init.
///
/// - `target` — repo working dir (already populated by
///   `prepare_local_repo` + `commit_initial`).
/// - `info_label` — narration tag (`"code"`, `"session"`, etc.).
/// - `step_label_provision` — `"Step 8"` (session) or
///   `"Step 9"` (code) — appears in the provision/push narration.
/// - `clean_exclude` — `Some(".claude")` for code-side dual to
///   preserve the nested session repo across `git clean -xdf`;
///   `None` otherwise.
/// - `plan` / `params` / `visibility` / `remote_url` / `gh_slug` /
///   `bare_path` — forwarded to `run_remote_step`.
#[allow(clippy::too_many_arguments)]
fn push_repo(
    target: &Path,
    info_label: &str,
    step_label_provision: &str,
    clean_exclude: Option<&str>,
    plan: &InitPlan,
    params: &InitParams,
    visibility: &str,
    remote_url: &str,
    gh_slug: Option<&str>,
    bare_path: Option<&Path>,
) -> Result<String, Box<dyn std::error::Error>> {
    info!("Step 7: Setting {info_label} bookmark and removing jj...");
    debug!("place {info_label}-side main bookmark at the initial commit");
    run("jj", &["bookmark", "set", "main", "-r", "@-"], target)?;
    match clean_exclude {
        Some(ex) => {
            debug!("strip jj state from {info_label} side, preserving nested {ex}/");
            run("git", &["clean", "-xdf", "--exclude", ex], target)?;
        }
        None => {
            debug!("strip jj state from {info_label} side");
            run("git", &["clean", "-xdf"], target)?;
        }
    }
    debug!("re-attach {info_label} HEAD to main after clean");
    run("git", &["checkout", "main"], target)?;

    run_remote_step(
        step_label_provision,
        info_label,
        plan,
        remote_url,
        gh_slug,
        bare_path,
        visibility,
        target,
        params,
    )?;

    info!("Step 10: Re-initializing jj on the {info_label} repo...");
    debug!("re-colocate jj atop the now-remote-linked {info_label} repo");
    run("jj", &["--quiet", "git", "init", "--colocate"], target)?;
    debug!("restore {info_label}-side main bookmark after re-init");
    run("jj", &["bookmark", "set", "main", "-r", "@-"], target)?;
    debug!("enable jj tracking of {info_label}-side main against origin");
    run(
        "jj",
        &["bookmark", "track", "main", "--remote=origin"],
        target,
    )?;
    crate::common::verify_tracking(target, "main")?;
    jj_chid("@-", target)
}

/// Execute Step 8 or Step 9 for one side.
///
/// - Provision the remote per the plan's provisioner.
/// - Add `origin` and push `main` with retry.
/// - Centralizing keeps both sides' step bodies identical.
#[allow(clippy::too_many_arguments)]
fn run_remote_step(
    step_label: &str,
    side_label: &str,
    plan: &InitPlan,
    remote_url: &str,
    gh_slug: Option<&str>,
    bare_path: Option<&Path>,
    visibility: &str,
    push_from: &Path,
    params: &InitParams,
) -> Result<(), Box<dyn std::error::Error>> {
    match &plan.provisioner {
        Provisioner::GhCreate => {
            // Safe: GhCreate always supplies gh_slug.
            #[allow(clippy::unwrap_used)]
            let slug = gh_slug.unwrap(); // OK: GhCreate path always sets gh_slug
            info!("{step_label}: Creating GitHub repo {slug} ({side_label})...");
            debug!("create {side_label}-side remote on GitHub");
            run(
                "gh",
                &["repo", "create", slug, visibility],
                &plan.project_dir,
            )?;
        }
        Provisioner::LocalBareInit => {
            // Safe: LocalBareInit always supplies bare_path.
            #[allow(clippy::unwrap_used)]
            let bare = bare_path.unwrap(); // OK: LocalBareInit path always sets bare_path
            info!(
                "{step_label}: Initializing local bare repo at {} ({side_label})...",
                bare.display()
            );
            debug!("init {side_label}-side bare repo as the local origin");
            run(
                "git",
                &["init", "--bare", &bare.to_string_lossy()],
                Path::new("."),
            )?;
        }
        Provisioner::ExternalPreExisting => {
            info!("{step_label}: Using pre-existing {side_label} remote {remote_url}");
        }
    }
    debug!("point {side_label}-side git at its remote");
    run("git", &["remote", "add", "origin", remote_url], push_from)?;
    debug!("publish {side_label}-side initial commit; retry for GhCreate's async propagation");
    run_retry(
        "git",
        &["push", "-u", "origin", "main"],
        push_from,
        &params.push_retry,
    )?;
    Ok(())
}

/// Split an `owner/name` slug. Errors if the shape is wrong (no `/`
/// or more than one `/`).
fn split_slug(slug: &str) -> Result<(&str, &str), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = slug.split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(format!("unexpected GitHub slug '{slug}' (want 'owner/name')").into());
    }
    Ok((parts[0], parts[1]))
}

#[cfg(test)]
mod tests;

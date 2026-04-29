use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info};

use crate::args::{ScopeKind, parse_repo_arg, parse_scope_kind};
use crate::config::{self, RepoSelector, UserConfig};
use crate::repo_url::{Target, derive_name, derive_session_url, parse_target};
use crate::scope::{Scope, Side};
use crate::symlink;

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

    /// Account name — picks `[account.<a>]` from user config.
    ///
    /// - Without this flag, `[default].account` (or top-level
    ///   `[repo]` shorthand) is used.
    /// - Meaningful only with Path or bare-NAME targets — URL /
    ///   owner/name targets supply the remote directly.
    #[arg(long, value_name = "NAME", verbatim_doc_comment)]
    pub account: Option<String>,

    /// Repo target — `<cat>` or `<cat>=<val>`.
    ///
    /// - Built-in categories: `remote` (URL prefix; init appends
    ///   `/<NAME>.git`) and `local` (parent dir for fixture bare
    ///   repos at `<parent>/remote-{code,claude}.git`).
    /// - `--repo <cat>` looks up the value via the account chain.
    /// - `--repo <cat>=<val>` uses the literal value, no config
    ///   lookup needed.
    /// - Meaningful only with Path or bare-NAME targets.
    #[arg(
        long,
        value_name = "CAT[=VAL]",
        value_parser = parse_repo_arg,
        verbatim_doc_comment
    )]
    pub repo: Option<RepoSelector>,

    /// ScopeKind — `code,bot` (dual, default) or `por` (single).
    #[arg(
        long,
        short,
        value_name = "SCOPE",
        value_parser = parse_scope_kind,
        default_value = "code,bot",
        verbatim_doc_comment
    )]
    pub scope: ScopeKind,

    /// Create private GitHub repos (default: public).
    ///
    /// - Only meaningful when the resolved provisioner is
    ///   `gh repo create` (GitHub URL or `--repo remote` whose
    ///   value points at GitHub).
    #[arg(long, verbatim_doc_comment)]
    pub private: bool,

    /// Dry run — show what would be done without executing
    #[arg(long)]
    pub dry_run: bool,

    /// Max push retries after repo creation [default: 5]
    #[arg(long, default_value_t = 5)]
    pub push_retries: u32,

    /// Seconds between push retries [default: 3]
    #[arg(long, default_value_t = 3)]
    pub push_retry_delay: u64,

    /// Seed repos from template directories.
    ///
    /// Value is `CODE[,BOT]`. Default bot path is `<CODE>.claude`
    /// (file-name concat, not path join — templates are siblings).
    ///
    /// - With `--scope=por`: only `CODE` is used; passing `,BOT`
    ///   is fatal (no session side to seed).
    /// - Non-hidden contents copied recursively; hidden entries
    ///   (names starting with `.`) are skipped — init writes its
    ///   own hidden files.
    /// - If a copied tree has a `README.md`, its first line is
    ///   rewritten to `# <repo-name>`.
    #[arg(long, value_name = "CODE[,BOT]", verbatim_doc_comment)]
    pub use_template: Option<String>,
}

/// Run a command with retries, sleeping between attempts.
fn run_retry(
    cmd: &str,
    args: &[&str],
    cwd: &Path,
    retries: u32,
    delay_secs: u64,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut last_err = String::new();
    for attempt in 1..=retries {
        match run(cmd, args, cwd) {
            Ok(out) => {
                if attempt > 1 {
                    debug!("succeeded after {attempt} attempts");
                }
                return Ok(out);
            }
            Err(e) => {
                last_err = e.to_string();
                if attempt < retries {
                    debug!("attempt {attempt}/{retries} failed: {last_err}");
                    debug!("retrying in {delay_secs}s...");
                    std::thread::sleep(std::time::Duration::from_secs(delay_secs));
                }
            }
        }
    }
    Err(format!("failed after {retries} attempts: {last_err}").into())
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
fn jj_chid(rev: &str, cwd: &Path) -> Result<String, Box<dyn std::error::Error>> {
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

const VC_CONFIG_CODE: &str = r#"# vc-config: Vibe Coding workspace configuration
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

const GITIGNORE_CODE: &str = "/target
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
const VC_CONFIG_APP_ONLY: &str = r#"# vc-config: Vibe Coding workspace configuration
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
const GITIGNORE_APP_ONLY: &str = "/target
/.git
/.jj
/.vc-x1
";

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
    args: &InitArgs,
    cfg: &UserConfig,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    debug!(
        "init args: target={:?}, name={:?}, account={:?}, repo={:?}, scope={:?}, private={}",
        args.target, args.name, args.account, args.repo, args.scope, args.private
    );

    let scope = match args.scope {
        ScopeKind::CodeBot => Scope(vec![Side::Code, Side::Bot]),
        ScopeKind::Por => Scope(vec![Side::Code]),
    };

    if scope.is_code_only()
        && let Some(t) = &args.use_template
        && t.contains(',')
    {
        return Err(format!(
            "--scope=por takes a single template path; got '{t}' (drop the `,BOT` half)"
        )
        .into());
    }

    let parsed = parse_target(&args.target)?;
    debug!("parse_target: {:?} → {:?}", args.target, parsed);
    let plan = match parsed {
        Target::Url(url) => plan_from_url(args, scope, url),
        Target::OwnerName(o, n) => {
            plan_from_url(args, scope, format!("git@github.com:{o}/{n}.git"))
        }
        Target::Path(p) => plan_from_path(args, scope, p, cfg),
        Target::BareName(n) => plan_from_bare_name(args, scope, n, cfg),
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
    args: &InitArgs,
    scope: Scope,
    url: String,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    if args.account.is_some() {
        return Err(
            "--account is meaningless with a URL or owner/name TARGET (config not consulted)"
                .into(),
        );
    }
    if args.repo.is_some() {
        return Err(
            "--repo is meaningless with a URL or owner/name TARGET (config not consulted)".into(),
        );
    }
    let code_url = ensure_git_suffix(&url);
    let derived_name = derive_name(&code_url)?;
    let name = args.name.clone().unwrap_or(derived_name);
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
    args: &InitArgs,
    scope: Scope,
    path: PathBuf,
    cfg: &UserConfig,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    if args.name.is_some() {
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
    let (cat, val) = config::resolve_repo(cfg, args.account.as_deref(), args.repo.as_ref())?;
    plan_from_resolved(scope, name, project_dir, &cat, &val)
}

/// Plan when TARGET is a bare alphanumeric NAME. Destination at
/// `cwd/<NAME>`; remote resolved via the `--repo` chain.
fn plan_from_bare_name(
    args: &InitArgs,
    scope: Scope,
    name: String,
    cfg: &UserConfig,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    if args.name.is_some() {
        return Err(
            "[NAME] is meaningless with a bare-NAME TARGET (TARGET already names the repo)".into(),
        );
    }
    let cwd = std::env::current_dir()?;
    let project_dir = cwd.join(&name);
    let (cat, val) = config::resolve_repo(cfg, args.account.as_deref(), args.repo.as_ref())?;
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

/// CLI entry point — dual-repo init creates the
/// `~/.claude/projects/` symlink on success.
pub fn init(args: &InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    init_with_symlink(args, true)
}

/// Core init routine with a caller-controlled symlink toggle.
///
/// - `create_symlink=true` — CLI behavior.
/// - `create_symlink=false` — suppresses step 11's side effect
///   on `$HOME/.claude/projects/` (test harnesses only).
pub(crate) fn init_with_symlink(
    args: &InitArgs,
    create_symlink: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("init: enter");

    let cfg = config::load()?;
    let plan = plan_init(args, &cfg)?;
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

    let templates = match &args.use_template {
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

    let visibility = if args.private {
        "--private"
    } else {
        "--public"
    };

    if args.dry_run {
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

    if is_dual {
        init_dual(args, &plan, templates, visibility, create_symlink)
    } else {
        init_one(args, &plan, templates, visibility)
    }
}

/// Create-from-empty primitive for `--scope=por` (single repo).
///
/// Runs the linear lifecycle on `plan.project_dir`:
///
/// - Step 1: mkdir.
/// - Step 2: `git init` + colocated `jj git init`.
/// - Step 3: write the POR-shape `.vc-config.toml` + `.gitignore`.
/// - Step 4: copy code template (if any) + rewrite README.
/// - Step 5: plain initial commit (no ochid).
/// - Step 7: bookmark + `git clean -xdf`.
/// - Step 9: provision remote + push.
/// - Step 10: re-init jj + bookmark tracking.
///
/// No cross-reference ochid, no session repo, no symlink.
fn init_one(
    args: &InitArgs,
    plan: &InitPlan,
    templates: Option<(PathBuf, Option<PathBuf>)>,
    visibility: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Create directories. For LocalBareInit also create the
    // parent so the bare-repo init (step 9) has a home.
    info!("Step 1: Creating project directories...");
    if let Some(parent) = plan.project_dir.parent() {
        mkdir_p(parent)?;
    }
    mkdir_p(&plan.project_dir)?;

    // Step 2: git init + jj init
    info!("Step 2: Initializing repos...");
    debug!("seed project_dir with an empty git repo");
    run("git", &["init"], &plan.project_dir)?;
    debug!("colocate jj atop the git repo so jj + git share .git state");
    run("jj", &["git", "init", "--colocate"], &plan.project_dir)?;

    // Step 3: Write config files (POR shape)
    info!("Step 3: Writing config files...");
    write_file(
        &plan.project_dir.join(".vc-config.toml"),
        VC_CONFIG_APP_ONLY,
    )?;
    write_file(&plan.project_dir.join(".gitignore"), GITIGNORE_APP_ONLY)?;

    // Step 4: Copy template (if --use-template)
    if let Some((code_t, _)) = &templates {
        info!("Step 4: Copying templates...");
        info!(
            "  code: {} -> {}",
            code_t.display(),
            plan.project_dir.display()
        );
        copy_template_recursive(code_t, &plan.project_dir)?;
        rewrite_readme_first_line(&plan.project_dir, &plan.name)?;
    }

    // Step 5: jj commit (plain — no ochid in single-repo)
    info!("Step 5: Committing code...");
    debug!("code side: initial commit (no ochid in single-repo)");
    run("jj", &["commit", "-m", "Initial commit"], &plan.project_dir)?;

    // Step 6: skipped — no cross-reference in single-repo
    info!("Step 6: (skipped — no cross-reference in single-repo)");
    let code_chid = jj_chid("@-", &plan.project_dir)?;
    let hash = run("git", &["rev-parse", "HEAD"], &plan.project_dir)?;
    debug!("code repo: chid={code_chid} hash={hash}");

    // Step 7: Set bookmark (creates git branch), then remove jj
    info!("Step 7: Setting bookmarks and removing jj...");
    debug!("place code-side main bookmark at the initial commit");
    run(
        "jj",
        &["bookmark", "set", "main", "-r", "@-"],
        &plan.project_dir,
    )?;
    debug!("strip jj state from code side (no .claude in single-repo)");
    run("git", &["clean", "-xdf"], &plan.project_dir)?;
    debug!("re-attach code HEAD to main after clean");
    run("git", &["checkout", "main"], &plan.project_dir)?;

    // Step 8: skipped — no session side in single-repo
    info!("Step 8: (skipped — no session side in single-repo)");

    // Step 9: provision remote and push code
    run_remote_step(
        "Step 9",
        "code",
        plan,
        &plan.code_url,
        plan.gh_code_slug.as_deref(),
        plan.code_bare_path.as_deref(),
        visibility,
        &plan.project_dir,
        args,
    )?;

    // Step 10: Re-initialize jj
    info!("Step 10: Re-initializing jj on the code repo...");
    debug!("re-colocate jj atop the now-remote-linked code repo");
    run(
        "jj",
        &["--quiet", "git", "init", "--colocate"],
        &plan.project_dir,
    )?;
    debug!("restore code-side main bookmark after re-init");
    run(
        "jj",
        &["bookmark", "set", "main", "-r", "@-"],
        &plan.project_dir,
    )?;
    debug!("enable jj tracking of code-side main against origin");
    run(
        "jj",
        &["bookmark", "track", "main", "--remote=origin"],
        &plan.project_dir,
    )?;
    crate::common::verify_tracking(&plan.project_dir, "main")?;
    let code_chid_final = jj_chid("@-", &plan.project_dir)?;

    // Step 11: skipped — no .claude symlink in single-repo
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
/// Runs the lifecycle across both code and session repos,
/// interleaved as needed:
///
/// - Step 5 commits both with placeholder ochids; step 6 rewrites
///   them once both chids are known.
/// - Step 7's code-side clean preserves `.claude/`.
/// - Step 11 installs the symlink (toggled by `create_symlink`).
fn init_dual(
    args: &InitArgs,
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

    // Step 1: Create directories. For LocalBareInit also create the
    // parent so the bare-repo init (step 8/9) has a home.
    info!("Step 1: Creating project directories...");
    if let Some(parent) = plan.project_dir.parent() {
        mkdir_p(parent)?;
    }
    mkdir_p(&plan.project_dir)?;
    mkdir_p(session_dir)?;

    // Step 2: git init + jj init both
    info!("Step 2: Initializing repos...");
    debug!("seed project_dir with an empty git repo");
    run("git", &["init"], &plan.project_dir)?;
    debug!("colocate jj atop the git repo so jj + git share .git state");
    run("jj", &["git", "init", "--colocate"], &plan.project_dir)?;
    debug!("seed session_dir with an empty git repo");
    run("git", &["init"], session_dir)?;
    debug!("colocate jj atop the session git repo");
    run("jj", &["git", "init", "--colocate"], session_dir)?;

    // Step 3: Write config files (CODE/SESSION shapes)
    info!("Step 3: Writing config files...");
    write_file(&plan.project_dir.join(".vc-config.toml"), VC_CONFIG_CODE)?;
    write_file(&plan.project_dir.join(".gitignore"), GITIGNORE_CODE)?;
    write_file(&session_dir.join(".vc-config.toml"), VC_CONFIG_SESSION)?;
    write_file(&session_dir.join(".gitignore"), GITIGNORE_SESSION)?;

    // Step 4: Copy templates (if --use-template)
    if let Some((code_t, bot_t)) = &templates {
        info!("Step 4: Copying templates...");
        info!(
            "  code: {} -> {}",
            code_t.display(),
            plan.project_dir.display()
        );
        copy_template_recursive(code_t, &plan.project_dir)?;
        rewrite_readme_first_line(&plan.project_dir, &plan.name)?;
        if let Some(bot_t) = bot_t.as_ref() {
            info!("  bot:  {} -> {}", bot_t.display(), session_dir.display());
            copy_template_recursive(bot_t, session_dir)?;
            rewrite_readme_first_line(session_dir, session_name)?;
        }
    }

    // Step 5: jj commit (placeholder ochids, rewritten in step 6)
    info!("Step 5: Committing both repos with placeholder ochids...");
    debug!("code side: initial commit with placeholder ochid (rewritten in step 6)");
    run(
        "jj",
        &["commit", "-m", "Initial commit\n\nochid: /none"],
        &plan.project_dir,
    )?;
    debug!("session side: initial commit with placeholder ochid (rewritten in step 6)");
    run(
        "jj",
        &["commit", "-m", "Initial commit\n\nochid: /none"],
        session_dir,
    )?;

    // Step 6: ochid cross-references
    info!("Step 6: Setting ochid cross-references...");
    let code_chid = jj_chid("@-", &plan.project_dir)?;
    let session_chid = jj_chid("@-", session_dir)?;

    let code_desc = format!("Initial commit\n\nochid: /.claude/{session_chid}");
    let session_desc = format!("Initial commit\n\nochid: /{code_chid}");
    debug!("code side: rewrite initial commit's ochid to point at session chid");
    run(
        "jj",
        &["describe", "@-", "-m", &code_desc],
        &plan.project_dir,
    )?;
    debug!("session side: rewrite initial commit's ochid to point at code chid");
    run("jj", &["describe", "@-", "-m", &session_desc], session_dir)?;

    debug!("surface post-describe git hashes for the debug log");
    let hash = run("git", &["rev-parse", "HEAD"], &plan.project_dir)?;
    debug!("code repo: chid={code_chid} hash={hash}");
    let hash = run("git", &["rev-parse", "HEAD"], session_dir)?;
    debug!(".claude:   chid={session_chid} hash={hash}");

    // Step 7: Set bookmarks, remove jj (code-side clean preserves .claude/)
    info!("Step 7: Setting bookmarks and removing jj...");
    debug!("place code-side main bookmark at the initial commit");
    run(
        "jj",
        &["bookmark", "set", "main", "-r", "@-"],
        &plan.project_dir,
    )?;
    debug!("place session-side main bookmark at the initial commit");
    run("jj", &["bookmark", "set", "main", "-r", "@-"], session_dir)?;
    debug!("strip jj state from session side before its git push");
    run("git", &["clean", "-xdf"], session_dir)?;
    debug!("strip jj state from code side, preserving nested .claude/");
    run(
        "git",
        &["clean", "-xdf", "--exclude", ".claude"],
        &plan.project_dir,
    )?;
    debug!("re-attach session HEAD to main after clean");
    run("git", &["checkout", "main"], session_dir)?;
    debug!("re-attach code HEAD to main after clean");
    run("git", &["checkout", "main"], &plan.project_dir)?;

    // Steps 8/9: provision remotes and push.
    run_remote_step(
        "Step 8",
        "session",
        plan,
        session_url,
        plan.gh_session_slug.as_deref(),
        plan.session_bare_path.as_deref(),
        visibility,
        session_dir,
        args,
    )?;
    run_remote_step(
        "Step 9",
        "code",
        plan,
        &plan.code_url,
        plan.gh_code_slug.as_deref(),
        plan.code_bare_path.as_deref(),
        visibility,
        &plan.project_dir,
        args,
    )?;

    // Step 10: Re-initialize jj on both repos
    info!("Step 10: Re-initializing jj on both repos...");
    debug!("re-colocate jj atop the now-remote-linked code repo");
    run(
        "jj",
        &["--quiet", "git", "init", "--colocate"],
        &plan.project_dir,
    )?;
    debug!("restore code-side main bookmark after re-init");
    run(
        "jj",
        &["bookmark", "set", "main", "-r", "@-"],
        &plan.project_dir,
    )?;
    debug!("enable jj tracking of code-side main against origin");
    run(
        "jj",
        &["bookmark", "track", "main", "--remote=origin"],
        &plan.project_dir,
    )?;
    crate::common::verify_tracking(&plan.project_dir, "main")?;
    let code_chid_final = jj_chid("@-", &plan.project_dir)?;

    debug!("re-colocate jj atop the now-remote-linked session repo");
    run("jj", &["--quiet", "git", "init", "--colocate"], session_dir)?;
    debug!("restore session-side main bookmark after re-init");
    run("jj", &["bookmark", "set", "main", "-r", "@-"], session_dir)?;
    debug!("enable jj tracking of session-side main against origin");
    run(
        "jj",
        &["bookmark", "track", "main", "--remote=origin"],
        session_dir,
    )?;
    crate::common::verify_tracking(session_dir, "main")?;
    let session_chid_final = jj_chid("@-", session_dir)?;

    // Step 11: Create Claude Code symlink (opt-out for tests)
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
    args: &InitArgs,
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
        args.push_retries,
        args.push_retry_delay,
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
mod tests {
    use super::*;
    use crate::{Cli, Commands};
    use clap::Parser;

    fn parse(args: &[&str]) -> InitArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Init(a) => a,
            _ => panic!("expected New"),
        }
    }

    #[test]
    fn defaults() {
        let args = parse(&["vc-x1", "init", "owner/repo"]);
        assert_eq!(args.target, "owner/repo");
        assert!(args.name.is_none());
        assert!(args.account.is_none());
        assert!(args.repo.is_none());
        assert_eq!(args.scope, ScopeKind::CodeBot);
        assert!(!args.private);
        assert!(!args.dry_run);
        assert_eq!(args.push_retries, 5);
        assert_eq!(args.push_retry_delay, 3);
        assert!(args.use_template.is_none());
    }

    #[test]
    fn all_opts() {
        let args = parse(&[
            "vc-x1",
            "init",
            "owner/repo",
            "my-dir",
            "--account",
            "work",
            "--repo",
            "local=/tmp/xyz",
            "--scope",
            "por",
            "--private",
            "--dry-run",
            "--push-retries",
            "10",
            "--push-retry-delay",
            "5",
            "--use-template",
            "/tmp/tmpl",
        ]);
        assert_eq!(args.target, "owner/repo");
        assert_eq!(args.name.as_deref(), Some("my-dir"));
        assert_eq!(args.account.as_deref(), Some("work"));
        let sel = args.repo.as_ref().expect("--repo set");
        assert_eq!(sel.category, "local");
        assert_eq!(sel.value.as_deref(), Some("/tmp/xyz"));
        assert_eq!(args.scope, ScopeKind::Por);
        assert!(args.private);
        assert!(args.dry_run);
        assert_eq!(args.push_retries, 10);
        assert_eq!(args.push_retry_delay, 5);
        assert_eq!(args.use_template.as_deref(), Some("/tmp/tmpl"));
    }

    #[test]
    fn target_required_at_parse_time() {
        // TARGET is a required positional; missing it errors.
        let err = Cli::try_parse_from(["vc-x1", "init"])
            .unwrap_err()
            .to_string();
        assert!(err.contains("TARGET"), "got: {err}");
    }

    #[test]
    fn config_content_code() {
        assert!(VC_CONFIG_CODE.contains("path = \"/\""));
        assert!(VC_CONFIG_CODE.contains("other-repo = \".claude\""));
    }

    #[test]
    fn config_content_session() {
        assert!(VC_CONFIG_SESSION.contains("path = \"/.claude\""));
        assert!(VC_CONFIG_SESSION.contains("other-repo = \"..\""));
    }

    #[test]
    fn gitignore_code_excludes_claude() {
        assert!(GITIGNORE_CODE.contains("/.claude"));
        assert!(GITIGNORE_CODE.contains("/.git"));
        assert!(GITIGNORE_CODE.contains("/.jj"));
        assert!(GITIGNORE_CODE.contains("/.vc-x1"));
    }

    #[test]
    fn gitignore_session_excludes_git() {
        assert!(GITIGNORE_SESSION.contains(".git"));
        assert!(GITIGNORE_SESSION.contains(".jj"));
    }

    use std::time::{SystemTime, UNIX_EPOCH};

    /// Create a unique temp dir for a test, sibling-style via file-name
    /// concat so both the code and bot template paths can live under it.
    fn tmp_root(tag: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!("vc-x1-inittest-{tag}-{ts}"));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn parse_use_template_both() {
        let (c, b) = parse_use_template("/a/code,/x/bot").unwrap();
        assert_eq!(c, PathBuf::from("/a/code"));
        assert_eq!(b, PathBuf::from("/x/bot"));
    }

    #[test]
    fn parse_use_template_default_bot() {
        let (c, b) = parse_use_template("/a/code").unwrap();
        assert_eq!(c, PathBuf::from("/a/code"));
        assert_eq!(b, PathBuf::from("/a/code.claude"));
    }

    #[test]
    fn parse_use_template_default_bot_trailing_slash() {
        // with_file_name normalises away the effect of a trailing slash.
        let (c, b) = parse_use_template("/a/code/").unwrap();
        assert_eq!(c, PathBuf::from("/a/code/"));
        assert_eq!(b, PathBuf::from("/a/code.claude"));
    }

    #[test]
    fn parse_use_template_empty_bot_falls_back_to_default() {
        let (c, b) = parse_use_template("/a/code,").unwrap();
        assert_eq!(c, PathBuf::from("/a/code"));
        assert_eq!(b, PathBuf::from("/a/code.claude"));
    }

    #[test]
    fn parse_use_template_empty_code_errors() {
        assert!(parse_use_template("").is_err());
        assert!(parse_use_template(",bot").is_err());
    }

    #[test]
    fn copy_template_skips_hidden_entries() {
        let root = tmp_root("copy-skip-hidden");
        let src = root.join("src");
        let dst = root.join("dst");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::create_dir_all(&dst).unwrap();

        // Non-hidden: visible file, visible dir with nested file.
        std::fs::write(src.join("keep.txt"), "keep").unwrap();
        std::fs::create_dir_all(src.join("sub")).unwrap();
        std::fs::write(src.join("sub").join("nested.txt"), "nested").unwrap();

        // Hidden: dotfile, dotdir (with contents that must NOT be copied).
        std::fs::write(src.join(".hidden"), "should-not-copy").unwrap();
        std::fs::create_dir_all(src.join(".dotdir")).unwrap();
        std::fs::write(src.join(".dotdir").join("inside"), "nope").unwrap();

        copy_template_recursive(&src, &dst).unwrap();

        assert_eq!(
            std::fs::read_to_string(dst.join("keep.txt")).unwrap(),
            "keep"
        );
        assert_eq!(
            std::fs::read_to_string(dst.join("sub").join("nested.txt")).unwrap(),
            "nested"
        );
        assert!(!dst.join(".hidden").exists());
        assert!(!dst.join(".dotdir").exists());

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn rewrite_readme_replaces_first_line() {
        let root = tmp_root("rewrite-readme");
        std::fs::write(
            root.join("README.md"),
            "# old-title\nbody line 1\nbody line 2\n",
        )
        .unwrap();

        rewrite_readme_first_line(&root, "new-name").unwrap();

        let got = std::fs::read_to_string(root.join("README.md")).unwrap();
        assert_eq!(got, "# new-name\nbody line 1\nbody line 2\n");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn rewrite_readme_no_newline() {
        let root = tmp_root("rewrite-readme-nonewline");
        std::fs::write(root.join("README.md"), "single-line-no-newline").unwrap();

        rewrite_readme_first_line(&root, "new-name").unwrap();

        let got = std::fs::read_to_string(root.join("README.md")).unwrap();
        assert_eq!(got, "# new-name");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn rewrite_readme_missing_is_noop() {
        let root = tmp_root("rewrite-readme-missing");
        // README.md not created — call must succeed silently.
        rewrite_readme_first_line(&root, "new-name").unwrap();
        assert!(!root.join("README.md").exists());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn validate_templates_missing_code() {
        let root = tmp_root("validate-missing-code");
        let code = root.join("nope");
        let bot = root.join("bot");
        std::fs::create_dir_all(&bot).unwrap();
        let err = validate_templates(&code, &bot).unwrap_err().to_string();
        assert!(err.contains("code template"));
        assert!(err.contains("does not exist"));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn validate_templates_not_a_dir() {
        let root = tmp_root("validate-not-dir");
        let code = root.join("code-file");
        let bot = root.join("bot");
        std::fs::write(&code, "i am a file").unwrap();
        std::fs::create_dir_all(&bot).unwrap();
        let err = validate_templates(&code, &bot).unwrap_err().to_string();
        assert!(err.contains("is not a directory"));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn end_to_end_copy_and_readme_rewrite() {
        // Simulates what init's Step 4 does: two sibling templates with
        // a README.md each, copied into two fresh target dirs, each
        // README retitled to the respective repo name.
        let root = tmp_root("e2e-copy-rewrite");
        let code_tmpl = root.join("vc-template-x1");
        let bot_tmpl = root.join("vc-template-x1.claude");
        let code_dst = root.join("dst-code");
        let bot_dst = root.join("dst-bot");
        std::fs::create_dir_all(&code_tmpl).unwrap();
        std::fs::create_dir_all(&bot_tmpl).unwrap();
        std::fs::create_dir_all(&code_dst).unwrap();
        std::fs::create_dir_all(&bot_dst).unwrap();

        std::fs::write(
            code_tmpl.join("README.md"),
            "# vc-template-x1\nCode template body.\n",
        )
        .unwrap();
        std::fs::write(code_tmpl.join("src.txt"), "code stuff").unwrap();
        std::fs::write(code_tmpl.join(".gitignore"), "should-not-copy").unwrap();

        std::fs::write(
            bot_tmpl.join("README.md"),
            "# vc-template-x1.claude\nBot template body.\n",
        )
        .unwrap();
        std::fs::write(bot_tmpl.join("session.md"), "bot stuff").unwrap();

        validate_templates(&code_tmpl, &bot_tmpl).unwrap();

        copy_template_recursive(&code_tmpl, &code_dst).unwrap();
        rewrite_readme_first_line(&code_dst, "my-proj").unwrap();
        copy_template_recursive(&bot_tmpl, &bot_dst).unwrap();
        rewrite_readme_first_line(&bot_dst, "my-proj.claude").unwrap();

        assert_eq!(
            std::fs::read_to_string(code_dst.join("README.md")).unwrap(),
            "# my-proj\nCode template body.\n"
        );
        assert_eq!(
            std::fs::read_to_string(code_dst.join("src.txt")).unwrap(),
            "code stuff"
        );
        assert!(!code_dst.join(".gitignore").exists());
        assert_eq!(
            std::fs::read_to_string(bot_dst.join("README.md")).unwrap(),
            "# my-proj.claude\nBot template body.\n"
        );
        assert_eq!(
            std::fs::read_to_string(bot_dst.join("session.md")).unwrap(),
            "bot stuff"
        );

        std::fs::remove_dir_all(&root).unwrap();
    }

    // ---------- TARGET / [NAME] / --account / --repo / --scope parsing ----------

    #[test]
    fn target_url_form_accepted() {
        let args = parse(&["vc-x1", "init", "git@github.com:u/p.git"]);
        assert_eq!(args.target, "git@github.com:u/p.git");
    }

    #[test]
    fn target_owner_name_form_accepted() {
        let args = parse(&["vc-x1", "init", "owner/repo"]);
        assert_eq!(args.target, "owner/repo");
    }

    #[test]
    fn target_path_form_accepted() {
        let args = parse(&["vc-x1", "init", "./tf1"]);
        assert_eq!(args.target, "./tf1");
    }

    #[test]
    fn target_bare_name_form_accepted() {
        let args = parse(&["vc-x1", "init", "tf1"]);
        assert_eq!(args.target, "tf1");
    }

    #[test]
    fn name_positional_accepted() {
        let args = parse(&["vc-x1", "init", "owner/repo", "custom-dir"]);
        assert_eq!(args.target, "owner/repo");
        assert_eq!(args.name.as_deref(), Some("custom-dir"));
    }

    #[test]
    fn account_flag_parses() {
        let args = parse(&["vc-x1", "init", "tf1", "--account", "work"]);
        assert_eq!(args.account.as_deref(), Some("work"));
    }

    #[test]
    fn repo_cat_only_parses() {
        let args = parse(&["vc-x1", "init", "tf1", "--repo", "remote"]);
        let sel = args.repo.expect("--repo set");
        assert_eq!(sel.category, "remote");
        assert!(sel.value.is_none());
    }

    #[test]
    fn repo_cat_value_parses() {
        let args = parse(&["vc-x1", "init", "tf1", "--repo", "remote=git@github.com:u"]);
        let sel = args.repo.expect("--repo set");
        assert_eq!(sel.category, "remote");
        assert_eq!(sel.value.as_deref(), Some("git@github.com:u"));
    }

    #[test]
    fn scope_short_flag_por() {
        let args = parse(&["vc-x1", "init", "tf1", "-s", "por"]);
        assert_eq!(args.scope, ScopeKind::Por);
    }

    #[test]
    fn is_remote_url_classifies() {
        assert!(is_remote_url("https://host/path"));
        assert!(is_remote_url("ssh://git@host/path"));
        assert!(is_remote_url("file:///abs/path"));
        assert!(is_remote_url("git@github.com:owner/repo.git"));
        assert!(!is_remote_url("/tmp/x.git"));
        assert!(!is_remote_url("./x.git"));
        assert!(!is_remote_url("../x.git"));
        assert!(!is_remote_url("plain/relative"));
    }

    #[test]
    fn is_github_url_classifies() {
        assert!(is_github_url("git@github.com:owner/repo"));
        assert!(is_github_url("git@github.com:owner/repo.git"));
        assert!(is_github_url("ssh://git@github.com/owner/repo.git"));
        assert!(is_github_url("https://github.com/owner/repo"));
        assert!(is_github_url("http://github.com/owner/repo"));
        assert!(!is_github_url("git@gitlab.com:owner/repo.git"));
        assert!(!is_github_url("https://gitlab.com/owner/repo"));
        assert!(!is_github_url("/tmp/foo.git"));
    }

    #[test]
    fn ensure_git_suffix_adds_when_missing() {
        assert_eq!(ensure_git_suffix("foo"), "foo.git");
        assert_eq!(ensure_git_suffix("foo.git"), "foo.git");
        assert_eq!(
            ensure_git_suffix("git@github.com:u/p"),
            "git@github.com:u/p.git"
        );
    }

    // Unit tests for derive_session_url / derive_name live in
    // src/repo_url.rs alongside the lifted functions.

    #[test]
    fn expand_vars_tilde() {
        let prev = std::env::var("HOME").ok();
        // SAFETY: env-var manipulation is genuinely racy with parallel
        // test runners. Tests touching $VAR and $HOME live in this
        // module; if flakiness emerges, run with --test-threads=1.
        unsafe {
            std::env::set_var("HOME", "/h");
        }
        assert_eq!(expand_vars("~/foo").unwrap(), "/h/foo");
        assert_eq!(expand_vars("~").unwrap(), "/h");
        // SAFETY: see above.
        unsafe {
            match prev {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
    }

    #[test]
    fn expand_vars_envvar() {
        let key = "VC_X1_TEST_EXPAND_VAR";
        // SAFETY: see expand_vars_tilde test.
        unsafe {
            std::env::set_var(key, "VAL");
        }
        assert_eq!(expand_vars("$VC_X1_TEST_EXPAND_VAR/x").unwrap(), "VAL/x");
        assert_eq!(expand_vars("a${VC_X1_TEST_EXPAND_VAR}b").unwrap(), "aVALb");
        assert_eq!(expand_vars("$/no-name").unwrap(), "$/no-name");
        assert!(expand_vars("$VC_X1_TEST_DEFINITELY_UNSET_xyz").is_err());
        // SAFETY: see expand_vars_tilde test.
        unsafe {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn expand_vars_unterminated_brace_errors() {
        assert!(expand_vars("${UNCLOSED").is_err());
    }

    // ---------- plan_init dispatch ----------

    use crate::config::AccountConfig;
    use std::collections::HashMap;

    /// Build an `InitArgs` with sane defaults; the caller overrides
    /// only the fields it cares about.
    fn args_for(target: &str) -> InitArgs {
        InitArgs {
            target: target.to_string(),
            name: None,
            account: None,
            repo: None,
            scope: ScopeKind::CodeBot,
            private: false,
            dry_run: true,
            push_retries: 5,
            push_retry_delay: 3,
            use_template: None,
        }
    }

    fn cfg_empty() -> UserConfig {
        UserConfig::default()
    }

    fn cfg_top_level_remote(prefix: &str) -> UserConfig {
        let tl = AccountConfig {
            repo_default: Some("remote".into()),
            repo_category: HashMap::from([("remote".into(), prefix.into())]),
        };
        UserConfig {
            default_account: None,
            default_debug: None,
            top_level_repo: Some(tl),
            accounts: HashMap::new(),
        }
    }

    fn cfg_top_level_local(parent: &str) -> UserConfig {
        let tl = AccountConfig {
            repo_default: Some("local".into()),
            repo_category: HashMap::from([("local".into(), parent.into())]),
        };
        UserConfig {
            default_account: None,
            default_debug: None,
            top_level_repo: Some(tl),
            accounts: HashMap::new(),
        }
    }

    fn cfg_two_accounts() -> UserConfig {
        let home = AccountConfig {
            repo_default: Some("remote".into()),
            repo_category: HashMap::from([
                ("remote".into(), "git@github.com:winksaville".into()),
                ("local".into(), "/tmp/home-fixtures".into()),
            ]),
        };
        let work = AccountConfig {
            repo_default: Some("remote".into()),
            repo_category: HashMap::from([
                ("remote".into(), "git@github.com:anthropic".into()),
                ("local".into(), "/work/fixtures".into()),
            ]),
        };
        UserConfig {
            default_account: Some("home".into()),
            default_debug: None,
            top_level_repo: None,
            accounts: HashMap::from([("home".into(), home), ("work".into(), work)]),
        }
    }

    // ---------- URL TARGET ----------

    #[test]
    fn plan_url_ssh_github_dual() {
        let args = args_for("git@github.com:winksaville/tf1");
        let plan = plan_init(&args, &cfg_empty()).unwrap();
        assert_eq!(plan.provisioner, Provisioner::GhCreate);
        assert_eq!(plan.name, "tf1");
        assert_eq!(plan.code_url, "git@github.com:winksaville/tf1.git");
        assert_eq!(
            plan.session_url.as_deref(),
            Some("git@github.com:winksaville/tf1.claude.git")
        );
        assert_eq!(plan.gh_code_slug.as_deref(), Some("winksaville/tf1"));
        assert_eq!(
            plan.gh_session_slug.as_deref(),
            Some("winksaville/tf1.claude")
        );
    }

    #[test]
    fn plan_url_https_github() {
        let args = args_for("https://github.com/owner/repo.git");
        let plan = plan_init(&args, &cfg_empty()).unwrap();
        assert_eq!(plan.provisioner, Provisioner::GhCreate);
        assert_eq!(plan.code_url, "https://github.com/owner/repo.git");
        assert_eq!(plan.gh_code_slug.as_deref(), Some("owner/repo"));
    }

    #[test]
    fn plan_url_non_github_uses_external_provisioner() {
        let args = args_for("git@gitlab.com:winksaville/tf1.git");
        let plan = plan_init(&args, &cfg_empty()).unwrap();
        assert_eq!(plan.provisioner, Provisioner::ExternalPreExisting);
        assert!(plan.gh_code_slug.is_none());
        assert!(plan.gh_session_slug.is_none());
        assert_eq!(plan.code_url, "git@gitlab.com:winksaville/tf1.git");
        assert_eq!(
            plan.session_url.as_deref(),
            Some("git@gitlab.com:winksaville/tf1.claude.git")
        );
    }

    #[test]
    fn plan_url_with_name_override() {
        // [NAME] overrides the URL-derived dir name; remote URL
        // itself is unchanged.
        let mut args = args_for("git@github.com:winksaville/tf1");
        args.name = Some("custom-dir".into());
        let plan = plan_init(&args, &cfg_empty()).unwrap();
        assert_eq!(plan.name, "custom-dir");
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(plan.project_dir, cwd.join("custom-dir"));
        assert_eq!(plan.code_url, "git@github.com:winksaville/tf1.git");
    }

    // ---------- owner/name shorthand TARGET ----------

    #[test]
    fn plan_owner_name_resolves_to_github_ssh() {
        let args = args_for("winksaville/tf1");
        let plan = plan_init(&args, &cfg_empty()).unwrap();
        assert_eq!(plan.provisioner, Provisioner::GhCreate);
        assert_eq!(plan.code_url, "git@github.com:winksaville/tf1.git");
        assert_eq!(plan.gh_code_slug.as_deref(), Some("winksaville/tf1"));
    }

    #[test]
    fn plan_owner_name_eq_ssh_url_form() {
        // owner/name shorthand must produce the same plan as the
        // explicit SSH URL it resolves to.
        let p1 = plan_init(&args_for("winksaville/tf1"), &cfg_empty()).unwrap();
        let p2 = plan_init(&args_for("git@github.com:winksaville/tf1"), &cfg_empty()).unwrap();
        assert_eq!(p1.code_url, p2.code_url);
        assert_eq!(p1.session_url, p2.session_url);
        assert_eq!(p1.provisioner, p2.provisioner);
        assert_eq!(p1.gh_code_slug, p2.gh_code_slug);
        assert_eq!(p1.gh_session_slug, p2.gh_session_slug);
    }

    // ---------- Path TARGET ----------

    #[test]
    fn plan_path_absolute_with_repo_local() {
        let mut args = args_for("/tmp/xyz/tf1");
        args.repo = Some(RepoSelector {
            category: "local".into(),
            value: Some("/tmp/xyz".into()),
        });
        let plan = plan_init(&args, &cfg_empty()).unwrap();
        assert_eq!(plan.provisioner, Provisioner::LocalBareInit);
        assert_eq!(plan.project_dir, PathBuf::from("/tmp/xyz/tf1"));
        assert_eq!(plan.name, "tf1");
        assert_eq!(
            plan.code_bare_path,
            Some(PathBuf::from("/tmp/xyz/remote-code.git"))
        );
        assert_eq!(
            plan.session_bare_path,
            Some(PathBuf::from("/tmp/xyz/remote-claude.git"))
        );
        assert_eq!(
            plan.session_dir,
            Some(PathBuf::from("/tmp/xyz/tf1/.claude"))
        );
        assert_eq!(plan.session_name.as_deref(), Some("tf1.claude"));
    }

    #[test]
    fn plan_path_relative_with_repo_local() {
        let mut args = args_for("./tf1");
        args.repo = Some(RepoSelector {
            category: "local".into(),
            value: Some("/tmp/xyz".into()),
        });
        let plan = plan_init(&args, &cfg_empty()).unwrap();
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(plan.project_dir, cwd.join("tf1"));
        assert_eq!(plan.name, "tf1");
        assert_eq!(plan.provisioner, Provisioner::LocalBareInit);
    }

    // ---------- Bare-NAME TARGET ----------

    #[test]
    fn plan_bare_name_uses_top_level_repo_remote() {
        let args = args_for("tf1");
        let cfg = cfg_top_level_remote("git@github.com:winksaville");
        let plan = plan_init(&args, &cfg).unwrap();
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(plan.project_dir, cwd.join("tf1"));
        assert_eq!(plan.name, "tf1");
        assert_eq!(plan.code_url, "git@github.com:winksaville/tf1.git");
        assert_eq!(plan.provisioner, Provisioner::GhCreate);
        assert_eq!(plan.gh_code_slug.as_deref(), Some("winksaville/tf1"));
    }

    #[test]
    fn plan_bare_name_uses_top_level_repo_local() {
        let args = args_for("tf1");
        let cfg = cfg_top_level_local("/tmp/fixtures");
        let plan = plan_init(&args, &cfg).unwrap();
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(plan.project_dir, cwd.join("tf1"));
        assert_eq!(plan.provisioner, Provisioner::LocalBareInit);
        assert_eq!(
            plan.code_bare_path,
            Some(PathBuf::from("/tmp/fixtures/remote-code.git"))
        );
        assert_eq!(
            plan.session_bare_path,
            Some(PathBuf::from("/tmp/fixtures/remote-claude.git"))
        );
    }

    #[test]
    fn plan_bare_name_account_override_picks_work() {
        let mut args = args_for("tf1");
        args.account = Some("work".into());
        let plan = plan_init(&args, &cfg_two_accounts()).unwrap();
        assert_eq!(plan.code_url, "git@github.com:anthropic/tf1.git");
        assert_eq!(plan.gh_code_slug.as_deref(), Some("anthropic/tf1"));
    }

    #[test]
    fn plan_bare_name_explicit_repo_value_skips_config() {
        // --repo cat=val short-circuits resolve_repo; works even
        // with an empty config.
        let mut args = args_for("tf1");
        args.repo = Some(RepoSelector {
            category: "local".into(),
            value: Some("/tmp/explicit".into()),
        });
        let plan = plan_init(&args, &cfg_empty()).unwrap();
        assert_eq!(plan.provisioner, Provisioner::LocalBareInit);
        assert_eq!(
            plan.code_bare_path,
            Some(PathBuf::from("/tmp/explicit/remote-code.git"))
        );
    }

    // ---------- Scope (POR vs CodeBot) ----------

    #[test]
    fn plan_por_path_local_single_bare() {
        let mut args = args_for("/tmp/xyz/tf1");
        args.scope = ScopeKind::Por;
        args.repo = Some(RepoSelector {
            category: "local".into(),
            value: Some("/tmp/xyz".into()),
        });
        let plan = plan_init(&args, &cfg_empty()).unwrap();
        assert!(plan.scope.is_code_only());
        assert_eq!(plan.provisioner, Provisioner::LocalBareInit);
        assert_eq!(
            plan.code_bare_path,
            Some(PathBuf::from("/tmp/xyz/remote.git"))
        );
        assert_eq!(plan.code_url, "/tmp/xyz/remote.git");
        assert!(plan.session_bare_path.is_none());
        assert!(plan.session_url.is_none());
        assert!(plan.session_dir.is_none());
        assert!(plan.session_name.is_none());
    }

    #[test]
    fn plan_por_url_no_session() {
        let mut args = args_for("git@github.com:winksaville/tf1");
        args.scope = ScopeKind::Por;
        let plan = plan_init(&args, &cfg_empty()).unwrap();
        assert!(plan.scope.is_code_only());
        assert_eq!(plan.code_url, "git@github.com:winksaville/tf1.git");
        assert!(plan.session_url.is_none());
        assert!(plan.gh_session_slug.is_none());
    }

    #[test]
    fn plan_default_scope_is_code_bot() {
        let args = args_for("git@github.com:winksaville/tf1");
        let plan = plan_init(&args, &cfg_empty()).unwrap();
        assert!(plan.scope.is_both());
        assert!(plan.session_url.is_some());
        assert!(plan.session_dir.is_some());
    }

    // ---------- Errors ----------

    #[test]
    fn error_url_target_with_account() {
        let mut args = args_for("git@github.com:u/p");
        args.account = Some("work".into());
        let err = plan_init(&args, &cfg_empty()).unwrap_err().to_string();
        assert!(err.contains("--account is meaningless"), "got: {err}");
    }

    #[test]
    fn error_url_target_with_repo() {
        let mut args = args_for("git@github.com:u/p");
        args.repo = Some(RepoSelector {
            category: "remote".into(),
            value: Some("git@github.com:other".into()),
        });
        let err = plan_init(&args, &cfg_empty()).unwrap_err().to_string();
        assert!(err.contains("--repo is meaningless"), "got: {err}");
    }

    #[test]
    fn error_path_target_with_name() {
        let mut args = args_for("./tf1");
        args.name = Some("custom".into());
        args.repo = Some(RepoSelector {
            category: "local".into(),
            value: Some("/tmp/xyz".into()),
        });
        let err = plan_init(&args, &cfg_empty()).unwrap_err().to_string();
        assert!(err.contains("[NAME] is meaningless"), "got: {err}");
        assert!(err.contains("path"), "got: {err}");
    }

    #[test]
    fn error_bare_name_target_with_name() {
        let mut args = args_for("tf1");
        args.name = Some("custom".into());
        args.repo = Some(RepoSelector {
            category: "local".into(),
            value: Some("/tmp/xyz".into()),
        });
        let err = plan_init(&args, &cfg_empty()).unwrap_err().to_string();
        assert!(err.contains("[NAME] is meaningless"), "got: {err}");
        assert!(err.contains("bare-NAME"), "got: {err}");
    }

    #[test]
    fn error_bare_name_no_config() {
        // Empty config + no --repo, no --account → step 1 of
        // resolve_repo errors with the "no account" message.
        let args = args_for("tf1");
        let err = plan_init(&args, &cfg_empty()).unwrap_err().to_string();
        assert!(err.contains("[default].account"), "got: {err}");
    }

    #[test]
    fn error_unknown_category() {
        let mut args = args_for("tf1");
        args.repo = Some(RepoSelector {
            category: "weird".into(),
            value: Some("xyz".into()),
        });
        let err = plan_init(&args, &cfg_empty()).unwrap_err().to_string();
        assert!(err.contains("'weird' is not recognized"), "got: {err}");
    }

    #[test]
    fn error_por_with_comma_template() {
        // --scope=por + --use-template foo,bar is ambiguous — bot
        // half has no home in a single-repo workspace.
        let mut args = args_for("git@github.com:u/p");
        args.scope = ScopeKind::Por;
        args.use_template = Some("/tmp/code,/tmp/bot".into());
        let err = plan_init(&args, &cfg_empty()).unwrap_err().to_string();
        assert!(err.contains("--scope=por"), "got: {err}");
        assert!(err.contains("single template path"), "got: {err}");
    }

    // ---------- Constants ----------

    #[test]
    fn config_content_app_only() {
        assert!(VC_CONFIG_APP_ONLY.contains("path = \"/\""));
        assert!(!VC_CONFIG_APP_ONLY.contains("other-repo"));
    }

    #[test]
    fn gitignore_app_only_omits_claude() {
        assert!(!GITIGNORE_APP_ONLY.contains("/.claude"));
        assert!(GITIGNORE_APP_ONLY.contains("/.git"));
        assert!(GITIGNORE_APP_ONLY.contains("/.jj"));
        assert!(GITIGNORE_APP_ONLY.contains("/.vc-x1"));
    }

    // ---------- POR end-to-end fixture (drives init_with_symlink with --scope=por) ----------

    /// POR fixture builds without panic and lays down the
    /// single-repo tree: `<base>/work/` exists, no `.claude/`
    /// peer, bare origin sits at `<base>/remote.git` (not the
    /// dual `remote-code.git` / `remote-claude.git` pair).
    #[test]
    fn por_fixture_creates_single_repo_layout() {
        let fx = crate::test_helpers::FixturePor::new("por-layout");

        assert!(fx.work.exists(), "work dir should exist");
        assert!(fx.work.is_dir(), "work should be a directory");
        assert!(
            !fx.work.join(".claude").exists(),
            "POR layout must not have a .claude/ peer"
        );
        assert!(
            fx.base.join("remote.git").exists(),
            "POR uses <base>/remote.git as the bare origin"
        );
        assert!(
            !fx.base.join("remote-code.git").exists(),
            "dual-shape bares should be absent in POR"
        );
        assert!(
            !fx.base.join("remote-claude.git").exists(),
            "dual-shape bares should be absent in POR"
        );
    }

    /// POR fixture writes the APP_ONLY config + .gitignore variants
    /// — `path = "/"` with no `other-repo` field, and `.gitignore`
    /// has no `/.claude` exclusion.
    #[test]
    fn por_fixture_writes_app_only_config_files() {
        let fx = crate::test_helpers::FixturePor::new("por-config");

        let cfg =
            std::fs::read_to_string(fx.work.join(".vc-config.toml")).expect("read .vc-config.toml");
        assert!(cfg.contains("path = \"/\""), "expected POR path = \"/\"");
        assert!(
            !cfg.contains("other-repo"),
            "POR config must not reference other-repo"
        );

        let gi = std::fs::read_to_string(fx.work.join(".gitignore")).expect("read .gitignore");
        assert!(
            !gi.contains("/.claude"),
            "POR .gitignore must not exclude /.claude"
        );
        assert!(gi.contains("/.git"), "expected /.git entry");
        assert!(gi.contains("/.jj"), "expected /.jj entry");
    }

    /// POR fixture has a `main` bookmark tracking `origin/main` —
    /// pins step 10 (re-init jj + bookmark track) ran successfully.
    #[test]
    fn por_fixture_main_tracks_origin() {
        let fx = crate::test_helpers::FixturePor::new("por-tracking");

        crate::common::verify_tracking(&fx.work, "main")
            .expect("main should track origin/main after init step 10");
    }
}

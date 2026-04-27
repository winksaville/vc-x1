use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info};

use crate::scope::{Scope, Side};
use crate::symlink;

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Project name (directory and GitHub repo name).
    ///
    /// - Optional with `--repo-remote` (derived from URL's last
    ///   path segment, trailing `.git` stripped).
    /// - Required with `--repo-local` and in default mode.
    /// - Fatal if both given and they disagree.
    #[arg(value_name = "NAME", verbatim_doc_comment)]
    pub name: Option<String>,

    /// GitHub user or organization.
    ///
    /// - Only meaningful in default mode.
    /// - Fatal with `--repo-local` or `--repo-remote`.
    #[arg(long, verbatim_doc_comment)]
    pub owner: Option<String>,

    /// Parent directory to create the project under [default: cwd].
    ///
    /// - Fatal with `--repo-local` (that flag supplies the parent).
    #[arg(long, verbatim_doc_comment)]
    pub dir: Option<PathBuf>,

    /// Create private GitHub repos (default: public).
    ///
    /// - Only meaningful in default mode.
    /// - Fatal with `--repo-local` or `--repo-remote`.
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
    /// - With `--scope=code`: only `CODE` is used; passing `,BOT`
    ///   is fatal (no session side to seed).
    /// - Non-hidden contents copied recursively; hidden entries
    ///   (names starting with `.`) are skipped — init writes its
    ///   own hidden files.
    /// - If a copied tree has a `README.md`, its first line is
    ///   rewritten to `# <repo-name>`.
    #[arg(long, value_name = "CODE[,BOT]", verbatim_doc_comment)]
    pub use_template: Option<String>,

    /// Fixture mode — fully self-contained local workspace.
    ///
    /// Value is a single parent directory.
    ///
    /// - Project + session at `<PARENT>/<NAME>/` and
    ///   `<PARENT>/<NAME>/.claude/`.
    /// - Bare origins at `<PARENT>/remote-code.git` and
    ///   `<PARENT>/remote-claude.git`.
    /// - No `gh`, no network.
    /// - Supports `~/…`, `$VAR`, `${VAR}`; relative paths against cwd.
    /// - Exclusive with `--repo-remote`, `--dir`, `--owner`, `--private`.
    #[arg(long, value_name = "PARENT", verbatim_doc_comment)]
    pub repo_local: Option<String>,

    /// Override the `origin` URL stem (single value).
    ///
    /// - Session URL derived: insert `.claude` before `.git` suffix,
    ///   or append if no `.git`.
    /// - GitHub URLs → `gh repo create` (same as default).
    /// - Other URLs → caller pre-creates the remote; init only pushes.
    /// - Supports `~/…`, `$VAR`, `${VAR}` in path-shaped values.
    /// - Exclusive with `--repo-local`, `--owner`, `--private`.
    #[arg(long, value_name = "URL", verbatim_doc_comment)]
    pub repo_remote: Option<String>,

    /// Which repo(s) of the workspace to create.
    ///
    /// `SCOPE=code|bot|code,bot`:
    ///
    /// - `code` — single-repo workspace, no `.claude/` session.
    /// - `bot` — fatal at init time (meaningless without code).
    /// - `code,bot` (default) — dual-repo workspace.
    #[arg(
        long,
        value_name = "SCOPE",
        value_delimiter = ',',
        verbatim_doc_comment
    )]
    pub scope: Option<Vec<Side>>,
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

/// Get the current GitHub user via `gh api user`.
fn gh_whoami() -> Result<String, Box<dyn std::error::Error>> {
    run("gh", &["api", "user", "--jq", ".login"], Path::new("."))
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

/// Normalize a `--repo-remote` value. Expand variables, then for
/// paths make absolute; URLs pass through after expansion.
pub(crate) fn normalize_remote(s: &str) -> Result<String, Box<dyn std::error::Error>> {
    let s = s.trim();
    if s.is_empty() {
        return Err("--repo-remote: value is empty".into());
    }
    let expanded = expand_vars(s)?;
    if is_remote_url(&expanded) {
        return Ok(expanded);
    }
    let p = PathBuf::from(&expanded);
    if p.is_absolute() {
        Ok(expanded)
    } else {
        let cwd = std::env::current_dir()?;
        Ok(cwd.join(p).to_string_lossy().to_string())
    }
}

/// Normalize a `--repo-local` value: expand variables, make
/// absolute. Parent doesn't need to exist yet — init creates it.
pub(crate) fn normalize_local_parent(s: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let s = s.trim();
    if s.is_empty() {
        return Err("--repo-local: value is empty".into());
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

/// Derive the session URL from the code URL.
///
/// - With trailing `.git`: insert `.claude` before it
///   (`foo.git` → `foo.claude.git`).
/// - Without `.git`: append `.claude`.
pub(crate) fn derive_session_url(code_url: &str) -> String {
    match code_url.strip_suffix(".git") {
        Some(stem) => format!("{stem}.claude.git"),
        None => format!("{code_url}.claude"),
    }
}

/// Derive the project NAME from a URL.
///
/// - Last path segment, trailing `.git` stripped.
/// - Segment boundary is the rightmost `/` or `:` (scp-like SSH).
pub(crate) fn derive_name_from_url(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let stem = url.strip_suffix(".git").unwrap_or(url);
    let last = stem.rsplit(['/', ':']).next().unwrap_or(""); // OK: rsplit always yields at least one element
    if last.is_empty() {
        return Err(format!("cannot derive project name from URL '{url}'").into());
    }
    Ok(last.to_string())
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

/// Build an `InitPlan` from CLI args.
///
/// - Validate flag mutual-exclusion.
/// - Resolve URLs and paths; derive NAME from URL if not given.
/// - Pick the provisioner.
/// - Errors on any ambiguous or incomplete flag combination.
pub(crate) fn plan_init(args: &InitArgs) -> Result<InitPlan, Box<dyn std::error::Error>> {
    // --- Resolve scope first: default (None) → dual; empty Vec is
    // invalid; `bot` alone is meaningless at init time. ---
    let scope = match &args.scope {
        None => Scope::Roles(vec![Side::Code, Side::Bot]),
        Some(sides) => {
            let s = Scope::Roles(sides.clone());
            if s.is_empty() {
                return Err("--scope: value is empty".into());
            }
            if s.is_bot_only() {
                return Err(
                    "--scope=bot is meaningless at init time — use --scope=code for single-repo or --scope=code,bot (default) for dual-repo"
                        .into(),
                );
            }
            s
        }
    };

    // --- Mutual-exclusion gates (flag-pair ambiguity) ---
    if args.repo_local.is_some() && args.repo_remote.is_some() {
        return Err("--repo-local and --repo-remote are mutually exclusive".into());
    }
    if args.repo_local.is_some() && args.dir.is_some() {
        return Err(
            "--repo-local and --dir are mutually exclusive (--repo-local is the parent)".into(),
        );
    }
    if args.repo_local.is_some() && args.owner.is_some() {
        return Err(
            "--repo-local and --owner are mutually exclusive (fixture mode has no GitHub)".into(),
        );
    }
    if args.repo_local.is_some() && args.private {
        return Err(
            "--repo-local and --private are mutually exclusive (fixture mode has no GitHub)".into(),
        );
    }
    if args.repo_remote.is_some() && args.owner.is_some() {
        return Err(
            "--repo-remote and --owner are mutually exclusive (URL already encodes owner)".into(),
        );
    }
    if args.repo_remote.is_some() && args.private {
        return Err(
            "--repo-remote and --private are mutually exclusive (visibility is set at repo creation, not here)"
                .into(),
        );
    }

    // --- Scope-dependent ambiguity. ---
    if scope.is_code_only()
        && let Some(t) = &args.use_template
        && t.contains(',')
    {
        return Err(format!(
            "--scope=code takes a single template path; got '{t}' (drop the `,BOT` half)"
        )
        .into());
    }

    match (&args.repo_local, &args.repo_remote) {
        (Some(parent_spec), None) => plan_local_fixture(args, parent_spec, scope),
        (None, Some(url_spec)) => plan_external_remote(args, url_spec, scope),
        (None, None) => plan_default_github(args, scope),
        #[allow(clippy::unwrap_used)]
        (Some(_), Some(_)) => {
            // OK: mutual-exclusion gate above already errored on this case
            unreachable!("mutual-exclusion gate earlier in plan_init")
        }
    }
}

/// Plan for `--repo-local PARENT` — fixture mode.
///
/// - Code-only: single bare at `<PARENT>/remote.git`, no session.
/// - Dual: `remote-code.git` + `remote-claude.git` plus a nested
///   `.claude/` session.
fn plan_local_fixture(
    args: &InitArgs,
    parent_spec: &str,
    scope: Scope,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    let name = args
        .name
        .clone()
        .ok_or("NAME is required with --repo-local")?;
    let parent = normalize_local_parent(parent_spec)?;
    let project_dir = parent.join(&name);

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

    // Dual: code + bot sides, nested session.
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

/// Plan for `--repo-remote URL` — explicit URL override.
///
/// - Session URL derived (dual only).
/// - NAME derived from URL when not given (must agree if both).
/// - GitHub URLs → `GhCreate`; other URLs → `ExternalPreExisting`.
fn plan_external_remote(
    args: &InitArgs,
    url_spec: &str,
    scope: Scope,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    let normalized = normalize_remote(url_spec)?;
    let code_url = ensure_git_suffix(&normalized);
    let url_name = derive_name_from_url(&code_url)?;

    let name = match &args.name {
        Some(n) if *n == url_name => n.clone(),
        Some(n) => {
            return Err(format!(
                "NAME '{n}' disagrees with --repo-remote URL '{url_spec}' (derived '{url_name}')"
            )
            .into());
        }
        None => url_name.clone(),
    };

    let parent_dir = match &args.dir {
        Some(d) => d.clone(),
        None => std::env::current_dir()?,
    };
    let project_dir = parent_dir.join(&name);

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

    // Dual.
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

/// Plan for default mode (no `--repo-local` / `--repo-remote`).
///
/// - NAME required.
/// - `owner` derived via `gh whoami` unless `--owner` given.
/// - Provisioner is always `GhCreate`.
fn plan_default_github(
    args: &InitArgs,
    scope: Scope,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    let name = args
        .name
        .clone()
        .ok_or("NAME is required (or pass --repo-remote URL)")?;

    let owner = match &args.owner {
        Some(o) => o.clone(),
        None => gh_whoami()?,
    };

    let code_url = format!("git@github.com:{owner}/{name}.git");
    let gh_code_slug = format!("{owner}/{name}");

    let parent_dir = match &args.dir {
        Some(d) => d.clone(),
        None => std::env::current_dir()?,
    };
    let project_dir = parent_dir.join(&name);

    if scope.is_code_only() {
        return Ok(InitPlan {
            scope,
            project_dir,
            name,
            code_url,
            provisioner: Provisioner::GhCreate,
            code_bare_path: None,
            gh_code_slug: Some(gh_code_slug),
            session_dir: None,
            session_name: None,
            session_url: None,
            session_bare_path: None,
            gh_session_slug: None,
        });
    }

    let session_name = format!("{name}.claude");
    let session_url = format!("git@github.com:{owner}/{session_name}.git");
    let gh_session_slug = format!("{owner}/{session_name}");
    let session_dir = project_dir.join(".claude");

    Ok(InitPlan {
        scope,
        project_dir,
        name,
        code_url,
        provisioner: Provisioner::GhCreate,
        code_bare_path: None,
        gh_code_slug: Some(gh_code_slug),
        session_url: Some(session_url),
        session_name: Some(session_name),
        session_dir: Some(session_dir),
        session_bare_path: None,
        gh_session_slug: Some(gh_session_slug),
    })
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

    let plan = plan_init(args)?;
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

    // Step 1: Create directories. For LocalBareInit also create the
    // parent so the bare-repo init (step 8/9) has a home.
    info!("Step 1: Creating project directories...");
    if let Some(parent) = plan.project_dir.parent() {
        mkdir_p(parent)?;
    }
    mkdir_p(&plan.project_dir)?;
    if let Some(session_dir) = plan.session_dir.as_ref() {
        mkdir_p(session_dir)?;
    }

    // Step 2: git init + jj init
    info!("Step 2: Initializing repos...");
    debug!("seed project_dir with an empty git repo");
    run("git", &["init"], &plan.project_dir)?;
    debug!("colocate jj atop the git repo so jj + git share .git state");
    run("jj", &["git", "init", "--colocate"], &plan.project_dir)?;
    if let Some(session_dir) = plan.session_dir.as_ref() {
        debug!("seed session_dir with an empty git repo");
        run("git", &["init"], session_dir)?;
        debug!("colocate jj atop the session git repo");
        run("jj", &["git", "init", "--colocate"], session_dir)?;
    }

    // Step 3: Write config files
    info!("Step 3: Writing config files...");
    if is_dual {
        write_file(&plan.project_dir.join(".vc-config.toml"), VC_CONFIG_CODE)?;
        write_file(&plan.project_dir.join(".gitignore"), GITIGNORE_CODE)?;
    } else {
        write_file(
            &plan.project_dir.join(".vc-config.toml"),
            VC_CONFIG_APP_ONLY,
        )?;
        write_file(&plan.project_dir.join(".gitignore"), GITIGNORE_APP_ONLY)?;
    }
    if let Some(session_dir) = plan.session_dir.as_ref() {
        write_file(&session_dir.join(".vc-config.toml"), VC_CONFIG_SESSION)?;
        write_file(&session_dir.join(".gitignore"), GITIGNORE_SESSION)?;
    }

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
        if let (Some(bot_t), Some(session_dir), Some(session_name)) = (
            bot_t.as_ref(),
            plan.session_dir.as_ref(),
            plan.session_name.as_ref(),
        ) {
            info!("  bot:  {} -> {}", bot_t.display(), session_dir.display());
            copy_template_recursive(bot_t, session_dir)?;
            rewrite_readme_first_line(session_dir, session_name)?;
        }
    }

    // Step 5: jj commit
    if is_dual {
        info!("Step 5: Committing both repos with placeholder ochids...");
        #[allow(clippy::unwrap_used)]
        let session_dir = plan.session_dir.as_ref().unwrap(); // OK: is_dual ⇒ session_dir set
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
    } else {
        info!("Step 5: Committing code...");
        debug!("code side: initial commit (no ochid in single-repo)");
        run("jj", &["commit", "-m", "Initial commit"], &plan.project_dir)?;
    }

    // Step 6: ochid cross-references (dual only)
    if is_dual {
        info!("Step 6: Setting ochid cross-references...");
        #[allow(clippy::unwrap_used)]
        let session_dir = plan.session_dir.as_ref().unwrap(); // OK: is_dual ⇒ session_dir set
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
    } else {
        info!("Step 6: (skipped — no cross-reference in single-repo)");
        let code_chid = jj_chid("@-", &plan.project_dir)?;
        let hash = run("git", &["rev-parse", "HEAD"], &plan.project_dir)?;
        debug!("code repo: chid={code_chid} hash={hash}");
    }

    // Step 7: Set bookmarks (creates git branches), then remove jj
    info!("Step 7: Setting bookmarks and removing jj...");
    debug!("place code-side main bookmark at the initial commit");
    run(
        "jj",
        &["bookmark", "set", "main", "-r", "@-"],
        &plan.project_dir,
    )?;
    if let Some(session_dir) = plan.session_dir.as_ref() {
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
    } else {
        debug!("strip jj state from code side (no .claude in single-repo)");
        run("git", &["clean", "-xdf"], &plan.project_dir)?;
    }
    debug!("re-attach code HEAD to main after clean");
    run("git", &["checkout", "main"], &plan.project_dir)?;

    // Steps 8/9: provision remotes and push.
    if let Some(session_dir) = plan.session_dir.as_ref() {
        #[allow(clippy::unwrap_used)]
        let session_url = plan.session_url.as_deref().unwrap(); // OK: is_dual ⇒ session_url set
        run_remote_step(
            "Step 8",
            "session",
            &plan,
            session_url,
            plan.gh_session_slug.as_deref(),
            plan.session_bare_path.as_deref(),
            visibility,
            session_dir,
            args,
        )?;
    } else {
        info!("Step 8: (skipped — no session side in single-repo)");
    }
    run_remote_step(
        "Step 9",
        "code",
        &plan,
        &plan.code_url,
        plan.gh_code_slug.as_deref(),
        plan.code_bare_path.as_deref(),
        visibility,
        &plan.project_dir,
        args,
    )?;

    // Step 10: Re-initialize jj
    info!(
        "Step 10: Re-initializing jj on {}...",
        if is_dual {
            "both repos"
        } else {
            "the code repo"
        }
    );
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

    let session_chid_final = if let Some(session_dir) = plan.session_dir.as_ref() {
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
        Some(jj_chid("@-", session_dir)?)
    } else {
        None
    };

    // Step 11: Create Claude Code symlink (dual only; opt-out for tests)
    let sl_opt = if is_dual && create_symlink {
        info!("Step 11: Creating Claude Code symlink...");
        let symlink_dir = {
            let home = std::env::var("HOME")
                .map_err(|_| "HOME environment variable not set".to_string())?;
            PathBuf::from(home).join(".claude").join("projects")
        };

        let sl = symlink::SymLink::new(&plan.project_dir, Path::new(".claude"), &symlink_dir)?;
        sl.create(false)?;
        Some(sl)
    } else if !is_dual {
        info!("Step 11: (skipped — no .claude symlink in single-repo)");
        None
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
    if let (Some(session_url), Some(session_chid)) =
        (plan.session_url.as_ref(), session_chid_final.as_ref())
    {
        info!("  Session repo: {session_url}  (chid={session_chid})");
    }
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
        let args = parse(&["vc-x1", "init", "my-project"]);
        assert_eq!(args.name.as_deref(), Some("my-project"));
        assert!(args.owner.is_none());
        assert!(args.dir.is_none());
        assert!(!args.private);
        assert!(!args.dry_run);
        assert_eq!(args.push_retries, 5);
        assert_eq!(args.push_retry_delay, 3);
        assert!(args.repo_local.is_none());
        assert!(args.repo_remote.is_none());
    }

    #[test]
    fn all_opts() {
        let args = parse(&[
            "vc-x1",
            "init",
            "my-project",
            "--owner",
            "myorg",
            "--dir",
            "/tmp/projects",
            "--private",
            "--dry-run",
            "--push-retries",
            "10",
            "--push-retry-delay",
            "5",
            "--use-template",
            "/tmp/tmpl,/tmp/tmpl.claude",
        ]);
        assert_eq!(args.name.as_deref(), Some("my-project"));
        assert_eq!(args.owner.as_deref(), Some("myorg"));
        assert_eq!(args.dir, Some(PathBuf::from("/tmp/projects")));
        assert!(args.private);
        assert!(args.dry_run);
        assert_eq!(args.push_retries, 10);
        assert_eq!(args.push_retry_delay, 5);
        assert_eq!(
            args.use_template.as_deref(),
            Some("/tmp/tmpl,/tmp/tmpl.claude")
        );
    }

    #[test]
    fn name_optional_at_parse_time() {
        // Clap accepts missing NAME; the dispatcher errors at runtime
        // if neither NAME nor --repo-remote provides one.
        let cli = Cli::try_parse_from(["vc-x1", "init"]);
        assert!(cli.is_ok());
        let InitArgs { name, .. } = match cli.unwrap().command {
            Commands::Init(a) => a,
            _ => panic!("expected Init"),
        };
        assert!(name.is_none());
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

    // ---------- 0.40.0-1: --repo-local / --repo-remote helpers ----------

    #[test]
    fn repo_local_parses() {
        let args = parse(&["vc-x1", "init", "tf1", "--repo-local", "/tmp/xyz"]);
        assert_eq!(args.repo_local.as_deref(), Some("/tmp/xyz"));
    }

    #[test]
    fn repo_remote_parses() {
        let args = parse(&[
            "vc-x1",
            "init",
            "tf1",
            "--repo-remote",
            "git@github.com:winksaville/tf1",
        ]);
        assert_eq!(
            args.repo_remote.as_deref(),
            Some("git@github.com:winksaville/tf1")
        );
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

    #[test]
    fn derive_session_url_inserts_claude_before_git() {
        assert_eq!(
            derive_session_url("git@github.com:u/p.git"),
            "git@github.com:u/p.claude.git"
        );
        assert_eq!(derive_session_url("/tmp/foo.git"), "/tmp/foo.claude.git");
        // Without .git suffix, falls back to plain append.
        assert_eq!(derive_session_url("/tmp/foo"), "/tmp/foo.claude");
    }

    #[test]
    fn derive_name_from_url_strips_git_and_takes_last_segment() {
        assert_eq!(
            derive_name_from_url("git@github.com:winksaville/tf1").unwrap(),
            "tf1"
        );
        assert_eq!(
            derive_name_from_url("git@github.com:winksaville/tf1.git").unwrap(),
            "tf1"
        );
        assert_eq!(
            derive_name_from_url("https://github.com/winksaville/tf1.git").unwrap(),
            "tf1"
        );
        assert_eq!(derive_name_from_url("/tmp/foo.git").unwrap(), "foo");
        assert_eq!(derive_name_from_url("/tmp/foo").unwrap(), "foo");
    }

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

    #[test]
    fn normalize_remote_passthrough_url() {
        let s = "git@github.com:u/p.git";
        assert_eq!(normalize_remote(s).unwrap(), s);
    }

    #[test]
    fn normalize_remote_canonicalizes_relative_path() {
        let cwd = std::env::current_dir().unwrap();
        let got = normalize_remote("./foo.git").unwrap();
        assert!(got.starts_with(&cwd.to_string_lossy().to_string()));
    }

    // ---------- plan_init dispatch + ambiguity ----------

    fn fixture(
        name: Option<&str>,
        repo_local: Option<&str>,
        repo_remote: Option<&str>,
        dir: Option<&str>,
        owner: Option<&str>,
        private: bool,
    ) -> InitArgs {
        fixture_scoped(name, repo_local, repo_remote, dir, owner, private, None)
    }

    fn fixture_scoped(
        name: Option<&str>,
        repo_local: Option<&str>,
        repo_remote: Option<&str>,
        dir: Option<&str>,
        owner: Option<&str>,
        private: bool,
        scope: Option<Vec<Side>>,
    ) -> InitArgs {
        InitArgs {
            name: name.map(str::to_string),
            owner: owner.map(str::to_string),
            dir: dir.map(PathBuf::from),
            private,
            dry_run: true,
            push_retries: 5,
            push_retry_delay: 3,
            use_template: None,
            repo_local: repo_local.map(str::to_string),
            repo_remote: repo_remote.map(str::to_string),
            scope,
        }
    }

    #[test]
    fn plan_local_fixture_layout() {
        let args = fixture(Some("tf1"), Some("/tmp/xyz"), None, None, None, false);
        let plan = plan_init(&args).unwrap();
        assert_eq!(plan.provisioner, Provisioner::LocalBareInit);
        assert_eq!(plan.project_dir, PathBuf::from("/tmp/xyz/tf1"));
        assert_eq!(
            plan.session_dir,
            Some(PathBuf::from("/tmp/xyz/tf1/.claude"))
        );
        assert_eq!(
            plan.code_bare_path,
            Some(PathBuf::from("/tmp/xyz/remote-code.git"))
        );
        assert_eq!(
            plan.session_bare_path,
            Some(PathBuf::from("/tmp/xyz/remote-claude.git"))
        );
        assert_eq!(plan.code_url, "/tmp/xyz/remote-code.git");
        assert_eq!(
            plan.session_url.as_deref(),
            Some("/tmp/xyz/remote-claude.git")
        );
        assert_eq!(plan.name, "tf1");
        assert_eq!(plan.session_name.as_deref(), Some("tf1.claude"));
    }

    #[test]
    fn plan_external_remote_github_with_name() {
        let args = fixture(
            Some("tf1"),
            None,
            Some("git@github.com:winksaville/tf1"),
            None,
            None,
            false,
        );
        let plan = plan_init(&args).unwrap();
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
    fn plan_external_remote_derives_name_from_url() {
        let args = fixture(
            None,
            None,
            Some("git@github.com:winksaville/tf1"),
            None,
            None,
            false,
        );
        let plan = plan_init(&args).unwrap();
        assert_eq!(plan.name, "tf1");
    }

    #[test]
    fn plan_external_remote_name_disagreement_errors() {
        let args = fixture(
            Some("DIFFERENT"),
            None,
            Some("git@github.com:winksaville/tf1"),
            None,
            None,
            false,
        );
        let err = plan_init(&args).unwrap_err().to_string();
        assert!(err.contains("DIFFERENT"));
        assert!(err.contains("tf1"));
    }

    #[test]
    fn plan_external_non_github_url_uses_external_provisioner() {
        let args = fixture(
            Some("tf1"),
            None,
            Some("git@gitlab.com:winksaville/tf1.git"),
            None,
            None,
            false,
        );
        let plan = plan_init(&args).unwrap();
        assert_eq!(plan.provisioner, Provisioner::ExternalPreExisting);
        assert!(plan.gh_code_slug.is_none());
        assert!(plan.gh_session_slug.is_none());
        assert_eq!(plan.code_url, "git@gitlab.com:winksaville/tf1.git");
        assert_eq!(
            plan.session_url.as_deref(),
            Some("git@gitlab.com:winksaville/tf1.claude.git")
        );
    }

    /// **`vc-x1 init tf1 --repo-remote git@github.com:winksaville/tf1` ==
    /// `vc-x1 init tf1 --owner winksaville`**: same resolved URLs +
    /// same provisioner means same end-state init flow.
    #[test]
    fn plan_default_and_explicit_github_url_are_equivalent() {
        let explicit = fixture(
            Some("tf1"),
            None,
            Some("git@github.com:winksaville/tf1"),
            None,
            None,
            false,
        );
        let default_with_owner = fixture(Some("tf1"), None, None, None, Some("winksaville"), false);
        let p1 = plan_init(&explicit).unwrap();
        let p2 = plan_init(&default_with_owner).unwrap();
        assert_eq!(p1.code_url, p2.code_url);
        assert_eq!(p1.session_url, p2.session_url);
        assert_eq!(p1.provisioner, p2.provisioner);
        assert_eq!(p1.gh_code_slug, p2.gh_code_slug);
        assert_eq!(p1.gh_session_slug, p2.gh_session_slug);
    }

    // ---------- Ambiguity rejections ----------

    #[test]
    fn ambig_repo_local_and_repo_remote() {
        let args = fixture(
            Some("tf1"),
            Some("/tmp/xyz"),
            Some("git@github.com:u/tf1"),
            None,
            None,
            false,
        );
        let err = plan_init(&args).unwrap_err().to_string();
        assert!(err.contains("--repo-local"));
        assert!(err.contains("--repo-remote"));
    }

    #[test]
    fn ambig_repo_local_and_dir() {
        let args = fixture(
            Some("tf1"),
            Some("/tmp/xyz"),
            None,
            Some("/tmp/parent"),
            None,
            false,
        );
        let err = plan_init(&args).unwrap_err().to_string();
        assert!(err.contains("--repo-local"));
        assert!(err.contains("--dir"));
    }

    #[test]
    fn ambig_repo_local_and_owner() {
        let args = fixture(
            Some("tf1"),
            Some("/tmp/xyz"),
            None,
            None,
            Some("user"),
            false,
        );
        let err = plan_init(&args).unwrap_err().to_string();
        assert!(err.contains("--repo-local"));
        assert!(err.contains("--owner"));
    }

    #[test]
    fn ambig_repo_local_and_private() {
        let args = fixture(Some("tf1"), Some("/tmp/xyz"), None, None, None, true);
        let err = plan_init(&args).unwrap_err().to_string();
        assert!(err.contains("--repo-local"));
        assert!(err.contains("--private"));
    }

    #[test]
    fn ambig_repo_remote_and_owner() {
        let args = fixture(
            Some("tf1"),
            None,
            Some("git@github.com:u/tf1"),
            None,
            Some("u"),
            false,
        );
        let err = plan_init(&args).unwrap_err().to_string();
        assert!(err.contains("--repo-remote"));
        assert!(err.contains("--owner"));
    }

    #[test]
    fn ambig_repo_remote_and_private() {
        let args = fixture(
            Some("tf1"),
            None,
            Some("git@github.com:u/tf1"),
            None,
            None,
            true,
        );
        let err = plan_init(&args).unwrap_err().to_string();
        assert!(err.contains("--repo-remote"));
        assert!(err.contains("--private"));
    }

    #[test]
    fn repo_local_requires_name() {
        let args = fixture(None, Some("/tmp/xyz"), None, None, None, false);
        let err = plan_init(&args).unwrap_err().to_string();
        assert!(err.contains("NAME"));
    }

    // ---------- 0.40.0-2: --scope=code|bot|code,bot ----------

    #[test]
    fn scope_parses_code() {
        let args = parse(&["vc-x1", "init", "tf1", "--scope", "code"]);
        assert_eq!(args.scope.as_deref(), Some(&[Side::Code][..]));
    }

    #[test]
    fn scope_parses_code_bot() {
        let args = parse(&["vc-x1", "init", "tf1", "--scope", "code,bot"]);
        assert_eq!(args.scope.as_deref(), Some(&[Side::Code, Side::Bot][..]));
    }

    #[test]
    fn scope_bot_only_is_fatal() {
        let args = fixture_scoped(
            Some("tf1"),
            None,
            None,
            None,
            Some("winksaville"),
            false,
            Some(vec![Side::Bot]),
        );
        let err = plan_init(&args).unwrap_err().to_string();
        assert!(err.contains("--scope=bot"));
    }

    #[test]
    fn scope_default_is_dual() {
        // No --scope → default to code+bot; existing behavior unchanged.
        let args = fixture(Some("tf1"), None, None, None, Some("winksaville"), false);
        let plan = plan_init(&args).unwrap();
        assert!(plan.scope.is_both());
        assert!(plan.session_url.is_some());
        assert!(plan.session_name.is_some());
        assert!(plan.session_dir.is_some());
    }

    #[test]
    fn scope_code_only_default_github() {
        let args = fixture_scoped(
            Some("tf1"),
            None,
            None,
            None,
            Some("winksaville"),
            false,
            Some(vec![Side::Code]),
        );
        let plan = plan_init(&args).unwrap();
        assert!(plan.scope.is_code_only());
        assert_eq!(plan.code_url, "git@github.com:winksaville/tf1.git");
        assert_eq!(plan.gh_code_slug.as_deref(), Some("winksaville/tf1"));
        assert!(plan.session_url.is_none());
        assert!(plan.session_name.is_none());
        assert!(plan.session_dir.is_none());
        assert!(plan.gh_session_slug.is_none());
        assert!(plan.session_bare_path.is_none());
    }

    #[test]
    fn scope_code_only_repo_local_single_bare() {
        let args = fixture_scoped(
            Some("tf1"),
            Some("/tmp/xyz"),
            None,
            None,
            None,
            false,
            Some(vec![Side::Code]),
        );
        let plan = plan_init(&args).unwrap();
        assert!(plan.scope.is_code_only());
        assert_eq!(plan.provisioner, Provisioner::LocalBareInit);
        assert_eq!(plan.project_dir, PathBuf::from("/tmp/xyz/tf1"));
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
    fn scope_code_only_repo_remote_github() {
        let args = fixture_scoped(
            Some("tf1"),
            None,
            Some("git@github.com:winksaville/tf1"),
            None,
            None,
            false,
            Some(vec![Side::Code]),
        );
        let plan = plan_init(&args).unwrap();
        assert!(plan.scope.is_code_only());
        assert_eq!(plan.provisioner, Provisioner::GhCreate);
        assert_eq!(plan.code_url, "git@github.com:winksaville/tf1.git");
        assert_eq!(plan.gh_code_slug.as_deref(), Some("winksaville/tf1"));
        assert!(plan.session_url.is_none());
        assert!(plan.gh_session_slug.is_none());
    }

    #[test]
    fn scope_code_only_repo_remote_non_github_skips_create() {
        let args = fixture_scoped(
            Some("tf1"),
            None,
            Some("git@gitlab.com:winksaville/tf1.git"),
            None,
            None,
            false,
            Some(vec![Side::Code]),
        );
        let plan = plan_init(&args).unwrap();
        assert!(plan.scope.is_code_only());
        assert_eq!(plan.provisioner, Provisioner::ExternalPreExisting);
        assert!(plan.session_url.is_none());
    }

    #[test]
    fn scope_code_only_with_comma_template_errors() {
        // `--scope=code` + `--use-template foo,bar` is ambiguous — the
        // bot half has no home.
        let mut args = fixture_scoped(
            Some("tf1"),
            Some("/tmp/xyz"),
            None,
            None,
            None,
            false,
            Some(vec![Side::Code]),
        );
        args.use_template = Some("/tmp/code,/tmp/bot".to_string());
        let err = plan_init(&args).unwrap_err().to_string();
        assert!(err.contains("--scope=code"));
        assert!(err.contains("single template path"));
    }

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
}

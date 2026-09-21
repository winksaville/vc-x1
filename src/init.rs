mod params;
pub use params::InitParams;

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info};

use crate::config::{self, UserConfig};
use crate::context::Context;
use crate::jj;
use crate::options_flags::account::AccountOption;
use crate::options_flags::config::{ConfigKind, ConfigOption};
use crate::options_flags::por::PorFlag;
use crate::options_flags::provision_bundle::ProvisionOptionFlagBundle;
use crate::options_flags::push_retry::PushRetryOptions;
use crate::options_flags::repo::RepoOption;
use crate::options_flags::scope::{Scope, Side};
use crate::options_flags::use_template::UseTemplateOption;
use crate::repo_utils::{OchidStrategy, commit_initial, cross_ref_ochids, prepare_local_repo};
use crate::subcommand::SubcommandRunner;
use crate::symlink;
use crate::url::{Target, derive_name, parse_target};

/// CLI args for `vc-x1 init`.
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Target: URL, path, or bare NAME.
    ///
    /// - URL: `git@host:owner/name(.git)?`, `https://...(.git)?`,
    ///   used as-is, config not consulted.
    /// - Path: `./X`, `../X`, `/X`, `~/X`, `~`, `.`, `..`, is the
    ///   directory path, remote resolved via `--repo` chain.
    /// - Bare NAME: becomes NAME.git, remote resolved via
    ///   `--repo` chain.
    ///
    /// A slashed target with no path prefix (`owner/name`) is
    /// refused: it reads equally as a path and as the retired
    /// owner/name shorthand, so pass `./owner/name` or a URL.
    #[arg(value_name = "TARGET", verbatim_doc_comment)]
    pub target: String,

    /// Repo directory name override (URL form only).
    ///
    /// - URL form: repo created at `cwd/<NAME>` instead of the
    ///   URL-derived name.
    /// - Path / bare-NAME forms: error if given (TARGET already
    ///   names the repo).
    #[arg(value_name = "NAME", verbatim_doc_comment)]
    pub name: Option<String>,

    /// `--account`: flatten of the shared [`AccountOption`] leaf.
    #[command(flatten)]
    pub account: AccountOption,

    /// `--repo`: flatten of the shared [`RepoOption`] leaf.
    /// Init's built-in categories: `remote` (a URL prefix init
    /// appends `/<NAME>.git` to) and `local` (parent dir for
    /// fixture bare repos). Meaningful only with Path or
    /// bare-NAME targets.
    #[command(flatten)]
    pub repo: RepoOption,

    /// `--por`: flatten of the shared [`PorFlag`] leaf. Absent
    /// (default) -> dual workspace (work + `.agent-session/` agent
    /// repo), present -> plain single repo.
    #[command(flatten)]
    pub por: PorFlag,

    /// `--dry-run` + `--private` + `--push-retries` /
    /// `--push-retry-delay`: flatten of the shared
    /// [`ProvisionOptionFlagBundle`] bundle.
    #[command(flatten)]
    pub provision: ProvisionOptionFlagBundle,

    /// `--use-template`: flatten of the shared
    /// [`UseTemplateOption`] leaf.
    #[command(flatten)]
    pub use_template: UseTemplateOption,

    /// `--config none|<path>`: flatten of the shared
    /// [`ConfigOption`] leaf. Only meaningful with `--por`,
    /// rejected at preflight when paired with the default dual
    /// shape. `.gitignore` is always written regardless of
    /// `--config`.
    #[command(flatten)]
    pub config: ConfigOption,

    /// The agent repo's directory, one name inside the project
    /// directory [default: .agent-session]. Dual only.
    #[arg(long, value_name = "DIR", value_parser = parse_agent_dir)]
    pub agent_dir: Option<String>,

    /// The agent repo's full name on its remote. Dual only.
    ///
    /// Default: the work repo's name plus `--agent-suffix`.
    #[arg(long, value_name = "NAME", conflicts_with = "agent_suffix")]
    pub agent_repo: Option<String>,

    /// The suffix the work repo's name takes to name the agent
    /// repo on its remote, beginning with `.` or `-`
    /// [default: .agent-session]. Dual only.
    #[arg(
        long,
        value_name = "SUFFIX",
        allow_hyphen_values = true,
        value_parser = parse_agent_suffix
    )]
    pub agent_suffix: Option<String>,
}

/// Parse `--agent-dir`: one name inside the project directory.
///
/// One name, not a path, since the agent side's own config reaches
/// the work repo as `..`. `.git` and `.jj` are the work repo's own.
fn parse_agent_dir(s: &str) -> Result<String, String> {
    if s.is_empty() || s == "." || s == ".." || s.contains(['/', '\\']) {
        return Err(format!(
            "'{s}' is not one directory name inside the project (e.g. .agent-session)"
        ));
    }
    if s == ".git" || s == ".jj" {
        return Err(format!("'{s}' is the work repo's own directory"));
    }
    Ok(s.to_string())
}

/// Parse `--agent-suffix`: it must begin with `.` or `-` and carry
/// more than that one character, so the agent repo's name reads as
/// the work repo's with something added.
fn parse_agent_suffix(s: &str) -> Result<String, String> {
    if s.len() < 2 || !s.starts_with(['.', '-']) || s.contains(['/', ':']) {
        return Err(format!(
            "'{s}' must begin with '.' or '-' and name something after it (e.g. .agent-session)"
        ));
    }
    Ok(s.to_string())
}

/// Run an operation with retries, sleeping between attempts.
fn retry_op<T>(
    retry: &PushRetryOptions,
    mut op: impl FnMut() -> Result<T, Box<dyn std::error::Error>>,
) -> Result<T, Box<dyn std::error::Error>> {
    let mut last_err = String::new();
    for attempt in 1..=retry.push_retries {
        match op() {
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

use crate::common::{mkdir_p, write_file};

/// Parse the `--use-template` value into `(work, bot)` template paths.
///
/// Format: `WORK[,BOT]`. If `BOT` is omitted, the default is the sibling
/// directory whose name is `<WORK-basename>.claude` (via
/// `Path::with_file_name`, so a trailing slash on `CODE` does not produce
/// a different result).
pub(crate) fn parse_use_template(
    s: &str,
) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let mut parts = s.splitn(2, ',');
    let work_raw = parts.next().unwrap_or(""); // OK: splitn always yields at least one element
    let bot_raw = parts.next();
    let work_trim = work_raw.trim();
    if work_trim.is_empty() {
        return Err("--use-template: work template path is empty".into());
    }
    let work = PathBuf::from(work_trim);
    let bot = match bot_raw.map(str::trim) {
        Some(b) if !b.is_empty() => PathBuf::from(b),
        _ => {
            let file_name = work.file_name().ok_or_else(|| {
                format!(
                    "--use-template: cannot derive default bot path from '{}' (no file name \
                     component)",
                    work.display()
                )
            })?;
            let new_name = format!("{}.claude", file_name.to_string_lossy());
            work.with_file_name(new_name)
        }
    };
    Ok((work, bot))
}

/// Default agent-repo directory a fresh init records and creates,
/// `--agent-dir` its override. Every *reader* resolves the dir from
/// `repos.agent`, so this is where a new workspace's default is
/// chosen. Not `.claude`, where the harness's bind mounts land.
pub(crate) const DEFAULT_AGENT_DIR: &str = ".agent-session";

/// Default suffix the work repo's name takes to name the agent repo
/// on its remote, `--agent-suffix` its override and `--agent-repo`
/// its replacement. A workspace's recorded `[remote] agent-repo`
/// wins over it, and its absence still means `.claude`.
pub(crate) const DEFAULT_AGENT_SUFFIX: &str = ".agent-session";

/// Top-level non-hidden files init writes. Kept here so that if init is
/// ever extended to write non-hidden top-level files, the pre-flight
/// conflict scan flags any template that would clash. Currently empty
/// because init only writes hidden files (`.vc-config.md`, `.gitignore`),
/// and the template copy skips hidden entries.
const RESERVED_TEMPLATE_ENTRIES: &[&str] = &[];

/// Validate one template directory: exists, is a directory, and has no
/// top-level non-hidden entry that would collide with init's own writes.
///
/// - `label` prefixes every error message (`work` / `agent`).
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
    work: &Path,
    bot: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    validate_template_one("work", work)?;
    validate_template_one("agent", bot)?;
    Ok(())
}

/// Recursively copy non-hidden entries from `src` to `dst`. Any entry whose
/// file name starts with `.` is skipped. Symlinks are skipped with a debug
/// log: templates don't need them, and following them risks escaping the
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
/// first newline is preserved verbatim, and a file with no newline becomes
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

/// Run `gh <args>`, returning trimmed stdout on success and the
/// stderr-carrying error on failure. The GhCreate path's one spawn
/// helper: `gh auth status`, `gh repo view`, `gh repo create`.
///
/// Logs the command and its streams at `debug!`, the shape the
/// retired `common::run` gave these callers.
fn gh(args: &[&str], cwd: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let args_str = args.join(" ");
    debug!("$ gh {args_str}");
    // Register entry 3 (clippy.toml): init's gh provisioning.
    // Creating a repo on GitHub is the forge's REST API, not a
    // version-control operation, and gh is its authenticated
    // client.
    #[allow(clippy::disallowed_methods)]
    let output = std::process::Command::new("gh")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("failed to run gh: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stdout.is_empty() {
        debug!("{stdout}");
    }
    if !output.status.success() {
        return Err(format!("gh {args_str} failed: {stderr}").into());
    }
    if !stderr.is_empty() {
        debug!("{stderr}");
    }
    Ok(stdout)
}

/// Check if a GitHub repo exists.
fn gh_repo_exists(owner: &str, name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let full = format!("{owner}/{name}");
    Ok(gh(&["repo", "view", &full], Path::new(".")).is_ok())
}

/// Which shape of `.vc-config.md` to generate.
///
/// The `[repos]` registry's values are file-relative, so the two
/// sides of a dual workspace carry different blocks: the side
/// whose entry resolves to the config's own directory (`"."`)
/// names that side.
#[derive(Clone, Copy)]
pub(crate) enum ConfigRole<'a> {
    /// The work repo of a dual-repo workspace: `agent_dir` is the
    /// agent repo's directory and `agent_repo` its name on the
    /// remote, `None` rendering no `[remote]` table.
    DualWork {
        agent_dir: &'a str,
        agent_repo: Option<&'a str>,
    },
    /// The bot repo of a dual-repo workspace.
    DualBot,
    /// The sole repo in a single-repo (POR) workspace.
    WorkOnly,
}

/// Renders the header comment + active `[repos]` registry for a
/// generated config file, role-specific. The text is TOML, and
/// `render_vc_config` fences it.
fn render_workspace_header(role: ConfigRole) -> String {
    match role {
        ConfigRole::DualWork {
            agent_dir,
            agent_repo,
        } => {
            // The agent-repo's remote name is the work side's to
            // record: clone and sync read it from here, since at the
            // moment they need it there is no agent-repo to ask. It
            // gets a table of its own because `[repos]` registers
            // local paths and this is a remote name. A `None` name
            // omits the table, which is the shape every workspace
            // created before the key has, and reads as the work name
            // plus `.claude`.
            let remote_table = match agent_repo {
                Some(name) => format!("\n[remote]\nagent-repo = \"{name}\"\n"),
                None => String::new(),
            };
            format!(
                r#"# vc-config: Vibe Coding workspace configuration
#
# [repos] is the workspace's repo registry: work and bot are paths
# relative to this file's directory (absolute allowed, discouraged).
# The entry that resolves to this config's own directory names the
# side: work = "." makes this the work repo.
#
# [remote] holds remote names rather than paths. agent-repo is the
# agent-repo's name on its remote, the last URL segment only, since
# the owner and the host come from the work repo's own remote.

[repos]
work = "."
agent = "{agent_dir}"
{remote_table}"#
            )
        }
        ConfigRole::DualBot => r#"# vc-config: Vibe Coding workspace configuration
#
# [repos] is the workspace's repo registry: work and bot are paths
# relative to this file's directory (absolute allowed, discouraged).
# The entry that resolves to this config's own directory names the
# side: agent = "." makes this the agent repo.

[repos]
work = ".."
agent = "."
"#
        .to_string(),
        ConfigRole::WorkOnly => r#"# vc-config: Vibe Coding workspace configuration
#
# [repos] is the workspace's repo registry: work is this repo's
# path relative to this file's directory. Used to resolve changeID
# paths in git trailers (e.g. ochid: /changeID).

[repos]
work = "."
"#
        .to_string(),
    }
}

/// Renders a commented block documenting every settable workspace
/// config key not already covered by the active `[repos]` block
/// above, sourced from `config_schema::schema()` (itself generated
/// from the `vc-config.md` prototype) so this list cannot drift
/// from the schema.
///
/// Grouped by TOML section (schema/first-seen order), each key
/// emitted via `config_schema::render_key_block`: a multi-line
/// doc-block whose assignment line is `# <leaf> = <value>` (a
/// non-`required` key always renders commented, and every optional
/// key here is non-`required` by construction, see the
/// `workspace.*` skip below), so the whole block parses as
/// comments only.
fn render_optional_keys_block() -> String {
    use crate::config_schema::{Home, render_key_block, schema, section_and_leaf};

    let mut out = String::new();
    out.push_str(
        "\n# Optional keys: uncomment and edit to override the built-in\n\
         # default. `vc-x1 config` prints this list from the binary.\n",
    );

    let mut current_section: Option<String> = None;
    for key in schema() {
        // `[repos]` and `[remote]` are rendered actively above, so
        // their keys are not offered again as commented overrides.
        if key.path.starts_with("repos.") || key.path.starts_with("remote.") {
            continue;
        }
        if !key
            .homes
            .iter()
            .any(|h| matches!(h, Home::WorkspaceCode | Home::WorkspaceBot))
        {
            continue;
        }
        // A key with no default has nothing to offer a generated
        // file: its value is the workspace's to invent. Skipped
        // before its section header is written, or a section whose
        // keys are all defaultless (`[family]`, `[validate]`) would
        // leave a bare header behind with nothing under it.
        if key.default.is_none() {
            continue;
        }
        let (section, _leaf) = section_and_leaf(key.path);
        if current_section.as_deref() != Some(section) {
            // The first section needs a blank line separating it from
            // the intro comment above, while later sections inherit
            // one from the previous key block's trailing blank line
            // (`render_key_block` always ends with one).
            if current_section.is_none() {
                out.push('\n');
            }
            out.push_str(&format!("[{section}]\n"));
            current_section = Some(section.to_string());
        }
        out.push_str(&render_key_block(key));
    }

    out
}

/// The prose a generated `.vc-config.md` opens with, above its
/// one `toml` fence.
///
/// A new workspace's file says what it is and where its
/// documentation lives, and the links are relative to nothing in
/// the new repo, so they name the vc-x1 documentation by url.
const CONFIG_MD_INTRO: &str = "\
# vc-x1 config file

This workspace's configuration. The `toml` fence below is the configuration itself, and the prose
around it is yours to write: only a fence tagged exactly `toml` is read.

Every settable key is documented in vc-x1's `vc-config.md`, and `vc-config-model.md` beside it
shows every table with its default or a typical value. `vc-x1 config` prints the same schema from
the binary you are running.

";

/// Renders the complete generated `.vc-config.md` content for
/// `role`: the intro prose, then one `toml` fence holding the
/// active header + `[repos]` registry and a commented block
/// documenting the rest of the settable-key surface (see
/// `render_optional_keys_block`).
///
/// One fence rather than one per table: everything after `[repos]`
/// is commented, so splitting it into per-table fences would make a
/// document of empty fences. A workspace that uncomments a key is
/// free to split the file up, which the carrier reads the same way.
pub(crate) fn render_vc_config(role: ConfigRole) -> String {
    let mut toml = render_workspace_header(role);
    toml.push_str(&render_optional_keys_block());
    format!("{CONFIG_MD_INTRO}```toml\n{toml}```\n")
}

/// Gitignore for a dual workspace's work repo: the agent repo's
/// directory is ignored, since it is a repo of its own.
pub(crate) fn render_work_gitignore(agent_dir: &str) -> String {
    format!("/target\n/{agent_dir}\n/.git\n/.jj\n")
}

const GITIGNORE_SESSION: &str = ".git
.jj
";

/// Gitignore for a single-repo workspace.
///
/// - Same as `render_work_gitignore` minus the agent directory's
///   entry: there's no agent subdir in single-repo mode.
pub(crate) const GITIGNORE_APP_ONLY: &str = "/target
/.git
/.jj
";

/// Write the POR (single-repo) canned `.vc-config.md` into `dir`.
///
/// Split from the legacy `write_por_config` so the config write is
/// gateable by `--config` while the `.gitignore` write stays
/// unconditional.
fn write_por_vc_config(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_file(
        &dir.join(crate::config_md::VC_CONFIG_MD),
        &render_vc_config(ConfigRole::WorkOnly),
    )
}

/// Write the POR (single-repo) `.gitignore` into `dir`. Always
/// unconditional: `--config` controls only the config file.
fn write_por_gitignore(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_file(&dir.join(".gitignore"), GITIGNORE_APP_ONLY)
}

/// Copy a user-supplied config file from `src` to `dir`.
///
/// The destination name follows the source's carrier, `.md` to
/// `.vc-config.md` and anything else to `.vc-config.toml`, since
/// naming a TOML file `.md` would hand the markdown filter a file
/// with no fences and yield an empty config.
///
/// Bytewise copy: no parse, no schema validation. If the content is
/// malformed the project will surface the problem on first
/// `find_workspace_root` / config-reader call. Caller is
/// responsible for path-existence preflight.
fn copy_user_config(src: &Path, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let name = match src.extension().and_then(|e| e.to_str()) {
        Some("md") => crate::config_md::VC_CONFIG_MD,
        _ => crate::desc_helpers::VC_CONFIG_FILE,
    };
    std::fs::copy(src, dir.join(name))
        .map_err(|e| format!("--config: failed to copy {}: {e}", src.display()))?;
    Ok(())
}

/// Write the dual-mode work-side `.vc-config.md` and `.gitignore`
/// into `dir`. Used by `create_dual` for the work repo.
///
/// `agent_dir` is the agent repo's directory, recorded as
/// `repos.agent` and ignored, and `agent_repo` its name on the
/// remote, recorded as `[remote] agent-repo` so clone need not
/// derive it.
fn write_work_config(
    dir: &Path,
    agent_dir: &str,
    agent_repo: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    write_file(
        &dir.join(crate::config_md::VC_CONFIG_MD),
        &render_vc_config(ConfigRole::DualWork {
            agent_dir,
            agent_repo: Some(agent_repo),
        }),
    )?;
    write_file(&dir.join(".gitignore"), &render_work_gitignore(agent_dir))?;
    Ok(())
}

/// Write the dual-mode agent-side `.vc-config.md` and
/// `.gitignore` into `dir`. Used by `create_dual` for the agent repo.
fn write_bot_config(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_file(
        &dir.join(crate::config_md::VC_CONFIG_MD),
        &render_vc_config(ConfigRole::DualBot),
    )?;
    write_file(&dir.join(".gitignore"), GITIGNORE_SESSION)?;
    Ok(())
}

/// Expand `~` and `$VAR` / `${VAR}` substitutions in a user string.
///
/// - `~` and `~/...` resolve via `HOME`.
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
                return Err(format!("unterminated '${{...}}' in '{s}'").into());
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
/// - Scheme-qualified (`scheme://...`).
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
/// - SSH scp-like (`git@github.com:...`).
/// - `ssh://git@github.com/...`.
/// - `https://github.com/...` and `http://github.com/...`.
/// - GitHub URLs route to the `gh repo create` provisioner.
pub(crate) fn is_github_url(s: &str) -> bool {
    s.starts_with("git@github.com:")
        || s.starts_with("ssh://git@github.com/")
        || s.starts_with("https://github.com/")
        || s.starts_with("http://github.com/")
}

/// Normalize a `--repo local=<parent>` value: expand variables,
/// make absolute. Parent doesn't need to exist yet, init creates
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
/// See `notes/chores/chores-06.md > Generalize --scope across commands
/// (0.40.0) > 0.40.0-1` for the dispatch rules.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Provisioner {
    /// `git init --bare` on local paths under `--repo-local`.
    LocalBareInit,
    /// `gh repo create` for GitHub URLs (default mode or
    /// `--repo-remote <github-url>`).
    GhCreate,
    /// Skip any create step: the caller pre-created the remote
    /// (non-GitHub URLs under `--repo-remote`).
    ExternalPreExisting,
}

/// Fully-resolved inputs to the init execution phase.
///
/// - Dispatch and ambiguity rules live in `plan_init`.
/// - Downstream code operates on an `InitPlan`, execution stays
///   linear.
#[derive(Debug)]
pub(crate) struct InitPlan {
    /// Which side(s) this plan creates. `scope.is_work_only()` means
    /// single-repo, `scope.is_both()` means dual.
    pub scope: Scope,
    pub project_dir: PathBuf,
    pub name: String,
    pub work_url: String,
    pub provisioner: Provisioner,
    /// Set only when `provisioner == LocalBareInit`.
    pub work_bare_path: Option<PathBuf>,
    /// GitHub `owner/name` for the work side, only populated for
    /// the `GhCreate` path (`gh repo create` needs it).
    pub gh_work_slug: Option<String>,
    /// The agent side, `Some` exactly when `scope.is_both()`, so a
    /// dual step takes it whole rather than unwrapping its parts.
    pub agent: Option<AgentPlan>,
}

/// A dual plan's agent side, filled by [`plan_agent_side`].
#[derive(Debug)]
pub(crate) struct AgentPlan {
    /// The directory as the work config records it, one name
    /// (`.agent-session`).
    pub dir: String,
    /// `project_dir` joined with `dir`.
    pub path: PathBuf,
    /// The project name its README takes (e.g. `<name>.agent-session`).
    pub name: String,
    /// The origin URL.
    pub url: String,
    /// The bare-repo path, `Some` only under `LocalBareInit`.
    pub bare_path: Option<PathBuf>,
    /// The GitHub slug, `Some` only under `GhCreate`.
    pub gh_slug: Option<String>,
}

/// Build an `InitPlan` from CLI args + user config.
///
/// Dispatches on the parsed `<TARGET>` form:
///
/// - `Url(u)` / `OwnerName(o, n)` -> URL is explicit, config not
///   consulted, and `--account` and `--repo` are rejected as
///   meaningless.
/// - `Path(p)` -> `p` is the destination, basename names the repo,
///   remote URL resolved from config via the `--repo` chain.
/// - `BareName(n)` -> destination at `cwd/<n>`, remote resolved
///   from config.
///
/// The `cfg` parameter is the loaded user config: `init` loads it
/// once at entry and passes it through so tests can supply a
/// synthetic config without touching disk.
pub(crate) fn plan_init(
    params: &InitParams,
    cfg: &UserConfig,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    debug!(
        "init args: target={:?}, name={:?}, account={:?}, repo={:?}, por={}, private={}",
        params.target, params.name, params.account, params.repo, params.por, params.private
    );

    let scope = if params.por {
        Scope(vec![Side::Work])
    } else {
        Scope(vec![Side::Work, Side::Bot])
    };

    if scope.is_work_only()
        && let Some(t) = &params.use_template
        && t.contains(',')
    {
        return Err(format!(
            "--por takes a single template path; got '{t}' (drop the `,BOT` half)"
        )
        .into());
    }

    if params.config.is_some() && !params.por {
        return Err(
            "--config is only valid with --por (dual configs are per-side and unconditional)"
                .into(),
        );
    }
    if let Some(ConfigKind::Path(p)) = &params.config
        && !p.exists()
    {
        return Err(format!("--config: path does not exist: {}", p.display()).into());
    }

    if scope.is_work_only() {
        for (set, flag) in [
            (params.agent_dir.is_some(), "--agent-dir"),
            (params.agent_repo.is_some(), "--agent-repo"),
            (params.agent_suffix.is_some(), "--agent-suffix"),
        ] {
            if set {
                return Err(
                    format!("{flag} is meaningless with --por (there is no agent repo)").into(),
                );
            }
        }
    }

    let parsed = parse_target(&params.target)?;
    debug!("parse_target: {:?} -> {:?}", params.target, parsed);
    let mut plan = match parsed {
        Target::Url(url) => plan_from_url(params, scope, url),
        Target::Path(p) => plan_from_path(params, scope, p, cfg),
        Target::BareName(n) => plan_from_bare_name(params, scope, n, cfg),
    }?;
    if plan.scope.is_both() {
        plan.agent = Some(plan_agent_side(&plan, params)?);
    }
    debug!(
        "plan_init: project_dir={}, name={}, work_url={}, provisioner={:?}, \
         gh_work_slug={:?}, work_bare_path={:?}, agent={:?}",
        plan.project_dir.display(),
        plan.name,
        plan.work_url,
        plan.provisioner,
        plan.gh_work_slug,
        plan.work_bare_path,
        plan.agent,
    );
    Ok(plan)
}

/// Plan when TARGET is a URL or `owner/name` shorthand.
///
/// - URL is explicit, config not consulted.
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
    let work_url = ensure_git_suffix(&url);
    let derived_name = derive_name(&work_url)?;
    let name = params.name.clone().unwrap_or(derived_name);
    let cwd = std::env::current_dir()?;
    let project_dir = cwd.join(&name);

    let provisioner = if is_github_url(&work_url) {
        Provisioner::GhCreate
    } else {
        Provisioner::ExternalPreExisting
    };
    let gh_work_slug = if provisioner == Provisioner::GhCreate {
        Some(github_slug_from_url(&work_url)?)
    } else {
        None
    };

    build_plan(
        scope,
        name,
        project_dir,
        work_url,
        provisioner,
        gh_work_slug,
    )
}

/// Plan when TARGET is a path. Path IS the destination and
/// basename names the repo. Remote resolved via the `--repo` chain.
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
    let last = project_dir
        .file_name()
        .ok_or_else(|| {
            format!(
                "path TARGET '{}' has no last component, so no repo name can be derived",
                path.display()
            )
        })?
        .to_str()
        .ok_or("path TARGET is not valid UTF-8")?;
    // Through `derive_name`, the same normalization a URL target
    // gets, so a `foo.git` directory yields the repo name `foo`
    // rather than `foo.git`. The two branches answering this
    // differently is what made a `./tmp/xx1.git` target ask GitHub
    // for `xx1.git`, get `xx1` (GitHub drops the suffix), and then
    // write a remote pointing at the name it never created
    // (2026-08-28, bugs.md).
    let name = derive_name(last)?;
    let (cat, val) = config::resolve_repo(cfg, params.account.as_deref(), params.repo.as_ref())?;
    plan_from_resolved(scope, name, project_dir, &cat, &val)
}

/// Plan when TARGET is a bare alphanumeric NAME. Destination at
/// `cwd/<NAME>`, remote resolved via the `--repo` chain.
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
            "--repo category '{other}' is not recognized: built-ins are 'remote' and 'local'"
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
    let work_url = format!("{prefix}/{name}.git");
    let provisioner = if is_github_url(&work_url) {
        Provisioner::GhCreate
    } else {
        Provisioner::ExternalPreExisting
    };
    let gh_work_slug = if provisioner == Provisioner::GhCreate {
        Some(github_slug_from_url(&work_url)?)
    } else {
        None
    };
    build_plan(
        scope,
        name,
        project_dir,
        work_url,
        provisioner,
        gh_work_slug,
    )
}

/// Plan for `category = "local"`: bare repos under a parent dir.
///
/// - Dual: `<parent>/remote-work.git`, and the agent bare beside it,
///   named by [`plan_agent_side`] (`remote-work.agent-session.git`).
/// - POR: `<parent>/remote.git`.
///
/// The agent bare's name is recorded in the work config, which is
/// what `clone` reads to locate it, so a locally init'd project
/// round-trips through `vc-x1 clone` (bugs.md #2).
fn plan_local(
    scope: Scope,
    name: String,
    project_dir: PathBuf,
    parent_spec: &str,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    let parent = normalize_local_parent(parent_spec)?;

    let work_bare = if scope.is_work_only() {
        parent.join("remote.git")
    } else {
        parent.join("remote-work.git")
    };
    Ok(InitPlan {
        scope,
        work_url: work_bare.to_string_lossy().to_string(),
        work_bare_path: Some(work_bare),
        project_dir,
        name,
        provisioner: Provisioner::LocalBareInit,
        gh_work_slug: None,
        agent: None,
    })
}

/// Compose an `InitPlan` for the URL-style provisioners (`GhCreate`
/// and `ExternalPreExisting`): both `plan_from_url` and
/// `plan_remote` share this tail because the only difference between
/// them is how they construct the URL.
fn build_plan(
    scope: Scope,
    name: String,
    project_dir: PathBuf,
    work_url: String,
    provisioner: Provisioner,
    gh_work_slug: Option<String>,
) -> Result<InitPlan, Box<dyn std::error::Error>> {
    Ok(InitPlan {
        scope,
        project_dir,
        name,
        work_url,
        provisioner,
        work_bare_path: None,
        gh_work_slug,
        agent: None,
    })
}

/// A dual plan's agent side, from the work side and the `--agent-*`
/// flags, one rule for every provisioner.
///
/// - The directory is `--agent-dir`, else `.agent-session`.
/// - The remote name is `--agent-repo`, else the work URL's last
///   segment plus `--agent-suffix`, else plus `.agent-session`, and
///   the URL is the work URL with that segment replaced, so the two
///   repos are siblings under one owner.
/// - The local bare, under `--repo local`, is that URL as a path,
///   and the GitHub slug, under `gh repo create`, is read off it.
fn plan_agent_side(
    plan: &InitPlan,
    params: &InitParams,
) -> Result<AgentPlan, Box<dyn std::error::Error>> {
    let agent_dir = params
        .agent_dir
        .clone()
        .unwrap_or_else(|| DEFAULT_AGENT_DIR.to_string()); // OK: absent flag means the default
    let suffix = params
        .agent_suffix
        .as_deref()
        .unwrap_or(DEFAULT_AGENT_SUFFIX); // OK: absent flag means the default
    let remote_name = match &params.agent_repo {
        Some(name) => name.clone(),
        None => format!("{}{suffix}", derive_name(&plan.work_url)?),
    };
    let url = crate::url::agent_url(&plan.work_url, Some(&remote_name));
    Ok(AgentPlan {
        path: plan.project_dir.join(&agent_dir),
        dir: agent_dir,
        name: match &params.agent_repo {
            Some(name) => name.clone(),
            None => format!("{}{suffix}", plan.name),
        },
        bare_path: match plan.provisioner {
            Provisioner::LocalBareInit => Some(PathBuf::from(&url)),
            _ => None,
        },
        gh_slug: match plan.provisioner {
            Provisioner::GhCreate => Some(github_slug_from_url(&url)?),
            _ => None,
        },
        url,
    })
}

/// Resolve a Path target to an absolute `PathBuf`. Handles `~`,
/// `~/X`, `$VAR`, `${VAR}` expansion, makes relative paths
/// absolute against cwd, and lexically normalizes `.` / `..`. The
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
            let name = parts[parts.len() - 1];
            // GitHub drops a trailing `.git` from a repo name at
            // creation, so asking for one means the repo it makes
            // is not the repo the remote we then write points at,
            // and the first push fails with "the repository
            // exists" being the false half (2026-08-28, bugs.md).
            // Refused here rather than repaired, since silently
            // renaming what the caller asked for is how the
            // mismatch started.
            if name.ends_with(".git") {
                return Err(format!(
                    "repo name '{name}' ends with '.git', which GitHub drops at creation, so \
                     the repo would be '{}'. Name it '{}' instead",
                    name.trim_end_matches(".git"),
                    name.trim_end_matches(".git")
                )
                .into());
            }
            return Ok(format!("{}/{name}", parts[parts.len() - 2]));
        }
    }
    Err(format!("cannot extract owner/name slug from GitHub URL '{url}'").into())
}

impl SubcommandRunner for InitArgs {
    type Params = InitParams;

    /// Delegate to the existing `From<&InitArgs>` impl in
    /// [`crate::init::params`] (total: never fails).
    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(InitParams::from(self))
    }

    /// Run the existing `init` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        init(ctx, params)
    }
}

/// Init entry point: runs the dual or POR provisioning flow.
///
/// Takes the shared `Context` (loaded user config) and the flat
/// `InitParams`. CLI builds both at the binary edge in main.rs,
/// and tests construct `InitParams` directly.
///
/// `params.create_symlink` controls the `~/.claude/projects/`
/// symlink side effect:
///
/// - `true`: CLI behavior, where step 11 creates the symlink for
///   dual-scope runs.
/// - `false`: suppresses that side effect, used by test
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

    // No "is jj installed" probe: init runs through
    // jj-lib, and main's version gate already errored out on a
    // missing or mismatched jj CLI before dispatch.
    if plan.provisioner == Provisioner::GhCreate {
        debug!("verify gh CLI is installed + authenticated");
        gh(&["auth", "status"], Path::new("."))
            .map_err(|_| "gh is not installed or not authenticated (run: gh auth login)")?;
    }

    if plan.project_dir.exists() {
        return Err(format!("'{}' already exists", plan.project_dir.display()).into());
    }

    match &plan.provisioner {
        Provisioner::GhCreate => {
            #[allow(clippy::unwrap_used)]
            // OK: GhCreate path always sets gh_work_slug
            let work_slug = plan.gh_work_slug.as_ref().unwrap();
            let (work_owner, work_name) = split_slug(work_slug)?;
            if gh_repo_exists(work_owner, work_name)? {
                return Err(format!("GitHub repo '{work_slug}' already exists").into());
            }
            if let Some(bot_slug) = plan.agent.as_ref().and_then(|a| a.gh_slug.as_ref()) {
                let (bot_owner, bot_name) = split_slug(bot_slug)?;
                if gh_repo_exists(bot_owner, bot_name)? {
                    return Err(format!("GitHub repo '{bot_slug}' already exists").into());
                }
            }
        }
        Provisioner::LocalBareInit => {
            #[allow(clippy::unwrap_used)]
            // OK: LocalBareInit path always sets work_bare_path
            let work_bare = plan.work_bare_path.as_ref().unwrap();
            if work_bare.exists() {
                return Err(format!(
                    "bare repo '{}' already exists; refusing to clobber",
                    work_bare.display()
                )
                .into());
            }
            if let Some(bot_bare) = plan.agent.as_ref().and_then(|a| a.bare_path.as_ref())
                && bot_bare.exists()
            {
                return Err(format!(
                    "bare repo '{}' already exists; refusing to clobber",
                    bot_bare.display()
                )
                .into());
            }
        }
        Provisioner::ExternalPreExisting => {
            // For path-shaped URLs: the path must exist (caller
            // pre-created with `git init --bare`). For scheme-URL
            // values we can't preflight cheaply, let git push
            // surface failures.
            if !is_remote_url(&plan.work_url) {
                let p = PathBuf::from(&plan.work_url);
                if !p.exists() {
                    return Err(format!(
                        "--repo-remote: work path '{}' does not exist, \
                         pre-create with `git init --bare`, or use --repo-local for \
                         fixture creation",
                        p.display()
                    )
                    .into());
                }
            }
            if let Some(bot_url) = plan.agent.as_ref().map(|a| &a.url)
                && !is_remote_url(bot_url)
            {
                let p = PathBuf::from(bot_url);
                if !p.exists() {
                    return Err(format!(
                        "--repo-remote: bot path '{}' does not exist, \
                         pre-create with `git init --bare`, or use --repo-local for \
                         fixture creation",
                        p.display()
                    )
                    .into());
                }
            }
        }
    }

    let templates = match &params.use_template {
        Some(s) => {
            let (work_t, bot_t) = parse_use_template(s)?;
            if is_dual {
                validate_templates(&work_t, &bot_t)?;
                Some((work_t, Some(bot_t)))
            } else {
                validate_template_one("work", &work_t)?;
                Some((work_t, None))
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
        info!("Dry run, would execute:");
        info!("  1. Create directories: {}", plan.project_dir.display());
        info!(
            "  2. git init + jj git init --colocate on {}",
            if is_dual {
                "both repos"
            } else {
                "the work repo"
            }
        );
        info!(
            "  3. Write .vc-config.md and .gitignore{}",
            if is_dual {
                " to both repos"
            } else {
                " (work-only layout)"
            }
        );
        match &templates {
            Some((c, b)) => {
                info!("  4. Copy templates (non-hidden) + rewrite README.md first line");
                info!("       work: {}", c.display());
                if let Some(b) = b {
                    info!("       bot:  {}", b.display());
                }
            }
            None => info!("  4. (skipped, no --use-template)"),
        }
        if is_dual {
            info!("  5. jj commit both with placeholder ochids");
            info!("  6. Get both chids, jj describe both with correct ochids");
            info!("  7. Remove jj from both (git clean -xdf)");
        } else {
            info!("  5. jj commit work with 'Initial commit'");
            info!("  6. (skipped, no cross-reference in single-repo)");
            info!("  7. Remove jj (git clean -xdf)");
        }
        let agent_url = plan.agent.as_ref().map(|a| a.url.as_str());
        let agent_slug = plan.agent.as_ref().and_then(|a| a.gh_slug.as_deref());
        match &plan.provisioner {
            Provisioner::GhCreate => {
                if is_dual {
                    info!(
                        "  8. gh repo create {} {visibility}; push to {}",
                        agent_slug.unwrap_or(""), // OK: dry-run display only
                        agent_url.unwrap_or(""),  // OK: dry-run display only
                    );
                } else {
                    info!("  8. (skipped, no bot side in single-repo)");
                }
                info!(
                    "  9. gh repo create {} {visibility}; push to {}",
                    plan.gh_work_slug.as_deref().unwrap_or(""), // OK: dry-run display only
                    plan.work_url
                );
            }
            Provisioner::LocalBareInit => {
                if is_dual {
                    info!(
                        "  8. git init --bare {}; push to {}",
                        plan.agent
                            .as_ref()
                            .and_then(|a| a.bare_path.as_ref())
                            .map(|p| p.display().to_string())
                            .unwrap_or_default(), // OK: dry-run display only
                        agent_url.unwrap_or(""), // OK: dry-run display only
                    );
                } else {
                    info!("  8. (skipped, no bot side in single-repo)");
                }
                info!(
                    "  9. git init --bare {}; push to {}",
                    plan.work_bare_path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default(), // OK: dry-run display only
                    plan.work_url
                );
            }
            Provisioner::ExternalPreExisting => {
                if is_dual {
                    info!(
                        "  8. skip create; push to pre-existing {}",
                        agent_url.unwrap_or(""), // OK: dry-run display only
                    );
                } else {
                    info!("  8. (skipped, no bot side in single-repo)");
                }
                info!("  9. skip create; push to pre-existing {}", plan.work_url);
            }
        }
        info!(
            "  10. jj git init --colocate on {}",
            if is_dual {
                "both repos"
            } else {
                "the work repo"
            }
        );
        if is_dual {
            info!("  11. Create Claude Code symlink");
        } else {
            info!("  11. (skipped, no .claude symlink in single-repo)");
        }
        return Ok(());
    }

    match &plan.agent {
        None => create_por(params, &plan, templates, visibility, create_symlink),
        Some(agent) => create_dual(params, &plan, agent, templates, visibility, create_symlink),
    }
}

/// Create-from-empty orchestrator for `--por` (single repo).
///
/// Composes (in order):
/// - `prepare_local_repo` -> conditional `.vc-config.md` write
///   (gated by `--config`) -> `write_por_gitignore` (always) ->
///   `commit_initial` (`OchidStrategy::None`).
/// - `push_repo` for work side (no `clean_exclude`).
/// - No cross-reference (no bot repo), no bot push, no symlink.
///
/// `_create_symlink` is unused (no symlink in single-repo), kept in
/// the signature for shape-symmetry with `create_dual`.
fn create_por(
    params: &InitParams,
    plan: &InitPlan,
    templates: Option<(PathBuf, Option<PathBuf>)>,
    visibility: &str,
    _create_symlink: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let work_template = templates.as_ref().map(|(c, _)| c.as_path());

    prepare_local_repo(&plan.project_dir, "work", work_template, &plan.name)?;
    match &params.config {
        None => write_por_vc_config(&plan.project_dir)?,
        Some(ConfigKind::None) => {} // skip, user asked not to write
        Some(ConfigKind::Path(p)) => copy_user_config(p, &plan.project_dir)?,
    }
    write_por_gitignore(&plan.project_dir)?;
    let work_chid = commit_initial(&plan.project_dir, "work", OchidStrategy::None)?;

    info!("Step 6: (skipped, no cross-reference in single-repo)");
    // Colocated, so the jj commit id is the git hash.
    let hash = jj::cid_of(&plan.project_dir, "@-")?;
    debug!("work repo: chid={work_chid} hash={hash}");

    info!("Step 8: (skipped, no bot side in single-repo)");

    let work_chid_final = push_repo(
        &plan.project_dir,
        "work",
        "Step 9",
        plan,
        params,
        visibility,
        &plan.work_url,
        plan.gh_work_slug.as_deref(),
        plan.work_bare_path.as_deref(),
    )?;

    info!("Step 11: (skipped, no .claude symlink in single-repo)");

    info!("");
    info!("Done! Project created at {}", plan.project_dir.display());
    info!(
        "  Work repo:    {}  (chid={work_chid_final})",
        plan.work_url
    );

    debug!("init: exit");
    Ok(())
}

/// Create-from-empty orchestrator for the default dual shape.
///
/// Composes (in order):
/// - Work side: `prepare_local_repo` -> `write_work_config` ->
///   `commit_initial` (`OchidStrategy::Placeholder`).
/// - Bot side: `prepare_local_repo` -> `write_bot_config`
///   -> `commit_initial` (`OchidStrategy::Placeholder`).
/// - `cross_ref_ochids`: rewrite both initial commits' placeholder
///   trailers once each side's chid is known.
/// - `push_repo` for bot side (no `clean_exclude`).
/// - `push_repo` for work side.
/// - `symlink::install` (when `create_symlink`).
fn create_dual(
    params: &InitParams,
    plan: &InitPlan,
    agent: &AgentPlan,
    templates: Option<(PathBuf, Option<PathBuf>)>,
    visibility: &str,
    create_symlink: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (work_template, agent_template) = match templates.as_ref() {
        Some((c, b)) => (Some(c.as_path()), b.as_deref()),
        None => (None, None),
    };

    prepare_local_repo(&plan.project_dir, "work", work_template, &plan.name)?;
    // The recorded name is the agent-repo's *remote* name, the last
    // segment of its origin URL, which is not `agent.name`: that is the
    // local directory's project name, and a fixture whose bare is
    // `remote-work.agent-session.git` under a project called `tr` shows the
    // two diverging.
    let agent_repo = derive_name(&agent.url)?;
    write_work_config(&plan.project_dir, &agent.dir, &agent_repo)?;
    let work_chid = commit_initial(&plan.project_dir, "work", OchidStrategy::Placeholder)?;

    prepare_local_repo(&agent.path, "agent", agent_template, &agent.name)?;
    write_bot_config(&agent.path)?;
    let agent_chid = commit_initial(&agent.path, "agent", OchidStrategy::Placeholder)?;

    cross_ref_ochids(&plan.project_dir, &work_chid, &agent.path, &agent_chid)?;

    let agent_chid_final = push_repo(
        &agent.path,
        "agent",
        "Step 8",
        plan,
        params,
        visibility,
        &agent.url,
        agent.gh_slug.as_deref(),
        agent.bare_path.as_deref(),
    )?;
    let work_chid_final = push_repo(
        &plan.project_dir,
        "work",
        "Step 9",
        plan,
        params,
        visibility,
        &plan.work_url,
        plan.gh_work_slug.as_deref(),
        plan.work_bare_path.as_deref(),
    )?;

    let sl_opt = if create_symlink {
        info!("Step 11: Creating Claude Code symlink...");
        Some(symlink::install(&plan.project_dir)?)
    } else {
        info!("Step 11: (skipped, symlink disabled by caller)");
        None
    };

    info!("");
    info!("Done! Project created at {}", plan.project_dir.display());
    info!(
        "  Work repo:    {}  (chid={work_chid_final})",
        plan.work_url
    );
    info!("  Agent repo:   {}  (chid={agent_chid_final})", agent.url);
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

/// Push primitive: bookmark + provision + publish on `target`.
/// Returns the final chid (`jj @-`).
///
/// jj-only since 0.76.0-5: jj stays colocated throughout, set
/// the bookmark, provision the remote, `jj git remote add`, then
/// `jj git push`, which establishes tracking as a side effect
/// (no `--allow-new`). The former strip-jj -> git-push ->
/// re-colocate dance (`git clean -xdf`, `git checkout`,
/// `git remote add`, `git push -u`, `jj git init --colocate`)
/// was a leftover of the abandoned submodule design, and the symlink
/// design never needed jj evicted.
///
/// - `target`: repo working dir (already populated by
///   `prepare_local_repo` + `commit_initial`).
/// - `info_label`: narration tag (`"work"`, `"agent"`, etc.).
/// - `step_label_provision`: `"Step 8"` (bot) or
///   `"Step 9"` (work), appears in the provision/push narration.
/// - `plan` / `params` / `visibility` / `remote_url` / `gh_slug` /
///   `bare_path`: forwarded to `run_remote_step`.
#[allow(clippy::too_many_arguments)]
fn push_repo(
    target: &Path,
    info_label: &str,
    step_label_provision: &str,
    plan: &InitPlan,
    params: &InitParams,
    visibility: &str,
    remote_url: &str,
    gh_slug: Option<&str>,
    bare_path: Option<&Path>,
) -> Result<String, Box<dyn std::error::Error>> {
    info!("Step 7: Setting {info_label} bookmark...");
    debug!("place {info_label}-side main bookmark at the initial commit");
    jj::bookmark_set(target, "main", "@-")?;

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

    crate::common::verify_tracking(target, "main")?;
    jj::chid_of(target, "@-")
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
            gh(&["repo", "create", slug, visibility], &plan.project_dir)?;
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
            init_bare_main(bare)?;
        }
        Provisioner::ExternalPreExisting => {
            info!("{step_label}: Using pre-existing {side_label} remote {remote_url}");
        }
    }
    debug!("point {side_label}-side jj at its remote");
    jj::git_remote_add(push_from, "origin", remote_url)?;
    debug!("publish {side_label}-side initial commit; retry for GhCreate's async propagation");
    // The facade's push establishes tracking as a side effect: no
    // `--allow-new`, no follow-up `jj bookmark track`.
    retry_op(&params.push_retry, || {
        jj::git_push_bookmark(push_from, "main")
    })?;
    Ok(())
}

/// Create a bare git repo at `path` with `main` as the initial
/// branch, via gix (was a `git init --bare
/// --initial-branch=main` spawn, the last `git` spawn in init).
///
/// The branch is pinned by an in-memory `init.defaultBranch`
/// override, matching the spawned form's `--initial-branch=main`:
/// the user's own git config must not steer which branch vc-x1
/// publishes to.
fn init_bare_main(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use gix::sec::trust::DefaultForLevel;
    let open_opts = gix::open::Options::default_for_level(gix::sec::Trust::Full)
        .config_overrides(["init.defaultBranch=main"]);
    gix::ThreadSafeRepository::init_opts(
        path,
        gix::create::Kind::Bare,
        gix::create::Options::default(),
        open_opts,
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

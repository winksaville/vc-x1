use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info};

use crate::symlink;

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Project name (directory and GitHub repo name)
    #[arg(value_name = "NAME")]
    pub name: String,

    /// GitHub user or organization
    #[arg(long)]
    pub owner: Option<String>,

    /// Parent directory to create project in [default: cwd]
    #[arg(long)]
    pub dir: Option<PathBuf>,

    /// Create private GitHub repos (default: public)
    #[arg(long)]
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

    /// Seed both repos from template directories.
    ///
    /// Value is `CODE[,BOT]`. If `BOT` is omitted, defaults to the sibling
    /// directory `<CODE>.claude` (file-name concat, not path join — they are
    /// not nested). Non-hidden contents are copied recursively; hidden
    /// entries (names starting with `.`) are skipped since init creates the
    /// repo's own hidden files. If either template has a `README.md`, its
    /// first line is rewritten to `# <repo-name>`.
    #[arg(long, value_name = "CODE[,BOT]")]
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

/// Validate that both template paths exist, are directories, and contain no
/// top-level non-hidden entry that would collide with a file init writes.
pub(crate) fn validate_templates(
    code: &Path,
    bot: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    for (label, p) in [("code", code), ("bot", bot)] {
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
    }
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

pub fn init(args: &InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    debug!("init: enter");
    let parent_dir = match &args.dir {
        Some(d) => d.clone(),
        None => std::env::current_dir()?,
    };
    let project_dir = parent_dir.join(&args.name);
    let session_dir = project_dir.join(".claude");

    // Preflight checks
    info!("Preflight checks...");

    // Check tools
    run("gh", &["auth", "status"], Path::new("."))
        .map_err(|_| "gh is not installed or not authenticated (run: gh auth login)")?;
    run("jj", &["--version"], Path::new(".")).map_err(|_| "jj is not installed")?;

    let owner = match &args.owner {
        Some(o) => o.clone(),
        None => gh_whoami()?,
    };

    if project_dir.exists() {
        return Err(format!("'{}' already exists", project_dir.display()).into());
    }

    let code_repo = format!("{owner}/{}", args.name);
    let session_repo = format!("{owner}/{}.claude", args.name);

    if gh_repo_exists(&owner, &args.name)? {
        return Err(format!("GitHub repo '{code_repo}' already exists").into());
    }
    if gh_repo_exists(&owner, &format!("{}.claude", args.name))? {
        return Err(format!("GitHub repo '{session_repo}' already exists").into());
    }

    let templates = match &args.use_template {
        Some(s) => {
            let (code_t, bot_t) = parse_use_template(s)?;
            validate_templates(&code_t, &bot_t)?;
            Some((code_t, bot_t))
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
        info!("  1. Create directories: {}", project_dir.display());
        info!("  2. git init + jj git init --colocate on both repos");
        info!("  3. Write .vc-config.toml and .gitignore to both");
        match &templates {
            Some((c, b)) => {
                info!(
                    "  4. Copy templates (non-hidden) into both repos + rewrite README.md first line"
                );
                info!("       code: {}", c.display());
                info!("       bot:  {}", b.display());
            }
            None => info!("  4. (skipped — no --use-template)"),
        }
        info!("  5. jj commit both with placeholder ochids");
        info!("  6. Get both chids, jj describe both with correct ochids");
        info!("  7. Remove jj from both (git clean -xdf)");
        info!("  8. gh repo create {session_repo} {visibility}, push");
        info!("  9. gh repo create {code_repo} {visibility}, push");
        info!("  10. jj git init --colocate on both repos");
        info!("  11. Create Claude Code symlink");
        return Ok(());
    }

    // Step 1: Create directories
    info!("Step 1: Creating project directories...");
    mkdir_p(&project_dir)?;
    mkdir_p(&session_dir)?;

    // Step 2: git init + jj init on both repos
    info!("Step 2: Initializing repos...");
    run("git", &["init"], &project_dir)?;
    run("jj", &["git", "init", "--colocate"], &project_dir)?;
    run("git", &["init"], &session_dir)?;
    run("jj", &["git", "init", "--colocate"], &session_dir)?;

    // Step 3: Write config files
    info!("Step 3: Writing config files...");
    write_file(&project_dir.join(".vc-config.toml"), VC_CONFIG_CODE)?;
    write_file(&project_dir.join(".gitignore"), GITIGNORE_CODE)?;
    write_file(&session_dir.join(".vc-config.toml"), VC_CONFIG_SESSION)?;
    write_file(&session_dir.join(".gitignore"), GITIGNORE_SESSION)?;

    // Step 4: Copy templates (if --use-template)
    if let Some((code_t, bot_t)) = &templates {
        info!("Step 4: Copying templates...");
        info!("  code: {} -> {}", code_t.display(), project_dir.display());
        copy_template_recursive(code_t, &project_dir)?;
        rewrite_readme_first_line(&project_dir, &args.name)?;
        info!("  bot:  {} -> {}", bot_t.display(), session_dir.display());
        copy_template_recursive(bot_t, &session_dir)?;
        let session_name = format!("{}.claude", args.name);
        rewrite_readme_first_line(&session_dir, &session_name)?;
    }

    // Step 5: jj commit both with placeholder ochids
    info!("Step 5: Committing both repos with placeholder ochids...");
    run(
        "jj",
        &["commit", "-m", "Initial commit\n\nochid: /none"],
        &project_dir,
    )?;
    run(
        "jj",
        &["commit", "-m", "Initial commit\n\nochid: /none"],
        &session_dir,
    )?;

    // Step 6: Get both chids, then describe both with correct ochids
    info!("Step 6: Setting ochid cross-references...");
    let code_chid = jj_chid("@-", &project_dir)?;
    let session_chid = jj_chid("@-", &session_dir)?;

    let code_desc = format!("Initial commit\n\nochid: /.claude/{session_chid}");
    let session_desc = format!("Initial commit\n\nochid: /{code_chid}");
    run("jj", &["describe", "@-", "-m", &code_desc], &project_dir)?;
    run("jj", &["describe", "@-", "-m", &session_desc], &session_dir)?;

    {
        let hash = run("git", &["rev-parse", "HEAD"], &project_dir)?;
        debug!("code repo: chid={code_chid} hash={hash}");
        let hash = run("git", &["rev-parse", "HEAD"], &session_dir)?;
        debug!(".claude:   chid={session_chid} hash={hash}");
    }

    // Step 7: Set bookmarks (creates git branches), then remove jj
    // Bookmarks must be set before removing .jj/ so git has a 'main' branch to push
    info!("Step 7: Setting bookmarks and removing jj...");
    run("jj", &["bookmark", "set", "main", "-r", "@-"], &project_dir)?;
    run("jj", &["bookmark", "set", "main", "-r", "@-"], &session_dir)?;
    // Clean .claude first, then code repo with --exclude to preserve .claude/
    run("git", &["clean", "-xdf"], &session_dir)?;
    run(
        "git",
        &["clean", "-xdf", "--exclude", ".claude"],
        &project_dir,
    )?;
    // After removing .jj/, git HEAD is detached — reattach to main
    run("git", &["checkout", "main"], &session_dir)?;
    run("git", &["checkout", "main"], &project_dir)?;

    // Step 8: Create .claude GitHub repo and push
    let session_url = format!("git@github.com:{session_repo}.git");
    info!("Step 8: Creating GitHub repo {session_repo}...");
    run(
        "gh",
        &["repo", "create", &session_repo, visibility],
        &project_dir,
    )?;
    run(
        "git",
        &["remote", "add", "origin", &session_url],
        &session_dir,
    )?;
    run_retry(
        "git",
        &["push", "-u", "origin", "main"],
        &session_dir,
        args.push_retries,
        args.push_retry_delay,
    )?;

    // Step 9: Create code GitHub repo and push
    let code_url = format!("git@github.com:{code_repo}.git");
    info!("Step 9: Creating GitHub repo {code_repo}...");
    run(
        "gh",
        &["repo", "create", &code_repo, visibility],
        &project_dir,
    )?;
    run("git", &["remote", "add", "origin", &code_url], &project_dir)?;
    run_retry(
        "git",
        &["push", "-u", "origin", "main"],
        &project_dir,
        args.push_retries,
        args.push_retry_delay,
    )?;

    // Step 10: Re-initialize jj on both repos
    info!("Step 10: Re-initializing jj on both repos...");
    run(
        "jj",
        &["--quiet", "git", "init", "--colocate"],
        &project_dir,
    )?;
    run("jj", &["bookmark", "set", "main", "-r", "@-"], &project_dir)?;
    run(
        "jj",
        &["bookmark", "track", "main", "--remote=origin"],
        &project_dir,
    )?;
    crate::common::verify_tracking(&project_dir, "main")?;
    let code_chid_final = jj_chid("@-", &project_dir)?;

    run(
        "jj",
        &["--quiet", "git", "init", "--colocate"],
        &session_dir,
    )?;
    run("jj", &["bookmark", "set", "main", "-r", "@-"], &session_dir)?;
    run(
        "jj",
        &["bookmark", "track", "main", "--remote=origin"],
        &session_dir,
    )?;
    crate::common::verify_tracking(&session_dir, "main")?;
    let session_chid_final = jj_chid("@-", &session_dir)?;

    // Step 11: Create Claude Code symlink
    info!("Step 11: Creating Claude Code symlink...");
    let symlink_dir = {
        let home =
            std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
        PathBuf::from(home).join(".claude").join("projects")
    };

    let sl = symlink::SymLink::new(&project_dir, Path::new(".claude"), &symlink_dir)?;
    sl.create(false)?;

    info!("");
    info!("Done! Project created at {}", project_dir.display());
    info!("  Code repo:    {code_repo}  (chid={code_chid_final})");
    info!("  Session repo: {session_repo}  (chid={session_chid_final})");
    info!(
        "  Symlink:      {} -> {}",
        sl.symlink_path.display(),
        sl.abs_target.display()
    );

    debug!("init: exit");
    Ok(())
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

    fn parse_err(args: &[&str]) -> String {
        Cli::try_parse_from(args).unwrap_err().to_string()
    }

    #[test]
    fn defaults() {
        let args = parse(&["vc-x1", "init", "my-project"]);
        assert_eq!(args.name, "my-project");
        assert!(args.owner.is_none());
        assert!(args.dir.is_none());
        assert!(!args.private);
        assert!(!args.dry_run);
        assert_eq!(args.push_retries, 5);
        assert_eq!(args.push_retry_delay, 3);
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
        assert_eq!(args.name, "my-project");
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
    fn missing_name() {
        let err = parse_err(&["vc-x1", "init"]);
        assert!(err.contains("NAME"));
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
}

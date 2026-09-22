//! Unit tests for the init module.

use super::*;
use crate::config::RepoSelector;
use crate::options_flags::dry_run::DryRunFlag;
use crate::{Cli, Commands};
use clap::Parser;

fn parse(args: &[&str]) -> InitArgs {
    let cli = Cli::try_parse_from(args).unwrap();
    match cli.command {
        Some(Commands::Init(a)) => a,
        _ => panic!("expected New"),
    }
}

#[test]
fn defaults() {
    let args = parse(&["vc-x1", "init", "https://github.com/owner/repo"]);
    assert_eq!(args.target, "https://github.com/owner/repo");
    assert!(args.name.is_none());
    assert!(args.account.value.is_none());
    assert!(args.repo.value.is_none());
    assert!(!args.por.value);
    assert!(!args.provision.private.value);
    assert!(!args.provision.dry_run.value);
    assert_eq!(args.provision.push_retry.push_retries, 5);
    assert_eq!(args.provision.push_retry.push_retry_delay, 3);
    assert!(args.use_template.value.is_none());
}

#[test]
fn all_opts() {
    let args = parse(&[
        "vc-x1",
        "init",
        "https://github.com/owner/repo",
        "my-dir",
        "--account",
        "work",
        "--repo",
        "local=/tmp/xyz",
        "--por",
        "--private",
        "--dry-run",
        "--push-retries",
        "10",
        "--push-retry-delay",
        "5",
        "--use-template",
        "/tmp/tmpl",
    ]);
    assert_eq!(args.target, "https://github.com/owner/repo");
    assert_eq!(args.name.as_deref(), Some("my-dir"));
    assert_eq!(args.account.value.as_deref(), Some("work"));
    let sel = args.repo.value.as_ref().expect("--repo set");
    assert_eq!(sel.category, "local");
    assert_eq!(sel.value.as_deref(), Some("/tmp/xyz"));
    assert!(args.por.value);
    assert!(args.provision.private.value);
    assert!(args.provision.dry_run.value);
    assert_eq!(args.provision.push_retry.push_retries, 10);
    assert_eq!(args.provision.push_retry.push_retry_delay, 5);
    assert_eq!(args.use_template.value.as_deref(), Some("/tmp/tmpl"));
}

#[test]
fn target_required_at_parse_time() {
    // TARGET is a required positional. Missing it errors.
    let err = Cli::try_parse_from(["vc-x1", "init"])
        .unwrap_err()
        .to_string();
    assert!(err.contains("TARGET"), "got: {err}");
}

#[test]
fn config_content_dual() {
    // Per-side variants: [repos] values are file-relative, so
    // the sides differ. The "." entry names the side.
    let work = render_vc_config(ConfigRole::DualWork {
        agent_dir: ".agent-session",
        agent_repo: Some("proj.claude"),
    });
    assert!(work.contains("work = \".\""));
    assert!(work.contains("agent = \".agent-session\""));
    // The work side records the agent-repo's remote name, since it is
    // what clone and sync read before an agent-repo exists to ask.
    // An active line, not the header comment that also names the
    // table.
    assert!(work.lines().any(|l| l.trim_start() == "[remote]"));
    assert!(work.contains("agent-repo = \"proj.claude\""));
    let bot = render_vc_config(ConfigRole::DualBot);
    assert!(bot.contains("work = \"..\""));
    assert!(bot.contains("agent = \".\""));
    assert!(!bot.lines().any(|l| l.trim_start() == "[remote]"));
}

/// No name renders no `agent-repo` line, which is the shape of every
/// workspace created before the key and reads as the work repo's name
/// plus `.claude`.
#[test]
fn config_content_dual_without_an_agent_repo_name() {
    let work = render_vc_config(ConfigRole::DualWork {
        agent_dir: ".agent-session",
        agent_repo: None,
    });
    assert!(work.contains("agent = \".agent-session\""));
    assert!(
        !work.lines().any(|l| l.trim_start() == "[remote]"),
        "no active [remote] table"
    );
}

#[test]
fn config_optional_keys_are_commented_only() {
    let work = render_vc_config(ConfigRole::DualWork {
        agent_dir: ".agent-session",
        agent_repo: None,
    });
    // The doc-block header/used-by/default lines precede the
    // commented assignment.
    assert!(work.contains("used by: agent-session --col-width"));
    assert!(work.contains("used by: agent-session --result-lines"));
    assert!(work.contains("# col-width = 68"));
    assert!(work.contains("# result-lines = 10"));
    assert!(
        !work
            .lines()
            .any(|l| l.trim_start().starts_with("col-width"))
    );
    assert!(
        !work
            .lines()
            .any(|l| l.trim_start().starts_with("result-lines"))
    );
}

#[test]
fn config_generated_toml_parses_to_active_keys_only() {
    let dir = std::env::temp_dir().join(format!(
        "vcx1-init-config-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0) // OK: test-only uniqueness suffix, and 0 fallback is harmless
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(crate::config_md::VC_CONFIG_MD);
    std::fs::write(
        &path,
        render_vc_config(ConfigRole::DualWork {
            agent_dir: ".agent-session",
            agent_repo: Some("proj.claude"),
        }),
    )
    .expect("write config");

    let map = crate::config_md::load_file(&path).expect("parse generated config");
    assert_eq!(map.get("repos.work").map(String::as_str), Some("."));
    assert_eq!(
        map.get("repos.agent").map(String::as_str),
        Some(".agent-session")
    );
    assert_eq!(
        map.get("remote.agent-repo").map(String::as_str),
        Some("proj.claude")
    );
    assert!(!map.contains_key("agent-session.col-width"));
    assert!(!map.contains_key("push.state-file"));

    let _ = std::fs::remove_dir_all(&dir);
}

/// No table header is emitted without a key under it.
///
/// A section whose keys all lack defaults (`[family]`, `[validate]`)
/// has nothing to render, and emitting its header anyway left a
/// generated config ending in two bare tables.
#[test]
fn config_generated_has_no_empty_table() {
    let rendered = render_vc_config(ConfigRole::DualWork {
        agent_dir: ".agent-session",
        agent_repo: Some("proj.claude"),
    });
    let is_header = |l: &str| l.starts_with('[') && l.ends_with(']');
    let mut lines = rendered.lines().filter(|l| !l.trim().is_empty()).peekable();
    while let Some(line) = lines.next() {
        if !is_header(line) {
            continue;
        }
        let next = lines.peek().copied().unwrap_or("```");
        assert!(
            !is_header(next) && next != "```",
            "{line} has no keys under it"
        );
    }
}

#[test]
fn gitignore_work_excludes_bot() {
    let gitignore = render_work_gitignore(".agent-session");
    assert!(gitignore.lines().any(|l| l == "/.agent-session"));
    assert!(gitignore.lines().any(|l| l == "/.git"));
    assert!(gitignore.lines().any(|l| l == "/.jj"));
    // No state dir: nothing of vc-x1's persists in a workspace.
    assert!(!gitignore.contains("/.vc-x1"));
}

#[test]
fn gitignore_bot_excludes_git() {
    assert!(GITIGNORE_SESSION.contains(".git"));
    assert!(GITIGNORE_SESSION.contains(".jj"));
}

use std::time::{SystemTime, UNIX_EPOCH};

/// `init_bare_main` creates a bare repo whose HEAD names `main`
/// regardless of the machine's `init.defaultBranch` (the in-memory
/// override, standing in for the spawned `--initial-branch=main`).
#[test]
fn init_bare_main_pins_head_to_main() {
    let base = tmp_root("bare-main");
    let bare = base.join("origin.git");
    init_bare_main(&bare).expect("init bare repo via gix");
    let head = std::fs::read_to_string(bare.join("HEAD")).expect("read HEAD");
    assert_eq!(head.trim(), "ref: refs/heads/main");
    assert!(bare.join("objects").is_dir(), "bare repo layout");
    let _ = std::fs::remove_dir_all(&base);
}

/// Create a unique temp dir for a test, sibling-style via file-name
/// concat so both the work and bot template paths can live under it.
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
    let (c, b) = parse_use_template("/a/work,/x/bot").unwrap();
    assert_eq!(c, PathBuf::from("/a/work"));
    assert_eq!(b, PathBuf::from("/x/bot"));
}

#[test]
fn parse_use_template_default_bot() {
    let (c, b) = parse_use_template("/a/work").unwrap();
    assert_eq!(c, PathBuf::from("/a/work"));
    assert_eq!(b, PathBuf::from("/a/work.claude"));
}

#[test]
fn parse_use_template_default_bot_trailing_slash() {
    // with_file_name normalises away the effect of a trailing slash.
    let (c, b) = parse_use_template("/a/work/").unwrap();
    assert_eq!(c, PathBuf::from("/a/work/"));
    assert_eq!(b, PathBuf::from("/a/work.claude"));
}

#[test]
fn parse_use_template_empty_bot_falls_back_to_default() {
    let (c, b) = parse_use_template("/a/work,").unwrap();
    assert_eq!(c, PathBuf::from("/a/work"));
    assert_eq!(b, PathBuf::from("/a/work.claude"));
}

#[test]
fn parse_use_template_empty_work_errors() {
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
    // README.md not created, call must succeed silently.
    rewrite_readme_first_line(&root, "new-name").unwrap();
    assert!(!root.join("README.md").exists());
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn validate_templates_missing_work() {
    let root = tmp_root("validate-missing-work");
    let work = root.join("nope");
    let bot = root.join("agent");
    std::fs::create_dir_all(&bot).unwrap();
    let err = validate_templates(&work, &bot).unwrap_err().to_string();
    assert!(err.contains("work template"));
    assert!(err.contains("does not exist"));
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn validate_templates_not_a_dir() {
    let root = tmp_root("validate-not-dir");
    let work = root.join("work-file");
    let bot = root.join("agent");
    std::fs::write(&work, "i am a file").unwrap();
    std::fs::create_dir_all(&bot).unwrap();
    let err = validate_templates(&work, &bot).unwrap_err().to_string();
    assert!(err.contains("is not a directory"));
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn end_to_end_copy_and_readme_rewrite() {
    // Simulates what init's Step 4 does: two sibling templates with
    // a README.md each, copied into two fresh target dirs, each
    // README retitled to the respective repo name.
    let root = tmp_root("e2e-copy-rewrite");
    let work_tmpl = root.join("vc-template-x1");
    let bot_tmpl = root.join("vc-template-x1.claude");
    let work_dst = root.join("dst-work");
    let bot_dst = root.join("dst-bot");
    std::fs::create_dir_all(&work_tmpl).unwrap();
    std::fs::create_dir_all(&bot_tmpl).unwrap();
    std::fs::create_dir_all(&work_dst).unwrap();
    std::fs::create_dir_all(&bot_dst).unwrap();

    std::fs::write(
        work_tmpl.join("README.md"),
        "# vc-template-x1\nCode template body.\n",
    )
    .unwrap();
    std::fs::write(work_tmpl.join("src.txt"), "work stuff").unwrap();
    std::fs::write(work_tmpl.join(".gitignore"), "should-not-copy").unwrap();

    std::fs::write(
        bot_tmpl.join("README.md"),
        "# vc-template-x1.claude\nBot template body.\n",
    )
    .unwrap();
    std::fs::write(bot_tmpl.join("bot.md"), "bot stuff").unwrap();

    validate_templates(&work_tmpl, &bot_tmpl).unwrap();

    copy_template_recursive(&work_tmpl, &work_dst).unwrap();
    rewrite_readme_first_line(&work_dst, "my-proj").unwrap();
    copy_template_recursive(&bot_tmpl, &bot_dst).unwrap();
    rewrite_readme_first_line(&bot_dst, "my-proj.claude").unwrap();

    assert_eq!(
        std::fs::read_to_string(work_dst.join("README.md")).unwrap(),
        "# my-proj\nCode template body.\n"
    );
    assert_eq!(
        std::fs::read_to_string(work_dst.join("src.txt")).unwrap(),
        "work stuff"
    );
    assert!(!work_dst.join(".gitignore").exists());
    assert_eq!(
        std::fs::read_to_string(bot_dst.join("README.md")).unwrap(),
        "# my-proj.claude\nBot template body.\n"
    );
    assert_eq!(
        std::fs::read_to_string(bot_dst.join("bot.md")).unwrap(),
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
    let args = parse(&["vc-x1", "init", "https://github.com/owner/repo"]);
    assert_eq!(args.target, "https://github.com/owner/repo");
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
    assert_eq!(args.account.value.as_deref(), Some("work"));
}

#[test]
fn repo_cat_only_parses() {
    let args = parse(&["vc-x1", "init", "tf1", "--repo", "remote"]);
    let sel = args.repo.value.expect("--repo set");
    assert_eq!(sel.category, "remote");
    assert!(sel.value.is_none());
}

#[test]
fn repo_cat_value_parses() {
    let args = parse(&["vc-x1", "init", "tf1", "--repo", "remote=git@github.com:u"]);
    let sel = args.repo.value.expect("--repo set");
    assert_eq!(sel.category, "remote");
    assert_eq!(sel.value.as_deref(), Some("git@github.com:u"));
}

#[test]
fn por_flag_parses() {
    let args = parse(&["vc-x1", "init", "tf1", "--por"]);
    assert!(args.por.value);
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

// Unit tests for derive_bot_url / derive_name live in
// src/repo_url.rs alongside the lifted functions.

#[test]
fn expand_vars_tilde() {
    let prev = std::env::var("HOME").ok();
    // SAFETY: env-var manipulation is genuinely racy with parallel
    // test runners. Tests touching $VAR and $HOME live in this
    // module. If flakiness emerges, run with --test-threads=1.
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

/// Build an `InitArgs` with sane defaults. The caller overrides
/// only the fields it cares about.
fn args_for(target: &str) -> InitArgs {
    InitArgs {
        target: target.to_string(),
        name: None,
        account: AccountOption::default(),
        repo: RepoOption::default(),
        por: PorFlag::default(),
        provision: ProvisionOptionFlagBundle {
            dry_run: DryRunFlag { value: true },
            ..Default::default()
        },
        use_template: UseTemplateOption::default(),
        config: ConfigOption::default(),
        adopt: false,
        agent_dir: None,
        agent_repo: None,
        agent_suffix: None,
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
        bot_session_items: None,
        bot_session_result_lines: None,
        bot_session_col_width: None,
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
        bot_session_items: None,
        bot_session_result_lines: None,
        bot_session_col_width: None,
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
        bot_session_items: None,
        bot_session_result_lines: None,
        bot_session_col_width: None,
    }
}

// ---------- the target's state ----------

/// A scratch directory for one state test, removed on drop.
struct StateDir(PathBuf);

impl StateDir {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "vcx1-init-state-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0) // OK: test-only uniqueness suffix, and 0 fallback is harmless
        ));
        std::fs::create_dir_all(&dir).expect("create state dir");
        Self(dir)
    }

    fn config(&self, body: &str) {
        let fenced = format!("```toml\n{body}```\n");
        std::fs::write(self.0.join(crate::config_md::VC_CONFIG_MD), fenced).expect("write config");
    }
}

impl Drop for StateDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn state_reads_each_shape() {
    use super::adopt::{TargetState, detect_target_state};

    let d = StateDir::new("shapes");
    assert_eq!(
        detect_target_state(&d.0.join("missing")).unwrap(),
        TargetState::Absent
    );
    std::fs::write(d.0.join("notes.txt"), "work\n").unwrap();
    assert_eq!(detect_target_state(&d.0).unwrap(), TargetState::PlainDir);

    std::fs::create_dir(d.0.join(".git")).unwrap();
    assert_eq!(detect_target_state(&d.0).unwrap(), TargetState::Por);

    d.config("[repos]\nwork = \".\"\n");
    assert_eq!(detect_target_state(&d.0).unwrap(), TargetState::SingleRepo);

    d.config("[repos]\nwork = \".\"\nagent = \".agent-session\"\n");
    assert_eq!(detect_target_state(&d.0).unwrap(), TargetState::Dual);

    // A dual workspace's agent repo declares itself as `.`, and is
    // dual too.
    d.config("[repos]\nwork = \"..\"\nagent = \".\"\n");
    assert_eq!(detect_target_state(&d.0).unwrap(), TargetState::Dual);
}

#[test]
fn state_refuses_what_no_state_describes() {
    use super::adopt::detect_target_state;

    let d = StateDir::new("refusals");
    let file = d.0.join("a-file");
    std::fs::write(&file, "x").unwrap();
    let err = detect_target_state(&file).unwrap_err().to_string();
    assert!(err.contains("not a directory"), "{err}");

    // A config with no repo beside it.
    d.config("[repos]\nwork = \".\"\n");
    let err = detect_target_state(&d.0).unwrap_err().to_string();
    assert!(err.contains("no repo"), "{err}");

    // A repo whose config has no work side.
    std::fs::create_dir(d.0.join(".jj")).unwrap();
    d.config("[family]\nmember = \"x\"\n");
    let err = detect_target_state(&d.0).unwrap_err().to_string();
    assert!(err.contains("repos.work"), "{err}");
}

#[test]
fn an_existing_target_needs_adopt() {
    use super::adopt::TargetState;
    let dir = Path::new("/x/proj");

    check_target_state(dir, &TargetState::Absent, false).expect("fresh create");
    let err = check_target_state(dir, &TargetState::Absent, true)
        .unwrap_err()
        .to_string();
    assert!(err.contains("does not exist"), "{err}");

    for state in [
        TargetState::PlainDir,
        TargetState::Por,
        TargetState::SingleRepo,
    ] {
        let err = check_target_state(dir, &state, false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("pass --adopt"), "{err}");
        assert!(err.contains(state.describe()), "{err}");
    }
    for adopting in [false, true] {
        let err = check_target_state(dir, &TargetState::Dual, adopting)
            .unwrap_err()
            .to_string();
        assert!(err.contains("nothing to adopt"), "{err}");
    }
}

#[test]
fn adopt_is_meaningless_with_por() {
    let mut args = args_for("git@github.com:winksaville/tf1");
    args.por.value = true;
    args.adopt = true;
    let err = plan_init(&InitParams::from(&args), &cfg_empty())
        .expect_err("refused")
        .to_string();
    assert!(err.contains("--adopt"), "{err}");
}

// ---------- adopting a plain directory ----------

/// Adopt a plain directory under `base/work`, with local bares under
/// `base`, and no symlink.
fn adopt_plain(base: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = args_for(&base.join("work").to_string_lossy());
    args.repo.value = Some(RepoSelector {
        category: "local".into(),
        value: Some(base.to_string_lossy().into_owned()),
    });
    args.provision.dry_run.value = false;
    args.adopt = true;
    let mut params = InitParams::from(&args);
    params.create_symlink = false;
    init(&crate::test_helpers::test_ctx(), &params)
}

/// A plain directory's content, large files included, is the work
/// repo's first commit, its `.gitignore` is kept and given the agent
/// line, and the agent repo grows beside it, both published.
#[test]
fn adopt_plain_directory_commits_its_content() {
    use crate::test_helpers::jj_ok_at;

    let base = crate::test_helpers::unique_base("adopt-plain");
    let work = base.join("work");
    std::fs::create_dir_all(work.join("pins")).unwrap();
    std::fs::write(work.join("notes.txt"), "a record\n").unwrap();
    // Over jj's 1MiB new-file limit, which a plain snapshot skips.
    std::fs::write(
        work.join("pins").join("big.jsonl"),
        vec![b'x'; 2 * 1024 * 1024],
    )
    .unwrap();
    std::fs::write(work.join(".gitignore"), "*.log").unwrap();
    std::fs::write(work.join("run.log"), "noise\n").unwrap();

    adopt_plain(&base).expect("adopt a plain directory");

    let files = jj_ok_at(&work, &["file", "list", "-r", "@-"]);
    for tracked in [".gitignore", ".vc-config.md", "notes.txt", "pins/big.jsonl"] {
        assert!(
            files.lines().any(|l| l == tracked),
            "{tracked} tracked: {files}"
        );
    }
    assert!(
        !files.lines().any(|l| l == "run.log"),
        "ignored stays out: {files}"
    );
    assert!(
        !files.lines().any(|l| l.starts_with(".agent-session")),
        "the agent repo is not in the work repo: {files}"
    );

    let gitignore = std::fs::read_to_string(work.join(".gitignore")).unwrap();
    assert_eq!(
        gitignore, "*.log\n/.agent-session\n",
        "kept, and given the line"
    );

    let agent = work.join(".agent-session");
    assert!(agent.join(".jj").exists(), "agent repo created");
    assert!(base.join("remote-work.git").exists());
    assert!(base.join("remote-work.agent-session.git").exists());
    assert_eq!(
        crate::common::configured_agent_repo(&work)
            .unwrap()
            .as_deref(),
        Some("remote-work.agent-session")
    );
    // Cross-linked like a fresh init's.
    let desc = crate::test_helpers::description(&work, "@-");
    assert!(desc.contains("ochid: /.claude/"), "{desc}");

    let _ = std::fs::remove_dir_all(&base);
}

/// An adopted `.gitignore` that already names the agent directory is
/// left as it is.
#[test]
fn adopt_keeps_a_gitignore_that_has_the_line() {
    let base = crate::test_helpers::unique_base("adopt-gitignore");
    let work = base.join("work");
    std::fs::create_dir_all(&work).unwrap();
    std::fs::write(work.join(".gitignore"), "/.agent-session\n/target\n").unwrap();
    adopt_plain(&base).expect("adopt");
    assert_eq!(
        std::fs::read_to_string(work.join(".gitignore")).unwrap(),
        "/.agent-session\n/target\n"
    );
    let _ = std::fs::remove_dir_all(&base);
}

/// A plain directory already holding the agent directory is refused
/// before anything is written.
#[test]
fn adopt_refuses_an_existing_agent_directory() {
    let base = crate::test_helpers::unique_base("adopt-agent-exists");
    let work = base.join("work");
    std::fs::create_dir_all(work.join(".agent-session")).unwrap();
    let err = adopt_plain(&base).expect_err("refused").to_string();
    assert!(err.contains("--agent-dir"), "{err}");
    assert!(!work.join(".jj").exists(), "nothing written");
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn adopt_refuses_a_template() {
    let mut args = args_for("git@github.com:winksaville/tf1");
    args.adopt = true;
    args.use_template.value = Some("../tmpl".into());
    let err = plan_init(&InitParams::from(&args), &cfg_empty())
        .expect_err("refused")
        .to_string();
    assert!(err.contains("--use-template"), "{err}");
}

// ---------- the steps, numbered as they run ----------

/// A dual plan runs ten steps in order, the symlink last, and drops
/// the symlink when it is turned off rather than printing it skipped.
#[test]
fn steps_dual_run_in_order() {
    use super::steps::{Step, Steps};
    let args = args_for("git@github.com:winksaville/tf1");
    let params = InitParams::from(&args);
    let plan = plan_init(&params, &cfg_empty()).unwrap();
    let all = Steps::new(&plan, &params, None, "--public", true);
    assert_eq!(
        all.steps(),
        vec![
            Step::PrepareWork,
            Step::ConfigWork,
            Step::CommitWork,
            Step::PrepareAgent,
            Step::ConfigAgent,
            Step::CommitAgent,
            Step::CrossLink,
            Step::PublishAgent,
            Step::PublishWork,
            Step::Symlink,
        ]
    );
    let no_symlink = Steps::new(&plan, &params, None, "--public", false);
    assert_eq!(no_symlink.steps().len(), 9);
    assert!(!no_symlink.steps().contains(&Step::Symlink));
}

/// A single-repo plan runs four steps, with no gaps where the agent
/// side's would be.
#[test]
fn steps_por_have_no_gaps() {
    use super::steps::{Step, Steps};
    let mut args = args_for("git@github.com:winksaville/tf1");
    args.por.value = true;
    let params = InitParams::from(&args);
    let plan = plan_init(&params, &cfg_empty()).unwrap();
    assert_eq!(
        Steps::new(&plan, &params, None, "--public", true).steps(),
        vec![
            Step::PrepareWork,
            Step::ConfigWork,
            Step::CommitWork,
            Step::PublishWork,
        ]
    );
}

// ---------- adopting a repo ----------

/// Adopt the repo at `work`, with no `--repo`, and no symlink.
fn adopt_repo(work: &Path, repo: Option<RepoSelector>) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = args_for(&work.to_string_lossy());
    args.repo.value = repo;
    args.provision.dry_run.value = false;
    args.adopt = true;
    let mut params = InitParams::from(&args);
    params.create_symlink = false;
    init(&crate::test_helpers::test_ctx(), &params)
}

/// A repo with an origin keeps its history, gains one commit that
/// carries the config, and is not pushed, while the agent repo is
/// created beside the origin and published.
#[test]
fn adopt_repo_with_origin_keeps_history_and_pushes_nothing() {
    use crate::test_helpers::{FixturePor, chid, description, jj_ok};

    let fx = FixturePor::new_with_config("adopt-por-origin", Some("none".into()));
    let first = chid(&fx.work, "@-");
    let origin_main = jj_ok(
        &fx.work,
        &["log", "--no-graph", "-r", "main@origin", "-T", "commit_id"],
    );

    adopt_repo(&fx.work, None).expect("adopt a repo with an origin");

    let desc = description(&fx.work, "@-");
    assert!(desc.starts_with(ADOPT_TITLE), "{desc}");
    assert!(desc.contains("ochid: /.claude/"), "{desc}");
    assert_eq!(
        chid(&fx.work, "@--"),
        first,
        "history kept under the adopt commit"
    );
    assert_eq!(
        jj_ok(
            &fx.work,
            &["log", "--no-graph", "-r", "main@origin", "-T", "commit_id"]
        ),
        origin_main,
        "the work side pushed nothing"
    );

    // The agent repo's remote is derived beside the origin.
    let agent_bare = fx.base.join("remote.agent-session.git");
    assert!(agent_bare.exists(), "agent bare created beside the origin");
    assert_eq!(
        crate::common::configured_agent_repo(&fx.work)
            .unwrap()
            .as_deref(),
        Some("remote.agent-session")
    );
    let agent = fx.work.join(".agent-session");
    let agent_desc = description(&agent, "@-");
    assert!(
        agent_desc.contains(&format!("ochid: /{}", chid(&fx.work, "@-"))),
        "{agent_desc}"
    );
}

/// A repo with no origin gets its remotes from `--repo`, as a fresh
/// init does, and its work side is pushed.
#[test]
fn adopt_repo_without_origin_publishes_both() {
    use crate::test_helpers::jj_ok;

    let base = crate::test_helpers::unique_base("adopt-por-no-origin");
    let work = base.join("work");
    std::fs::create_dir_all(&work).unwrap();
    crate::jj::git_init_colocated(&work).unwrap();
    std::fs::write(work.join("a.txt"), "a\n").unwrap();
    jj_ok(&work, &["commit", "-m", "first"]);

    adopt_repo(
        &work,
        Some(RepoSelector {
            category: "local".into(),
            value: Some(base.to_string_lossy().into_owned()),
        }),
    )
    .expect("adopt a repo with no origin");

    assert!(base.join("remote-work.git").exists());
    assert!(base.join("remote-work.agent-session.git").exists());
    assert_eq!(
        crate::test_helpers::description(&work, "@--").trim(),
        "first",
        "history kept"
    );
    let _ = std::fs::remove_dir_all(&base);
}

/// Uncommitted work, a git-only repo, and `--repo` against an origin
/// are refused before anything is written.
#[test]
fn adopt_repo_refusals() {
    use crate::test_helpers::FixturePor;

    let fx = FixturePor::new_with_config("adopt-por-refuse", Some("none".into()));
    let err = adopt_repo(
        &fx.work,
        Some(RepoSelector {
            category: "local".into(),
            value: Some(fx.base.to_string_lossy().into_owned()),
        }),
    )
    .expect_err("--repo against an origin")
    .to_string();
    assert!(err.contains("--repo is meaningless"), "{err}");

    std::fs::write(fx.work.join("wip.txt"), "wip\n").unwrap();
    let err = adopt_repo(&fx.work, None).expect_err("dirty").to_string();
    assert!(err.contains("uncommitted work"), "{err}");
    assert!(!fx.work.join(".agent-session").exists(), "nothing written");

    let base = crate::test_helpers::unique_base("adopt-git-only");
    let work = base.join("work");
    std::fs::create_dir_all(work.join(".git")).unwrap();
    let err = adopt_repo(&work, None).expect_err("git only").to_string();
    assert!(err.contains("jj git init --colocate"), "{err}");
    let _ = std::fs::remove_dir_all(&base);
}

// ---------- adopting a single-repo workspace ----------

/// The config `init --por` writes gains the two keys, and every line
/// it had stays, in order.
#[test]
fn add_agent_keys_edits_the_por_config_in_place() {
    use super::adopt::add_agent_keys;
    let before = render_vc_config(ConfigRole::WorkOnly);
    let after = add_agent_keys(&before, true, ".agent-session", "proj.agent-session").unwrap();

    let mut kept = after.lines();
    for line in before.lines() {
        assert!(kept.any(|l| l == line), "kept in order: {line}");
    }
    let path = std::env::temp_dir().join(format!("vcx1-add-agent-keys-{}.md", std::process::id()));
    std::fs::write(&path, &after).unwrap();
    let map = crate::config_md::load_file(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!(map.get("repos.work").map(String::as_str), Some("."));
    assert_eq!(
        map.get("repos.agent").map(String::as_str),
        Some(".agent-session")
    );
    assert_eq!(
        map.get("remote.agent-repo").map(String::as_str),
        Some("proj.agent-session")
    );
}

/// An existing `[remote]` header takes the key, a plain TOML file is
/// edited without fences, and a file with no `[repos]` key is refused.
#[test]
fn add_agent_keys_shapes() {
    use super::adopt::add_agent_keys;
    let toml = "[remote]\n\n[repos]\nwork = \".\"\n";
    assert_eq!(
        add_agent_keys(toml, false, ".s", "p.s").unwrap(),
        "[remote]\nagent-repo = \"p.s\"\n\n[repos]\nwork = \".\"\nagent = \".s\"\n"
    );
    let err = add_agent_keys("[family]\nmember = \"x\"\n", false, ".s", "p.s").unwrap_err();
    assert!(err.contains("[repos]"), "{err}");
    // A fence's TOML is read, and prose that looks like a table is not.
    let md = "Prose [repos] here.\n\n```toml\n[repos]\nwork = \".\"\n```\n";
    let out = add_agent_keys(md, true, ".s", "p.s").unwrap();
    assert!(out.starts_with("Prose [repos] here.\n"), "{out}");
    assert!(
        out.contains("work = \".\"\nagent = \".s\"\n\n[remote]\nagent-repo = \"p.s\"\n```"),
        "{out}"
    );
}

/// The single-repo workspace `init --por` makes is adopted: its config
/// is edited in place, its history stays, and nothing is pushed on
/// the work side.
#[test]
fn adopt_single_repo_workspace() {
    use crate::test_helpers::{FixturePor, chid, description, jj_ok};

    let fx = FixturePor::new("adopt-single-repo");
    let before = std::fs::read_to_string(fx.work.join(".vc-config.md")).unwrap();
    let first = chid(&fx.work, "@-");
    let origin_main = jj_ok(
        &fx.work,
        &["log", "--no-graph", "-r", "main@origin", "-T", "commit_id"],
    );

    adopt_repo(&fx.work, None).expect("adopt a single-repo workspace");

    let after = std::fs::read_to_string(fx.work.join(".vc-config.md")).unwrap();
    let mut kept = after.lines();
    for line in before.lines() {
        assert!(kept.any(|l| l == line), "config line kept: {line}");
    }
    assert_eq!(
        crate::common::configured_bot_dir(&fx.work).unwrap(),
        Some(fx.work.join(".agent-session"))
    );
    assert_eq!(
        crate::common::configured_agent_repo(&fx.work)
            .unwrap()
            .as_deref(),
        Some("remote.agent-session")
    );
    assert!(description(&fx.work, "@-").starts_with(ADOPT_TITLE));
    assert_eq!(chid(&fx.work, "@--"), first, "history kept");
    assert_eq!(
        jj_ok(
            &fx.work,
            &["log", "--no-graph", "-r", "main@origin", "-T", "commit_id"]
        ),
        origin_main
    );
    assert!(fx.base.join("remote.agent-session.git").exists());
}

// ---------- the agent side's names ----------

/// `--agent-dir` names the directory, and the work config records it.
#[test]
fn plan_agent_dir_names_the_directory() {
    let mut args = args_for("git@github.com:winksaville/tf1");
    args.agent_dir = Some(".sess".into());
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert_eq!(plan.agent.as_ref().map(|a| a.dir.as_str()), Some(".sess"));
    let cwd = std::env::current_dir().unwrap();
    assert_eq!(
        plan.agent.as_ref().map(|a| a.path.clone()),
        Some(cwd.join("tf1").join(".sess"))
    );
    // The remote name does not follow the directory.
    assert_eq!(
        plan.agent.as_ref().map(|a| a.url.as_str()),
        Some("git@github.com:winksaville/tf1.agent-session.git")
    );
}

/// `--agent-suffix` replaces the default suffix on the remote name.
#[test]
fn plan_agent_suffix_names_the_remote() {
    let mut args = args_for("git@github.com:winksaville/tf1");
    args.agent_suffix = Some("-agent".into());
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert_eq!(
        plan.agent.as_ref().map(|a| a.url.as_str()),
        Some("git@github.com:winksaville/tf1-agent.git")
    );
    assert_eq!(
        plan.agent.as_ref().and_then(|a| a.gh_slug.as_deref()),
        Some("winksaville/tf1-agent")
    );
    assert_eq!(
        plan.agent.as_ref().map(|a| a.dir.as_str()),
        Some(".agent-session")
    );
}

/// `--agent-repo` is the whole remote name, under the work repo's
/// owner, and a local bare takes it too.
#[test]
fn plan_agent_repo_is_the_whole_remote_name() {
    let mut args = args_for("git@github.com:winksaville/tf1");
    args.agent_repo = Some("tf1.claude".into());
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert_eq!(
        plan.agent.as_ref().map(|a| a.url.as_str()),
        Some("git@github.com:winksaville/tf1.claude.git")
    );
    assert_eq!(
        plan.agent.as_ref().map(|a| a.name.as_str()),
        Some("tf1.claude")
    );

    let mut args = args_for("/tmp/xyz/tf1");
    args.repo.value = Some(RepoSelector {
        category: "local".into(),
        value: Some("/tmp/xyz".into()),
    });
    args.agent_repo = Some("sessions".into());
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert_eq!(
        plan.agent.as_ref().and_then(|a| a.bare_path.clone()),
        Some(PathBuf::from("/tmp/xyz/sessions.git"))
    );
}

/// `--agent-repo` and `--agent-suffix` both name the remote, so the
/// pair is refused.
#[test]
fn agent_repo_conflicts_with_agent_suffix() {
    let err = Cli::try_parse_from([
        "vc-x1",
        "init",
        "tf1",
        "--agent-repo",
        "x",
        "--agent-suffix",
        ".y",
    ])
    .expect_err("the pair is refused");
    assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
}

/// A suffix opens with `.` or `-` and names something after it.
#[test]
fn agent_suffix_must_open_with_a_dot_or_dash() {
    for bad in ["agent", ".", "-", "_agent", ".a/b"] {
        assert!(
            Cli::try_parse_from(["vc-x1", "init", "tf1", "--agent-suffix", bad]).is_err(),
            "{bad:?} is refused"
        );
    }
    for good in [".agent-session", "-agent"] {
        let args = parse(&["vc-x1", "init", "tf1", "--agent-suffix", good]);
        assert_eq!(args.agent_suffix.as_deref(), Some(good));
    }
}

/// A directory is one name inside the project, never the work
/// repo's own.
#[test]
fn agent_dir_is_one_name_inside_the_project() {
    for bad in ["", ".", "..", "a/b", ".git", ".jj"] {
        assert!(
            Cli::try_parse_from(["vc-x1", "init", "tf1", "--agent-dir", bad]).is_err(),
            "{bad:?} is refused"
        );
    }
    let args = parse(&["vc-x1", "init", "tf1", "--agent-dir", ".sess"]);
    assert_eq!(args.agent_dir.as_deref(), Some(".sess"));
}

/// A POR has no agent side, so its flags are refused.
#[test]
fn agent_flags_are_meaningless_with_por() {
    let mut args = args_for("git@github.com:winksaville/tf1");
    args.por.value = true;
    args.agent_dir = Some(".sess".into());
    let err = plan_init(&InitParams::from(&args), &cfg_empty())
        .expect_err("refused")
        .to_string();
    assert!(err.contains("--agent-dir"), "{err}");
    assert!(err.contains("--por"), "{err}");
}

// ---------- URL TARGET ----------

#[test]
fn plan_url_ssh_github_dual() {
    let args = args_for("git@github.com:winksaville/tf1");
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert_eq!(plan.provisioner, Provisioner::GhCreate);
    assert_eq!(plan.name, "tf1");
    assert_eq!(plan.work_url, "git@github.com:winksaville/tf1.git");
    assert_eq!(
        plan.agent.as_ref().map(|a| a.url.as_str()),
        Some("git@github.com:winksaville/tf1.agent-session.git")
    );
    assert_eq!(plan.gh_work_slug.as_deref(), Some("winksaville/tf1"));
    assert_eq!(
        plan.agent.as_ref().and_then(|a| a.gh_slug.as_deref()),
        Some("winksaville/tf1.agent-session")
    );
}

#[test]
fn plan_url_https_github() {
    let args = args_for("https://github.com/owner/repo.git");
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert_eq!(plan.provisioner, Provisioner::GhCreate);
    assert_eq!(plan.work_url, "https://github.com/owner/repo.git");
    assert_eq!(plan.gh_work_slug.as_deref(), Some("owner/repo"));
}

#[test]
fn plan_url_non_github_uses_external_provisioner() {
    let args = args_for("git@gitlab.com:winksaville/tf1.git");
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert_eq!(plan.provisioner, Provisioner::ExternalPreExisting);
    assert!(plan.gh_work_slug.is_none());
    assert!(
        plan.agent
            .as_ref()
            .and_then(|a| a.gh_slug.as_ref())
            .is_none()
    );
    assert_eq!(plan.work_url, "git@gitlab.com:winksaville/tf1.git");
    assert_eq!(
        plan.agent.as_ref().map(|a| a.url.as_str()),
        Some("git@gitlab.com:winksaville/tf1.agent-session.git")
    );
}

#[test]
fn plan_url_with_name_override() {
    // [NAME] overrides the URL-derived dir name. Remote URL
    // itself is unchanged.
    let mut args = args_for("git@github.com:winksaville/tf1");
    args.name = Some("custom-dir".into());
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert_eq!(plan.name, "custom-dir");
    let cwd = std::env::current_dir().unwrap();
    assert_eq!(plan.project_dir, cwd.join("custom-dir"));
    assert_eq!(plan.work_url, "git@github.com:winksaville/tf1.git");
}

// ---------- the retired owner/name shorthand ----------

/// A slashed TARGET with no path prefix is refused before a plan
/// exists, so nothing reaches GitHub under a guessed reading.
#[test]
fn plan_slashed_target_is_refused() {
    let err = plan_init(
        &InitParams::from(&args_for("winksaville/tf1")),
        &cfg_empty(),
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("ambiguous"), "{err}");
    assert!(err.contains("./winksaville/tf1"), "{err}");
}

/// A `foo.git` directory yields the repo name `foo`, the same
/// normalization a URL target gets. The two branches disagreeing
/// is what made a `./tmp/xx1.git` target ask GitHub for `xx1.git`
/// and then write a remote pointing at a repo GitHub never made.
#[test]
fn plan_path_target_strips_the_git_suffix() {
    let args = args_for("./tmp/xx1.git");
    let plan = plan_init(&InitParams::from(&args), &cfg_top_level_local("./tmp/bare")).unwrap();
    assert_eq!(plan.name, "xx1");
    assert_eq!(
        plan.agent.as_ref().map(|a| a.name.as_str()),
        Some("xx1.agent-session")
    );
}

/// A repo name GitHub would rename is refused before it is asked
/// for, since GitHub drops a trailing `.git` at creation and the
/// remote we write afterwards would point at a repo that does not
/// exist.
#[test]
fn github_slug_refuses_a_dot_git_name() {
    let err = github_slug_from_url("https://github.com/owner/foo.git.git")
        .unwrap_err()
        .to_string();
    assert!(err.contains("GitHub drops"), "{err}");
    assert!(err.contains("'foo'"), "{err}");
}

#[test]
fn plan_ssh_url_form_still_works() {
    let p2 = plan_init(
        &InitParams::from(&args_for("git@github.com:winksaville/tf1")),
        &cfg_empty(),
    )
    .unwrap();
    assert_eq!(p2.work_url, "git@github.com:winksaville/tf1.git");
    assert_eq!(p2.gh_work_slug.as_deref(), Some("winksaville/tf1"));
    assert_eq!(p2.provisioner, Provisioner::GhCreate);
    assert_eq!(
        p2.agent.as_ref().and_then(|a| a.gh_slug.as_deref()),
        Some("winksaville/tf1.agent-session")
    );
}

// ---------- Path TARGET ----------

#[test]
fn plan_path_absolute_with_repo_local() {
    let mut args = args_for("/tmp/xyz/tf1");
    args.repo.value = Some(RepoSelector {
        category: "local".into(),
        value: Some("/tmp/xyz".into()),
    });
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert_eq!(plan.provisioner, Provisioner::LocalBareInit);
    assert_eq!(plan.project_dir, PathBuf::from("/tmp/xyz/tf1"));
    assert_eq!(plan.name, "tf1");
    assert_eq!(
        plan.work_bare_path,
        Some(PathBuf::from("/tmp/xyz/remote-work.git"))
    );
    assert_eq!(
        plan.agent.as_ref().and_then(|a| a.bare_path.clone()),
        Some(PathBuf::from("/tmp/xyz/remote-work.agent-session.git"))
    );
    assert_eq!(
        plan.agent.as_ref().map(|a| a.path.clone()),
        Some(PathBuf::from("/tmp/xyz/tf1/.agent-session"))
    );
    assert_eq!(
        plan.agent.as_ref().map(|a| a.name.as_str()),
        Some("tf1.agent-session")
    );
}

#[test]
fn plan_path_relative_with_repo_local() {
    let mut args = args_for("./tf1");
    args.repo.value = Some(RepoSelector {
        category: "local".into(),
        value: Some("/tmp/xyz".into()),
    });
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
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
    let plan = plan_init(&InitParams::from(&args), &cfg).unwrap();
    let cwd = std::env::current_dir().unwrap();
    assert_eq!(plan.project_dir, cwd.join("tf1"));
    assert_eq!(plan.name, "tf1");
    assert_eq!(plan.work_url, "git@github.com:winksaville/tf1.git");
    assert_eq!(plan.provisioner, Provisioner::GhCreate);
    assert_eq!(plan.gh_work_slug.as_deref(), Some("winksaville/tf1"));
}

#[test]
fn plan_bare_name_uses_top_level_repo_local() {
    let args = args_for("tf1");
    let cfg = cfg_top_level_local("/tmp/fixtures");
    let plan = plan_init(&InitParams::from(&args), &cfg).unwrap();
    let cwd = std::env::current_dir().unwrap();
    assert_eq!(plan.project_dir, cwd.join("tf1"));
    assert_eq!(plan.provisioner, Provisioner::LocalBareInit);
    assert_eq!(
        plan.work_bare_path,
        Some(PathBuf::from("/tmp/fixtures/remote-work.git"))
    );
    assert_eq!(
        plan.agent.as_ref().and_then(|a| a.bare_path.clone()),
        Some(PathBuf::from("/tmp/fixtures/remote-work.agent-session.git"))
    );
}

#[test]
fn plan_bare_name_account_override_picks_work() {
    let mut args = args_for("tf1");
    args.account.value = Some("work".into());
    let plan = plan_init(&InitParams::from(&args), &cfg_two_accounts()).unwrap();
    assert_eq!(plan.work_url, "git@github.com:anthropic/tf1.git");
    assert_eq!(plan.gh_work_slug.as_deref(), Some("anthropic/tf1"));
}

#[test]
fn plan_bare_name_explicit_repo_value_skips_config() {
    // --repo cat=val short-circuits resolve_repo. Works even
    // with an empty config.
    let mut args = args_for("tf1");
    args.repo.value = Some(RepoSelector {
        category: "local".into(),
        value: Some("/tmp/explicit".into()),
    });
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert_eq!(plan.provisioner, Provisioner::LocalBareInit);
    assert_eq!(
        plan.work_bare_path,
        Some(PathBuf::from("/tmp/explicit/remote-work.git"))
    );
}

// ---------- Scope (POR vs CodeBot) ----------

#[test]
fn plan_por_path_local_single_bare() {
    let mut args = args_for("/tmp/xyz/tf1");
    args.por.value = true;
    args.repo.value = Some(RepoSelector {
        category: "local".into(),
        value: Some("/tmp/xyz".into()),
    });
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert!(plan.scope.is_work_only());
    assert_eq!(plan.provisioner, Provisioner::LocalBareInit);
    assert_eq!(
        plan.work_bare_path,
        Some(PathBuf::from("/tmp/xyz/remote.git"))
    );
    assert_eq!(plan.work_url, "/tmp/xyz/remote.git");
    assert!(plan.agent.is_none());
}

#[test]
fn plan_por_url_no_bot() {
    let mut args = args_for("git@github.com:winksaville/tf1");
    args.por.value = true;
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert!(plan.scope.is_work_only());
    assert_eq!(plan.work_url, "git@github.com:winksaville/tf1.git");
    assert!(plan.agent.is_none());
}

#[test]
fn plan_default_scope_is_work_bot() {
    let args = args_for("git@github.com:winksaville/tf1");
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert!(plan.scope.is_both());
    assert!(plan.agent.is_some());
}

// ---------- Errors ----------

#[test]
fn error_url_target_with_account() {
    let mut args = args_for("git@github.com:u/p");
    args.account.value = Some("work".into());
    let err = plan_init(&InitParams::from(&args), &cfg_empty())
        .unwrap_err()
        .to_string();
    assert!(err.contains("--account is meaningless"), "got: {err}");
}

#[test]
fn error_url_target_with_repo() {
    let mut args = args_for("git@github.com:u/p");
    args.repo.value = Some(RepoSelector {
        category: "remote".into(),
        value: Some("git@github.com:other".into()),
    });
    let err = plan_init(&InitParams::from(&args), &cfg_empty())
        .unwrap_err()
        .to_string();
    assert!(err.contains("--repo is meaningless"), "got: {err}");
}

#[test]
fn error_path_target_with_name() {
    let mut args = args_for("./tf1");
    args.name = Some("custom".into());
    args.repo.value = Some(RepoSelector {
        category: "local".into(),
        value: Some("/tmp/xyz".into()),
    });
    let err = plan_init(&InitParams::from(&args), &cfg_empty())
        .unwrap_err()
        .to_string();
    assert!(err.contains("[NAME] is meaningless"), "got: {err}");
    assert!(err.contains("path"), "got: {err}");
}

#[test]
fn error_bare_name_target_with_name() {
    let mut args = args_for("tf1");
    args.name = Some("custom".into());
    args.repo.value = Some(RepoSelector {
        category: "local".into(),
        value: Some("/tmp/xyz".into()),
    });
    let err = plan_init(&InitParams::from(&args), &cfg_empty())
        .unwrap_err()
        .to_string();
    assert!(err.contains("[NAME] is meaningless"), "got: {err}");
    assert!(err.contains("bare-NAME"), "got: {err}");
}

#[test]
fn error_bare_name_no_config() {
    // Empty config + no --repo, no --account -> step 1 of
    // resolve_repo errors with the "no account" message.
    let args = args_for("tf1");
    let err = plan_init(&InitParams::from(&args), &cfg_empty())
        .unwrap_err()
        .to_string();
    assert!(err.contains("[default].account"), "got: {err}");
}

#[test]
fn error_unknown_category() {
    let mut args = args_for("tf1");
    args.repo.value = Some(RepoSelector {
        category: "weird".into(),
        value: Some("xyz".into()),
    });
    let err = plan_init(&InitParams::from(&args), &cfg_empty())
        .unwrap_err()
        .to_string();
    assert!(err.contains("'weird' is not recognized"), "got: {err}");
}

#[test]
fn error_por_with_comma_template() {
    // --scope=por + --use-template foo,bar is ambiguous: bot
    // half has no home in a single-repo workspace.
    let mut args = args_for("git@github.com:u/p");
    args.por.value = true;
    args.use_template.value = Some("/tmp/work,/tmp/bot".into());
    let err = plan_init(&InitParams::from(&args), &cfg_empty())
        .unwrap_err()
        .to_string();
    assert!(err.contains("--por"), "got: {err}");
    assert!(err.contains("single template path"), "got: {err}");
}

// ---------- Constants ----------

#[test]
fn config_content_work_only() {
    let work_only_repo = render_vc_config(ConfigRole::WorkOnly);
    assert!(work_only_repo.contains("work = \".\""));
    assert!(!work_only_repo.contains("bot ="));
}

#[test]
fn gitignore_work_only_omits_bot() {
    assert!(!GITIGNORE_APP_ONLY.contains("/.agent-session"));
    assert!(GITIGNORE_APP_ONLY.contains("/.git"));
    assert!(GITIGNORE_APP_ONLY.contains("/.jj"));
    assert!(!GITIGNORE_APP_ONLY.contains("/.vc-x1"));
}

// ---------- POR end-to-end fixture (drives init with --scope=por) ----------

/// POR fixture builds without panic and lays down the
/// single-repo tree: `<base>/work/` exists, no `.agent-session/`
/// peer, bare origin sits at `<base>/remote.git` (not the
/// dual `remote-work.git` / `remote-work.agent-session.git` pair).
#[test]
fn por_fixture_creates_single_repo_layout() {
    let fx = crate::test_helpers::FixturePor::new("por-layout");

    assert!(fx.work.exists(), "work dir should exist");
    assert!(fx.work.is_dir(), "work should be a directory");
    assert!(
        !fx.work.join(".agent-session").exists(),
        "POR layout must not have a .agent-session/ peer"
    );
    assert!(
        fx.base.join("remote.git").exists(),
        "POR uses <base>/remote.git as the bare origin"
    );
    assert!(
        !fx.base.join("remote-work.git").exists(),
        "dual-shape bares should be absent in POR"
    );
    assert!(
        !fx.base.join("remote-work.agent-session.git").exists(),
        "dual-shape bares should be absent in POR"
    );
}

/// POR fixture writes the WorkOnly config + .gitignore variants:
/// `work = "."` with no `agent` key, and `.gitignore` has no
/// `/.agent-session` exclusion.
#[test]
fn por_fixture_writes_work_only_config_files() {
    let fx = crate::test_helpers::FixturePor::new("por-config");

    let cfg = std::fs::read_to_string(fx.work.join(crate::config_md::VC_CONFIG_MD))
        .expect("read the config");
    assert!(cfg.contains("work = \".\""), "expected POR work = \".\"");
    assert!(
        !cfg.contains("bot ="),
        "POR config must not declare a bot repo"
    );

    let gi = std::fs::read_to_string(fx.work.join(".gitignore")).expect("read .gitignore");
    assert!(
        !gi.contains("/.agent-session"),
        "POR .gitignore must not exclude /.agent-session"
    );
    assert!(gi.contains("/.git"), "expected /.git entry");
    assert!(gi.contains("/.jj"), "expected /.jj entry");
}

/// POR fixture has a `main` bookmark tracking `origin/main`:
/// pins step 10 (re-init jj + bookmark track) ran successfully.
#[test]
fn por_fixture_main_tracks_origin() {
    let fx = crate::test_helpers::FixturePor::new("por-tracking");

    crate::common::verify_tracking(&fx.work, "main")
        .expect("main should track origin/main after init step 10");
}

// ---------- --config flag (POR only) ----------

/// `--config none` skips writing the config file while still
/// writing `.gitignore`. The repo gets created and pushed
/// successfully: config-less repos remain valid POR shape from
/// jj/git's perspective. Downstream commands that need a config
/// will fail loudly when they try to read it.
#[test]
fn por_config_none_skips_vc_config_writes_gitignore() {
    let fx = crate::test_helpers::FixturePor::new_with_config(
        "por-config-none",
        Some("none".to_string()),
    );

    assert!(
        !fx.work.join(crate::config_md::VC_CONFIG_MD).exists(),
        "--config none must skip the config file"
    );
    assert!(
        fx.work.join(".gitignore").exists(),
        "--config none must still write .gitignore"
    );
}

/// `--config <path>` copies the user-supplied file bytewise, under
/// the name its carrier calls for: a `.toml` source stays
/// `.vc-config.toml`, which is what this case covers. `.gitignore`
/// is still written from the canned source.
#[test]
fn por_config_path_copies_user_file() {
    let base = crate::test_helpers::unique_base("por-config-path");
    std::fs::create_dir_all(&base).expect("create base");
    let custom = base.join("custom-config.toml");
    let custom_body = "# custom user config\n[workspace]\npath = \"/\"\ncustom = true\n";
    std::fs::write(&custom, custom_body).expect("write custom config");

    let fx = crate::test_helpers::FixturePor::new_with_config(
        "por-config-path",
        Some(custom.to_string_lossy().into_owned()),
    );

    let written =
        std::fs::read_to_string(fx.work.join(".vc-config.toml")).expect("read .vc-config.toml");
    assert_eq!(written, custom_body, "user config must be copied verbatim");
    assert!(
        fx.work.join(".gitignore").exists(),
        "--config <path> must still write .gitignore"
    );
}

/// `--config` without `--por` (i.e. the default dual shape) is
/// rejected at preflight.
#[test]
fn config_rejected_without_por() {
    let mut args = args_for("./foo");
    args.por.value = false;
    args.config.value = Some("none".to_string());
    let err = plan_init(&InitParams::from(&args), &cfg_empty())
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("--config is only valid with --por"),
        "unexpected error: {err}"
    );
}

/// `--config <missing-path>` errors at preflight, not at write
/// time, so the user gets a clear diagnostic before any
/// repo-mutating side effects start. The arg is meaningful only
/// under `--por`. Setting `args.por.value = true` keeps the
/// preflight on the path-validation branch.
#[test]
fn config_path_missing_rejected_at_preflight() {
    let mut args = args_for("./foo");
    args.por.value = true;
    args.config.value = Some("/nonexistent/path/to/config.toml".to_string());
    let err = plan_init(&InitParams::from(&args), &cfg_empty())
        .unwrap_err()
        .to_string();
    assert!(err.contains("does not exist"), "unexpected error: {err}");
}

/// `--config none` with `--por` passes preflight (it's the
/// happy path: `none` is a literal keyword, not a path). URL
/// target sidesteps the account-config lookup that plan_init
/// would trigger for path-form targets in cfg_empty.
#[test]
fn config_none_passes_preflight() {
    let mut args = args_for("git@github.com:foo/bar.git");
    args.por.value = true;
    args.config.value = Some("none".to_string());
    plan_init(&InitParams::from(&args), &cfg_empty())
        .expect("--config none with --scope=por should pass preflight");
}

// ---------- Dual end-to-end fixture (drives init with --scope=work,bot) ----------
//
// Counterparts to the POR fixture tests above, and pin the dual-shape
// invariants `push_repo` must preserve (-6.3 extraction).

/// Dual fixture lays down both repos and both bare origins:
/// `<base>/work/`, `<base>/work/.agent-session/`, `<base>/remote-work.git`,
/// `<base>/remote-work.agent-session.git`. POR-shape `remote.git` is absent.
#[test]
fn dual_fixture_creates_dual_repo_layout() {
    let fx = crate::test_helpers::Fixture::new("dual-layout");

    assert!(fx.work.exists() && fx.work.is_dir(), "work dir present");
    assert!(
        fx.bot.exists() && fx.bot.is_dir(),
        "nested .agent-session dir present"
    );
    assert!(
        fx.base.join("remote-work.git").exists(),
        "work-side bare origin present"
    );
    assert!(
        fx.base.join("remote-work.agent-session.git").exists(),
        "bot-side bare origin present"
    );
    assert!(
        !fx.base.join("remote.git").exists(),
        "POR-shape bare must not appear in dual layout"
    );
}

/// Dual fixture writes the WORK / BOT config + .gitignore
/// variants: per-side `[repos]` registries:
/// - work side `work = "."`, `bot = ".agent-session"`
/// - bot side `work = ".."`, `bot = "."`
///
/// Side detection is by self-resolution.
/// Work-side `.gitignore` excludes `/.agent-session` (bot subdir is
/// git-ignored from the work-side view).
#[test]
fn dual_fixture_writes_work_and_bot_config_files() {
    let fx = crate::test_helpers::Fixture::new("dual-config");

    let work_cfg = std::fs::read_to_string(fx.work.join(crate::config_md::VC_CONFIG_MD))
        .expect("read the work config");
    assert!(work_cfg.contains("work = \".\""), "work work = \".\"");
    assert!(
        work_cfg.contains("agent = \".agent-session\""),
        "work agent = \".agent-session\""
    );

    let bot_cfg = std::fs::read_to_string(fx.bot.join(crate::config_md::VC_CONFIG_MD))
        .expect("read the agent config");
    assert!(bot_cfg.contains("work = \"..\""), "bot work = \"..\"");
    assert!(bot_cfg.contains("agent = \".\""), "bot agent = \".\"");

    let work_gi =
        std::fs::read_to_string(fx.work.join(".gitignore")).expect("read work .gitignore");
    assert!(
        work_gi.contains("/.agent-session"),
        "work .gitignore excludes /.agent-session"
    );
}

/// Dual fixture has `main` bookmarks tracking `origin/main` on
/// both sides: pins per-side step 10 (re-init + track) ran
/// for each `push_repo` call.
#[test]
fn dual_fixture_both_sides_track_origin() {
    let fx = crate::test_helpers::Fixture::new("dual-tracking");

    crate::common::verify_tracking(&fx.work, "main")
        .expect("work-side main should track origin/main after push_repo");
    crate::common::verify_tracking(&fx.bot, "main")
        .expect("bot-side main should track origin/main after push_repo");
}

/// The work side's publishing must preserve the nested agent repo's
/// `.jj/` and `.git/` state, which sits inside the work tree.
#[test]
fn dual_fixture_preserves_bot_across_work_clean() {
    let fx = crate::test_helpers::Fixture::new("dual-clean-exclude");

    assert!(
        fx.bot.join(".jj").exists(),
        "bot .jj must survive work-side clean"
    );
    assert!(
        fx.bot.join(".git").exists(),
        "bot .git must survive work-side clean"
    );
    assert!(
        fx.bot.join(crate::config_md::VC_CONFIG_MD).exists(),
        "the agent config must survive work-side clean"
    );
}

/// The recorded `[remote] agent-repo` is the agent-repo's *remote* name,
/// not the local project's.
///
/// The fixture's project is called `work` while its agent bare is
/// `remote-work.agent-session.git`, so a key written from the project name
/// would send clone at a repo that does not exist. Writing the
/// project name is the bug this test was added for.
#[test]
fn init_records_the_agent_repos_remote_name() {
    let fx = crate::test_helpers::Fixture::new("init-agent-repo-key");
    let recorded = crate::common::configured_agent_repo(&fx.work)
        .expect("read config")
        .expect("the key is written");
    assert_eq!(recorded, "remote-work.agent-session");

    // And it is what the derivation turns into the agent-repo's URL,
    // beside the work bare it sits next to.
    let work_url = format!("{}/remote-work.git", fx.base.display());
    assert_eq!(
        crate::url::agent_url(&work_url, Some(&recorded)),
        format!("{}/remote-work.agent-session.git", fx.base.display())
    );
}

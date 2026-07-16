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
    let args = parse(&["vc-x1", "init", "owner/repo"]);
    assert_eq!(args.target, "owner/repo");
    assert!(args.name.is_none());
    assert!(args.account.account.is_none());
    assert!(args.repo.repo.is_none());
    assert!(!args.por.value);
    assert!(!args.provision.private.private);
    assert!(!args.provision.dry_run.dry_run);
    assert_eq!(args.provision.push_retry.push_retries, 5);
    assert_eq!(args.provision.push_retry.push_retry_delay, 3);
    assert!(args.use_template.use_template.is_none());
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
    assert_eq!(args.target, "owner/repo");
    assert_eq!(args.name.as_deref(), Some("my-dir"));
    assert_eq!(args.account.account.as_deref(), Some("work"));
    let sel = args.repo.repo.as_ref().expect("--repo set");
    assert_eq!(sel.category, "local");
    assert_eq!(sel.value.as_deref(), Some("/tmp/xyz"));
    assert!(args.por.value);
    assert!(args.provision.private.private);
    assert!(args.provision.dry_run.dry_run);
    assert_eq!(args.provision.push_retry.push_retries, 10);
    assert_eq!(args.provision.push_retry.push_retry_delay, 5);
    assert_eq!(args.use_template.use_template.as_deref(), Some("/tmp/tmpl"));
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
    assert_eq!(args.account.account.as_deref(), Some("work"));
}

#[test]
fn repo_cat_only_parses() {
    let args = parse(&["vc-x1", "init", "tf1", "--repo", "remote"]);
    let sel = args.repo.repo.expect("--repo set");
    assert_eq!(sel.category, "remote");
    assert!(sel.value.is_none());
}

#[test]
fn repo_cat_value_parses() {
    let args = parse(&["vc-x1", "init", "tf1", "--repo", "remote=git@github.com:u"]);
    let sel = args.repo.repo.expect("--repo set");
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
        account: AccountOption::default(),
        repo: RepoOption::default(),
        por: PorFlag::default(),
        provision: ProvisionOptionFlagBundle {
            dry_run: DryRunFlag { dry_run: true },
            ..Default::default()
        },
        use_template: UseTemplateOption::default(),
        config: ConfigOption::default(),
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
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
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
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert_eq!(plan.provisioner, Provisioner::GhCreate);
    assert_eq!(plan.code_url, "https://github.com/owner/repo.git");
    assert_eq!(plan.gh_code_slug.as_deref(), Some("owner/repo"));
}

#[test]
fn plan_url_non_github_uses_external_provisioner() {
    let args = args_for("git@gitlab.com:winksaville/tf1.git");
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
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
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert_eq!(plan.name, "custom-dir");
    let cwd = std::env::current_dir().unwrap();
    assert_eq!(plan.project_dir, cwd.join("custom-dir"));
    assert_eq!(plan.code_url, "git@github.com:winksaville/tf1.git");
}

// ---------- owner/name shorthand TARGET ----------

#[test]
fn plan_owner_name_resolves_to_github_ssh() {
    let args = args_for("winksaville/tf1");
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert_eq!(plan.provisioner, Provisioner::GhCreate);
    assert_eq!(plan.code_url, "git@github.com:winksaville/tf1.git");
    assert_eq!(plan.gh_code_slug.as_deref(), Some("winksaville/tf1"));
}

#[test]
fn plan_owner_name_eq_ssh_url_form() {
    // owner/name shorthand must produce the same plan as the
    // explicit SSH URL it resolves to.
    let p1 = plan_init(
        &InitParams::from(&args_for("winksaville/tf1")),
        &cfg_empty(),
    )
    .unwrap();
    let p2 = plan_init(
        &InitParams::from(&args_for("git@github.com:winksaville/tf1")),
        &cfg_empty(),
    )
    .unwrap();
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
    args.repo.repo = Some(RepoSelector {
        category: "local".into(),
        value: Some("/tmp/xyz".into()),
    });
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
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
    args.repo.repo = Some(RepoSelector {
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
    assert_eq!(plan.code_url, "git@github.com:winksaville/tf1.git");
    assert_eq!(plan.provisioner, Provisioner::GhCreate);
    assert_eq!(plan.gh_code_slug.as_deref(), Some("winksaville/tf1"));
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
    args.account.account = Some("work".into());
    let plan = plan_init(&InitParams::from(&args), &cfg_two_accounts()).unwrap();
    assert_eq!(plan.code_url, "git@github.com:anthropic/tf1.git");
    assert_eq!(plan.gh_code_slug.as_deref(), Some("anthropic/tf1"));
}

#[test]
fn plan_bare_name_explicit_repo_value_skips_config() {
    // --repo cat=val short-circuits resolve_repo; works even
    // with an empty config.
    let mut args = args_for("tf1");
    args.repo.repo = Some(RepoSelector {
        category: "local".into(),
        value: Some("/tmp/explicit".into()),
    });
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
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
    args.por.value = true;
    args.repo.repo = Some(RepoSelector {
        category: "local".into(),
        value: Some("/tmp/xyz".into()),
    });
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
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
    args.por.value = true;
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert!(plan.scope.is_code_only());
    assert_eq!(plan.code_url, "git@github.com:winksaville/tf1.git");
    assert!(plan.session_url.is_none());
    assert!(plan.gh_session_slug.is_none());
}

#[test]
fn plan_default_scope_is_code_bot() {
    let args = args_for("git@github.com:winksaville/tf1");
    let plan = plan_init(&InitParams::from(&args), &cfg_empty()).unwrap();
    assert!(plan.scope.is_both());
    assert!(plan.session_url.is_some());
    assert!(plan.session_dir.is_some());
}

// ---------- Errors ----------

#[test]
fn error_url_target_with_account() {
    let mut args = args_for("git@github.com:u/p");
    args.account.account = Some("work".into());
    let err = plan_init(&InitParams::from(&args), &cfg_empty())
        .unwrap_err()
        .to_string();
    assert!(err.contains("--account is meaningless"), "got: {err}");
}

#[test]
fn error_url_target_with_repo() {
    let mut args = args_for("git@github.com:u/p");
    args.repo.repo = Some(RepoSelector {
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
    args.repo.repo = Some(RepoSelector {
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
    args.repo.repo = Some(RepoSelector {
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
    // Empty config + no --repo, no --account → step 1 of
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
    args.repo.repo = Some(RepoSelector {
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
    // --scope=por + --use-template foo,bar is ambiguous — bot
    // half has no home in a single-repo workspace.
    let mut args = args_for("git@github.com:u/p");
    args.por.value = true;
    args.use_template.use_template = Some("/tmp/code,/tmp/bot".into());
    let err = plan_init(&InitParams::from(&args), &cfg_empty())
        .unwrap_err()
        .to_string();
    assert!(err.contains("--por"), "got: {err}");
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

// ---------- POR end-to-end fixture (drives init with --scope=por) ----------

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

// ---------- --config flag (POR only) ----------

/// `--config none` skips writing `.vc-config.toml` while still
/// writing `.gitignore`. The repo gets created and pushed
/// successfully — config-less repos remain valid POR shape from
/// jj/git's perspective; downstream commands that need
/// `.vc-config.toml` will fail loudly when they try to read it.
#[test]
fn por_config_none_skips_vc_config_writes_gitignore() {
    let fx = crate::test_helpers::FixturePor::new_with_config(
        "por-config-none",
        Some("none".to_string()),
    );

    assert!(
        !fx.work.join(".vc-config.toml").exists(),
        "--config none must skip .vc-config.toml"
    );
    assert!(
        fx.work.join(".gitignore").exists(),
        "--config none must still write .gitignore"
    );
}

/// `--config <path>` copies the user-supplied file to
/// `.vc-config.toml` bytewise. `.gitignore` still written from
/// the canned source.
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
    args.config.raw = Some("none".to_string());
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
/// under `--por`; setting `args.por.value = true` keeps the
/// preflight on the path-validation branch.
#[test]
fn config_path_missing_rejected_at_preflight() {
    let mut args = args_for("./foo");
    args.por.value = true;
    args.config.raw = Some("/nonexistent/path/to/config.toml".to_string());
    let err = plan_init(&InitParams::from(&args), &cfg_empty())
        .unwrap_err()
        .to_string();
    assert!(err.contains("does not exist"), "unexpected error: {err}");
}

/// `--config none` with `--por` passes preflight (it's the
/// happy path — `none` is a literal keyword, not a path). URL
/// target sidesteps the account-config lookup that plan_init
/// would trigger for path-form targets in cfg_empty.
#[test]
fn config_none_passes_preflight() {
    let mut args = args_for("git@github.com:foo/bar.git");
    args.por.value = true;
    args.config.raw = Some("none".to_string());
    plan_init(&InitParams::from(&args), &cfg_empty())
        .expect("--config none with --scope=por should pass preflight");
}

// ---------- Dual end-to-end fixture (drives init with --scope=code,bot) ----------
//
// Counterparts to the POR fixture tests above; pin the dual-shape
// invariants `push_repo` must preserve (-6.3 extraction).

/// Dual fixture lays down both repos and both bare origins:
/// `<base>/work/`, `<base>/work/.claude/`, `<base>/remote-code.git`,
/// `<base>/remote-claude.git`. POR-shape `remote.git` is absent.
#[test]
fn dual_fixture_creates_dual_repo_layout() {
    let fx = crate::test_helpers::Fixture::new("dual-layout");

    assert!(fx.work.exists() && fx.work.is_dir(), "work dir present");
    assert!(
        fx.claude.exists() && fx.claude.is_dir(),
        "nested .claude dir present"
    );
    assert!(
        fx.base.join("remote-code.git").exists(),
        "code-side bare origin present"
    );
    assert!(
        fx.base.join("remote-claude.git").exists(),
        "session-side bare origin present"
    );
    assert!(
        !fx.base.join("remote.git").exists(),
        "POR-shape bare must not appear in dual layout"
    );
}

/// Dual fixture writes the CODE / SESSION config + .gitignore
/// variants — code side has `path = "/"` and
/// `other-repo = ".claude"`; session side has
/// `path = "/.claude"` and `other-repo = ".."`. Code-side
/// `.gitignore` excludes `/.claude` (session subdir is git-ignored
/// from the code-side view).
#[test]
fn dual_fixture_writes_code_and_session_config_files() {
    let fx = crate::test_helpers::Fixture::new("dual-config");

    let code_cfg = std::fs::read_to_string(fx.work.join(".vc-config.toml"))
        .expect("read code .vc-config.toml");
    assert!(code_cfg.contains("path = \"/\""), "code path = \"/\"");
    assert!(
        code_cfg.contains("other-repo = \".claude\""),
        "code other-repo = \".claude\""
    );

    let session_cfg = std::fs::read_to_string(fx.claude.join(".vc-config.toml"))
        .expect("read session .vc-config.toml");
    assert!(
        session_cfg.contains("path = \"/.claude\""),
        "session path = \"/.claude\""
    );
    assert!(
        session_cfg.contains("other-repo = \"..\""),
        "session other-repo = \"..\""
    );

    let code_gi =
        std::fs::read_to_string(fx.work.join(".gitignore")).expect("read code .gitignore");
    assert!(
        code_gi.contains("/.claude"),
        "code .gitignore excludes /.claude"
    );
}

/// Dual fixture has `main` bookmarks tracking `origin/main` on
/// both sides — pins per-side step 10 (re-init + track) ran
/// for each `push_repo` call.
#[test]
fn dual_fixture_both_sides_track_origin() {
    let fx = crate::test_helpers::Fixture::new("dual-tracking");

    crate::common::verify_tracking(&fx.work, "main")
        .expect("code-side main should track origin/main after push_repo");
    crate::common::verify_tracking(&fx.claude, "main")
        .expect("session-side main should track origin/main after push_repo");
}

/// Code-side `git clean -xdf --exclude .claude` must preserve
/// the nested bot repo's `.jj/` and `.git/` state. This
/// pins `push_repo`'s `clean_exclude = Some(".claude")` path
/// in the dual code-side call.
#[test]
fn dual_fixture_preserves_claude_across_code_clean() {
    let fx = crate::test_helpers::Fixture::new("dual-clean-exclude");

    assert!(
        fx.claude.join(".jj").exists(),
        "session .jj must survive code-side clean (clean_exclude=.claude)"
    );
    assert!(
        fx.claude.join(".git").exists(),
        "session .git must survive code-side clean"
    );
    assert!(
        fx.claude.join(".vc-config.toml").exists(),
        "session .vc-config.toml must survive code-side clean"
    );
}

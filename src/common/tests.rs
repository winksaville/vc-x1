//! Unit tests for the common module.

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
fn find_tracked_remote_synced() {
    let output = "\
main: zzwozmkn 40a8309a title here
  @git: zzwozmkn 40a8309a title here
  @origin: zzwozmkn 40a8309a title here";
    assert!(find_tracked_remote(output, "origin"));
}

#[test]
fn find_tracked_remote_divergent() {
    let output = "\
main: zzwozmkn 40a8309a title here
  @origin (ahead by 2 commits): zzwozmkn 40a8309a title";
    assert!(find_tracked_remote(output, "origin"));
}

#[test]
fn find_tracked_remote_non_tracking_is_not_tracked() {
    let output = "\
main: zzwozmkn 40a8309a title here
main@origin: zzwozmkn 40a8309a title here";
    assert!(!find_tracked_remote(output, "origin"));
}

#[test]
fn find_tracked_remote_absent() {
    assert!(!find_tracked_remote(
        "main: zzwozmkn 40a8309a title here",
        "origin"
    ));
    assert!(!find_tracked_remote("", "origin"));
}

#[test]
fn find_tracked_remote_other_remote_only() {
    let output = "\
main: zzwozmkn 40a8309a title here
  @git: zzwozmkn 40a8309a title here";
    assert!(!find_tracked_remote(output, "origin"));
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
/// has a `work` key, even when starting from a deep subdir.
#[test]
fn find_workspace_root_walks_up() {
    let base = ws_tempdir("walk-up");
    let root = base.join("ws");
    let nested = root.join("a").join("b").join("c");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        "[workspace]\nwork = \"/\"\nbot = \"/.claude\"\n",
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

/// Starting inside the bot repo — whose config is identical to the
/// root's — resolves to the *root*, not the bot dir itself.
#[test]
fn find_workspace_root_from_bot_dir() {
    let base = ws_tempdir("skip-non-root");
    let root = base.join("ws");
    let bot = root.join(".bot");
    std::fs::create_dir_all(&bot).unwrap();
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        "[workspace]\nwork = \"/\"\nbot = \"/.bot\"\n",
    )
    .unwrap();
    std::fs::write(
        bot.join(VC_CONFIG_FILE),
        "[workspace]\nwork = \"/\"\nbot = \"/.bot\"\n",
    )
    .unwrap();
    // From inside .bot, the root walker still resolves to the
    // work root, not to .bot itself (side detection by location).
    assert_eq!(find_workspace_root_from(&bot).as_deref(), Some(&*root));
    std::fs::remove_dir_all(&base).ok();
}

/// Default scope: workspace with non-empty `bot` → dual.
#[test]
fn default_scope_dual_workspace() {
    let base = ws_tempdir("default-dual");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        "[workspace]\nwork = \"/\"\nbot = \"/.claude\"\n",
    )
    .unwrap();
    assert_eq!(
        default_scope(Some(&root)),
        Scope(vec![Side::Work, Side::Bot])
    );
    std::fs::remove_dir_all(&base).ok();
}

/// Default scope: workspace with no `bot` → work-only.
#[test]
fn default_scope_single_repo_workspace() {
    let base = ws_tempdir("default-single");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), "[workspace]\nwork = \"/\"\n").unwrap();
    assert_eq!(default_scope(Some(&root)), Scope(vec![Side::Work]));
    std::fs::remove_dir_all(&base).ok();
}

/// Default scope: empty `bot` value treated like missing.
#[test]
fn default_scope_empty_other_repo() {
    let base = ws_tempdir("default-empty");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        "[workspace]\nwork = \"/\"\nbot = \"\"\n",
    )
    .unwrap();
    assert_eq!(default_scope(Some(&root)), Scope(vec![Side::Work]));
    std::fs::remove_dir_all(&base).ok();
}

/// Default scope: POR (no workspace_root) → `Scope([Work])`.
/// `scope_to_repos` then resolves `Side::Work` to cwd's `.`.
#[test]
fn default_scope_por_returns_work() {
    assert_eq!(default_scope(None), Scope(vec![Side::Work]));
}

/// `bot_repo_path`: dual workspace → `Some(root/<bot dir>)`.
#[test]
fn bot_repo_path_dual() {
    let base = ws_tempdir("botpath-dual");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        "[workspace]\nwork = \"/\"\nbot = \"/.claude\"\n",
    )
    .unwrap();
    assert_eq!(bot_repo_path(&root).unwrap(), Some(root.join(".claude")));
    std::fs::remove_dir_all(&base).ok();
}

/// `bot_repo_path`: single-repo workspace (no `bot`) →
/// `None` — the caller's no-op case, not an error.
#[test]
fn bot_repo_path_single_repo_workspace() {
    let base = ws_tempdir("botpath-single");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), "[workspace]\nwork = \"/\"\n").unwrap();
    assert_eq!(bot_repo_path(&root).unwrap(), None);
    std::fs::remove_dir_all(&base).ok();
}

/// A pre-0.75.0 legacy config (`path`/`other-repo`, no `work`) is
/// still *found* as a root, and the resolvers reject it with the
/// rewrite instead of silently degrading to POR.
#[test]
fn legacy_config_found_and_rejected() {
    let base = ws_tempdir("legacy-reject");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        "[workspace]\npath = \"/\"\nother-repo = \".claude\"\n",
    )
    .unwrap();
    // The walk still locates the legacy root...
    assert_eq!(find_workspace_root_from(&root).as_deref(), Some(&*root));
    // ...and every resolver errors with the fix-it message.
    let err = scope_to_repos(&Scope(vec![Side::Work]), Some(&root))
        .unwrap_err()
        .to_string();
    assert!(err.contains("legacy [workspace] schema"), "got: {err}");
    assert!(err.contains("work = \"/\""), "got: {err}");
    let err = bot_repo_path(&root).unwrap_err().to_string();
    assert!(err.contains("legacy [workspace] schema"), "got: {err}");
    std::fs::remove_dir_all(&base).ok();
}

/// A config carrying both old and new keys passes the legacy
/// guard — the new keys drive behavior; `config --validate`
/// flags the strays.
#[test]
fn legacy_guard_accepts_mixed_keys() {
    let base = ws_tempdir("legacy-mixed");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        "[workspace]\nwork = \"/\"\nbot = \"/.claude\"\npath = \"/\"\n",
    )
    .unwrap();
    assert!(reject_legacy_config(&root).is_ok());
    std::fs::remove_dir_all(&base).ok();
}

/// `bot_repo_path`: no `.vc-config.toml` at all (POR) → `None`.
#[test]
fn bot_repo_path_no_config() {
    let base = ws_tempdir("botpath-por");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    assert_eq!(bot_repo_path(&root).unwrap(), None);
    std::fs::remove_dir_all(&base).ok();
}

/// `scope_to_repos`: dual workspace resolves to root + root/<bot dir>.
#[test]
fn scope_to_repos_dual() {
    let base = ws_tempdir("repos-dual");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        "[workspace]\nwork = \"/\"\nbot = \"/.claude\"\n",
    )
    .unwrap();
    let repos = scope_to_repos(&Scope(vec![Side::Work, Side::Bot]), Some(&root)).unwrap();
    assert_eq!(repos, vec![root.clone(), root.join(".claude")]);
    std::fs::remove_dir_all(&base).ok();
}

/// `scope_to_repos`: work-only inside a workspace yields just root.
#[test]
fn scope_to_repos_work_only() {
    let base = ws_tempdir("repos-work");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        "[workspace]\nwork = \"/\"\nbot = \"/.claude\"\n",
    )
    .unwrap();
    let repos = scope_to_repos(&Scope(vec![Side::Work]), Some(&root)).unwrap();
    assert_eq!(repos, vec![root.clone()]);
    std::fs::remove_dir_all(&base).ok();
}

/// `scope_to_repos`: work-only with POR → cwd `.`.
#[test]
fn scope_to_repos_work_por() {
    let repos = scope_to_repos(&Scope(vec![Side::Work]), None).unwrap();
    assert_eq!(repos, vec![PathBuf::from(".")]);
}

/// `scope_to_repos`: bot in POR errors with the documented message.
#[test]
fn scope_to_repos_bot_por_errors() {
    let err = scope_to_repos(&Scope(vec![Side::Bot]), None)
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
    std::fs::write(root.join(VC_CONFIG_FILE), "[workspace]\nwork = \"/\"\n").unwrap();
    let err = scope_to_repos(&Scope(vec![Side::Bot]), Some(&root))
        .unwrap_err()
        .to_string();
    assert!(err.contains("no bot repo configured"), "got: {err}");
    std::fs::remove_dir_all(&base).ok();
}

/// `resolve_repos`: no flags → today's `["."]` default.
#[test]
fn resolve_repos_no_flags_defaults_to_dot() {
    let repos = resolve_repos(None, None).unwrap();
    assert_eq!(repos, vec![PathBuf::from(".")]);
}

/// `resolve_repos`: `-R <path>` alone → `[path]`, workspace context not consulted.
#[test]
fn resolve_repos_repo_only_returns_path() {
    let p = PathBuf::from("/some/repo");
    let repos = resolve_repos(Some(&p), None).unwrap();
    assert_eq!(repos, vec![p]);
}

/// `resolve_repos`: `-R <ws> -s work,bot` composes — the path is the
/// workspace root, the roles are resolved within it.
#[test]
fn resolve_repos_repo_plus_scope_uses_path_as_workspace_root() {
    let base = ws_tempdir("resolve-compose");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        "[workspace]\nwork = \"/\"\nbot = \"/.claude\"\n",
    )
    .unwrap();
    let scope = Scope(vec![Side::Work, Side::Bot]);
    let repos = resolve_repos(Some(&root), Some(&scope)).unwrap();
    assert_eq!(repos, vec![root.clone(), root.join(".claude")]);
    std::fs::remove_dir_all(&base).ok();
}

/// `resolve_repos`: `-R <ws> -s bot` composes to just the bot side.
#[test]
fn resolve_repos_repo_plus_scope_bot_only() {
    let base = ws_tempdir("resolve-compose-bot");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        "[workspace]\nwork = \"/\"\nbot = \"/.claude\"\n",
    )
    .unwrap();
    let scope = Scope(vec![Side::Bot]);
    let repos = resolve_repos(Some(&root), Some(&scope)).unwrap();
    assert_eq!(repos, vec![root.join(".claude")]);
    std::fs::remove_dir_all(&base).ok();
}

//! Unit tests for the common module.

use super::*;

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

/// Work-side dual `[repos]` registry with a `.claude` bot dir.
const WORK_DUAL: &str = "[repos]\nwork = \".\"\nagent = \".claude\"\n";
/// Bot-side dual `[repos]` registry (nested directly under root).
const BOT_DUAL: &str = "[repos]\nwork = \"..\"\nagent = \".\"\n";
/// Single-repo (POR-workspace) `[repos]` registry.
const WORK_ONLY: &str = "[repos]\nwork = \".\"\n";

/// Workspace root walk finds the dir whose `.vc-config.toml`
/// has a `repos.work` key, even when starting from a deep subdir.
#[test]
fn find_workspace_root_walks_up() {
    let base = ws_tempdir("walk-up");
    let root = base.join("ws");
    let nested = root.join("a").join("b").join("c");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_DUAL).unwrap();
    assert_eq!(
        find_workspace_root_from(&nested),
        Some(root.canonicalize().unwrap())
    );
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

/// Starting inside the bot repo resolves to the *root*: its own
/// config's `repos.work` points there (self-resolution needs no
/// nesting assumption).
#[test]
fn find_workspace_root_from_bot_dir() {
    let base = ws_tempdir("skip-non-root");
    let root = base.join("ws");
    let bot = root.join(".bot");
    std::fs::create_dir_all(&bot).unwrap();
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        "[repos]\nwork = \".\"\nagent = \".bot\"\n",
    )
    .unwrap();
    std::fs::write(bot.join(VC_CONFIG_FILE), BOT_DUAL).unwrap();
    // From inside .bot, the walk finds .bot's config first, and its
    // work = ".." resolves to the work root.
    assert_eq!(
        find_workspace_root_from(&bot),
        Some(root.canonicalize().unwrap())
    );
    std::fs::remove_dir_all(&base).ok();
}

/// Default scope: workspace with non-empty `agent` -> dual.
#[test]
fn default_scope_dual_workspace() {
    let base = ws_tempdir("default-dual");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_DUAL).unwrap();
    assert_eq!(
        default_scope(Some(&root)),
        Scope(vec![Side::Work, Side::Bot])
    );
    std::fs::remove_dir_all(&base).ok();
}

/// Default scope: workspace with no `agent` -> work-only.
#[test]
fn default_scope_single_repo_workspace() {
    let base = ws_tempdir("default-single");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_ONLY).unwrap();
    assert_eq!(default_scope(Some(&root)), Scope(vec![Side::Work]));
    std::fs::remove_dir_all(&base).ok();
}

/// Default scope: empty `agent` value treated like missing.
#[test]
fn default_scope_empty_other_repo() {
    let base = ws_tempdir("default-empty");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        "[repos]\nwork = \".\"\nagent = \"\"\n",
    )
    .unwrap();
    assert_eq!(default_scope(Some(&root)), Scope(vec![Side::Work]));
    std::fs::remove_dir_all(&base).ok();
}

/// Default scope: POR (no workspace_root) -> `Scope([Work])`.
/// `scope_to_repos` then resolves `Side::Work` to cwd's `.`.
#[test]
fn default_scope_por_returns_work() {
    assert_eq!(default_scope(None), Scope(vec![Side::Work]));
}

/// `bot_repo_path`: coherent dual workspace -> `Some(root/<bot dir>)`.
#[test]
fn bot_repo_path_dual() {
    let base = ws_tempdir("botpath-dual");
    let root = base.join("ws");
    let bot = root.join(".claude");
    std::fs::create_dir_all(&bot).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_DUAL).unwrap();
    std::fs::write(bot.join(VC_CONFIG_FILE), BOT_DUAL).unwrap();
    assert_eq!(bot_repo_path(&root).unwrap(), Some(root.join(".claude")));
    std::fs::remove_dir_all(&base).ok();
}

/// Dual-mode entry preflight: a declared-but-missing bot dir errors
/// loudly, changing nothing.
#[test]
fn bot_repo_path_missing_dir_errors() {
    let base = ws_tempdir("botpath-nodir");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_DUAL).unwrap();
    let err = bot_repo_path(&root).unwrap_err().to_string();
    assert!(err.contains("workspace incoherent"), "got: {err}");
    assert!(err.contains("does not exist"), "got: {err}");
    std::fs::remove_dir_all(&base).ok();
}

/// Dual-mode entry preflight: `[repos]` registries resolving to
/// different directories error with both sides printed.
#[test]
fn bot_repo_path_mismatched_blocks_error() {
    let base = ws_tempdir("botpath-mismatch");
    let root = base.join("ws");
    let bot = root.join(".claude");
    let other = root.join("other");
    std::fs::create_dir_all(&bot).unwrap();
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_DUAL).unwrap();
    // The bot side claims a different bot dir than the root side.
    std::fs::write(
        bot.join(VC_CONFIG_FILE),
        "[repos]\nwork = \"..\"\nagent = \"../other\"\n",
    )
    .unwrap();
    let err = bot_repo_path(&root).unwrap_err().to_string();
    assert!(err.contains("workspace incoherent"), "got: {err}");
    assert!(err.contains("different directories"), "got: {err}");
    std::fs::remove_dir_all(&base).ok();
}

/// `configured_bot_dir` is the pure config read: no existence
/// check (clone resolves the destination before it exists).
#[test]
fn configured_bot_dir_no_existence_check() {
    let base = ws_tempdir("botpath-configured");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_DUAL).unwrap();
    assert_eq!(
        configured_bot_dir(&root).unwrap(),
        Some(root.join(".claude"))
    );
    std::fs::remove_dir_all(&base).ok();
}

/// `bot_repo_path`: single-repo workspace (no `agent`) ->
/// `None`: the caller's no-op case, not an error.
#[test]
fn bot_repo_path_single_repo_workspace() {
    let base = ws_tempdir("botpath-single");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_ONLY).unwrap();
    assert_eq!(bot_repo_path(&root).unwrap(), None);
    std::fs::remove_dir_all(&base).ok();
}

/// `other_repo_path` from the work side: the far side is the bot
/// repo.
#[test]
fn other_repo_path_from_work_side() {
    let base = ws_tempdir("otherpath-work");
    let root = base.join("ws");
    let bot = root.join(".claude");
    std::fs::create_dir_all(&bot).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_DUAL).unwrap();
    std::fs::write(bot.join(VC_CONFIG_FILE), BOT_DUAL).unwrap();
    let canon_root = root.canonicalize().unwrap();
    assert_eq!(
        other_repo_path(&root).unwrap(),
        Some(canon_root.join(".claude"))
    );
    std::fs::remove_dir_all(&base).ok();
}

/// `other_repo_path` from the bot side: the far side is the work
/// repo (the 2026-08-14 validate-desc regression: `bot_repo_path`
/// treated the bot dir as the workspace root and the coherence
/// preflight rejected it).
#[test]
fn other_repo_path_from_bot_side() {
    let base = ws_tempdir("otherpath-bot");
    let root = base.join("ws");
    let bot = root.join(".claude");
    std::fs::create_dir_all(&bot).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_DUAL).unwrap();
    std::fs::write(bot.join(VC_CONFIG_FILE), BOT_DUAL).unwrap();
    let canon_root = root.canonicalize().unwrap();
    assert_eq!(other_repo_path(&bot).unwrap(), Some(canon_root));
    std::fs::remove_dir_all(&base).ok();
}

/// `other_repo_path`: no workspace config anywhere up the walk
/// (POR) -> `None`, the caller's no-op case.
#[test]
fn other_repo_path_por() {
    let base = ws_tempdir("otherpath-por");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    assert_eq!(other_repo_path(&root).unwrap(), None);
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
    assert!(err.contains("work = \".\""), "got: {err}");
    let err = bot_repo_path(&root).unwrap_err().to_string();
    assert!(err.contains("legacy [workspace] schema"), "got: {err}");
    std::fs::remove_dir_all(&base).ok();
}

/// The 0.75.x root-anchored `[workspace] work`/`agent` schema is the
/// second rejected legacy generation: still *found* as a root
/// (via the legacy location rule), rejected with the rewrite.
#[test]
fn legacy_workspace_work_bot_rejected() {
    let base = ws_tempdir("legacy-075x");
    let root = base.join("ws");
    let bot = root.join(".claude");
    std::fs::create_dir_all(&bot).unwrap();
    let block = "[workspace]\nwork = \"/\"\nbot = \"/.claude\"\n";
    std::fs::write(root.join(VC_CONFIG_FILE), block).unwrap();
    std::fs::write(bot.join(VC_CONFIG_FILE), block).unwrap();
    // The walk still locates the legacy root, even from the bot
    // side (legacy location rule)...
    assert_eq!(find_workspace_root_from(&root).as_deref(), Some(&*root));
    assert_eq!(find_workspace_root_from(&bot).as_deref(), Some(&*root));
    // ...and every resolver errors with the fix-it message.
    let err = scope_to_repos(&Scope(vec![Side::Work]), Some(&root))
        .unwrap_err()
        .to_string();
    assert!(err.contains("legacy [workspace] schema"), "got: {err}");
    assert!(err.contains("[repos]"), "got: {err}");
    std::fs::remove_dir_all(&base).ok();
}

/// An empty `repos.work` errors: every reader would silently
/// misresolve it.
#[test]
fn empty_repos_work_rejected() {
    let base = ws_tempdir("empty-work");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), "[repos]\nwork = \"\"\n").unwrap();
    let err = scope_to_repos(&Scope(vec![Side::Work]), Some(&root))
        .unwrap_err()
        .to_string();
    assert!(err.contains("repos.work is empty"), "got: {err}");
    std::fs::remove_dir_all(&base).ok();
}

/// A config carrying both a `[repos]` registry and stray legacy
/// keys passes the legacy guard: the registry drives behavior,
/// and `config --validate` flags the strays.
#[test]
fn legacy_guard_accepts_mixed_keys() {
    let base = ws_tempdir("legacy-mixed");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        "[repos]\nwork = \".\"\nagent = \".claude\"\n\n[workspace]\npath = \"/\"\n",
    )
    .unwrap();
    assert!(reject_legacy_config(&root).is_ok());
    std::fs::remove_dir_all(&base).ok();
}

/// The pre-0.80.0 `bot` spellings on a `[repos]`-schema config are
/// rejected with a fix-it naming every old key and its replacement,
/// rather than aliased.
#[test]
fn old_agent_spellings_rejected_with_fixit() {
    let base = ws_tempdir("old-agent-keys");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        "[repos]\nwork = \".\"\nbot = \".claude\"\n\n[bot-session]\ncol-width = 40\n",
    )
    .unwrap();
    let msg = reject_legacy_config(&root).unwrap_err().to_string();
    assert!(msg.contains("pre-0.80.0"), "got: {msg}");
    assert!(msg.contains("repos.bot -> repos.agent"), "got: {msg}");
    assert!(
        msg.contains("bot-session.col-width -> agent-session.col-width"),
        "got: {msg}"
    );
    assert!(msg.contains("--scope=agent"), "got: {msg}");
    std::fs::remove_dir_all(&base).ok();
}

/// Coherence: a bot-side registry missing `repos.agent` errors with
/// the missing-key detail, not a bare mismatch.
#[test]
fn coherence_missing_bot_key_errors() {
    let base = ws_tempdir("coh-missing-key");
    let root = base.join("ws");
    let bot = root.join(".claude");
    std::fs::create_dir_all(&bot).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_DUAL).unwrap();
    std::fs::write(bot.join(VC_CONFIG_FILE), "[repos]\nwork = \"..\"\n").unwrap();
    let err = bot_repo_path(&root).unwrap_err().to_string();
    assert!(err.contains("no `repos.agent`"), "got: {err}");
    std::fs::remove_dir_all(&base).ok();
}

/// Coherence: a declared dir that doesn't exist errors with the
/// unresolvable-value detail (which key, which value).
#[test]
fn coherence_unresolvable_dir_errors() {
    let base = ws_tempdir("coh-unresolvable");
    let root = base.join("ws");
    let bot = root.join(".claude");
    std::fs::create_dir_all(&bot).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_DUAL).unwrap();
    std::fs::write(
        bot.join(VC_CONFIG_FILE),
        "[repos]\nwork = \"../missing\"\nagent = \".\"\n",
    )
    .unwrap();
    let err = bot_repo_path(&root).unwrap_err().to_string();
    assert!(err.contains("repos.work"), "got: {err}");
    assert!(err.contains("../missing"), "got: {err}");
    assert!(err.contains("does not resolve"), "got: {err}");
    std::fs::remove_dir_all(&base).ok();
}

/// Coherence self-identification: both sides agreeing on the same
/// *wrong* pair (root's `work` naming a third dir) is caught.
#[test]
fn coherence_self_identification_errors() {
    let base = ws_tempdir("coh-selfid");
    let root = base.join("ws");
    let bot = root.join(".claude");
    let other = root.join("other");
    std::fs::create_dir_all(&bot).unwrap();
    std::fs::create_dir_all(&other).unwrap();
    // Both sides name (other, .claude), perfectly agreeing, but
    // the root's own dir is not at `work`.
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        "[repos]\nwork = \"other\"\nagent = \".claude\"\n",
    )
    .unwrap();
    std::fs::write(
        bot.join(VC_CONFIG_FILE),
        "[repos]\nwork = \"../other\"\nagent = \".\"\n",
    )
    .unwrap();
    let err = bot_repo_path(&root).unwrap_err().to_string();
    assert!(
        err.contains("not to the workspace root itself"),
        "got: {err}"
    );
    std::fs::remove_dir_all(&base).ok();
}

/// Coherence: absolute values are allowed (discouraged), a dual
/// workspace mixing absolute and relative spellings still agrees
/// on resolved reality.
#[test]
fn coherence_absolute_values_agree() {
    let base = ws_tempdir("coh-absolute");
    let root = base.join("ws");
    let bot = root.join(".claude");
    std::fs::create_dir_all(&bot).unwrap();
    let canon_bot = bot.canonicalize().unwrap();
    std::fs::write(
        root.join(VC_CONFIG_FILE),
        format!(
            "[repos]\nwork = \".\"\nagent = \"{}\"\n",
            canon_bot.display()
        ),
    )
    .unwrap();
    std::fs::write(bot.join(VC_CONFIG_FILE), BOT_DUAL).unwrap();
    assert_eq!(bot_repo_path(&root).unwrap(), Some(canon_bot));
    std::fs::remove_dir_all(&base).ok();
}

/// Side detection by self-resolution: the bot side's own config
/// (`bot = "."`) names it, and the work side is not the bot side.
#[test]
fn is_bot_dir_by_self_resolution() {
    let base = ws_tempdir("selfres");
    let root = base.join("ws");
    let bot = root.join(".claude");
    std::fs::create_dir_all(&bot).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_DUAL).unwrap();
    std::fs::write(bot.join(VC_CONFIG_FILE), BOT_DUAL).unwrap();
    assert!(is_bot_dir(&bot));
    assert!(!is_bot_dir(&root));
    std::fs::remove_dir_all(&base).ok();
}

/// `is_bot_dir` legacy fallback: both rejected generations'
/// parent configs still name the bot side (0.75.x
/// `workspace.bot` and pre-0.75.0 `workspace.other-repo`) so
/// read-only surfaces bypassing the resolvers stay correct.
#[test]
fn is_bot_dir_legacy_fallback() {
    for (tag, block) in [
        (
            "legacy-075x",
            "[workspace]\nwork = \"/\"\nbot = \"/.claude\"\n",
        ),
        (
            "legacy-pre075",
            "[workspace]\npath = \"/\"\nother-repo = \".claude\"\n",
        ),
    ] {
        let base = ws_tempdir(tag);
        let root = base.join("ws");
        let bot = root.join(".claude");
        std::fs::create_dir_all(&bot).unwrap();
        std::fs::write(root.join(VC_CONFIG_FILE), block).unwrap();
        assert!(is_bot_dir(&bot), "generation: {tag}");
        assert!(!is_bot_dir(&root), "generation: {tag}");
        std::fs::remove_dir_all(&base).ok();
    }
}

/// `bot_repo_path`: no `.vc-config.toml` at all (POR) -> `None`.
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
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_DUAL).unwrap();
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
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_DUAL).unwrap();
    let repos = scope_to_repos(&Scope(vec![Side::Work]), Some(&root)).unwrap();
    assert_eq!(repos, vec![root.clone()]);
    std::fs::remove_dir_all(&base).ok();
}

/// `scope_to_repos`: work-only with POR -> cwd `.`.
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
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_ONLY).unwrap();
    let err = scope_to_repos(&Scope(vec![Side::Bot]), Some(&root))
        .unwrap_err()
        .to_string();
    assert!(err.contains("no bot repo configured"), "got: {err}");
    std::fs::remove_dir_all(&base).ok();
}

/// `resolve_repos`: no flags -> today's `["."]` default.
#[test]
fn resolve_repos_no_flags_defaults_to_dot() {
    let repos = resolve_repos(None, None).unwrap();
    assert_eq!(repos, vec![PathBuf::from(".")]);
}

/// `resolve_repos`: `-R <path>` alone -> `[path]`, workspace context not consulted.
#[test]
fn resolve_repos_repo_only_returns_path() {
    let p = PathBuf::from("/some/repo");
    let repos = resolve_repos(Some(&p), None).unwrap();
    assert_eq!(repos, vec![p]);
}

/// `resolve_repos`: `-R <ws> -s work,bot` composes, the path is the
/// workspace root, the roles are resolved within it.
#[test]
fn resolve_repos_repo_plus_scope_uses_path_as_workspace_root() {
    let base = ws_tempdir("resolve-compose");
    let root = base.join("ws");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_DUAL).unwrap();
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
    std::fs::write(root.join(VC_CONFIG_FILE), WORK_DUAL).unwrap();
    let scope = Scope(vec![Side::Bot]);
    let repos = resolve_repos(Some(&root), Some(&scope)).unwrap();
    assert_eq!(repos, vec![root.join(".claude")]);
    std::fs::remove_dir_all(&base).ok();
}

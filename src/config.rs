//! User-level config loaded from `~/.config/vc-x1/config.toml`.
//!
//! Backs init's account/repo-target resolution. No magic
//! fallbacks — a missing file or missing key produces a
//! predictable error pointing at the exact key to set.
//!
//! Schema (multi-account, dotted keys for compactness):
//!
//! ```toml
//! [default]
//! account = "home"      # default --account when absent
//! debug   = "trace"     # default --debug value when used without arg
//!
//! [account.home]
//! repo.default          = "remote"                       # default --repo cat when absent
//! repo.category.remote  = "git@github.com:winksaville"   # value for --repo remote (no =val)
//! repo.category.local   = "~/test-fixtures"              # value for --repo local (no =val)
//!
//! [account.work]
//! repo.default          = "remote"
//! repo.category.remote  = "git@github.com:anthropic"
//! repo.category.local   = "/work/fixtures"
//! ```
//!
//! Three-layer resolution — see
//! `notes/chores-08.md > User config (0.41.1-3, redesigned in 0.41.1-4)`:
//!
//! 1. account: CLI `--account` → `[default].account` → error.
//! 2. category: CLI `--repo <cat>` → `[account.<a>.repo].default`
//!    → error.
//! 3. value: CLI `--repo <cat>=<val>` →
//!    `[account.<a>.repo.category].<cat>` → error.
//!
//! Values are **literal targets**, not section-name pointers.
//! For built-in `category = "remote"` the value is a URL prefix
//! (init appends `/<NAME>.git`); for `category = "local"` it's
//! the parent dir for fixture bare repos.
//!
//! `-3` shipped a flat first-cut (`[default] repo-remote-provider`,
//! `[github] owner`); `-4` is the redesign before any consumer
//! is wired in. `-5` (init reshape) is the first consumer.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::toml_simple;

/// Per-account configuration.
///
/// - `repo_default` — `[account.<name>.repo].default` — the
///   category to use when `--repo` is absent.
/// - `repo_category` — `[account.<name>.repo.category]` — map
///   from category name (e.g. `"remote"`, `"local"`) to its
///   literal value (URL prefix, fixture parent dir, etc.).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AccountConfig {
    pub repo_default: Option<String>,
    pub repo_category: HashMap<String, String>,
}

/// Resolved user config.
///
/// All fields are optional or default-empty — absent file or
/// absent keys yield empty values. Consumers apply their own
/// errors at use time.
///
/// **Two write modes** for repo defaults:
/// - **Top-level `[repo]`** — single-account shorthand. Sits in
///   `top_level_repo`. Used when neither `--account` nor
///   `[default].account` resolves to a known account.
/// - **`[account.<name>]`** — multi-account. Sits in `accounts`.
///   Selected by `--account` CLI or `[default].account`.
///
/// Mixing both is rejected at load time (ambiguous which one
/// init should consult).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct UserConfig {
    /// `[default].account` — default account when `--account`
    /// is absent.
    pub default_account: Option<String>,

    /// `[default].debug` — default value when `--debug` is used
    /// without an argument. (Reserved; not currently consumed.)
    pub default_debug: Option<String>,

    /// Top-level `[repo]` — single-account shorthand. `Some`
    /// only when the file has top-level `repo.*` keys and no
    /// `[account.*]` sections.
    pub top_level_repo: Option<AccountConfig>,

    /// Per-account configuration, keyed by account name.
    pub accounts: HashMap<String, AccountConfig>,
}

/// CLI selector for `--repo <cat>[=<val>]`.
///
/// - `--repo` absent → `None`.
/// - `--repo <cat>` → `Some({ category: cat, value: None })`.
/// - `--repo <cat>=<val>` → `Some({ category: cat, value: Some(val) })`.
#[allow(dead_code)] // OK: 0.41.1-5 is the first consumer; staged ahead in -4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoSelector {
    pub category: String,
    pub value: Option<String>,
}

/// Path to the user config file.
///
/// - Honors `$XDG_CONFIG_HOME` if set (XDG Base Directory spec).
/// - Falls back to `$HOME/.config`.
/// - Errors if neither variable is set.
pub fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let base = if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(xdg)
    } else if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".config")
    } else {
        return Err("neither $XDG_CONFIG_HOME nor $HOME is set".into());
    };
    Ok(base.join("vc-x1").join("config.toml"))
}

/// Load the user config from the default location.
///
/// - Missing file → empty `UserConfig`.
/// - Malformed file → fatal error.
#[allow(dead_code)] // OK: 0.41.1-5 is the first consumer; staged ahead in -4.
pub fn load() -> Result<UserConfig, Box<dyn std::error::Error>> {
    load_from(&config_path()?)
}

/// Load the user config from an explicit path.
///
/// - Missing file → empty `UserConfig`.
/// - Malformed file → fatal error (propagated from `toml_simple`).
/// - Unknown sections / keys are silently ignored
///   (forward-compatible with future schema additions).
pub fn load_from(path: &Path) -> Result<UserConfig, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(UserConfig::default());
    }
    let map = toml_simple::toml_load(path)?;

    let mut cfg = UserConfig {
        default_account: map.get("default.account").cloned(),
        default_debug: map.get("default.debug").cloned(),
        top_level_repo: None,
        accounts: HashMap::new(),
    };

    let mut top_level = AccountConfig::default();
    let mut top_level_seen = false;

    // Walk all keys; route `repo.*` to top-level, `account.<n>.…`
    // to per-account.
    for (full_key, value) in &map {
        if let Some(suffix) = full_key.strip_prefix("repo.") {
            top_level_seen = true;
            apply_repo_subkey(&mut top_level, suffix, value);
            continue;
        }
        let Some(rest) = full_key.strip_prefix("account.") else {
            continue;
        };
        let Some((name, suffix)) = rest.split_once('.') else {
            continue;
        };
        let Some(repo_suffix) = suffix.strip_prefix("repo.") else {
            continue; // unknown sub-section under [account.<n>]
        };
        let account = cfg.accounts.entry(name.to_string()).or_default();
        apply_repo_subkey(account, repo_suffix, value);
    }

    if top_level_seen && !cfg.accounts.is_empty() {
        return Err(format!(
            "{}: mixing top-level [repo] with [account.*] is ambiguous — \
             remove top-level [repo] or move it under [account.<name>]",
            path.display()
        )
        .into());
    }
    if top_level_seen {
        cfg.top_level_repo = Some(top_level);
    }

    Ok(cfg)
}

/// Apply one `repo.*` sub-key (e.g. `default`, `category.remote`)
/// onto an `AccountConfig`. Used for both the top-level `[repo]`
/// path and the `[account.<n>.repo]` path.
fn apply_repo_subkey(account: &mut AccountConfig, suffix: &str, value: &str) {
    match suffix {
        "default" => {
            account.repo_default = Some(value.to_string());
        }
        s if let Some(cat) = s.strip_prefix("category.") => {
            account
                .repo_category
                .insert(cat.to_string(), value.to_string());
        }
        _ => {} // unknown sub-key; ignore (forward-compat)
    }
}

/// Resolve `(category, value)` for an init invocation.
///
/// Three-step chain — each step has its own error message
/// pointing at the exact config key to set or CLI arg to pass.
///
/// - `account_override` — `Some(a)` from `--account a` CLI;
///   `None` falls back to `[default].account`.
/// - `repo_cli` — the parsed `--repo` selector (or `None` if
///   the flag was absent).
#[allow(dead_code)] // OK: 0.41.1-5 is the first consumer; staged ahead in -4.
pub fn resolve_repo(
    cfg: &UserConfig,
    account_override: Option<&str>,
    repo_cli: Option<&RepoSelector>,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Step 1: pick the AccountConfig to use.
    //
    // - `--account` CLI → must hit `cfg.accounts[name]`.
    // - `[default].account` → must hit `cfg.accounts[that]`.
    // - Otherwise fall back to top-level `[repo]` (no account).
    // - Otherwise error.
    //
    // `account_label` is for error messages — section-name "<name>"
    // when an account was selected, "<top-level>" for the no-account
    // shorthand path.
    let (account, account_label) = match (account_override, &cfg.default_account) {
        (Some(name), _) => {
            let acct = cfg.accounts.get(name).ok_or_else(|| {
                format!("account '{name}' not found in config (no [account.{name}] section)")
            })?;
            (acct, name.to_string())
        }
        (None, Some(name)) => {
            let acct = cfg.accounts.get(name).ok_or_else(|| {
                format!(
                    "account '{name}' (from [default].account) not found in config \
                     (no [account.{name}] section)"
                )
            })?;
            (acct, name.clone())
        }
        (None, None) => match cfg.top_level_repo.as_ref() {
            Some(r) => (r, String::from("<top-level>")),
            None => {
                return Err("no account specified; set [default].account, use \
                     --account <name>, or write a top-level [repo] section"
                    .into());
            }
        },
    };

    // Step 2: category.
    let category = match repo_cli {
        Some(sel) => sel.category.clone(),
        None => account.repo_default.clone().ok_or_else(|| {
            section_error(
                &account_label,
                "no default category",
                "repo",
                "default",
                "use --repo <cat>",
            )
        })?,
    };

    // Step 3: value.
    let value = match repo_cli.and_then(|s| s.value.clone()) {
        Some(v) => v,
        None => account
            .repo_category
            .get(&category)
            .cloned()
            .ok_or_else(|| {
                section_error(
                    &account_label,
                    &format!("no value for --repo {category}"),
                    "repo.category",
                    &category,
                    &format!("--repo {category}=<val>"),
                )
            })?,
    };

    Ok((category, value))
}

/// Compose an error message for a missing-key case in `resolve_repo`.
///
/// Renders as `[account.<label>.<section_subpath>].<leaf>` for
/// named accounts and `[<section_subpath>].<leaf>` for the
/// top-level `[repo]` shorthand.
fn section_error(
    account_label: &str,
    msg: &str,
    section_subpath: &str,
    leaf: &str,
    cli_alt: &str,
) -> String {
    let section = if account_label == "<top-level>" {
        format!("[{section_subpath}]")
    } else {
        format!("[account.{account_label}.{section_subpath}]")
    };
    format!("{msg}; set {section}.{leaf} or use {cli_alt}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn cfg_tempdir(tag: &str) -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("vc-x1-cfg-{tag}-{ts}"));
        fs::create_dir_all(&dir).expect("mkdir tempdir");
        dir
    }

    fn write_cfg(tag: &str, contents: &str) -> PathBuf {
        let dir = cfg_tempdir(tag);
        let path = dir.join("config.toml");
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn missing_file_returns_default() {
        let dir = cfg_tempdir("missing");
        let path = dir.join("nonexistent.toml");
        let cfg = load_from(&path).unwrap();
        assert_eq!(cfg, UserConfig::default());
    }

    #[test]
    fn empty_file_returns_default() {
        let path = write_cfg("empty", "");
        let cfg = load_from(&path).unwrap();
        assert_eq!(cfg, UserConfig::default());
    }

    #[test]
    fn full_schema_parses() {
        let path = write_cfg(
            "full",
            r#"[default]
account = "home"
debug   = "trace"

[account.home]
repo.default          = "remote"
repo.category.remote  = "git@github.com:winksaville"
repo.category.local   = "~/test-fixtures"

[account.work]
repo.default          = "remote"
repo.category.remote  = "git@github.com:anthropic"
repo.category.local   = "/work/fixtures"
"#,
        );
        let cfg = load_from(&path).unwrap();
        assert_eq!(cfg.default_account.as_deref(), Some("home"));
        assert_eq!(cfg.default_debug.as_deref(), Some("trace"));
        assert_eq!(cfg.accounts.len(), 2);

        let home = cfg.accounts.get("home").expect("home account");
        assert_eq!(home.repo_default.as_deref(), Some("remote"));
        assert_eq!(
            home.repo_category.get("remote").map(String::as_str),
            Some("git@github.com:winksaville")
        );
        assert_eq!(
            home.repo_category.get("local").map(String::as_str),
            Some("~/test-fixtures")
        );

        let work = cfg.accounts.get("work").expect("work account");
        assert_eq!(work.repo_default.as_deref(), Some("remote"));
        assert_eq!(
            work.repo_category.get("remote").map(String::as_str),
            Some("git@github.com:anthropic")
        );
    }

    #[test]
    fn account_only_no_default_section() {
        // No [default]; one account. accounts populated, top-level None.
        let path = write_cfg(
            "no-default",
            r#"[account.solo]
repo.default          = "remote"
repo.category.remote  = "git@github.com:wink"
"#,
        );
        let cfg = load_from(&path).unwrap();
        assert!(cfg.default_account.is_none());
        assert!(cfg.default_debug.is_none());
        assert_eq!(cfg.accounts.len(), 1);
        assert!(cfg.accounts.contains_key("solo"));
    }

    #[test]
    fn top_level_repo_shorthand() {
        // Single-account shorthand — top-level [repo], no [account.*].
        let path = write_cfg(
            "top-level",
            r#"[repo]
default          = "remote"
category.remote  = "git@github.com:wink"
category.local   = "~/test-fixtures"
"#,
        );
        let cfg = load_from(&path).unwrap();
        assert!(cfg.accounts.is_empty());
        let tl = cfg.top_level_repo.expect("top_level_repo populated");
        assert_eq!(tl.repo_default.as_deref(), Some("remote"));
        assert_eq!(
            tl.repo_category.get("remote").map(String::as_str),
            Some("git@github.com:wink")
        );
        assert_eq!(
            tl.repo_category.get("local").map(String::as_str),
            Some("~/test-fixtures")
        );
    }

    #[test]
    fn mixing_top_level_and_accounts_errors() {
        // Both top-level [repo] and [account.*] is ambiguous.
        let path = write_cfg(
            "mixed",
            r#"[repo]
default = "remote"

[account.home]
repo.default = "remote"
"#,
        );
        let err = load_from(&path).unwrap_err().to_string();
        assert!(err.contains("mixing top-level [repo]"), "got: {err}");
    }

    #[test]
    fn unknown_keys_ignored() {
        // Forward-compat: unknown sections/sub-keys silently dropped.
        let path = write_cfg(
            "unknown",
            r#"[account.home]
repo.category.remote = "git@github.com:wink"
repo.category.local  = "~/fixtures"
repo.unknown-subkey  = "ignored"
[future-section]
some-key = "ignored"
"#,
        );
        let cfg = load_from(&path).unwrap();
        let home = cfg.accounts.get("home").unwrap();
        assert_eq!(home.repo_category.len(), 2);
    }

    // ---------- resolve_repo ----------

    fn cfg_with_two_accounts() -> UserConfig {
        let home = AccountConfig {
            repo_default: Some("remote".into()),
            repo_category: HashMap::from([
                ("remote".into(), "git@github.com:winksaville".into()),
                ("local".into(), "~/test-fixtures".into()),
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

    #[test]
    fn resolve_no_flags_uses_all_defaults() {
        let cfg = cfg_with_two_accounts();
        let (cat, val) = resolve_repo(&cfg, None, None).unwrap();
        assert_eq!(cat, "remote");
        assert_eq!(val, "git@github.com:winksaville");
    }

    #[test]
    fn resolve_account_override() {
        let cfg = cfg_with_two_accounts();
        let (cat, val) = resolve_repo(&cfg, Some("work"), None).unwrap();
        assert_eq!(cat, "remote");
        assert_eq!(val, "git@github.com:anthropic");
    }

    #[test]
    fn resolve_repo_cat_only() {
        let cfg = cfg_with_two_accounts();
        let sel = RepoSelector {
            category: "local".into(),
            value: None,
        };
        let (cat, val) = resolve_repo(&cfg, None, Some(&sel)).unwrap();
        assert_eq!(cat, "local");
        assert_eq!(val, "~/test-fixtures");
    }

    #[test]
    fn resolve_repo_cat_and_value() {
        let cfg = cfg_with_two_accounts();
        let sel = RepoSelector {
            category: "remote".into(),
            value: Some("git@gitlab.com:other".into()),
        };
        let (cat, val) = resolve_repo(&cfg, None, Some(&sel)).unwrap();
        assert_eq!(cat, "remote");
        assert_eq!(val, "git@gitlab.com:other");
    }

    #[test]
    fn resolve_account_combined_with_repo_cat() {
        let cfg = cfg_with_two_accounts();
        let sel = RepoSelector {
            category: "local".into(),
            value: None,
        };
        let (cat, val) = resolve_repo(&cfg, Some("work"), Some(&sel)).unwrap();
        assert_eq!(cat, "local");
        assert_eq!(val, "/work/fixtures");
    }

    #[test]
    fn resolve_step1_no_account() {
        let cfg = UserConfig::default();
        let err = resolve_repo(&cfg, None, None).unwrap_err().to_string();
        assert!(err.contains("[default].account"), "got: {err}");
    }

    #[test]
    fn resolve_account_not_in_config() {
        let cfg = cfg_with_two_accounts();
        let err = resolve_repo(&cfg, Some("missing"), None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("[account.missing]"), "got: {err}");
    }

    #[test]
    fn resolve_step2_no_default_category() {
        let mut cfg = cfg_with_two_accounts();
        cfg.accounts.get_mut("home").unwrap().repo_default = None;
        let err = resolve_repo(&cfg, None, None).unwrap_err().to_string();
        assert!(err.contains("[account.home.repo].default"), "got: {err}");
    }

    #[test]
    fn resolve_step3_no_value_for_category() {
        let mut cfg = cfg_with_two_accounts();
        cfg.accounts.get_mut("home").unwrap().repo_category.clear();
        let sel = RepoSelector {
            category: "remote".into(),
            value: None,
        };
        let err = resolve_repo(&cfg, None, Some(&sel))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("[account.home.repo.category].remote"),
            "got: {err}"
        );
    }

    // ---------- top-level [repo] shorthand resolution ----------

    fn cfg_top_level() -> UserConfig {
        let tl = AccountConfig {
            repo_default: Some("remote".into()),
            repo_category: HashMap::from([
                ("remote".into(), "git@github.com:wink".into()),
                ("local".into(), "~/test-fixtures".into()),
            ]),
        };
        UserConfig {
            default_account: None,
            default_debug: None,
            top_level_repo: Some(tl),
            accounts: HashMap::new(),
        }
    }

    #[test]
    fn resolve_top_level_no_flags() {
        let cfg = cfg_top_level();
        let (cat, val) = resolve_repo(&cfg, None, None).unwrap();
        assert_eq!(cat, "remote");
        assert_eq!(val, "git@github.com:wink");
    }

    #[test]
    fn resolve_top_level_with_repo_cat() {
        let cfg = cfg_top_level();
        let sel = RepoSelector {
            category: "local".into(),
            value: None,
        };
        let (cat, val) = resolve_repo(&cfg, None, Some(&sel)).unwrap();
        assert_eq!(cat, "local");
        assert_eq!(val, "~/test-fixtures");
    }

    #[test]
    fn resolve_top_level_account_override_skips_top_level() {
        // Even with top-level [repo] present, --account forces the
        // account lookup path (no fallback). Account missing → error.
        let cfg = cfg_top_level();
        let err = resolve_repo(&cfg, Some("home"), None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("[account.home]"), "got: {err}");
    }

    #[test]
    fn resolve_top_level_step3_error_uses_bracket_form() {
        // Top-level shorthand error message uses [repo.category].<cat>
        // (no [account.*] prefix).
        let mut cfg = cfg_top_level();
        cfg.top_level_repo.as_mut().unwrap().repo_category.clear();
        let sel = RepoSelector {
            category: "remote".into(),
            value: None,
        };
        let err = resolve_repo(&cfg, None, Some(&sel))
            .unwrap_err()
            .to_string();
        assert!(err.contains("[repo.category].remote"), "got: {err}");
        assert!(!err.contains("[account."), "got: {err}");
    }
}

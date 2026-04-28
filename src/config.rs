//! User-level config loaded from `~/.config/vc-x1/config.toml`.
//!
//! Backs init's bare-NAME expansion (`vc-x1 init tf1` →
//! `git@<service>:<owner>/tf1.git`). No magic fallbacks — a
//! missing file or missing key produces a predictable error
//! pointing at the config.
//!
//! Schema:
//!
//! ```toml
//! [default]
//! remote-provider = "github"
//!
//! [github]
//! owner = "winksaville"
//! service = "github.com"   # optional, defaults to "github.com"
//! ```
//!
//! `[default].remote-provider` is the selector; per-provider
//! sections (`[github]`, future `[gitlab]`, …) carry the
//! provider-specific fields.
//!
//! See `notes/chores-08.md > User config (0.41.1-3)`.
//!
//! - `load()` reads the file at `config_path()` (honors
//!   `$XDG_CONFIG_HOME`); missing file → empty `UserConfig`;
//!   malformed file → fatal.
//! - `load_from(path)` is the same machinery against a caller-
//!   supplied path; used by tests and by `load()` itself.
//! - `-3` lands the module standalone; `-4` wires it into init.

use std::path::{Path, PathBuf};

use crate::toml_simple;

/// Resolved user config. All fields are `Option` — absent file or
/// absent keys yield `None`. Consumers apply their own defaults
/// (e.g. `service.unwrap_or("github.com")` at the use site).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct UserConfig {
    /// `[default].remote-provider` — selects which `[<provider>]`
    /// section's fields are used when expanding bare NAME.
    pub remote_provider: Option<String>,

    /// `[github].owner` — the GitHub user/org used as the implicit
    /// owner for bare NAME on the GitHub provider.
    pub github_owner: Option<String>,

    /// `[github].service` — host portion of the SSH URL (e.g.
    /// `github.com`, or a GHE host). Defaults at use site.
    pub github_service: Option<String>,
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
#[allow(dead_code)] // OK: 0.41.1-4 is the first consumer; staged ahead in -3.
pub fn load() -> Result<UserConfig, Box<dyn std::error::Error>> {
    load_from(&config_path()?)
}

/// Load the user config from an explicit path.
///
/// - Missing file → empty `UserConfig`.
/// - Malformed file → fatal error (propagated from `toml_simple`).
/// - Unknown sections / keys are silently ignored
///   (forward-compatible with future provider sections).
pub fn load_from(path: &Path) -> Result<UserConfig, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(UserConfig::default());
    }
    let map = toml_simple::toml_load(path)?;
    Ok(UserConfig {
        remote_provider: map.get("default.remote-provider").cloned(),
        github_owner: map.get("github.owner").cloned(),
        github_service: map.get("github.service").cloned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Build a unique tempdir per-test to avoid cross-thread races.
    fn cfg_tempdir(tag: &str) -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("vc-x1-cfg-{tag}-{ts}"));
        fs::create_dir_all(&dir).expect("mkdir tempdir");
        dir
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
        let dir = cfg_tempdir("empty");
        let path = dir.join("config.toml");
        fs::write(&path, "").unwrap();
        let cfg = load_from(&path).unwrap();
        assert_eq!(cfg, UserConfig::default());
    }

    #[test]
    fn full_schema_parses() {
        let dir = cfg_tempdir("full");
        let path = dir.join("config.toml");
        fs::write(
            &path,
            r#"[default]
remote-provider = "github"

[github]
owner = "winksaville"
service = "github.com"
"#,
        )
        .unwrap();
        let cfg = load_from(&path).unwrap();
        assert_eq!(cfg.remote_provider.as_deref(), Some("github"));
        assert_eq!(cfg.github_owner.as_deref(), Some("winksaville"));
        assert_eq!(cfg.github_service.as_deref(), Some("github.com"));
    }

    #[test]
    fn partial_schema_owner_only() {
        let dir = cfg_tempdir("owner-only");
        let path = dir.join("config.toml");
        fs::write(
            &path,
            r#"[github]
owner = "winksaville"
"#,
        )
        .unwrap();
        let cfg = load_from(&path).unwrap();
        assert!(cfg.remote_provider.is_none());
        assert_eq!(cfg.github_owner.as_deref(), Some("winksaville"));
        assert!(cfg.github_service.is_none());
    }

    #[test]
    fn partial_schema_provider_only() {
        let dir = cfg_tempdir("provider-only");
        let path = dir.join("config.toml");
        fs::write(
            &path,
            r#"[default]
remote-provider = "github"
"#,
        )
        .unwrap();
        let cfg = load_from(&path).unwrap();
        assert_eq!(cfg.remote_provider.as_deref(), Some("github"));
        assert!(cfg.github_owner.is_none());
        assert!(cfg.github_service.is_none());
    }

    #[test]
    fn unknown_sections_ignored() {
        let dir = cfg_tempdir("unknown-sections");
        let path = dir.join("config.toml");
        fs::write(
            &path,
            r#"[gitlab]
owner = "wink"

[github]
owner = "winksaville"
"#,
        )
        .unwrap();
        let cfg = load_from(&path).unwrap();
        // [gitlab] section silently ignored — no struct field for it yet.
        assert_eq!(cfg.github_owner.as_deref(), Some("winksaville"));
    }

    #[test]
    fn comments_and_blank_lines_tolerated() {
        let dir = cfg_tempdir("comments");
        let path = dir.join("config.toml");
        fs::write(
            &path,
            r#"# top-level comment
[default]
remote-provider = "github"

# section comment
[github]
owner = "winksaville"
"#,
        )
        .unwrap();
        let cfg = load_from(&path).unwrap();
        assert_eq!(cfg.remote_provider.as_deref(), Some("github"));
        assert_eq!(cfg.github_owner.as_deref(), Some("winksaville"));
    }
}

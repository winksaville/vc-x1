//! `--config none|<path>` flag — controls whether a subcommand
//! writes its canned `.vc-config.toml` or substitutes a
//! user-provided file.
//!
//! - `ConfigKind::None` — skip writing entirely.
//! - `ConfigKind::Path(p)` — copy `p` into place instead of the
//!   canned content.
//! - Empty input → caller-supplied default. The parser takes the
//!   default as a parameter so each consumer can plug in its own
//!   canonical canned shape (init's POR uses one canned config;
//!   future consumers may use others).

use std::path::PathBuf;

/// Parsed `--config` value.
///
/// - `None` — explicit skip (`--config none`).
/// - `Path(p)` — explicit user-provided file (`--config <path>`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigKind {
    None,
    Path(PathBuf),
}

/// Parse the `--config` value into a `ConfigKind`, substituting
/// `default` when the input is empty.
///
/// - `""` → `default` (caller-supplied — each consumer plugs in its
///   own canonical canned shape).
/// - `"none"` → `ConfigKind::None`.
/// - Anything else → `ConfigKind::Path(s.into())`.
///
/// No path-prefix discipline (`./`, `~/`, etc.) — `--config` has
/// only one keyword (`none`), so any other string is unambiguously
/// a path. Path validation (existence, readability) happens at the
/// consumer's preflight, not here.
pub fn parse_config_kind(s: &str, default: ConfigKind) -> ConfigKind {
    match s {
        "" => default,
        "none" => ConfigKind::None,
        _ => ConfigKind::Path(PathBuf::from(s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stand-in default used in tests where the default branch isn't
    /// the one being exercised. `ConfigKind::None` is distinguishable
    /// from any `Path(_)` and from an "explicit none" return so
    /// confusions surface as test failures.
    fn test_default() -> ConfigKind {
        ConfigKind::None
    }

    #[test]
    fn keyword_none() {
        assert_eq!(parse_config_kind("none", test_default()), ConfigKind::None,);
    }

    #[test]
    fn relative_path() {
        assert_eq!(
            parse_config_kind("./my-config.toml", test_default()),
            ConfigKind::Path(PathBuf::from("./my-config.toml")),
        );
    }

    #[test]
    fn absolute_path() {
        assert_eq!(
            parse_config_kind("/etc/vc-x1/config.toml", test_default()),
            ConfigKind::Path(PathBuf::from("/etc/vc-x1/config.toml")),
        );
    }

    #[test]
    fn home_relative_path() {
        assert_eq!(
            parse_config_kind("~/configs/foo.toml", test_default()),
            ConfigKind::Path(PathBuf::from("~/configs/foo.toml")),
        );
    }

    #[test]
    fn bare_filename_treated_as_path() {
        // No `./` prefix discipline — single keyword `none`, anything
        // else is a path. Existence-check happens at consumer preflight.
        assert_eq!(
            parse_config_kind("foo.toml", test_default()),
            ConfigKind::Path(PathBuf::from("foo.toml")),
        );
    }

    #[test]
    fn empty_returns_default() {
        let default = ConfigKind::Path(PathBuf::from("/canned/init-por.toml"));
        assert_eq!(parse_config_kind("", default.clone()), default);
    }

    #[test]
    fn empty_default_can_be_none() {
        // Caller may pass ConfigKind::None as its own default; in that
        // case empty and "none" both return None — same value, different
        // intent. Distinguishing them is the caller's responsibility.
        assert_eq!(parse_config_kind("", ConfigKind::None), ConfigKind::None,);
    }
}

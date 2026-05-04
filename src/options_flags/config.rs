//! `--config none|<path>` — `.vc-config.toml` write override.
//! See [options_flags](README.md) for shared architecture.

use std::path::PathBuf;

use clap::Args;

/// Parsed `--config` value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigKind {
    /// Skip writing entirely (`--config none`).
    None,
    /// User-provided file (`--config <path>`).
    Path(PathBuf),
}

/// Parse `--config`; `""` returns `default`, `"none"` returns
/// `ConfigKind::None`, anything else is a path.
pub fn parse_config_kind(s: &str, default: ConfigKind) -> ConfigKind {
    match s {
        "" => default,
        "none" => ConfigKind::None,
        _ => ConfigKind::Path(PathBuf::from(s)),
    }
}

/// `--config none|<path>` leaf — see
/// [Consuming an OF](README.md#consuming-an-of).
#[derive(Args, Debug, Clone, Default)]
pub struct ConfigFlag {
    /// Override the canned `.vc-config.toml` write.
    ///
    /// - Absent: write the canned `.vc-config.toml`.
    /// - `--config none`: skip writing entirely.
    /// - `--config <path>`: copy `<path>` to `.vc-config.toml`
    ///   (bytewise; no schema validation).
    #[arg(long = "config", value_name = "none|PATH", verbatim_doc_comment)]
    pub raw: Option<String>,
}

impl super::FlagBundle for ConfigFlag {}

impl ConfigFlag {
    /// Resolve `raw` against `default`; `None` when flag absent.
    pub fn resolve(&self, default: ConfigKind) -> Option<ConfigKind> {
        self.raw.as_deref().map(|s| parse_config_kind(s, default))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stand-in default for tests where the default branch isn't
    /// the one being exercised.
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
        assert_eq!(parse_config_kind("", ConfigKind::None), ConfigKind::None,);
    }

    #[test]
    fn config_flag_resolve_absent() {
        let flag = ConfigFlag { raw: None };
        assert_eq!(flag.resolve(test_default()), None);
    }

    #[test]
    fn config_flag_resolve_explicit_none() {
        let flag = ConfigFlag {
            raw: Some("none".to_string()),
        };
        assert_eq!(flag.resolve(test_default()), Some(ConfigKind::None));
    }

    #[test]
    fn config_flag_resolve_path() {
        let flag = ConfigFlag {
            raw: Some("/etc/foo.toml".to_string()),
        };
        assert_eq!(
            flag.resolve(test_default()),
            Some(ConfigKind::Path(PathBuf::from("/etc/foo.toml"))),
        );
    }

    #[test]
    fn config_flag_resolve_empty_uses_default() {
        let default = ConfigKind::Path(PathBuf::from("/canned/init-por.toml"));
        let flag = ConfigFlag {
            raw: Some(String::new()),
        };
        assert_eq!(flag.resolve(default.clone()), Some(default));
    }
}

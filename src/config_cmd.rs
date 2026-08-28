//! The `config` subcommand: print the settable config schema for a
//! target config file (or check it with `--validate`), grouped by
//! TOML section, sshd_config style.
//!
//! - Read-only. Consumes `crate::config_schema::schema()`: the
//!   single source of truth for every settable key.
//! - The positional target is `work`, `agent`, `work,agent` (default),
//!   or an explicit config-file path. The user config
//!   (`~/.config/vc-x1/config.toml`) has no keyword: reach it by
//!   passing its path.
//! - The side keywords filter to that side's keys. A path target
//!   carries no side information, so it gets the whole schema: no
//!   guessing what kind of file the path names.
//! - `--validate` checks the target file(s) instead of printing:
//!   unknown keys, and (for keyword targets) the legacy-schema
//!   rejection plus the resolved-agreement invariant of a dual
//!   workspace's `[repos]` registries (via the bot-side
//!   resolution).

use std::path::{Path, PathBuf};

use clap::Args;

use log::info;

use crate::common::{configured_bot_dir, find_workspace_root};
use crate::config_schema::{ConfigKey, Home, render_key_block, schema, section_and_leaf};
use crate::context::Context;
use crate::desc_helpers::VC_CONFIG_FILE;
use crate::options_flags::scope::{Scope, Side, parse_scope};
use crate::subcommand::SubcommandRunner;

/// Parsed positional target: a side keyword set or an explicit
/// config-file path.
///
/// - `Scope`: `work`, `agent`, `work,agent`, `agent,work` (the `--scope`
///   grammar), resolved against the surrounding workspace.
/// - `Path`: anything else, an explicit config file. The only way
///   to reach the user config.
#[derive(Clone, Debug, PartialEq)]
pub enum ConfigTarget {
    Scope(Scope),
    Path(PathBuf),
}

/// Parse the positional target: side keyword form first, else a
/// path.
///
/// A file literally named `work` (etc.) in cwd is shadowed by the
/// keyword: target it as `./work`.
pub(crate) fn parse_target(s: &str) -> Result<ConfigTarget, String> {
    match parse_scope(s) {
        Ok(scope) => return Ok(ConfigTarget::Scope(scope)),
        // The old `bot` keywords are a rejection, not a path.
        Err(e) if matches!(s, "bot" | "work,bot" | "bot,work") => return Err(e),
        Err(_) => {}
    }
    if s.is_empty() {
        return Err("config: target is empty".into());
    }
    Ok(ConfigTarget::Path(PathBuf::from(s)))
}

#[derive(Args, Debug)]
pub struct ConfigArgs {
    /// What to print or validate: side keyword(s) `work`, `agent`,
    /// `work,agent`, or a config-file path. The user config
    /// (`~/.config/vc-x1/config.toml`) has no keyword: pass its
    /// path.
    #[arg(value_parser = parse_target, default_value = "work,agent", verbatim_doc_comment)]
    pub target: ConfigTarget,

    /// Retired at 0.80.6: the check is `vc-x1 validate-config`,
    /// which takes the same target. Kept as a hidden flag so the
    /// old spelling gets a fix-it rather than clap's "unexpected
    /// argument".
    #[arg(long, hide = true)]
    pub validate: bool,
}

/// Inputs to the `config` op, flat, owned, clap-free.
///
/// Mirrors `ConfigArgs`: positional target (`target`), `--validate`
/// (`validate`).
pub struct ConfigParams {
    pub target: ConfigTarget,
    pub validate: bool,
}

impl From<&ConfigArgs> for ConfigParams {
    /// Convert clap-derived `ConfigArgs` into the flat
    /// `ConfigParams` (total: every field copies straight over).
    fn from(a: &ConfigArgs) -> Self {
        Self {
            target: a.target.clone(),
            validate: a.validate,
        }
    }
}

impl SubcommandRunner for ConfigArgs {
    type Params = ConfigParams;

    /// Delegate to the existing `From<&ConfigArgs>` impl above
    /// (total: never fails).
    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(ConfigParams::from(self))
    }

    /// Run the `config` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        config(ctx, params)
    }
}

/// A key is settable on the work side if its homes include
/// `Home::WorkspaceCode`.
pub(crate) fn in_work_side(homes: &[Home]) -> bool {
    homes.contains(&Home::WorkspaceCode)
}

/// A key is settable on the bot side if its homes include
/// `Home::WorkspaceBot`.
pub(crate) fn in_bot_side(homes: &[Home]) -> bool {
    homes.contains(&Home::WorkspaceBot)
}

/// Every key: a path target carries no side information, so it
/// prints/validates against the whole schema, no guessing.
pub(crate) fn in_any(_homes: &[Home]) -> bool {
    true
}

/// Print one target group: a divider (`header` verbatim), then each
/// key grouped by section (schema order, first-seen section order),
/// one `[section]` header per section. Each key renders as a
/// multi-line doc-block via `render_key_block` (shared with
/// `crate::init`'s generated `.vc-config.md`).
fn print_group(header: &str, keys: &[&ConfigKey]) {
    info!("# -- {header} --");
    let mut sections: Vec<&str> = Vec::new();
    for key in keys {
        let (section, _leaf) = section_and_leaf(key.path);
        if !sections.contains(&section) {
            sections.push(section);
        }
    }
    for section in sections {
        info!("[{section}]");
        for key in keys {
            let (key_section, _leaf) = section_and_leaf(key.path);
            if key_section != section {
                continue;
            }
            for line in render_key_block(key).lines() {
                info!("{line}");
            }
        }
    }
    info!("");
}

/// The group-hint path for a side's directory: the carrier that
/// actually exists there, falling back to the toml name when the
/// side has none (or holds both).
fn resolved_hint(dir: &Path) -> String {
    match crate::config_md::vc_config_path(dir) {
        Ok(Some(p)) => p.display().to_string(),
        _ => dir.join(VC_CONFIG_FILE).display().to_string(),
    }
}

/// Print the settable config schema for the target: one group per
/// resolved side (keyword target) or one group for the named file
/// (path target), with the resolved file path as the group hint
/// when the workspace provides one.
fn print_schema(params: &ConfigParams, root: Option<&Path>) {
    info!(
        "# vc-x1 settable config keys (from vc-x1 {})",
        env!("CARGO_PKG_VERSION")
    );
    info!("# Keys below are shown at their built-in default (commented");
    info!("# unless required); run `vc-x1 config --validate` to check a");
    info!("# config file's keys against this schema.");
    info!("");

    let all = schema();
    let group = |pred: fn(&[Home]) -> bool| -> Vec<&ConfigKey> {
        all.iter().filter(|k| pred(k.homes)).collect()
    };

    match &params.target {
        ConfigTarget::Scope(scope) => {
            for side in &scope.0 {
                match side {
                    Side::Work => {
                        let hint = match root {
                            Some(r) => resolved_hint(r),
                            None => format!("<root>/{VC_CONFIG_FILE}"),
                        };
                        print_group(&format!("work: {hint}"), &group(in_work_side));
                    }
                    Side::Bot => {
                        let bot = root.and_then(|r| configured_bot_dir(r).ok().flatten());
                        let hint = match bot {
                            Some(b) => resolved_hint(&b),
                            None => format!("<root>/<agent-dir>/{VC_CONFIG_FILE}"),
                        };
                        print_group(&format!("agent: {hint}"), &group(in_bot_side));
                    }
                }
            }
        }
        ConfigTarget::Path(path) => {
            print_group(&path.display().to_string(), &group(in_any));
        }
    }
}

/// Print the settable config schema for the target (default), or
/// (with `--validate`) check the target's config file(s) and exit
/// non-zero if any problem is found.
pub fn config(_ctx: &Context, params: &ConfigParams) -> Result<(), Box<dyn std::error::Error>> {
    if params.validate {
        return Err(
            "--validate: retired at 0.80.6. The check is `vc-x1 validate-config`, \
                    which takes the same target."
                .into(),
        );
    }
    let root = find_workspace_root();
    print_schema(params, root.as_deref());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cli, Commands};
    use clap::Parser;

    fn parse(args: &[&str]) -> ConfigArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::Config(a)) => a,
            _ => panic!("expected Config"),
        }
    }

    /// The old `bot` keywords are a rejection, not a path target.
    #[test]
    fn parse_target_rejects_old_bot_keywords() {
        let err = parse_target("bot").unwrap_err();
        assert!(err.contains("`agent`"), "got: {err}");
        assert!(parse_target("work,bot").is_err());
    }

    #[test]
    fn parse_target_keywords() {
        assert_eq!(
            parse_target("work").unwrap(),
            ConfigTarget::Scope(Scope(vec![Side::Work]))
        );
        assert_eq!(
            parse_target("agent").unwrap(),
            ConfigTarget::Scope(Scope(vec![Side::Bot]))
        );
        assert_eq!(
            parse_target("work,agent").unwrap(),
            ConfigTarget::Scope(Scope(vec![Side::Work, Side::Bot]))
        );
    }

    #[test]
    fn parse_target_path_fallback() {
        assert_eq!(
            parse_target("./work").unwrap(),
            ConfigTarget::Path(PathBuf::from("./work"))
        );
        assert_eq!(
            parse_target("some/config.toml").unwrap(),
            ConfigTarget::Path(PathBuf::from("some/config.toml"))
        );
    }

    #[test]
    fn parse_target_empty_errors() {
        assert!(parse_target("").is_err());
    }

    #[test]
    fn no_args_defaults_to_both_sides() {
        let args = parse(&["vc-x1", "config"]);
        assert_eq!(
            args.target,
            ConfigTarget::Scope(Scope(vec![Side::Work, Side::Bot]))
        );
        assert!(!args.validate);
    }

    #[test]
    fn positional_path_target() {
        let args = parse(&["vc-x1", "config", "../foo/.vc-config.toml"]);
        assert_eq!(
            args.target,
            ConfigTarget::Path(PathBuf::from("../foo/.vc-config.toml"))
        );
    }

    #[test]
    fn home_flag_retired() {
        let err = Cli::try_parse_from(["vc-x1", "config", "--home", "user"])
            .unwrap_err()
            .to_string();
        assert!(err.contains("--home"), "got: {err}");
    }
}

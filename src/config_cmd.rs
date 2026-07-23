//! The `config` subcommand: print the settable config schema for a
//! target config file (or check it with `--validate`), grouped by
//! TOML section, sshd_config style.
//!
//! - Read-only; consumes `crate::config_schema::schema()` — the
//!   single source of truth for every settable key.
//! - The positional target is `work`, `bot`, `work,bot` (default),
//!   or an explicit config-file path. The user config
//!   (`~/.config/vc-x1/config.toml`) has no keyword — reach it by
//!   passing its path.
//! - The side keywords filter to that side's keys; a path target
//!   carries no side information, so it gets the whole schema —
//!   no guessing what kind of file the path names.
//! - `--validate` checks the target file(s) instead of printing:
//!   unknown keys, and — for keyword targets — the `[workspace]`
//!   grammar plus the identical-`[workspace]`-block invariant of a
//!   dual workspace (via the bot-side resolution).

use std::error::Error;
use std::path::{Path, PathBuf};

use clap::Args;

use log::{info, warn};

use crate::common::{bot_repo_path, configured_bot_dir, find_workspace_root, reject_legacy_config};
use crate::config_schema::{ConfigKey, Home, render_key_block, schema, section_and_leaf};
use crate::context::Context;
use crate::desc_helpers::VC_CONFIG_FILE;
use crate::options_flags::scope::{Scope, Side, parse_scope};
use crate::subcommand::SubcommandRunner;
use crate::toml_simple::toml_load;

/// Parsed positional target: a side keyword set or an explicit
/// config-file path.
///
/// - `Scope` — `work`, `bot`, `work,bot`, `bot,work` (the `--scope`
///   grammar), resolved against the surrounding workspace.
/// - `Path` — anything else: an explicit config file. The only way
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
/// keyword — target it as `./work`.
fn parse_target(s: &str) -> Result<ConfigTarget, String> {
    if let Ok(scope) = parse_scope(s) {
        return Ok(ConfigTarget::Scope(scope));
    }
    if s.is_empty() {
        return Err("config: target is empty".into());
    }
    Ok(ConfigTarget::Path(PathBuf::from(s)))
}

#[derive(Args, Debug)]
pub struct ConfigArgs {
    /// What to print or validate: side keyword(s) `work`, `bot`,
    /// `work,bot`, or a config-file path. The user config
    /// (`~/.config/vc-x1/config.toml`) has no keyword — pass its
    /// path.
    #[arg(value_parser = parse_target, default_value = "work,bot", verbatim_doc_comment)]
    pub target: ConfigTarget,

    /// Check the target config file(s) — unknown/misspelled keys,
    /// [workspace] grammar, identical-[workspace]-block invariant —
    /// instead of printing the schema
    #[arg(long)]
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
    /// `ConfigParams` (total — every field copies straight over).
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
    /// (total — never fails).
    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(ConfigParams::from(self))
    }

    /// Run the `config` op.
    fn run(ctx: &Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        config(ctx, params)
    }
}

/// A key is settable on the work side if its homes include
/// `Home::WorkspaceCode`.
fn in_work_side(homes: &[Home]) -> bool {
    homes.contains(&Home::WorkspaceCode)
}

/// A key is settable on the bot side if its homes include
/// `Home::WorkspaceBot`.
fn in_bot_side(homes: &[Home]) -> bool {
    homes.contains(&Home::WorkspaceBot)
}

/// Every key — a path target carries no side information, so it
/// prints/validates against the whole schema, no guessing.
fn in_any(_homes: &[Home]) -> bool {
    true
}

/// Print one target group: a divider (`header` verbatim), then each
/// key grouped by section (schema order, first-seen section order),
/// one `[section]` header per section. Each key renders as a
/// multi-line doc-block via `render_key_block` (shared with
/// `crate::init`'s generated `.vc-config.toml`).
fn print_group(header: &str, keys: &[&ConfigKey]) {
    info!("# ── {header} ──");
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

/// True if `actual` (a config key from a loaded config file) is
/// recognized by some `schema()` entry whose homes satisfy
/// `home_pred`.
///
/// - Non-dynamic entries match by exact path equality.
/// - Dynamic entries (`key.dynamic`) match segment-wise: equal
///   segment counts, each entry segment either equal to the actual
///   segment or a `<placeholder>` matching any single segment.
fn key_known(actual: &str, home_pred: fn(&[Home]) -> bool) -> bool {
    schema().iter().any(|key| {
        if !home_pred(key.homes) {
            return false;
        }
        if !key.dynamic {
            return key.path == actual;
        }
        let entry_segs: Vec<&str> = key.path.split('.').collect();
        let actual_segs: Vec<&str> = actual.split('.').collect();
        entry_segs.len() == actual_segs.len()
            && entry_segs
                .iter()
                .zip(actual_segs.iter())
                .all(|(e, a)| e == a || (e.starts_with('<') && e.ends_with('>')))
    })
}

/// Validate one config file against the schema, filtered to the
/// homes accepted at that file by `home_pred`.
///
/// - A missing file is not an error — `info!`s that it's absent
///   and returns `Ok(0)`.
/// - Each key not recognized by `key_known` is reported with
///   `warn!`, naming `label` and the key; keys are checked in
///   sorted order for stable output.
/// - Returns the count of unknown keys found. A load error
///   (malformed TOML) propagates as `Err`.
fn validate_file(
    path: &Path,
    label: &str,
    home_pred: fn(&[Home]) -> bool,
) -> Result<usize, Box<dyn Error>> {
    if !path.exists() {
        info!("{label}: {} not found — skipping", path.display());
        return Ok(0);
    }
    let map = toml_load(path)?;
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();

    let mut unknown = 0;
    for key in keys {
        if !key_known(key, home_pred) {
            warn!("{label} ({}): unknown key {key:?}", path.display());
            unknown += 1;
        }
    }
    Ok(unknown)
}

/// Validate the target's config file(s), returning the total count
/// of problems found (unknown keys, legacy/grammar rejections,
/// workspace-coherence failures).
///
/// - Keyword targets resolve against `root`; outside a workspace
///   there is nothing to check (info + `Ok(0)`).
/// - The bot side resolves via `bot_repo_path`, which runs the
///   dual-preflight coherence check (bot dir exists, both configs
///   load, identical `[workspace]` blocks) — its failure is
///   reported as a finding, not a hard error, so the work-side
///   report still lands.
/// - A path target carries no side information, so it validates
///   against the whole schema (any home's keys accepted).
fn validate(params: &ConfigParams, root: Option<&Path>) -> Result<usize, Box<dyn Error>> {
    let mut findings = 0;
    match &params.target {
        ConfigTarget::Scope(scope) => {
            let Some(root) = root else {
                info!("not inside a workspace — nothing to validate");
                return Ok(0);
            };
            if let Err(e) = reject_legacy_config(root) {
                warn!("{e}");
                findings += 1;
            }
            for side in &scope.0 {
                match side {
                    Side::Work => {
                        findings +=
                            validate_file(&root.join(VC_CONFIG_FILE), "work config", in_work_side)?;
                    }
                    Side::Bot => match bot_repo_path(root) {
                        Ok(Some(bot)) => {
                            findings += validate_file(
                                &bot.join(VC_CONFIG_FILE),
                                "bot config",
                                in_bot_side,
                            )?;
                        }
                        Ok(None) => info!("bot config: no bot repo configured — skipping"),
                        Err(e) => {
                            warn!("{e}");
                            findings += 1;
                        }
                    },
                }
            }
        }
        ConfigTarget::Path(path) => {
            findings += validate_file(path, "config file", in_any)?;
        }
    }
    Ok(findings)
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
                            Some(r) => r.join(VC_CONFIG_FILE).display().to_string(),
                            None => format!("<root>/{VC_CONFIG_FILE}"),
                        };
                        print_group(&format!("work: {hint}"), &group(in_work_side));
                    }
                    Side::Bot => {
                        let bot = root.and_then(|r| configured_bot_dir(r).ok().flatten());
                        let hint = match bot {
                            Some(b) => b.join(VC_CONFIG_FILE).display().to_string(),
                            None => format!("<root>/<bot-dir>/{VC_CONFIG_FILE}"),
                        };
                        print_group(&format!("bot: {hint}"), &group(in_bot_side));
                    }
                }
            }
        }
        ConfigTarget::Path(path) => {
            print_group(&path.display().to_string(), &group(in_any));
        }
    }
}

/// Print the settable config schema for the target (default), or —
/// with `--validate` — check the target's config file(s) and exit
/// non-zero if any problem is found.
pub fn config(_ctx: &Context, params: &ConfigParams) -> Result<(), Box<dyn std::error::Error>> {
    let root = find_workspace_root();
    if params.validate {
        let total = validate(params, root.as_deref())?;
        return if total > 0 {
            Err(format!("config: {total} problem(s) found").into())
        } else {
            info!("config: all checks passed");
            Ok(())
        };
    }
    print_schema(params, root.as_deref());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{Fixture, FixturePor};
    use crate::{Cli, Commands};
    use clap::Parser;

    fn parse(args: &[&str]) -> ConfigArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::Config(a)) => a,
            _ => panic!("expected Config"),
        }
    }

    /// Append raw TOML to a config file.
    fn append(path: &Path, extra: &str) {
        let mut text = std::fs::read_to_string(path).expect("read config");
        text.push_str(extra);
        std::fs::write(path, text).expect("write config");
    }

    #[test]
    fn parse_target_keywords() {
        assert_eq!(
            parse_target("work").unwrap(),
            ConfigTarget::Scope(Scope(vec![Side::Work]))
        );
        assert_eq!(
            parse_target("bot").unwrap(),
            ConfigTarget::Scope(Scope(vec![Side::Bot]))
        );
        assert_eq!(
            parse_target("work,bot").unwrap(),
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

    #[test]
    fn validate_dual_workspace_clean() {
        let fx = Fixture::new("config-validate-clean");
        let params = ConfigParams {
            target: ConfigTarget::Scope(Scope(vec![Side::Work, Side::Bot])),
            validate: true,
        };
        let findings = validate(&params, Some(&fx.work)).expect("validate");
        assert_eq!(findings, 0);
    }

    #[test]
    fn validate_flags_unknown_key() {
        let fx = Fixture::new("config-validate-unknown");
        append(
            &fx.work.join(VC_CONFIG_FILE),
            "\n[bogus-section]\nkey = \"v\"\n",
        );
        let params = ConfigParams {
            target: ConfigTarget::Scope(Scope(vec![Side::Work, Side::Bot])),
            validate: true,
        };
        let findings = validate(&params, Some(&fx.work)).expect("validate");
        assert_eq!(findings, 1);
    }

    #[test]
    fn validate_flags_incoherent_workspace_blocks() {
        // Diverge the bot side's [workspace] block: the bot-side
        // resolution runs the dual-preflight coherence check, which
        // must surface as a finding (not a hard error) so the
        // work-side report still lands.
        let fx = Fixture::new("config-validate-incoherent");
        std::fs::write(
            fx.bot.join(VC_CONFIG_FILE),
            "[workspace]\nwork = \"/\"\nbot = \"/other\"\n",
        )
        .expect("rewrite bot config");
        let params = ConfigParams {
            target: ConfigTarget::Scope(Scope(vec![Side::Work, Side::Bot])),
            validate: true,
        };
        let findings = validate(&params, Some(&fx.work)).expect("validate");
        assert_eq!(findings, 1);
    }

    #[test]
    fn validate_single_repo_skips_bot_side() {
        let fx = FixturePor::new("config-validate-por");
        let params = ConfigParams {
            target: ConfigTarget::Scope(Scope(vec![Side::Work, Side::Bot])),
            validate: true,
        };
        let findings = validate(&params, Some(&fx.work)).expect("validate");
        assert_eq!(findings, 0);
    }

    #[test]
    fn validate_outside_workspace_is_clean() {
        let params = ConfigParams {
            target: ConfigTarget::Scope(Scope(vec![Side::Work, Side::Bot])),
            validate: true,
        };
        let findings = validate(&params, None).expect("validate");
        assert_eq!(findings, 0);
    }

    #[test]
    fn validate_explicit_path_target() {
        let fx = Fixture::new("config-validate-path");
        let path = fx.work.join(VC_CONFIG_FILE);
        append(&path, "\n[bogus-section]\nkey = \"v\"\n");
        let params = ConfigParams {
            target: ConfigTarget::Path(path),
            validate: true,
        };
        let findings = validate(&params, Some(&fx.work)).expect("validate");
        assert_eq!(findings, 1);
    }
}

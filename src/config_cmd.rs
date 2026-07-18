//! The `config` subcommand: print the settable config schema,
//! grouped by config home then TOML section, sshd_config style.
//!
//! Read-only; consumes `crate::config_schema::schema()` — the
//! single source of truth for every settable key. `--home` filters
//! which home group(s) are printed.

use std::error::Error;

use clap::{Args, ValueEnum};

use log::{info, warn};

use crate::common::find_workspace_root;
use crate::config::config_path;
use crate::config_schema::{ConfigKey, Home, render_key_block, schema, section_and_leaf};
use crate::context::Context;
use crate::desc_helpers::VC_CONFIG_FILE;
use crate::subcommand::SubcommandRunner;
use crate::toml_simple::toml_load;

/// Which config home(s) to print.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum HomeFilter {
    /// Only the user config (`~/.config/vc-x1/config.toml`).
    User,
    /// Only the workspace config (`<root>/.vc-config.toml`).
    Workspace,
    /// Both homes (default).
    All,
}

#[derive(Args, Debug)]
pub struct ConfigArgs {
    /// Which config home(s) to print: user, workspace, or all
    #[arg(long, value_enum, default_value_t = HomeFilter::All)]
    pub home: HomeFilter,

    /// Check the config file(s) for unknown/misspelled keys instead
    /// of printing the schema
    #[arg(long)]
    pub validate: bool,
}

/// Inputs to the `config` op, flat, owned, clap-free.
///
/// Mirrors `ConfigArgs`: `--home` (`home`), `--validate` (`validate`).
pub struct ConfigParams {
    pub home: HomeFilter,
    pub validate: bool,
}

impl From<&ConfigArgs> for ConfigParams {
    /// Convert clap-derived `ConfigArgs` into the flat
    /// `ConfigParams` (total — every field copies straight over).
    fn from(a: &ConfigArgs) -> Self {
        Self {
            home: a.home,
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

/// A key belongs to the User group if any of its homes is
/// `Home::User`.
fn in_user_group(key: &ConfigKey) -> bool {
    key.homes.contains(&Home::User)
}

/// A key belongs to the Workspace group if any of its homes is
/// `Home::WorkspaceCode` or `Home::WorkspaceBot`.
fn in_workspace_group(key: &ConfigKey) -> bool {
    key.homes.contains(&Home::WorkspaceCode) || key.homes.contains(&Home::WorkspaceBot)
}

/// Print one home group: a divider, then each key grouped by
/// section (schema order, first-seen section order), one
/// `[section]` header per section. Each key renders as a
/// multi-line doc-block via `render_key_block` (shared with
/// `crate::init`'s generated `.vc-config.toml`).
fn print_group(title: &str, path_hint: &str, keys: &[&ConfigKey]) {
    info!("# ── {title} config: {path_hint} ──");
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

/// True if `actual` (a dotted key from a loaded config file) is
/// recognized by some `schema()` entry whose homes satisfy
/// `home_pred`.
///
/// - Non-dynamic entries match by exact path equality.
/// - Dynamic entries (`key.dynamic`) match segment-wise: equal
///   segment counts, each entry segment either equal to the actual
///   segment or a `<placeholder>` matching any single segment.
fn key_known(actual: &str, home_pred: impl Fn(&[Home]) -> bool) -> bool {
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
    path: &std::path::Path,
    label: &str,
    home_pred: impl Fn(&[Home]) -> bool,
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
        if !key_known(key, &home_pred) {
            warn!("{label} ({}): unknown key {key:?}", path.display());
            unknown += 1;
        }
    }
    Ok(unknown)
}

/// Validate the selected home(s)' config file(s) against the
/// schema, returning the total count of unknown keys.
fn validate(params: &ConfigParams) -> Result<usize, Box<dyn Error>> {
    let mut unknown = 0;

    if matches!(params.home, HomeFilter::User | HomeFilter::All) {
        unknown += validate_file(&config_path()?, "user config", |homes| {
            homes.contains(&Home::User)
        })?;
    }

    if matches!(params.home, HomeFilter::Workspace | HomeFilter::All) {
        match find_workspace_root() {
            Some(root) => {
                unknown +=
                    validate_file(&root.join(VC_CONFIG_FILE), "workspace config", |homes| {
                        homes.contains(&Home::WorkspaceCode) || homes.contains(&Home::WorkspaceBot)
                    })?;
            }
            None => info!("workspace config: not inside a workspace — skipping"),
        }
    }

    Ok(unknown)
}

/// Print the settable config schema, grouped by config home then
/// TOML section (`--home` selects which home group(s) to print),
/// or — with `--validate` — check the config file(s) for unknown
/// keys and exit non-zero if any are found.
pub fn config(_ctx: &Context, params: &ConfigParams) -> Result<(), Box<dyn std::error::Error>> {
    if params.validate {
        let total = validate(params)?;
        return if total > 0 {
            Err(format!("config: {total} unknown key(s) found").into())
        } else {
            info!("config: all keys recognized");
            Ok(())
        };
    }

    info!(
        "# vc-x1 settable config keys (from vc-x1 {})",
        env!("CARGO_PKG_VERSION")
    );
    info!("# Keys below are shown at their built-in default (commented");
    info!("# unless required); run `vc-x1 config --validate` to check a");
    info!("# config file's keys against this schema.");
    info!("");

    let all_keys = schema();

    if matches!(params.home, HomeFilter::User | HomeFilter::All) {
        let user_keys: Vec<&ConfigKey> = all_keys.iter().filter(|k| in_user_group(k)).collect();
        print_group("User", "~/.config/vc-x1/config.toml", &user_keys);
    }

    if matches!(params.home, HomeFilter::Workspace | HomeFilter::All) {
        let workspace_keys: Vec<&ConfigKey> =
            all_keys.iter().filter(|k| in_workspace_group(k)).collect();
        print_group("Workspace", "<root>/.vc-config.toml", &workspace_keys);
    }

    Ok(())
}

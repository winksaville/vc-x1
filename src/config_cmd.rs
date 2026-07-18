//! The `config` subcommand: print the settable config schema,
//! grouped by config home then TOML section, sshd_config style.
//!
//! Read-only; consumes `crate::config_schema::schema()` — the
//! single source of truth for every settable key. `--home` filters
//! which home group(s) are printed.

use clap::{Args, ValueEnum};

use log::info;

use crate::config_schema::{ConfigKey, Home, ValueKind, schema};
use crate::context::Context;
use crate::subcommand::SubcommandRunner;

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
}

/// Inputs to the `config` op, flat, owned, clap-free.
///
/// Mirrors `ConfigArgs`: `--home` (`home`).
pub struct ConfigParams {
    pub home: HomeFilter,
}

impl From<&ConfigArgs> for ConfigParams {
    /// Convert clap-derived `ConfigArgs` into the flat
    /// `ConfigParams` (total — the only field copies straight
    /// over).
    fn from(a: &ConfigArgs) -> Self {
        Self { home: a.home }
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

/// Split a dotted key path on its last `.` into `(section, leaf)`.
///
/// A path with no `.` (none exist in the current schema) falls
/// back to an empty section so the key still renders under a
/// blank `[]` header rather than panicking.
fn section_and_leaf(path: &str) -> (&str, &str) {
    match path.rfind('.') {
        Some(idx) => (&path[..idx], &path[idx + 1..]),
        None => ("", path),
    }
}

/// Render a key's value cell: the quoted/bare default, or an
/// angle-bracket placeholder by kind when there is no default —
/// except a required key with no default, which renders
/// `<required>` so it stands out.
fn render_value(key: &ConfigKey) -> String {
    match key.default {
        Some(d) => match key.kind {
            ValueKind::Usize => d.to_string(),
            ValueKind::Str | ValueKind::ItemList => format!("{d:?}"),
        },
        None => {
            if key.required {
                "<required>".to_string()
            } else {
                match key.kind {
                    ValueKind::Str => "<str>".to_string(),
                    ValueKind::Usize => "<usize>".to_string(),
                    ValueKind::ItemList => "<items>".to_string(),
                }
            }
        }
    }
}

/// Print one home group: a divider, then each key grouped by
/// section (schema order, first-seen section order), one
/// `[section]` header per section.
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
            let (key_section, leaf) = section_and_leaf(key.path);
            if key_section != section {
                continue;
            }
            let prefix = if key.required { "" } else { "# " };
            let value = render_value(key);
            info!("{prefix}{leaf} = {value}   # {}", key.doc);
        }
    }
    info!("");
}

/// Print the settable config schema, grouped by config home then
/// TOML section. `--home` selects which home group(s) to print.
pub fn config(_ctx: &Context, params: &ConfigParams) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "# vc-x1 settable config keys (from vc-x1 {})",
        env!("CARGO_PKG_VERSION")
    );
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

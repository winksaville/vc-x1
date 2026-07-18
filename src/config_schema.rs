//! Single source of truth for every settable config key across
//! vc-x1's two config homes (the user config file and the
//! per-repo workspace config).
//!
//! - `crate::config_cmd` prints this registry as an annotated
//!   schema, and — with `--validate` — checks a config file's keys
//!   against it (dynamic-segment aware).
//! - `crate::init` derives init's commented defaults from
//!   `schema()`, so these surfaces cannot drift from this list.

/// Which config home a key belongs to.
/// - `User` — `~/.config/vc-x1/config.toml`
/// - `WorkspaceCode` / `WorkspaceBot` — `<root>/.vc-config.toml`
///   in the work repo vs the `.claude` bot repo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Home {
    User,
    WorkspaceCode,
    WorkspaceBot,
}

/// The value shape a key expects (for rendering / future typed
/// use).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Str,
    Usize,
    ItemList,
}

/// One settable configuration key.
pub struct ConfigKey {
    /// Dotted TOML path, e.g. `"bot-session.col-width"`.
    pub path: &'static str,
    /// Which home(s) accept this key.
    pub homes: &'static [Home],
    /// The value shape this key expects.
    pub kind: ValueKind,
    /// Rendered default, `None` if there is no default.
    pub default: Option<&'static str>,
    /// Active (not commented) in init; the value is role-specific.
    pub required: bool,
    /// The path has a `<placeholder>` segment matching a family
    /// of keys (e.g. `repo.category.<cat>`).
    pub dynamic: bool,
    /// One-line description.
    pub doc: &'static str,
}

/// Homes accepted by every `bot-session.*` key: all three homes,
/// since the user config and both workspace configs may each
/// carry a default.
const BOT_SESSION_HOMES: &[Home] = &[Home::User, Home::WorkspaceCode, Home::WorkspaceBot];

/// Homes accepted by workspace-only keys (`workspace.*`,
/// `push.*`): the two `.vc-config.toml` homes, never the user
/// config.
const WORKSPACE_HOMES: &[Home] = &[Home::WorkspaceCode, Home::WorkspaceBot];

/// The complete registry of settable keys.
///
/// - User home: `default.*`, `repo.*`, and the per-account
///   `account.<name>.repo.*` family.
/// - `bot-session.*`: accepted in all three homes.
/// - Workspace-only: `workspace.*`, `push.*`.
const SCHEMA: &[ConfigKey] = &[
    ConfigKey {
        path: "default.account",
        homes: &[Home::User],
        kind: ValueKind::Str,
        default: None,
        required: false,
        dynamic: false,
        doc: "Default account when --account is absent",
    },
    ConfigKey {
        path: "default.debug",
        homes: &[Home::User],
        kind: ValueKind::Str,
        default: None,
        required: false,
        dynamic: false,
        doc: "Default --debug value when used without an argument (reserved; not yet consumed)",
    },
    ConfigKey {
        path: "repo.default",
        homes: &[Home::User],
        kind: ValueKind::Str,
        default: None,
        required: false,
        dynamic: false,
        doc: "Top-level shorthand: repo category to use when --repo is absent",
    },
    ConfigKey {
        path: "repo.category.<cat>",
        homes: &[Home::User],
        kind: ValueKind::Str,
        default: None,
        required: false,
        dynamic: true,
        doc: "Top-level shorthand: literal value for repo category <cat>",
    },
    ConfigKey {
        path: "account.<name>.repo.default",
        homes: &[Home::User],
        kind: ValueKind::Str,
        default: None,
        required: false,
        dynamic: true,
        doc: "Per-account default repo category",
    },
    ConfigKey {
        path: "account.<name>.repo.category.<cat>",
        homes: &[Home::User],
        kind: ValueKind::Str,
        default: None,
        required: false,
        dynamic: true,
        doc: "Per-account literal value for repo category <cat>",
    },
    ConfigKey {
        path: "bot-session.items",
        homes: BOT_SESSION_HOMES,
        kind: ValueKind::ItemList,
        default: Some("headers,user,assistant,tool,summary"),
        required: false,
        dynamic: false,
        doc: "Default bot-session item set (comma-separated)",
    },
    ConfigKey {
        path: "bot-session.result-lines",
        homes: BOT_SESSION_HOMES,
        kind: ValueKind::Usize,
        default: Some("10"),
        required: false,
        dynamic: false,
        doc: "Default --result-lines: max lines shown per tool result (0 = unlimited)",
    },
    ConfigKey {
        path: "bot-session.col-width",
        homes: BOT_SESSION_HOMES,
        kind: ValueKind::Usize,
        default: Some("68"),
        required: false,
        dynamic: false,
        doc: "Default --col-width: first-column width in the field-inventory views",
    },
    ConfigKey {
        path: "workspace.path",
        homes: WORKSPACE_HOMES,
        kind: ValueKind::Str,
        default: None,
        required: true,
        dynamic: false,
        doc: "This repo's path relative to the workspace root (role-specific: \"/\" for the work repo, \"/.claude\" for the bot repo)",
    },
    ConfigKey {
        path: "workspace.other-repo",
        homes: WORKSPACE_HOMES,
        kind: ValueKind::Str,
        default: None,
        required: false,
        dynamic: false,
        doc: "Relative path to the counterpart repo; presence signals dual-repo mode (role-specific: \".claude\" for the work repo, \"..\" for the bot repo)",
    },
    ConfigKey {
        path: "push.state-dir",
        homes: WORKSPACE_HOMES,
        kind: ValueKind::Str,
        default: Some(".vc-x1"),
        required: false,
        dynamic: false,
        doc: "Directory (relative to repo root) holding the push state file",
    },
    ConfigKey {
        path: "push.state-file",
        homes: WORKSPACE_HOMES,
        kind: ValueKind::Str,
        default: Some("push-state.toml"),
        required: false,
        dynamic: false,
        doc: "Filename of the push state file under push.state-dir",
    },
];

/// Returns the complete registry of settable config keys.
pub fn schema() -> &'static [ConfigKey] {
    SCHEMA
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Finds a schema entry by its dotted path.
    ///
    /// - Panics if not found: tests only, and a missing key is a
    ///   test bug worth failing loudly on.
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    fn find(path: &str) -> &'static ConfigKey {
        schema()
            .iter()
            .find(|k| k.path == path)
            .unwrap_or_else(|| panic!("schema key {path:?} not found"))
    }

    #[test]
    fn defaults_match_source_consts() {
        assert_eq!(
            find("bot-session.col-width").default,
            Some(crate::bot_session::COL_WIDTH.to_string()).as_deref()
        );
        assert_eq!(
            find("bot-session.result-lines").default,
            Some(crate::bot_session::RESULT_LINE_CAP.to_string()).as_deref()
        );
        assert_eq!(
            find("push.state-dir").default,
            Some(crate::push::DEFAULT_STATE_DIR)
        );
        assert_eq!(
            find("push.state-file").default,
            Some(crate::push::DEFAULT_STATE_FILE)
        );
    }

    #[test]
    fn paths_unique() {
        let mut paths: Vec<&str> = schema().iter().map(|k| k.path).collect();
        paths.sort_unstable();
        let mut deduped = paths.clone();
        deduped.dedup();
        assert_eq!(paths.len(), deduped.len(), "duplicate path(s) in schema()");
    }

    #[test]
    fn homes_non_empty() {
        for key in schema() {
            assert!(!key.homes.is_empty(), "{} has no homes", key.path);
        }
    }

    #[test]
    fn dynamic_keys_have_placeholder() {
        for key in schema() {
            let has_placeholder = key.path.contains('<');
            assert_eq!(
                key.dynamic, has_placeholder,
                "{}: dynamic={} but placeholder-presence={}",
                key.path, key.dynamic, has_placeholder
            );
        }
    }
}

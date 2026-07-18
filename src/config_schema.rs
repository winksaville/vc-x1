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
    /// The command/flag or structural context this key is
    /// associated with (e.g. `"bot-session --col-width"`).
    pub used_by: &'static str,
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
        used_by: "--account (init and account-aware commands)",
    },
    ConfigKey {
        path: "default.debug",
        homes: &[Home::User],
        kind: ValueKind::Str,
        default: None,
        required: false,
        dynamic: false,
        doc: "Default --debug value when used without an argument (reserved; not yet consumed)",
        used_by: "--debug (reserved; not yet consumed)",
    },
    ConfigKey {
        path: "repo.default",
        homes: &[Home::User],
        kind: ValueKind::Str,
        default: None,
        required: false,
        dynamic: false,
        doc: "Top-level shorthand: repo category to use when --repo is absent",
        used_by: "--repo (default category when --repo is bare)",
    },
    ConfigKey {
        path: "repo.category.<cat>",
        homes: &[Home::User],
        kind: ValueKind::Str,
        default: None,
        required: false,
        dynamic: true,
        doc: "Top-level shorthand: literal value for repo category <cat>",
        used_by: "--repo <cat> (init remote/local resolution)",
    },
    ConfigKey {
        path: "account.<name>.repo.default",
        homes: &[Home::User],
        kind: ValueKind::Str,
        default: None,
        required: false,
        dynamic: true,
        doc: "Per-account default repo category",
        used_by: "--account <name> with --repo",
    },
    ConfigKey {
        path: "account.<name>.repo.category.<cat>",
        homes: &[Home::User],
        kind: ValueKind::Str,
        default: None,
        required: false,
        dynamic: true,
        doc: "Per-account literal value for repo category <cat>",
        used_by: "--account <name> with --repo <cat>",
    },
    ConfigKey {
        path: "bot-session.items",
        homes: BOT_SESSION_HOMES,
        kind: ValueKind::ItemList,
        default: Some("headers,user,assistant,tool,summary"),
        required: false,
        dynamic: false,
        doc: "Default bot-session item set (comma-separated)",
        used_by: "bot-session --<item> / --no-<item> / --all / --none",
    },
    ConfigKey {
        path: "bot-session.result-lines",
        homes: BOT_SESSION_HOMES,
        kind: ValueKind::Usize,
        default: Some("10"),
        required: false,
        dynamic: false,
        doc: "Default --result-lines: max lines shown per tool result (0 = unlimited)",
        used_by: "bot-session --result-lines",
    },
    ConfigKey {
        path: "bot-session.col-width",
        homes: BOT_SESSION_HOMES,
        kind: ValueKind::Usize,
        default: Some("68"),
        required: false,
        dynamic: false,
        doc: "Default --col-width: first-column width in the field-inventory views",
        used_by: "bot-session --col-width",
    },
    ConfigKey {
        path: "workspace.path",
        homes: WORKSPACE_HOMES,
        kind: ValueKind::Str,
        default: None,
        required: true,
        dynamic: false,
        doc: "This repo's path relative to the workspace root (role-specific: \"/\" for the work repo, \"/.claude\" for the bot repo)",
        used_by: "find_workspace_root, sync, push, validate-desc (structural; written by init)",
    },
    ConfigKey {
        path: "workspace.other-repo",
        homes: WORKSPACE_HOMES,
        kind: ValueKind::Str,
        default: None,
        required: false,
        dynamic: false,
        doc: "Relative path to the counterpart repo; presence signals dual-repo mode (role-specific: \".claude\" for the work repo, \"..\" for the bot repo)",
        used_by: "default_scope, validate-desc/fix-desc --other-repo (structural)",
    },
    ConfigKey {
        path: "push.state-dir",
        homes: WORKSPACE_HOMES,
        kind: ValueKind::Str,
        default: Some(".vc-x1"),
        required: false,
        dynamic: false,
        doc: "Directory (relative to repo root) holding the push state file",
        used_by: "push / squash-push (state-file directory)",
    },
    ConfigKey {
        path: "push.state-file",
        homes: WORKSPACE_HOMES,
        kind: ValueKind::Str,
        default: Some("push-state.toml"),
        required: false,
        dynamic: false,
        doc: "Filename of the push state file under push.state-dir",
        used_by: "push / squash-push (state-file name)",
    },
];

/// Returns the complete registry of settable config keys.
pub fn schema() -> &'static [ConfigKey] {
    SCHEMA
}

/// Split a dotted key path on its last `.` into `(section, leaf)`.
///
/// A path with no `.` (none exist in the current schema) falls
/// back to an empty section so the key still renders under a
/// blank `[]` header rather than panicking.
pub fn section_and_leaf(path: &str) -> (&str, &str) {
    match path.rfind('.') {
        Some(idx) => (&path[..idx], &path[idx + 1..]),
        None => ("", path),
    }
}

/// Render a key's value cell: the quoted/bare default, or an
/// angle-bracket placeholder by kind when there is no default —
/// except a required key with no default, which renders
/// `<required>` so it stands out.
pub fn render_value(key: &ConfigKey) -> String {
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

/// Render a key's `default:` note: the quoted/bare default, or a
/// parenthetical explaining the absence — `(required; ...)` for a
/// required key with no default (role-specific, filled by init),
/// `(none)` otherwise.
fn render_default_note(key: &ConfigKey) -> String {
    match key.default {
        Some(d) => match key.kind {
            ValueKind::Usize => d.to_string(),
            ValueKind::Str | ValueKind::ItemList => format!("{d:?}"),
        },
        None => {
            if key.required {
                "(required; role-specific — see init)".to_string()
            } else {
                "(none)".to_string()
            }
        }
    }
}

/// Word-wrap `text` into `#`-prefixed lines, `first_prefix` on the
/// first line and `cont_prefix` on continuations, each kept to
/// `width` columns where possible (a single word longer than
/// `width` still gets its own line, unsplit).
fn wrap_hash_comment(text: &str, first_prefix: &str, cont_prefix: &str, width: usize) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let prefix_len = if lines.is_empty() {
            first_prefix.len()
        } else {
            cont_prefix.len()
        };
        let sep = usize::from(!current.is_empty());
        let tentative_len = prefix_len + current.len() + sep + word.len();
        if !current.is_empty() && tentative_len > width {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    let mut out = String::new();
    for (i, l) in lines.iter().enumerate() {
        let prefix = if i == 0 { first_prefix } else { cont_prefix };
        out.push_str(prefix);
        out.push_str(l);
        out.push('\n');
    }
    out
}

/// Render one key as a thorough, self-documenting doc-block:
/// - `# <path> — <doc>` (word-wrapped onto `#   ...`
///   continuations past ~72 cols),
/// - `#   used by: <used_by>`,
/// - `#   default: <rendered default, or a "(none)"/"(required...)"
///   note>`,
/// - the assignment line itself — uncommented for a `required` key
///   (init fills in the role-specific value), commented (`# `)
///   otherwise, with the rendered default or a kind placeholder
///   (`<str>` etc.) when there is no default,
/// - a trailing blank line, so consecutive blocks read as
///   paragraph-separated entries.
///
/// Shared by `crate::config_cmd` (printed schema) and `crate::init`
/// (commented defaults in the generated `.vc-config.toml`) so the
/// two surfaces cannot drift from each other's wording.
pub fn render_key_block(key: &ConfigKey) -> String {
    let mut out = String::new();
    let header_text = format!("{} — {}", key.path, key.doc);
    out.push_str(&wrap_hash_comment(&header_text, "# ", "#   ", 72));
    out.push_str(&format!("#   used by: {}\n", key.used_by));
    out.push_str(&format!("#   default: {}\n", render_default_note(key)));
    let (_section, leaf) = section_and_leaf(key.path);
    let prefix = if key.required { "" } else { "# " };
    let value = render_value(key);
    out.push_str(&format!("{prefix}{leaf} = {value}\n"));
    out.push('\n');
    out
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

//! The settable-config-key registry, generated from the
//! `vc-config.md` prototype at the repo root: build.rs parses
//! the prototype and renders the `SCHEMA_GEN` table plus the
//! `<PATH>_DEFAULT` constants included below, so the registry,
//! the behavioral defaults, and the prototype cannot disagree.
//!
//! - `crate::config_cmd` prints this registry as an annotated
//!   schema, and (with `--validate`) checks a config file's keys
//!   against it (dynamic-segment aware).
//! - `crate::init` derives init's commented defaults from
//!   `schema()`, so these surfaces cannot drift from this list.
//! - Behavioral defaults (e.g. `agent-session --col-width`) consume
//!   the generated constants rather than hand-kept copies.

/// Which config home a key belongs to.
/// - `User`: `~/.config/vc-x1/config.toml`
/// - `WorkspaceCode` / `WorkspaceBot`: `<root>/.vc-config.toml`
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
    /// A comma-separated list inside one string.
    ItemList,
    /// A TOML array of strings, one element per item (the
    /// `[validate]` command lists, each element one invocation).
    StrList,
}

/// One settable configuration key.
pub struct ConfigKey {
    /// The config key (TOML table and key joined by `.`), e.g.
    /// `"agent-session.col-width"`.
    pub path: &'static str,
    /// Which home(s) accept this key.
    pub homes: &'static [Home],
    /// The value shape this key expects.
    pub kind: ValueKind,
    /// Rendered default, `None` if there is no default.
    pub default: Option<&'static str>,
    /// Active (not commented) in init. The value is role-specific.
    pub required: bool,
    /// The path has a `<placeholder>` segment matching a family
    /// of keys (e.g. `repo.category.<cat>`).
    pub dynamic: bool,
    /// One-line description.
    pub doc: &'static str,
    /// The command/flag or structural context this key is
    /// associated with (e.g. `"agent-session --col-width"`).
    pub used_by: &'static str,
    /// A representative example value for keys with no default,
    /// and rendered on the assignment line, marked `# example`,
    /// instead of a bare placeholder. `None` when the key has a
    /// real default (the default serves as the example).
    pub example: Option<&'static str>,
    /// Docs url for the key, rendered as a `reference:` line so a
    /// reader can click through from a generated config.
    pub reference: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/config_schema_gen.rs"));

/// Returns the complete registry of settable config keys.
pub fn schema() -> &'static [ConfigKey] {
    SCHEMA_GEN
}

/// Split a config key on its last `.` into `(section, leaf)`.
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

/// Format a raw value string per `kind`, the way a default or an
/// example is rendered on the assignment line: `Usize` bare,
/// `Str`/`ItemList` quoted.
fn format_value(kind: ValueKind, raw: &str) -> String {
    match kind {
        ValueKind::Usize => raw.to_string(),
        ValueKind::Str | ValueKind::ItemList => format!("{raw:?}"),
        ValueKind::StrList => raw.to_string(),
    }
}

/// Render a key's value cell: the quoted/bare default, or an
/// angle-bracket placeholder by kind when there is no default,
/// except a required key with no default, which renders
/// `<required>` so it stands out.
pub fn render_value(key: &ConfigKey) -> String {
    match key.default {
        Some(d) => format_value(key.kind, d),
        None => {
            if key.required {
                "<required>".to_string()
            } else {
                match key.kind {
                    ValueKind::Str => "<str>".to_string(),
                    ValueKind::Usize => "<usize>".to_string(),
                    ValueKind::ItemList => "<items>".to_string(),
                    ValueKind::StrList => "<list>".to_string(),
                }
            }
        }
    }
}

/// Render a key's `default:` note: the quoted/bare default, or a
/// parenthetical explaining the absence: `(required; ...)` for a
/// required key with no default (role-specific, filled by init),
/// `(none)` otherwise.
fn render_default_note(key: &ConfigKey) -> String {
    match key.default {
        Some(d) => format_value(key.kind, d),
        None => {
            if key.required {
                "(required, role-specific, see init)".to_string()
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
/// - `# <path>: <doc>` (word-wrapped onto `#   ...`
///   continuations past ~100 cols, the project's prose width),
/// - `#   used by: <used_by>`,
/// - `#   default: <rendered default, or a "(none)"/"(required...)"
///   note>`,
/// - `#   reference: <docs url>`,
/// - the assignment line itself: uncommented for a `required` key
///   (init fills in the role-specific value), commented (`# `)
///   otherwise, with the rendered default, a `key.example` value
///   marked with a trailing `# example` when there is no default,
///   or (absent both) a kind placeholder (`<str>` etc.),
/// - a trailing blank line, so consecutive blocks read as
///   paragraph-separated entries.
///
/// Shared by `crate::config_cmd` (printed schema) and `crate::init`
/// (commented defaults in the generated `.vc-config.toml`) so the
/// two surfaces cannot drift from each other's wording.
pub fn render_key_block(key: &ConfigKey) -> String {
    let mut out = String::new();
    let header_text = format!("{}: {}", key.path, key.doc);
    out.push_str(&wrap_hash_comment(&header_text, "# ", "#   ", 100));
    out.push_str(&format!("#   used by: {}\n", key.used_by));
    out.push_str(&format!("#   default: {}\n", render_default_note(key)));
    out.push_str(&format!("#   reference: {}\n", key.reference));
    let (_section, leaf) = section_and_leaf(key.path);
    let prefix = if key.required { "" } else { "# " };
    match key.example {
        Some(ex) => {
            let value = format_value(key.kind, ex);
            out.push_str(&format!("{prefix}{leaf} = {value}   # example\n"));
        }
        None => {
            let value = render_value(key);
            out.push_str(&format!("{prefix}{leaf} = {value}\n"));
        }
    }
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Finds a schema entry by its config key.
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

    /// The generated constants and the generated table both come
    /// from the prototype, so equality here is a codegen
    /// self-check, not a drift check: it fails only if the two
    /// render paths in build.rs diverge.
    #[test]
    fn generated_consts_match_table() {
        assert_eq!(
            find("agent-session.col-width").default,
            Some(AGENT_SESSION_COL_WIDTH_DEFAULT.to_string()).as_deref()
        );
        assert_eq!(
            find("agent-session.result-lines").default,
            Some(AGENT_SESSION_RESULT_LINES_DEFAULT.to_string()).as_deref()
        );
        assert_eq!(
            find("agent-session.items").default,
            Some(AGENT_SESSION_ITEMS_DEFAULT)
        );
    }

    /// Every key must point a reader somewhere: a non-empty
    /// `reference` url starting with https.
    #[test]
    fn every_key_has_reference() {
        for key in schema() {
            assert!(
                key.reference.starts_with("https://"),
                "{}: reference {:?} is not an https url",
                key.path,
                key.reference
            );
        }
    }

    /// Every derived reference must anchor at a real `##` heading
    /// in the per-key doc: build.rs derives the fragment from the
    /// key's path, and this re-derives the headings' GitHub slugs
    /// (lowercase, keep `[a-z0-9-]`, spaces to hyphens, markdown
    /// escapes dropped) and demands a match, so a key added to
    /// the prototype without docs fails here. Override references
    /// pointing elsewhere are skipped. Doc and schema share one
    /// file, so this reads the prototype itself: it is checking
    /// that each key's fence sits under a heading the derived url
    /// names, not that two files agree.
    #[test]
    #[allow(clippy::unwrap_used)]
    fn references_anchor_into_vc_config_md() {
        let md =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/vc-config.md")).unwrap();
        let slugs: std::collections::HashSet<String> = md
            .lines()
            .filter_map(|l| l.strip_prefix("## "))
            .map(|h| {
                h.chars()
                    .filter_map(|c| match c {
                        'a'..='z' | '0'..='9' | '-' => Some(c),
                        'A'..='Z' => Some(c.to_ascii_lowercase()),
                        ' ' => Some('-'),
                        _ => None,
                    })
                    .collect()
            })
            .collect();
        for key in schema() {
            let Some((_, fragment)) = key.reference.rsplit_once("/vc-config.md#") else {
                continue;
            };
            assert!(
                slugs.contains(fragment),
                "{}: no vc-config.md `##` heading slugs to #{fragment}",
                key.path
            );
        }
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

    /// Every key must show a concrete value in the rendered
    /// schema: a real `default`, or an `example`, never both
    /// `None` (which would render a bare `<str>` placeholder).
    #[test]
    fn every_key_has_default_or_example() {
        for key in schema() {
            assert!(
                key.default.is_some() || key.example.is_some(),
                "{}: has neither a default nor an example",
                key.path
            );
        }
    }

    /// The family and validate tables are work-side only, the
    /// command lists typed `str-list` with array examples.
    #[test]
    fn family_and_validate_tables() {
        let find = |p: &str| schema().iter().find(|k| k.path == p).expect(p);
        for p in ["family.member", "family.template", "family.messages"] {
            let k = find(p);
            assert_eq!(k.homes, &[Home::WorkspaceCode], "{p} homes");
            assert_eq!(k.kind, ValueKind::Str, "{p} kind");
        }
        for p in ["validate.full", "validate.fast"] {
            let k = find(p);
            assert_eq!(k.homes, &[Home::WorkspaceCode], "{p} homes");
            assert_eq!(k.kind, ValueKind::StrList, "{p} kind");
            let ex = k.example.expect("example");
            assert!(
                ex.starts_with('[') && ex.ends_with(']'),
                "{p} example: {ex}"
            );
            assert_eq!(
                format_value(k.kind, ex),
                ex,
                "{p} renders its array verbatim"
            );
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

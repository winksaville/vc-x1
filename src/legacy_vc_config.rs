//! Backward compatibility for pre-`[repos]` `.vc-config.toml`
//! schemas: the whole legacy surface lives here so it is easy to
//! find and, eventually, remove.
//!
//! Two rejected generations, both under a `[workspace]` section:
//!
//! - pre-0.75.0: `path` (root-anchored) + `other-repo` (bare name).
//! - 0.75.x: root-anchored `work`/`bot` (leading-`/`-means-root).
//!
//! The resolvers hard-reject both via [`reject`]'s fix-it message.
//! The other functions keep read-only bootstrap surfaces (clone,
//! symlink, validate-desc / fix-desc explicit paths) working on
//! unmigrated repos.
//!
//! **Retirement plan:** once every workspace is migrated to the
//! `[repos]` registry, delete this module and simplify its call
//! sites: `grep -rn 'legacy_vc_config::' src/` lists them all.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::desc_helpers::VC_CONFIG_FILE;
use crate::toml_simple;

/// True when `cfg` is a legacy-schema config: no `repos.work`,
/// but any legacy `[workspace]` key present.
pub fn is_legacy_config(cfg: &HashMap<String, String>) -> bool {
    toml_simple::toml_get(cfg, "repos.work").is_none()
        && (toml_simple::toml_get(cfg, "workspace.work").is_some()
            || toml_simple::toml_get(cfg, "workspace.path").is_some()
            || toml_simple::toml_get(cfg, "workspace.other-repo").is_some())
}

/// True when `cfg` marks a legacy workspace *root* for the
/// upward walk: the 0.75.x work side (`workspace.work` present)
/// or the pre-0.75.0 work side (`path = "/"`).
pub fn is_root_marker(cfg: &HashMap<String, String>) -> bool {
    toml_simple::toml_get(cfg, "workspace.work").is_some()
        || toml_simple::toml_get(cfg, "workspace.path").map(String::as_str) == Some("/")
}

/// True when `dir` is a *legacy* workspace's bot repo.
///
/// The legacy schemas detect side by location: `dir` is the bot
/// side iff the parent directory's `.vc-config.toml` names `dir`
/// as the workspace's bot: 0.75.x `workspace.bot`
/// (root-anchored) or pre-0.75.0 `workspace.other-repo` (bare
/// name). Used by the root walk to point the fix-it at the
/// work-side root, and as `common::is_bot_dir`'s
/// backward-compat fallback.
pub fn is_bot_dir(dir: &Path) -> bool {
    let Ok(dir) = dir.canonicalize() else {
        return false;
    };
    let Some(parent) = dir.parent() else {
        return false;
    };
    let Ok(cfg) = toml_simple::toml_load(&parent.join(VC_CONFIG_FILE)) else {
        return false;
    };
    match toml_simple::toml_get(&cfg, "workspace.bot")
        .or_else(|| toml_simple::toml_get(&cfg, "workspace.other-repo"))
    {
        Some(bot) if !bot.is_empty() => parent.join(bot.trim_start_matches('/')) == dir,
        _ => false,
    }
}

/// The bot dir a legacy config declares: the backward-compat
/// read for bootstrap paths (clone, the symlink default).
///
/// The resolvers hard-reject legacy schemas with a fix-it (see
/// [`reject`]), but clone is where an old repo first arrives
/// locally: it must complete correctly before the user can
/// apply the rewrite. Reads the rejected generations' keys and
/// resolves against `root`:
///
/// - 0.75.x `workspace.bot`: root-anchored (`"/.claude"`).
/// - pre-0.75.0 `workspace.other-repo`: a bare dir name.
///
/// `None` when the config is missing, already `[repos]`-schema,
/// or declares no bot side.
pub fn configured_bot_dir(root: &Path) -> Option<PathBuf> {
    let cfg = toml_simple::toml_load(&root.join(VC_CONFIG_FILE)).ok()?;
    if toml_simple::toml_get(&cfg, "repos.work").is_some() {
        return None;
    }
    let bot = toml_simple::toml_get(&cfg, "workspace.bot")
        .or_else(|| toml_simple::toml_get(&cfg, "workspace.other-repo"))
        .filter(|v| !v.is_empty())?;
    Some(root.join(bot.trim_start_matches('/')))
}

/// The old spellings of the agent-side keys, each paired with the
/// key that replaced it (the 0.80.0 agent naming: `repos.agent`,
/// `[agent-session]`).
const OLD_AGENT_KEYS: &[(&str, &str)] = &[
    ("repos.bot", "repos.agent"),
    ("bot-session.items", "agent-session.items"),
    ("bot-session.result-lines", "agent-session.result-lines"),
    ("bot-session.col-width", "agent-session.col-width"),
];

/// The fix-it rejection for a `[repos]`-schema config still
/// carrying the pre-0.80.0 `bot` spellings, or `None` when it
/// carries none.
///
/// A rejection rather than an alias (wink, 2026-08-12): an alias is
/// a live dual-name path that stays temporary only if someone later
/// deletes it, while a rejection is permanent and harmless, and the
/// fix-it makes the flag day a five-second edit. Lists every old key
/// found so one edit clears them all.
pub fn reject_old_agent_keys(path: &Path, cfg: &HashMap<String, String>) -> Option<String> {
    let found: Vec<String> = OLD_AGENT_KEYS
        .iter()
        .filter(|(old, _)| toml_simple::toml_get(cfg, old).is_some())
        .map(|(old, new)| format!("  {old} -> {new}"))
        .collect();
    if found.is_empty() {
        return None;
    }
    Some(format!(
        "{}: pre-0.80.0 key spelling(s). The agent side is `agent`, \
         not `bot`, in the config and on the CLI. Rename, same value, \
         on both sides' config files:\n\n{}\n\n\
         (`--scope=bot` is `--scope=agent`, and the `bot-session` \
         subcommand is `agent-session`)",
        path.display(),
        found.join("\n")
    ))
}

/// The fix-it rejection for a legacy config, or `None` when `cfg`
/// isn't legacy.
///
/// The rewrite example echoes the workspace's actual bot dir name
/// when the legacy config declares one.
pub fn reject(dir: &Path, cfg: &HashMap<String, String>) -> Option<String> {
    if !is_legacy_config(cfg) {
        return None;
    }
    let bot_name = configured_bot_dir(dir)
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| ".claude".to_string()); // OK: default dir name for the rewrite example
    Some(format!(
        "{}/.vc-config.toml: legacy [workspace] schema. Replace it with a \
         [repos] registry: paths are relative to each config file's \
         directory, so the sides differ:\n\n\
         work side:\n\
         [repos]\n\
         work = \".\"\n\
         agent = \"{bot_name}\"\n\n\
         bot side:\n\
         [repos]\n\
         work = \"..\"\n\
         agent = \".\"\n\n\
         (drop `agent` for a single-repo workspace)",
        dir.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unique tempdir for the legacy tests.
    fn tempdir(tag: &str) -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("vc-x1-legacy-{tag}-{ts}"));
        std::fs::create_dir_all(&dir).expect("mkdir tempdir");
        dir
    }

    /// Both generations' parent configs name the bot side. The
    /// work side is never the bot side.
    #[test]
    fn is_bot_dir_both_generations() {
        for (tag, block) in [
            ("075x", "[workspace]\nwork = \"/\"\nbot = \"/.claude\"\n"),
            (
                "pre075",
                "[workspace]\npath = \"/\"\nother-repo = \".claude\"\n",
            ),
        ] {
            let base = tempdir(tag);
            let root = base.join("ws");
            let bot = root.join(".claude");
            std::fs::create_dir_all(&bot).unwrap();
            std::fs::write(root.join(VC_CONFIG_FILE), block).unwrap();
            assert!(is_bot_dir(&bot), "generation: {tag}");
            assert!(!is_bot_dir(&root), "generation: {tag}");
            std::fs::remove_dir_all(&base).ok();
        }
    }

    /// `configured_bot_dir` reads both generations' declarations,
    /// and stays `None` for the `[repos]` schema and absent
    /// configs.
    #[test]
    fn configured_bot_dir_reads_old_schemas() {
        let base = tempdir("botdir");
        let root = base.join("ws");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join(VC_CONFIG_FILE),
            "[workspace]\nwork = \"/\"\nbot = \"/.claude\"\n",
        )
        .unwrap();
        assert_eq!(configured_bot_dir(&root), Some(root.join(".claude")));
        std::fs::write(
            root.join(VC_CONFIG_FILE),
            "[workspace]\npath = \"/\"\nother-repo = \".bot\"\n",
        )
        .unwrap();
        assert_eq!(configured_bot_dir(&root), Some(root.join(".bot")));
        std::fs::write(
            root.join(VC_CONFIG_FILE),
            "[repos]\nwork = \".\"\nagent = \".claude\"\n",
        )
        .unwrap();
        assert_eq!(configured_bot_dir(&root), None);
        std::fs::remove_file(root.join(VC_CONFIG_FILE)).unwrap();
        assert_eq!(configured_bot_dir(&root), None);
        std::fs::remove_dir_all(&base).ok();
    }
}

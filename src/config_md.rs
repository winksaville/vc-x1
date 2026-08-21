//! Markdown-carried instance config: the resolver/loader every
//! `.vc-config.*` reader goes through.
//!
//! The format: a config file is a markdown document whose `toml`
//! fences, concatenated in document order, form the TOML the loader
//! parses. Prose between fences is documentation (doc one-liners,
//! reference links) and never reaches the parser. One rule falls
//! out for authors: a `[table]` header captures every key after it
//! until the next header, so a table's keys must stay in its
//! stretch of the document.
//!
//! The filter itself is [`crate::md_fence`], which build.rs shares
//! to parse the `vc-config.md` schema prototype in the same format.
//!
//! `.vc-config.toml` remains a valid carrier through the family's
//! migration window, and a side holding both carriers is an error.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::desc_helpers::VC_CONFIG_FILE;
use crate::md_fence::md_to_toml;
use crate::toml_simple;

/// The markdown instance-config filename (the toml carrier's name
/// is [`VC_CONFIG_FILE`]).
pub const VC_CONFIG_MD: &str = ".vc-config.md";

/// Resolve which instance-config file `dir` holds.
///
/// `Ok(None)` when neither carrier exists. Holding both is an
/// error: two files that can disagree are no longer a config.
pub fn vc_config_path(dir: &Path) -> Result<Option<PathBuf>, String> {
    let md = dir.join(VC_CONFIG_MD);
    let toml = dir.join(VC_CONFIG_FILE);
    match (md.exists(), toml.exists()) {
        (true, true) => Err(format!(
            "{}: both {VC_CONFIG_MD} and {VC_CONFIG_FILE} exist: keep one \
             ({VC_CONFIG_MD} is the current carrier); nothing was changed",
            dir.display()
        )),
        (true, false) => Ok(Some(md)),
        (false, true) => Ok(Some(toml)),
        (false, false) => Ok(None),
    }
}

/// Load one config file, whatever its carrier: a `.md` path runs
/// through [`md_to_toml`] first, anything else parses as plain
/// TOML.
pub fn load_file(path: &Path) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    if path.extension().is_some_and(|e| e == "md") {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read '{}': {e}", path.display()))?;
        let toml = md_to_toml(&content).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(toml_simple::toml_parse(&toml))
    } else {
        toml_simple::toml_load(path)
    }
}

/// A loaded instance config: the file it came from (for messages)
/// and its flat key map.
pub struct VcConfig {
    pub path: PathBuf,
    pub map: HashMap<String, String>,
}

/// Resolve and load `dir`'s instance config.
///
/// `Ok(None)` when the directory has no config file. An error is a
/// real problem (both carriers present, unreadable file, a bad
/// fence), never a plain miss.
pub fn load(dir: &Path) -> Result<Option<VcConfig>, Box<dyn std::error::Error>> {
    let Some(path) = vc_config_path(dir)? else {
        return Ok(None);
    };
    let map = load_file(&path)?;
    Ok(Some(VcConfig { path, map }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The model rendering (vc-config-test.md's shape): compact,
    /// one fence per table, doc-link bullets above it.
    const COMPACT: &str = "\
# vc-x1 config file

vc-x1 config file document [0]

The two repos
- work doc[[4]]
- agent docs[[5]]
```toml
[repos]
work = \".\"
agent = \".claude\"
```

The agent-session table
- items [[1]]
```toml
[agent-session]
items = \"headers,user\"
col-width = 68
```

[0]: ./vc-config.md#vc-config-settable-configuration-keys
[1]: ./vc-config.md#agent-sessionitems
";

    /// The separated per-key shape: a lone header fence, then bare
    /// key fences relying on its scope.
    const SEPARATED: &str = "\
The agent-session table
```toml
[agent-session]
```

items
```toml
items = \"headers,user\"
```

col-width
```toml
col-width = 68
```
";

    #[test]
    fn compact_shape_parses() {
        let toml = md_to_toml(COMPACT).unwrap();
        let map = toml_simple::toml_parse(&toml);
        assert_eq!(map.get("repos.work").map(String::as_str), Some("."));
        assert_eq!(map.get("repos.agent").map(String::as_str), Some(".claude"));
        assert_eq!(
            map.get("agent-session.items").map(String::as_str),
            Some("headers,user")
        );
        assert_eq!(
            map.get("agent-session.col-width").map(String::as_str),
            Some("68")
        );
    }

    #[test]
    fn separated_shape_parses_like_compact() {
        let toml = md_to_toml(SEPARATED).unwrap();
        let map = toml_simple::toml_parse(&toml);
        assert_eq!(
            map.get("agent-session.items").map(String::as_str),
            Some("headers,user")
        );
        assert_eq!(
            map.get("agent-session.col-width").map(String::as_str),
            Some("68")
        );
    }

    #[test]
    fn line_count_is_preserved() {
        let toml = md_to_toml(COMPACT).unwrap();
        assert_eq!(toml.lines().count(), COMPACT.lines().count());
    }

    #[test]
    fn prose_never_reaches_the_parser() {
        // A prose line containing `=` or `[..]` outside a fence
        // must not become a key or a section.
        let doc = "prose with spurious = sign\n[not-a-section] in prose\n```toml\nk = \"v\"\n```\n";
        let map = toml_simple::toml_parse(&md_to_toml(doc).unwrap());
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("k").map(String::as_str), Some("v"));
    }

    #[test]
    fn non_toml_fences_are_illustration() {
        let doc = "```\nignored = \"yes\"\n```\n```sh\nexport X=1\n```\n```toml\nk = \"v\"\n```\n";
        let map = toml_simple::toml_parse(&md_to_toml(doc).unwrap());
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("k"));
    }

    #[test]
    fn unclosed_fence_errors() {
        let doc = "prose\n```toml\nk = \"v\"\n";
        let err = md_to_toml(doc).unwrap_err();
        assert!(err.contains("line 2"), "unexpected error: {err}");
    }

    #[test]
    fn resolve_prefers_lone_carrier_and_rejects_both() {
        let dir = crate::test_tmp_root::resolve_tmp_root().join("vc_x1_config_md_resolve");
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();

        assert!(vc_config_path(&dir).unwrap().is_none());

        std::fs::write(dir.join(VC_CONFIG_FILE), "[repos]\nwork = \".\"\n").unwrap();
        assert_eq!(
            vc_config_path(&dir).unwrap(),
            Some(dir.join(VC_CONFIG_FILE))
        );

        std::fs::write(
            dir.join(VC_CONFIG_MD),
            "```toml\n[repos]\nwork = \".\"\n```\n",
        )
        .unwrap();
        let err = vc_config_path(&dir).unwrap_err();
        assert!(err.contains("both"), "unexpected error: {err}");

        std::fs::remove_file(dir.join(VC_CONFIG_FILE)).unwrap();
        assert_eq!(vc_config_path(&dir).unwrap(), Some(dir.join(VC_CONFIG_MD)));

        let cfg = load(&dir).unwrap().unwrap();
        assert_eq!(cfg.path, dir.join(VC_CONFIG_MD));
        assert_eq!(cfg.map.get("repos.work").map(String::as_str), Some("."));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn md_carrier_drives_workspace_topology() {
        let root = crate::test_tmp_root::resolve_tmp_root().join("vc_x1_config_md_workspace");
        std::fs::remove_dir_all(&root).ok();
        let bot = root.join(".claude");
        std::fs::create_dir_all(&bot).unwrap();

        std::fs::write(
            root.join(VC_CONFIG_MD),
            "The two repos\n```toml\n[repos]\nwork = \".\"\nagent = \".claude\"\n```\n",
        )
        .unwrap();
        std::fs::write(
            bot.join(VC_CONFIG_MD),
            "The two repos\n```toml\n[repos]\nwork = \"..\"\nagent = \".\"\n```\n",
        )
        .unwrap();

        let canon_root = root.canonicalize().unwrap();
        assert_eq!(
            crate::common::find_workspace_root_from(&bot),
            Some(canon_root)
        );
        assert!(crate::common::is_bot_dir(&bot));
        assert!(!crate::common::is_bot_dir(&root));
        assert!(crate::common::default_scope(Some(&root)).has_bot());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn mixed_carriers_across_sides() {
        let root = crate::test_tmp_root::resolve_tmp_root().join("vc_x1_config_md_mixed");
        std::fs::remove_dir_all(&root).ok();
        let bot = root.join(".claude");
        std::fs::create_dir_all(&bot).unwrap();

        std::fs::write(
            root.join(VC_CONFIG_FILE),
            "[repos]\nwork = \".\"\nagent = \".claude\"\n",
        )
        .unwrap();
        std::fs::write(
            bot.join(VC_CONFIG_MD),
            "The two repos\n```toml\n[repos]\nwork = \"..\"\nagent = \".\"\n```\n",
        )
        .unwrap();

        let canon_root = root.canonicalize().unwrap();
        assert_eq!(
            crate::common::find_workspace_root_from(&bot),
            Some(canon_root)
        );
        assert!(crate::common::is_bot_dir(&bot));
        assert!(crate::common::default_scope(Some(&root)).has_bot());

        let work_cfg = load(&root).unwrap().unwrap();
        assert_eq!(work_cfg.path, root.join(VC_CONFIG_FILE));
        assert_eq!(
            work_cfg.map.get("repos.agent").map(String::as_str),
            Some(".claude")
        );
        let bot_cfg = load(&bot).unwrap().unwrap();
        assert_eq!(bot_cfg.path, bot.join(VC_CONFIG_MD));
        assert_eq!(
            bot_cfg.map.get("repos.work").map(String::as_str),
            Some("..")
        );

        std::fs::remove_dir_all(&root).ok();
    }
}

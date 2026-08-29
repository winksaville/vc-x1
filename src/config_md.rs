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

/// The config-file model's generator.
///
/// Test-only on purpose. The artifact is the committed
/// `vc-config-model.md`, and this module is what writes it, run by
/// `model_file_is_current` with `VC_X1_UPDATE_MODEL=1`. Nothing in
/// the shipped binary reads a model, so shipping its generator
/// would be dead weight, and the checks that will read one lift out
/// what they need when they land.
#[cfg(test)]
mod model {
    use crate::config_schema::{
        ConfigKey, Home, ValueKind, format_value, schema, section_and_leaf, wrap_prefixed,
    };
    use crate::toml_simple;

    /// The model config file's name, at the repo root.
    ///
    /// Generated rather than maintained, so "carries every key" holds
    /// by construction rather than by a hand count. The
    /// `model_file_is_current` test keeps the committed copy so.
    pub const VC_CONFIG_MODEL: &str = "vc-config-model.md";

    /// The prose width the project wraps durable text at, which the
    /// model's bullets and its `str-list` values follow.
    const PROSE_WIDTH: usize = 100;

    /// The model's prose header, everything above its first table.
    const MODEL_INTRO: &str = "\
# vc-x1 config file

A model config file: every table and key a workspace config may carry, each with its default or a
typical value. Generated from the schema, so it cannot fall behind the keys the binary knows. Copy
the tables you need into your own `.vc-config.md` and drop the rest.

The values inside a `toml` fence are the workspace's own, and the prose around them is
documentation. Only a fence tagged exactly `toml` is read, so any other fence is prose like the
text beside it. Each bullet links to its key's entry in the schema documentation.
";

    /// The value a model file shows for `key`: its default, else its
    /// representative example, else a placeholder by kind.
    ///
    /// The prototype requires one of the two, so the placeholder arm is
    /// unreachable for a schema build.rs accepted, and it stays for the
    /// same reason [`crate::config_schema::render_value`] has one.
    fn model_value(key: &ConfigKey) -> String {
        match key.default.or(key.example) {
            Some(v) => format_value(key.kind, v),
            None => crate::config_schema::render_value(key),
        }
    }

    /// Render one key's assignment line (or lines) for a model fence.
    ///
    /// A `str-list` whose one-line form would run past the prose width
    /// breaks one element per line, which is the shape a real config
    /// file uses for a `[validate]` table and the one a reader can diff
    /// a command out of. Everything else is one line.
    fn model_assignment(key: &ConfigKey) -> String {
        let (_section, leaf) = section_and_leaf(key.path);
        let value = model_value(key);
        let one_line = format!("{leaf} = {value}\n");
        if key.kind != ValueKind::StrList || one_line.len() <= PROSE_WIDTH {
            return one_line;
        }
        let Ok(items) = toml_simple::parse_array(&value, key.path) else {
            return one_line;
        };
        let mut out = format!("{leaf} = [\n");
        for item in items {
            out.push_str(&format!("  {item:?},\n"));
        }
        out.push_str("]\n");
        out
    }

    /// True when `key` belongs in a workspace config file at all.
    ///
    /// The user-home-only keys (`default.*`, `repo.*`, `account.*`)
    /// are a different file, so a model of a workspace config never
    /// shows them.
    pub(super) fn in_workspace(key: &ConfigKey) -> bool {
        key.homes
            .iter()
            .any(|h| matches!(h, Home::WorkspaceCode | Home::WorkspaceBot))
    }

    /// Render the model config file: every workspace-side key grouped
    /// into its table, each table's keys listed as doc-link bullets
    /// above a `toml` fence carrying that table's values.
    ///
    /// The shape is the compact one a hand-written `.vc-config.md`
    /// uses, so the model reads as the thing it models:
    ///
    /// - one `- <leaf>: <doc> [[N]]` bullet per key, wrapped at the
    ///   prose width, its `[[N]]` defined in the trailing
    ///   `# References` section as that key's documentation url
    /// - one fence per table, keys in schema order, so the file's
    ///   reading order is the schema's
    ///
    /// A key's url is its `reference`, which build.rs derives from
    /// `[vc-config] reference-base`, so a fork's model points at the
    /// fork's own copy of the documentation.
    pub fn render_model() -> String {
        let mut sections: Vec<(&str, Vec<&ConfigKey>)> = Vec::new();
        for key in schema().iter().filter(|k| in_workspace(k)) {
            let (section, _leaf) = section_and_leaf(key.path);
            match sections.last_mut() {
                Some((seen, keys)) if *seen == section => keys.push(key),
                _ => sections.push((section, vec![key])),
            }
        }

        let mut out = String::from(MODEL_INTRO);
        let mut references: Vec<&'static str> = Vec::new();
        for (section, keys) in &sections {
            out.push_str(&format!("\nThe `[{section}]` table\n"));
            for key in keys {
                references.push(key.reference);
                let (_section, leaf) = section_and_leaf(key.path);
                let bullet = format!("{leaf}: {} [[{}]]", key.doc, references.len());
                out.push_str(&wrap_prefixed(&bullet, "- ", "  ", PROSE_WIDTH));
            }
            out.push_str(&format!("```toml\n[{section}]\n"));
            for key in keys {
                out.push_str(&model_assignment(key));
            }
            out.push_str("```\n");
        }

        out.push_str("\n# References\n\n");
        for (i, reference) in references.iter().enumerate() {
            out.push_str(&format!("[{}]: {reference}\n", i + 1));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::model::{VC_CONFIG_MODEL, in_workspace, render_model};
    use super::*;
    use crate::config_schema::schema;

    /// The committed model file, resolved from the manifest dir so
    /// the test does not depend on the harness's working directory.
    fn model_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(VC_CONFIG_MODEL)
    }

    /// The committed model is what the schema renders today.
    ///
    /// This is what makes the file generated rather than
    /// maintained: a schema change that does not reach it fails
    /// here. Set `VC_X1_UPDATE_MODEL=1` to rewrite it, which is how
    /// a schema change lands in the file.
    #[test]
    fn model_file_is_current() {
        let rendered = render_model();
        let path = model_path();
        if std::env::var_os("VC_X1_UPDATE_MODEL").is_some() {
            std::fs::write(&path, &rendered).expect("write the model");
            return;
        }
        let committed = std::fs::read_to_string(&path).expect("read the model");
        assert_eq!(
            committed, rendered,
            "{} is stale: re-run with VC_X1_UPDATE_MODEL=1",
            VC_CONFIG_MODEL
        );
    }

    /// Every workspace-side key reaches the model.
    ///
    /// The property the file exists to hold, asserted against the
    /// schema rather than counted by hand.
    #[test]
    fn model_carries_every_workspace_key() {
        let rendered = render_model();
        let toml = md_to_toml(&rendered).expect("the model's fences");
        let map = toml_simple::toml_parse(&toml);
        for key in schema().iter().filter(|k| in_workspace(k)) {
            assert!(
                map.contains_key(key.path),
                "the model is missing {}",
                key.path
            );
        }
    }

    /// No key the model shows is one a workspace config may not
    /// hold, which is the other half of `--validate`'s question.
    #[test]
    fn model_shows_no_user_only_key() {
        let rendered = render_model();
        let toml = md_to_toml(&rendered).expect("the model's fences");
        let map = toml_simple::toml_parse(&toml);
        for path in map.keys() {
            let key = schema()
                .iter()
                .find(|k| k.path == path)
                .unwrap_or_else(|| panic!("the model shows an unknown key {path}"));
            assert!(in_workspace(key), "the model shows a user-only key {path}");
        }
    }

    /// The compact shape the model renders and a hand-written
    /// config file uses: one fence per table, doc-link bullets
    /// above it.
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

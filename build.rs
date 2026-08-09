//! Export the resolved `jj-lib` version as `JJ_LIB_VERSION`,
//! guard the single-name convention, and generate the config
//! schema from the `vc-config.toml` prototype.
//!
//! jj-lib exports no version constant of its own, so the only
//! statement of what this binary links against is the resolved
//! version in `Cargo.lock`:
//!
//! - the lock is read from `$CARGO_MANIFEST_DIR` directly, not by
//!   walking ancestors: we are not a workspace, and a walk can
//!   bind a sibling project's lock, which is worse than failing.
//! - a missing or jj-lib-less lock fails the build rather than
//!   emitting an "unknown" that would later be reported as fact.
//!
//! The name guard (chores-16, 0.78.3): a suffixed version marks a
//! dev rung, and a dev rung must never carry the stable package
//! name, or installing it replaces the stable `vc-x1` binary with
//! unpushed code. It lives here rather than in a `#[test]` because
//! `cargo install` never runs tests: a build script runs on every
//! cargo verb, so the forbidden combination fails to compile at
//! all, install included.

use std::path::Path;

/// Extract `jj-lib`'s resolved version from `Cargo.lock` text.
///
/// Scans for the `[[package]]` block whose `name` is `jj-lib` and
/// returns that block's `version`. Line-based rather than a TOML
/// parse: build scripts get no dev-dependencies, and the lock's
/// shape is fixed by cargo.
fn jj_lib_version(lock: &str) -> Option<String> {
    let mut in_jj_lib = false;
    for line in lock.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            in_jj_lib = false;
        } else if let Some(rest) = line.strip_prefix("name = ") {
            in_jj_lib = rest.trim_matches('"') == "jj-lib";
        } else if let Some(rest) = line.strip_prefix("version = ")
            && in_jj_lib
        {
            return Some(rest.trim_matches('"').to_string());
        }
    }
    None
}

/// The single-name convention's guard: refuse to build a suffixed
/// (dev-rung) version under the stable package name.
///
/// Reported via the `cargo::error` directive rather than `panic!`:
/// cargo then prints one clean error line and fails the build,
/// instead of wrapping a build-script panic in exit-status and
/// backtrace noise.
fn guard_single_name() {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    if version.contains('-') && name == "vc-x1" {
        // The version is not echoed in the text: cargo's own
        // `error: <name>@<version>:` prefix already carries it.
        println!("cargo::error=");
        println!("cargo::error=A suffixed version is used with the stable package name, `vc-x1`");
        println!("cargo::error=so refusing to build `vc-x1` as it is the stable binary name.");
        println!("cargo::error=");
    }
}

/// One key parsed from the `vc-config.toml` prototype, metadata
/// as raw strings (typing happens at render time in
/// `render_config_schema`).
struct ProtoKey {
    path: String,
    homes: Vec<String>,
    kind: String,
    doc: String,
    used_by: String,
    /// Override url; the norm is `None`, deriving
    /// `<reference-base>vc-config.md#<anchor_slug(path)>`.
    reference: Option<String>,
    default: Option<String>,
    example: Option<String>,
    required: bool,
}

/// Parse a TOML basic-string body (the text between the quotes),
/// unescaping `\"` and `\\`. Anything fancier (unicode escapes,
/// multi-line strings) is out of the prototype's dialect and
/// fails the build via the caller's validation.
fn unescape_basic_string(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                other => panic!("vc-config.toml: unsupported escape \\{other:?}"),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Parse one metadata value: a quoted string, a `[...]` string
/// array (returned joined by `\u{0}` for the caller to split), a
/// bare integer, or a bare bool. Returns the raw string form.
fn parse_value(raw: &str) -> String {
    let raw = raw.trim();
    if let Some(body) = raw.strip_prefix('"') {
        let Some(body) = body.strip_suffix('"') else {
            panic!("vc-config.toml: unterminated string: {raw}");
        };
        unescape_basic_string(body)
    } else {
        raw.to_string()
    }
}

/// Line-based parse of the `vc-config.toml` prototype into
/// `ProtoKey`s, file order preserved.
///
/// Line-based for the same reason `jj_lib_version` is: build
/// scripts see no dev-dependencies, and the prototype's dialect
/// is ours to bound (single-line `key = value` entries, dotted
/// table headers with optionally-quoted segments, `#` comments).
fn parse_prototype(text: &str) -> (String, Vec<ProtoKey>) {
    let mut keys: Vec<ProtoKey> = Vec::new();
    let mut base: Option<String> = None;
    let mut current: Option<(String, Vec<(String, String)>)> = None;

    fn finish(
        keys: &mut Vec<ProtoKey>,
        base: &mut Option<String>,
        current: Option<(String, Vec<(String, String)>)>,
    ) {
        let Some((path, fields)) = current else {
            return;
        };
        if path == "vc-config" {
            for (name, value) in fields {
                match name.as_str() {
                    "reference-base" => *base = Some(value),
                    other => panic!("vc-config.toml: [vc-config]: unknown entry {other:?}"),
                }
            }
            return;
        }
        let mut homes: Vec<String> = Vec::new();
        let mut kind = None;
        let mut doc = None;
        let mut used_by = None;
        let mut reference = None;
        let mut default = None;
        let mut example = None;
        let mut required = false;
        for (name, value) in fields {
            match name.as_str() {
                "homes" => {
                    homes = value.split('\u{0}').map(str::to_string).collect();
                }
                "kind" => kind = Some(value),
                "doc" => doc = Some(value),
                "used-by" => used_by = Some(value),
                "reference" => reference = Some(value),
                "default" => default = Some(value),
                "example" => example = Some(value),
                "required" => match value.as_str() {
                    "true" => required = true,
                    "false" => required = false,
                    other => panic!("vc-config.toml: [{path}] required = {other}: not a bool"),
                },
                other => panic!("vc-config.toml: [{path}]: unknown entry {other:?}"),
            }
        }
        let missing = |what: &str| -> ! { panic!("vc-config.toml: [{path}] is missing {what}") };
        let kind = kind.unwrap_or_else(|| missing("kind"));
        if !matches!(kind.as_str(), "str" | "usize" | "item-list") {
            panic!("vc-config.toml: [{path}] kind = {kind:?}: not str/usize/item-list");
        }
        if kind == "usize"
            && let Some(d) = &default
            && d.parse::<usize>().is_err()
        {
            panic!("vc-config.toml: [{path}] default {d:?} is not a usize");
        }
        if homes.is_empty() {
            missing("homes");
        }
        if default.is_none() && example.is_none() {
            missing("a default or an example");
        }
        let doc = doc.unwrap_or_else(|| missing("doc"));
        let used_by = used_by.unwrap_or_else(|| missing("used-by"));
        keys.push(ProtoKey {
            path,
            homes,
            kind,
            doc,
            used_by,
            reference,
            default,
            example,
            required,
        });
    }

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(inner) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            finish(&mut keys, &mut base, current.take());
            current = Some((inner.replace('"', ""), Vec::new()));
            continue;
        }
        let Some((name, value)) = trimmed.split_once('=') else {
            panic!("vc-config.toml: unparseable line: {trimmed}");
        };
        let Some((_, fields)) = current.as_mut() else {
            panic!("vc-config.toml: entry before first table header: {trimmed}");
        };
        let name = name.trim().to_string();
        let raw = value.trim();
        let value = if name == "homes" {
            let Some(body) = raw.strip_prefix('[').and_then(|s| s.strip_suffix(']')) else {
                panic!("vc-config.toml: homes must be an array: {raw}");
            };
            body.split(',')
                .map(|s| parse_value(s.trim()))
                .collect::<Vec<_>>()
                .join("\u{0}")
        } else {
            parse_value(raw)
        };
        fields.push((name, value));
    }
    finish(&mut keys, &mut base, current);

    let mut paths: Vec<&str> = keys.iter().map(|k| k.path.as_str()).collect();
    paths.sort_unstable();
    paths.dedup();
    if paths.len() != keys.len() {
        panic!("vc-config.toml: duplicate key path(s)");
    }
    let Some(base) = base else {
        panic!("vc-config.toml: no [vc-config] reference-base");
    };
    (base, keys)
}

/// A key path's GitHub-style heading anchor: lowercased, keeping
/// only `[a-z0-9-]` (dots and `<`/`>` drop out), spaces to
/// hyphens. `bot-session.col-width` -> `bot-sessioncol-width`,
/// matching the slug of that key's `##` heading in vc-config.md.
fn anchor_slug(path: &str) -> String {
    path.chars()
        .filter_map(|c| match c {
            'a'..='z' | '0'..='9' | '-' => Some(c),
            'A'..='Z' => Some(c.to_ascii_lowercase()),
            ' ' => Some('-'),
            _ => None,
        })
        .collect()
}

/// Render the generated schema module: the `SCHEMA_GEN` table
/// plus one `<PATH>_DEFAULT` constant per non-dynamic defaulted
/// key, typed by `kind`. Each key's reference is its override
/// url when given, else derived from `base` + the key's
/// vc-config.md anchor.
fn render_config_schema(keys: &[ProtoKey], base: &str) -> String {
    let mut out = String::from(
        "// Generated from vc-config.toml by build.rs. Do not edit.\n\
         static SCHEMA_GEN: &[ConfigKey] = &[\n",
    );
    for key in keys {
        let homes = key
            .homes
            .iter()
            .map(|h| match h.as_str() {
                "user" => "Home::User",
                "workspace-code" => "Home::WorkspaceCode",
                "workspace-bot" => "Home::WorkspaceBot",
                other => panic!("vc-config.toml: [{}] unknown home {other:?}", key.path),
            })
            .collect::<Vec<_>>()
            .join(", ");
        let kind = match key.kind.as_str() {
            "str" => "ValueKind::Str",
            "usize" => "ValueKind::Usize",
            "item-list" => "ValueKind::ItemList",
            other => panic!("vc-config.toml: [{}] unknown kind {other:?}", key.path),
        };
        let opt = |v: &Option<String>| match v {
            Some(s) => format!("Some({s:?})"),
            None => "None".to_string(),
        };
        // `blob/HEAD` resolves to the repo's default branch at
        // click time, so the base names only the repo and no
        // branch is baked in (a build-time branch would point
        // shipped references at whatever the builder had checked
        // out). The join is GitHub-shaped; a host needing another
        // shape uses the per-key `reference` override.
        let reference = key.reference.clone().unwrap_or_else(|| {
            format!(
                "{}/blob/HEAD/vc-config.md#{}",
                base.trim_end_matches('/'),
                anchor_slug(&key.path)
            )
        });
        out.push_str(&format!(
            "    ConfigKey {{\n\
             \x20       path: {path:?},\n\
             \x20       homes: &[{homes}],\n\
             \x20       kind: {kind},\n\
             \x20       default: {default},\n\
             \x20       required: {required},\n\
             \x20       dynamic: {dynamic},\n\
             \x20       doc: {doc:?},\n\
             \x20       used_by: {used_by:?},\n\
             \x20       example: {example},\n\
             \x20       reference: {reference:?},\n\
             \x20   }},\n",
            path = key.path,
            default = opt(&key.default),
            required = key.required,
            dynamic = key.path.contains('<'),
            doc = key.doc,
            used_by = key.used_by,
            example = opt(&key.example),
            reference = reference,
        ));
    }
    out.push_str("];\n");

    for key in keys {
        let Some(default) = &key.default else {
            continue;
        };
        if key.path.contains('<') {
            continue;
        }
        let name = format!(
            "{}_DEFAULT",
            key.path.to_uppercase().replace(['.', '-'], "_")
        );
        // Not every generated constant has a non-test consumer
        // (e.g. the items default's runtime twin is
        // `ItemSet::BUILTIN`), so silence dead_code per constant.
        out.push_str("#[allow(dead_code)]\n");
        match key.kind.as_str() {
            "usize" => out.push_str(&format!("pub const {name}: usize = {default};\n")),
            _ => out.push_str(&format!("pub const {name}: &str = {default:?};\n")),
        }
    }
    out
}

/// Parse the prototype and write the generated schema module
/// into `$OUT_DIR/config_schema_gen.rs`.
fn generate_config_schema(manifest_dir: &str) {
    let proto_path = Path::new(manifest_dir).join("vc-config.toml");
    println!("cargo::rerun-if-changed={}", proto_path.display());
    let text = match std::fs::read_to_string(&proto_path) {
        Ok(text) => text,
        Err(e) => panic!("cannot read {}: {e}", proto_path.display()),
    };
    let (base, keys) = parse_prototype(&text);
    let out_dir = match std::env::var("OUT_DIR") {
        Ok(dir) => dir,
        Err(e) => panic!("OUT_DIR unset: {e}"),
    };
    let gen_path = Path::new(&out_dir).join("config_schema_gen.rs");
    if let Err(e) = std::fs::write(&gen_path, render_config_schema(&keys, &base)) {
        panic!("cannot write {}: {e}", gen_path.display());
    }
}

/// Read the lock beside `Cargo.toml` and emit the version env var;
/// enforce the single-name guard first, then generate the config
/// schema from the prototype.
fn main() {
    guard_single_name();

    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => dir,
        Err(e) => panic!("CARGO_MANIFEST_DIR unset: {e}"),
    };
    generate_config_schema(&manifest_dir);

    let lock_path = Path::new(&manifest_dir).join("Cargo.lock");
    println!("cargo::rerun-if-changed={}", lock_path.display());

    let lock = match std::fs::read_to_string(&lock_path) {
        Ok(text) => text,
        Err(e) => panic!("cannot read {}: {e}", lock_path.display()),
    };
    match jj_lib_version(&lock) {
        Some(version) => println!("cargo::rustc-env=JJ_LIB_VERSION={version}"),
        None => panic!("no jj-lib package in {}", lock_path.display()),
    }
}

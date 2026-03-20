use std::collections::HashMap;
use std::path::Path;

/// Load a TOML file into a flat key-value map.
///
/// Handles `[section]` headers, bare `key = "value"` pairs, comments, and
/// blank lines. Keys under a section are stored as `section.key`. Quoted
/// string values have their quotes stripped; unquoted values are stored
/// as-is.
///
/// This is intentionally minimal — just enough for `.vc-config.toml`.
pub fn toml_load(path: &Path) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read '{}': {e}", path.display()))?;

    let mut map = HashMap::new();
    let mut section = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip blank lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Section header: [name]
        if let Some(inner) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            section = inner.trim().to_string();
            continue;
        }

        // Key = value
        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            // Strip surrounding quotes from string values
            let value = if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                &value[1..value.len() - 1]
            } else {
                value
            };

            let full_key = if section.is_empty() {
                key.to_string()
            } else {
                format!("{section}.{key}")
            };

            map.insert(full_key, value.to_string());
        }
    }

    Ok(map)
}

/// Look up a dotted key (e.g. `"workspace.path"`) in a loaded config map.
pub fn toml_get<'a>(map: &'a HashMap<String, String>, key: &str) -> Option<&'a String> {
    map.get(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_section_and_key() {
        let dir = std::env::temp_dir().join("toml_simple_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.toml");
        std::fs::write(&path, "# comment\n\n[workspace]\npath = \"/\"\n").unwrap();

        let map = toml_load(&path).unwrap();
        assert_eq!(toml_get(&map, "workspace.path"), Some(&"/".to_string()));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn parse_quoted_value() {
        let dir = std::env::temp_dir().join("toml_simple_quoted");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.toml");
        std::fs::write(&path, "[section]\nkey = \"value\"\n").unwrap();

        let map = toml_load(&path).unwrap();
        assert_eq!(toml_get(&map, "section.key"), Some(&"value".to_string()));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn bare_key_no_section() {
        let dir = std::env::temp_dir().join("toml_simple_bare");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.toml");
        std::fs::write(&path, "name = \"hello\"\n").unwrap();

        let map = toml_load(&path).unwrap();
        assert_eq!(toml_get(&map, "name"), Some(&"hello".to_string()));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_file_errors() {
        let path = std::path::PathBuf::from("/nonexistent/test.toml");
        assert!(toml_load(&path).is_err());
    }
}

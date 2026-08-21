use std::collections::HashMap;
use std::path::Path;

/// Load a TOML file into a flat key-value map.
///
/// The file-reading front of [`toml_parse`]. See it for the
/// dialect. This is intentionally minimal: just enough for the
/// instance config.
pub fn toml_load(path: &Path) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read '{}': {e}", path.display()))?;
    Ok(toml_parse(&content))
}

/// Parse TOML text into a flat key-value map.
///
/// Handles `[section]` headers, bare `key = "value"` pairs, comments, and
/// blank lines. Keys under a section are stored as `section.key`. Quoted
/// string values have their quotes stripped, and unquoted values are
/// stored as-is. An array value (`key = [ ... ]`, on one line or spread
/// over several up to the closing `]`) is stored as its TOML text with
/// the brackets, and [`toml_get_list`] splits it into elements.
pub fn toml_parse(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut section = String::new();
    let mut open_array: Option<(String, String)> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Inside a multi-line array: accumulate until the closing `]`.
        if let Some((_, text)) = open_array.as_mut() {
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                text.push(' ');
                text.push_str(trimmed);
            }
            if trimmed.ends_with(']')
                && let Some((key, text)) = open_array.take()
            {
                map.insert(key, text);
            }
            continue;
        }

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

            if value.starts_with('[') && !value.ends_with(']') {
                open_array = Some((full_key, value.to_string()));
                continue;
            }
            map.insert(full_key, value.to_string());
        }
    }

    map
}

/// Split an array value into its string elements.
///
/// - `Ok(None)` when `key` is absent.
/// - `Err` when the value is not an array of double-quoted strings
///   (a bare scalar, or an element without quotes): the message
///   names the key, so a `[validate]` table holding `cargo test`
///   unquoted fails by name rather than running nothing.
/// - Elements keep their text verbatim apart from the quotes, so a
///   command line with its own spaces and dashes is one element.
pub fn toml_get_list(
    map: &HashMap<String, String>,
    key: &str,
) -> Result<Option<Vec<String>>, String> {
    let Some(raw) = toml_get(map, key) else {
        return Ok(None);
    };
    let Some(body) = raw
        .trim()
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
    else {
        return Err(format!(
            "{key}: expected an array like [\"a\", \"b\"], got {raw}"
        ));
    };
    let mut items = Vec::new();
    let mut rest = body.trim();
    while !rest.is_empty() {
        let Some(after_open) = rest.strip_prefix('"') else {
            return Err(format!(
                "{key}: array elements must be double-quoted strings, at: {rest}"
            ));
        };
        let Some(close) = after_open.find('"') else {
            return Err(format!("{key}: unterminated string in array: {raw}"));
        };
        items.push(after_open[..close].to_string());
        rest = after_open[close + 1..].trim();
        if let Some(r) = rest.strip_prefix(',') {
            rest = r.trim();
        } else if !rest.is_empty() {
            return Err(format!(
                "{key}: expected `,` between array elements, at: {rest}"
            ));
        }
    }
    Ok(Some(items))
}

/// Look up a `.`-joined config key (e.g. `"repos.work"`) in a loaded config map.
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
        std::fs::write(&path, "# comment\n\n[repos]\nwork = \".\"\n").unwrap();

        let map = toml_load(&path).unwrap();
        assert_eq!(toml_get(&map, "repos.work"), Some(&".".to_string()));
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

    /// A one-line array is stored as its TOML text and split by
    /// `toml_get_list`, elements verbatim apart from the quotes.
    #[test]
    fn array_single_line() {
        let map = toml_parse("[validate]\nfast = [\"cargo test --bins\", \"cargo fmt\"]\n");
        assert_eq!(
            toml_get_list(&map, "validate.fast").unwrap(),
            Some(vec![
                "cargo test --bins".to_string(),
                "cargo fmt".to_string()
            ])
        );
    }

    /// A multi-line array accumulates to the closing `]`, a
    /// trailing comma and interior comments allowed, and the keys
    /// after it still parse under the same section.
    #[test]
    fn array_multi_line() {
        let map = toml_parse(
            "[validate]\nfull = [\n  \"cargo fmt\",\n  # the slow one\n  \"cargo test\",\n]\n\
             fast = [\"cargo test --bins\"]\n",
        );
        assert_eq!(
            toml_get_list(&map, "validate.full").unwrap(),
            Some(vec!["cargo fmt".to_string(), "cargo test".to_string()])
        );
        assert_eq!(
            toml_get_list(&map, "validate.fast").unwrap(),
            Some(vec!["cargo test --bins".to_string()])
        );
    }

    /// `toml_get_list` is `Ok(None)` for an absent key and names the
    /// key on a scalar or an unquoted element.
    #[test]
    fn array_errors_name_the_key() {
        let map = toml_parse("[validate]\nfast = \"cargo test\"\nfull = [cargo test]\n");
        assert_eq!(toml_get_list(&map, "validate.none").unwrap(), None);
        let e = toml_get_list(&map, "validate.fast").unwrap_err();
        assert!(
            e.contains("validate.fast") && e.contains("expected an array"),
            "{e}"
        );
        let e = toml_get_list(&map, "validate.full").unwrap_err();
        assert!(
            e.contains("validate.full") && e.contains("double-quoted"),
            "{e}"
        );
    }
}

//! The md -> toml filter, on its own so build.rs can share it.
//!
//! A config markdown document's `toml` fences, concatenated in
//! document order, form the TOML a reader parses. That rule holds
//! for both markdown carriers: the instance config
//! (`.vc-config.md`, loaded by `crate::config_md`) and the schema
//! prototype (`vc-config.md`, parsed by build.rs), so the filter
//! is a file both can pull in.
//!
//! A build script is its own crate, so build.rs declares this file
//! with `#[path]` rather than a crate-relative module path. Nothing
//! here may name a crate item or a dependency: std only, and no
//! intra-doc links, which would not resolve on the build-script
//! side.

/// What the filter is inside of, line by line.
enum Fence {
    /// Outside any fence: prose, blanked.
    None,
    /// Inside a ```toml fence: lines pass through.
    Toml,
    /// Inside any other fence (illustration idiom): blanked.
    Other,
}

/// Extract the TOML a config markdown document carries.
///
/// - `toml`-tagged fence interiors pass through verbatim.
/// - Every other line (prose, fence markers, other fences'
///   interiors) is blanked rather than removed, so the result has
///   the source's line count and any parse diagnostic points at
///   the real line.
/// - The tag must be exactly `toml` (` ```toml `); a fence tagged
///   otherwise or untagged is illustration and is ignored.
/// - An unclosed fence is an error naming its opening line.
pub fn md_to_toml(content: &str) -> Result<String, String> {
    let mut state = Fence::None;
    let mut opened_at = 0usize;
    let mut out = String::with_capacity(content.len());
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        match state {
            Fence::None => {
                if let Some(info) = trimmed.strip_prefix("```") {
                    state = if info.trim() == "toml" {
                        Fence::Toml
                    } else {
                        Fence::Other
                    };
                    opened_at = idx + 1;
                }
            }
            Fence::Toml => {
                if is_fence_close(trimmed) {
                    state = Fence::None;
                } else {
                    out.push_str(line);
                }
            }
            Fence::Other => {
                if is_fence_close(trimmed) {
                    state = Fence::None;
                }
            }
        }
        out.push('\n');
    }
    if matches!(state, Fence::None) {
        Ok(out)
    } else {
        Err(format!("unclosed fence opened at line {opened_at}"))
    }
}

/// True when a (trim_start'ed) line closes a fence: backticks with
/// nothing but whitespace after them.
fn is_fence_close(trimmed: &str) -> bool {
    trimmed
        .strip_prefix("```")
        .is_some_and(|rest| rest.trim().is_empty())
}

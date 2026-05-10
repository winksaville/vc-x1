//! `--squash SOURCE,TARGET` — pick the `jj squash --from/--into`
//! revision pair (e.g. `@,@-`). See [options_flags](README.md) for
//! shared architecture.

use clap::Args;

/// Parsed `--squash` value: the source and target revisions.
///
/// - Produced by [`SquashSpec::parse`] (also wired as the leaf's
///   `value_parser`, so clap rejects malformed input at parse time).
/// - Both halves are guaranteed non-empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SquashSpec {
    pub source: String,
    pub target: String,
}

impl SquashSpec {
    /// Parse a `SOURCE,TARGET` pair (e.g. `@,@-`); both halves
    /// must be non-empty.
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(format!(
                "invalid --squash value '{s}': expected SOURCE,TARGET (e.g. @,@-)"
            ));
        }
        Ok(SquashSpec {
            source: parts[0].to_string(),
            target: parts[1].to_string(),
        })
    }
}

/// `OptionParser` impl for `--squash` (non-boolean domain).
/// Documentation-level — consumers can use either
/// `SquashSpec::parse` directly or `SquashSpecParser::parse`.
pub struct SquashSpecParser;

impl super::OptionParser for SquashSpecParser {
    type Value = SquashSpec;

    fn parse(s: &str) -> Result<Self::Value, String> {
        SquashSpec::parse(s)
    }
}

/// `--squash` leaf (Option — non-boolean domain) — see
/// [Consuming an OF](README.md#consuming-an-of).
///
/// - The single field is named `value` (the parsed value-side of
///   the option); the flag name comes from `#[arg(long = "squash")]`,
///   so the consumer reads `args.<leaf>.value` rather than doubling
///   the flag name.
/// - Bare `--squash` (no value) defaults to `@,@-`.
/// - `num_args = 0..=1` so the value form `--squash @,@-` also
///   works without quoting.
#[derive(Args, Debug, Clone, Default)]
pub struct SquashOption {
    /// Squash SOURCE into TARGET [default: @,@-]
    #[arg(
        long = "squash",
        value_name = "SOURCE,TARGET",
        value_parser = SquashSpec::parse,
        default_missing_value = "@,@-",
        num_args = 0..=1
    )]
    pub value: Option<SquashSpec>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squash_parse_valid() {
        let sq = SquashSpec::parse("@,@-").unwrap();
        assert_eq!(sq.source, "@");
        assert_eq!(sq.target, "@-");
    }

    #[test]
    fn squash_parse_invalid() {
        assert!(SquashSpec::parse("@").is_err());
        assert!(SquashSpec::parse(",").is_err());
        assert!(SquashSpec::parse("@,").is_err());
        assert!(SquashSpec::parse(",@-").is_err());
    }
}

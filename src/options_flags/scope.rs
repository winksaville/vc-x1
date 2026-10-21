//! `--scope`: which side(s) of a dual workspace a command
//! operates on. See [options_flags](README.md) for shared
//! architecture.
//!
//! - `Side` is the role enum: `Work` or `Bot`. The CLI
//!   keywords that name them are `work` and `agent`.
//! - `Scope` is a newtype over `Vec<Side>`: the parsed role
//!   set (`work`, `agent`, `both`, their short forms `w`, `a`,
//!   `b`, and the spelled-out `work,agent` / `agent,work`).
//! - `RepoScope` is the write commands' scope, and status's: a
//!   `Scope`, or `Por`, a plain repo with no pair, spelled `por` or
//!   `p`. It is required where a write command takes it, so a write
//!   command never guesses which repos it acts on. `-R/--repo` beside it is
//!   only ever where: the workspace for a `Scope`, the repo
//!   itself for `Por`.

use clap::ValueEnum;

/// One side of a dual-repo workspace.
///
/// - `Work`: the primary work repo.
/// - `Bot`: the bot repo (typically at `.claude/`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Side {
    Work,
    Bot,
}

/// Parsed `--scope` value: the requested role set of a
/// dual-repo workspace.
///
/// Newtype over `Vec<Side>`. The vector preserves the order the
/// keywords were given in (`work,agent` vs `agent,work`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope(pub Vec<Side>);

/// Parse a `--scope` value string.
///
/// Accepts the role-keyword forms (`work`, `agent`, `both`, and
/// the spelled-out `work,agent` / `agent,work`) preserving order,
/// `both` being `work,agent`. Anything else (an
/// empty string, a bare name, duplicate or out-of-set
/// combinations, a path) is an error: path-based single-repo
/// operation uses `-R/--repo`, not `--scope`.
pub fn parse_scope(s: &str) -> Result<Scope, String> {
    match s {
        "work" | "w" => Ok(Scope(vec![Side::Work])),
        "agent" | "a" => Ok(Scope(vec![Side::Bot])),
        "both" | "b" | "work,agent" => Ok(Scope(vec![Side::Work, Side::Bot])),
        "agent,work" => Ok(Scope(vec![Side::Bot, Side::Work])),
        "" => Err("--scope: value is empty".into()),
        "bot" | "work,bot" | "bot,work" => Err(format!(
            "--scope: '{s}' is the pre-0.80.0 spelling. The agent side is \
             `agent`: use `{}`.",
            s.replace("bot", "agent")
        )),
        other => Err(format!(
            "--scope: '{other}' is not a recognized form. \
             Expected one of `work`, `agent`, `both` (or `w`, `a`, `b`), \
             `work,agent`, `agent,work`. \
             For single-repo operation by path, use `-R/--repo`."
        )),
    }
}

/// The scope a write command acts on: one or both sides of a dual
/// workspace, or a plain repo with no pair.
///
/// Required where it is taken, so the command never guesses. The
/// resolver behind it is
/// [`resolve_scoped_repos`](crate::common::resolve_scoped_repos).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepoScope {
    /// One or both sides of a dual workspace, `w`, `a`, or `b`.
    Sides(Scope),
    /// A plain old repo, `p`: the repo `-R` names, or the cwd's,
    /// which is no side of any workspace.
    Por,
}

/// The four scopes as the help and the errors list them.
pub const REPO_SCOPE_FORMS: &str = "`w|work`, `a|agent`, `b|both`, or `p|por`";

/// Parse a write command's `-s` value: the four scopes, long or
/// short, and nothing else. The spelled-out `work,agent` forms are
/// `parse_scope`'s, for the read commands that predate `both`.
pub fn parse_repo_scope(s: &str) -> Result<RepoScope, String> {
    match s {
        "work" | "w" => Ok(RepoScope::Sides(Scope(vec![Side::Work]))),
        "agent" | "a" => Ok(RepoScope::Sides(Scope(vec![Side::Bot]))),
        "both" | "b" => Ok(RepoScope::Sides(Scope(vec![Side::Work, Side::Bot]))),
        "por" | "p" => Ok(RepoScope::Por),
        "" => Err("-s: value is empty".into()),
        "bot" => Err("-s: 'bot' is the pre-0.80.0 spelling. The agent side is `a|agent`.".into()),
        other => Err(format!(
            "-s: '{other}' is not a scope. Expected one of {REPO_SCOPE_FORMS}."
        )),
    }
}

/// Every scope a [`RepoScope`] can name, by its long name, for a
/// command that offers all four.
pub const ALL_REPO_SCOPES: &[&str] = &["work", "agent", "both", "por"];

/// The four scopes' long and short names, in the order help lists them.
const REPO_SCOPE_NAMES: [(&str, &str); 4] =
    [("work", "w"), ("agent", "a"), ("both", "b"), ("por", "p")];

/// A command's `-s` parser: the four scopes declared to clap, so shell
/// completion offers the long names `offered` lists, where a bare
/// function parser offers nothing. Each command passes the scopes it
/// takes, and this one builder makes every command's parser.
///
/// - The short forms `w`, `a`, `b`, `p` are hidden aliases, so
///   completion offers the long names alone.
/// - A scope `offered` leaves out is declared hidden, not dropped: it
///   still parses, and reaches the command's own refusal, which says
///   why, as commit's says an agent commit made first is unpaired.
///   Completion and the listed values show only what is offered.
/// - `bot`, the pre-0.80.0 spelling, is hidden too, so typing it still
///   reaches `parse_repo_scope`'s fix-it naming `a|agent`.
/// - Any other value is clap's error listing the offered scopes.
pub fn repo_scope_parser(
    offered: &'static [&'static str],
) -> impl clap::builder::TypedValueParser<Value = RepoScope> {
    use clap::builder::PossibleValue;
    use clap::builder::TypedValueParser as _;
    debug_assert!(
        offered.iter().all(|o| ALL_REPO_SCOPES.contains(o)),
        "repo_scope_parser: {offered:?} names a scope outside {ALL_REPO_SCOPES:?}"
    );
    let mut values: Vec<PossibleValue> = REPO_SCOPE_NAMES
        .iter()
        .map(|(long, short)| {
            PossibleValue::new(*long)
                .alias(*short)
                .hide(!offered.contains(long))
        })
        .collect();
    values.push(PossibleValue::new("bot").hide(true));
    clap::builder::PossibleValuesParser::new(values).try_map(|s: String| parse_repo_scope(&s))
}

/// The one-side keywords as a parser that declares them, `work`
/// and `agent`, for a flag that names the side a thing is on.
pub fn side_keywords() -> impl clap::builder::TypedValueParser<Value = Side> {
    use clap::builder::TypedValueParser as _;
    clap::builder::PossibleValuesParser::new(["work", "agent"]).try_map(|s: String| {
        match s.as_str() {
            "work" => Ok(Side::Work),
            "agent" => Ok(Side::Bot),
            other => Err(format!(
                "'{other}' is not a side: expected `work` or `agent`"
            )),
        }
    })
}

/// A side's CLI keyword, `work` or `agent`.
pub fn side_keyword(side: Side) -> &'static str {
    match side {
        Side::Work => "work",
        Side::Bot => "agent",
    }
}

impl Scope {
    /// True when the role set includes the work side.
    pub fn has_work(&self) -> bool {
        self.0.contains(&Side::Work)
    }

    /// True when the role set includes the bot side.
    pub fn has_bot(&self) -> bool {
        self.0.contains(&Side::Bot)
    }

    /// Exactly the work side: a single-side dual-repo op.
    pub fn is_work_only(&self) -> bool {
        self.has_work() && !self.has_bot()
    }

    /// Exactly the bot side: a single-side dual-repo op.
    #[allow(dead_code)]
    pub fn is_bot_only(&self) -> bool {
        !self.has_work() && self.has_bot()
    }

    /// Both sides: a full dual-repo op.
    pub fn is_both(&self) -> bool {
        self.has_work() && self.has_bot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pre-0.80.0 `bot` keywords are rejected with the respelled
    /// form as the fix-it, never aliased.
    #[test]
    fn parse_scope_rejects_old_bot_keywords() {
        for (old, fixit) in [
            ("bot", "`agent`"),
            ("work,bot", "`work,agent`"),
            ("bot,work", "`agent,work`"),
        ] {
            let err = parse_scope(old).unwrap_err();
            assert!(err.contains("pre-0.80.0"), "{old}: {err}");
            assert!(err.contains(fixit), "{old}: {err}");
        }
    }

    #[test]
    fn work_only() {
        let s = Scope(vec![Side::Work]);
        assert!(s.is_work_only());
        assert!(!s.is_bot_only());
        assert!(!s.is_both());
    }

    #[test]
    fn bot_only() {
        let s = Scope(vec![Side::Bot]);
        assert!(!s.is_work_only());
        assert!(s.is_bot_only());
        assert!(!s.is_both());
    }

    #[test]
    fn both_work_then_bot() {
        let s = Scope(vec![Side::Work, Side::Bot]);
        assert!(!s.is_work_only());
        assert!(!s.is_bot_only());
        assert!(s.is_both());
    }

    #[test]
    fn both_bot_then_work() {
        // Order doesn't matter: contains-based checks.
        let s = Scope(vec![Side::Bot, Side::Work]);
        assert!(s.is_both());
    }

    #[test]
    fn empty_roles() {
        // Empty side list isn't a state anything constructs (the
        // parser rejects empty input), but the helpers still need
        // to be well-defined on it.
        let s = Scope(vec![]);
        assert!(!s.is_work_only());
        assert!(!s.is_bot_only());
        assert!(!s.is_both());
    }

    #[test]
    fn parse_keyword_work() {
        assert_eq!(parse_scope("work").unwrap(), Scope(vec![Side::Work]));
    }

    /// The short forms are the long ones' equals, the user's
    /// spelling for the command typed most.
    #[test]
    fn parse_short_forms() {
        assert_eq!(parse_scope("w").unwrap(), parse_scope("work").unwrap());
        assert_eq!(parse_scope("a").unwrap(), parse_scope("agent").unwrap());
        assert_eq!(parse_scope("b").unwrap(), parse_scope("both").unwrap());
    }

    /// The write commands' scope: the four forms, long and short,
    /// `p` being the plain repo, and the comma forms refused.
    #[test]
    fn parse_repo_scope_forms() {
        let both = RepoScope::Sides(Scope(vec![Side::Work, Side::Bot]));
        assert_eq!(
            parse_repo_scope("w").unwrap(),
            RepoScope::Sides(Scope(vec![Side::Work]))
        );
        assert_eq!(
            parse_repo_scope("agent").unwrap(),
            RepoScope::Sides(Scope(vec![Side::Bot]))
        );
        assert_eq!(parse_repo_scope("b").unwrap(), both);
        assert_eq!(parse_repo_scope("both").unwrap(), both);
        assert_eq!(parse_repo_scope("p").unwrap(), RepoScope::Por);
        assert_eq!(parse_repo_scope("por").unwrap(), RepoScope::Por);
        for bad in ["", "work,agent", "bot", "x"] {
            let err = parse_repo_scope(bad).unwrap_err();
            assert!(err.starts_with("-s:"), "{bad:?}: {err}");
        }
        let err = parse_repo_scope("x").unwrap_err();
        assert!(err.contains("`p|por`"), "{err}");
    }

    /// The parser takes the four scopes, long and short, as
    /// `parse_repo_scope` does, hides `bot` but still answers it with
    /// the fix-it, and refuses anything else, listing what is offered.
    /// A scope left out of `offered` still parses, for the command's
    /// own refusal, and is missing from the listed values.
    #[test]
    fn repo_scope_parser_forms() {
        use clap::Parser;
        #[derive(Parser)]
        struct All {
            #[arg(short = 's', value_parser = repo_scope_parser(ALL_REPO_SCOPES))]
            s: RepoScope,
        }
        #[derive(Parser)]
        struct WorkOrPor {
            #[arg(short = 's', value_parser = repo_scope_parser(&["work", "por"]))]
            s: RepoScope,
        }
        let all = |v: &str| All::try_parse_from(["t", "-s", v]).map(|t| t.s);
        for (short, long) in [("w", "work"), ("a", "agent"), ("b", "both"), ("p", "por")] {
            assert_eq!(all(short).unwrap(), parse_repo_scope(long).unwrap());
            assert_eq!(all(long).unwrap(), parse_repo_scope(long).unwrap());
        }
        let bot = all("bot").unwrap_err().to_string();
        assert!(bot.contains("pre-0.80.0"), "{bot}");
        let other = all("work,agent").unwrap_err().to_string();
        assert!(
            other.contains("[possible values: work, agent, both, por]"),
            "{other}"
        );
        let some = |v: &str| WorkOrPor::try_parse_from(["t", "-s", v]).map(|t| t.s);
        assert_eq!(
            some("a").unwrap(),
            RepoScope::Sides(Scope(vec![Side::Bot])),
            "a scope not offered still parses, for the command to refuse"
        );
        let other = some("x").unwrap_err().to_string();
        assert!(other.contains("[possible values: work, por]"), "{other}");
    }

    #[test]
    fn parse_keyword_bot() {
        assert_eq!(parse_scope("agent").unwrap(), Scope(vec![Side::Bot]));
    }

    #[test]
    fn parse_keyword_work_bot_preserves_order() {
        assert_eq!(
            parse_scope("work,agent").unwrap(),
            Scope(vec![Side::Work, Side::Bot])
        );
    }

    /// `both` is `work,agent`, the work side first.
    #[test]
    fn parse_keyword_both() {
        assert_eq!(
            parse_scope("both").unwrap(),
            Scope(vec![Side::Work, Side::Bot])
        );
    }

    #[test]
    fn parse_keyword_bot_work_preserves_order() {
        assert_eq!(
            parse_scope("agent,work").unwrap(),
            Scope(vec![Side::Bot, Side::Work])
        );
    }

    #[test]
    fn parse_former_code_keyword_now_errors() {
        // `code` was the pre-0.74.0 spelling. The rename dropped it
        // (unreleased, no alias carried forward).
        assert!(parse_scope("code").is_err());
        assert!(parse_scope("code,bot").is_err());
    }

    #[test]
    fn parse_bare_name_errors() {
        let err = parse_scope("foo").unwrap_err();
        assert!(err.contains("'foo'"), "got: {err}");
    }

    #[test]
    fn parse_empty_errors() {
        let err = parse_scope("").unwrap_err();
        assert!(err.contains("empty"), "got: {err}");
    }

    #[test]
    fn parse_path_form_errors() {
        // Path forms are no longer a `--scope` value: `-R/--repo`
        // handles single-repo operation. The error points there.
        let err = parse_scope("./foo").unwrap_err();
        assert!(err.contains("-R/--repo"), "got: {err}");
    }

    #[test]
    fn parse_unknown_keyword_combo_errors() {
        // Duplicates and out-of-set combos are rejected. Only the
        // exact four keyword forms are accepted.
        assert!(parse_scope("work,work").is_err());
        assert!(parse_scope("work,bot,work").is_err());
        assert!(parse_scope("bot,bot").is_err());
    }
}

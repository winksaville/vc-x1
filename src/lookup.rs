//! `lookup` subcommand: resolve a line in either repo of a dual
//! workspace to the window of lines in the other repo.
//!
//! This is the command's edge: the `[SCOPE] FILE:LINE` arguments
//! parsed, the file placed on one side of the workspace, and the
//! line checked to exist. The window itself is what
//! `notes/transcript-write.md > What the lookup command needs`
//! specifies, and the placement is what every later step starts
//! from.
//!
//! - `LookupArgs`: clap surface, `SCOPE FILE:LINE` or `FILE:LINE`
//!   alone as positionals, `-s` as the scope's flag form, and `-R`
//!   for the workspace root.
//! - `LookupParams`: clap-free, the side when named, the file as
//!   given, and the line.
//! - `locate`: the placement, a `Located` naming the side, the
//!   repo, the repo-relative path, and the line's text.
//! - `lookup(&Context, &LookupParams)`: the op.

use std::path::{Path, PathBuf};

use clap::Args;
use log::info;

use crate::common;
use crate::context::Context;
use crate::options_flags::scope::{Side, side_keyword, side_keywords};
use crate::subcommand::SubcommandRunner;

/// CLI args for `lookup`.
#[derive(Args, Debug)]
pub struct LookupArgs {
    /// `SCOPE FILE:LINE`, or `FILE:LINE` alone: SCOPE is the side
    /// the line is on, `work` or `agent`, inferred from the path
    /// when omitted
    #[arg(value_name = "SCOPE|FILE:LINE")]
    pub first: String,

    /// `FILE:LINE` when the first argument is SCOPE
    #[arg(value_name = "FILE:LINE")]
    pub second: Option<String>,

    /// The side the line is on, the positional's flag form
    #[arg(short = 's', long = "scope", value_name = "SCOPE", value_parser = side_keywords())]
    pub scope: Option<Side>,

    /// Workspace root [default: the workspace around the file]
    #[arg(short = 'R', long = "repo", value_name = "PATH")]
    pub repo: Option<PathBuf>,
}

/// Clap-free params for `lookup`.
#[derive(Debug, PartialEq, Eq)]
pub struct LookupParams {
    /// The side the line is on when the caller named it.
    pub side: Option<Side>,
    /// The file as given, before placement resolves it.
    pub file: PathBuf,
    /// The 1-based line number.
    pub line: usize,
    /// The workspace root when `-R` gave one.
    pub root: Option<PathBuf>,
}

impl TryFrom<&LookupArgs> for LookupParams {
    type Error = String;

    /// Two positionals mean `SCOPE FILE:LINE`, one means
    /// `FILE:LINE`, and a positional SCOPE beside `-s` is an
    /// error rather than a tie-break.
    fn try_from(a: &LookupArgs) -> Result<Self, String> {
        let (side, target) = match &a.second {
            Some(target) => {
                if a.scope.is_some() {
                    return Err("lookup: SCOPE given twice, as a positional and as -s".into());
                }
                (Some(parse_side(&a.first)?), target.as_str())
            }
            None => (a.scope, a.first.as_str()),
        };
        let (file, line) = parse_file_line(target)?;
        Ok(LookupParams {
            side,
            file,
            line,
            root: a.repo.clone(),
        })
    }
}

impl SubcommandRunner for LookupArgs {
    type Params = LookupParams;

    fn to_params(&self) -> Result<Self::Params, String> {
        LookupParams::try_from(self)
    }

    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        lookup(ctx, params)
    }
}

/// One side's keyword as a `Side`, `work` or `agent` and nothing
/// wider, since a line is on one side.
fn parse_side(s: &str) -> Result<Side, String> {
    match s {
        "work" => Ok(Side::Work),
        "agent" => Ok(Side::Bot),
        "both" => {
            Err("lookup: SCOPE is one side, `work` or `agent`, since a line is on one".into())
        }
        other => Err(format!(
            "lookup: '{other}' is not a SCOPE. Expected `work` or `agent`, or FILE:LINE alone"
        )),
    }
}

/// Split `FILE:LINE` at its last colon, so a path holding a colon
/// still parses, and read LINE as a positive number.
pub fn parse_file_line(s: &str) -> Result<(PathBuf, usize), String> {
    let Some((file, line)) = s.rsplit_once(':') else {
        return Err(format!("lookup: '{s}' is not FILE:LINE"));
    };
    if file.is_empty() {
        return Err(format!("lookup: '{s}' has no FILE before the colon"));
    }
    let line: usize = line
        .parse()
        .map_err(|_| format!("lookup: '{s}': LINE is not a number"))?;
    if line == 0 {
        return Err(format!("lookup: '{s}': lines count from 1"));
    }
    Ok((PathBuf::from(file), line))
}

/// A line placed in the workspace: the side its file is on, that
/// side's repo, the path within it, and the line itself.
#[derive(Debug, PartialEq, Eq)]
pub struct Located {
    /// The side the file is on.
    pub side: Side,
    /// The repo root of that side, canonical.
    pub repo: PathBuf,
    /// The file's path relative to `repo`.
    pub rel: PathBuf,
    /// The 1-based line number.
    pub line: usize,
    /// The line's text, without its newline.
    pub text: String,
}

/// The workspace's two sides, canonical: the root and the agent
/// repo when the workspace has one.
struct Sides {
    root: PathBuf,
    bot: Option<PathBuf>,
}

impl Sides {
    /// The sides of the workspace at `root`, or the one around
    /// `near` when no root was given.
    fn find(root: Option<&Path>, near: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let root = match root {
            Some(r) => r
                .canonicalize()
                .map_err(|e| format!("cannot resolve repo path '{}': {e}", r.display()))?,
            None => common::find_workspace_root_from(near).ok_or_else(|| {
                format!(
                    "lookup: '{}' is not in a vc-x1 workspace, and -R names none",
                    near.display()
                )
            })?,
        };
        let bot = common::bot_repo_path(&root)?
            .map(|b| b.canonicalize())
            .transpose()?;
        Ok(Sides { root, bot })
    }

    /// The repo of `side`, an error for `agent` in a workspace with
    /// no agent repo.
    fn repo_of(&self, side: Side) -> Result<&Path, Box<dyn std::error::Error>> {
        match side {
            Side::Work => Ok(&self.root),
            Side::Bot => self.bot.as_deref().ok_or_else(|| {
                format!(
                    "lookup: the workspace at '{}' has no agent repo",
                    self.root.display()
                )
                .into()
            }),
        }
    }

    /// The side a canonical path is on, the agent repo checked
    /// first since it may nest in the work repo, or `None` outside
    /// both.
    fn side_of(&self, file: &Path) -> Option<Side> {
        if self.bot.as_deref().is_some_and(|b| file.starts_with(b)) {
            Some(Side::Bot)
        } else if file.starts_with(&self.root) {
            Some(Side::Work)
        } else {
            None
        }
    }
}

/// Place the line: resolve the file, find the side it is on, and
/// read the line.
///
/// - The file is taken as given, relative to `cwd`, when that
///   exists. Otherwise, with a side named, it is relative to that
///   side's repo root, so a repo-relative path from anywhere works
///   once the side is said.
/// - The side is where the file's canonical path lands. A named
///   side that disagrees is an error, never a hint.
/// - The line must exist in the file as it is on disk, since the
///   working copy is what an editor or a compiler printed the line
///   from.
pub fn locate(params: &LookupParams, cwd: &Path) -> Result<Located, Box<dyn std::error::Error>> {
    let given = if params.file.is_absolute() {
        params.file.clone()
    } else {
        cwd.join(&params.file)
    };
    let (sides, file) = if given.exists() {
        let file = given.canonicalize()?;
        let near = file.parent().unwrap_or(&file); // OK: a canonical file has a parent
        (Sides::find(params.root.as_deref(), near)?, file)
    } else if let Some(side) = params.side {
        let sides = Sides::find(params.root.as_deref(), cwd)?;
        let candidate = sides.repo_of(side)?.join(&params.file);
        if !candidate.exists() {
            return Err(format!(
                "lookup: '{}' is neither under '{}' nor under the {} repo '{}'",
                params.file.display(),
                cwd.display(),
                side_keyword(side),
                sides.repo_of(side)?.display()
            )
            .into());
        }
        let file = candidate.canonicalize()?;
        (sides, file)
    } else {
        return Err(format!(
            "lookup: no such file '{}'. A path relative to a repo root needs its SCOPE, \
             `work` or `agent`",
            params.file.display()
        )
        .into());
    };
    let Some(side) = sides.side_of(&file) else {
        return Err(format!(
            "lookup: '{}' is outside the workspace at '{}'",
            file.display(),
            sides.root.display()
        )
        .into());
    };
    if let Some(named) = params.side
        && named != side
    {
        return Err(format!(
            "lookup: '{}' is on the {} side, not {}",
            file.display(),
            side_keyword(side),
            side_keyword(named)
        )
        .into());
    }
    let repo = sides.repo_of(side)?.to_path_buf();
    let rel = file.strip_prefix(&repo)?.to_path_buf();
    let content = std::fs::read_to_string(&file)
        .map_err(|e| format!("lookup: cannot read '{}': {e}", file.display()))?;
    let count = content.lines().count();
    let Some(text) = content.lines().nth(params.line - 1) else {
        return Err(format!(
            "lookup: '{}' has {count} line(s), so there is no line {}",
            rel.display(),
            params.line
        )
        .into());
    };
    Ok(Located {
        side,
        repo,
        rel,
        line: params.line,
        text: text.to_string(),
    })
}

/// Place the line and print where it landed, `<side> <path>:<line>`
/// and the line's text.
pub fn lookup(_ctx: &Context, params: &LookupParams) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let at = locate(params, &cwd)?;
    info!("{} {}:{}", side_keyword(at.side), at.rel.display(), at.line);
    info!("    {}", at.text);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::Fixture;
    use clap::Parser;

    #[derive(Parser)]
    struct T {
        #[command(flatten)]
        a: LookupArgs,
    }

    fn params(args: &[&str]) -> Result<LookupParams, String> {
        let t = T::try_parse_from(args).map_err(|e| e.to_string())?;
        LookupParams::try_from(&t.a)
    }

    #[test]
    fn file_line_splits_at_the_last_colon() {
        assert_eq!(
            parse_file_line("TODO.md:53").unwrap(),
            (PathBuf::from("TODO.md"), 53)
        );
        assert_eq!(
            parse_file_line("a:b/c.rs:7").unwrap(),
            (PathBuf::from("a:b/c.rs"), 7)
        );
    }

    #[test]
    fn file_line_rejects_the_malformed() {
        for bad in ["TODO.md", ":5", "TODO.md:0", "TODO.md:x", "TODO.md:"] {
            assert!(parse_file_line(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn one_positional_is_the_target() {
        let p = params(&["t", "TODO.md:3"]).unwrap();
        assert_eq!(p.side, None);
        assert_eq!(p.file, PathBuf::from("TODO.md"));
        assert_eq!(p.line, 3);
        assert_eq!(p.root, None);
    }

    #[test]
    fn two_positionals_are_scope_then_target() {
        let p = params(&["t", "agent", "s.jsonl:9"]).unwrap();
        assert_eq!(p.side, Some(Side::Bot));
        assert_eq!(p.file, PathBuf::from("s.jsonl"));
        let p = params(&["t", "work", "TODO.md:1", "-R", "/x"]).unwrap();
        assert_eq!(p.side, Some(Side::Work));
        assert_eq!(p.root, Some(PathBuf::from("/x")));
    }

    #[test]
    fn scope_flag_and_positional_conflict() {
        let err = params(&["t", "work", "TODO.md:1", "-s", "agent"]).unwrap_err();
        assert!(err.contains("twice"), "{err}");
        let p = params(&["t", "TODO.md:1", "-s", "agent"]).unwrap();
        assert_eq!(p.side, Some(Side::Bot));
    }

    #[test]
    fn scope_is_one_side() {
        let err = params(&["t", "both", "TODO.md:1"]).unwrap_err();
        assert!(err.contains("one side"), "{err}");
        assert!(T::try_parse_from(["t", "TODO.md:1", "-s", "both"]).is_err());
        let err = params(&["t", "TODO.md", "TODO.md:1"]).unwrap_err();
        assert!(err.contains("not a SCOPE"), "{err}");
    }

    /// A dual workspace with one three-line file on each side.
    fn seeded(tag: &str) -> Fixture {
        let fx = Fixture::new(tag);
        std::fs::write(fx.work.join("notes.md"), "one\ntwo\nthree\n").unwrap();
        std::fs::write(fx.bot.join("s.jsonl"), "{\"a\":1}\n{\"a\":2}\n{\"a\":3}\n").unwrap();
        fx
    }

    fn p(side: Option<Side>, file: &Path, line: usize) -> LookupParams {
        LookupParams {
            side,
            file: file.to_path_buf(),
            line,
            root: None,
        }
    }

    #[test]
    fn locate_places_each_side_by_path() {
        let fx = seeded("lookup-place");
        let at = locate(&p(None, &fx.work.join("notes.md"), 2), &fx.base).unwrap();
        assert_eq!(at.side, Side::Work);
        assert_eq!(at.repo, fx.work.canonicalize().unwrap());
        assert_eq!(at.rel, PathBuf::from("notes.md"));
        assert_eq!(at.text, "two");
        let at = locate(&p(None, &fx.bot.join("s.jsonl"), 3), &fx.base).unwrap();
        assert_eq!(at.side, Side::Bot);
        assert_eq!(at.repo, fx.bot.canonicalize().unwrap());
        assert_eq!(at.text, "{\"a\":3}");
    }

    #[test]
    fn locate_takes_a_relative_path_from_cwd() {
        let fx = seeded("lookup-relative");
        let at = locate(&p(None, Path::new("notes.md"), 1), &fx.work).unwrap();
        assert_eq!(at.side, Side::Work);
        assert_eq!(at.text, "one");
    }

    #[test]
    fn locate_takes_a_repo_relative_path_when_the_side_is_named() {
        let fx = seeded("lookup-repo-relative");
        let at = locate(&p(Some(Side::Bot), Path::new("s.jsonl"), 1), &fx.work).unwrap();
        assert_eq!(at.side, Side::Bot);
        assert_eq!(at.rel, PathBuf::from("s.jsonl"));
        let err = locate(&p(None, Path::new("s.jsonl"), 1), &fx.work).unwrap_err();
        assert!(err.to_string().contains("needs its SCOPE"), "{err}");
    }

    #[test]
    fn locate_rejects_a_side_that_disagrees() {
        let fx = seeded("lookup-disagree");
        let err = locate(&p(Some(Side::Bot), &fx.work.join("notes.md"), 1), &fx.base).unwrap_err();
        assert!(
            err.to_string().contains("on the work side, not agent"),
            "{err}"
        );
    }

    #[test]
    fn locate_rejects_a_line_past_the_end() {
        let fx = seeded("lookup-past-end");
        let err = locate(&p(None, &fx.work.join("notes.md"), 4), &fx.base).unwrap_err();
        assert!(err.to_string().contains("3 line(s)"), "{err}");
    }

    #[test]
    fn locate_rejects_a_file_outside_the_workspace() {
        let fx = seeded("lookup-outside");
        let outside = fx.base.join("stray.md");
        std::fs::write(&outside, "x\n").unwrap();
        let err = locate(&p(None, &outside, 1), &fx.base).unwrap_err();
        assert!(
            err.to_string().contains("not in a vc-x1 workspace"),
            "{err}"
        );
    }
}

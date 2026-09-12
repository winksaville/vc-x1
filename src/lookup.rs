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

use jj_lib::commit::Commit;
use jj_lib::repo::Repo;

use crate::common;
use crate::context::Context;
use crate::options_flags::scope::{Side, side_keyword, side_keywords};
use crate::subcommand::SubcommandRunner;

pub mod blame;
pub mod partner;
pub mod window;

use partner::{Partner, short_chid, title};

/// The default tolerance, in seconds, inside which a commit with no
/// trailer takes the other side's commits as candidate partners.
const DEFAULT_TOLERANCE_SECS: i64 = 60;

/// The default number of timeline entries printed on each side of a
/// transcript write.
const DEFAULT_CONTEXT: usize = 3;

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

    /// Read the line from the file as of REV rather than from disk,
    /// for a line the working copy no longer holds
    #[arg(short = 'r', long = "revision", value_name = "REV")]
    pub revision: Option<String>,

    /// Seconds within which a commit with no ochid trailer takes the
    /// other side's commits as candidate partners
    #[arg(long = "tolerance", value_name = "SECS", default_value_t = DEFAULT_TOLERANCE_SECS)]
    pub tolerance: i64,

    /// Timeline entries to print on each side of the transcript write
    #[arg(short = 'C', long = "context", value_name = "N", default_value_t = DEFAULT_CONTEXT)]
    pub context: usize,
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
    /// The revision to read the line from, in place of disk.
    pub rev: Option<String>,
    /// The candidates' tolerance, in seconds.
    pub tolerance_secs: i64,
    /// Entries printed on each side of the transcript write.
    pub context: usize,
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
        if a.tolerance < 0 {
            return Err("lookup: --tolerance is a number of seconds, so not negative".into());
        }
        Ok(LookupParams {
            side,
            file,
            line,
            root: a.repo.clone(),
            rev: a.revision.clone(),
            tolerance_secs: a.tolerance,
            context: a.context,
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
    /// The other side's repo root, canonical, when the workspace has
    /// one.
    pub other: Option<PathBuf>,
    /// The file's path relative to `repo`.
    pub rel: PathBuf,
    /// The 1-based line number.
    pub line: usize,
    /// The line's text from disk, without its newline, or `None`
    /// when a revision is to supply it.
    pub text: Option<String>,
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

    /// The other side's repo, `None` for `work` in a workspace with
    /// no agent repo.
    fn other_of(&self, side: Side) -> Option<&Path> {
        match side {
            Side::Work => self.bot.as_deref(),
            Side::Bot => Some(&self.root),
        }
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
///   from, unless a revision is named, which supplies it from that
///   tree instead.
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
    let other = sides.other_of(side).map(Path::to_path_buf);
    let rel = file.strip_prefix(&repo)?.to_path_buf();
    let text = if params.rev.is_some() {
        None
    } else {
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
        Some(text.to_string())
    };
    Ok(Located {
        side,
        repo,
        other,
        rel,
        line: params.line,
        text,
    })
}

/// One commit for a revset that must name exactly one.
fn one_commit(
    workspace: &jj_lib::workspace::Workspace,
    repo: &std::sync::Arc<jj_lib::repo::ReadonlyRepo>,
    rev: &str,
) -> Result<Commit, Box<dyn std::error::Error>> {
    let ids = common::resolve_revset(workspace, repo, rev)?;
    match ids.as_slice() {
        [id] => Ok(repo.store().get_commit(id)?),
        other => Err(format!("lookup: '{rev}' names {} commits, not one", other.len()).into()),
    }
}

/// Print a partner answer under `label`: one line per linked commit
/// or candidate, or the one line saying there is none.
fn print_partner(label: &str, answer: &Partner) {
    match answer {
        Partner::Linked(commits) => {
            for c in commits {
                info!("{label} {} {}", short_chid(c), title(c));
            }
        }
        Partner::Candidates {
            tolerance_secs,
            commits,
        } if commits.is_empty() => {
            info!("{label} none: no ochid trailer, and no commit within {tolerance_secs}s");
        }
        Partner::Candidates { commits, .. } => {
            for c in commits {
                info!("{label} candidate {} {}", short_chid(c), title(c));
            }
        }
    }
}

/// A committed work line resolved to the agent side: the blame, the
/// partner of the commit that wrote it, that partner's window, and
/// the transcript write when one is found.
pub struct WorkResolution {
    /// The partner answer for the writer, the moved-from commit when
    /// the line moved, else the blamed commit.
    pub partner: Partner,
    /// The agent commit the window is read from: the one linked
    /// partner, or the nearest candidate.
    pub agent: Option<Commit>,
    /// The window's spans in the agent repo's session files.
    pub spans: Vec<window::Span>,
    /// The agent repo's timeline on disk.
    pub timeline: window::Timeline,
    /// The timeline position of the window's end.
    pub end: Option<usize>,
    /// The timeline position of the transcript write.
    pub write: Option<usize>,
}

/// Resolve a committed work line, blame's `origin` for it, to its
/// transcript write.
///
/// - The writer is the commit the line was first written in, the
///   reach back's answer when it moved, and its partner is where the
///   write's window is.
/// - The search runs over the whole timeline from that window's end,
///   backwards and then forwards, so a line set aside before an
///   earlier push or amended in after its own is still found.
pub fn resolve_work(
    origin: &blame::Origin,
    agent_root: &Path,
    rel: &Path,
    tolerance_secs: i64,
) -> Result<WorkResolution, Box<dyn std::error::Error>> {
    let (aws, arepo) = common::load_repo(agent_root)?;
    let writer = origin.written.as_ref().unwrap_or(&origin.commit);
    let partner = partner::partners(writer, (&aws, &arepo), tolerance_secs)?;
    let agent = match &partner {
        Partner::Linked(commits) => commits.first().cloned(),
        Partner::Candidates { commits, .. } => commits.first().cloned(),
    };
    let spans = match &agent {
        Some(a) => window::window_of(&arepo, a)?,
        None => Vec::new(),
    };
    let timeline = window::Timeline::load(agent_root)?;
    let end = timeline.window_end(&spans);
    let write = end.and_then(|e| timeline.find_write(e, rel, &origin.text));
    Ok(WorkResolution {
        partner,
        agent,
        spans,
        timeline,
        end,
        write,
    })
}

/// Print the window's spans, the transcript write, and the entries
/// around it.
fn print_window(r: &WorkResolution, rel: &Path, context: usize) {
    let candidate = matches!(r.partner, Partner::Candidates { .. });
    let label = if candidate {
        "candidate window"
    } else {
        "window"
    };
    for span in &r.spans {
        info!(
            "{label} {}:{}-{}",
            span.file.display(),
            span.start,
            span.end
        );
    }
    let Some(w) = r.write else {
        if r.agent.is_some() {
            info!(
                "write   none found: no Write, Edit, or Bash call on {} carries the line",
                rel.display()
            );
        }
        return;
    };
    let placed = &r.timeline.entries[w];
    let where_ = match r.end {
        Some(end) if w > end => " (after the window, a later amend)",
        _ if !r.spans.iter().any(|s| {
            s.file == placed.file && (s.start..=s.end).contains(&placed.entry.line_no)
        }) =>
        {
            " (before the window, written early)"
        }
        _ => "",
    };
    info!(
        "write   {}:{}{where_}",
        placed.file.display(),
        placed.entry.line_no
    );
    let from = w.saturating_sub(context);
    let to = (w + context).min(r.timeline.entries.len() - 1);
    for i in from..=to {
        let mark = if i == w { ">" } else { " " };
        info!("  {mark} {}", window::render(&r.timeline.entries[i]));
    }
}

/// Place the line and print where it landed, `<side> <path>:<line>`
/// and the line's text, then for a work line the commit it arrived
/// in, the commit that wrote it when it moved, the writer's partner,
/// that partner's window, and the transcript write.
///
/// The work repo is snapshotted first, as any jj command does, so a
/// line edited since the last snapshot blames to the working copy
/// rather than to a tree that lacks it.
pub fn lookup(ctx: &mut Context, params: &LookupParams) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let at = locate(params, &cwd)?;
    info!("{} {}:{}", side_keyword(at.side), at.rel.display(), at.line);
    if let Some(text) = &at.text {
        info!("    {text}");
    }
    if at.side != Side::Work {
        return Ok(());
    }
    if params.rev.is_none() {
        ctx.session(&at.repo)?.snapshot()?;
    }
    let (ws, repo) = common::load_repo(&at.repo)?;
    let wc_id = repo.view().get_wc_commit_id(ws.workspace_name()).cloned();
    let start = match &params.rev {
        Some(rev) => one_commit(&ws, &repo, rev)?,
        None => {
            let id = wc_id
                .clone()
                .ok_or("lookup: the work repo has no working-copy commit")?;
            repo.store().get_commit(&id)?
        }
    };
    let found = blame::origin(&ws, &repo, &start, &at.rel, at.line)?;
    match &at.text {
        Some(text) if *text != found.text => {
            return Err(format!(
                "lookup: line {} of '{}' on disk differs from the last snapshot, so run \
                 `jj status` to snapshot it first:\n    disk: {text}\n    tree: {}",
                at.line,
                at.rel.display(),
                found.text
            )
            .into());
        }
        Some(_) => {}
        None => info!("    {}", found.text),
    }
    if wc_id.as_ref() == Some(found.commit.id()) {
        info!("commit  the working copy, not committed yet, so no partner");
        return Ok(());
    }
    info!(
        "commit  {} {}:{} {}",
        short_chid(&found.commit),
        at.rel.display(),
        found.line_at_origin,
        title(&found.commit)
    );
    if let Some(w) = &found.written {
        info!("written {} {}", short_chid(w), title(w));
    }
    let Some(other) = &at.other else {
        info!("partner none: the workspace has no agent repo");
        return Ok(());
    };
    let r = resolve_work(&found, other, &at.rel, params.tolerance_secs)?;
    print_partner("partner", &r.partner);
    print_window(&r, &at.rel, params.context);
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
        assert_eq!(p.rev, None);
        assert_eq!(p.tolerance_secs, DEFAULT_TOLERANCE_SECS);
        let p = params(&["t", "TODO.md:3", "-r", "main", "--tolerance", "5"]).unwrap();
        assert_eq!(p.rev.as_deref(), Some("main"));
        assert_eq!(p.tolerance_secs, 5);
        assert!(params(&["t", "TODO.md:3", "--tolerance=-5"]).is_err());
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
            rev: None,
            tolerance_secs: DEFAULT_TOLERANCE_SECS,
            context: DEFAULT_CONTEXT,
        }
    }

    #[test]
    fn locate_places_each_side_by_path() {
        let fx = seeded("lookup-place");
        let at = locate(&p(None, &fx.work.join("notes.md"), 2), &fx.base).unwrap();
        assert_eq!(at.side, Side::Work);
        assert_eq!(at.repo, fx.work.canonicalize().unwrap());
        assert_eq!(at.rel, PathBuf::from("notes.md"));
        assert_eq!(at.text.as_deref(), Some("two"));
        assert_eq!(at.other, Some(fx.bot.canonicalize().unwrap()));
        let at = locate(&p(None, &fx.bot.join("s.jsonl"), 3), &fx.base).unwrap();
        assert_eq!(at.side, Side::Bot);
        assert_eq!(at.repo, fx.bot.canonicalize().unwrap());
        assert_eq!(at.text.as_deref(), Some("{\"a\":3}"));
        assert_eq!(at.other, Some(fx.work.canonicalize().unwrap()));
        let mut with_rev = p(None, &fx.work.join("notes.md"), 9);
        with_rev.rev = Some("@".into());
        assert_eq!(locate(&with_rev, &fx.base).unwrap().text, None);
    }

    #[test]
    fn locate_takes_a_relative_path_from_cwd() {
        let fx = seeded("lookup-relative");
        let at = locate(&p(None, Path::new("notes.md"), 1), &fx.work).unwrap();
        assert_eq!(at.side, Side::Work);
        assert_eq!(at.text.as_deref(), Some("one"));
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

    /// The dr-1 fixture's root and its relationships, or `None` when
    /// the fixture is absent.
    pub(super) fn dr1() -> Option<(PathBuf, serde_json::Value)> {
        let root = crate::test_helpers::fixture("dr-1")?;
        let text = std::fs::read_to_string(root.join("relationships.json")).unwrap();
        Some((root, serde_json::from_str(&text).unwrap()))
    }

    /// Every push's two change ids resolve to one commit each, the
    /// work commit carries the push's title, and each side's trailer
    /// names the other, except the pair made by hand.
    #[test]
    fn dr1_pushes_are_cross_linked() {
        use jj_lib::repo::Repo;
        let Some((root, rel)) = dr1() else { return };
        let (ww, wr) = common::load_repo(&root).unwrap();
        let (aw, ar) = common::load_repo(&root.join(".claude")).unwrap();
        for p in rel["pushes"].as_array().unwrap() {
            let work = p["work"].as_str().unwrap();
            let agent = p["agent"].as_str().unwrap();
            let wid = common::resolve_revset(&ww, &wr, work).unwrap();
            let aid = common::resolve_revset(&aw, &ar, agent).unwrap();
            assert_eq!(wid.len(), 1, "work {work}");
            assert_eq!(aid.len(), 1, "agent {agent}");
            let wc = wr.store().get_commit(&wid[0]).unwrap();
            let ac = ar.store().get_commit(&aid[0]).unwrap();
            assert_eq!(wc.description().lines().next(), p["title"].as_str());
            assert_eq!(ac.description().lines().next(), p["title"].as_str());
            let w_ochid = common::extract_ochid(&wc);
            let a_ochid = common::extract_ochid(&ac);
            if p["case"] == "no-trailer" {
                assert!(w_ochid.is_none() && a_ochid.is_none(), "{}", p["title"]);
            } else {
                let w_bare = crate::desc_helpers::extract_bare_id(w_ochid.as_deref().unwrap());
                let a_bare = crate::desc_helpers::extract_bare_id(a_ochid.as_deref().unwrap());
                assert!(
                    agent.starts_with(w_bare),
                    "{}: {w_bare} vs {agent}",
                    p["title"]
                );
                assert!(
                    work.starts_with(a_bare),
                    "{}: {a_bare} vs {work}",
                    p["title"]
                );
            }
        }
    }

    /// Every window lies within its session file, and every write is
    /// a tool call of the named tool on the named file at the named
    /// line.
    #[test]
    fn dr1_writes_are_where_the_record_says() {
        use crate::transcript::{ContentBlock, EntryKind, parse_str};
        let Some((root, rel)) = dr1() else { return };
        let agent = root.join(".claude");
        let session = |sid: &str| {
            let text = std::fs::read_to_string(agent.join(format!("{sid}.jsonl"))).unwrap();
            parse_str(&text)
        };
        for p in rel["pushes"].as_array().unwrap() {
            for w in p["window"].as_array().unwrap() {
                let t = session(w["session"].as_str().unwrap());
                let end = w["end"].as_u64().unwrap() as usize;
                assert!(t.malformed.is_empty());
                assert!(end <= t.entries.len(), "{}: {end}", p["title"]);
            }
        }
        for w in rel["writes"].as_array().unwrap() {
            let write = &w["write"];
            let t = session(write["session"].as_str().unwrap());
            let line = write["line"].as_u64().unwrap() as usize;
            let entry = t.entries.iter().find(|e| e.line_no == line).unwrap();
            let EntryKind::Assistant { content, .. } = &entry.kind else {
                panic!("{}: not an assistant line", w["text"]);
            };
            let [ContentBlock::ToolUse { name, input, .. }] = content.as_slice() else {
                panic!("{}: not one tool call", w["text"]);
            };
            assert_eq!(name, write["tool"].as_str().unwrap());
            let file = w["file"].as_str().unwrap();
            let names_it = match name.as_str() {
                "Bash" => input["command"].as_str().unwrap().contains(file),
                _ => input["file_path"].as_str().unwrap().ends_with(file),
            };
            assert!(names_it, "{}: {name} on {file}", w["text"]);
        }
    }

    /// From the tree that first carries each line, or the closing
    /// that moved it, the resolution finds the write the fixture
    /// recorded, the set-aside line's before its window and the
    /// no-trailer line's through the candidate window.
    #[test]
    fn dr1_every_committed_line_finds_its_transcript_write() {
        use crate::lookup::blame::lines_at;
        use crate::lookup::partner::tests::by_chid;
        let Some((root, rel)) = dr1() else { return };
        let (ws, repo) = common::load_repo(&root).unwrap();
        let agent_root = root.join(".claude");
        let pushes = rel["pushes"].as_array().unwrap();
        let mut checked = 0;
        for w in rel["writes"].as_array().unwrap() {
            let Some(arrives) = w["arrives"].as_u64() else {
                continue;
            };
            let at = w["moved_at"].as_u64().unwrap_or(arrives) as usize;
            let start = by_chid(&ws, &repo, pushes[at]["work"].as_str().unwrap());
            let file = Path::new(w["file"].as_str().unwrap());
            let text = w["text"].as_str().unwrap();
            let idx = lines_at(&start, file)
                .unwrap()
                .iter()
                .position(|l| l == text)
                .unwrap();
            let o = crate::lookup::blame::origin(&ws, &repo, &start, file, idx + 1).unwrap();
            let r = resolve_work(&o, &agent_root, file, 60).unwrap();
            let found = &r.timeline.entries[r.write.expect(text)];
            let want = &w["write"];
            assert_eq!(
                found.file,
                PathBuf::from(format!("{}.jsonl", want["session"].as_str().unwrap())),
                "{text}"
            );
            assert_eq!(
                found.entry.line_no as u64,
                want["line"].as_u64().unwrap(),
                "{text}"
            );
            checked += 1;
        }
        assert_eq!(checked, 10);
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

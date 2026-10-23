//! The `move-bookmark` subcommand: point one existing local bookmark
//! at one commit, locally, the repositioning a bookmark needs before
//! it is pushed, since `push` only ever sets the line's bookmark to
//! the work `@-`.
//!
//! - The meaning is `jj bookmark set <bookmark> -r <target>`: the
//!   bookmark must already exist, and the target is a revset naming
//!   exactly one commit, a bookmark or a rev.
//! - A move along the bookmark's line, the target at or above where
//!   it points, is the normal case. A move backward or sideways is
//!   refused unless `--allow-backwards`, as `jj` refuses it, and a
//!   bookmark already at the target is left as it is.
//! - Nothing is pushed: publishing is `push`'s, with its approval.
//! - `-s w` moves a bookmark in the work repo. `-s a` in the agent
//!   repo and `-s b` in both take `--yes`, since the agent repo's
//!   `main` is otherwise push's agent half's to set: the same name in
//!   each, the target resolved in each, every repo checked before any
//!   moves, and a failure part-way restoring the ones moved.
//! - `-s p` takes the repo from `-R`, required, as it is, with none of
//!   the workspace's checks, so it can move a bookmark in either side
//!   of a dual workspace or in any jj repo. Careful use is the user's.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info};

use crate::context::Context;
use crate::jj;
use crate::options_flags::dry_run::DryRunFlag;
use crate::options_flags::scope::{
    ALL_REPO_SCOPES, REPO_SCOPE_FORMS, RepoScope, Side, repo_scope_parser,
};
use crate::pair::{Snapshot, resolve_pair};
use crate::subcommand::SubcommandRunner;

/// Point an existing bookmark at one commit, locally.
#[derive(Args, Debug)]
pub struct MoveBookmarkArgs {
    /// The bookmark to move, which must already exist
    #[arg(long, value_name = "NAME", required = true)]
    pub bookmark: String,

    /// Where it moves to, a revset of exactly one commit, a bookmark
    /// or a rev, resolved in each repo of the scope
    #[arg(long, value_name = "REV", required = true)]
    pub target: String,

    /// Move it backward or sideways, as `jj` allows it
    #[arg(long)]
    pub allow_backwards: bool,

    /// Where: a path in the workspace for `-s w|a|b`, or the repo
    /// itself for `-s p`, required there [default: the current
    /// directory]
    #[arg(short = 'R', long, value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Which repos, required: `w|work`, `a|agent` or `b|both` with
    /// `--yes`, or `p|por` with `-R`
    #[arg(
        short = 's',
        long,
        value_name = "SCOPE",
        value_parser = repo_scope_parser(ALL_REPO_SCOPES),
        required = true,
        help = format!(
            "Which repos, required: {REPO_SCOPE_FORMS}, `a` and `b` with --yes, `p` with -R"
        )
    )]
    pub scope: RepoScope,

    /// Move a bookmark in the agent repo, `-s a` or `-s b`
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Print the move it would make, and change nothing
    #[command(flatten)]
    pub dry_run: DryRunFlag,
}

/// Inputs to the move-bookmark op, flat, owned, clap-free.
#[derive(Debug)]
pub struct MoveBookmarkParams {
    pub repos: Vec<PathBuf>,
    pub bookmark: String,
    pub target: String,
    pub allow_backwards: bool,
    pub dry_run: bool,
}

impl TryFrom<&MoveBookmarkArgs> for MoveBookmarkParams {
    type Error = String;

    /// Resolve the scope to its repos: `-R` as it is for `-s p`, the
    /// pair's sides for `-s w|a|b`, the agent side only with `--yes`.
    fn try_from(a: &MoveBookmarkArgs) -> Result<Self, String> {
        let repos = match &a.scope {
            RepoScope::Por => match &a.repo {
                Some(repo) => vec![repo.clone()],
                None => {
                    return Err("move-bookmark -s p: -R is required, the repo to move the \
                                bookmark in, taken as it is with no workspace checks"
                        .into());
                }
            },
            RepoScope::Sides(sides) => {
                if sides.has_bot() && !a.yes {
                    return Err("move-bookmark: -s a and -s b move a bookmark in the agent \
                                repo, whose `main` is otherwise push's agent half's to set. \
                                Pass --yes to move it"
                        .into());
                }
                let pair = resolve_pair(a.repo.as_deref()).map_err(|e| e.to_string())?;
                sides
                    .0
                    .iter()
                    .map(|side| match side {
                        Side::Work => pair.work.clone(),
                        Side::Bot => pair.agent.clone(),
                    })
                    .collect()
            }
        };
        Ok(MoveBookmarkParams {
            repos,
            bookmark: a.bookmark.clone(),
            target: a.target.clone(),
            allow_backwards: a.allow_backwards,
            dry_run: a.dry_run.value,
        })
    }
}

impl SubcommandRunner for MoveBookmarkArgs {
    type Params = MoveBookmarkParams;

    /// Delegate to the `TryFrom<&MoveBookmarkArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        MoveBookmarkParams::try_from(self)
    }

    /// Run the `move-bookmark` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        move_bookmark(ctx, params)
    }
}

/// One repo's move: where the bookmark points, a conflicted one at
/// several commits, and the commit it moves to.
#[derive(Debug)]
struct Move {
    repo: PathBuf,
    from: Vec<String>,
    to: String,
}

impl Move {
    /// True when the bookmark already points at the target alone.
    fn is_noop(&self) -> bool {
        self.from == [self.to.clone()]
    }

    /// The commits it points at, 12 characters each, joined.
    fn shown_from(&self) -> String {
        self.from
            .iter()
            .map(|id| short(id))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// The 12-character form of a commit id.
fn short(id: &str) -> String {
    id[..id.len().min(12)].to_string()
}

/// Check one repo's move, refusing a bookmark that does not exist, a
/// target that is not one commit, and a backward or sideways move
/// unless `allow_backwards`.
fn plan(
    repo: &Path,
    name: &str,
    target: &str,
    allow_backwards: bool,
) -> Result<Move, Box<dyn std::error::Error>> {
    if !jj::local_bookmark_exists(repo, name)? {
        return Err(format!(
            "move-bookmark: the bookmark '{name}' does not exist in {}, and move-bookmark \
             creates none",
            repo.display()
        )
        .into());
    }
    let to = match jj::cids_of(repo, target)?.as_slice() {
        [one] => one.clone(),
        other => {
            return Err(format!(
                "move-bookmark: the target '{target}' names {} commits in {}, and a bookmark \
                 points at exactly one",
                other.len(),
                repo.display()
            )
            .into());
        }
    };
    let at = format!("bookmarks(exact:{name:?})");
    let mv = Move {
        repo: repo.to_path_buf(),
        from: jj::cids_of(repo, &at)?,
        to,
    };
    // Forward when every commit the bookmark points at, a conflicted
    // one's included, is an ancestor of the target.
    let forward = !jj::matches(repo, &format!("{at} ~ ::{}", mv.to))?;
    if !mv.is_noop() && !forward && !allow_backwards {
        return Err(format!(
            "move-bookmark: moving '{name}' from {} to {} in {} is backward or sideways, \
             leaving commits off its line. Pass --allow-backwards to move it anyway",
            mv.shown_from(),
            short(&mv.to),
            repo.display()
        )
        .into());
    }
    Ok(mv)
}

/// Move the bookmark to the target in each repo of the scope, every
/// repo checked before any moves, and a failure restoring the repos
/// already moved.
pub fn move_bookmark(
    ctx: &mut Context,
    params: &MoveBookmarkParams,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("move-bookmark: enter {params:?}");
    let name = &params.bookmark;
    let mut moves = Vec::with_capacity(params.repos.len());
    for repo in &params.repos {
        moves.push(plan(repo, name, &params.target, params.allow_backwards)?);
    }
    let dry = if params.dry_run {
        "[dry-run] would move"
    } else {
        "moving"
    };
    let mut moved: Vec<Snapshot> = Vec::new();
    for mv in &moves {
        if mv.is_noop() {
            info!(
                "move-bookmark: '{name}' is already at {}, so nothing changes, in {}",
                short(&mv.to),
                mv.repo.display()
            );
            continue;
        }
        info!(
            "move-bookmark: {dry} '{name}' from {} to {}, locally, in {}",
            mv.shown_from(),
            short(&mv.to),
            mv.repo.display()
        );
        if params.dry_run {
            continue;
        }
        let snapshot = Snapshot::take(&mv.repo)?;
        if let Err(e) = ctx.session(&mv.repo)?.bookmark_set(name, &mv.to) {
            for s in moved.iter().rev() {
                s.restore("move-bookmark", e.as_ref());
            }
            return Err(e);
        }
        moved.push(snapshot);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{Fixture, jj_ok, test_ctx};
    use crate::{Cli, Commands};
    use clap::Parser;
    use std::fs;

    fn parse(args: &[&str]) -> Result<MoveBookmarkArgs, String> {
        match Cli::try_parse_from(args) {
            Ok(Cli {
                command: Some(Commands::MoveBookmark(a)),
                ..
            }) => Ok(a),
            Ok(_) => panic!("expected MoveBookmark"),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Parse `args` with `-R` set to `repo` and resolve them.
    fn resolve(args: &[&str], repo: Option<&Path>) -> Result<MoveBookmarkParams, String> {
        let mut a = parse(args).unwrap();
        a.repo = repo.map(Path::to_path_buf);
        MoveBookmarkParams::try_from(&a)
    }

    fn commit_id(repo: &Path, rev: &str) -> String {
        jj_ok(repo, &["log", "-r", rev, "--no-graph", "-T", "commit_id"])
    }

    fn params(repos: &[&Path], bookmark: &str, target: &str) -> MoveBookmarkParams {
        MoveBookmarkParams {
            repos: repos.iter().map(|r| r.to_path_buf()).collect(),
            bookmark: bookmark.to_string(),
            target: target.to_string(),
            allow_backwards: false,
            dry_run: false,
        }
    }

    /// Write `file` and commit it with `title` on `@`.
    fn commit_file(repo: &Path, file: &str, title: &str) {
        fs::write(repo.join(file), file).expect("write");
        jj_ok(repo, &["commit", "-m", title]);
    }

    /// `-s`, `--bookmark`, and `--target` are required.
    #[test]
    fn scope_bookmark_and_target_are_required() {
        let err = parse(&[
            "vc-x1",
            "move-bookmark",
            "--bookmark",
            "b",
            "--target",
            "@-",
        ])
        .unwrap_err();
        assert!(err.contains("--scope"), "{err}");
        let err = parse(&["vc-x1", "move-bookmark", "-s", "w", "--target", "@-"]).unwrap_err();
        assert!(err.contains("--bookmark"), "{err}");
        let err = parse(&["vc-x1", "move-bookmark", "-s", "w", "--bookmark", "b"]).unwrap_err();
        assert!(err.contains("--target"), "{err}");
    }

    /// `-s a` and `-s b` take `--yes`, and resolve to the agent repo
    /// and to both, and `-s p` takes `-R`, as it is, a dual
    /// workspace's side included.
    #[test]
    fn the_scopes_resolve_with_their_guards() {
        let fx = Fixture::new("mvbm-scopes");
        let base = [
            "vc-x1",
            "move-bookmark",
            "--bookmark",
            "main",
            "--target",
            "@-",
        ];
        let with = |extra: &[&'static str]| [&base[..], extra].concat();

        for scope in ["a", "b"] {
            let err = resolve(&with(&["-s", scope]), Some(&fx.work)).unwrap_err();
            assert!(err.contains("--yes"), "{scope}: {err}");
        }
        let a = resolve(&with(&["-s", "a", "--yes"]), Some(&fx.work)).expect("a");
        assert_eq!(a.repos, std::slice::from_ref(&fx.bot));
        let b = resolve(&with(&["-s", "b", "-y"]), Some(&fx.work)).expect("b");
        assert_eq!(b.repos, [fx.work.clone(), fx.bot.clone()]);
        let w = resolve(&with(&["-s", "w"]), Some(&fx.bot)).expect("w");
        assert_eq!(w.repos, std::slice::from_ref(&fx.work));

        let err = resolve(&with(&["-s", "p"]), None).unwrap_err();
        assert!(err.contains("-R is required"), "{err}");
        let p = resolve(&with(&["-s", "p"]), Some(&fx.work)).expect("p on a side");
        assert_eq!(p.repos, std::slice::from_ref(&fx.work));
    }

    /// A bookmark moves forward along its line to a rev or to another
    /// bookmark, locally, the remote left where it was, and a move to
    /// where it is changes nothing.
    #[test]
    fn a_bookmark_moves_forward_locally() {
        let fx = Fixture::new("mvbm-forward");
        let w = fx.work.as_path();
        jj_ok(w, &["bookmark", "create", "line", "-r", "main"]);
        commit_file(w, "one.txt", "feat: one");
        commit_file(w, "two.txt", "feat: two");
        let one = commit_id(w, "@--");
        let two = commit_id(w, "@-");
        let remote = commit_id(w, "main@origin");

        move_bookmark(&mut test_ctx(), &params(&[w], "line", "@--")).expect("to a rev");
        assert_eq!(commit_id(w, "line"), one);
        jj_ok(w, &["bookmark", "create", "tip", "-r", &two]);
        move_bookmark(&mut test_ctx(), &params(&[w], "line", "tip")).expect("to a bookmark");
        assert_eq!(commit_id(w, "line"), two);
        move_bookmark(&mut test_ctx(), &params(&[w], "main", "line")).expect("main forward");
        assert_eq!(commit_id(w, "main"), two);
        assert_eq!(commit_id(w, "main@origin"), remote);

        move_bookmark(&mut test_ctx(), &params(&[w], "line", &two)).expect("already there");
        assert_eq!(commit_id(w, "line"), two);
    }

    /// Both repos' `main` move, the target resolved in each, and a
    /// bookmark or a target missing from one stops the move before
    /// either moves.
    #[test]
    fn both_repos_move_or_neither() {
        let fx = Fixture::new("mvbm-both");
        let (w, a) = (fx.work.as_path(), fx.bot.as_path());
        commit_file(w, "w.txt", "feat: work");
        commit_file(a, "a.txt", "feat: agent");
        let (main_w, main_a) = (commit_id(w, "main"), commit_id(a, "main"));

        jj_ok(w, &["bookmark", "create", "line", "-r", "@-"]);
        let err = move_bookmark(&mut test_ctx(), &params(&[w, a], "line", "@-")).unwrap_err();
        assert!(err.to_string().contains("does not exist"), "{err}");
        // `line` resolves in the work repo, a forward move there, and in
        // no other, so neither moves.
        move_bookmark(&mut test_ctx(), &params(&[w, a], "main", "line")).unwrap_err();
        assert_eq!(
            (commit_id(w, "main"), commit_id(a, "main")),
            (main_w, main_a)
        );

        move_bookmark(&mut test_ctx(), &params(&[w, a], "main", "@-")).expect("both");
        assert_eq!(commit_id(w, "main"), commit_id(w, "@-"));
        assert_eq!(commit_id(a, "main"), commit_id(a, "@-"));
    }

    /// A backward or sideways move is refused, and taken with
    /// `--allow-backwards`.
    #[test]
    fn a_backward_or_sideways_move_takes_the_flag() {
        let fx = Fixture::new("mvbm-back");
        let w = fx.work.as_path();
        let base = commit_id(w, "main");
        commit_file(w, "one.txt", "feat: one");
        let one = commit_id(w, "@-");
        jj_ok(w, &["bookmark", "create", "line", "-r", &one]);
        jj_ok(w, &["new", &base]);
        commit_file(w, "side.txt", "feat: side");
        let side = commit_id(w, "@-");

        for to in [&base, &side] {
            let err = move_bookmark(&mut test_ctx(), &params(&[w], "line", to)).unwrap_err();
            assert!(err.to_string().contains("--allow-backwards"), "{err}");
            assert_eq!(commit_id(w, "line"), one);
        }
        let mut back = params(&[w], "line", &side);
        back.allow_backwards = true;
        move_bookmark(&mut test_ctx(), &back).expect("sideways, with the flag");
        assert_eq!(commit_id(w, "line"), side);
    }

    /// A bookmark that does not exist, a target of no commit or of
    /// several, are refused, and a dry run moves nothing.
    #[test]
    fn refusals_and_a_dry_run_change_nothing() {
        let fx = Fixture::new("mvbm-refused");
        let w = fx.work.as_path();
        jj_ok(w, &["bookmark", "create", "line", "-r", "main"]);
        commit_file(w, "one.txt", "feat: one");
        let at = commit_id(w, "line");

        let err = move_bookmark(&mut test_ctx(), &params(&[w], "absent", "@-")).unwrap_err();
        assert!(err.to_string().contains("creates none"), "{err}");
        for to in ["none()", "@- | @"] {
            let err = move_bookmark(&mut test_ctx(), &params(&[w], "line", to)).unwrap_err();
            assert!(err.to_string().contains("exactly one"), "{to}: {err}");
        }
        let mut dry = params(&[w], "line", "@-");
        dry.dry_run = true;
        move_bookmark(&mut test_ctx(), &dry).expect("dry run");
        assert_eq!(commit_id(w, "line"), at);
    }
}

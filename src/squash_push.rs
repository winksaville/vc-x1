//! The `squash-push` subcommand: squash the working copy into its
//! parent, advance a bookmark, and push: capture a repo's trailing
//! writes and publish them in one step.
//!
//! - Built for the bot's `.claude` bot repo, whose working copy
//!   accumulates session data continuously (the session tail). Also
//!   useful on the work repo as a deliberate amend-and-push (a
//!   published-history rewrite, so the push is a forced update).
//! - Runs synchronously: preflight validations, then squash +
//!   bookmark-set + push. A failure is a visible non-zero exit
//!   (the retired 0.69.0-2 predecessor delegated to a detached
//!   child that a sandboxed run silently killed: the loss
//!   diagnosed in 0.68.1).
//! - Prechecks and after-checks with one verdict: the working copy
//!   and the bookmark's publish state from `status`, plus this
//!   command's own question of whether the bookmark has reached the
//!   squash target. A run with nothing to do prints `<label>: clean`
//!   and stops, and every other run prints the line again after the
//!   push. The after-check reports only, since the exit code is the
//!   push's.
//! - Asks before acting, when asking is on. `squash-push.yes` sets
//!   the default and is true, so today's behavior is what a bare run
//!   gets, `--ask` turns the prompt on, and `--yes` turns it off,
//!   since a boolean flag cannot turn a configured true back off.
//!   With the prompt on, a non-tty stdin is an error rather than a
//!   hang, and a declined prompt is an error too.
//! - Reports an at-rest publish mismatch (BOOKMARK not matching
//!   `BOOKMARK@origin`, an earlier publish was lost) and proceeds:
//!   publishing is the command's job, so healing is not
//!   auto-fixing.
//! - Says nothing and asks nothing as push's `squash-push-bot`
//!   stage, which `at_rest` marks. Every line above is addressed to
//!   a person, and mid-push there is neither a person to read them
//!   nor a state worth describing. The precheck's decision still
//!   applies, so a stage with nothing to do still does nothing.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info, warn};

use crate::context::Context;
use crate::desc_helpers::extract_ochids;
use crate::jj;
use crate::options_flags::squash::{SquashOption, SquashSpec};
use crate::status;
use crate::subcommand::SubcommandRunner;

/// Squash `@` into `@-`, advance BOOKMARK, and push.
///
/// Captures a repo's trailing working-copy writes into the last
/// commit and publishes it: rewriting an already-pushed commit is
/// a forced update. Zero-ceremony default: bare `vc-x1 squash-push`
/// squashes `@ -> @-` and pushes `main` in `.`.
#[derive(Args, Debug)]
pub struct SquashPushArgs {
    /// Bookmark to advance and push [default: the bookmark of the
    /// line you are on, the nearest one at or above the squash
    /// target]
    #[arg(value_name = "BOOKMARK")]
    pub bookmark: Option<String>,

    /// Path to jj repo
    #[arg(short = 'R', long, default_value = ".")]
    pub repo: PathBuf,

    #[command(flatten)]
    pub squash: SquashOption,

    /// Act without asking, once the precheck has found work
    #[arg(short = 'y', long = "yes", conflicts_with = "ask")]
    pub yes: bool,

    /// Ask before acting, overriding a configured squash-push.yes
    #[arg(long = "ask")]
    pub ask: bool,
}

/// Per-invocation squash-push inputs: the clap-free shape the op
/// works against. Built from `SquashPushArgs` at the binary edge
/// via `TryFrom` (fallible only on repo-path canonicalization),
/// while `vc-x1 push`'s `squash-push-bot` stage constructs it
/// directly.
#[derive(Debug)]
pub struct SquashPushParams {
    pub repo: PathBuf,
    pub squash: SquashSpec,
    pub bookmark: String,
    /// This run is the user's at-rest invocation rather than
    /// `vc-x1 push`'s `squash-push-bot` stage.
    ///
    /// Everything addressed to a person is gated on it: the
    /// publish-mismatch report, the precheck's and after-check's
    /// lines, and the prompt. Mid-push every one of them is wrong.
    /// The mismatch is the normal state there, since `bookmark-set`
    /// just moved the bookmark and this stage is what publishes it.
    /// The verdict lines describe a repo caught mid-sequence. And
    /// nobody is watching to answer a prompt.
    ///
    /// What is not gated is the precheck's decision. A stage with
    /// nothing to do should still do nothing, which is the behavior
    /// the `already sync'd` early return gave push before the
    /// precheck replaced it.
    pub at_rest: bool,
    /// Act without asking once the precheck has found work. True is
    /// today's behavior and the built-in default, so the prompt is
    /// opt-in. Resolved from `--yes`, then `--ask`, then the repo's
    /// `squash-push.yes`, then the built-in.
    pub yes: bool,
}

/// The `yes` choice: the flags, then the config key, then the
/// built-in default.
///
/// The same order `agent-files diff` resolves its `--custom` by. A
/// boolean flag cannot turn a configured `true` back off, which is
/// why `--ask` exists rather than a `--no-yes`.
pub fn resolve_yes(yes: bool, ask: bool, key_value: Option<bool>, default: bool) -> bool {
    if yes {
        true
    } else if ask {
        false
    } else {
        key_value.unwrap_or(default)
    }
}

/// The `squash-push.yes` key of the config in `repo`, if any.
///
/// Read from the repo the command is pointed at rather than from the
/// workspace, so each side may answer differently and a plain repo
/// outside a workspace simply has no answer.
fn config_yes(repo: &Path) -> Result<Option<bool>, Box<dyn std::error::Error>> {
    let Some(cfg) = crate::config_md::load(repo)? else {
        return Ok(None);
    };
    match crate::toml_simple::toml_get(&cfg.map, "squash-push.yes").map(String::as_str) {
        None => Ok(None),
        Some("true") => Ok(Some(true)),
        Some("false") => Ok(Some(false)),
        Some(other) => Err(format!(
            "squash-push.yes: invalid bool {other:?}: expected true or false, unquoted"
        )
        .into()),
    }
}

impl TryFrom<&SquashPushArgs> for SquashPushParams {
    type Error = String;

    /// Canonicalize `--repo` (early, visible failure on a bad path)
    /// and fill the `--squash` default. BOOKMARK maps straight over.
    fn try_from(a: &SquashPushArgs) -> Result<Self, String> {
        let repo = std::fs::canonicalize(&a.repo)
            .map_err(|e| format!("cannot resolve repo path '{}': {e}", a.repo.display()))?;
        let yes = resolve_yes(
            a.yes,
            a.ask,
            config_yes(&repo).map_err(|e| e.to_string())?,
            crate::config_schema::SQUASH_PUSH_YES_DEFAULT,
        );
        let squash = a.squash.value.clone().unwrap_or_else(|| SquashSpec {
            source: "@".to_string(),
            target: "@-".to_string(),
        }); // OK: --squash absent -> the command's default @,@- pair
        let bookmark = match &a.bookmark {
            Some(b) => b.clone(),
            None => current_bookmark(&repo, &squash.target).map_err(|e| e.to_string())?,
        };
        Ok(SquashPushParams {
            repo,
            squash: squash.clone(),
            bookmark,
            at_rest: true,
            yes,
        })
    }
}

impl SubcommandRunner for SquashPushArgs {
    type Params = SquashPushParams;

    /// Delegate to the `TryFrom<&SquashPushArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        SquashPushParams::try_from(self)
    }

    /// Run the `squash-push` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        squash_push(ctx, params)
    }
}

/// Validate inputs before mutating anything.
///
/// Catches the common failure modes up front (unresolvable
/// revsets, an ochid-dropping squash, conflicts, a missing /
/// untracked / non-forward bookmark, an undescribed push target)
/// so the run fails before the squash rewrites history.
fn preflight(params: &SquashPushParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("preflight: checking params");
    let repo = &params.repo;
    let repo_str = params.repo.to_string_lossy();
    let sq = &params.squash;
    let bookmark = &params.bookmark;

    // Verify squash revsets resolve to something, and that the squash
    // won't drop source-only ochid: trailers.
    if !jj::rev_exists(repo, &sq.source)? {
        return Err(format!("squash source '{}' does not resolve", sq.source).into());
    }
    if !jj::rev_exists(repo, &sq.target)? {
        return Err(format!("squash target '{}' does not resolve", sq.target).into());
    }
    check_squash_keeps_ochids(repo, sq)?;

    // Refuse to operate on a repo with conflicts.
    if jj::matches(repo, "conflicts()")? {
        return Err(format!("repo '{repo_str}' has conflicts: resolve before squash-push").into());
    }

    // Bookmark: existence, tracking, forward-only move, push-target description.
    if !jj::local_bookmark_exists(repo, bookmark)? {
        return Err(format!("bookmark '{bookmark}' does not exist").into());
    }

    crate::common::verify_tracking(&params.repo, bookmark)?;

    if !jj::matches(repo, &format!("{bookmark}::({})", sq.target))? {
        return Err(format!(
            "bookmark '{bookmark}' move is not forward: current position is not an \
             ancestor of '{}' (would diverge)",
            sq.target
        )
        .into());
    }

    if jj::desc_of(repo, &sq.target)?.is_empty() {
        return Err(format!(
            "push target '{}' has no description: push would fail \
             (run `jj describe -r {} -R {repo_str}` first)",
            sq.target, sq.target
        )
        .into());
    }

    Ok(())
}

/// Return the `ochid:` trailer values present in `source_desc` but
/// absent from `target_desc`: the trailers a squash with
/// `--use-destination-message` would silently drop (the ochid-loss incident recorded in 0.65.1).
fn ochids_at_risk(source_desc: &str, target_desc: &str) -> Vec<String> {
    let kept = extract_ochids(target_desc);
    extract_ochids(source_desc)
        .into_iter()
        .filter(|ochid| !kept.contains(ochid))
        .collect()
}

/// Refuse a squash that would drop the source message's `ochid:`
/// trailers.
///
/// - Compares the two messages' `ochid:` trailers, and errors when the
///   source carries any the destination's message lacks:
///   `--use-destination-message` would discard them, leaving the
///   counterpart repo's cross-links dangling (recorded 0.65.1, guarded since 0.65.2).
fn check_squash_keeps_ochids(
    repo: &Path,
    sq: &SquashSpec,
) -> Result<(), Box<dyn std::error::Error>> {
    let at_risk = ochids_at_risk(
        &jj::desc_of(repo, &sq.source)?,
        &jj::desc_of(repo, &sq.target)?,
    );
    if at_risk.is_empty() {
        return Ok(());
    }
    let listed = at_risk
        .iter()
        .map(|ochid| format!("  {ochid}"))
        .collect::<Vec<_>>()
        .join("\n");
    Err(format!(
        "refusing squash {} -> {}: the squash would drop ochid: trailers\n\
         the destination's message lacks:\n\
         {listed}\n\
         merge the messages by hand (`jj describe {} -R {}`) or clear\n\
         the source's description, then retry",
        sq.source,
        sq.target,
        sq.target,
        repo.display(),
    )
    .into())
}

/// The bookmark a run defaults to: the one on the line it is on,
/// meaning the nearest bookmarked ancestor of the squash target.
///
/// There is no literal default any more. `main` was one, and it was
/// only ever right by coincidence. On the agent repo `main` is the
/// working line, so it was correct there. On a work repo running a
/// cycle `main` is deliberately behind, the cycle's commits living on
/// a topic bookmark, so a defaulted run there meant "advance main to
/// the cycle tip and publish it", which is Land's fast-forward step
/// done by accident. Observed 2026-09-17, which cost a rewind of
/// `main` on the remote.
///
/// The nearest bookmarked ancestor is "the branch you are on" in the
/// only sense jj affords, and it is right on both repos: the agent
/// side's last commit carries `main`, a cycle's ladder carries the
/// topic bookmark, and a local ladder's unbookmarked commits still
/// resolve past themselves to the topic bookmark rather than to
/// `main`.
///
/// Two cases are decided by the caller rather than guessed, since a
/// wrong guess here publishes something: several candidates, and
/// none.
fn current_bookmark(repo: &Path, target: &str) -> Result<String, Box<dyn std::error::Error>> {
    let nearest = format!("heads(::({target}) & bookmarks())");
    let mut found = jj::local_bookmarks_at(repo, &nearest)?;
    match found.len() {
        1 => Ok(found.remove(0)), // OK: len() == 1 checked by this arm
        0 => Err(format!(
            "no bookmark on this line: nothing at or above '{target}' in '{}' carries one,              so name the bookmark to advance",
            repo.display()
        )
        .into()),
        _ => Err(format!(
            "several bookmarks on this line at or above '{target}' in '{}': {},              so name the one to advance",
            repo.display(),
            found.join(", ")
        )
        .into()),
    }
}

/// The repo's label for a verdict line: its directory name, since
/// this command takes a path rather than a scope, and the name is
/// what tells two repos of one workspace apart.
fn label(repo: &Path) -> String {
    repo.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| repo.display().to_string())
}

/// One reading of the repo: the verdict `status` shares, plus
/// whether the bookmark has reached the squash target.
///
/// The third condition is this command's own. A bookmark behind its
/// squash target has a commit to publish even when it matches
/// origin, so a verdict-only test would skip exactly the work the
/// command exists to do.
#[derive(Debug)]
struct RunState {
    verdict: status::RepoVerdict,
    bookmark_at_target: bool,
}

impl RunState {
    /// Nothing for this run to do: at rest, published, and the
    /// bookmark already where the squash would leave it.
    fn nothing_to_do(&self) -> bool {
        self.why().is_none()
    }

    /// Why this run has work, `None` when it has none.
    fn why(&self) -> Option<String> {
        let mut parts: Vec<String> = self.verdict.why().into_iter().collect();
        if !self.bookmark_at_target {
            parts.push("bookmark behind the squash target".to_string());
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(", "))
        }
    }

    /// The verdict line, `<label>: clean` or `<label>: dirty: <why>`.
    fn line(&self, label: &str) -> String {
        match self.why() {
            None => format!("{label}: clean"),
            Some(why) => format!("{label}: dirty: {why}"),
        }
    }
}

/// Read the repo's state: the shared verdict over the bookmark, and
/// the bookmark against the squash target.
fn read_state(params: &SquashPushParams) -> Result<RunState, Box<dyn std::error::Error>> {
    let verdict = status::repo_verdict(&params.repo, Some(&params.bookmark))?;
    let bookmark_at_target = jj::cid_of(&params.repo, &params.bookmark)?
        == jj::cid_of(&params.repo, &params.squash.target)?;
    Ok(RunState {
        verdict,
        bookmark_at_target,
    })
}

/// Ask whether to act, when asking is on.
///
/// `yes` skips it, which is the default. With the prompt on, a
/// non-tty stdin (`stdin_is_tty`, from the `Context`) is an error
/// rather than a hang, the rule `push`'s step gate follows, and a
/// declined prompt is an error too, so a caller that scripted the
/// run learns it did not happen.
fn confirm(
    params: &SquashPushParams,
    state: &RunState,
    label: &str,
    stdin_is_tty: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !params.at_rest {
        debug!("squash-push: push's stage does not ask");
        return Ok(());
    }
    if params.yes {
        debug!("squash-push: --yes skips the prompt");
        return Ok(());
    }
    if !stdin_is_tty {
        return Err(
            "squash-push: asking requires a tty (stdin is not interactive); add --yes to act \
             without asking"
                .into(),
        );
    }
    let answer = crate::common::prompt(&format!("{}. squash-push? [y/N] ", state.line(label)))?;
    let normalized = answer.trim().to_ascii_lowercase();
    if normalized != "y" && normalized != "yes" {
        return Err(format!("squash-push: declined (got {answer:?})").into());
    }
    Ok(())
}

/// True when `rev` has no file changes and no description: nothing
/// worth squashing.
fn rev_is_empty_undescribed(repo: &Path, rev: &str) -> Result<bool, Box<dyn std::error::Error>> {
    if !jj::is_empty(repo, rev)? {
        return Ok(false);
    }
    Ok(jj::desc_of(repo, rev)?.is_empty())
}

/// Run the `squash-push` op: preflight, the precheck, the prompt,
/// then squash (skipped when the source is empty and undescribed) +
/// bookmark-set + push, then the after-check.
///
/// - The precheck stops a run with no work: the working copy at
///   rest, the bookmark at its origin, and the bookmark already at
///   the squash target. It prints `<label>: clean` and exits 0.
/// - With an empty source but work still to do, skips the squash
///   and still pushes.
/// - The after-check prints the state the run left and never fails,
///   so the exit code is the push's ([`RunState`]).
/// - As push's stage (`at_rest` false) the decision stands and every
///   line and the prompt go.
pub fn squash_push(
    ctx: &mut Context,
    params: &SquashPushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("squash_push: entry params={params:?}");
    let sq = &params.squash;
    let bookmark = &params.bookmark;

    preflight(params)?;

    // Report an at-rest publish mismatch before touching anything
    // (0.69.0-3): the bookmark should match its origin counterpart
    // between runs, so a mismatch means an earlier publish was
    // lost. Publishing is this command's job, so it proceeds: the
    // report is the point, not a refusal. Suppressed when run as
    // push's `squash-push-bot` stage (see `at_rest`).
    if params.at_rest {
        match crate::common::bookmark_publish_state(&params.repo, bookmark)? {
            crate::common::PublishState::InSync => {}
            crate::common::PublishState::NeverPushed => info!(
                "squash-push: '{bookmark}' has never been pushed to origin: this run will \
                 publish it"
            ),
            crate::common::PublishState::Mismatch { local, remote } => warn!(
                "squash-push: '{bookmark}' ({}) does not match '{bookmark}@origin' ({}): an \
                 earlier publish was likely lost; this run will publish it",
                &local[..local.len().min(12)],
                &remote[..remote.len().min(12)]
            ),
        }
    }

    // The precheck: nothing at rest, published, and already at the
    // squash target is a run with no work, so it says so and stops.
    // It subsumes the narrower "already sync'd" test this replaced,
    // whose three comparisons are the three the verdict now carries.
    let label = label(&params.repo);
    let before = read_state(params)?;
    if params.at_rest {
        info!("{}", before.line(&label));
    }
    if before.nothing_to_do() {
        return Ok(());
    }

    // The prompt sits between the precheck and the work, so it is
    // asked only when there is something to decline, and it names
    // what the precheck found.
    confirm(params, &before, &label, ctx.stdin_is_tty)?;

    // Empty-source handling: nothing to squash, but the precheck
    // found work, so the push still runs.
    if rev_is_empty_undescribed(&params.repo, &sq.source)? {
        info!(
            "squash-push: {} is empty, skipping squash, still pushing",
            sq.source
        );
    } else {
        info!("squash-push: squashing {} -> {}...", sq.source, sq.target);
        ctx.session(&params.repo)?
            .squash_into(&sq.source, &sq.target)?;
    }

    info!(
        "squash-push: setting bookmark '{bookmark}' to {}...",
        sq.target
    );
    ctx.session(&params.repo)?
        .bookmark_set(bookmark, &sq.target)?;

    info!("squash-push: pushing '{bookmark}' to origin...");
    ctx.session(&params.repo)?.git_push_bookmark(bookmark)?;

    info!("squash-push: done");

    // The after-check reports and never fails. The agent repo's
    // transcript grows while the push runs, so a correct push often
    // reads dirty a moment later, and the exit code says whether the
    // push completed rather than what this read found.
    if params.at_rest {
        info!("{}", read_state(params)?.line(&label));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::{Cli, Commands};
    use clap::Parser;

    fn parse(args: &[&str]) -> SquashPushArgs {
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::SquashPush(a)) => a,
            _ => panic!("expected SquashPush"),
        }
    }

    fn parse_err(args: &[&str]) -> String {
        Cli::try_parse_from(args).unwrap_err().to_string()
    }

    fn squash_at() -> SquashSpec {
        SquashSpec {
            source: "@".to_string(),
            target: "@-".to_string(),
        }
    }

    /// No bookmark given parses as none, the resolution being the
    /// repo's own answer rather than a literal clap can supply.
    #[test]
    fn no_args_defaults() {
        let args = parse(&["vc-x1", "squash-push"]);
        assert_eq!(args.bookmark, None);
        assert_eq!(args.repo, PathBuf::from("."));
        assert!(args.squash.value.is_none());
    }

    #[test]
    fn bookmark_positional() {
        let args = parse(&["vc-x1", "squash-push", "dev-0.14.0"]);
        assert_eq!(args.bookmark.as_deref(), Some("dev-0.14.0"));
        assert_eq!(args.repo, PathBuf::from("."));
    }

    #[test]
    fn all_opts() {
        let cli = Cli::try_parse_from([
            "vc-x1",
            "--log",
            "/tmp/test.log",
            "squash-push",
            "dev-0.14.0",
            "-R",
            ".claude",
            "--squash",
            "@,@-",
        ])
        .unwrap();
        assert_eq!(cli.log, Some(PathBuf::from("/tmp/test.log")));
        if let Some(Commands::SquashPush(args)) = cli.command {
            assert_eq!(args.bookmark.as_deref(), Some("dev-0.14.0"));
            assert_eq!(args.repo, PathBuf::from(".claude"));
            assert_eq!(args.squash.value, Some(squash_at()));
        } else {
            panic!("expected SquashPush");
        }
    }

    #[test]
    fn long_repo_flag() {
        let args = parse(&["vc-x1", "squash-push", "--repo", ".claude"]);
        assert_eq!(args.repo, PathBuf::from(".claude"));
    }

    #[test]
    fn bare_squash() {
        let args = parse(&["vc-x1", "squash-push", "--squash"]);
        assert_eq!(args.squash.value, Some(squash_at()));
    }

    #[test]
    fn bad_squash() {
        let err = parse_err(&["vc-x1", "squash-push", "--squash", "@"]);
        assert!(err.contains("expected SOURCE,TARGET"), "got: {err}");
    }

    #[test]
    fn retired_flags_rejected() {
        for flag in ["--detach", "--delay", "--exec", "--push"] {
            let err = parse_err(&["vc-x1", "squash-push", flag]);
            assert!(err.contains(flag), "flag {flag}: {err}");
        }
    }

    #[test]
    fn unknown_opt() {
        let err = parse_err(&["vc-x1", "squash-push", "--bogus"]);
        assert!(err.contains("--bogus"));
    }

    #[test]
    fn try_from_canonicalizes_and_defaults() {
        use crate::test_helpers::Fixture;

        let fx = Fixture::new("sp-try-from");
        let args = SquashPushArgs {
            bookmark: None,
            repo: fx.bot.clone(),
            squash: crate::options_flags::squash::SquashOption { value: None },
            yes: false,
            ask: false,
        };
        let params = SquashPushParams::try_from(&args).expect("params");
        assert_eq!(
            params.repo,
            std::fs::canonicalize(&fx.bot).expect("canonical")
        );
        assert_eq!(params.squash, squash_at());
        assert!(params.at_rest, "a CLI invocation runs at rest");
        // The agent side's last commit carries `main`, so the line's
        // bookmark is `main` and the old literal default's answer is
        // reached by reading the repo instead of assuming it.
        assert_eq!(params.bookmark, "main");
    }

    /// The default bookmark is the line's, not a literal: on a work
    /// repo whose cycle runs on a topic bookmark it resolves to that
    /// bookmark and never to `main`, which is the bug that landed a
    /// cycle early on 2026-09-17.
    #[test]
    fn the_default_bookmark_is_the_line_not_main() {
        use crate::test_helpers::{Fixture, jj_ok};

        let fx = Fixture::new("sp-line-bookmark");
        assert_eq!(current_bookmark(&fx.work, "@-").expect("on main"), "main");

        // Open a topic bookmark the way a cycle does, and commit on
        // it, leaving `main` behind.
        jj_ok(&fx.work, &["git", "push", "--named", "topic=@-"]);
        std::fs::write(fx.work.join("rung.txt"), "work\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "a rung"]);
        jj_ok(&fx.work, &["bookmark", "set", "topic", "-r", "@-"]);
        assert_eq!(current_bookmark(&fx.work, "@-").expect("on topic"), "topic");

        // A local ladder's unbookmarked commits still resolve past
        // themselves to the topic bookmark rather than to `main`.
        jj_ok(&fx.work, &["new"]);
        std::fs::write(fx.work.join("ladder.txt"), "scratch\n").expect("write");
        jj_ok(&fx.work, &["commit", "-m", "a local ladder commit"]);
        assert_eq!(
            current_bookmark(&fx.work, "@-").expect("on ladder"),
            "topic"
        );
    }

    /// Several candidates and none are decided by the caller, since a
    /// wrong guess publishes something.
    #[test]
    fn an_ambiguous_or_absent_line_is_refused() {
        use crate::test_helpers::{Fixture, jj_ok};

        let fx = Fixture::new("sp-line-ambiguous");
        jj_ok(&fx.work, &["bookmark", "create", "second", "-r", "@-"]);
        let err = current_bookmark(&fx.work, "@-")
            .expect_err("two bookmarks on one commit")
            .to_string();
        assert!(err.contains("several bookmarks"), "{err}");
        assert!(err.contains("main") && err.contains("second"), "{err}");

        let err = current_bookmark(&fx.work, "root()")
            .expect_err("nothing bookmarked at or above root")
            .to_string();
        assert!(err.contains("no bookmark on this line"), "{err}");
    }

    /// A lost publish (`main` moved without a push) is healed by a
    /// bare squash-push run: it reports the mismatch and publishes,
    /// leaving `main == main@origin`.
    #[test]
    fn squash_push_publishes_unpushed_bookmark_move() {
        use crate::test_helpers::{Fixture, jj_ok};

        let fx = Fixture::new("sp-heal");
        std::fs::write(fx.bot.join("lost.txt"), "lost session data\n").expect("write lost file");
        jj_ok(&fx.bot, &["commit", "-m", "lost bot commit"]);
        jj_ok(&fx.bot, &["bookmark", "set", "main", "-r", "@-"]);

        let params = SquashPushParams {
            repo: fx.bot.clone(),
            squash: squash_at(),
            bookmark: "main".to_string(),
            at_rest: true,
            yes: true,
        };
        squash_push(&mut crate::test_helpers::test_ctx(), &params)
            .expect("squash-push should publish the lost commit");

        let cid = |rev: &str| {
            jj_ok(
                &fx.bot,
                &["log", "-r", rev, "--no-graph", "-T", "commit_id"],
            )
        };
        assert_eq!(cid("main"), cid("main@origin"), "main should be published");
    }

    /// The precheck: a repo at rest, published, and with its
    /// bookmark already at the squash target has no work, so the
    /// run says `<label>: clean` and pushes nothing.
    #[test]
    fn precheck_stops_a_run_with_no_work() {
        use crate::test_helpers::{Fixture, jj_ok};

        let fx = Fixture::new("sp-precheck-clean");
        let params = SquashPushParams {
            repo: fx.bot.clone(),
            squash: squash_at(),
            bookmark: "main".to_string(),
            at_rest: true,
            yes: true,
        };
        let state = read_state(&params).expect("read state");
        assert!(state.nothing_to_do(), "{state:?}");
        assert_eq!(state.line("bot"), "bot: clean");

        let op_before = jj_ok(&fx.bot, &["op", "log", "--no-graph", "-T", "id", "-n", "1"]);
        squash_push(&mut crate::test_helpers::test_ctx(), &params).expect("clean run");
        assert_eq!(
            jj_ok(&fx.bot, &["op", "log", "--no-graph", "-T", "id", "-n", "1"]),
            op_before,
            "a clean run touches nothing"
        );
    }

    /// The third condition is this command's own: a bookmark behind
    /// its squash target has a commit to publish even though the
    /// working copy is at rest and the bookmark matches origin, so
    /// the precheck must not call that clean.
    #[test]
    fn a_bookmark_behind_the_target_is_not_clean() {
        use crate::test_helpers::{Fixture, jj_ok};

        let fx = Fixture::new("sp-behind-target");
        std::fs::write(fx.bot.join("session.txt"), "data\n").expect("write");
        jj_ok(
            &fx.bot,
            &["commit", "-m", "a commit the bookmark has not reached"],
        );

        let params = SquashPushParams {
            repo: fx.bot.clone(),
            squash: squash_at(),
            bookmark: "main".to_string(),
            at_rest: true,
            yes: true,
        };
        let state = read_state(&params).expect("read state");
        // `@` is empty and undescribed and `main` matches origin, so
        // the shared verdict alone would read clean.
        assert_eq!(state.verdict.why(), None);
        assert!(!state.bookmark_at_target);
        assert!(!state.nothing_to_do());
        assert_eq!(
            state.line("bot"),
            "bot: dirty: bookmark behind the squash target"
        );

        squash_push(&mut crate::test_helpers::test_ctx(), &params).expect("publishes the commit");
        let cid = |rev: &str| {
            jj_ok(
                &fx.bot,
                &["log", "-r", rev, "--no-graph", "-T", "commit_id"],
            )
        };
        assert_eq!(cid("main"), cid("main@origin"), "the commit is published");
        assert!(
            read_state(&params).expect("after").nothing_to_do(),
            "and the repo is left with nothing to do"
        );
    }

    /// A dirty working copy is named by the line the precheck and
    /// the after-check share.
    #[test]
    fn a_dirty_working_copy_is_named_then_left_clean() {
        use crate::test_helpers::{Fixture, jj_ok};

        let fx = Fixture::new("sp-dirty-wc");
        std::fs::write(fx.bot.join("tail.txt"), "session tail\n").expect("write");

        let params = SquashPushParams {
            repo: fx.bot.clone(),
            squash: squash_at(),
            bookmark: "main".to_string(),
            at_rest: true,
            yes: true,
        };
        let before = read_state(&params).expect("read state");
        assert_eq!(before.line("bot"), "bot: dirty: @ has changes");

        squash_push(&mut crate::test_helpers::test_ctx(), &params).expect("squash and push");
        let cid = |rev: &str| {
            jj_ok(
                &fx.bot,
                &["log", "-r", rev, "--no-graph", "-T", "commit_id"],
            )
        };
        assert_eq!(cid("main"), cid("main@origin"));
        assert!(read_state(&params).expect("after").nothing_to_do());
    }

    /// Flag, then key, then default, in that order, and `-y` with
    /// `--ask` is refused so the two cannot disagree.
    #[test]
    fn yes_resolution_order() {
        assert!(resolve_yes(true, false, Some(false), false));
        assert!(!resolve_yes(false, true, Some(true), true));
        assert!(resolve_yes(false, false, Some(true), false));
        assert!(!resolve_yes(false, false, Some(false), true));
        assert!(resolve_yes(false, false, None, true));
        assert!(!resolve_yes(false, false, None, false));
        assert!(Cli::try_parse_from(["vc-x1", "squash-push", "-y", "--ask"]).is_err());
        let a = parse(&["vc-x1", "squash-push", "--yes"]);
        assert!(a.yes && !a.ask);
        let a = parse(&["vc-x1", "squash-push", "--ask"]);
        assert!(a.ask && !a.yes);
    }

    /// The built-in default is to act without asking, so today's
    /// behavior is what a bare invocation still gets.
    ///
    /// The params are built against a fixture rather than the
    /// developer's checkout. `try_from` resolves the line's bookmark
    /// now, and a checkout with a cycle in flight carries both `main`
    /// and the cycle's bookmark on one line, an ambiguity the answer
    /// this test asks for does not depend on. Its sibling
    /// `try_from_canonicalizes_and_defaults` moved onto a fixture for
    /// the same reason (0.84.10) and this one was missed.
    #[test]
    fn the_default_is_to_act_without_asking() {
        use crate::test_helpers::Fixture;

        let fx = Fixture::new("sp-default-yes");
        let mut args = parse(&["vc-x1", "squash-push"]);
        args.repo = fx.bot.clone();
        let params = SquashPushParams::try_from(&args).expect("params");
        assert!(params.yes, "a bare run does not prompt");
    }

    /// A repo's own config answers, and a malformed value is an
    /// error naming the key rather than a silent default.
    #[test]
    fn the_repo_config_answers_and_a_bad_value_errors() {
        use crate::test_helpers::Fixture;

        let fx = Fixture::new("sp-config-yes");
        let cfg = fx.bot.join(crate::config_md::VC_CONFIG_MD);
        let base = std::fs::read_to_string(&cfg).unwrap_or_default();

        let with_key = |v: &str| format!("{base}\n```toml\n[squash-push]\nyes = {v}\n```\n");
        std::fs::write(&cfg, with_key("false")).expect("write config");
        assert_eq!(config_yes(&fx.bot).expect("read"), Some(false));

        std::fs::write(&cfg, with_key("\"no\"")).expect("write config");
        let err = config_yes(&fx.bot).expect_err("bad bool").to_string();
        assert!(err.contains("squash-push.yes"), "{err}");
        assert!(err.contains("\"no\""), "{err}");

        std::fs::write(&cfg, base).expect("restore config");
        assert_eq!(config_yes(&fx.bot).expect("read"), None);
    }

    /// With asking on and no tty, the run errors rather than
    /// hanging on `read_line`, the rule push's step gate follows.
    #[test]
    fn asking_without_a_tty_errors_rather_than_hangs() {
        use crate::test_helpers::Fixture;

        let fx = Fixture::new("sp-ask-no-tty");
        std::fs::write(fx.bot.join("tail.txt"), "session tail\n").expect("write");
        let params = SquashPushParams {
            repo: fx.bot.clone(),
            squash: squash_at(),
            bookmark: "main".to_string(),
            at_rest: true,
            yes: false,
        };
        let state = read_state(&params).expect("read state");
        // The test states no tty rather than inheriting `cargo
        // test`'s stdin, which is a terminal when run from one.
        let err = confirm(&params, &state, "bot", false)
            .expect_err("no tty and asking")
            .to_string();
        assert!(err.contains("requires a tty"), "{err}");
        assert!(err.contains("--yes"), "{err}");

        // And the whole run refuses for the same reason, before it
        // has touched anything.
        let err = squash_push(&mut crate::test_helpers::test_ctx(), &params)
            .expect_err("run refuses")
            .to_string();
        assert!(err.contains("requires a tty"), "{err}");
    }

    /// A clean run never reaches the prompt, so asking is on and the
    /// run still succeeds without a tty.
    #[test]
    fn a_clean_run_never_asks() {
        use crate::test_helpers::Fixture;

        let fx = Fixture::new("sp-clean-no-ask");
        let params = SquashPushParams {
            repo: fx.bot.clone(),
            squash: squash_at(),
            bookmark: "main".to_string(),
            at_rest: true,
            yes: false,
        };
        squash_push(&mut crate::test_helpers::test_ctx(), &params)
            .expect("nothing to decline, so nothing is asked");
    }

    /// push's stage asks nothing even with asking on and no tty,
    /// which at rest is an error. The prompt is a person's question
    /// and mid-push there is no person.
    #[test]
    fn pushs_stage_never_asks() {
        use crate::test_helpers::Fixture;

        let fx = Fixture::new("sp-stage-silent");
        std::fs::write(fx.bot.join("tail.txt"), "session tail\n").expect("write");
        let stage = SquashPushParams {
            repo: fx.bot.clone(),
            squash: squash_at(),
            bookmark: "main".to_string(),
            at_rest: false,
            yes: false,
        };
        let state = read_state(&stage).expect("read state");
        confirm(&stage, &state, "bot", false).expect("push's stage does not ask");

        // The same params at rest are the error rung 3 introduced,
        // so it is `at_rest` doing the work here and not `yes`.
        let at_rest = SquashPushParams {
            at_rest: true,
            ..stage
        };
        assert!(
            confirm(&at_rest, &state, "bot", false).is_err(),
            "at rest it asks"
        );

        squash_push(&mut crate::test_helpers::test_ctx(), &at_rest)
            .expect_err("and the at-rest run refuses without a tty");
    }

    /// The precheck's decision is not gated: a stage with nothing to
    /// do still does nothing, which is what the `already sync'd`
    /// early return gave push before the precheck replaced it.
    #[test]
    fn pushs_stage_still_skips_a_run_with_no_work() {
        use crate::test_helpers::{Fixture, jj_ok};

        let fx = Fixture::new("sp-stage-noop");
        let stage = SquashPushParams {
            repo: fx.bot.clone(),
            squash: squash_at(),
            bookmark: "main".to_string(),
            at_rest: false,
            yes: true,
        };
        assert!(read_state(&stage).expect("state").nothing_to_do());

        let op = || jj_ok(&fx.bot, &["op", "log", "--no-graph", "-T", "id", "-n", "1"]);
        let before = op();
        squash_push(&mut crate::test_helpers::test_ctx(), &stage).expect("no-op stage");
        assert_eq!(before, op(), "a stage with no work touches nothing");
    }

    /// The label is the repo's directory name, since the command
    /// takes a path rather than a scope.
    #[test]
    fn label_is_the_directory_name() {
        assert_eq!(label(Path::new("/a/b/.agent-session")), ".agent-session");
        assert_eq!(label(Path::new("/a/b/work")), "work");
    }

    #[test]
    fn ochids_at_risk_detects_source_only() {
        let source = "journal\n\nochid: /aaa\nochid: /bbb\n";
        let target = "previous journal\n\nochid: /aaa\n";
        assert_eq!(ochids_at_risk(source, target), vec!["/bbb"]);
    }

    #[test]
    fn ochids_at_risk_empty_cases() {
        // Undescribed source (the normal squash-push snapshot) is safe.
        assert!(ochids_at_risk("", "prev\n\nochid: /aaa\n").is_empty());
        // Source trailers all present in the destination are safe.
        let both = "msg\n\nochid: /aaa\n";
        assert!(ochids_at_risk(both, both).is_empty());
        // Source without trailers is safe regardless of destination.
        assert!(ochids_at_risk("described, no trailers\n", "prev\n").is_empty());
    }
}

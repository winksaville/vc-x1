//! The `squash-push` subcommand: squash the working copy into its
//! parent, advance a bookmark, and push — capture a repo's trailing
//! writes and publish them in one step.
//!
//! - Built for the bot's `.claude` bot repo, whose working copy
//!   accumulates session data continuously (the session tail); also
//!   useful on the work repo as a deliberate amend-and-push (a
//!   published-history rewrite, so the push is a forced update).
//! - Runs fully in-process: preflight validations, then squash +
//!   bookmark-set + push. A failure is a visible non-zero exit
//!   (the retired 0.69.0-2 predecessor delegated to a detached
//!   child that a sandboxed run silently killed — Bugs #1).
//! - Reports an at-rest publish mismatch (BOOKMARK not matching
//!   `BOOKMARK@origin` — an earlier publish was lost) and proceeds:
//!   publishing is the command's job, so healing is not
//!   auto-fixing. Suppressed when run as push's `squash-push-bot`
//!   stage, where the mismatch is the normal mid-push state.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info, warn};

use crate::common::run;
use crate::context::Context;
use crate::options_flags::squash::{SquashOption, SquashSpec};
use crate::subcommand::SubcommandRunner;

/// Squash `@` into `@-`, advance BOOKMARK, and push.
///
/// Captures a repo's trailing working-copy writes into the last
/// commit and publishes it — rewriting an already-pushed commit is
/// a forced update. Zero-ceremony default: bare `vc-x1 squash-push`
/// squashes `@ → @-` and pushes `main` in `.`.
#[derive(Args, Debug)]
pub struct SquashPushArgs {
    /// Bookmark to advance and push
    #[arg(value_name = "BOOKMARK", default_value = "main")]
    pub bookmark: String,

    /// Path to jj repo
    #[arg(short = 'R', long, default_value = ".")]
    pub repo: PathBuf,

    #[command(flatten)]
    pub squash: SquashOption,
}

/// Per-invocation squash-push inputs — the clap-free shape the op
/// works against. Built from `SquashPushArgs` at the binary edge
/// via `TryFrom` (fallible only on repo-path canonicalization);
/// `vc-x1 push`'s `squash-push-bot` stage constructs it
/// directly.
#[derive(Debug)]
pub struct SquashPushParams {
    pub repo: PathBuf,
    pub squash: SquashSpec,
    pub bookmark: String,
    /// Report an at-rest publish mismatch (BOOKMARK not matching
    /// `BOOKMARK@origin` — an earlier publish was lost) before
    /// proceeding. True for CLI invocations, which run at rest;
    /// `vc-x1 push`'s `squash-push-bot` stage sets false — there
    /// the mismatch is the normal mid-push state (`bookmark-set`
    /// just moved the bookmark, this stage publishes it), so the
    /// report would be a false alarm.
    pub report_publish_state: bool,
}

impl TryFrom<&SquashPushArgs> for SquashPushParams {
    type Error = String;

    /// Canonicalize `--repo` (early, visible failure on a bad path)
    /// and fill the `--squash` default; BOOKMARK maps straight over.
    fn try_from(a: &SquashPushArgs) -> Result<Self, String> {
        let repo = std::fs::canonicalize(&a.repo)
            .map_err(|e| format!("cannot resolve repo path '{}': {e}", a.repo.display()))?;
        Ok(SquashPushParams {
            repo,
            squash: a.squash.value.clone().unwrap_or_else(|| SquashSpec {
                source: "@".to_string(),
                target: "@-".to_string(),
            }), // OK: --squash absent → the command's default @,@- pair
            bookmark: a.bookmark.clone(),
            report_publish_state: true,
        })
    }
}

impl SubcommandRunner for SquashPushArgs {
    type Params = SquashPushParams;

    /// Delegate to the `TryFrom<&SquashPushArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        SquashPushParams::try_from(self)
    }

    /// Run the `squash-push` op (`ctx` unused — the op is fully
    /// parameterized by `Params`).
    fn run(_ctx: &Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        squash_push(params)
    }
}

/// Validate inputs before mutating anything.
///
/// Catches the common failure modes up front — unresolvable
/// revsets, an ochid-dropping squash, conflicts, a missing /
/// untracked / non-forward bookmark, an undescribed push target —
/// so the run fails before the squash rewrites history.
fn preflight(params: &SquashPushParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("preflight: checking params");
    let repo_str = params.repo.to_string_lossy();
    let cwd = std::path::Path::new(".");
    let sq = &params.squash;
    let bookmark = &params.bookmark;

    // Verify squash revsets resolve to something, and that the squash
    // won't drop source-only ochid: trailers.
    jj_rev_exists(&repo_str, cwd, &sq.source)
        .map_err(|e| format!("squash source '{}' does not resolve: {e}", sq.source))?;
    jj_rev_exists(&repo_str, cwd, &sq.target)
        .map_err(|e| format!("squash target '{}' does not resolve: {e}", sq.target))?;
    check_squash_keeps_ochids(&repo_str, cwd, sq)?;

    // Refuse to operate on a repo with conflicts.
    let conflicts = run(
        "jj",
        &[
            "log",
            "-r",
            "conflicts()",
            "--no-graph",
            "-T",
            "\"x\"",
            "-R",
            &repo_str,
        ],
        cwd,
    )?;
    if !conflicts.is_empty() {
        return Err(format!("repo '{repo_str}' has conflicts — resolve before squash-push").into());
    }

    // Bookmark: existence, tracking, forward-only move, push-target description.
    let exists = run("jj", &["bookmark", "list", bookmark, "-R", &repo_str], cwd)?;
    if exists.is_empty() {
        return Err(format!("bookmark '{bookmark}' does not exist").into());
    }

    crate::common::verify_tracking(&params.repo, bookmark)?;

    let range = run(
        "jj",
        &[
            "log",
            "-r",
            &format!("{bookmark}::({})", sq.target),
            "--no-graph",
            "-T",
            "\"x\"",
            "-R",
            &repo_str,
        ],
        cwd,
    )?;
    if range.is_empty() {
        return Err(format!(
            "bookmark '{bookmark}' move is not forward — current position is not an \
             ancestor of '{}' (would diverge)",
            sq.target
        )
        .into());
    }

    let desc = run(
        "jj",
        &[
            "log",
            "-r",
            &sq.target,
            "--no-graph",
            "-T",
            "description",
            "-R",
            &repo_str,
        ],
        cwd,
    )?;
    if desc.trim().is_empty() {
        return Err(format!(
            "push target '{}' has no description — push would fail \
             (run `jj describe -r {} -R {repo_str}` first)",
            sq.target, sq.target
        )
        .into());
    }

    Ok(())
}

/// Extract the values of column-0 `ochid:` trailer lines from a
/// commit description, in order of appearance.
fn extract_ochids(desc: &str) -> Vec<String> {
    desc.lines()
        .filter_map(|line| line.strip_prefix("ochid:"))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

/// Return the `ochid:` trailer values present in `source_desc` but
/// absent from `target_desc` — the trailers a squash with
/// `--use-destination-message` would silently drop (Bugs #2).
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
/// - Compares the two messages' `ochid:` trailers; errors when the
///   source carries any the destination's message lacks —
///   `--use-destination-message` would discard them, leaving the
///   counterpart repo's cross-links dangling (Bugs #2).
fn check_squash_keeps_ochids(
    repo: &str,
    cwd: &std::path::Path,
    sq: &SquashSpec,
) -> Result<(), Box<dyn std::error::Error>> {
    let desc_of = |rev: &str| {
        run(
            "jj",
            &[
                "log",
                "-r",
                rev,
                "--no-graph",
                "-T",
                "description",
                "-R",
                repo,
            ],
            cwd,
        )
    };
    let at_risk = ochids_at_risk(&desc_of(&sq.source)?, &desc_of(&sq.target)?);
    if at_risk.is_empty() {
        return Ok(());
    }
    let listed = at_risk
        .iter()
        .map(|ochid| format!("  {ochid}"))
        .collect::<Vec<_>>()
        .join("\n");
    Err(format!(
        "refusing squash {} → {}: the squash would drop ochid: trailers\n\
         the destination's message lacks:\n\
         {listed}\n\
         merge the messages by hand (`jj describe {} -R {}`) or clear\n\
         the source's description, then retry",
        sq.source, sq.target, sq.target, repo,
    )
    .into())
}

/// Return Ok(()) if the revset resolves to one or more commits in `repo`.
fn jj_rev_exists(repo: &str, cwd: &std::path::Path, rev: &str) -> Result<(), String> {
    run(
        "jj",
        &["log", "-r", rev, "--no-graph", "-T", "\"x\"", "-R", repo],
        cwd,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// True when `rev` has no file changes and no description — nothing
/// worth squashing.
fn rev_is_empty_undescribed(
    repo: &str,
    cwd: &Path,
    rev: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let empty = run(
        "jj",
        &["log", "-r", rev, "--no-graph", "-T", "empty", "-R", repo],
        cwd,
    )?;
    if empty.trim() != "true" {
        return Ok(false);
    }
    let desc = run(
        "jj",
        &[
            "log",
            "-r",
            rev,
            "--no-graph",
            "-T",
            "description",
            "-R",
            repo,
        ],
        cwd,
    )?;
    Ok(desc.trim().is_empty())
}

/// Full commit id of a revset in `repo`.
fn jj_commit_id(repo: &str, cwd: &Path, rev: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(run(
        "jj",
        &[
            "log",
            "-r",
            rev,
            "--no-graph",
            "-T",
            "commit_id",
            "-R",
            repo,
        ],
        cwd,
    )?
    .trim()
    .to_string())
}

/// Run the `squash-push` op: preflight, then squash (skipped when
/// the source is empty and undescribed) + bookmark-set + push.
///
/// - With an empty source and the bookmark already matching both
///   the squash target and the remote, reports "already sync'd"
///   and exits 0 — nothing to do.
/// - With an empty source but the remote behind, skips the squash
///   and still pushes.
pub fn squash_push(params: &SquashPushParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("squash_push: entry params={params:?}");
    let repo_str = params.repo.to_string_lossy().to_string();
    let cwd = std::path::Path::new(".");
    let sq = &params.squash;
    let bookmark = &params.bookmark;

    preflight(params)?;

    // Report an at-rest publish mismatch before touching anything
    // (0.69.0-3): the bookmark should match its origin counterpart
    // between runs, so a mismatch means an earlier publish was
    // lost. Publishing is this command's job, so it proceeds — the
    // report is the point, not a refusal. Suppressed when run as
    // push's `squash-push-bot` stage (see `report_publish_state`).
    if params.report_publish_state {
        match crate::common::bookmark_publish_state(&params.repo, bookmark)? {
            crate::common::PublishState::InSync => {}
            crate::common::PublishState::NeverPushed => info!(
                "squash-push: '{bookmark}' has never been pushed to origin — this run will publish it"
            ),
            crate::common::PublishState::Mismatch { local, remote } => warn!(
                "squash-push: '{bookmark}' ({}) does not match '{bookmark}@origin' ({}) — an \
                 earlier publish was likely lost; this run will publish it",
                &local[..local.len().min(12)],
                &remote[..remote.len().min(12)]
            ),
        }
    }

    // Empty-source handling: nothing to squash. If the bookmark
    // already matches both the target and the remote, nothing to
    // push either — report and exit 0.
    if rev_is_empty_undescribed(&repo_str, cwd, &sq.source)? {
        let target_cid = jj_commit_id(&repo_str, cwd, &sq.target)?;
        let bookmark_cid = jj_commit_id(&repo_str, cwd, bookmark)?;
        let remote_cid =
            jj_commit_id(&repo_str, cwd, &format!("{bookmark}@origin")).unwrap_or_default(); // OK: unresolvable remote bookmark (never pushed) → treated as not sync'd
        if bookmark_cid == target_cid && bookmark_cid == remote_cid {
            info!("squash-push: repo '{repo_str}' is already sync'd with remote");
            return Ok(());
        }
        info!(
            "squash-push: {} is empty — skipping squash, still pushing",
            sq.source
        );
    } else {
        info!("squash-push: squashing {} → {}...", sq.source, sq.target);
        run(
            "jj",
            &[
                "squash",
                "--ignore-immutable",
                "--use-destination-message",
                "--from",
                &sq.source,
                "--into",
                &sq.target,
                "-R",
                &repo_str,
            ],
            cwd,
        )?;
    }

    info!(
        "squash-push: setting bookmark '{bookmark}' to {}...",
        sq.target
    );
    run(
        "jj",
        &[
            "bookmark", "set", bookmark, "-r", &sq.target, "-R", &repo_str,
        ],
        cwd,
    )?;

    info!("squash-push: pushing '{bookmark}' to origin...");
    run(
        "jj",
        &["git", "push", "--bookmark", bookmark, "-R", &repo_str],
        &params.repo,
    )?;

    info!("squash-push: done");
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

    #[test]
    fn no_args_defaults() {
        let args = parse(&["vc-x1", "squash-push"]);
        assert_eq!(args.bookmark, "main");
        assert_eq!(args.repo, PathBuf::from("."));
        assert!(args.squash.value.is_none());
    }

    #[test]
    fn bookmark_positional() {
        let args = parse(&["vc-x1", "squash-push", "dev-0.14.0"]);
        assert_eq!(args.bookmark, "dev-0.14.0");
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
            assert_eq!(args.bookmark, "dev-0.14.0");
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
        let args = parse(&["vc-x1", "squash-push"]);
        let params = SquashPushParams::try_from(&args).unwrap();
        assert_eq!(params.repo, std::fs::canonicalize(".").unwrap());
        assert_eq!(params.bookmark, "main");
        assert_eq!(params.squash, squash_at());
        assert!(params.report_publish_state, "CLI invocations report");
    }

    /// A lost publish (`main` moved without a push) is healed by a
    /// bare squash-push run: it reports the mismatch and publishes,
    /// leaving `main == main@origin`.
    #[test]
    fn squash_push_publishes_unpushed_bookmark_move() {
        use crate::test_helpers::{Fixture, jj_ok};

        let fx = Fixture::new("sp-heal");
        std::fs::write(fx.claude.join("lost.txt"), "lost session data\n").expect("write lost file");
        jj_ok(&fx.claude, &["commit", "-m", "lost session commit"]);
        jj_ok(&fx.claude, &["bookmark", "set", "main", "-r", "@-"]);

        let params = SquashPushParams {
            repo: fx.claude.clone(),
            squash: squash_at(),
            bookmark: "main".to_string(),
            report_publish_state: true,
        };
        squash_push(&params).expect("squash-push should publish the lost commit");

        let cid = |rev: &str| {
            jj_ok(
                &fx.claude,
                &["log", "-r", rev, "--no-graph", "-T", "commit_id"],
            )
        };
        assert_eq!(cid("main"), cid("main@origin"), "main should be published");
    }

    #[test]
    fn extract_ochids_none() {
        assert!(extract_ochids("").is_empty());
        assert!(extract_ochids("title\n\nbody, no trailers\n").is_empty());
    }

    #[test]
    fn extract_ochids_trailers() {
        let desc = "title\n\nbody line\n\nochid: /abcdefabcdef\nochid: /.claude/xyzxyzxyzxyz\n";
        assert_eq!(
            extract_ochids(desc),
            vec!["/abcdefabcdef", "/.claude/xyzxyzxyzxyz"]
        );
    }

    #[test]
    fn extract_ochids_column_zero_only() {
        // Indented mentions aren't trailers; bare "ochid:" has no value.
        let desc = "title\n\n  ochid: /indented\nochid:\nochid:   /trimmed  \n";
        assert_eq!(extract_ochids(desc), vec!["/trimmed"]);
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

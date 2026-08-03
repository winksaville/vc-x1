//! `push` subcommand: collapse the dual-repo commit+push+
//! squash-push ceremony into a single command.
//!
//! See `notes/chores/chores-05.md > Add push subcommand (0.37.0)` for the
//! original design.
//!
//! The run is a straight line: review -> message -> commit-work ->
//! commit-bot -> bookmark-set -> push-work -> squash-push-bot. There
//! is no saved state and no resume, because a recorded position
//! describes a world that may have changed for reasons push cannot
//! know. What push owes instead is that **rerunning is always
//! safe**: every stage checks its own precondition and no-ops when
//! its work is already done, so a rerun after a failure is simply a
//! run.
//!
//! Failures before `push-work` are rolled back from `jj op`
//! snapshots taken moments earlier in this same process. Failures
//! from `push-work` on have crossed the remote boundary: push
//! stops and reports, and putting the repos right is the user's
//! call, not something a tool can infer.

use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info, warn};

use crate::common::{prompt, run};
use crate::context::Context;
use crate::jj;
use crate::options_flags::squash::SquashSpec;
use crate::subcommand::SubcommandRunner;
use crate::sync::{current_op_id, op_restore};

/// Bookmark the bot (`.claude`) repo always advances and
/// pushes. The bot repo is a linear journal on `main` by
/// design: `<bookmark>` names a work-repo bookmark only.
const BOT_BOOKMARK: &str = "main";

/// CLI arguments for the `push` subcommand.
///
/// Flag set mirrors the design in `notes/chores/chores-05.md`. Flags are
/// parsed in 0.37.0-0; their real effects land across the remaining
/// 0.37.0-N steps. See the module docstring for which flags activate
/// when.
#[derive(Args, Debug)]
pub struct PushArgs {
    /// Bookmark to advance in the work repo (positional form of
    /// `--bookmark`). The bot repo always advances `main`.
    ///
    /// Accepting a positional lets the common case read as `vc-x1 push main`
    /// without the `--bookmark` ceremony; `--bookmark` is kept as an
    /// alias for scripts and for composition with other args. The two
    /// forms conflict if both supplied.
    #[arg(value_name = "BOOKMARK", conflicts_with = "bookmark")]
    pub bookmark_pos: Option<String>,

    /// Bookmark to advance in the work repo (flag form; see positional).
    #[arg(long, conflicts_with = "bookmark_pos")]
    pub bookmark: Option<String>,

    /// Pause between every stage for an interactive approval gate.
    #[arg(long)]
    pub step: bool,

    /// Stop before `squash-push-bot` so it can be run manually.
    #[arg(long)]
    pub no_squash_push: bool,

    /// Print the exact commands for every stage without side effects.
    #[arg(long)]
    pub dry_run: bool,

    /// Commit title (skip `$EDITOR` for the message stage).
    #[arg(long, value_name = "STR")]
    pub title: Option<String>,

    /// Commit body (skip `$EDITOR` for the message stage).
    #[arg(long, value_name = "STR")]
    pub body: Option<String>,

    /// Auto-approve interactive prompts (review gate, message-edit
    /// confirmation). Required when `--title` / `--body` aren't both
    /// supplied in non-interactive contexts.
    #[arg(short = 'y', long)]
    pub yes: bool,
}

/// Inputs to the push op, flat, owned, clap-free.
///
/// Mirrors `PushArgs`, with the two clap bookmark spellings
/// (positional `BOOKMARK` and `--bookmark`) collapsed into one
/// `bookmark` field. The rest map straight over: `step`,
/// `no_squash_push`, `dry_run`, `title`, `body`, `yes`.
pub struct PushParams {
    pub bookmark: Option<String>,
    pub step: bool,
    pub no_squash_push: bool,
    pub dry_run: bool,
    pub title: Option<String>,
    pub body: Option<String>,
    pub yes: bool,
}

/// What the message stage resolves and the later stages consume.
///
/// Lives for one run in one process: the deliberate replacement
/// for the persisted `PushState`. Both chids are captured before
/// any commit happens, so each repo's `ochid:` trailer can name the
/// other side's change.
struct Run {
    title: String,
    body: String,
    /// Chid of the work-repo change this run publishes: `@` when the
    /// working copy has changes (it becomes `@-` once committed),
    /// else the existing `@-`.
    work_chid: String,
    /// Same rule on the bot side.
    bot_chid: String,
    /// Whether the bot working copy had changes: decides whether
    /// `commit-bot` runs.
    bot_had_changes: bool,
}

/// `jj op` ids captured immediately before the first mutation, used
/// to restore both repos if a pre-remote stage fails.
pub(crate) struct Snapshots {
    pub(crate) work: String,
    pub(crate) bot: String,
}

impl From<&PushArgs> for PushParams {
    /// Convert clap-derived `PushArgs` into the flat `PushParams`,
    /// collapsing `bookmark_pos` / `bookmark` (clap already enforces
    /// they don't both appear) into the single `bookmark` field.
    fn from(a: &PushArgs) -> Self {
        Self {
            bookmark: a.bookmark_pos.clone().or_else(|| a.bookmark.clone()),
            step: a.step,
            no_squash_push: a.no_squash_push,
            dry_run: a.dry_run,
            title: a.title.clone(),
            body: a.body.clone(),
            yes: a.yes,
        }
    }
}

impl SubcommandRunner for PushArgs {
    type Params = PushParams;

    /// Delegate to the existing `From<&PushArgs>` impl above
    /// (total: never fails).
    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(PushParams::from(self))
    }

    /// Run the existing `push` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        push(ctx, params)
    }
}

/// Whether `stdin` is attached to a terminal.
///
/// Interactive prompts (`stage_review`, `$EDITOR` launch in
/// `stage_message`, `--step` gates) need this to fail fast in
/// scripted / CI contexts instead of hanging on `read_line`.
/// `--yes` overrides the check: script callers opt in by
/// asserting "all prompts auto-approved".
fn is_stdin_tty() -> bool {
    std::io::stdin().is_terminal()
}

/// Entry point for the `push` subcommand.
///
/// 0.37.0-5 behavior: interactive flow with two approval gates
/// (review + message) plus polish flags: `--dry-run` prints what
/// would run without side effects, `--step` pauses between every
/// stage, non-tty stdin fails fast when prompts are required,
/// `--yes` opts out of all prompts. On any failure in stages 4-6
/// (the local mutation window between `commit-work` and
/// `bookmark-set`), both repos roll back to the `jj op` snapshot
/// recorded at the start of `commit-work`. After `push-work` the
/// remote boundary is crossed and recovery is forward-only.
///
/// `ctx` supplies the repo sessions the mutation stages run their
/// verbs on (`Context::session`, one open per repo for the whole
/// run).
pub fn push(ctx: &mut Context, params: &PushParams) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    push_in(ctx, &cwd, params)
}

/// `push` parameterized on the workspace root. CLI dispatch calls
/// this with `std::env::current_dir()`; integration tests call it
/// with a fixture tempdir so the stages mutate the fixture's repos
/// instead of the developer's working tree.
///
/// The stages run in a straight line. The mutation window
/// (`commit-work` -> `bookmark-set`) is wrapped so a failure inside
/// it restores both repos from snapshots taken just before it;
/// after `push-work` the remote boundary is crossed and a failure
/// simply propagates.
pub(crate) fn push_in(
    ctx: &mut Context,
    workspace_root: &Path,
    params: &PushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    if params.dry_run {
        info!("push: DRY-RUN: no side effects (no commits, no pushes)");
    }

    let bookmark = params.bookmark.as_deref().ok_or(
        "push: a bookmark is required: pass it as a positional \
         (`vc-x1 push main`) or via `--bookmark main`",
    )?;

    stage_review(workspace_root, params)?;
    step_gate(params, "review", "message")?;

    let run = stage_message(workspace_root, params)?;
    step_gate(params, "message", "commit-work")?;

    // Snapshot both repos before the first mutation. Dry-runs
    // mutate nothing, so they need no rollback target.
    let snapshots = if params.dry_run {
        None
    } else {
        Some(Snapshots {
            work: current_op_id(workspace_root)?,
            bot: current_op_id(&bot_path(workspace_root)?)?,
        })
    };

    if let Err(e) = mutate(ctx, workspace_root, bookmark, &run, params) {
        if let Some(snapshots) = &snapshots {
            rollback_on_failure(workspace_root, snapshots, e.as_ref());
        }
        return Err(e);
    }

    stage_push_work(ctx, workspace_root, bookmark, params)?;

    if params.no_squash_push {
        info!("push squash-push-bot: skip (--no-squash-push)");
    } else {
        step_gate(params, "push-work", "squash-push-bot")?;
        stage_squash_push_bot(ctx, workspace_root, params)?;
    }

    if params.dry_run {
        info!("push: DRY-RUN complete: no changes written");
        return Ok(());
    }

    // Honest completion: confirm the world matches what the stages
    // claim to have done. Catches the 0.37.1 false-success symptom.
    // Warning-only: work has crossed the remote boundary by now, so
    // rollback isn't sound and the user needs to investigate.
    match verify_completion(workspace_root, bookmark, &run) {
        Ok(()) => info!("push: completed all stages (verified)"),
        Err(e) => {
            warn!("push: completed stages but post-completion verification failed:");
            warn!("  {e}");
            warn!("  investigate the discrepancy before the next push");
        }
    }
    Ok(())
}

/// The rollback-eligible window: everything that mutates a repo
/// before anything reaches a remote.
fn mutate(
    ctx: &mut Context,
    root: &Path,
    bookmark: &str,
    run: &Run,
    params: &PushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    stage_commit_work(ctx, root, run, params)?;
    step_gate(params, "commit-work", "commit-bot")?;
    stage_commit_bot(ctx, root, run, params)?;
    step_gate(params, "commit-bot", "bookmark-set")?;
    stage_bookmark_set(ctx, root, bookmark, params)?;
    step_gate(params, "bookmark-set", "push-work")?;
    Ok(())
}

/// `--step`: pause between stages so the user can inspect the
/// repos before the next one runs.
///
/// Skipped entirely without `--step`, and under `--yes` or a
/// non-tty stdin the prompt would hang, so `--yes` bypasses it and
/// a non-tty without `--yes` is an error rather than a hang.
fn step_gate(
    params: &PushParams,
    done: &str,
    next: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !params.step {
        return Ok(());
    }
    if params.yes {
        debug!("push step: --yes skips step gate between {done} and {next}");
        return Ok(());
    }
    if !is_stdin_tty() {
        return Err("push: --step requires a tty (stdin is not interactive); \
                    add --yes to bypass step gates in non-interactive contexts"
            .into());
    }
    let answer = prompt(&format!(
        "push step: {done} done. Continue to {next}? [y/N] "
    ))?;
    let normalized = answer.trim().to_ascii_lowercase();
    if normalized != "y" && normalized != "yes" {
        return Err(format!("push: step gate declined after {done} (got {answer:?})").into());
    }
    Ok(())
}

/// Full path of the bot repo for a given workspace root, resolved
/// from `.vc-config.toml`'s `repos.bot` (push requires a dual
/// workspace). Also the dual-mode entry preflight: resolution
/// verifies both repos' recorded topology agrees and errors loudly
/// (with everything known, changing nothing) when it doesn't.
fn bot_path(workspace_root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    crate::common::require_bot_dir(workspace_root)
}

/// Restore both repos to their `jj op` snapshots.
///
/// Best-effort: if the restore itself fails we warn but don't
/// shadow the original error the caller will propagate. Exposed at
/// `pub(crate)` so integration tests can exercise the rollback path
/// directly rather than forcing a stage failure.
pub(crate) fn rollback_on_failure(
    root: &Path,
    snapshots: &Snapshots,
    original: &dyn std::error::Error,
) {
    warn!("push: rolling back both repos after: {original}");
    match op_restore(root, &snapshots.work) {
        Ok(()) => info!("push: restored work repo to op {}", snapshots.work),
        Err(e) => warn!("push: work repo restore failed: {e}"),
    }
    // Rollback is best-effort inside an error path: a failed
    // bot-dir resolution is reported, not propagated.
    match bot_path(root) {
        Ok(bot) => match op_restore(&bot, &snapshots.bot) {
            Ok(()) => info!("push: restored bot repo to op {}", snapshots.bot),
            Err(e) => warn!("push: bot repo restore failed: {e}"),
        },
        Err(e) => warn!("push: bot repo restore skipped (unresolvable): {e}"),
    }
}

/// Review: first approval gate, "is the work done right?".
///
/// Shows a `jj diff --stat` of the pending changes in both repos
/// and prompts the user to continue. `--yes` short-circuits the
/// prompt (required for scripted / non-tty use). In `--dry-run`
/// the diff is still shown (that's the point of dry-run: see
/// what *would* be reviewed) but approval is auto-granted.
fn stage_review(root: &Path, params: &PushParams) -> Result<(), Box<dyn std::error::Error>> {
    let bot = bot_path(root)?;
    let work_arg = root.to_string_lossy();
    let bot_arg = bot.to_string_lossy();
    info!("push review: pending changes:");
    info!("  work ({work_arg}):");
    let work_stat = run("jj", &["diff", "--stat", "-R", &work_arg], root)?;
    for line in work_stat.lines() {
        info!("    {line}");
    }
    info!("  .claude ({bot_arg}):");
    let bot_stat = run("jj", &["diff", "--stat", "-R", &bot_arg], root)?;
    for line in bot_stat.lines() {
        info!("    {line}");
    }
    if params.yes {
        info!("push review: auto-approved (--yes)");
        return Ok(());
    }
    if params.dry_run {
        info!("push review: [dry-run] auto-approved");
        return Ok(());
    }
    if !is_stdin_tty() {
        return Err("push review: stdin is not a tty; \
                    pass --yes to auto-approve in non-interactive contexts"
            .into());
    }
    let answer = prompt("push review: approve and continue to message stage? [y/N] ")?;
    let normalized = answer.trim().to_ascii_lowercase();
    if normalized == "y" || normalized == "yes" {
        Ok(())
    } else {
        Err(format!(
            "push review: declined (got {answer:?}); re-run with --yes or confirm with 'y'"
        )
        .into())
    }
}

/// Message: resolve the commit message and capture both repos'
/// pre-commit changeIDs, so each side's `ochid:` trailer can name
/// the other.
///
/// Title/body come from `--title`/`--body`, else `$EDITOR`. There
/// is no persisted message to fall back on: a rerun supplies the
/// flags again or opens the editor again, which is the cost of
/// having no state file and a cheap one: the message is in the
/// user's scrollback either way.
///
/// When neither repo has pending changes, no commit will be made on
/// either side, so an absent message is not an error: the run is
/// publish-only (bookmark + remote), which is the trapezoid
/// recipe's final step.
fn stage_message(root: &Path, params: &PushParams) -> Result<Run, Box<dyn std::error::Error>> {
    let bot = bot_path(root)?;
    // Both sides resolve their chid the same way: `@` when the
    // working copy has changes (it becomes `@-` once committed),
    // else the existing `@-` (which stays put, because that side's
    // commit stage skips). Taking `@` unconditionally on the work
    // side is what minted an empty stamped duplicate in bugs.md #4.
    let work_had_changes = !jj::is_empty(root, "@")?;
    let work_ref = if work_had_changes { "@" } else { "@-" };
    let work_chid = jj::chid_of(root, work_ref)?;
    let bot_had_changes = !jj::is_empty(&bot, "@")?;
    let bot_ref = if bot_had_changes { "@" } else { "@-" };
    let bot_chid = jj::chid_of(&bot, bot_ref)?;
    let will_commit = work_had_changes || bot_had_changes;

    let (title, body) = match (params.title.clone(), params.body.clone()) {
        (Some(t), Some(b)) => (t, b),
        _ if !will_commit => {
            info!("push message: skip (neither repo has pending changes)");
            (String::new(), String::new())
        }
        _ => {
            if params.yes {
                return Err("push message: --yes given but --title/--body missing: \
                            pass both flags or run interactively."
                    .into());
            }
            if params.dry_run {
                return Err("push message: --dry-run given but --title/--body missing: \
                            pass both flags so dry-run has a message to preview."
                    .into());
            }
            if !is_stdin_tty() {
                return Err("push message: stdin is not a tty and no --title/--body \
                            supplied; cannot launch $EDITOR in a non-interactive \
                            context. Pass --title and --body, or run interactively."
                    .into());
            }
            compose_message_via_editor()?
        }
    };

    info!(
        "push message: title=\"{}\", work_chid={work_chid}, bot_chid={bot_chid}, \
         work_had_changes={work_had_changes}, bot_had_changes={bot_had_changes}",
        title.lines().next().unwrap_or("") // OK: obvious
    );
    Ok(Run {
        title,
        body,
        work_chid,
        bot_chid,
        bot_had_changes,
    })
}

/// Launch `$EDITOR` (falling back to `vi`) on a scratch template,
/// then parse the saved content into a `(title, body)` tuple. Lines
/// starting with `#` are treated as comments and stripped. Empty
/// input aborts the push.
///
/// The template lives in the system temp dir, keyed by pid: with no
/// state dir left to put it in, and the file being read back within
/// the same call, it never needs to outlive the process.
fn compose_message_via_editor() -> Result<(String, String), Box<dyn std::error::Error>> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string()); // OK: POSIX fallback when nothing is configured
    let msg_path =
        std::env::temp_dir().join(format!("vc-x1-push-message-{}.txt", std::process::id()));
    let template = "\
# Leave as-is to abort. Enter the Title on the first line,
# optionally followed by the Body on subsequent lines. The
# blank line between Title and Body is inserted automatically,
# and the ochid trailer is appended per-repo automatically:
# don't add either here.
# Lines starting with `#` are ignored.
";
    fs::write(&msg_path, template)?;
    info!("push message: launching {editor} on {}", msg_path.display());
    let status = std::process::Command::new(&editor)
        .arg(&msg_path)
        .status()
        .map_err(|e| format!("failed to launch {editor}: {e}"))?;
    if !status.success() {
        return Err(format!("{editor} exited non-zero ({status})").into());
    }
    let content = fs::read_to_string(&msg_path)?;
    let _ = fs::remove_file(&msg_path);
    parse_message(&content)
        .ok_or_else(|| "push message: template was left empty or all-comments, aborting".into())
}

/// Parse the editor-saved message into `(title, body)`. Strips
/// `#`-prefixed comment lines, trims surrounding blanks, and treats
/// the first non-comment line as the title with everything after as
/// the body (a blank line between is allowed but not required:
/// the commit stages insert the title/body separator themselves).
/// Returns `None` when the message has no non-comment content.
fn parse_message(raw: &str) -> Option<(String, String)> {
    let mut meaningful = String::new();
    for line in raw.lines() {
        let trimmed_left = line.trim_start();
        if trimmed_left.starts_with('#') {
            continue;
        }
        meaningful.push_str(line);
        meaningful.push('\n');
    }
    let trimmed = meaningful.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut lines = trimmed.lines();
    // OK: trimmed non-empty => first line exists
    let title = lines.next().unwrap_or("").trim().to_string();
    let body = lines.collect::<Vec<_>>().join("\n").trim().to_string();
    if title.is_empty() {
        return None;
    }
    Some((title, body))
}

/// Commit work repo with `title` / `body` and the `ochid:` trailer
/// pointing at `.claude`'s chid, or skip when the working copy has
/// nothing pending: the same rule `stage_commit_bot` follows.
///
/// Skipping matters in two situations:
///
/// - The rung was committed by hand before invoking push. Committing
///   the empty `@` anyway minted a stamped empty duplicate on top of
///   the real commit (bugs.md #4).
/// - A publish-only run, where the commits already exist and only
///   the bookmark and the remote still need advancing: the
///   trapezoid recipe's final step.
///
/// In both cases the existing top commit is published as-is; push
/// does not rewrite a description it didn't author.
fn stage_commit_work(
    ctx: &mut Context,
    root: &Path,
    run_msg: &Run,
    params: &PushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    if jj::is_empty(root, "@")? {
        warn!(
            "push commit-work: skip (work repo had no pending changes); \
             the supplied message is not applied: the existing top \
             commit is published as-is"
        );
        return Ok(());
    }
    let (title, body) = (&run_msg.title, &run_msg.body);
    let bot_prefix = crate::desc_helpers::ochid_prefix_for(&bot_path(root)?)?;
    let body_with_trailer = format!("{body}\n\nochid: {bot_prefix}{}", run_msg.bot_chid);
    let work_arg = root.to_string_lossy();
    if params.dry_run {
        info!(
            "push commit-work: [dry-run] would run jj commit -R {work_arg} -m \"{title}\" \
             -m <body+ochid>"
        );
        return Ok(());
    }
    info!("push commit-work: commit -R {work_arg}");
    ctx.session(root)?
        .commit(&format!("{title}\n\n{body_with_trailer}"))?;
    Ok(())
}

/// Commit `.claude` with the same title/body and the ochid trailer
/// pointing at the work commit's chid, or skip if `.claude` had no
/// pending changes.
fn stage_commit_bot(
    ctx: &mut Context,
    root: &Path,
    run_msg: &Run,
    params: &PushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    if !run_msg.bot_had_changes {
        info!("push commit-bot: skip (.claude had no pending changes)");
        return Ok(());
    }
    let (title, body) = (&run_msg.title, &run_msg.body);
    let body_with_trailer = format!("{body}\n\nochid: /{}", run_msg.work_chid);
    let bot = bot_path(root)?;
    let bot_arg = bot.to_string_lossy();
    if params.dry_run {
        info!(
            "push commit-bot: [dry-run] would run jj commit -R {bot_arg} -m \"{title}\" \
             -m <body+ochid>"
        );
        return Ok(());
    }
    info!("push commit-bot: commit -R {bot_arg}");
    ctx.session(&bot)?
        .commit(&format!("{title}\n\n{body_with_trailer}"))?;
    Ok(())
}

/// Advance each repo's bookmark to its `@-`: the work repo advances
/// `state.bookmark`, the bot repo always advances
/// `BOT_BOOKMARK` (`main`): a feature bookmark must never be
/// created in the linear-journal bot repo.
fn stage_bookmark_set(
    ctx: &mut Context,
    root: &Path,
    bookmark: &str,
    params: &PushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let bk = bookmark;
    let work_arg = root.to_string_lossy();
    let bot = bot_path(root)?;
    let bot_arg = bot.to_string_lossy();
    if params.dry_run {
        info!(
            "push bookmark-set: [dry-run] would run jj bookmark set {bk} -r @- -R {work_arg} \
             / {BOT_BOOKMARK} -r @- -R {bot_arg}"
        );
        return Ok(());
    }
    info!(
        "push bookmark-set: bookmark set {bk} -r @- -R {work_arg} / {BOT_BOOKMARK} -r @- \
         -R {bot_arg}"
    );
    ctx.session(root)?.bookmark_set(bk, "@-")?;
    ctx.session(&bot)?.bookmark_set(BOT_BOOKMARK, "@-")?;
    Ok(())
}

/// Push the work repo's bookmark to origin.
///
/// Checks its own precondition: every remote ref for the bookmark
/// is tracked, which is where preflight's tracking check moved
/// when preflight was retired. A never-pushed bookmark has no
/// remote ref and passes; the check catches a ref that exists but
/// isn't tracked, which would otherwise push into a ref jj won't
/// follow afterwards.
///
/// Rerunning is safe without a guard of its own: `jj git push` on
/// an already-published bookmark reports nothing to do.
fn stage_push_work(
    ctx: &mut Context,
    root: &Path,
    bookmark: &str,
    params: &PushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let bk = bookmark;
    let work_arg = root.to_string_lossy();
    if params.dry_run {
        info!("push push-work: [dry-run] would run jj git push --bookmark {bk} -R {work_arg}");
        return Ok(());
    }
    crate::common::verify_tracking(root, bk)?;
    info!("push push-work: git push --bookmark {bk} -R {work_arg}");
    ctx.session(root)?.git_push_bookmark(bk)?;
    Ok(())
}

/// Squash-push `.claude` in-process, always pushing
/// `BOT_BOOKMARK` (`main`): the bot repo's bookmark is
/// pinned, so the work-repo bookmark plays no part here.
///
/// - Squashes the tail (session writes that landed since
///   `commit-bot`) into the bot commit, then pushes, via
///   `squash_push::squash_push`, so the ochid-drop guard
///   applies and a failure is a visible push failure.
/// - In-process since 0.69.0-1 (the stage's detached child died
///   silently at sandbox teardown: the loss diagnosed in
///   0.68.1).
/// - `--no-squash-push` turns this stage into a no-op so the
///   squash+push can be run manually.
fn stage_squash_push_bot(
    ctx: &mut Context,
    root: &Path,
    params: &PushParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let bk = BOT_BOOKMARK;
    let bot = bot_path(root)?;
    let bot_arg = bot.to_string_lossy();
    if params.dry_run {
        info!(
            "push squash-push-bot: [dry-run] would squash @ -> @- and push {bk} in {bot_arg} \
             (in-process)"
        );
        return Ok(());
    }
    info!("push squash-push-bot: squash @ -> @- + push {bk} -R {bot_arg} (in-process)");
    let sp = crate::squash_push::SquashPushParams {
        repo: bot.clone(),
        squash: SquashSpec {
            source: "@".to_string(),
            target: "@-".to_string(),
        },
        bookmark: bk.to_string(),
        // Mid-push the mismatch is the normal state (bookmark-set
        // just moved main; this stage publishes it): don't report
        // a lost publish.
        report_publish_state: false,
    };
    crate::squash_push::squash_push(ctx, &sp)
}

/// Verify the world matches what the stages claim to have done.
///
/// Runs after the last stage and confirms the post-conditions are
/// real: directly addresses the 0.37.1 false-success symptom (push
/// declared "completed all stages" while the working copy still held
/// uncommitted changes).
///
/// Three checks:
///
/// 1. Work's bookmark is at a commit whose chid matches the run's
///    `work_chid`.
/// 2. Work's working copy has no uncommitted changes (commit-work
///    should have captured anything that was there).
/// 3. `.claude`'s bookmark is at the run's `bot_chid`.
///    `.claude`'s working copy (`@`) may legitimately have new
///    session writes that landed after squash-push-bot's squash
///    (the tail rides into the next cycle's commit) so no
///    working-copy-clean check on `.claude`.
///
/// On any mismatch: returns `Err`. The caller surfaces it as a loud
/// warning rather than blocking: push has already crossed the
/// remote boundary, work is live on origin, and the warning prompts
/// the user to investigate.
fn verify_completion(
    root: &Path,
    bookmark: &str,
    run_msg: &Run,
) -> Result<(), Box<dyn std::error::Error>> {
    let root_str = root.to_string_lossy();
    let cwd = Path::new(".");

    // 1. work bookmark at the run's work_chid.
    let actual = jj::chid_of(root, bookmark)?;
    if actual != run_msg.work_chid {
        return Err(format!(
            "completion sanity: work bookmark '{bookmark}' is at chid '{actual}' but \
             this run committed '{}'.",
            run_msg.work_chid
        )
        .into());
    }

    // 2. work working copy (`@`) clean (no uncommitted changes). Use the `empty`
    // template: `jj diff --stat` always emits a "0 files changed"
    // line even when clean, so plain output-emptiness check would
    // false-positive.
    if !jj::is_empty(root, "@")? {
        let wc_diff = run("jj", &["diff", "--stat", "-r", "@", "-R", &root_str], cwd)?;
        return Err(format!(
            "completion sanity: work working copy has uncommitted changes (push completed \
             but working copy dirty?). Diff stat:\n{wc_diff}"
        )
        .into());
    }

    // 3. .claude's pinned bookmark (main) at the run's bot_chid.
    let actual = jj::chid_of(&bot_path(root)?, BOT_BOOKMARK)?;
    if actual != run_msg.bot_chid {
        return Err(format!(
            "completion sanity: .claude bookmark '{BOT_BOOKMARK}' is at chid \
             '{actual}' but this run committed '{}'.",
            run_msg.bot_chid
        )
        .into());
    }

    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

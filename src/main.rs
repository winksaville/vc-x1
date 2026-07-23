mod bot_session;
mod chid;
mod clone;
mod common;
mod config;
mod config_cmd;
mod config_schema;
mod context;
mod desc;
mod desc_helpers;
mod fix_desc;
mod fix_todo;
mod init;
mod jj;
mod list;
mod logging;
mod options_flags;
mod push;
mod repo_utils;
mod revert;
mod show;
mod squash_push;
mod subcommand;
mod symlink;
mod sync;
#[cfg(test)]
mod test_helpers;
#[cfg(test)]
mod test_tmp_root;
mod todo_helpers;
mod toml_simple;
mod transcript;
mod url;
mod validate_bot;
mod validate_desc;
mod validate_todo;

use std::path::Path;
use std::process::ExitCode;

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use clap_complete::CompleteEnv;
use log::error;

use crate::subcommand::SubcommandRunner;

/// Banner string emitted as the first line of normal command runs
/// and shown at the top of subcommand `--help` output. Built from
/// `Cargo.toml`'s name + version at compile time so it stays in
/// sync with the bumped version.
const BANNER: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

/// Top-level about line — name, version, and the project tagline
/// on a single line. Used as the top-level `about` so `vc-x1 -h`
/// reads as one banner-plus-tagline header instead of two stacked
/// lines.
const TOP_ABOUT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    " ",
    env!("CARGO_PKG_VERSION"),
    " - jj workspace tooling"
);

/// Build the clap command tree with `BANNER` set as `before_help`
/// on every subcommand (transitively). Top-level skips `before_help`
/// because its own `about` already carries the name+version+tagline.
/// Walks via `mut_subcommand` so individual subcommand
/// `#[command(long_about = ...)]` blocks don't have to repeat the
/// banner text.
fn cli_with_banner() -> clap::Command {
    fn add_to_subs(mut cmd: clap::Command) -> clap::Command {
        let names: Vec<String> = cmd
            .get_subcommands()
            .map(|c| c.get_name().to_string())
            .collect();
        for name in names {
            cmd = cmd.mut_subcommand(name, add_with_banner);
        }
        cmd
    }
    fn add_with_banner(mut cmd: clap::Command) -> clap::Command {
        cmd = cmd.before_help(BANNER);
        let names: Vec<String> = cmd
            .get_subcommands()
            .map(|c| c.get_name().to_string())
            .collect();
        for name in names {
            cmd = cmd.mut_subcommand(name, add_with_banner);
        }
        cmd
    }
    add_to_subs(Cli::command())
}

#[derive(Parser, Debug)]
#[command(about = TOP_ABOUT, max_term_width = 80)]
pub struct Cli {
    /// Print the `vc-x1 X.Y.Z` banner as the first line, then
    /// continue. With no subcommand, prints the banner and exits.
    ///
    /// Replaces clap's auto-version (which would exit after
    /// printing): the banner now rides along with normal
    /// subcommand execution rather than gating it, so scripts
    /// can capture the version *and* the command's output in
    /// one invocation.
    #[arg(short = 'V', long = "version", global = true, action = clap::ArgAction::SetTrue)]
    pub version: bool,

    /// Verbose output: -v debug, -vv trace
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Log file path (captures all levels)
    #[arg(long, global = true)]
    pub log: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Print the changeID for a revision
    Chid(chid::ChidArgs),

    /// Show full description of a commit
    Desc(desc::DescArgs),

    /// List commits in a jj repo
    List(list::ListArgs),

    /// Show commit details and diff summary
    Show(show::ShowArgs),

    /// Display a bot session transcript as a conversation
    #[command(
        long_about = "Display a Claude Code bot session transcript (.jsonl) as a\n\
        readable conversation.\n\n\
        Output is a set of items — headers, user, assistant, tool,\n\
        thinking, results, meta, summary — each toggled by --<item> /\n\
        --no-<item> (last one wins), with --all / --none as bulk bases.\n\
        The default set (headers, user, assistant, tool, summary) can\n\
        be replaced by [bot-session].items in the user config\n\
        (comma-separated list); CLI flags then adjust the resolved\n\
        set. Malformed lines (e.g. a live session's truncated last\n\
        line) warn to stderr and never fail the run.\n\n\
        Alternate views: --fields (field inventory per entry type),\n\
        --unknown (only unmodeled paths — how the format moved), and\n\
        --raw (pretty-printed source lines). --lines slices by source\n\
        JSONL line — the same unit in every view."
    )]
    BotSession(bot_session::BotSessionArgs),

    /// Check the bot repo is published (main matches main@origin)
    #[command(
        long_about = "Check the bot repo is published (main matches main@origin).\n\n\
        At rest the bot repo's `main` always matches `main@origin` — the\n\
        bookmark only moves inside a push / squash-push run, which\n\
        publishes it in the same invocation. A mismatch means an earlier\n\
        publish was lost. Read-only and cheap (two jj lookups; no cargo\n\
        steps); also verifies main's remote refs are tracked. Exits\n\
        non-zero on any finding and fixes nothing — resolve with\n\
        `vc-x1 squash-push -R <bot-repo>`."
    )]
    ValidateBot(validate_bot::ValidateBotArgs),

    /// Validate commit descriptions against the other repo
    #[command(
        long_about = "Validate commit descriptions against the other repo.\n\n\
        Output columns: STATUS  CHANGEID  TITLE  [DETAILS]\n\n\
        Status labels:\n  \
          ok   — ochid trailer is valid\n  \
          err  — ochid has issues (wrong prefix, wrong length, ID not found)\n  \
          miss — no ochid trailer; shows match from other repo if found"
    )]
    ValidateDesc(validate_desc::ValidateDescArgs),

    /// Fix commit descriptions against the other repo (dry-run by default)
    #[command(long_about = "Fix commit descriptions against the other repo.\n\n\
        Default is dry-run; use --no-dry-run to write changes.\n\n\
        Output columns: STATUS  CHANGEID  TITLE  [DETAILS]\n\n\
        Status labels:\n  \
          ok    — ochid trailer is valid (no change)\n  \
          fix   — ochid has issues, shows proposed fix (dry-run)\n  \
          fixed — ochid was rewritten (--no-dry-run)\n  \
          add   — missing ochid, match found, shows proposed addition (dry-run)\n  \
          added — missing ochid was added (--no-dry-run)\n  \
          skip  — skipped (no ochid, no match, or max-fixes reached)\n  \
          err   — ID not found and no --fallback provided")]
    FixDesc(fix_desc::FixDescArgs),

    /// Check todo-file entry numbering and indent
    #[command(
        long_about = "Check a todo file's `## Todo` and `## Bugs` entry numbering.\n\n\
        Verifies each section is numbered 1..N in document order and\n\
        that continuation-line indent matches the number-prefix width.\n\
        Read-only; exits non-zero if any entry needs fixing — use\n\
        `fix-todo` to rewrite."
    )]
    ValidateTodo(validate_todo::ValidateTodoArgs),

    /// Renumber todo-file entries (dry-run by default)
    #[command(
        long_about = "Renumber a todo file's `## Todo` and `## Bugs` sections.\n\n\
        Renumbers each section 1..N in document order and normalizes\n\
        continuation-line indent to the number-prefix width. Dry-run\n\
        by default — prints each changed entry's corrected line; pass\n\
        --no-dry-run to write the file in place."
    )]
    FixTodo(fix_todo::FixTodoArgs),

    /// Clone a dual-repo project
    Clone(clone::CloneArgs),

    /// Create a new dual-repo project
    Init(init::InitArgs),

    /// Create Claude Code project symlink
    Symlink(symlink::SymlinkArgs),

    /// Fetch and sync a set of repos to their remotes
    #[command(long_about = format!("Fetch and sync a set of repos to their remotes.\n\n\
        Repo set is resolved (in order):\n  \
          - `-R` / `--repo`     exact list (back-compat / arbitrary multi-repo)\n  \
          - `--scope=work|bot|work,bot` dual-repo roles via `.vc-config.toml`\n  \
          - neither             default: `work,bot` when dual, else `work`\n\n\
        One atomic operation: fetch, then per repo:\n  \
          - up-to-date        nothing to do\n  \
          - behind            fast-forward bookmark to remote\n  \
          - ahead             nothing to sync (local has unpushed work)\n  \
          - diverged          rebase local onto remote; fail on conflicts\n  \
          - no remote         bookmark has no @<remote> counterpart; skip\n\n\
        After a successful sync, `@` is repositioned onto the synced\n\
        bookmark: the work repo `jj new`s a clean `@` (or rebases a\n\
        dirty one with --rebase / a prompt), the `.claude` session\n\
        repo `jj new main`s when main moved (no-op when `@-` is\n\
        already the main tip).\n\n\
        On failure sync stops where the failing step stopped — nothing\n\
        is auto-reverted, so the state can be inspected. Each repo's\n\
        pre-sync op id is persisted to `.vc-x1/sync-state.toml`; undo\n\
        explicitly with `vc-x1 revert` (state is cleared on success).\n\n\
        Output shape:\n  \
          - all-up-to-date: one-line summary (`sync: N repos are {}`)\n  \
          - action needed:  per-repo fetch + state + actions\n  \
          - --quiet:        no output; exit code signals success", sync::UP_TO_DATE_MSG))]
    Sync(sync::SyncArgs),

    /// Restore repos to their persisted pre-sync snapshots
    #[command(
        long_about = "Restore repos to their persisted pre-sync snapshots.\n\n\
        A failed `vc-x1 sync` stops where it failed and leaves each\n\
        repo's pre-sync `jj op` id in `.vc-x1/sync-state.toml` for\n\
        inspection-then-undo. `revert` resolves repos the same way\n\
        sync does (`-R` / `--scope` / workspace default), runs\n\
        `jj op restore <op>` in every repo holding a snapshot, and\n\
        clears the consumed state files.\n\n\
        Repos without a snapshot are skipped (sync clears state on\n\
        success); finding no snapshot anywhere is an error."
    )]
    Revert(revert::RevertArgs),

    /// Squash SOURCE into TARGET, advance a bookmark, and push
    #[command(
        long_about = "Squash SOURCE into TARGET (defaults: SOURCE=@, TARGET=@-),\n\
        advance a bookmark, and push.\n\n\
        Captures a repo's trailing working-copy writes into the last\n\
        commit and publishes it — rewriting an already-pushed commit,\n\
        so the push is a forced update. Built for the bot repo\n\
        (`.claude`, the session tail); also useful on the work repo\n\
        as a deliberate amend-and-push.\n\n\
        Zero-ceremony default: bare `vc-x1 squash-push` squashes\n\
        @ → @- and pushes `main` in `.`. With an empty `@` the squash\n\
        is skipped; if the bookmark already matches the remote the\n\
        command reports \"already sync'd\" and exits 0."
    )]
    SquashPush(squash_push::SquashPushArgs),

    /// Print settable config keys and their defaults
    Config(config_cmd::ConfigArgs),

    /// Commit both repos, push the work repo, squash-push the bot repo
    #[command(long_about = "Commit both repos, push the work repo's BOOKMARK, and\n\
        squash-push the bot repo's `main` — one resumable command.\n\n\
        Collapses the manual commit-push-publish ceremony into a\n\
        single subcommand with two interactive approval gates and a\n\
        state machine with persistent progress so interruptions can\n\
        resume without re-doing completed stages.\n\n\
        Stages: preflight (tracking / bot-published / sync checks —\n\
        no build steps; run project checks yourself before pushing)\n\
        → review (approve diff)\n\
        → message ($EDITOR / --title+--body, approve text) →\n\
        commit-work → commit-bot (skipped if clean) → bookmark-set\n\
        (work repo → <bookmark>, bot repo → main) → push-work →\n\
        squash-push-bot. Failures in commit-work / commit-bot /\n\
        bookmark-set roll both repos back via\n\
        `jj op restore` to the snapshot recorded before commit-work.\n\
        After push-work succeeds the remote boundary is crossed and\n\
        recovery is forward-only.\n\n\
        Non-interactive use: pass both --title and --body plus --yes\n\
        to skip the review gate. Saved state carries title/body\n\
        across resumes so only the first invocation needs them.")]
    Push(push::PushArgs),
}

/// Permanent sanity check for the `main`-bookmark tracking state
/// in both repos of the dual-repo workspace. Emits one line on
/// entry and one on exit of every command. If entry and exit
/// differ, the executing command is the culprit; if entry differs
/// from the previous command's exit, something *between*
/// invocations broke it. Originally added in 0.37.2 as a temporary
/// diagnostic; promoted to permanent in 0.37.4 after the user
/// reported "happens more than once".
///
/// Emits at `log::debug!` (since 0.52.0-1) — default runs stay
/// quiet, and the signal remains available under `-v` when
/// investigating.
///
/// Walks up from cwd to locate the workspace root (the directory
/// whose `.vc-config.toml` has a `work` key), then probes `<root>`
/// and the config-resolved bot dir. Same labeling whether the user
/// runs from the work root, from the bot dir, or from any subdir.
pub fn bm_track(phase: &str, command_name: &str) {
    let header = format!("bm-track {phase} vc-x1 {command_name}");
    let root = match common::find_workspace_root() {
        Some(r) => r,
        None => {
            log::debug!("{header}: no-workspace");
            return;
        }
    };
    // Diagnostics only — an unresolvable / single-repo workspace
    // just probes the work side.
    let mut repos: Vec<(std::path::PathBuf, String)> = vec![(root.clone(), "work".to_string())];
    if let Ok(Some(bot)) = common::bot_repo_path(&root) {
        let label = bot
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "bot".to_string());
        repos.push((bot, label));
    }
    let mut parts: Vec<String> = Vec::new();
    for (repo, label) in repos {
        if !repo.join(".jj").exists() {
            parts.push(format!("{label}(main)=no-jj"));
            continue;
        }
        match bm_track_one(&repo, "main", "origin") {
            Ok(true) => parts.push(format!("{label}(main)=tracked")),
            Ok(false) => parts.push(format!("{label}(main)=NOT_TRACKED")),
            Err(e) => parts.push(format!(
                "{label}(main)=err({})",
                e.lines().next().unwrap_or("")
            )),
        }
    }
    log::debug!("{header}: {}", parts.join(", "));
}

/// Query jj for whether `bookmark` in `repo` is tracking `remote`.
/// Returns `Ok(true)` when the `-a` listing shows a tracked
/// `@<remote>` entry (synced or divergent-decorated — both count),
/// `Ok(false)` when it doesn't (not tracking, or the bookmark
/// doesn't exist), `Err` on subprocess failure. Shares the listing
/// (`jj::bookmark_list_all`) and parser family
/// (`common::find_tracked_remote` alongside verify_tracking's
/// `find_non_tracking_remote`) so the two can't drift.
fn bm_track_one(repo: &Path, bookmark: &str, remote: &str) -> Result<bool, String> {
    let all = jj::bookmark_list_all(repo, bookmark).map_err(|e| e.to_string())?;
    Ok(common::find_tracked_remote(&all, remote))
}

fn main() -> ExitCode {
    CompleteEnv::with_factory(cli_with_banner).complete();
    let matches = cli_with_banner().get_matches();
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(e) => {
            e.exit();
        }
    };

    let log_path = cli.log.as_ref().map(|p| p.to_string_lossy().to_string());
    logging::CliLogger::init(cli.verbose, log_path.as_deref());

    // `-V` / `--version`: emit the `vc-x1 X.Y.Z` banner as the
    // first line and continue. With no subcommand, the banner is
    // the whole invocation. Uniform `BANNER` (not the
    // `vc-x1-<sub>` form clap's `propagate_version` would print)
    // — the version is the binary's regardless of which
    // subcommand it routes to.
    if cli.version {
        log::info!("{BANNER}");
    }

    let Some(cmd) = cli.command else {
        // No subcommand. If `-V` was set the banner has already
        // printed, so exit success; otherwise mirror clap's "a
        // subcommand is required" error by printing usage and
        // exiting non-zero.
        if cli.version {
            return ExitCode::SUCCESS;
        }
        let mut cmd = cli_with_banner();
        let _ = cmd.print_help();
        return ExitCode::FAILURE;
    };

    let ctx = match context::Context::load() {
        Ok(c) => c,
        Err(e) => {
            error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    match cmd {
        Commands::Chid(args) => args.dispatch(&ctx),
        Commands::Desc(args) => args.dispatch(&ctx),
        Commands::List(args) => args.dispatch(&ctx),
        Commands::Show(args) => args.dispatch(&ctx),
        Commands::BotSession(args) => args.dispatch(&ctx),
        Commands::ValidateBot(args) => args.dispatch(&ctx),
        Commands::ValidateDesc(args) => args.dispatch(&ctx),
        Commands::FixDesc(args) => args.dispatch(&ctx),
        Commands::ValidateTodo(args) => args.dispatch(&ctx),
        Commands::FixTodo(args) => args.dispatch(&ctx),
        Commands::Clone(args) => args.dispatch(&ctx),
        Commands::Init(args) => args.dispatch(&ctx),
        Commands::Symlink(args) => args.dispatch(&ctx),
        Commands::Sync(args) => args.dispatch(&ctx),
        Commands::Revert(args) => args.dispatch(&ctx),
        Commands::SquashPush(args) => args.dispatch(&ctx),
        Commands::Config(args) => args.dispatch(&ctx),
        Commands::Push(args) => args.dispatch(&ctx),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_command() {
        let err = Cli::try_parse_from(["vc-x1", "bogus"])
            .unwrap_err()
            .to_string();
        assert!(err.contains("bogus"));
    }
}

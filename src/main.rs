mod bot_session;
mod chid;
mod clone;
mod common;
mod config;
mod config_cmd;
mod config_md;
mod config_schema;
mod context;
mod desc;
mod desc_helpers;
mod fix_desc;
mod fix_todo;
mod init;
mod jj;
mod legacy_vc_config;
mod list;
mod logging;
mod md_fence;
mod options_flags;
mod push;
mod repo_utils;
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
mod validate;
mod validate_anchors;
mod validate_bot;
mod validate_desc;
mod validate_todo;
mod version;

use std::path::Path;
use std::process::ExitCode;

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use clap_complete::CompleteEnv;
use log::error;

use crate::subcommand::SubcommandRunner;

/// Name this invocation was launched under: `argv[0]`'s basename,
/// falling back to the compiled bin name when argv is empty or
/// unreadable. Runtime on purpose: the binary's on-disk name is
/// the manifest's per-line package name, and a copy or rename
/// afterwards still self-reports the name it actually runs as,
/// which no compile-time constant can know.
fn invoked_name() -> String {
    std::env::args_os()
        .next()
        .as_deref()
        .and_then(|p| Path::new(p).file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| env!("CARGO_BIN_NAME").to_string())
}

/// Banner string emitted as the first line of every run (stderr,
/// or stdout when `-V` asked for it), and shown at the top of
/// subcommand `--help` output: `<invoked name> <version>`.
fn banner() -> &'static str {
    static BANNER: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    BANNER.get_or_init(|| format!("{} {}", invoked_name(), env!("CARGO_PKG_VERSION")))
}

/// Top-level about line: name, version, and the project tagline
/// on a single line. Used as the top-level `about` so `vc-x1 -h`
/// reads as one banner-plus-tagline header instead of two stacked
/// lines.
fn top_about() -> String {
    format!("{} - jj workspace tooling", banner())
}

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
        cmd = cmd.before_help(banner());
        let names: Vec<String> = cmd
            .get_subcommands()
            .map(|c| c.get_name().to_string())
            .collect();
        for name in names {
            cmd = cmd.mut_subcommand(name, add_with_banner);
        }
        cmd
    }
    // `bin_name` from the invoked name: `get_matches` would set it
    // from argv itself, but the manual `print_help` on the
    // no-subcommand path never sees argv and would fall back to
    // the package name, printing `Usage: vc-x1` under vc-x1-dev.
    add_to_subs(Cli::command().bin_name(invoked_name()))
}

#[derive(Parser, Debug)]
#[command(about = top_about(), max_term_width = 80)]
pub struct Cli {
    /// Version detail on stdout: -V the banner, -VV the full
    /// report (as `vc-x1 version`). Counted like -v/-vv.
    ///
    /// Both ride along: they print, then the subcommand runs, so
    /// one invocation captures the version and the command's
    /// output together. That is why this replaces clap's
    /// auto-version, which would exit after printing. Without
    /// either, the banner still prints, on stderr.
    #[arg(short = 'V', long = "version", global = true, action = clap::ArgAction::Count)]
    pub version: u8,

    /// Suppress the banner stderr normally carries on every run.
    ///
    /// Only the ambient banner: an explicit `-V` still prints,
    /// since asking for it outranks suppressing it.
    #[arg(long = "no-banner", global = true, action = clap::ArgAction::SetTrue)]
    pub no_banner: bool,

    /// Run even when `jj -V` disagrees with our linked jj-lib.
    ///
    /// Per-invocation on purpose, never a config key: a key gets
    /// set once during a frustrating afternoon and then silently
    /// protects nothing.
    #[arg(long = "allow-jj-mismatch", global = true, action = clap::ArgAction::SetTrue)]
    pub allow_jj_mismatch: bool,

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
    /// Print vc-x1, jj-lib, and jj-data versions
    #[command(
        long_about = "Print every version that describes this run: vc-x1's own,\n\
        the jj-lib it links against, and what each repo's jj data\n\
        records about itself.\n\n\
        The jj-data lines are backend type names, not versions:\n\
        jj records no format version we can read."
    )]
    Version,

    /// Print the changeID for a revision
    Chid(chid::ChidArgs),

    /// Show full description of a commit
    Desc(desc::DescArgs),

    /// List commits in a jj repo
    List(list::ListArgs),

    /// Show commit details and diff summary
    Show(show::ShowArgs),

    /// Display an agent session transcript as a conversation
    #[command(
        name = "agent-session",
        long_about = "Display a Claude Code agent session transcript (.jsonl) as a\n\
        readable conversation.\n\n\
        Output is a set of items (headers, user, assistant, tool,\n\
        thinking, results, meta, summary) each toggled by --<item> /\n\
        --no-<item> (last one wins), with --all / --none as bulk bases.\n\
        The default set (headers, user, assistant, tool, summary) can\n\
        be replaced by [agent-session].items in the user config\n\
        (comma-separated list); CLI flags then adjust the resolved\n\
        set. Malformed lines (e.g. a live session's truncated last\n\
        line) warn to stderr and never fail the run.\n\n\
        Alternate views: --fields (field inventory per entry type),\n\
        --unknown (only unmodeled paths: how the format moved), and\n\
        --raw (pretty-printed source lines). --lines slices by source\n\
        JSONL line: the same unit in every view."
    )]
    BotSession(bot_session::BotSessionArgs),

    /// Rejected pre-0.80.0 name of `agent-session`: prints the fix-it
    /// and exits non-zero, for any flags.
    #[command(name = "bot-session", hide = true, disable_help_flag = true)]
    BotSessionOld {
        /// Swallowed so the fix-it shows for any invocation.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
        rest: Vec<String>,
    },

    /// Run the workspace's configured validation commands
    #[command(long_about = "Run the workspace's configured validation commands.\n\n\
        Reads `[validate] full` (or `fast` with --fast) from the work\n\
        side's config and runs each element as one command, in order,\n\
        from the work repo root, printing each before it runs. The\n\
        first failure stops the run, naming the command and its exit\n\
        status, and the subcommand exits non-zero. An empty or missing\n\
        table is an error, not a pass. See `vc-x1 config work` for the\n\
        table's shape.")]
    Validate(validate::ValidateArgs),

    /// Check the agent repo is published (main matches main@origin)
    #[command(
        name = "validate-agent",
        long_about = "Check the agent repo is published (main matches main@origin).\n\n\
        At rest the agent repo's `main` always matches `main@origin`: the\n\
        bookmark only moves inside a push / squash-push run, which\n\
        publishes it in the same invocation. A mismatch means an earlier\n\
        publish was lost. Read-only and cheap (two jj lookups; no cargo\n\
        steps); also verifies main's remote refs are tracked. Exits\n\
        non-zero on any finding and fixes nothing: resolve with\n\
        `vc-x1 squash-push -R <bot-repo>`."
    )]
    ValidateBot(validate_bot::ValidateBotArgs),

    /// Rejected pre-0.80.0 name of `validate-agent`: prints the fix-it
    /// and exits non-zero, for any flags.
    #[command(name = "validate-bot", hide = true, disable_help_flag = true)]
    ValidateBotOld {
        /// Swallowed so the fix-it shows for any invocation.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
        rest: Vec<String>,
    },

    /// Validate commit descriptions against the other repo
    #[command(
        long_about = "Validate commit descriptions against the other repo.\n\n\
        Output columns: STATUS  CHANGEID  TITLE  [DETAILS]\n\n\
        Status labels:\n  \
          ok   : ochid trailer is valid\n  \
          err  : ochid has issues (wrong prefix, wrong length, ID not found)\n  \
          miss : no ochid trailer; shows match from other repo if found"
    )]
    ValidateDesc(validate_desc::ValidateDescArgs),

    /// Fix commit descriptions against the other repo (dry-run by default)
    #[command(long_about = "Fix commit descriptions against the other repo.\n\n\
        Default is dry-run; use --no-dry-run to write changes.\n\n\
        Output columns: STATUS  CHANGEID  TITLE  [DETAILS]\n\n\
        Status labels:\n  \
          ok    : ochid trailer is valid (no change)\n  \
          fix   : ochid has issues, shows proposed fix (dry-run)\n  \
          fixed : ochid was rewritten (--no-dry-run)\n  \
          add   : missing ochid, match found, shows proposed addition (dry-run)\n  \
          added : missing ochid was added (--no-dry-run)\n  \
          skip  : skipped (no ochid, no match, or max-fixes reached)\n  \
          err   : ID not found and no --fallback provided")]
    FixDesc(fix_desc::FixDescArgs),

    /// Check a markdown record's own anchors and references
    #[command(long_about = "Check a markdown file's same-file links.\n\n\
        Verifies every `](#slug)` link and `[N]: #slug` definition\n\
        resolves to a heading in the same file, every `[[N]]` citation\n\
        has a definition and every definition is cited, and no two\n\
        headings slug to one anchor. Cross-file targets are skipped.\n\
        Read-only; exits non-zero when anything is found. With no\n\
        FILE, checks every `.md` in the workspace.")]
    ValidateAnchors(validate_anchors::ValidateAnchorsArgs),

    /// Check todo-file entry numbering and indent
    #[command(
        long_about = "Check a todo file's `## Todo` and `## Bugs` entry numbering.\n\n\
        Verifies each section is numbered 1..N in document order and\n\
        that continuation-line indent matches the number-prefix width.\n\
        Read-only; exits non-zero if any entry needs fixing: use\n\
        `fix-todo` to rewrite."
    )]
    ValidateTodo(validate_todo::ValidateTodoArgs),

    /// Renumber todo-file entries (dry-run by default)
    #[command(
        long_about = "Renumber a todo file's `## Todo` and `## Bugs` sections.\n\n\
        Renumbers each section 1..N in document order and normalizes\n\
        continuation-line indent to the number-prefix width. Dry-run\n\
        by default: prints each changed entry's corrected line; pass\n\
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
          - `--scope=work|agent|work,agent` dual-repo roles via `.vc-config.toml`\n  \
          - neither             default: `work,agent` when dual, else `work`\n\n\
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
        On failure sync stops where the failing step stopped: nothing\n\
        is auto-reverted, so the state can be inspected. The failure\n\
        report prints each repo's pre-sync op id; undo explicitly with\n\
        `jj op restore <op> -R <repo>`. Nothing persists across\n\
        invocations.\n\n\
        Output shape:\n  \
          - all-up-to-date: one-line summary (`sync: N repos are {}`)\n  \
          - action needed:  per-repo fetch + state + actions\n  \
          - --quiet:        no output; exit code signals success", sync::UP_TO_DATE_MSG))]
    Sync(sync::SyncArgs),

    /// Squash SOURCE into TARGET, advance a bookmark, and push
    #[command(
        long_about = "Squash SOURCE into TARGET (defaults: SOURCE=@, TARGET=@-),\n\
        advance a bookmark, and push.\n\n\
        Captures a repo's trailing working-copy writes into the last\n\
        commit and publishes it: rewriting an already-pushed commit,\n\
        so the push is a forced update. Built for the bot repo\n\
        (`.claude`, the session tail); also useful on the work repo\n\
        as a deliberate amend-and-push.\n\n\
        Zero-ceremony default: bare `vc-x1 squash-push` squashes\n\
        @ -> @- and pushes `main` in `.`. With an empty `@` the squash\n\
        is skipped; if the bookmark already matches the remote the\n\
        command reports \"already sync'd\" and exits 0."
    )]
    SquashPush(squash_push::SquashPushArgs),

    /// Print settable config keys and their defaults
    Config(config_cmd::ConfigArgs),

    /// Commit both repos, push the work repo, squash-push the bot repo
    #[command(long_about = "Commit both repos, push the work repo's BOOKMARK, and\n\
        squash-push the bot repo's `main`: one command.\n\n\
        Collapses the manual commit-push-publish ceremony into a\n\
        single subcommand with two interactive approval gates.\n\n\
        Stages, in order:\n\
        \x20 - review           approve the diff (first gate)\n\
        \x20 - message          $EDITOR, or --title/--body; approve\n\
        \x20                    the text (second gate). Skipped when\n\
        \x20                    neither repo has pending changes\n\
        \x20 - commit-work      commit the work repo. Skipped when\n\
        \x20                    `@` is empty\n\
        \x20 - commit-bot       commit `.claude`. Skipped when it is\n\
        \x20                    clean\n\
        \x20 - bookmark-set     work repo -> <bookmark>, bot -> main\n\
        \x20 - push-work        publish <bookmark> to origin\n\
        \x20 - squash-push-bot  fold `.claude`'s trailing writes\n\
        \x20                    into its commit and push main\n\n\
        Rerunning is always safe: each stage does nothing when its\n\
        work is already done, so a failed run is re-run rather than\n\
        resumed. There is no saved state: vc-x1 cannot know why a\n\
        run failed, so it stops and reports. Please fix and try\n\
        again.\n\n\
        Failures in commit-work / commit-bot / bookmark-set roll both\n\
        repos back via `jj op restore` to a snapshot taken moments\n\
        earlier. Once push-work succeeds the work is published: from\n\
        there a change is either a new commit appended on top by the\n\
        next push, or an amend of what was pushed: `vc-x1\n\
        squash-push` folds the working copy into the last commit and\n\
        force-updates the remote.\n\n\
        vc-x1 runs no build or test steps: run your project's checks\n\
        yourself before pushing.\n\n\
        Non-interactive use: pass both --title and --body plus --yes\n\
        to skip both gates.")]
    Push(push::PushArgs),
}

/// Permanent sanity check for the `main`-bookmark tracking state
/// in both repos of the dual-repo workspace. Emits one line on
/// entry and one on exit of every command. If entry and exit
/// differ, the executing command is the culprit. If entry differs
/// from the previous command's exit, something *between*
/// invocations broke it. Originally added in 0.37.2 as a temporary
/// diagnostic. Promoted to permanent in 0.37.4 after the user
/// reported "happens more than once".
///
/// Emits at `log::debug!` (since 0.52.0-1): default runs stay
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
    // Diagnostics only: an unresolvable / single-repo workspace
    // just probes the work side.
    let mut repos: Vec<(std::path::PathBuf, String)> = vec![(root.clone(), "work".to_string())];
    if let Ok(Some(bot)) = common::bot_repo_path(&root) {
        let label = bot
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "agent".to_string());
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

/// Query whether `bookmark` in `repo` is tracking `remote`.
/// Returns `Ok(true)` when a present, tracked remote ref exists
/// (synced or divergent, both count), `Ok(false)` when it doesn't
/// (not tracking, or the bookmark doesn't exist). The typed view
/// query (`jj::has_tracked_remote`) replaced the CLI-listing
/// parser family here.
fn bm_track_one(repo: &Path, bookmark: &str, remote: &str) -> Result<bool, String> {
    jj::has_tracked_remote(repo, bookmark, remote).map_err(|e| e.to_string())
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

    // Version output rides along with every run so any captured
    // output says which version produced it. Stream by who asked:
    // `-V`/`-VV` are explicit requests, so they go to stdout where
    // a script capturing output collects them. Unasked-for
    // provenance goes to stderr, keeping stdout parseable for the
    // commands that emit data there. The `version` subcommand
    // prints the report itself, so it skips all of this. Uniform
    // `BANNER` (not the `vc-x1-<sub>` form clap's
    // `propagate_version` would print): the version is the
    // binary's regardless of which subcommand it routes to.
    //
    // `eprintln!` rather than the logger because `CliLogger` puts
    // info on stdout by level. The cost is that `--log` does not
    // capture the ambient banner.
    let version_cmd = matches!(cli.command, Some(Commands::Version));
    if !version_cmd {
        match cli.version {
            0 => {
                // Skip the ambient banner when no subcommand was
                // given: that path prints help, whose about line
                // opens with the same banner text, and the pair
                // read as a stuttered header (seen dogfooding the
                // argv0 banner, 2026-08-06).
                if !cli.no_banner && cli.command.is_some() {
                    eprintln!("{}", banner());
                }
            }
            1 => log::info!("{}", banner()),
            _ => {
                for line in version::report(banner()) {
                    log::info!("{line}");
                }
            }
        }
    }

    let Some(cmd) = cli.command else {
        // No subcommand. If `-V` was set the banner or report has
        // already printed, so exit success. Otherwise mirror
        // clap's "a subcommand is required" error by printing
        // usage and exiting non-zero.
        if cli.version > 0 {
            return ExitCode::SUCCESS;
        }
        let mut cmd = cli_with_banner();
        let _ = cmd.print_help();
        return ExitCode::FAILURE;
    };

    // Answered before `Context::load`, which needs a workspace:
    // the versions that describe a run are exactly what you want
    // when the workspace is what's broken.
    if let Commands::Version = cmd {
        for line in version::report(banner()) {
            log::info!("{line}");
        }
        return ExitCode::SUCCESS;
    }

    // The version gate: every subcommand, no exceptions beyond
    // `version`, which returned above. Not scoped to the write path,
    // because "read" in jj-lib means "writes nothing the caller
    // asked for": `load_at_head` merges divergent op heads, the
    // index self-heals by rewriting segments, and an `@`-relative
    // read snapshots the working copy.
    //
    // Nor scoped to a list of repo-touching commands. Such a list
    // enforces only its own completeness: a command that grows a
    // repo read later stays classified as safe, silently. `--help`
    // and shell completion need no exempting, having exited inside
    // clap and `CompleteEnv` before this point. See
    // notes/jj-version-policy.md.
    if !cli.allow_jj_mismatch
        && let Err(e) = version::gate()
    {
        error!("{e}");
        return ExitCode::FAILURE;
    }

    let mut ctx = match context::Context::load() {
        Ok(c) => c,
        Err(e) => {
            error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    match cmd {
        // Handled above, before `Context::load`.
        Commands::Version => ExitCode::SUCCESS,
        Commands::Chid(args) => args.dispatch(&mut ctx),
        Commands::Desc(args) => args.dispatch(&mut ctx),
        Commands::List(args) => args.dispatch(&mut ctx),
        Commands::Show(args) => args.dispatch(&mut ctx),
        Commands::BotSession(args) => args.dispatch(&mut ctx),
        Commands::BotSessionOld { .. } => {
            error!(
                "bot-session: pre-0.80.0 name. The agent side is `agent`, so the \
                 subcommand is `agent-session` (same flags)."
            );
            ExitCode::FAILURE
        }
        Commands::Validate(args) => args.dispatch(&mut ctx),
        Commands::ValidateBot(args) => args.dispatch(&mut ctx),
        Commands::ValidateBotOld { .. } => {
            error!(
                "validate-bot: pre-0.80.0 name. The agent side is `agent`, so the \
                 subcommand is `validate-agent` (same flags)."
            );
            ExitCode::FAILURE
        }
        Commands::ValidateDesc(args) => args.dispatch(&mut ctx),
        Commands::FixDesc(args) => args.dispatch(&mut ctx),
        Commands::ValidateAnchors(args) => args.dispatch(&mut ctx),
        Commands::ValidateTodo(args) => args.dispatch(&mut ctx),
        Commands::FixTodo(args) => args.dispatch(&mut ctx),
        Commands::Clone(args) => args.dispatch(&mut ctx),
        Commands::Init(args) => args.dispatch(&mut ctx),
        Commands::Symlink(args) => args.dispatch(&mut ctx),
        Commands::Sync(args) => args.dispatch(&mut ctx),
        Commands::SquashPush(args) => args.dispatch(&mut ctx),
        Commands::Config(args) => args.dispatch(&mut ctx),
        Commands::Push(args) => args.dispatch(&mut ctx),
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

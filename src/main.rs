mod chid;
mod clone;
mod common;
mod config;
mod desc;
mod desc_helpers;
mod finalize;
mod fix_desc;
mod init;
mod list;
mod logging;
mod push;
mod repo_url;
mod scope;
mod show;
mod symlink;
mod sync;
#[cfg(test)]
mod test_helpers;
mod toml_simple;
mod validate_desc;

use std::path::Path;
use std::process::ExitCode;

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use clap_complete::CompleteEnv;
use log::error;

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
#[command(
    version,
    propagate_version = true,
    about = TOP_ABOUT,
    max_term_width = 80
)]
pub struct Cli {
    /// Verbose output: -v debug, -vv trace
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Log file path (captures all levels)
    #[arg(long, global = true)]
    pub log: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub(crate) command: Commands,
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

    /// Clone a dual-repo project
    Clone(clone::CloneArgs),

    /// Create a new dual-repo project
    Init(init::InitArgs),

    /// Create Claude Code project symlink
    Symlink(symlink::SymlinkArgs),

    /// Fetch and sync a set of repos to their remotes
    #[command(long_about = "Fetch and sync a set of repos to their remotes.\n\n\
        Repo set is resolved (in order):\n  \
          - `-R` / `--repo`     exact list (back-compat / arbitrary multi-repo)\n  \
          - `--scope=code|bot|code,bot` dual-repo roles via `.vc-config.toml`\n  \
          - neither             default: `code,bot` when dual, else `code`\n\n\
        Default is --check (verify only) — fatal if any repo needs\n\
        action. Re-run with --no-check to apply (rebase / fast-forward).\n\
        Per repo:\n  \
          - up-to-date        nothing to do\n  \
          - behind            fast-forward bookmark to remote\n  \
          - ahead             nothing to sync (local has unpushed work)\n  \
          - diverged          rebase local onto remote; fail on conflicts\n  \
          - no remote         bookmark has no @<remote> counterpart; skip\n\n\
        Scripts and automation should pass --check or --no-check\n\
        explicitly — defaults can shift, explicit flags lock in the\n\
        contract. Interactive use can rely on the default.\n\n\
        After the bookmark action, `@` is rebased onto the (possibly\n\
        advanced) bookmark if it isn't already a descendant, so trailing\n\
        working-copy writes (e.g. `.claude`'s `/exit` tail) don't end up\n\
        orphaned on a stale branch.\n\n\
        On any failure, every repo is reverted to its starting state via\n\
        `jj op restore`. Working-copy files are preserved across the\n\
        revert — the operation log rewinds but disk content stays.\n\n\
        Output shape:\n  \
          - all-up-to-date: one-line summary (`sync: N repos, all bookmarks up-to-date`)\n  \
          - action needed (--check):  per-repo fetch + state + fatal error\n  \
          - action needed (--no-check): per-repo fetch + state + actions\n  \
          - --quiet:        no output; exit code signals success")]
    Sync(sync::SyncArgs),

    /// Squash, set bookmark, and/or push a jj repo
    #[command(long_about = "Squash, set bookmark, and/or push a jj repo.\n\n\
        Designed for the bot to atomically finalize its session repo:\n\
        --detach exits immediately, --delay waits for trailing writes,\n\
        --squash folds them in, --bookmark + --push sends it upstream.\n\
        Every flag is opt-in. See README.md for details.")]
    Finalize(finalize::FinalizeArgs),

    /// Dual-repo commit+push+finalize in one resumable command
    #[command(
        long_about = "Dual-repo commit+push+finalize in one resumable command.\n\n\
        Collapses today's manual Commit-Push-Finalize Flow into a\n\
        single subcommand with two interactive approval gates and a\n\
        state machine with persistent progress so interruptions can\n\
        resume without re-doing completed stages.\n\n\
        Stages: preflight (fmt/clippy/test) → review (approve diff)\n\
        → message ($EDITOR / --title+--body, approve text) →\n\
        commit-app → commit-claude (skipped if clean) → bookmark-both\n\
        → push-app → finalize-claude. Failures in commit-app /\n\
        commit-claude / bookmark-both roll both repos back via\n\
        `jj op restore` to the snapshot recorded before commit-app.\n\
        After push-app succeeds the remote boundary is crossed and\n\
        recovery is forward-only.\n\n\
        Non-interactive use: pass both --title and --body plus --yes\n\
        to skip the review gate. Saved state carries title/body\n\
        across resumes so only the first invocation needs them."
    )]
    Push(push::PushArgs),
}

fn run_command(result: Result<(), Box<dyn std::error::Error>>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// Permanent sanity check for the `main`-bookmark tracking state
/// in both repos of the dual-repo workspace. Prints one line on
/// entry and one on exit of every command. If entry and exit
/// differ, the executing command is the culprit; if entry differs
/// from the previous command's exit, something *between*
/// invocations broke it. Originally added in 0.37.2 as a temporary
/// diagnostic; promoted to permanent in 0.37.4 after the user
/// reported "happens more than once" — the signal is worth keeping
/// in place until confidence in its correctness is well-established
/// across varied use, at which point a "silent when clean"
/// refinement would remove the steady-state noise without losing
/// detection value (see `notes/todo.md`).
///
/// Walks up from cwd to locate the workspace root (the directory
/// whose `.vc-config.toml` has `path = "/"`), then probes `<root>`
/// and `<root>/.claude`. Same labeling whether the user runs from
/// the app root, from `.claude`, or from any subdir.
fn bm_track(phase: &str, command_name: &str) {
    let header = format!("bm-track {phase} vc-x1 {command_name}");
    let root = match common::find_workspace_root() {
        Some(r) => r,
        None => {
            log::info!("{header}: no-workspace");
            return;
        }
    };
    let repos: [(std::path::PathBuf, &str); 2] =
        [(root.clone(), "app"), (root.join(".claude"), ".claude")];
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
    log::info!("{header}: {}", parts.join(", "));
}

/// Query jj for whether `bookmark` in `repo` is tracking `remote`.
/// Returns `Ok(true)` if the tracked-list entry for `bookmark` shows
/// an `@<remote>` line (in any form — `@origin:` when synced, or
/// `@origin (ahead by N commits):` / similar when divergent — both
/// still count as tracking). `Ok(false)` when no such line exists
/// (bookmark isn't tracking this remote or doesn't exist). `Err` on
/// subprocess failure.
fn bm_track_one(repo: &Path, bookmark: &str, remote: &str) -> Result<bool, String> {
    let repo_str = repo.to_string_lossy();
    let output = std::process::Command::new("jj")
        .args(["bookmark", "list", "--tracked", bookmark, "-R", &repo_str])
        .output()
        .map_err(|e| format!("spawn: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Accept both the synced form (`@origin:`) and the decorated
    // divergent form (`@origin (ahead by N commits):`), which still
    // represents a tracking relationship.
    let colon = format!("@{remote}:");
    let paren = format!("@{remote} ");
    Ok(stdout.lines().any(|l| {
        let t = l.trim_start();
        t.starts_with(&colon) || t.starts_with(&paren)
    }))
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

    // The detached `finalize --exec` re-entry is the bot's
    // session-end child; it shouldn't print the banner (extra
    // chatter in its log) or surface user-facing failure markers
    // (those are meant for the user's next interactive run).
    let is_detached_exec = matches!(
        cli.command,
        Commands::Finalize(ref f) if f.exec
    );

    // `-L` / `--no-label` on the read-only multi-repo subcommands
    // (chid, desc, list, show) makes their output script-parseable
    // — adding a leading `vc-x1 X.Y.Z` line would break that. Skip
    // the banner whenever the active subcommand has the flag set.
    let suppress_banner = match &cli.command {
        Commands::Chid(a) => a.common.no_label,
        Commands::Desc(a) => a.common.no_label,
        Commands::List(a) => a.common.no_label,
        Commands::Show(a) => a.common.no_label,
        _ => false,
    };

    if !is_detached_exec {
        if !suppress_banner {
            // Banner on every normal run, mirroring what `--help`
            // shows at the top.
            log::info!("{BANNER}");
        }
        finalize::surface_previous_failures();
    }

    // Command name for bm-track output (first positional arg after
    // the binary; clap has already validated it by the time we get here).
    let command_name = std::env::args().nth(1).unwrap_or_else(|| "?".to_string()); // OK: default when somehow invoked without a subcommand

    if !is_detached_exec {
        bm_track("enter", &command_name);
    }

    let exit_code = match cli.command {
        Commands::Chid(chid_args) => run_command(chid::chid(&chid_args)),
        Commands::Desc(desc_args) => run_command(desc::desc(&desc_args)),
        Commands::List(list_args) => run_command(list::list(&list_args)),
        Commands::Show(show_args) => run_command(show::show(&show_args)),
        Commands::ValidateDesc(validate_desc_args) => {
            run_command(validate_desc::validate_desc(&validate_desc_args))
        }
        Commands::FixDesc(fix_desc_args) => run_command(fix_desc::fix_desc(&fix_desc_args)),
        Commands::Clone(clone_args) => run_command(clone::clone_repo(&clone_args)),
        Commands::Init(init_args) => run_command(init::init(&init_args)),
        Commands::Symlink(symlink_args) => run_command(symlink::symlink(&symlink_args)),
        Commands::Sync(sync_args) => run_command(sync::sync(&sync_args)),
        Commands::Finalize(finalize_args) => {
            let opts = match finalize_args.into_opts(cli.log) {
                Ok(opts) => opts,
                Err(e) => {
                    error!("{e}");
                    return ExitCode::FAILURE;
                }
            };
            run_command(finalize::finalize(&opts))
        }
        Commands::Push(push_args) => run_command(push::push(&push_args)),
    };

    if !is_detached_exec {
        bm_track("exit ", &command_name);
    }

    exit_code
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

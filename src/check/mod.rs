//! The `check` subcommand: vc-x1's own checks of a workspace, its
//! config, its records, its commit descriptions, and its agent repo,
//! each read-only and exiting non-zero on a finding.
//!
//! - `check agent`, `check desc`, `check config`, `check anchors`, and
//!   `check todo` were `validate-agent`, `validate-desc`,
//!   `validate-config`, `validate-anchors`, and `validate-todo` before
//!   0.85.25. The old names still run, deprecated, each warning with
//!   the new by [`deprecated_name`], and a later release refuses them.
//!   `validate-bot`, the pre-0.80.0 `validate-agent`, was refused
//!   before, and is refused naming `check agent` by [`retired_name`].
//! - They are the domain's checks, apart from the work's validation,
//!   which is the project's own (`cargo xtask validate` here).

pub mod agent;
pub mod anchors;
pub mod config;
pub mod desc;
pub mod todo;

use std::process::ExitCode;

use clap::{Args, Subcommand};

use log::warn;

use crate::context::Context;
use crate::subcommand::SubcommandRunner;

/// Check the workspace.
#[derive(Args, Debug)]
pub struct CheckArgs {
    #[command(subcommand)]
    pub what: CheckCommand,
}

/// What `check` checks.
#[derive(Subcommand, Debug)]
pub enum CheckCommand {
    /// Check the agent repo is published (main matches main@origin)
    #[command(
        long_about = "Check the agent repo is published (main matches main@origin).\n\n\
        At rest the agent repo's `main` always matches `main@origin`: the\n\
        bookmark only moves inside a push / squash-push run, which\n\
        publishes it in the same invocation. A mismatch means an earlier\n\
        publish was lost. Read-only and cheap (two jj lookups; no cargo\n\
        steps); also verifies main's remote refs are tracked. Exits\n\
        non-zero on any finding and fixes nothing: resolve with\n\
        `vc-x1 squash-push -s a`."
    )]
    Agent(agent::CheckAgentArgs),

    /// Check commit descriptions against the other repo
    #[command(long_about = "Check commit descriptions against the other repo.\n\n\
        Output columns: STATUS  CHANGEID  TITLE  [DETAILS]\n\n\
        Status labels:\n  \
          ok   : ochid trailer is valid\n  \
          err  : ochid has issues (wrong prefix, wrong length, ID not found), or\n         \
          an orepo names the wrong repo\n  \
          note : ok, with something worth saying, an agent commit not yet pushed\n  \
          miss : no ochid trailer; shows match from other repo if found")]
    Desc(desc::CheckDescArgs),

    /// Check the workspace's config files
    #[command(long_about = "Check a workspace's config files.\n\n\
        Verifies every key is one the schema declares for that side,\n\
        the legacy schema is rejected, a dual workspace's two [repos]\n\
        registries agree, the file's own links resolve, and a side's\n\
        config carries `repos.work`. Read-only; exits non-zero when\n\
        anything is found. Was `config --validate` before 0.80.6.")]
    Config(config::CheckConfigArgs),

    /// Check a markdown record's own anchors and references
    #[command(long_about = "Check a markdown file's same-file links.\n\n\
        Verifies every `](#slug)` link and `[N]: #slug` definition\n\
        resolves to a heading in the same file, every `[[N]]` citation\n\
        has a definition and every definition is cited, and no two\n\
        headings slug to one anchor. Cross-file targets are skipped.\n\
        Read-only; exits non-zero when anything is found. With no\n\
        FILE, checks every `.md` in the workspace.")]
    Anchors(anchors::CheckAnchorsArgs),

    /// Check todo-file entry numbering and indent
    #[command(
        long_about = "Check a todo file's `## Todo` and `## Bugs` entry numbering.\n\n\
        Verifies each section is numbered 1..N in document order and\n\
        that continuation-line indent matches the number-prefix width.\n\
        Read-only; exits non-zero if any entry needs fixing: use\n\
        `fix-todo` to rewrite."
    )]
    Todo(todo::CheckTodoArgs),
}

impl CheckArgs {
    /// Run the check named.
    pub fn dispatch(&self, ctx: &mut Context) -> ExitCode {
        match &self.what {
            CheckCommand::Agent(args) => args.dispatch(ctx),
            CheckCommand::Desc(args) => args.dispatch(ctx),
            CheckCommand::Config(args) => args.dispatch(ctx),
            CheckCommand::Anchors(args) => args.dispatch(ctx),
            CheckCommand::Todo(args) => args.dispatch(ctx),
        }
    }
}

/// The arguments of a retired name, swallowed so its fix-it shows for
/// any invocation.
#[derive(Args, Debug)]
#[command(disable_help_flag = true)]
pub struct RetiredArgs {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
    pub rest: Vec<String>,
}

/// The fix-it for a retired check name, `old`, now `check <new>`,
/// its flags unchanged.
pub fn retired_name(old: &str, new: &str) -> String {
    format!("Retired `vc-x1 {old}`: use `vc-x1 check {new}` instead")
}

/// The warning for a deprecated check name, `old`, now `check <new>`,
/// its flags unchanged, which still runs.
pub fn deprecation(old: &str, new: &str) -> String {
    format!("Deprecated `vc-x1 {old}`: use `vc-x1 check {new}` instead")
}

/// Warn that `old` is deprecated, before the check it names runs.
pub fn deprecated_name(old: &str, new: &str) {
    warn!("{}", deprecation(old, new));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fix-it and the warning name the old command and the new.
    #[test]
    fn a_retired_or_deprecated_name_names_its_check() {
        assert_eq!(
            retired_name("validate-bot", "agent"),
            "Retired `vc-x1 validate-bot`: use `vc-x1 check agent` instead"
        );
        assert_eq!(
            deprecation("validate-config", "config"),
            "Deprecated `vc-x1 validate-config`: use `vc-x1 check config` instead"
        );
    }

    /// `validate-bot` parses, whatever flags follow, to its refusal,
    /// each deprecated name to its check's own arguments, and each
    /// check by its new name.
    #[test]
    fn the_old_names_parse_to_a_refusal_or_their_check() {
        use crate::{Cli, Commands};
        use clap::Parser;
        let parse = |args: &[&str]| Cli::try_parse_from(args).expect("parses").command;
        let bot = parse(&["vc-x1", "validate-bot", "-s", "b", "--bogus"]);
        assert!(matches!(bot, Some(Commands::ValidateBotOld(_))), "{bot:?}");
        for (old, flags, expect) in [
            ("validate-agent", &[][..], "ValidateAgentDeprecated("),
            ("validate-desc", &["-s", "b"][..], "ValidateDescDeprecated("),
            ("validate-config", &[][..], "ValidateConfigDeprecated("),
            ("validate-anchors", &[][..], "ValidateAnchorsDeprecated("),
            ("validate-todo", &[][..], "ValidateTodoDeprecated("),
        ] {
            let mut args = vec!["vc-x1", old];
            args.extend(flags);
            let parsed = format!("{:?}", parse(&args));
            assert!(parsed.contains(expect), "{old}: {parsed}");
        }
        assert!(
            Cli::try_parse_from(["vc-x1", "validate-config", "--bogus"]).is_err(),
            "a deprecated name takes its check's flags alone"
        );
        for (new, flags, expect) in [
            ("agent", &[][..], "Agent"),
            ("desc", &["-s", "b"][..], "Desc"),
            ("config", &[][..], "Config"),
            ("anchors", &[][..], "Anchors"),
            ("todo", &[][..], "Todo"),
        ] {
            let mut args = vec!["vc-x1", "check", new];
            args.extend(flags);
            let parsed = format!("{:?}", parse(&args));
            assert!(parsed.contains(&format!("{expect}(")), "{new}: {parsed}");
        }
    }
}

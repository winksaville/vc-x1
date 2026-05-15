//! `SubcommandRunner` trait
//!
//! The purpose is to simplify invoking a subcommand so `main`
//! can invoke any subcommand with one line:
//!
//! `Commands::Chid(args) => args.dispatch(&ctx),`
//!
//! Instead of the ~11 lines that the default `dispatch` below
//! now encapsulates.
//!
//! Implementors provide:
//!
//! - [`SubcommandRunner::to_params`] — build the clap-free
//!   `Params` from the clap `Args` (covers both `From` /
//!   `TryFrom` shapes uniformly via `Result<_, String>`).
//! - [`SubcommandRunner::run`] — the subcommand body.
//!
//! The default [`SubcommandRunner::dispatch`] builds `Params`
//! via `to_params`, runs via `run`, and maps the result to
//! `ExitCode`. Errors at any stage log via `error!` and return
//! `ExitCode::FAILURE`. `Context` is loaded once in `main` and
//! passed in by reference so a single `Context` is shared across
//! the (one) match arm that actually runs.
//!
//! ## See also
//!
//! - The wider CLI-args / subcommand-layer split this trait
//!   formalizes: [`ARCHITECTURE.md > args → Context + Params`][arch].
//! - Cycle design, ladder, evaluation gate:
//!   [`chores-10.md > 0.50.0-0`][open].
//! - Worked example, trait-shape decisions, naming:
//!   [`chores-10.md > 0.50.0-1`][port].
//! - First implementor: [`crate::chid`].
//!
//! [arch]: https://github.com/winksaville/vc-x1/blob/main/ARCHITECTURE.md#args--context--params
//! [open]: https://github.com/winksaville/vc-x1/blob/main/notes/chores/chores-10.md#chore-open-subcommand-trait-sweep-0500-0
//! [port]: https://github.com/winksaville/vc-x1/blob/main/notes/chores/chores-10.md#refactor-subcommandrunner-trait--chid-0500-1

use std::process::ExitCode;

use log::error;

use crate::context::Context;

/// Trait implemented by every subcommand's clap `Args` type so
/// `main.rs` can dispatch via a single `args.dispatch(&ctx)`
/// call.
pub trait SubcommandRunner {
    /// The clap-free `Params` struct the subcommand body
    /// consumes.
    type Params;

    /// Build `Params` from `Args`. `Result<_, String>` absorbs
    /// both `From` (return `Ok(...)`) and `TryFrom` (forward
    /// the error) shapes uniformly.
    fn to_params(&self) -> Result<Self::Params, String>;

    /// Run the subcommand body.
    fn run(ctx: &Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>>;

    /// Whether to suppress the leading `vc-x1 X.Y.Z` banner.
    /// Default `false`; commands whose `Params` carry a
    /// banner-suppression flag (e.g. `chid` / `desc` / `list` /
    /// `show` under `-L`) override to read it.
    fn suppress_banner(_params: &Self::Params) -> bool {
        false
    }

    /// Whether this invocation is the detached `finalize --exec`
    /// child. Default `false`; `finalize` overrides to read it
    /// from its `Params`. Consumed by `dispatch` for both the
    /// banner suppression (via `crate::sb_ide`) and the
    /// `bm_track` enter/exit gating (the detached child shouldn't
    /// emit user-facing bookmark-tracking lines).
    fn is_detached_exec(_params: &Self::Params) -> bool {
        false
    }

    /// Default dispatch: build `Params` via `to_params`, emit
    /// session chrome via [`crate::sb_ide`], bracket the run with
    /// `crate::bm_track` enter/exit (skipped when
    /// `is_detached_exec`), execute via `run`, and map the result
    /// to `ExitCode`. Errors at any stage log via `error!` and
    /// return `ExitCode::FAILURE`.
    fn dispatch(&self, ctx: &Context) -> ExitCode {
        let params = match self.to_params() {
            Ok(p) => p,
            Err(e) => {
                error!("{e}");
                return ExitCode::FAILURE;
            }
        };
        let is_detached = Self::is_detached_exec(&params);
        crate::sb_ide(Self::suppress_banner(&params), is_detached);

        // Command name is the first positional after the binary;
        // clap has already validated it by the time we reach
        // dispatch (top-level parse errors exit earlier).
        let command_name = std::env::args().nth(1).unwrap_or_else(|| "?".to_string()); // OK: default when somehow invoked without a subcommand

        if !is_detached {
            crate::bm_track("enter", &command_name);
        }
        let exit_code = match Self::run(ctx, &params) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                error!("{e}");
                ExitCode::FAILURE
            }
        };
        if !is_detached {
            crate::bm_track("exit ", &command_name);
        }
        exit_code
    }
}

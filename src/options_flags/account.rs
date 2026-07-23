//! `--account` — selects an account section in the user config.
//! See [options_flags](README.md) for shared architecture.

use clap::Args;

/// `--account` leaf (Option — non-boolean domain) — see
/// [Consuming an OF](README.md#consuming-an-of).
#[derive(Args, Debug, Clone, Default)]
pub struct AccountOption {
    /// Account name — picks `[account.<a>]` from user config.
    ///
    /// - Without this flag, `[default].account` (or top-level
    ///   `[repo]` shorthand) is used.
    /// - Meaningful only with Path or bare-NAME targets — URL /
    ///   owner/name targets supply the remote directly.
    #[arg(
        id = "account",
        long = "account",
        value_name = "NAME",
        verbatim_doc_comment
    )]
    pub value: Option<String>,
}

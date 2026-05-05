//! `--private` — create private repos on the remote provisioner.
//! See [options_flags](README.md) for shared architecture.

use clap::Args;

/// `--private` leaf (Flag — boolean domain) — see
/// [Consuming an OF](README.md#consuming-an-of).
#[derive(Args, Debug, Clone, Default)]
pub struct PrivateFlag {
    /// Create private GitHub repos (default: public).
    ///
    /// - Only meaningful when the resolved provisioner is
    ///   `gh repo create` (GitHub URL or `--repo remote` whose
    ///   value points at GitHub).
    #[arg(long, verbatim_doc_comment)]
    pub private: bool,
}

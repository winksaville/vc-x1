//! `--use-template` — seed repos from template directories.
//! See [options_flags](README.md) for shared architecture.

use clap::Args;

/// `--use-template` leaf (Option — non-boolean domain) — see
/// [Consuming an OF](README.md#consuming-an-of).
#[derive(Args, Debug, Clone, Default)]
pub struct UseTemplateOption {
    /// Seed repos from template directories.
    ///
    /// Value is `CODE[,BOT]`. Default bot path is `<CODE>.claude`
    /// (file-name concat, not path join — templates are siblings).
    ///
    /// - With `--scope=por`: only `CODE` is used; passing `,BOT`
    ///   is fatal (no session side to seed).
    /// - Non-hidden contents copied recursively; hidden entries
    ///   (names starting with `.`) are skipped — init writes its
    ///   own hidden files.
    /// - If a copied tree has a `README.md`, its first line is
    ///   rewritten to `# <repo-name>`.
    #[arg(long, value_name = "CODE[,BOT]", verbatim_doc_comment)]
    pub use_template: Option<String>,
}

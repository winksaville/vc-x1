//! `--use-template` — seed repos from template directories.
//! See [options_flags](README.md) for shared architecture.

use clap::Args;

/// `--use-template` leaf (Option — non-boolean domain) — see
/// [Consuming an OF](README.md#consuming-an-of).
#[derive(Args, Debug, Clone, Default)]
pub struct UseTemplateOption {
    /// Seed repos from template directories.
    ///
    /// Value is `WORK[,BOT]`. Default bot path is `<CODE>.claude`
    /// (file-name concat, not path join — templates are siblings).
    ///
    /// - With `--scope=por`: only `WORK` is used; passing `,BOT`
    ///   is fatal (no bot side to seed).
    /// - Non-hidden contents copied recursively; hidden entries
    ///   (names starting with `.`) are skipped — init writes its
    ///   own hidden files.
    /// - If a copied tree has a `README.md`, its first line is
    ///   rewritten to `# <repo-name>`.
    #[arg(
        id = "use_template",
        long = "use-template",
        value_name = "WORK[,BOT]",
        verbatim_doc_comment
    )]
    pub value: Option<String>,
}

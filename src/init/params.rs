//! Flat plain-struct input to the init op.
//!
//! `InitParams` is the clap-free shape that `pub fn init` reads
//! from. Built `From<&InitArgs>` at the binary edge (main.rs);
//! callers without clap (tests, future TUI / library embedding)
//! construct it directly.
//!
//! Per `notes/chores/chores-09.md > ## InitParams implementation
//! (0.44.0)`: domain types pass through (`RepoSelector`,
//! `ConfigKind`, `PushRetryOptions`); leaf clap wrappers do not.

use crate::config::RepoSelector;
use crate::init::InitArgs;
use crate::options_flags::config::ConfigKind;
use crate::options_flags::push_retry::PushRetryOptions;

/// Inputs to the init op, flat, owned, clap-free.
///
/// - `target`: the TARGET positional argument (URL, owner/name,
///   path, or bare NAME).
/// - `name`: optional `NAME` positional override.
/// - `account`: `--account` value (None ⇒ default account chain).
/// - `repo`: `--repo` value parsed into a `RepoSelector`.
/// - `por`: `--por` resolved — `true` for plain single repo,
///   `false` (default) for dual workspace.
/// - `private`: `--private`.
/// - `dry_run`: `--dry-run`.
/// - `push_retry`: `--push-retries` / `--push-retry-delay`.
/// - `use_template`: `--use-template` value.
/// - `config`: resolved `--config` value (None ⇒ canned write).
/// - `create_symlink`: whether `init` should create the
///   `~/.claude/projects/` symlink for dual runs (production
///   path = true; test fixtures suppress with false).
pub struct InitParams {
    pub target: String,
    pub name: Option<String>,
    pub account: Option<String>,
    pub repo: Option<RepoSelector>,
    pub por: bool,
    pub private: bool,
    pub dry_run: bool,
    pub push_retry: PushRetryOptions,
    pub use_template: Option<String>,
    pub config: Option<ConfigKind>,
    pub create_symlink: bool,
}

impl From<&InitArgs> for InitParams {
    /// Convert clap-derived `InitArgs` into the flat `InitParams`.
    ///
    /// Production path: sets `create_symlink: true`. Tests that
    /// need `create_symlink: false` construct `InitParams`
    /// directly (or post-edit the field).
    fn from(a: &InitArgs) -> Self {
        Self {
            target: a.target.clone(),
            name: a.name.clone(),
            account: a.account.value.clone(),
            repo: a.repo.value.clone(),
            por: a.por.value,
            private: a.provision.private.value,
            dry_run: a.provision.dry_run.value,
            push_retry: a.provision.push_retry.clone(),
            use_template: a.use_template.value.clone(),
            config: a.config.resolve(ConfigKind::None),
            create_symlink: true,
        }
    }
}

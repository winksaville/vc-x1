//! `ProvisionOptionFlagBundle` — bundle of OFs shared by every
//! provisioning subcommand (today `init`; later `clone` once it
//! migrates). Composes `DryRunFlag`, `PrivateFlag` (Flag leaves)
//! and `PushRetryOptions` (Option leaf), so a consumer picks them
//! all up with one `#[command(flatten)]` line. Mixed-domain →
//! implements `OptionFlagBundle`. See [options_flags](README.md) for
//! shared architecture.

use clap::Args;

use super::dry_run::DryRunFlag;
use super::private::PrivateFlag;
use super::push_retry::PushRetryOptions;

/// Provisioning bundle — see [Bundle](README.md#architecture).
///
/// - `dry_run` — `--dry-run` (Flag).
/// - `private` — `--private` (Flag).
/// - `push_retry` — `--push-retries` + `--push-retry-delay`
///   (Option).
#[derive(Args, Debug, Clone, Default)]
pub struct ProvisionOptionFlagBundle {
    #[command(flatten)]
    pub dry_run: DryRunFlag,

    #[command(flatten)]
    pub private: PrivateFlag,

    #[command(flatten)]
    pub push_retry: PushRetryOptions,
}

impl super::OptionFlagBundle for ProvisionOptionFlagBundle {}

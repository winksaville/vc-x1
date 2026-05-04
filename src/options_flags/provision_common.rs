//! `ProvisionCommon` — bundle of OFs shared by every provisioning
//! subcommand (today `init`; later `clone` once it migrates).
//! Composes `DryRunFlag`, `PrivateFlag`, and `PushRetryFlags` so a
//! consumer picks them all up with one `#[command(flatten)]` line.
//! See [options_flags](README.md) for shared architecture.

use clap::Args;

use super::dry_run::DryRunFlag;
use super::private::PrivateFlag;
use super::push_retry::PushRetryFlags;

/// Provisioning bundle — see [Bundle](README.md#architecture).
///
/// - `dry_run` — `--dry-run` (boolean).
/// - `private` — `--private` (boolean).
/// - `push_retry` — `--push-retries` + `--push-retry-delay`.
#[derive(Args, Debug, Clone, Default)]
pub struct ProvisionCommon {
    #[command(flatten)]
    pub dry_run: DryRunFlag,

    #[command(flatten)]
    pub private: PrivateFlag,

    #[command(flatten)]
    pub push_retry: PushRetryFlags,
}

impl super::FlagBundle for ProvisionCommon {}

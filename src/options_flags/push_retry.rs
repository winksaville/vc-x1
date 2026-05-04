//! `--push-retries` + `--push-retry-delay` — retry policy for
//! the post-create `git push` (waits out remote provisioner
//! propagation lag, e.g. GitHub's async repo creation).
//! See [options_flags](README.md) for shared architecture.

use clap::Args;

/// `--push-retries` / `--push-retry-delay` leaf — see
/// [Consuming an OF](README.md#consuming-an-of).
#[derive(Args, Debug, Clone)]
pub struct PushRetryFlags {
    /// Max push retries after repo creation [default: 5]
    #[arg(long, default_value_t = 5)]
    pub push_retries: u32,

    /// Seconds between push retries [default: 3]
    #[arg(long, default_value_t = 3)]
    pub push_retry_delay: u64,
}

impl super::FlagBundle for PushRetryFlags {}

impl Default for PushRetryFlags {
    /// Mirrors the clap defaults so fixture code can use
    /// `PushRetryFlags::default()` and stay aligned with what
    /// clap produces when the flags are absent.
    fn default() -> Self {
        Self {
            push_retries: 5,
            push_retry_delay: 3,
        }
    }
}

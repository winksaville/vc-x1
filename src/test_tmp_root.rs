//! Shared `resolve_tmp_root` for unit and integration tests.
//!
//! Pulled in from two contexts:
//! - `src/test_helpers.rs` (binary's `#[cfg(test)]` unit-test
//!   harness) via `mod test_tmp_root;` declared at the crate root.
//! - `tests/common/mod.rs` (integration-test crate) via
//!   `#[path = "../../src/test_tmp_root.rs"] mod test_tmp_root;`.
//!
//! Single source of truth. Keep this file **dependency-free** —
//! pure `std::env` + `std::path::PathBuf` — so it compiles in
//! both contexts. No `crate::…` imports; the integration-test
//! crate has a different crate root and would fail to resolve
//! them.
//!
//! Future work (per `notes/todo.md`) extends the priority chain
//! through `~/.config/vc-x1/config.toml` and project-local
//! `.vc-config.toml`. At that point, config-loading should pass
//! its result *in* (function arg) rather than be invoked from
//! here, preserving the dependency-free constraint.

use std::path::PathBuf;

/// Resolve the parent directory for test tempdirs.
///
/// Priority: `$VC_X1_TEST_TMPDIR` (if set and non-empty) →
/// `std::env::temp_dir()` (= `$TMPDIR` on Unix, else `/tmp`).
pub fn resolve_tmp_root() -> PathBuf {
    if let Ok(p) = std::env::var("VC_X1_TEST_TMPDIR")
        && !p.is_empty()
    {
        return PathBuf::from(p);
    }
    std::env::temp_dir()
}

/// Whether to preserve test tempdirs across `Drop`.
///
/// Returns `true` if `$VC_X1_TEST_KEEP` is set and non-empty.
/// Consulted by RAII `Drop` impls (`Fixture`, `FixturePor`,
/// `CliFixture`) to suppress tempdir cleanup for debugging:
///
/// ```bash
/// VC_X1_TEST_KEEP=1 cargo test -- --nocapture
/// ```
///
/// Pairs with `--nocapture` so the per-fixture stderr line
/// announcing the preserved path is visible.
pub fn should_keep_tempdir() -> bool {
    keep_decision(std::env::var("VC_X1_TEST_KEEP").ok().as_deref())
}

/// Pure-policy form of `should_keep_tempdir`. Takes the env-var
/// value (`None` if unset, `Some("")` if set-but-empty) and
/// returns whether to preserve. Factored out so the decision is
/// unit-testable without manipulating global env state — env
/// mutation is not thread-safe and would race with other tests
/// reading `VC_X1_TEST_KEEP` from `Drop`.
fn keep_decision(env_value: Option<&str>) -> bool {
    match env_value {
        None => false,     // env var not set
        Some("") => false, // env var set but explicitly empty
        Some(_) => true,   // any non-empty value → preserve
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `$VC_X1_TEST_KEEP` unset → no preserve.
    #[test]
    fn keep_decision_unset_is_false() {
        assert!(!keep_decision(None));
    }

    /// `$VC_X1_TEST_KEEP=""` → no preserve. Empty-string is the
    /// "set but explicitly disabled" form (matches how env vars
    /// are unset on some shells via `VAR=`).
    #[test]
    fn keep_decision_empty_is_false() {
        assert!(!keep_decision(Some("")));
    }

    /// Any non-empty value → preserve. Aligns with the conventional
    /// env-var-as-flag pattern (`VAR=1`, `VAR=yes`, `VAR=true` —
    /// or even `VAR=0`, since the policy treats *any* non-empty
    /// value as "set").
    #[test]
    fn keep_decision_nonempty_is_true() {
        assert!(keep_decision(Some("1")));
        assert!(keep_decision(Some("yes")));
        assert!(keep_decision(Some("true")));
        assert!(keep_decision(Some("0")));
    }
}

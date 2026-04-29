//! CLI subprocess test: `$VC_X1_TEST_KEEP` preserves the fixture
//! across `Drop`.
//!
//! Single-test file by design — env-var writes are not thread-safe
//! (`std::env::set_var` is `unsafe` since Rust 1.83) and would race
//! with sibling tests reading `$VC_X1_TEST_KEEP` from their own
//! `Drop` impls. Cargo compiles each `tests/*.rs` as its own
//! binary, so this process is isolated from `cli_init` /
//! `cli_smoke`.

mod common;

use common::CliFixture;

/// `$VC_X1_TEST_KEEP=1` makes `CliFixture::drop` skip
/// `remove_dir_all`. After the fixture goes out of scope, the
/// tempdir must still exist on disk; the test removes it manually.
#[test]
fn keep_env_preserves_fixture_across_drop() {
    // SAFETY: `set_var` is unsafe due to non-thread-safe libc
    // implementations. This is the only test in this binary, so no
    // other thread in this process is reading `$VC_X1_TEST_KEEP`
    // concurrently. Other test binaries run as separate processes
    // and don't share env state.
    unsafe {
        std::env::set_var("VC_X1_TEST_KEEP", "1");
    }

    let preserved_path = {
        let fx = CliFixture::new("keep-env");
        assert!(
            fx.base.exists() && fx.base.is_dir(),
            "fixture base must exist mid-test"
        );
        fx.base.clone()
        // fx drops here; should NOT remove the tree because
        // `should_keep_tempdir()` returns true.
    };

    assert!(
        preserved_path.exists(),
        "VC_X1_TEST_KEEP=1 should have preserved {}",
        preserved_path.display()
    );
    assert!(
        preserved_path.join("home").exists(),
        "preserved tree should include the home subdir created by CliFixture::new"
    );

    // Clean up manually since Drop didn't.
    std::fs::remove_dir_all(&preserved_path).expect("cleanup preserved fixture");

    // Restore env state. SAFETY: same as above — single-test binary.
    unsafe {
        std::env::remove_var("VC_X1_TEST_KEEP");
    }
}

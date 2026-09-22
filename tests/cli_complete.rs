//! CLI subprocess tests for shell completion of path arguments.
//!
//! The binary answers `COMPLETE=<shell> vc-x1 -- vc-x1 <args>` with
//! the candidates for the last argument, which is how the
//! `source <(COMPLETE=bash vc-x1)` hook asks. The fish protocol
//! prints one candidate per line, so it is the one read here. A
//! path argument completes only when clap knows it is one: a
//! `PathBuf`, or a `value_hint` on anything else.

mod common;

use common::{CliFixture, run_ok};

/// The candidates the binary offers for the last of `args`, run in
/// `dir`.
fn complete(fx: &CliFixture, dir: &std::path::Path, args: &[&str]) -> Vec<String> {
    let bin = env!("CARGO_PKG_NAME");
    let out = run_ok(
        fx.cmd()
            .current_dir(dir)
            .env("COMPLETE", "fish")
            .arg("--")
            .arg(bin)
            .args(args),
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.split('\t').next().unwrap_or(l).to_string()) // OK: split always yields a first piece
        .collect()
}

/// Every argument that takes a path completes one, the ones typed as
/// a `String` or parsed by a function of their own included.
#[test]
fn path_arguments_complete() {
    let fx = CliFixture::new("complete-paths");
    let dir = fx.path("tree");
    std::fs::create_dir_all(dir.join("src")).expect("mkdir src");
    std::fs::write(dir.join("notes.md"), "notes\n").expect("write notes");

    let cases: &[(&[&str], &str)] = &[
        (&["init", "sr"], "src/"),
        (&["clone", "sr"], "src/"),
        (&["symlink", "sr"], "src/"),
        (&["lookup", "no"], "notes.md"),
        (&["lookup", "work", "no"], "notes.md"),
        (&["init", "x", "--config", "no"], "notes.md"),
        (&["init", "x", "--use-template", "sr"], "src/"),
        (&["config", "no"], "notes.md"),
        (&["validate-config", "no"], "notes.md"),
        // Already a `PathBuf`, so it completed before the hints.
        (&["validate-anchors", "no"], "notes.md"),
    ];
    for (args, want) in cases {
        let got = complete(&fx, &dir, args);
        assert!(
            got.iter().any(|c| c == want),
            "{args:?} offers {want}: {got:?}"
        );
    }
}

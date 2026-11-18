//! `vc-x1 xtask copy` and `diff` into scratch workspaces of each
//! shape, and the tasks run there after: a single package, a
//! root-package workspace, a virtual one of two binaries, a directory
//! with no Cargo.toml, and an update of a copy already there.
//!
//! Each workspace sits under the test tmp root, outside this one, so
//! cargo sees no parent workspace, and every build shares one target
//! directory, so xtask's dependencies compile once. Ignored by a plain
//! `cargo test`, since it builds xtask, and run by the full
//! validation's `--include-ignored`.

mod common;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::{CliFixture, cargo, run_ok};

/// This workspace, the source every copy takes `xtask/` from.
fn source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Write `files` under `root`, each a path and its text.
fn write(root: &Path, files: &[(&str, &str)]) {
    for (path, text) in files {
        let p = root.join(path);
        std::fs::create_dir_all(p.parent().expect("a parent")).expect("mkdir");
        std::fs::write(p, text).expect("write");
    }
}

/// A binary package's manifest and main, printing its name.
fn bin(name: &str) -> [(String, String); 2] {
    [
        (
            format!("{name}/Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n"),
        ),
        (
            format!("{name}/src/main.rs"),
            format!("fn main() {{ println!(\"{name}\"); }}\n"),
        ),
    ]
}

/// `vc-x1 xtask <args>` from the fixture's isolated home.
fn xtask(fx: &CliFixture, args: &[&str]) -> Command {
    let mut c = fx.cmd();
    c.arg("xtask").args(args);
    c
}

/// `cargo xtask <args>` in `dir`, offline, the builds in `target`.
fn cargo_xtask(dir: &Path, target: &Path, args: &[&str]) -> Output {
    let mut c = cargo();
    c.arg("xtask")
        .args(args)
        .current_dir(dir)
        .env("CARGO_NET_OFFLINE", "true")
        .env("CARGO_TARGET_DIR", target);
    run_ok(&mut c)
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// Copy into each shape, run its tasks, diff it clean, then update it.
#[test]
#[ignore = "builds xtask in scratch workspaces"]
fn xtask_copies_into_every_shape_and_updates() {
    let fx = CliFixture::new("xtask-shapes");
    let target = fx.path("target");
    let src = source();
    let from = src.to_str().expect("a UTF-8 path");

    // A single package becomes a workspace of it and xtask.
    let single = fx.path("single");
    write(
        &single,
        &[
            (
                "Cargo.toml",
                "# the app\n[package]\nname = \"single\"\nversion = \"0.3.0\"\nedition = \"2024\"\n",
            ),
            ("src/main.rs", "fn main() { println!(\"single\"); }\n"),
        ],
    );
    let to = single.to_str().expect("a UTF-8 path");
    let out = run_ok(&mut xtask(&fx, &["copy", "--from", from, "--to", to]));
    assert!(text(&out).contains("new to the target"), "{}", text(&out));
    let manifest = std::fs::read_to_string(single.join("Cargo.toml")).expect("read");
    assert!(manifest.starts_with("# the app\n"), "{manifest}");
    assert!(
        manifest.contains("members = [\".\", \"xtask\"]"),
        "{manifest}"
    );
    assert!(single.join(".cargo/config.toml").is_file());
    assert_eq!(
        text(&cargo_xtask(&single, &target, &["version"])).trim(),
        "0.3.0"
    );
    assert!(text(&cargo_xtask(&single, &target, &["run"])).contains("single"));
    run_ok(&mut xtask(&fx, &["diff", "--from", from, "--to", to]));

    // A root-package workspace gains the member on a line of its own.
    let rooted = fx.path("rooted");
    let mut files = vec![
        (
            "Cargo.toml".to_string(),
            "[package]\nname = \"rooted\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n\
             [workspace]\nmembers = [\n  \".\",\n  \"tool\", # the tool\n]\n"
                .to_string(),
        ),
        (
            "src/main.rs".to_string(),
            "fn main() { println!(\"rooted\"); }\n".to_string(),
        ),
    ];
    files.extend(bin("tool"));
    let files: Vec<(&str, &str)> = files
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect();
    write(&rooted, &files);
    let to = rooted.to_str().expect("a UTF-8 path");
    run_ok(&mut xtask(&fx, &["copy", "--from", from, "--to", to]));
    let manifest = std::fs::read_to_string(rooted.join("Cargo.toml")).expect("read");
    assert!(
        manifest.contains("  \"tool\", # the tool\n  \"xtask\",\n]"),
        "{manifest}"
    );
    assert!(text(&cargo_xtask(&rooted, &target, &["run", "--bin", "tool"])).contains("tool"));

    // A virtual workspace of two binaries, its default declared, and a
    // task of its own.
    let virt = fx.path("virtual");
    let mut files = vec![(
        "Cargo.toml".to_string(),
        "[workspace]\nmembers = [\"app\", \"tool\"]\nresolver = \"3\"\n\n\
         [workspace.package]\nversion = \"2.0.0\"\n\n\
         [workspace.metadata.xtask]\ndefault-bin = \"app\"\n\n\
         [workspace.metadata.xtask.tasks]\nhello = [[\"cargo\", \"run\", \"--quiet\", \"--bin\"]]\n"
            .to_string(),
    )];
    files.extend(bin("app"));
    files.extend(bin("tool"));
    let files: Vec<(&str, &str)> = files
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect();
    write(&virt, &files);
    let to = virt.to_str().expect("a UTF-8 path");
    run_ok(&mut xtask(&fx, &["copy", "--from", from, "--to", to]));
    let manifest = std::fs::read_to_string(virt.join("Cargo.toml")).expect("read");
    assert!(
        manifest.contains("members = [\"app\", \"tool\", \"xtask\"]"),
        "{manifest}"
    );
    assert_eq!(
        text(&cargo_xtask(&virt, &target, &["version"])).trim(),
        "2.0.0"
    );
    assert!(text(&cargo_xtask(&virt, &target, &["run"])).contains("app"));
    assert!(text(&cargo_xtask(&virt, &target, &["hello", "tool"])).contains("tool"));

    // A directory with no Cargo.toml, as `vc-x1 init` leaves one, gets a
    // workspace of xtask alone and `/target` ignored, and runs a task
    // of its own.
    let bare = fx.path("bare");
    write(&bare, &[(".gitignore", "*.log")]);
    let to = bare.to_str().expect("a UTF-8 path");
    let out = run_ok(&mut xtask(&fx, &["copy", "--from", from, "--to", to]));
    assert!(text(&out).contains("xtask alone"), "{}", text(&out));
    let manifest = std::fs::read_to_string(bare.join("Cargo.toml")).expect("read");
    assert!(manifest.contains("members = [\"xtask\"]"), "{manifest}");
    assert_eq!(
        std::fs::read_to_string(bare.join(".gitignore")).expect("read"),
        "*.log\n/target\n"
    );
    assert_eq!(
        text(&cargo_xtask(&bare, &target, &["version"])).trim(),
        "0.1.0"
    );
    std::fs::write(
        bare.join("Cargo.toml"),
        format!(
            "{manifest}\n[workspace.metadata.xtask.tasks]\nhello = [[\"cargo\", \"--version\"]]\n"
        ),
    )
    .expect("write");
    assert!(text(&cargo_xtask(&bare, &target, &["hello"])).starts_with("cargo "));
    run_ok(&mut xtask(&fx, &["diff", "--from", from, "--to", to]));

    // An update: an older copy is replaced, reporting both versions, a
    // local change at the same version is refused, and --force takes
    // it.
    let manifest = single.join("xtask/Cargo.toml");
    let current = std::fs::read_to_string(&manifest).expect("read");
    let version = current
        .lines()
        .find_map(|l| l.strip_prefix("version = \""))
        .and_then(|l| l.strip_suffix('"'))
        .expect("xtask's version")
        .to_string();
    std::fs::write(
        &manifest,
        current.replacen(
            &format!("version = \"{version}\""),
            "version = \"0.0.1\"",
            1,
        ),
    )
    .expect("write");
    std::fs::write(single.join("xtask/src/stale.rs"), "// gone\n").expect("write");
    let to = single.to_str().expect("a UTF-8 path");
    let out = run_ok(&mut xtask(&fx, &["copy", "--from", from, "--to", to]));
    assert!(
        text(&out).contains(&format!("xtask 0.0.1 -> {version}")),
        "{}",
        text(&out)
    );
    assert!(
        !single.join("xtask/src/stale.rs").exists(),
        "deletions included"
    );
    std::fs::write(single.join("xtask/src/local.rs"), "// mine\n").expect("write");
    let out = xtask(&fx, &["copy", "--from", from, "--to", to])
        .output()
        .expect("spawn vc-x1");
    assert!(!out.status.success());
    assert!(text(&out).contains("a local change"), "{}", text(&out));
    run_ok(&mut xtask(
        &fx,
        &["copy", "--from", from, "--to", to, "--force"],
    ));
    assert!(!single.join("xtask/src/local.rs").exists());
    run_ok(&mut xtask(&fx, &["diff", "--from", from, "--to", to]));
}

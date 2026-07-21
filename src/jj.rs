//! Typed facade over `jj` subprocess queries.
//!
//! Every read-only `jj log -T <template>` spawn goes through this
//! module — one primitive plus typed helpers — so call sites stop
//! hand-rolling argument lists (the DRY-facade stage of
//! `notes/refactor-20260716.md`):
//!
//! - `log` — the raw primitive; the helpers below cover the
//!   common templates.
//! - `matches` / `rev_exists` — does a revset match / does a
//!   revision resolve.
//! - `chid_of` / `cid_of` / `cid_short_of` — change / commit ids.
//! - `desc_of` / `is_empty` — description and emptiness.
//!
//! Mutations still spawn `jj` at their call sites; the jj-lib
//! migration stage moves both in-process.

use std::path::Path;

use crate::common::run;

/// Crate-standard boxed-error result, aliased locally for brevity.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Run `jj log -r <rev> --no-graph -T <template> -R <repo>` and
/// return its stdout (`run` trims surrounding whitespace).
pub fn log(repo: &Path, rev: &str, template: &str) -> Result<String> {
    run(
        "jj",
        &[
            "log",
            "-r",
            rev,
            "--no-graph",
            "-T",
            template,
            "-R",
            &repo.to_string_lossy(),
        ],
        Path::new("."),
    )
}

/// True when `revset` matches at least one commit in `repo`.
///
/// A valid-but-empty revset (e.g. `conflicts()` on a clean repo)
/// is `Ok(false)`; an unresolvable revision is an `Err` — use
/// `rev_exists` to fold that case to `false`.
pub fn matches(repo: &Path, revset: &str) -> Result<bool> {
    Ok(!log(repo, revset, "\"x\"")?.is_empty())
}

/// True when `rev` resolves in `repo`.
///
/// Maps jj's unresolvable-revision errors (`doesn't exist`,
/// `No such revision`) to `Ok(false)`; other failures (bad repo
/// path, spawn error) stay `Err`.
pub fn rev_exists(repo: &Path, rev: &str) -> Result<bool> {
    match matches(repo, rev) {
        Ok(found) => Ok(found),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("doesn't exist") || msg.contains("No such revision") {
                Ok(false)
            } else {
                Err(e)
            }
        }
    }
}

/// The 12-character change id of `rev`.
pub fn chid_of(repo: &Path, rev: &str) -> Result<String> {
    log(repo, rev, "change_id.short(12)")
}

/// The full commit id of `rev`.
pub fn cid_of(repo: &Path, rev: &str) -> Result<String> {
    log(repo, rev, "commit_id")
}

/// The 12-character commit id of `rev`.
pub fn cid_short_of(repo: &Path, rev: &str) -> Result<String> {
    log(repo, rev, "commit_id.short(12)")
}

/// The full description (title + body) of `rev`.
pub fn desc_of(repo: &Path, rev: &str) -> Result<String> {
    log(repo, rev, "description")
}

/// True when `rev` is empty (no file changes relative to its
/// parent). Strict: anything but `true` / `false` from the
/// template is an error.
pub fn is_empty(repo: &Path, rev: &str) -> Result<bool> {
    let out = log(repo, rev, "empty")?;
    match out.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(format!("jj::is_empty: unexpected template output {other:?}").into()),
    }
}

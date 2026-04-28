//! Parsing and derivation helpers for repository URLs and targets.
//!
//! Single source of truth for the positional shapes that `init` and
//! `clone` accept (URL, owner/name shorthand, path) and for the
//! URL-derivation helpers shared between them.
//!
//! Lifted from `clone.rs` / `init.rs` in 0.41.1-1; consumers
//! migrate to `parse_target` in 0.41.1-2 (clone) and 0.41.1-3
//! (init).

use std::path::PathBuf;

/// A parsed positional `<TARGET>` argument to `init` or `clone`.
///
/// - `Url` — full git URL (`scheme://...` or SSH `user@host:path`).
/// - `OwnerName(owner, name)` — `owner/name` shorthand;
///   resolves to `git@github.com:owner/name.git`.
/// - `Path` — local path with explicit prefix
///   (`./`, `../`, `/`, `~/`, or bare `~`). Path text is preserved
///   literally; tilde expansion is the consumer's responsibility.
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)] // OK: staged for 0.41.1-2 (clone) / 0.41.1-3 (init) consumers
pub enum Target {
    Url(String),
    OwnerName(String, String),
    Path(PathBuf),
}

/// Parse a positional `<TARGET>` argument into one of the three
/// shapes.
///
/// Detection order (path forms first, then URL, then shorthand):
///
/// - Path forms: bare `.`, `..`, or `~`; or starts with `./`,
///   `../`, `/`, or `~/`. `.` and `..` are POSIX cwd/parent and
///   are unambiguous; the consumer resolves them to a real
///   workspace name via `canonicalize` + `file_name`.
/// - URL: contains `://`, or SSH-style `user@host:path` (an `@`
///   followed somewhere by `:`).
/// - `owner/name` shorthand: exactly one `/`, both sides non-empty,
///   no path or URL indicators.
///
/// Errors on bare alphanumeric names (genuinely ambiguous —
/// "missing `./`?" / "missing `/name` suffix?") and empty input.
#[allow(dead_code)] // OK: staged for 0.41.1-2 (clone) / 0.41.1-3 (init) consumers
pub fn parse_target(s: &str) -> Result<Target, String> {
    if s.is_empty() {
        return Err("empty target".into());
    }

    if s == "."
        || s == ".."
        || s == "~"
        || s.starts_with("./")
        || s.starts_with("../")
        || s.starts_with('/')
        || s.starts_with("~/")
    {
        return Ok(Target::Path(PathBuf::from(s)));
    }

    if s.contains("://") {
        return Ok(Target::Url(s.to_string()));
    }
    if let Some(at) = s.find('@')
        && s[at + 1..].contains(':')
    {
        return Ok(Target::Url(s.to_string()));
    }

    if s.matches('/').count() == 1
        && let Some((owner, name)) = s.split_once('/')
        && !owner.is_empty()
        && !name.is_empty()
    {
        return Ok(Target::OwnerName(owner.to_string(), name.to_string()));
    }

    Err(format!(
        "'{s}' is not a recognized target — expected URL, owner/name shorthand, or path prefix (./X, ../X, /X, ~/X, ~)"
    ))
}

/// Derive the project name from a repo URL or `owner/name` shorthand.
///
/// - Strips trailing `.git`.
/// - Returns the last segment after the rightmost `/` or `:`.
/// - Errors when the resulting name would be empty.
pub fn derive_name(repo: &str) -> Result<String, Box<dyn std::error::Error>> {
    let stem = repo.strip_suffix(".git").unwrap_or(repo); // OK: .git suffix is optional
    let last = stem.rsplit(['/', ':']).next().unwrap_or(""); // OK: rsplit always yields at least one element
    if last.is_empty() {
        return Err(format!("cannot derive project name from '{repo}'").into());
    }
    Ok(last.to_string())
}

/// Resolve a repo argument to a git clone URL.
///
/// - `owner/name` (single `/`, no `:` or scheme) →
///   `git@github.com:owner/name.git`.
/// - Anything else is passed through as-is (already a URL).
pub fn resolve_url(repo: &str) -> String {
    if repo.contains("://") || repo.contains('@') {
        return repo.to_string();
    }
    if repo.matches('/').count() == 1 && !repo.contains(':') {
        return format!("git@github.com:{repo}.git");
    }
    repo.to_string()
}

/// Derive the session repo URL from a code repo URL.
///
/// - With trailing `.git`: insert `.claude` before it
///   (`foo.git` → `foo.claude.git`).
/// - Without `.git`: append `.claude` (`foo` → `foo.claude`).
pub fn derive_session_url(code_url: &str) -> String {
    match code_url.strip_suffix(".git") {
        Some(stem) => format!("{stem}.claude.git"),
        None => format!("{code_url}.claude"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- derive_name -------------------------------------------------

    #[test]
    fn derive_name_owner_slash_name() {
        assert_eq!(derive_name("owner/my-project").unwrap(), "my-project");
    }

    #[test]
    fn derive_name_ssh_url() {
        assert_eq!(
            derive_name("git@github.com:owner/my-project.git").unwrap(),
            "my-project"
        );
    }

    #[test]
    fn derive_name_https_url() {
        assert_eq!(
            derive_name("https://github.com/owner/my-project.git").unwrap(),
            "my-project"
        );
    }

    #[test]
    fn derive_name_https_no_suffix() {
        assert_eq!(
            derive_name("https://github.com/owner/my-project").unwrap(),
            "my-project"
        );
    }

    #[test]
    fn derive_name_bare_name() {
        assert_eq!(derive_name("my-project").unwrap(), "my-project");
    }

    #[test]
    fn derive_name_local_bare_path_with_git() {
        assert_eq!(derive_name("/tmp/foo.git").unwrap(), "foo");
    }

    #[test]
    fn derive_name_local_bare_path_without_git() {
        assert_eq!(derive_name("/tmp/foo").unwrap(), "foo");
    }

    // --- resolve_url -------------------------------------------------

    #[test]
    fn resolve_url_shorthand() {
        assert_eq!(resolve_url("owner/repo"), "git@github.com:owner/repo.git");
    }

    #[test]
    fn resolve_url_ssh_passthrough() {
        let url = "git@github.com:owner/repo.git";
        assert_eq!(resolve_url(url), url);
    }

    #[test]
    fn resolve_url_https_passthrough() {
        let url = "https://github.com/owner/repo.git";
        assert_eq!(resolve_url(url), url);
    }

    // --- derive_session_url ------------------------------------------

    #[test]
    fn session_url_ssh() {
        assert_eq!(
            derive_session_url("git@github.com:owner/repo.git"),
            "git@github.com:owner/repo.claude.git"
        );
    }

    #[test]
    fn session_url_https_with_git() {
        assert_eq!(
            derive_session_url("https://github.com/owner/repo.git"),
            "https://github.com/owner/repo.claude.git"
        );
    }

    #[test]
    fn session_url_https_no_suffix() {
        assert_eq!(
            derive_session_url("https://github.com/owner/repo"),
            "https://github.com/owner/repo.claude"
        );
    }

    #[test]
    fn session_url_local_bare_with_git() {
        assert_eq!(derive_session_url("/tmp/foo.git"), "/tmp/foo.claude.git");
    }

    #[test]
    fn session_url_local_bare_without_git() {
        assert_eq!(derive_session_url("/tmp/foo"), "/tmp/foo.claude");
    }

    // --- parse_target: URL forms -------------------------------------

    #[test]
    fn parse_target_https_url() {
        assert_eq!(
            parse_target("https://github.com/owner/repo.git").unwrap(),
            Target::Url("https://github.com/owner/repo.git".into()),
        );
    }

    #[test]
    fn parse_target_ssh_url() {
        assert_eq!(
            parse_target("git@github.com:owner/repo.git").unwrap(),
            Target::Url("git@github.com:owner/repo.git".into()),
        );
    }

    // --- parse_target: owner/name shorthand --------------------------

    #[test]
    fn parse_target_owner_name() {
        assert_eq!(
            parse_target("owner/repo").unwrap(),
            Target::OwnerName("owner".into(), "repo".into()),
        );
    }

    // --- parse_target: path forms ------------------------------------

    #[test]
    fn parse_target_dot_slash_path() {
        assert_eq!(
            parse_target("./foo").unwrap(),
            Target::Path(PathBuf::from("./foo")),
        );
    }

    #[test]
    fn parse_target_dot_dot_slash_path() {
        assert_eq!(
            parse_target("../foo/bar").unwrap(),
            Target::Path(PathBuf::from("../foo/bar")),
        );
    }

    #[test]
    fn parse_target_absolute_path() {
        assert_eq!(
            parse_target("/tmp/foo").unwrap(),
            Target::Path(PathBuf::from("/tmp/foo")),
        );
    }

    #[test]
    fn parse_target_tilde_alone() {
        assert_eq!(parse_target("~").unwrap(), Target::Path(PathBuf::from("~")),);
    }

    #[test]
    fn parse_target_tilde_path() {
        assert_eq!(
            parse_target("~/work/foo").unwrap(),
            Target::Path(PathBuf::from("~/work/foo")),
        );
    }

    // --- parse_target: errors ----------------------------------------

    #[test]
    fn parse_target_empty_errors() {
        let err = parse_target("").unwrap_err();
        assert!(err.contains("empty"), "got: {err}");
    }

    #[test]
    fn parse_target_dot_is_cwd_path() {
        assert_eq!(parse_target(".").unwrap(), Target::Path(PathBuf::from(".")),);
    }

    #[test]
    fn parse_target_dot_dot_is_parent_path() {
        assert_eq!(
            parse_target("..").unwrap(),
            Target::Path(PathBuf::from("..")),
        );
    }

    #[test]
    fn parse_target_bare_name_errors() {
        let err = parse_target("my-project").unwrap_err();
        assert!(err.contains("not a recognized target"), "got: {err}");
    }
}

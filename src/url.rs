//! Parsing and derivation helpers for URLs and target strings.
//!
//! Single source of truth for the positional `<TARGET>` forms
//! that `init` and `clone` accept (URL, owner/name shorthand,
//! path) and for the URL-derivation helpers shared between them.
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
/// - `BareName` — a bare alphanumeric (no `/`, `:`, or path
///   prefix). Init resolves it via the user-config remote chain;
///   clone errors on it (no config-driven default).
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Target {
    Url(String),
    OwnerName(String, String),
    Path(PathBuf),
    BareName(String),
}

/// Parse a positional `<TARGET>` argument into one of the four
/// `Target` variants.
///
/// Detection order (path forms first, then URL, then shorthand,
/// then bare NAME):
///
/// - Path forms: bare `.`, `..`, or `~`; or starts with `./`,
///   `../`, `/`, or `~/`. `.` and `..` are POSIX cwd/parent and
///   are unambiguous; the consumer resolves them to a real
///   directory name via `canonicalize` + `file_name`.
/// - URL: contains `://`, or SSH-style `user@host:path` (an `@`
///   followed somewhere by `:`).
/// - `owner/name` shorthand: exactly one `/`, both sides non-empty,
///   no path or URL indicators.
/// - Bare NAME: no `/`, no `:`, no URL/path indicators. Init
///   resolves it via the user-config remote chain (repo created
///   at `cwd/<NAME>`); clone errors on it (no config).
///
/// Errors only on empty input or syntactic garbage that fits
/// none of the above (e.g. `owner/name/extra`).
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
        && !owner.contains(':')
        && !name.contains(':')
    {
        return Ok(Target::OwnerName(owner.to_string(), name.to_string()));
    }

    // Bare NAME: no slash, no colon, no URL pattern. Init expands
    // via config; clone rejects.
    if !s.contains('/') && !s.contains(':') {
        return Ok(Target::BareName(s.to_string()));
    }

    // Catch-all. If it looks like an SSH scp-like form missing the
    // `git@` prefix (host:owner/name), suggest the canonical form
    // — easy mistake to make and the resulting "did you mean…?"
    // is concrete enough to fix with one re-type.
    if let Some(colon) = s.find(':')
        && colon > 0
        && !s[..colon].contains('/')
        && s[colon + 1..].contains('/')
        && !s.contains('@')
    {
        return Err(format!(
            "'{s}' is not a recognized target — looks like an SSH URL missing the 'git@' prefix; did you mean 'git@{s}'?"
        ));
    }

    Err(format!(
        "'{s}' is not a recognized target — expected URL, owner/name shorthand, path prefix (./X, ../X, /X, ~/X, ~), or bare NAME"
    ))
}

/// Derive the project name from a URL or `owner/name` shorthand.
///
/// - Strips trailing `.git`.
/// - Returns the last segment after the rightmost `/` or `:`.
/// - Errors when the resulting name would be empty.
pub fn derive_name(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let stem = url.strip_suffix(".git").unwrap_or(url); // OK: .git suffix is optional
    let last = stem.rsplit(['/', ':']).next().unwrap_or(""); // OK: rsplit always yields at least one element
    if last.is_empty() {
        return Err(format!("cannot derive project name from '{url}'").into());
    }
    Ok(last.to_string())
}

/// Resolve a target string to a git clone URL.
///
/// - `owner/name` (single `/`, no `:` or scheme) →
///   `git@github.com:owner/name.git`.
/// - Anything else is passed through as-is (already a URL).
pub fn resolve_url(url: &str) -> String {
    if url.contains("://") || url.contains('@') {
        return url.to_string();
    }
    if url.matches('/').count() == 1 && !url.contains(':') {
        return format!("git@github.com:{url}.git");
    }
    url.to_string()
}

/// Derive the session URL from a code-side URL.
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
    fn parse_target_bare_name() {
        assert_eq!(
            parse_target("my-project").unwrap(),
            Target::BareName("my-project".into()),
        );
    }

    #[test]
    fn parse_target_bare_name_with_dots() {
        // Names with dots (e.g. "v2.0") are still bare names.
        assert_eq!(
            parse_target("v2.0").unwrap(),
            Target::BareName("v2.0".into()),
        );
    }

    #[test]
    fn parse_target_too_many_slashes_errors() {
        let err = parse_target("owner/name/extra").unwrap_err();
        assert!(err.contains("not a recognized target"), "got: {err}");
    }

    #[test]
    fn parse_target_host_colon_path_without_at_suggests_ssh_form() {
        // `github.com:winksaville/tf1` looks like an SSH URL missing
        // the `git@` prefix. Without rejection it would have been
        // mis-parsed as OwnerName("github.com:winksaville", "tf1")
        // and the dispatcher would build a doubled-up URL
        // `git@github.com:github.com:winksaville/tf1.git`.
        let err = parse_target("github.com:winksaville/tf1").unwrap_err();
        assert!(err.contains("missing the 'git@' prefix"), "got: {err}");
        assert!(err.contains("git@github.com:winksaville/tf1"), "got: {err}");
    }

    #[test]
    fn parse_target_owner_with_colon_rejected() {
        // Standalone reproducer for the same family: any `:` in the
        // owner half of `owner/name` shorthand is suspicious.
        let err = parse_target("a:b/c").unwrap_err();
        assert!(err.contains("missing the 'git@' prefix"), "got: {err}");
    }
}

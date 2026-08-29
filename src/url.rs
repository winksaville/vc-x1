//! Parsing and derivation helpers for URLs and target strings.
//!
//! Single source of truth for the positional `<TARGET>` forms
//! that `init` and `clone` accept (URL, path, bare NAME) and for
//! the URL-derivation helpers shared between them.
//!
//! Lifted from `clone.rs` / `init.rs` in 0.41.1-1; consumers
//! migrate to `parse_target` in 0.41.1-2 (clone) and 0.41.1-3
//! (init).

use std::path::PathBuf;

/// A parsed positional `<TARGET>` argument to `init` or `clone`.
///
/// - `Url`: full git URL (`scheme://...` or SSH `user@host:path`).
/// - `Path`: local path with explicit prefix
///   (`./`, `../`, `/`, `~/`, or bare `~`). Path text is preserved
///   literally, and tilde expansion is the consumer's
///   responsibility.
/// - `BareName`: a bare alphanumeric (no `/`, `:`, or path
///   prefix). Init resolves it via the user-config remote chain,
///   and clone errors on it (no config-driven default).
///
/// The retired fourth form is the `owner/name` shorthand, which
/// [`parse_target`] now rejects. See there for why.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Target {
    Url(String),
    Path(PathBuf),
    BareName(String),
}

/// Parse a positional `<TARGET>` argument into one of the three
/// `Target` variants.
///
/// Detection order (path forms first, then URL, then bare NAME):
///
/// - Path forms: bare `.`, `..`, or `~`; or starts with `./`,
///   `../`, `/`, or `~/`. `.` and `..` are POSIX cwd/parent and
///   are unambiguous; the consumer resolves them to a real
///   directory name via `canonicalize` + `file_name`.
/// - URL: contains `://`, or SSH-style `user@host:path` (an `@`
///   followed somewhere by `:`).
/// - Bare NAME: no `/`, no `:`, no URL/path indicators. Init
///   resolves it via the user-config remote chain (repo created
///   at `cwd/<NAME>`), and clone errors on it (no config).
///
/// A slashed target with no path prefix (`owner/name`, `tmp/foo`)
/// is refused, naming both readings. It used to be the `owner/name`
/// shorthand, and the reading is undecidable: nothing needs to
/// exist for a path target, since init creates missing parents, so
/// `tmp/foo` is a well-formed path and a well-formed `owner/name`
/// at the same time. The old rule broke the tie toward the
/// shorthand, which silently turned a `tmp/foo` path into a request
/// to create a repo in an organization named `tmp` (2026-08-28,
/// bugs.md). Refusing is the transitional step: once nobody reaches
/// for the shorthand, a slashed target can simply mean the path.
///
/// Errors otherwise only on empty input or syntactic garbage that
/// fits none of the above.
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

    // A slash with no path prefix: the retired shorthand's shape,
    // and also a perfectly ordinary relative path. Neither reading
    // wins on syntax, so neither is guessed.
    if s.contains('/') && !s.contains(':') {
        return Err(format!(
            "'{s}' is ambiguous: it could name the local path './{s}' or the GitHub repo \
             '{s}', and the owner/name shorthand is retired. Pass './{s}' for the path, or \
             the repo's URL"
        ));
    }

    // Bare NAME: no slash, no colon, no URL pattern. Init expands
    // via config; clone rejects.
    if !s.contains('/') && !s.contains(':') {
        return Ok(Target::BareName(s.to_string()));
    }

    // Catch-all. If it looks like an SSH scp-like form missing the
    // `git@` prefix (host:owner/name), suggest the canonical form,
    // easy mistake to make and the resulting "did you mean...?"
    // is concrete enough to fix with one re-type.
    if let Some(colon) = s.find(':')
        && colon > 0
        && !s[..colon].contains('/')
        && s[colon + 1..].contains('/')
        && !s.contains('@')
    {
        return Err(format!(
            "'{s}' is not a recognized target: looks like an SSH URL missing the 'git@' \
             prefix; did you mean 'git@{s}'?"
        ));
    }

    Err(format!(
        "'{s}' is not a recognized target: expected URL, owner/name shorthand, path prefix \
         (./X, ../X, /X, ~/X, ~), or bare NAME"
    ))
}

/// Derive the project name from a URL or `owner/name` shorthand.
///
/// - Strips trailing `.git`.
/// - Returns the last segment after the rightmost `/` or `:`.
/// - Errors when the resulting name would be empty.
pub fn derive_name(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let stem = url.strip_suffix(".git").unwrap_or(url); // OK: .git suffix is optional
    // OK: rsplit always yields at least one element
    let last = stem.rsplit(['/', ':']).next().unwrap_or("");
    if last.is_empty() {
        return Err(format!("cannot derive project name from '{url}'").into());
    }
    Ok(last.to_string())
}

/// Derive the bot-repo URL from a work-side URL.
///
/// - With trailing `.git`: insert `.claude` before it
///   (`foo.git` -> `foo.claude.git`).
/// - Without `.git`: append `.claude` (`foo` -> `foo.claude`).
pub fn derive_bot_url(work_url: &str) -> String {
    match work_url.strip_suffix(".git") {
        Some(stem) => format!("{stem}.claude.git"),
        None => format!("{work_url}.claude"),
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

    // --- derive_bot_url ------------------------------------------

    #[test]
    fn bot_url_ssh() {
        assert_eq!(
            derive_bot_url("git@github.com:owner/repo.git"),
            "git@github.com:owner/repo.claude.git"
        );
    }

    #[test]
    fn bot_url_https_with_git() {
        assert_eq!(
            derive_bot_url("https://github.com/owner/repo.git"),
            "https://github.com/owner/repo.claude.git"
        );
    }

    #[test]
    fn bot_url_https_no_suffix() {
        assert_eq!(
            derive_bot_url("https://github.com/owner/repo"),
            "https://github.com/owner/repo.claude"
        );
    }

    #[test]
    fn bot_url_local_bare_with_git() {
        assert_eq!(derive_bot_url("/tmp/foo.git"), "/tmp/foo.claude.git");
    }

    #[test]
    fn bot_url_local_bare_without_git() {
        assert_eq!(derive_bot_url("/tmp/foo"), "/tmp/foo.claude");
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

    // --- parse_target: the retired owner/name shorthand --------------

    /// A slashed target with no path prefix is refused, and the
    /// message carries both readings so the caller can pick one.
    #[test]
    fn parse_target_slashed_is_refused() {
        let err = parse_target("owner/repo").unwrap_err();
        assert!(err.contains("ambiguous"), "{err}");
        assert!(err.contains("./owner/repo"), "{err}");
        assert!(err.contains("URL"), "{err}");
    }

    /// The case that found the bug: a plausible relative path used
    /// to become a request to create a repo in an org named `tmp`.
    #[test]
    fn parse_target_relative_path_without_prefix_is_refused() {
        let err = parse_target("tmp/foo").unwrap_err();
        assert!(err.contains("./tmp/foo"), "{err}");
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

    /// A multi-slash target is refused by the same rule as a
    /// single-slash one: it is a path or it is nothing, and saying
    /// so needs the `./` prefix.
    #[test]
    fn parse_target_too_many_slashes_errors() {
        let err = parse_target("owner/name/extra").unwrap_err();
        assert!(err.contains("./owner/name/extra"), "got: {err}");
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

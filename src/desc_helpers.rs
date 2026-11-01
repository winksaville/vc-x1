use std::sync::Arc;

use jj_lib::object_id::ObjectId;
use jj_lib::repo::{ReadonlyRepo, Repo};
use jj_lib::workspace::Workspace;

use crate::common;

pub const DEFAULT_ID_LEN: usize = 12;
pub const VC_CONFIG_FILE: &str = ".vc-config.toml";

/// Canonical ochid label for the work side.
pub const OCHID_WORK_LABEL: &str = "/";

/// Canonical ochid label for the bot side.
///
/// An opaque label resolved through the `[repos]` registry, not
/// a filesystem path: a workspace whose bot dir is named
/// something else still reads and writes `/.claude`. Kept as the
/// historical spelling so every published trailer stays valid,
/// and intended to grow into URL labels (see
/// forks-multi-user.md#per-user-bot-repos-via-url-shaped-ochid).
pub const OCHID_BOT_LABEL: &str = "/.claude";

/// A repo's ochid prefix: its side's canonical label.
///
/// The prefix comes from *which side* the repo is (by
/// self-resolution: see `common::is_bot_dir`), never from the
/// directory's spelling:
///
/// - work side -> `/`
/// - bot side -> `/.claude/` (the label + separator)
pub fn ochid_prefix_for(repo: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
    if common::is_bot_dir(repo) {
        Ok(format!("{OCHID_BOT_LABEL}/"))
    } else {
        Ok(OCHID_WORK_LABEL.to_string())
    }
}

/// A `--body` value: the text as given, or the file an `@PATH`
/// names, since a body in commit-body form has markers and blank
/// lines a shell quote mangles. `who` prefixes the error.
pub fn read_body(body: &str, who: &str) -> Result<String, String> {
    match body.strip_prefix('@') {
        Some(path) if !path.is_empty() => std::fs::read_to_string(path)
            .map_err(|e| format!("{who}: --body @{path}: cannot read the file: {e}")),
        _ => Ok(body.to_string()),
    }
}

/// The trailers `vc-x1` stamps. `ochid:` is the partner's change id in
/// the other repo, one per commit an agent commit covers. A work commit
/// also links by URL: `ocid:` the agent commit, and `pcid:` each parent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Trailers {
    pub ochids: Vec<String>,
    pub ocid: Option<String>,
    pub pcids: Vec<String>,
}

impl Trailers {
    /// The trailers `desc` carries, column 0 only, as `extract_ochids`
    /// reads them.
    pub fn of(desc: &str) -> Self {
        let values = |key: &str| -> Vec<String> {
            desc.lines()
                .filter_map(|line| line.strip_prefix(key))
                .map(str::trim)
                .filter(|value| is_trailer_value(value))
                .map(str::to_string)
                .collect()
        };
        Self {
            ochids: values("ochid:"),
            ocid: values("ocid:").pop(),
            pcids: values("pcid:"),
        }
    }

    /// The trailers as lines, in the order they are stamped.
    fn lines(&self) -> Vec<String> {
        let ochids = self.ochids.iter().map(|v| format!("ochid: {v}"));
        let ocid = self.ocid.iter().map(|v| format!("ocid: {v}"));
        let pcids = self.pcids.iter().map(|v| format!("pcid: {v}"));
        ochids.chain(ocid).chain(pcids).collect()
    }
}

/// Whether a value is one a stamped trailer holds: one word, so a body
/// sentence that happens to start `ochid: ` at column 0 is not a trailer.
fn is_trailer_value(value: &str) -> bool {
    !value.is_empty() && !value.contains(char::is_whitespace)
}

/// Whether a line reads as a git trailer `vc-x1` does not stamp, one
/// `custom.md` may add: a key of lowercase or hyphenated words, so a
/// paragraph reading `Solution: ...` stays body.
fn is_other_trailer(line: &str) -> bool {
    line.split_once(": ").is_some_and(|(key, value)| {
        !value.is_empty()
            && !key.is_empty()
            && !["ochid", "ocid", "pcid"].contains(&key)
            && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            && (key.contains('-') || !key.chars().any(|c| c.is_ascii_uppercase()))
    })
}

/// `text` with `trailers` stamped in place of any it carries. The result
/// ends in one trailer block, the stamped lines and then any other
/// trailers the text ended in, so a rewrite keeps a custom trailer and
/// never leaves it stranded in the body.
pub fn with_trailers(text: &str, trailers: &Trailers) -> String {
    let stripped = without_trailer_lines(text).collect::<Vec<_>>().join("\n");
    let stripped = stripped.trim_end();
    let (body, others) = match stripped.rsplit_once("\n\n") {
        Some((body, last)) if last.lines().all(is_other_trailer) => (body.trim_end(), Some(last)),
        _ => (stripped, None),
    };
    let mut block = trailers.lines();
    block.extend(others.into_iter().flat_map(str::lines).map(str::to_string));
    if block.is_empty() {
        return body.to_string();
    }
    format!("{body}\n\n{}", block.join("\n"))
}

/// `text` with one `ochid:` trailer, an agent commit's for one work commit.
pub fn with_trailer(text: &str, chid: &str) -> String {
    with_ochids(text, &[chid.to_string()])
}

/// `text` with one `ochid:` per change id, in order, the agent commit a
/// Land makes for a whole cycle.
pub fn with_ochids(text: &str, chids: &[String]) -> String {
    with_trailers(
        text,
        &Trailers {
            ochids: chids.to_vec(),
            ..Trailers::default()
        },
    )
}

/// What an `ocid:` trailer's SHA is until the agent commit is made.
pub const OCID_PLACEHOLDER: &str = "xxxx";

/// The link to commit `sha` in the repo at canonical `url`.
pub fn commit_url(url: &str, sha: &str) -> String {
    format!("{url}/commit/{sha}")
}

/// The URL `repo`'s commits are linked by, from its `origin`: canonical,
/// or `file:///` for a local remote, `None` with no `origin`.
pub fn repo_url_for(repo: &std::path::Path) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let url = crate::jj::remote_url(repo, "origin")?;
    Ok(url
        .as_deref()
        .and_then(|url| crate::url::link_repo_url(url, repo)))
}

/// The placeholder `ocid:` value for a commit in `repo`, `None` when
/// it has no `origin`, since the value is a link.
pub fn ocid_placeholder_for(
    repo: &std::path::Path,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    Ok(repo_url_for(repo)?.map(|url| commit_url(&url, OCID_PLACEHOLDER)))
}

/// The `pcid:` values for `parents`, commit ids in `repo`, none with no
/// `origin`, and the root commit, which is not a real one, left out.
pub fn pcids_for(
    repo: &std::path::Path,
    parents: &[String],
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let Some(url) = repo_url_for(repo)? else {
        return Ok(Vec::new());
    };
    let root = crate::jj::cid_of(repo, "root()")?;
    Ok(parents
        .iter()
        .filter(|parent| **parent != root)
        .map(|parent| commit_url(&url, parent))
        .collect())
}

/// A description without its `ochid:`, `ocid:`, and `pcid:` lines.
pub fn without_trailer_lines(desc: &str) -> impl Iterator<Item = &str> {
    desc.lines().filter(|line| {
        !["ochid:", "ocid:", "pcid:"].iter().any(|key| {
            line.strip_prefix(key)
                .is_some_and(|value| is_trailer_value(value.trim()))
        })
    })
}

/// Problems found with an ochid trailer, with details for reporting.
#[derive(Debug)]
pub struct OchidIssues {
    /// None if prefix is correct, Some((actual_prefix, expected_prefix)) if wrong.
    pub wrong_prefix: Option<(String, String)>,
    /// None if length is correct, Some((actual_len, expected_len)) if wrong.
    pub wrong_length: Option<(usize, usize)>,
    /// true if the bare ID does not resolve in the other repo.
    pub not_found: bool,
    /// The bare ID extracted from the ochid value.
    pub bare_id: String,
}

impl OchidIssues {
    pub fn any(&self) -> bool {
        self.wrong_prefix.is_some() || self.wrong_length.is_some() || self.not_found
    }

    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if let Some((ref actual, ref expected)) = self.wrong_prefix {
            parts.push(format!("prefix: {actual} (want {expected})"));
        }
        if let Some((actual, expected)) = self.wrong_length {
            parts.push(format!("len: {actual} (want {expected})"));
        }
        if self.not_found {
            parts.push(format!("not found: {}", self.bare_id));
        }
        parts.join(", ")
    }
}

/// Whether a prefix, what precedes the bare ID, is a URL form's:
/// `<scheme>://<repo>#`.
fn is_url_prefix(prefix: &str) -> bool {
    prefix.contains("://") && prefix.ends_with('#')
}

/// The forms a trailer naming one repo may take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OchidForms {
    /// The side's label, what every published trailer carries.
    pub label: String,
    /// The path form by the directory's own name, for a repo with no
    /// URL to stamp: `/` on the work side, `/<dir>/` on the agent's.
    pub path: String,
    /// The repo's canonical URL, from its `origin`, when it has one.
    pub url: Option<String>,
}

impl OchidForms {
    /// Whether a trailer's prefix is one of these forms. A URL naming
    /// another repo is not, compared canonically, and with no `url` to
    /// compare against any URL passes on its ID alone.
    fn accepts(&self, actual: &str) -> bool {
        if is_url_prefix(actual) {
            let named = crate::url::canonical_repo_url(actual.trim_end_matches('#'));
            self.url.is_none() || named == self.url
        } else {
            actual.is_empty() || actual == self.label || actual == self.path
        }
    }

    /// The form a wrong prefix is reported against.
    fn wanted(&self, actual: &str) -> String {
        match &self.url {
            Some(url) if is_url_prefix(actual) => format!("{url}#"),
            _ => self.label.clone(),
        }
    }
}

/// The forms a trailer naming `repo` may take, the URL read from its
/// `origin`.
pub fn ochid_forms_for(repo: &std::path::Path) -> Result<OchidForms, Box<dyn std::error::Error>> {
    let label = ochid_prefix_for(repo)?;
    let path = if label == OCHID_WORK_LABEL {
        label.clone()
    } else {
        let canonical = repo.canonicalize()?;
        match canonical.file_name().and_then(|n| n.to_str()) {
            Some(name) => format!("/{name}/"),
            None => label.clone(),
        }
    };
    let url =
        crate::jj::remote_url(repo, "origin")?.and_then(|u| crate::url::canonical_repo_url(&u));
    Ok(OchidForms { label, path, url })
}

/// Whether an ochid value is the URL form, naming a repo by URL.
pub fn is_url_form(ochid_value: &str) -> bool {
    let bare_id = extract_bare_id(ochid_value);
    is_url_prefix(&ochid_value[..ochid_value.len() - bare_id.len()])
}

/// Validate an ochid trailer value against the repo's forms, length, and other repo.
pub fn validate_ochid(
    value: &str,
    forms: &OchidForms,
    id_len: usize,
    other_workspace: &Workspace,
    other_repo: &Arc<ReadonlyRepo>,
) -> OchidIssues {
    // Extract the actual prefix (everything before the bare ID)
    let bare_id = extract_bare_id(value);
    let actual_prefix = &value[..value.len() - bare_id.len()];

    let wrong_prefix = if !forms.accepts(actual_prefix) {
        Some((actual_prefix.to_string(), forms.wanted(actual_prefix)))
    } else {
        None
    };

    let wrong_length = if bare_id.len() != id_len {
        Some((bare_id.len(), id_len))
    } else {
        None
    };

    // Check if the ID resolves in the other repo
    let not_found = common::resolve_revset(other_workspace, other_repo, bare_id)
        .map(|ids| ids.is_empty())
        .unwrap_or(true); // OK: resolve failure -> treat as not found

    OchidIssues {
        wrong_prefix,
        wrong_length,
        not_found,
        bare_id: bare_id.to_string(),
    }
}

/// Result of searching the other repo for a matching commit by title.
#[derive(Debug)]
pub enum TitleMatch {
    /// No title to search for (empty description).
    NoTitle,
    /// Exactly one commit with the same title: unambiguous match.
    One(String),
    /// Multiple commits share the same title: ambiguous.
    Ambiguous(usize),
    /// No commit in the other repo has the same title.
    None,
}

/// Find matching commits in the other repo by title (exact match).
pub fn find_matching_commit(
    commit: &jj_lib::commit::Commit,
    other_workspace: &Workspace,
    other_repo: &Arc<ReadonlyRepo>,
) -> Result<TitleMatch, Box<dyn std::error::Error>> {
    let title = commit.description().lines().next().unwrap_or(""); // OK: obvious
    if title.is_empty() {
        return Ok(TitleMatch::NoTitle);
    }

    let all_ids = common::resolve_revset(other_workspace, other_repo, "all()")?;
    let root_id = other_repo.store().root_commit_id().clone();

    let mut matches = Vec::new();
    for cid in &all_ids {
        if *cid == root_id {
            continue;
        }
        let other_commit = other_repo.store().get_commit(cid)?;
        let other_title = other_commit.description().lines().next().unwrap_or(""); // OK: obvious
        if other_title == title {
            let full_hex =
                jj_lib::hex_util::encode_reverse_hex(other_commit.change_id().as_bytes());
            matches.push(full_hex);
        }
    }

    match matches.len() {
        0 => Ok(TitleMatch::None),
        1 => {
            #[allow(clippy::unwrap_used)]
            // OK: `1 =>` arm guarantees matches.len() == 1
            Ok(TitleMatch::One(matches.into_iter().next().unwrap()))
        }
        n => Ok(TitleMatch::Ambiguous(n)),
    }
}

/// Resolve a short change ID to its full hex via the other repo.
pub fn resolve_full_change_id(
    bare_id: &str,
    other_workspace: &Workspace,
    other_repo: &Arc<ReadonlyRepo>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if let Ok(commit_ids) = common::resolve_revset(other_workspace, other_repo, bare_id)
        && let Some(cid) = commit_ids.first()
    {
        let other_commit = other_repo.store().get_commit(cid)?;
        let full_hex = jj_lib::hex_util::encode_reverse_hex(other_commit.change_id().as_bytes());
        Ok(Some(full_hex))
    } else {
        Ok(None)
    }
}

/// Extract the bare change ID from an ochid value: what follows the
/// path form's last `/`, or the URL form's `#`.
pub fn extract_bare_id(ochid_value: &str) -> &str {
    if let Some(pos) = ochid_value.rfind(['/', '#']) {
        &ochid_value[pos + 1..]
    } else {
        ochid_value
    }
}

/// An ochid value as a column shows it: the URL form's `#<id>`, since
/// the URL would push the column out, and any other form as it is.
pub fn ochid_for_display(ochid_value: &str) -> String {
    if is_url_form(ochid_value) {
        format!("#{}", extract_bare_id(ochid_value))
    } else {
        ochid_value.to_string()
    }
}

/// Fix the ochid trailer in a commit description.
///
/// If `resolved_id` is provided, it replaces the bare changeID entirely
/// (used when the existing ID is too short to extend by truncation alone).
/// A URL form keeps its URL, since it is a valid prefix and not the
/// label's to replace.
pub fn fix_ochid_in_description(
    desc: &str,
    other_prefix: &str,
    id_len: usize,
    new_title: Option<&str>,
    resolved_id: Option<&str>,
) -> String {
    let mut lines: Vec<String> = desc.lines().map(|l| l.to_string()).collect();

    // Fix title if requested
    if let Some(title) = new_title
        && !lines.is_empty()
    {
        lines[0] = title.to_string();
    }

    // Fix ochid trailer
    for line in &mut lines {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("ochid:") {
            let value = value.trim();
            let bare_id = extract_bare_id(value);
            let actual_prefix = &value[..value.len() - bare_id.len()];
            let prefix = if is_url_prefix(actual_prefix) {
                actual_prefix
            } else {
                other_prefix
            };
            let id = if let Some(rid) = resolved_id {
                &rid[..rid.len().min(id_len)]
            } else {
                &bare_id[..bare_id.len().min(id_len)]
            };
            *line = format!("ochid: {prefix}{id}");
        }
    }

    // Preserve trailing newline if original had one
    let mut result = lines.join("\n");
    if desc.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// Append an ochid trailer to a commit description.
pub fn append_ochid_trailer(
    desc: &str,
    other_prefix: &str,
    change_id: &str,
    id_len: usize,
) -> String {
    let short_id = &change_id[..change_id.len().min(id_len)];
    let trailer = format!("ochid: {other_prefix}{short_id}");

    let mut result = desc.trim_end().to_string();
    // Add blank line before trailer if body exists, otherwise just newline
    if result.lines().count() > 1 {
        result.push('\n');
    } else {
        result.push_str("\n\n");
    }
    result.push_str(&trailer);
    result.push('\n');
    result
}

/// Extract the values of column-0 `ochid:` trailer lines from a
/// commit description, in order of appearance: the crate's one
/// string-level ochid parser.
///
/// - Column-0 only: an indented `ochid:` is quoted prose, not a
///   trailer (git trailers sit at column 0).
/// - Values are whitespace-trimmed. Blank values are skipped.
pub fn extract_ochids(desc: &str) -> Vec<String> {
    desc.lines()
        .filter_map(|line| line.strip_prefix("ochid:"))
        .map(str::trim)
        .filter(|value| is_trailer_value(value))
        .map(str::to_string)
        .collect()
}

/// Extract "the" ochid value from a description string (without
/// needing a Commit): the *last* `ochid:` trailer, since trailers
/// sit at the end of the body. On a multi-ochid commit the
/// single-value view is the final trailer.
pub fn extract_ochid_from_desc(desc: &str) -> Option<String> {
    extract_ochids(desc).pop()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A body sentence starting `ochid: ` at column 0 is not a trailer, so
    /// it is not read as one and a rewrite keeps it.
    #[test]
    fn a_sentence_is_not_a_trailer() {
        let desc = "feat: x\n\nochid: trailers. A squash loses them.\n\nochid: abc";
        assert_eq!(extract_ochids(desc), ["abc"]);
        assert_eq!(Trailers::of(desc).ochids, ["abc"]);
        assert_eq!(with_trailer(desc, "def"), desc.replace("abc", "def"));
    }

    /// Stamped trailers replace what the text carries and sit above a
    /// custom trailer, and body lines that only look like one stay body.
    #[test]
    fn trailers_round_trip_and_keep_a_custom_one() {
        let t = Trailers {
            ochids: vec!["abc".to_string()],
            ocid: Some("u/commit/xxxx".to_string()),
            pcids: vec!["w/commit/1".to_string(), "w/commit/2".to_string()],
        };
        let desc = with_trailers("feat: x\n\nBody.\n", &t);
        assert_eq!(
            desc,
            "feat: x\n\nBody.\n\nochid: abc\nocid: u/commit/xxxx\npcid: w/commit/1\npcid: w/commit/2"
        );
        assert_eq!(Trailers::of(&desc), t);
        assert_eq!(with_trailers(&desc, &t), desc, "stamping again is the same");

        let custom = "feat: x\n\nBody.\n\nochid: old\nreviewed-by: wink\nCo-Authored-By: a <b@c>";
        assert_eq!(
            with_trailers(custom, &t),
            "feat: x\n\nBody.\n\nochid: abc\nocid: u/commit/xxxx\npcid: w/commit/1\npcid: w/commit/2\n\
             reviewed-by: wink\nCo-Authored-By: a <b@c>"
        );

        assert_eq!(
            with_trailer("feat: x\n\nSolution: do it.", "abc"),
            "feat: x\n\nSolution: do it.\n\nochid: abc",
            "a one-line paragraph is body"
        );
        assert_eq!(with_trailer("feat: x", "abc"), "feat: x\n\nochid: abc");
        assert_eq!(
            with_trailers("feat: x\n\nochid: abc", &Trailers::default()),
            "feat: x"
        );
    }

    /// Build `<base>/ws` (+ optional `<bot-dir>`) with per-side
    /// file-relative `[repos]` registries in each repo dir.
    fn ws_fixture(tag: &str, bot_dir: Option<&str>) -> (std::path::PathBuf, std::path::PathBuf) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let base = std::env::temp_dir().join(format!("vc-x1-desc-helpers-{tag}-{ts}"));
        let root = base.join("ws");
        std::fs::create_dir_all(&root).unwrap();
        let work_block = match bot_dir {
            Some(b) => format!("[repos]\nwork = \".\"\nagent = \"{b}\"\n"),
            None => "[repos]\nwork = \".\"\n".to_string(),
        };
        std::fs::write(root.join(VC_CONFIG_FILE), &work_block).unwrap();
        if let Some(b) = bot_dir {
            let bot = root.join(b);
            std::fs::create_dir_all(&bot).unwrap();
            std::fs::write(
                bot.join(VC_CONFIG_FILE),
                "[repos]\nwork = \"..\"\nagent = \".\"\n",
            )
            .unwrap();
        }
        (base, root)
    }

    /// Work side (the workspace root) -> `/`.
    #[test]
    fn prefix_for_work_side() {
        let (base, root) = ws_fixture("prefix-work", Some(".claude"));
        assert_eq!(ochid_prefix_for(&root).unwrap(), "/");
        std::fs::remove_dir_all(&base).ok();
    }

    /// Bot side -> the canonical bot label `/.claude/`.
    #[test]
    fn prefix_for_bot_side() {
        let (base, root) = ws_fixture("prefix-bot", Some(".claude"));
        assert_eq!(
            ochid_prefix_for(&root.join(".claude")).unwrap(),
            "/.claude/"
        );
        std::fs::remove_dir_all(&base).ok();
    }

    /// A non-`.claude` bot dir still gets the canonical label:
    /// the prefix is an opaque registry label, not a filesystem
    /// spelling.
    #[test]
    fn prefix_for_custom_bot_dir() {
        let (base, root) = ws_fixture("prefix-custom", Some(".bot"));
        assert_eq!(ochid_prefix_for(&root.join(".bot")).unwrap(), "/.claude/");
        std::fs::remove_dir_all(&base).ok();
    }

    /// A repo that is no workspace's bot side -> work prefix `/`.
    #[test]
    fn prefix_for_por_repo() {
        let (base, root) = ws_fixture("prefix-por", None);
        assert_eq!(ochid_prefix_for(&root).unwrap(), "/");
        std::fs::remove_dir_all(&base).ok();
    }

    const AGENT_URL: &str = "https://github.com/winksaville/vc-x1.agent-session#";

    #[test]
    fn bare_id_of_each_form() {
        assert_eq!(extract_bare_id("tzupykyyvnrp"), "tzupykyyvnrp");
        assert_eq!(extract_bare_id("/tzupykyyvnrp"), "tzupykyyvnrp");
        assert_eq!(extract_bare_id("/.claude/tzupykyyvnrp"), "tzupykyyvnrp");
        assert_eq!(
            extract_bare_id(&format!("{AGENT_URL}tzupykyyvnrp")),
            "tzupykyyvnrp"
        );
    }

    fn agent_forms(url: Option<&str>) -> OchidForms {
        OchidForms {
            label: "/.claude/".to_string(),
            path: "/.agent-session/".to_string(),
            url: url.map(str::to_string),
        }
    }

    /// The label, the path form, and with no URL to compare any URL
    /// are valid, and a URL with no `#` is not, since its last path
    /// segment would read as the ID.
    #[test]
    fn a_prefix_is_a_form_of_the_repo() {
        let forms = agent_forms(None);
        assert!(forms.accepts("/.claude/"));
        assert!(forms.accepts("/.agent-session/"));
        assert!(forms.accepts(AGENT_URL));
        assert!(!forms.accepts("/wrong/"));
        assert!(!forms.accepts("/"));
        assert!(!forms.accepts("https://github.com/winksaville/"));
        assert!(!forms.accepts("vc-x1#"));
    }

    /// With a URL to compare, the trailer's URL must name that repo,
    /// in any spelling of it, and a wrong one reports the right one.
    #[test]
    fn a_url_must_name_the_repo() {
        let want = "https://github.com/winksaville/vc-x1.agent-session";
        let forms = agent_forms(Some(want));
        assert!(forms.accepts(AGENT_URL));
        assert!(forms.accepts("https://github.com/winksaville/vc-x1.agent-session.git#"));
        assert!(!forms.accepts("https://github.com/winksaville/vc-x1#"));
        assert_eq!(
            forms.wanted("https://github.com/winksaville/vc-x1#"),
            format!("{want}#")
        );
        assert_eq!(forms.wanted("/wrong/"), "/.claude/");
    }

    #[test]
    fn a_column_shows_the_url_form_short() {
        assert_eq!(
            ochid_for_display(&format!("{AGENT_URL}tzupykyyvnrp")),
            "#tzupykyyvnrp"
        );
        assert_eq!(
            ochid_for_display("/.claude/tzupykyyvnrp"),
            "/.claude/tzupykyyvnrp"
        );
    }

    #[test]
    fn a_fix_keeps_the_url() {
        let desc = format!("Title\n\nochid: {AGENT_URL}abcdefghijklmnop\n");
        let result = fix_ochid_in_description(&desc, "/.claude/", DEFAULT_ID_LEN, None, None);
        assert_eq!(result, format!("Title\n\nochid: {AGENT_URL}abcdefghijkl\n"));
    }

    #[test]
    fn fix_bare_id() {
        let desc = "Some title\n\nBody text.\n\nochid: tzupykyyvnrp\n";
        let result = fix_ochid_in_description(desc, "/.claude/", DEFAULT_ID_LEN, None, None);
        assert!(result.contains("ochid: /.claude/tzupykyyvnrp"));
        assert!(result.starts_with("Some title\n"));
    }

    #[test]
    fn fix_wrong_prefix() {
        let desc = "Title\n\nochid: /wrong/tzupykyyvnrp\n";
        let result = fix_ochid_in_description(desc, "/.claude/", DEFAULT_ID_LEN, None, None);
        assert!(result.contains("ochid: /.claude/tzupykyyvnrp"));
    }

    #[test]
    fn already_correct() {
        let desc = "Title\n\nochid: /.claude/tzupykyyvnrp\n";
        let result = fix_ochid_in_description(desc, "/.claude/", DEFAULT_ID_LEN, None, None);
        assert_eq!(result, desc);
    }

    #[test]
    fn truncate_long_id() {
        let desc = "Title\n\nochid: abcdefghijklmnop\n";
        let result = fix_ochid_in_description(desc, "/", DEFAULT_ID_LEN, None, None);
        assert!(result.contains("ochid: /abcdefghijkl"));
        assert!(!result.contains("mnop"));
    }

    #[test]
    fn fix_title_and_ochid() {
        let desc = "Old title\n\nochid: bare12345678\n";
        let result =
            fix_ochid_in_description(desc, "/.claude/", DEFAULT_ID_LEN, Some("New title"), None);
        assert!(result.starts_with("New title\n"));
        assert!(result.contains("ochid: /.claude/bare12345678"));
    }

    #[test]
    fn no_ochid_no_change() {
        let desc = "Title\n\nNo trailer here.\n";
        let result = fix_ochid_in_description(desc, "/.claude/", DEFAULT_ID_LEN, None, None);
        assert_eq!(result, desc);
    }

    #[test]
    fn other_path_without_trailing_slash() {
        let desc = "Title\n\nochid: abcdefghijkl\n";
        let result = fix_ochid_in_description(desc, "/", DEFAULT_ID_LEN, None, None);
        assert!(result.contains("ochid: /abcdefghijkl"));
    }

    #[test]
    fn custom_id_len() {
        let desc = "Title\n\nochid: abcdefghijklmnop\n";
        let result = fix_ochid_in_description(desc, "/", 8, None, None);
        assert!(result.contains("ochid: /abcdefgh"));
        assert!(!result.contains("ijkl"));
    }

    #[test]
    fn extend_short_id_with_resolved() {
        let desc = "Title\n\nochid: /abcdefgh\n";
        let result =
            fix_ochid_in_description(desc, "/", DEFAULT_ID_LEN, None, Some("abcdefghijkl"));
        assert!(result.contains("ochid: /abcdefghijkl"));
    }

    #[test]
    fn resolved_id_with_prefix_fix() {
        let desc = "Title\n\nochid: /wrong/abcdefgh\n";
        let result = fix_ochid_in_description(
            desc,
            "/.claude/",
            DEFAULT_ID_LEN,
            None,
            Some("abcdefghijklmnop"),
        );
        assert!(result.contains("ochid: /.claude/abcdefghijkl"));
    }

    #[test]
    fn append_ochid_title_only() {
        let desc = "Title\n";
        let result = append_ochid_trailer(desc, "/.claude/", "abcdefghijklmnop", DEFAULT_ID_LEN);
        assert_eq!(result, "Title\n\nochid: /.claude/abcdefghijkl\n");
    }

    #[test]
    fn append_ochid_with_body() {
        let desc = "Title\n\nSome body text.\n";
        let result = append_ochid_trailer(desc, "/", "abcdefghijklmnop", DEFAULT_ID_LEN);
        assert_eq!(result, "Title\n\nSome body text.\nochid: /abcdefghijkl\n");
    }

    #[test]
    fn append_ochid_root_prefix() {
        let desc = "Title\n";
        let result = append_ochid_trailer(desc, "/", "xyzxyzxyzxyz", DEFAULT_ID_LEN);
        assert_eq!(result, "Title\n\nochid: /xyzxyzxyzxyz\n");
    }

    #[test]
    fn extract_bare_id_with_prefix() {
        assert_eq!(extract_bare_id("/.claude/abcdefghijkl"), "abcdefghijkl");
    }

    #[test]
    fn extract_bare_id_root_prefix() {
        assert_eq!(extract_bare_id("/abcdefghijkl"), "abcdefghijkl");
    }

    #[test]
    fn extract_bare_id_no_prefix() {
        assert_eq!(extract_bare_id("abcdefghijkl"), "abcdefghijkl");
    }

    #[test]
    fn extract_ochid_from_desc_found() {
        let desc = "Title\n\nBody.\n\nochid: /.claude/abcdefghijkl\n";
        assert_eq!(
            extract_ochid_from_desc(desc),
            Some("/.claude/abcdefghijkl".to_string())
        );
    }

    #[test]
    fn extract_ochid_from_desc_missing() {
        let desc = "Title\n\nNo trailer.\n";
        assert_eq!(extract_ochid_from_desc(desc), None);
    }

    #[test]
    fn extract_ochid_from_desc_multi_takes_last() {
        let desc = "Title\n\nBody.\n\nochid: /first0first0\nochid: /.claude/last1last1x\n";
        assert_eq!(
            extract_ochid_from_desc(desc),
            Some("/.claude/last1last1x".to_string())
        );
    }

    #[test]
    fn extract_ochids_none() {
        assert!(extract_ochids("").is_empty());
        assert!(extract_ochids("title\n\nbody, no trailers\n").is_empty());
    }

    #[test]
    fn extract_ochids_trailers() {
        let desc = "title\n\nbody line\n\nochid: /abcdefabcdef\nochid: /.claude/xyzxyzxyzxyz\n";
        assert_eq!(
            extract_ochids(desc),
            vec!["/abcdefabcdef", "/.claude/xyzxyzxyzxyz"]
        );
    }

    #[test]
    fn extract_ochids_column_zero_only() {
        // Indented mentions aren't trailers. Bare "ochid:" has no value.
        let desc = "title\n\n  ochid: /indented\nochid:\nochid:   /trimmed  \n";
        assert_eq!(extract_ochids(desc), vec!["/trimmed"]);
    }
}

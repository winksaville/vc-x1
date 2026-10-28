//! The three versions `--version` reports, and where each comes
//! from.
//!
//! They answer different questions and have different sources:
//!
//! - ours: `CARGO_PKG_VERSION`, compile time, and the commit the
//!   build was made from with the state of its source, `VC_X1_COMMIT`
//!   and `VC_X1_STATE` from `build.rs`, shown as semver build
//!   metadata, `0.85.7+d70cf9ecdfd9`. `.dirty` follows for a build
//!   with edits, and `.unknown` for one whose edits were not checked,
//!   a hand-typed cargo build, whose commit comes from git's `HEAD`.
//! - jj-lib's: `JJ_LIB_VERSION`, resolved from `Cargo.lock` by
//!   `build.rs`, because jj-lib exports no version constant. It is
//!   what writes ops, so it is compile-time by nature, not by
//!   compromise.
//! - the data's: read through jj-lib's public accessors only. What
//!   comes back is backend *type* names (the `.jj/repo/<backend>/type`
//!   files `RepoLoader::init_from_file_system` reads), because that
//!   is all jj-lib exposes: as of 0.44 it has no public version
//!   constant and no public accessor for one. The commit index does
//!   carry a format version, but the constant is `pub(super)` and a
//!   mismatch self-heals by reindexing, while the op store, the
//!   thing we would be co-writing, carries no stamp at all.

use std::path::Path;

use jj_lib::config::StackedConfig;
use jj_lib::default_backend_factories::{
    default_backend_factories, default_working_copy_factories,
};
use jj_lib::settings::UserSettings;
use jj_lib::workspace::Workspace;

use crate::common;
use crate::jj::session::home_dir;

/// The full commit id this binary was built from, `None` for a build
/// with no git checkout to read, a source package's.
///
/// `build.rs` takes it from `VC_X1_BUILD_COMMIT`, which `vc-x1 build`,
/// for every profile, and `validate` pass, else from git's `HEAD`, and
/// reruns when either changes.
pub fn build_commit() -> Option<&'static str> {
    option_env!("VC_X1_COMMIT").filter(|c| !c.is_empty())
}

/// `state`: the state of the source code that was compiled, relative to
/// the commit the build names. Clean when the work repo's `@` was
/// empty, so the compiled code is exactly that commit, dirty when `@`
/// held edits that were compiled too, unknown when no one checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceState {
    /// `@` was empty, so the build is exactly its commit's tree.
    Clean,
    /// `@` held edits, so the build is its commit plus them.
    Dirty,
    /// Not checked: the commit came from git's `HEAD`, and whether
    /// `@` held edits is a jj question a build script cannot ask.
    Unknown,
}

impl SourceState {
    /// The state's word, as the variables and the banner spell it.
    pub fn as_str(self) -> &'static str {
        match self {
            SourceState::Clean => "clean",
            SourceState::Dirty => "dirty",
            SourceState::Unknown => "unknown",
        }
    }

    /// The state a word names, `None` for any other word.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "clean" => Some(SourceState::Clean),
            "dirty" => Some(SourceState::Dirty),
            "unknown" => Some(SourceState::Unknown),
            _ => None,
        }
    }
}

/// The state of the source code compiled into this binary, from
/// `VC_X1_STATE`, `None` for a build with no commit.
pub fn build_state() -> Option<SourceState> {
    option_env!("VC_X1_STATE").and_then(SourceState::parse)
}

/// The banner's build metadata: `+<12-char commit>`, then `.dirty` or
/// `.unknown` for a source that was not clean, or empty for a build
/// with no commit.
pub fn build_metadata() -> String {
    render_build_metadata(build_commit(), build_state())
}

/// [`build_metadata`] over explicit facts, for the tests. A commit with
/// no state reads as unknown.
fn render_build_metadata(commit: Option<&str>, state: Option<SourceState>) -> String {
    match commit {
        None => String::new(),
        Some(c) => {
            let short = &c[..c.len().min(12)];
            // OK: a commit with no state was not checked, which is unknown
            match state.unwrap_or(SourceState::Unknown) {
                SourceState::Clean => format!("+{short}"),
                other => format!("+{short}.{}", other.as_str()),
            }
        }
    }
}

/// The `vc-x1 version` line for the build's provenance.
fn build_line(commit: Option<&str>, state: Option<SourceState>) -> String {
    match commit {
        None => "build none: no git checkout to read the commit from".to_string(),
        // OK: a commit with no state was not checked, which is unknown
        Some(c) => match state.unwrap_or(SourceState::Unknown) {
            SourceState::Clean => format!("build {c}, state clean, `@` empty"),
            SourceState::Dirty => {
                format!("build {c}, state dirty, `@` held edits, so not exactly that commit")
            }
            SourceState::Unknown => {
                format!("build {c}, state unknown, `@` not checked, a build vc-x1 did not make")
            }
        },
    }
}

/// The jj-lib version this binary links against.
pub fn jj_lib_version() -> &'static str {
    env!("JJ_LIB_VERSION")
}

/// Why the gate refused to let a run touch a repo.
///
/// Three outcomes rather than one, because the fix differs: install
/// jj, report an unexpected `jj -V` format, or reconcile versions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateError {
    /// `jj -V` could not be run at all.
    JjUnavailable(String),
    /// `jj -V` ran but printed something we cannot read a triple from.
    Unparsable(String),
    /// Both versions known, and different.
    Mismatch {
        /// Our linked jj-lib.
        ours: String,
        /// The `jj` on `$PATH`.
        theirs: String,
    },
}

impl std::fmt::Display for GateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateError::JjUnavailable(why) => write!(
                f,
                "cannot run `jj -V` ({why}); vc-x1 writes through jj-lib \
                 {ours} and will not touch a repo without knowing the jj \
                 it shares that repo with",
                ours = jj_lib_version()
            ),
            GateError::Unparsable(got) => write!(
                f,
                "cannot read a version from `jj -V` output {got:?}; \
                 expected `jj <x.y.z>[-<hash>]`"
            ),
            GateError::Mismatch { ours, theirs } => write!(
                f,
                "jj-lib {ours} != jj {theirs}; refusing to run, because \
                 either version can silently drop what the other wrote. \
                 Fix by matching them, or pass --allow-jj-mismatch to \
                 override for this one run"
            ),
        }
    }
}

impl std::error::Error for GateError {}

/// Run `jj -V` and return its trimmed stdout: the gate's one
/// spawn, owned here so the register entry and its subject live
/// in the same module.
fn probe_jj_version() -> Result<String, GateError> {
    // Register entry 1 (clippy.toml): the version gate's probe.
    // The gate compares the user's jj CLI against our jj-lib, so
    // the CLI spawn is its subject matter.
    #[allow(clippy::disallowed_methods)]
    let output = std::process::Command::new("jj")
        .arg("-V")
        .output()
        .map_err(|e| GateError::JjUnavailable(format!("failed to run jj: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(GateError::JjUnavailable(format!("jj -V failed: {stderr}")));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// The `jj` on `$PATH`, as a version triple, resolved once.
///
/// Cached because the gate and the report both want it and it costs a
/// process spawn: one per run, never one per operation.
fn jj_cli_triple() -> &'static Result<String, GateError> {
    static TRIPLE: std::sync::OnceLock<Result<String, GateError>> = std::sync::OnceLock::new();
    TRIPLE.get_or_init(|| {
        let out = probe_jj_version()?;
        match parse_jj_triple(&out) {
            Some(triple) => Ok(triple.to_string()),
            None => Err(GateError::Unparsable(out.trim().to_string())),
        }
    })
}

/// Pull the `x.y.z` out of `jj -V` output like `jj 0.43.0-<hash>`.
///
/// Prefix match on the triple, never whole-string equality: the hash
/// suffix identifies a build, and we have no way to map a build to a
/// schema.
fn parse_jj_triple(output: &str) -> Option<&str> {
    let rest = output.trim().strip_prefix("jj ")?;
    let triple = rest.split(['-', '+', ' ']).next()?;
    (!triple.is_empty()).then_some(triple)
}

/// The rule: if our jj-lib and the user's jj differ, stop.
///
/// See [the policy](../notes/jj-version-policy.md). Equality rather
/// than a compatibility test, because compatibility is not something
/// we can compute: an unequal pair is *unknown*, and unknown on a
/// path that can write must stop.
pub fn gate() -> Result<(), GateError> {
    let theirs = jj_cli_triple().clone()?;
    if theirs == jj_lib_version() {
        Ok(())
    } else {
        Err(GateError::Mismatch {
            ours: jj_lib_version().to_string(),
            theirs,
        })
    }
}

/// The backend type names one repo's jj data records about itself.
///
/// One field per `type` file jj-lib loads a backend from. Every
/// value is an identity, never a version.
#[derive(Debug, PartialEq, Eq)]
pub struct DataVersion {
    /// Commit backend, e.g. `git`.
    pub commit: String,
    /// Operation store, e.g. `simple_op_store`.
    pub op: String,
    /// Operation heads store, e.g. `simple_op_heads_store`.
    pub op_heads: String,
    /// Commit index store, e.g. `default`.
    pub index: String,
    /// Submodule store, e.g. `default`.
    pub submodule: String,
    /// Working copy, e.g. `local`.
    pub working_copy: String,
}

impl DataVersion {
    /// Render as one `key=value` line, without the repo label.
    pub fn fields(&self) -> String {
        format!(
            "commit={} op={} op-heads={} index={} submodule={} working-copy={}",
            self.commit, self.op, self.op_heads, self.index, self.submodule, self.working_copy
        )
    }
}

/// Read one repo's recorded backend types through jj-lib.
///
/// Loads the workspace only, never `load_at_head`: resolving op
/// heads can merge divergent ones, which is a write, and reporting
/// a version must not mutate the repo it reports on.
pub fn data_version(path: &Path) -> Result<DataVersion, Box<dyn std::error::Error>> {
    let config = StackedConfig::with_defaults();
    let settings = UserSettings::from_config_and_home_dir(config, home_dir())?;
    let store_factories = default_backend_factories();
    let working_copy_factories = default_working_copy_factories();

    let workspace = Workspace::load(&settings, path, &store_factories, &working_copy_factories)?;
    let loader = workspace.repo_loader();

    Ok(DataVersion {
        commit: loader.store().backend().name().to_string(),
        op: loader.op_store().name().to_string(),
        op_heads: loader.op_heads_store().name().to_string(),
        index: loader.index_store().name().to_string(),
        submodule: loader.submodule_store().name().to_string(),
        working_copy: workspace.working_copy().name().to_string(),
    })
}

/// Render one repo's `jj-data` line, load failure included.
fn data_line(label: &str, path: &Path) -> String {
    match data_version(path) {
        Ok(data) => format!("jj-data {label} {}", data.fields()),
        Err(e) => format!("jj-data {label} unavailable: {e}"),
    }
}

/// Build the full `--version` report, one line per fact.
///
/// The repo lines are best-effort: outside a workspace, or on a
/// repo that fails to load, the line says so rather than being
/// dropped, so a missing repo never reads as a clean report.
pub fn report(banner: &str) -> Vec<String> {
    let root = common::find_workspace_root();
    let mut lines = vec![
        banner.to_string(),
        build_line(build_commit(), build_state()),
        crate::agent_files::report_line(root.as_deref()),
        format!("jj-lib {}", jj_lib_version()),
    ];

    match jj_cli_triple() {
        Ok(theirs) => lines.push(format!("jj {theirs}")),
        Err(e) => lines.push(format!("jj unavailable: {e}")),
    }

    // The report is the diagnostic you run when the gate has
    // stopped you, so it answers rather than refusing. But reading
    // backend types opens repos, which is the thing the gate
    // forbids, so on a refusal it says what it withheld.
    if let Err(e) = gate() {
        lines.push(format!("jj-data withheld: {e}"));
        return lines;
    }

    match root {
        None => lines.push("jj-data none: not in a vc-x1 workspace".to_string()),
        Some(root) => {
            lines.push(data_line("work", &root));
            match common::bot_repo_path(&root) {
                Ok(Some(bot)) => lines.push(data_line("agent", &bot)),
                Ok(None) => lines.push("jj-data bot unavailable: no bot repo".to_string()),
                Err(e) => lines.push(format!("jj-data bot unavailable: {e}")),
            }
        }
    }

    lines.push(
        "jj-data note: backend type names only; jj-lib exposes no on-disk format version"
            .to_string(),
    );
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The compiled-in version must be jj-lib's own, not another
    /// package's. Scanned here deliberately unlike `build.rs` does
    /// it (locate the name line, take the next version line), so a
    /// parser that drifted onto the wrong `[[package]]` block
    /// fails the test rather than agreeing with itself.
    #[test]
    fn jj_lib_version_matches_the_lock() {
        let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock"))
            .expect("Cargo.lock is readable");
        let mut lines = lock
            .lines()
            .skip_while(|l| l.trim() != r#"name = "jj-lib""#);
        assert!(lines.next().is_some(), "no jj-lib package in Cargo.lock");
        let version = lines
            .find_map(|l| l.trim().strip_prefix("version = "))
            .map(|v| v.trim_matches('"').to_string());
        assert_eq!(version.as_deref(), Some(jj_lib_version()));
    }

    #[test]
    fn jj_lib_version_is_a_version_triple() {
        let v = jj_lib_version();
        let parts: Vec<&str> = v.split('.').collect();
        assert_eq!(parts.len(), 3, "not a triple: {v}");
        for part in parts {
            assert!(
                part.chars().next().is_some_and(|c| c.is_ascii_digit()),
                "not numeric: {v}"
            );
        }
    }

    /// The banner, the build, then the set version, then jj-lib: what
    /// describes the binary and the run comes before the tool it
    /// links.
    #[test]
    fn report_starts_with_banner_build_set_version_and_jj_lib() {
        let lines = report("vc-x1-dev 0.0.0");
        assert_eq!(lines[0], "vc-x1-dev 0.0.0");
        assert!(lines[1].starts_with("build "), "got: {}", lines[1]);
        assert!(lines[2].starts_with("agent-files "), "got: {}", lines[2]);
        assert_eq!(lines[3], format!("jj-lib {}", jj_lib_version()));
    }

    /// The metadata is empty without a commit, `+` and twelve hex
    /// with one, then `.dirty` or `.unknown` for a source that was not
    /// clean, a commit with no state reading as unknown.
    #[test]
    fn build_metadata_forms() {
        use SourceState::*;
        let c = "d70cf9ecdfd9aaaabbbbccccddddeeeeffff0000";
        assert_eq!(render_build_metadata(None, None), "");
        assert_eq!(render_build_metadata(Some(c), Some(Clean)), "+d70cf9ecdfd9");
        assert_eq!(
            render_build_metadata(Some(c), Some(Dirty)),
            "+d70cf9ecdfd9.dirty"
        );
        assert_eq!(
            render_build_metadata(Some(c), Some(Unknown)),
            "+d70cf9ecdfd9.unknown"
        );
        assert_eq!(
            render_build_metadata(Some(c), None),
            "+d70cf9ecdfd9.unknown"
        );
    }

    /// The report's build line names the full commit and the state, or
    /// says there was no commit to read.
    #[test]
    fn build_line_forms() {
        use SourceState::*;
        let c = "d70cf9ecdfd9aaaabbbbccccddddeeeeffff0000";
        assert!(build_line(None, None).starts_with("build none: "));
        assert_eq!(
            build_line(Some(c), Some(Clean)),
            format!("build {c}, state clean, `@` empty")
        );
        assert!(build_line(Some(c), Some(Dirty)).contains("state dirty"));
        assert!(build_line(Some(c), Some(Unknown)).contains("state unknown"));
    }

    /// Each state's word parses back to it, and no other word does.
    #[test]
    fn source_state_words_round_trip() {
        for st in [SourceState::Clean, SourceState::Dirty, SourceState::Unknown] {
            assert_eq!(SourceState::parse(st.as_str()), Some(st));
        }
        assert_eq!(SourceState::parse("true"), None);
    }

    #[test]
    fn parses_the_triple_out_of_jj_v() {
        assert_eq!(parse_jj_triple("jj 0.43.0-89f62ede8c1c\n"), Some("0.43.0"));
        assert_eq!(parse_jj_triple("jj 0.43.0"), Some("0.43.0"));
        assert_eq!(parse_jj_triple("jujutsu 0.43.0"), None);
        assert_eq!(parse_jj_triple("jj "), None);
    }

    /// The gate must pass in this workspace: the whole cycle's
    /// premise is that we run against a matching jj. A failure here
    /// means the lock and the installed jj have drifted, which is
    /// exactly what the gate exists to catch.
    #[test]
    fn gate_passes_against_the_installed_jj() {
        assert_eq!(
            gate(),
            Ok(()),
            "jj-lib {} vs installed jj",
            jj_lib_version()
        );
    }

    #[test]
    fn fields_render_every_backend() {
        let data = DataVersion {
            commit: "git".to_string(),
            op: "simple_op_store".to_string(),
            op_heads: "simple_op_heads_store".to_string(),
            index: "default".to_string(),
            submodule: "default".to_string(),
            working_copy: "local".to_string(),
        };
        assert_eq!(
            data.fields(),
            "commit=git op=simple_op_store op-heads=simple_op_heads_store index=default \
             submodule=default working-copy=local"
        );
    }
}

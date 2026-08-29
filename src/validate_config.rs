//! The `validate-config` subcommand: check a workspace's config
//! files against the schema, their own links, and the structural
//! key every instance config carries.
//!
//! Four checks:
//!
//! - every key is one `vc-config.md` declares for that side, and a
//!   `str-list` key holds an array rather than a scalar
//! - the legacy `[workspace]` schema is rejected with a fix-it, and
//!   a dual workspace's two `[repos]` registries must resolve to the
//!   same pair
//! - the file's own `[[N]]` references and `#anchor` links resolve
//!   ([`crate::validate_anchors`]), and a `vc-config.md#<anchor>`
//!   link names a real key section, which is a schema lookup since
//!   build.rs derives those anchors from key paths
//! - a side's config carries `repos.work`, the one `required` key
//!
//! Read-only, and exits non-zero when anything is found. Split out
//! of `config --validate` at 0.80.6: `config` prints the schema,
//! and checking a file against it is a different verb that belongs
//! beside `validate-desc`, `validate-todo`, `validate-agent`, and
//! `validate-anchors`.

use std::error::Error;
use std::path::Path;

use clap::Args;
use log::{debug, info, trace, warn};

use crate::common::{bot_repo_path, find_workspace_root, reject_legacy_config};
use crate::config_cmd::{ConfigTarget, in_any, in_bot_side, in_work_side, parse_target};
use crate::config_schema::{Home, schema};
use crate::context::Context;
use crate::desc_helpers::VC_CONFIG_FILE;
use crate::options_flags::scope::Side;
use crate::subcommand::SubcommandRunner;

/// Clap-derived args for `validate-config`.
#[derive(Args, Debug)]
pub struct ValidateConfigArgs {
    /// What to check: side keyword(s) `work`, `agent`,
    /// `work,agent`, or a config-file path. The user config
    /// (`~/.config/vc-x1/config.toml`) has no keyword: pass its
    /// path.
    #[arg(value_parser = parse_target, default_value = "work,agent", verbatim_doc_comment)]
    pub target: ConfigTarget,
}

/// Inputs to the validate-config op, flat, owned, clap-free.
pub struct ValidateConfigParams {
    pub target: ConfigTarget,
}

impl From<&ValidateConfigArgs> for ValidateConfigParams {
    /// Convert clap-derived args into the flat params (total).
    fn from(a: &ValidateConfigArgs) -> Self {
        Self {
            target: a.target.clone(),
        }
    }
}

impl SubcommandRunner for ValidateConfigArgs {
    type Params = ValidateConfigParams;

    /// Delegate to the `From<&ValidateConfigArgs>` impl (total).
    fn to_params(&self) -> Result<Self::Params, String> {
        Ok(ValidateConfigParams::from(self))
    }

    /// Run the `validate_config` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        validate_config(ctx, params)
    }
}

/// Run the `validate-config` subcommand.
///
/// `ctx` is unused: the check reads config files and neither the
/// user config nor the `--log` path applies. It is present for the
/// uniform subcommand-layer signature.
pub fn validate_config(
    _ctx: &Context,
    params: &ValidateConfigParams,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("validate-config: enter");
    let root = find_workspace_root();
    let c = validate(params, root.as_deref())?;
    // The counts are the summary's point: they separate a pass that
    // checked something from one that had nothing to check.
    info!(
        "validate-config: {} file(s), {} key(s) and {} link(s) checked, {} required key(s), \
         {} problem(s) found",
        c.files, c.keys, c.links, c.required, c.findings
    );
    if c.findings > 0 {
        debug!("validate-config: exit with findings");
        Err(format!("validate-config: {} problem(s) found", c.findings).into())
    } else {
        debug!("validate-config: exit");
        Ok(())
    }
}

/// True if `actual` (a config key from a loaded config file) is
/// recognized by some `schema()` entry whose homes satisfy
/// `home_pred`.
///
/// - Non-dynamic entries match by exact path equality.
/// - Dynamic entries (`key.dynamic`) match segment-wise: equal
///   segment counts, each entry segment either equal to the actual
///   segment or a `<placeholder>` matching any single segment.
fn key_known(actual: &str, home_pred: fn(&[Home]) -> bool) -> bool {
    schema().iter().any(|key| {
        if !home_pred(key.homes) {
            return false;
        }
        if !key.dynamic {
            return key.path == actual;
        }
        let entry_segs: Vec<&str> = key.path.split('.').collect();
        let actual_segs: Vec<&str> = actual.split('.').collect();
        entry_segs.len() == actual_segs.len()
            && entry_segs
                .iter()
                .zip(actual_segs.iter())
                .all(|(e, a)| e == a || (e.starts_with('<') && e.ends_with('>')))
    })
}

/// What one run looked at, beside what it found.
///
/// The counts exist so a pass says how much it covered: "all checks
/// passed" over a config with two keys reads the same as over one
/// with forty, and the same complaint as bug #9's applies, that a
/// reader cannot tell a real pass from a vacuous one.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub files: usize,
    /// Keys resolved against the schema for that side.
    pub keys: usize,
    /// The file's own references and anchors, plus its links into a
    /// key's section of the schema documentation.
    pub links: usize,
    /// Required keys confirmed present.
    pub required: usize,
    pub findings: usize,
}

impl Counts {
    /// Fold one file's counts into a run's.
    fn add(&mut self, other: &Counts) {
        self.files += other.files;
        self.keys += other.keys;
        self.links += other.links;
        self.required += other.required;
        self.findings += other.findings;
    }

    /// Count a finding that belongs to no file (a legacy schema, an
    /// unresolvable side).
    fn finding(&mut self) {
        self.findings += 1;
    }
}

/// Validate one config file against the schema, filtered to the
/// homes accepted at that file by `home_pred`.
///
/// - A missing file is not an error: `info!`s that it's absent and
///   returns an empty [`Counts`].
/// - Each key not recognized by `key_known` is reported with
///   `warn!`, naming `label` and the key. Keys are checked in
///   sorted order for stable output.
/// - A known `str-list` key whose value is not an array of quoted
///   strings is reported the same way, with the parse message.
/// - `require_structural` adds the `required` keys, which a side
///   target owes and a path target does not.
/// - Returns what was checked and what was found. A load error
///   (malformed TOML) propagates as `Err`.
fn validate_file(
    path: &Path,
    label: &str,
    home_pred: fn(&[Home]) -> bool,
    require_structural: bool,
) -> Result<Counts, Box<dyn Error>> {
    if !path.exists() {
        info!("{label}: {} not found, skipping", path.display());
        return Ok(Counts::default());
    }
    let map = crate::config_md::load_file(path)?;
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();

    let mut counts = Counts {
        files: 1,
        ..Counts::default()
    };
    for key in keys {
        counts.keys += 1;
        if !key_known(key, home_pred) {
            warn!("{label} ({}): unknown key {key:?}", path.display());
            counts.findings += 1;
            continue;
        }
        if is_str_list(key)
            && let Err(e) = crate::toml_simple::toml_get_list(&map, key)
        {
            warn!("{label} ({}): {e}", path.display());
            counts.findings += 1;
            continue;
        }
        trace!("{label} ({}): {key} is known", path.display());
    }

    if require_structural {
        for key in schema().iter().filter(|k| k.required && home_pred(k.homes)) {
            if map.contains_key(key.path) {
                counts.required += 1;
                trace!("{label} ({}): {} is present", path.display(), key.path);
                continue;
            }
            warn!(
                "{label} ({}): missing {:?}, which every instance config carries",
                path.display(),
                key.path
            );
            counts.findings += 1;
        }
    }

    let links = validate_links(path, label)?;
    counts.links += links.links;
    counts.findings += links.findings;

    debug!(
        "{label} ({}): {} key(s), {} link(s), {} required key(s), {} problem(s)",
        path.display(),
        counts.keys,
        counts.links,
        counts.required,
        counts.findings
    );
    Ok(counts)
}

/// Check a config file's prose links: its own anchors and
/// references, and its `vc-config.md#<anchor>` links into the
/// schema documentation.
///
/// The second half is why a config file gets more than
/// `validate-anchors` gives any other file. Those links are
/// generated from key paths, so "does this resolve" is a schema
/// lookup rather than the markdown crawl the cross-file check
/// deliberately leaves out.
fn validate_links(path: &Path, label: &str) -> Result<Counts, Box<dyn Error>> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let mut counts = Counts::default();

    let report = crate::validate_anchors::analyze(&text);
    for site in &report.sites {
        trace!(
            "{label} ({}):{}: {}",
            path.display(),
            site.line,
            site.detail
        );
    }
    for finding in &report.findings {
        warn!(
            "{label} ({}):{}: {}",
            path.display(),
            finding.line,
            finding.message
        );
        counts.findings += 1;
    }
    counts.links += report.counts.anchors_checked + report.counts.refs_cited;

    // Each key's `reference` ends with the anchor build.rs derived
    // from its path, so the schema is the authority on what a
    // vc-config.md fragment may name.
    let anchors: std::collections::BTreeSet<&str> = schema()
        .iter()
        .filter_map(|k| k.reference.rsplit_once('#').map(|(_, a)| a))
        .collect();
    for (file, slug, line) in crate::validate_anchors::cross_file_targets(&text) {
        if !file.ends_with("vc-config.md") {
            continue;
        }
        // A vc-config.md fragment is the one cross-file target this
        // check can resolve, so it counts as a link checked rather
        // than one skipped.
        counts.links += 1;
        if anchors.contains(slug.as_str()) {
            trace!(
                "{label} ({}):{line}: '#{slug}' names a key's section",
                path.display()
            );
            continue;
        }
        warn!(
            "{label} ({}):{line}: '#{slug}' is not a key's section in vc-config.md",
            path.display()
        );
        counts.findings += 1;
    }

    Ok(counts)
}

/// True when the schema types `path` as a `str-list`, whose value
/// `--validate` also checks for shape (an array of quoted strings).
fn is_str_list(path: &str) -> bool {
    schema()
        .iter()
        .any(|k| k.path == path && k.kind == crate::config_schema::ValueKind::StrList)
}

/// Validate the target's config file(s), returning the total count
/// of problems found (unknown keys, legacy/grammar rejections,
/// workspace-coherence failures).
///
/// - Keyword targets resolve against `root`. Outside a workspace
///   there is nothing to check (info + `Ok(0)`).
/// - The bot side resolves via `bot_repo_path`, which runs the
///   dual-preflight coherence check (bot dir exists, both configs
///   load, resolved `[repos]` agreement), its failure is
///   reported as a finding, not a hard error, so the work-side
///   report still lands.
/// - A path target carries no side information, so it validates
///   against the whole schema (any home's keys accepted).
fn validate(params: &ValidateConfigParams, root: Option<&Path>) -> Result<Counts, Box<dyn Error>> {
    let mut counts = Counts::default();
    match &params.target {
        ConfigTarget::Scope(scope) => {
            let Some(root) = root else {
                info!("not inside a workspace: nothing to validate");
                return Ok(counts);
            };
            if let Err(e) = reject_legacy_config(root) {
                // One finding, reported once: a legacy schema makes
                // the remaining checks redundant: the unknown-key
                // scan would re-flag the legacy keys and the
                // bot-side resolution would re-print this same
                // rejection.
                warn!("{e}");
                counts.finding();
                return Ok(counts);
            }
            for side in &scope.0 {
                match side {
                    Side::Work => match crate::config_md::vc_config_path(root) {
                        Ok(Some(path)) => {
                            counts.add(&validate_file(&path, "work config", in_work_side, true)?);
                        }
                        Ok(None) => {
                            counts.add(&validate_file(
                                &root.join(VC_CONFIG_FILE),
                                "work config",
                                in_work_side,
                                true,
                            )?);
                        }
                        Err(e) => {
                            warn!("{e}");
                            counts.finding();
                        }
                    },
                    Side::Bot => match bot_repo_path(root) {
                        Ok(Some(bot)) => match crate::config_md::vc_config_path(&bot) {
                            Ok(Some(path)) => {
                                counts.add(&validate_file(&path, "bot config", in_bot_side, true)?);
                            }
                            Ok(None) => {
                                counts.add(&validate_file(
                                    &bot.join(VC_CONFIG_FILE),
                                    "bot config",
                                    in_bot_side,
                                    true,
                                )?);
                            }
                            Err(e) => {
                                warn!("{e}");
                                counts.finding();
                            }
                        },
                        Ok(None) => info!("bot config: no bot repo configured, skipping"),
                        Err(e) => {
                            warn!("{e}");
                            counts.finding();
                        }
                    },
                }
            }
        }
        ConfigTarget::Path(path) => {
            // No structural requirement: a path target carries no
            // side information and may be the user config, which
            // has no `[repos]` to be missing.
            counts.add(&validate_file(
                path.as_path(),
                "config file",
                in_any,
                false,
            )?);
        }
    }
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_md::VC_CONFIG_MD;
    use crate::options_flags::scope::Scope;
    use crate::test_helpers::{Fixture, FixturePor};
    use std::path::PathBuf;

    /// Append TOML to a markdown config file, as its own fence.
    ///
    /// The carrier concatenates every `toml` fence in document
    /// order, so a trailing fence adds keys exactly as a trailing
    /// table did when the carrier was one TOML file.
    fn append(path: &Path, extra: &str) {
        let mut text = std::fs::read_to_string(path).expect("read config");
        text.push_str("\n```toml\n");
        text.push_str(extra.trim_start_matches('\n'));
        text.push_str("```\n");
        std::fs::write(path, text).expect("write config");
    }

    /// Write one config file into a fresh temp dir and hand back
    /// its path, for the checks that need a file rather than a
    /// whole workspace.
    fn config_file(tag: &str, body: &str) -> PathBuf {
        let dir = crate::test_helpers::unique_base(tag);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join(VC_CONFIG_MD);
        std::fs::write(&path, body).expect("write config");
        path
    }

    /// The shape this cycle inherited: citations with no
    /// definitions, and a reference into a key section the agent
    /// rename killed. Both passed `--validate` before this check.
    const INHERITED: &str = "\
# vc-x1 config file

The two repos
- work [[1]]
- bot [[2]]
```toml
[repos]
work = \".\"
agent = \".claude\"
```

The family
- member [[3]]
```toml
[family]
member = \"x\"
```

# References

[1]: vc-config.md#reposwork
[2]: vc-config.md#reposbot
";

    /// The inherited file's five defects, found rather than passed
    /// over: one uncited citation and one dead key section.
    #[test]
    fn validate_reports_the_inherited_links() {
        let path = config_file("config-links-inherited", INHERITED);
        let findings = validate_file(&path, "config", in_any, false)
            .expect("validate")
            .findings;
        assert_eq!(
            findings, 2,
            "the [[3]] with no definition, and #reposbot naming no key"
        );
        let _ = std::fs::remove_dir_all(path.parent().expect("temp dir"));
    }

    /// A `vc-config.md` fragment is checked against the schema
    /// rather than by crawling the file, since build.rs derives
    /// those anchors from the key paths.
    #[test]
    fn validate_resolves_key_sections_against_the_schema() {
        let good = INHERITED
            .replace("#reposbot", "#reposagent")
            .replace("- member [[3]]\n", "- member\n");
        let path = config_file("config-links-good", &good);
        let findings = validate_file(&path, "config", in_any, false)
            .expect("validate")
            .findings;
        assert_eq!(findings, 0, "every link now resolves");
        let _ = std::fs::remove_dir_all(path.parent().expect("temp dir"));
    }

    /// `repos.work` is the one key every instance config carries,
    /// so its absence is a finding for a side target.
    #[test]
    fn validate_reports_a_missing_required_key() {
        let body = "# c\n\n```toml\n[repos]\nagent = \".claude\"\n```\n";
        let path = config_file("config-required-missing", body);
        let findings = validate_file(&path, "work config", in_work_side, true)
            .expect("validate")
            .findings;
        assert_eq!(findings, 1, "repos.work is missing");
        let _ = std::fs::remove_dir_all(path.parent().expect("temp dir"));
    }

    /// A path target carries no side information and may be the
    /// user config, which has no `[repos]`, so the structural
    /// requirement does not apply to it.
    #[test]
    fn a_path_target_is_not_required_to_be_an_instance_config() {
        let body = "# c\n\n```toml\n[agent-session]\ncol-width = 68\n```\n";
        let path = config_file("config-required-path", body);
        let findings = validate_file(&path, "config file", in_any, false)
            .expect("validate")
            .findings;
        assert_eq!(findings, 0);
        let _ = std::fs::remove_dir_all(path.parent().expect("temp dir"));
    }

    #[test]
    fn validate_dual_workspace_clean() {
        let fx = Fixture::new("config-validate-clean");
        let params = ValidateConfigParams {
            target: ConfigTarget::Scope(Scope(vec![Side::Work, Side::Bot])),
        };
        let findings = validate(&params, Some(&fx.work))
            .expect("validate")
            .findings;
        assert_eq!(findings, 0);
    }

    #[test]
    fn validate_flags_unknown_key() {
        let fx = Fixture::new("config-validate-unknown");
        append(
            &fx.work.join(VC_CONFIG_MD),
            "\n[bogus-section]\nkey = \"v\"\n",
        );
        let params = ValidateConfigParams {
            target: ConfigTarget::Scope(Scope(vec![Side::Work, Side::Bot])),
        };
        let findings = validate(&params, Some(&fx.work))
            .expect("validate")
            .findings;
        assert_eq!(findings, 1);
    }

    /// A `[family]` key on the agent side is unknown there (work-side
    /// only), and a `str-list` key holding a scalar is a finding by
    /// shape, not just by name.
    #[test]
    fn validate_flags_family_on_agent_side_and_bad_list() {
        let fx = Fixture::new("config-validate-family");
        append(
            &fx.work.join(VC_CONFIG_MD),
            "\n[family]\nmember = \"x\"\n\n[validate]\nfast = \"cargo test\"\n",
        );
        append(&fx.bot.join(VC_CONFIG_MD), "\n[family]\nmember = \"x\"\n");
        let params = ValidateConfigParams {
            target: ConfigTarget::Scope(Scope(vec![Side::Work, Side::Bot])),
        };
        let findings = validate(&params, Some(&fx.work))
            .expect("validate")
            .findings;
        assert_eq!(findings, 2, "the bad list on work, the family key on agent");
    }

    #[test]
    fn validate_flags_incoherent_workspace_blocks() {
        // Diverge the bot side's [repos] registry (its bot entry
        // resolves to a different dir): the bot-side resolution
        // runs the dual-preflight coherence check, which must
        // surface as a finding (not a hard error) so the
        // work-side report still lands.
        let fx = Fixture::new("config-validate-incoherent");
        std::fs::create_dir_all(fx.work.join("other")).expect("mkdir other");
        std::fs::write(
            fx.bot.join(VC_CONFIG_FILE),
            "[repos]\nwork = \"..\"\nagent = \"../other\"\n",
        )
        .expect("rewrite bot config");
        let params = ValidateConfigParams {
            target: ConfigTarget::Scope(Scope(vec![Side::Work, Side::Bot])),
        };
        let findings = validate(&params, Some(&fx.work))
            .expect("validate")
            .findings;
        assert_eq!(findings, 1);
    }

    #[test]
    fn validate_legacy_schema_is_one_finding() {
        // A legacy schema short-circuits: one warn, one finding,
        // no unknown-key re-flagging of the legacy keys, no
        // repeated rejection via the bot-side resolution.
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("vc-x1-config-legacy-{ts}"));
        std::fs::create_dir_all(&root).expect("mkdir");
        std::fs::write(
            root.join(VC_CONFIG_FILE),
            "[workspace]\npath = \"/\"\nother-repo = \".claude\"\n",
        )
        .expect("write legacy config");
        let params = ValidateConfigParams {
            target: ConfigTarget::Scope(Scope(vec![Side::Work, Side::Bot])),
        };
        let findings = validate(&params, Some(&root)).expect("validate").findings;
        assert_eq!(findings, 1);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn validate_single_repo_skips_bot_side() {
        let fx = FixturePor::new("config-validate-por");
        let params = ValidateConfigParams {
            target: ConfigTarget::Scope(Scope(vec![Side::Work, Side::Bot])),
        };
        let findings = validate(&params, Some(&fx.work))
            .expect("validate")
            .findings;
        assert_eq!(findings, 0);
    }

    #[test]
    fn validate_outside_workspace_is_clean() {
        let params = ValidateConfigParams {
            target: ConfigTarget::Scope(Scope(vec![Side::Work, Side::Bot])),
        };
        let findings = validate(&params, None).expect("validate").findings;
        assert_eq!(findings, 0);
    }

    #[test]
    fn validate_explicit_path_target() {
        let fx = Fixture::new("config-validate-path");
        let path = fx.work.join(VC_CONFIG_MD);
        append(&path, "\n[bogus-section]\nkey = \"v\"\n");
        let params = ValidateConfigParams {
            target: ConfigTarget::Path(path),
        };
        let findings = validate(&params, Some(&fx.work))
            .expect("validate")
            .findings;
        assert_eq!(findings, 1);
    }
}

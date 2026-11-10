//! The `open` subcommand: open a cycle by making its bookmark first, on
//! the work `@`, before the opening's record, ladder, and bump are
//! written, so every edit of the cycle is on its branch.
//!
//! - The name is `--bookmark`'s, or `--title`'s slug, the anchor
//!   algorithm a heading of the title gets, which is the rules' name
//!   for a cycle's bookmark and the reason for a command over `jj
//!   bookmark create`. A title carrying a bookend's suffix is refused,
//!   as `commit --opening` refuses it.
//! - It refuses only what is dangerous: a name in use, locally or on
//!   `origin`, which it would move or shadow. `@` may hold edits, which
//!   become the opening's, and a `main` not where its `origin` was last
//!   fetched is a warning, a Land onto a target that moved being
//!   refused anyway.
//! - The bookmark goes on `@`, so it follows the change through every
//!   snapshot and the opening's commit, and `@` is not described: jj
//!   keeps an empty `@` that carries a bookmark when it is left.
//! - The agent-repo gets no bookmark, being on `main` alone. Nothing is
//!   pushed: the opening's push publishes the bookmark with it.

use std::path::{Path, PathBuf};

use clap::Args;
use log::{debug, info, warn};

use crate::common::PublishState;
use crate::context::Context;
use crate::jj;
use crate::options_flags::dry_run::DryRunFlag;
use crate::options_flags::scope::RepoScope;
use crate::subcommand::SubcommandRunner;

/// Open a cycle: make its bookmark on the work `@`.
#[derive(Args, Debug)]
#[group(id = "name", required = true, multiple = false, args = ["title", "bookmark"])]
pub struct OpenArgs {
    /// The bare cycle title, the bookmark its slug, the anchor a
    /// heading of the title gets
    #[arg(long, value_name = "STR")]
    pub title: Option<String>,

    /// The bookmark's name, in place of the title's slug
    #[arg(long, value_name = "NAME")]
    pub bookmark: Option<String>,

    /// Where: a path in the workspace, its root or either repo, or a
    /// plain repo [default: the current directory]
    #[arg(short = 'R', long, value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Print the bookmark it would make, and make none
    #[command(flatten)]
    pub dry_run: DryRunFlag,
}

/// Inputs to the open op, flat, owned, clap-free.
#[derive(Debug)]
pub struct OpenParams {
    /// The work repo, where the bookmark goes.
    pub work: PathBuf,
    /// The agent repo, checked for a `main` behind, when a pair.
    pub agent: Option<PathBuf>,
    /// The bookmark's name.
    pub bookmark: String,
    pub dry_run: bool,
}

impl TryFrom<&OpenArgs> for OpenParams {
    type Error = String;

    /// Resolve the workspace, a pair or a plain repo, and the name.
    fn try_from(a: &OpenArgs) -> Result<Self, String> {
        let (work, agent) = match crate::pair::resolve_pair(a.repo.as_deref()) {
            Ok(pair) => (pair.work, Some(pair.agent)),
            Err(_) => {
                let repos = crate::common::resolve_scoped_repos(&RepoScope::Por, a.repo.as_deref())
                    .map_err(|e| e.to_string())?;
                match repos.as_slice() {
                    [repo] => (repo.clone(), None),
                    other => return Err(format!("open: expected one repo, got {}", other.len())),
                }
            }
        };
        Ok(OpenParams {
            work,
            agent,
            bookmark: bookmark_name(a.bookmark.as_deref(), a.title.as_deref())?,
            dry_run: a.dry_run.value,
        })
    }
}

impl SubcommandRunner for OpenArgs {
    type Params = OpenParams;

    /// Delegate to the `TryFrom<&OpenArgs>` impl above.
    fn to_params(&self) -> Result<Self::Params, String> {
        OpenParams::try_from(self)
    }

    /// Run the `open` op.
    fn run(ctx: &mut Context, params: &Self::Params) -> Result<(), Box<dyn std::error::Error>> {
        open(ctx, params)
    }
}

/// The bookmark's name: `--bookmark`'s, else the bare title's slug,
/// refused when the title carries a bookend's suffix or has no letters
/// or digits to name it.
pub(crate) fn bookmark_name(named: Option<&str>, title: Option<&str>) -> Result<String, String> {
    if let Some(name) = named {
        return Ok(name.to_string());
    }
    let title = title.ok_or("open: --title or --bookmark names the bookmark")?;
    crate::commit::refuse_suffixed(title, "open --title")?;
    let slug = crate::validate_anchors::anchor_slug(title.trim_end());
    if slug.trim_matches(['-', '_']).is_empty() {
        return Err(format!(
            "open: the title '{title}' has no letters or digits to name the bookmark: name it, \
             --bookmark <NAME>"
        ));
    }
    Ok(slug)
}

/// Entry point: refuse a name in use, warn of a `main` that is not
/// where its `origin` was last fetched, and make the bookmark on the
/// work `@`.
pub fn open(ctx: &mut Context, params: &OpenParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("open: enter {params:?}");
    let work = params.work.as_path();
    let name = &params.bookmark;
    if jj::local_bookmark_exists(work, name)? {
        return Err(format!(
            "open: the bookmark '{name}' exists in {} already, and open moves none: name another, \
             --bookmark <NAME>",
            work.display()
        )
        .into());
    }
    if let Some(at) = jj::remote_bookmark_targets(work, "origin")?.get(name.as_str()) {
        return Err(format!(
            "open: the bookmark '{name}' is on origin at {at} already, so a new one would shadow \
             it: name another, --bookmark <NAME>"
        )
        .into());
    }
    for repo in std::iter::once(work).chain(params.agent.as_deref()) {
        warn_if_behind(repo)?;
    }
    let at = jj::chid_of(work, "@")?;
    if params.dry_run {
        info!("open: [dry-run] create '{name}' on the work @ {at}, locally");
        return Ok(());
    }
    info!("open: create '{name}' on the work @ {at}, locally");
    ctx.session(work)?.bookmark_set(name, "@")?;
    Ok(())
}

/// Warn when `repo`'s `main` is not where its `origin` was last
/// fetched, offline, since the opening syncs first by the rules and a
/// Land onto a target that moved is refused anyway.
fn warn_if_behind(repo: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !jj::local_bookmark_exists(repo, "main")? {
        return Ok(());
    }
    if let PublishState::Mismatch { local, remote } =
        crate::common::bookmark_publish_state(repo, "main")?
    {
        warn!(
            "open: 'main' in {} is at {} and origin's at {} as last fetched: `vc-x1 sync -s b` \
             brings it up to date",
            repo.display(),
            short(&local),
            short(&remote)
        );
    }
    Ok(())
}

/// A commit id cut to the length the ids are shown at.
fn short(id: &str) -> String {
    id.chars()
        .take(crate::desc_helpers::DEFAULT_ID_LEN)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{Fixture, jj_ok, test_ctx};
    use crate::{Cli, Commands};
    use clap::Parser;
    use std::fs;

    fn parse(args: &[&str]) -> Result<OpenArgs, String> {
        match Cli::try_parse_from(args) {
            Ok(Cli {
                command: Some(Commands::Open(a)),
                ..
            }) => Ok(a),
            Ok(_) => panic!("expected Open"),
            Err(e) => Err(e.to_string()),
        }
    }

    fn params(fx: &Fixture, args: &[&str]) -> OpenParams {
        let mut all = vec!["vc-x1", "open"];
        all.extend_from_slice(args);
        let mut a = parse(&all).expect("parses");
        a.repo = Some(fx.work.clone());
        OpenParams::try_from(&a).expect("params")
    }

    /// The bookmark at `name`, or empty when there is none.
    fn bookmark_at(repo: &Path, name: &str) -> String {
        jj_ok(
            repo,
            &[
                "log",
                "-r",
                &format!("present(bookmarks(exact:\"{name}\"))"),
                "--no-graph",
                "-T",
                "change_id.short(12)",
            ],
        )
    }

    /// One of `--title` and `--bookmark` is required, not both, and the
    /// title's slug names the bookmark, a bookend's suffix refused.
    #[test]
    fn the_name_is_the_titles_slug_or_named() {
        let err = parse(&["vc-x1", "open"]).unwrap_err();
        assert!(
            err.contains("--title") && err.contains("--bookmark"),
            "{err}"
        );
        let err = parse(&["vc-x1", "open", "--title", "t", "--bookmark", "b"]).unwrap_err();
        assert!(err.contains("cannot be used with"), "{err}");
        assert_eq!(
            bookmark_name(None, Some("feat: no trunk, every target is named")).unwrap(),
            "feat-no-trunk-every-target-is-named"
        );
        assert_eq!(bookmark_name(Some("mine"), None).unwrap(), "mine");
        let err = bookmark_name(None, Some("feat: x opening")).unwrap_err();
        assert!(err.contains("opening"), "{err}");
        let err = bookmark_name(None, Some(": ,")).unwrap_err();
        assert!(err.contains("no letters or digits"), "{err}");
    }

    /// The bookmark goes on the work `@`, edits in it or not, and follows
    /// the change through the commit, the agent repo given none. A dry
    /// run makes none.
    #[test]
    fn the_bookmark_goes_on_the_work_at() {
        let fx = Fixture::new("open-at");
        fs::write(fx.work.join("edit.txt"), "an edit first\n").expect("write");
        open(
            &mut test_ctx(),
            &params(&fx, &["--title", "feat: a cycle", "--dry-run"]),
        )
        .expect("a dry run");
        assert_eq!(
            bookmark_at(&fx.work, "feat-a-cycle"),
            "",
            "a dry run makes none"
        );

        let at = jj_ok(
            &fx.work,
            &["log", "-r", "@", "--no-graph", "-T", "change_id.short(12)"],
        );
        open(&mut test_ctx(), &params(&fx, &["--title", "feat: a cycle"])).expect("opened");
        assert_eq!(bookmark_at(&fx.work, "feat-a-cycle"), at, "on the work @");
        assert_eq!(
            bookmark_at(&fx.bot, "feat-a-cycle"),
            "",
            "the agent repo gets none"
        );

        jj_ok(&fx.work, &["commit", "-m", "feat: a cycle opening"]);
        assert_eq!(
            bookmark_at(&fx.work, "feat-a-cycle"),
            at,
            "it follows the change"
        );
    }

    /// A name in use, locally or on origin, is refused, nothing made.
    #[test]
    fn a_name_in_use_is_refused() {
        let fx = Fixture::new("open-in-use");
        let err = open(&mut test_ctx(), &params(&fx, &["--bookmark", "main"]))
            .unwrap_err()
            .to_string();
        assert!(err.contains("exists") && err.contains("already"), "{err}");

        jj_ok(&fx.work, &["bookmark", "create", "taken", "-r", "main"]);
        jj_ok(&fx.work, &["git", "push", "--bookmark", "taken"]);
        jj_ok(&fx.work, &["bookmark", "delete", "taken"]);
        let err = open(&mut test_ctx(), &params(&fx, &["--bookmark", "taken"]))
            .unwrap_err()
            .to_string();
        assert!(err.contains("on origin"), "{err}");
        assert_eq!(bookmark_at(&fx.work, "taken"), "", "nothing made");
    }
}

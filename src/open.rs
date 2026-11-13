//! Opening a cycle's bookmark, `cycle --open`'s first act: the bookmark
//! is made on the work `@`, before the opening's record, ladder, and
//! bump are written, so every edit of the cycle is on its branch.
//!
//! - The name is `--bookmark`'s, or `--title`'s slug, the anchor
//!   algorithm a heading of the title gets, which is the rules' name
//!   for a cycle's bookmark and the reason for a command over `jj
//!   bookmark create`. A title carrying a bookend's suffix is refused.
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

use log::{debug, info, warn};

use crate::common::PublishState;
use crate::context::Context;
use crate::jj;

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

/// The bookmark's name: `--bookmark`'s, else the bare title's slug,
/// refused when the title carries a bookend's suffix or has no letters
/// or digits to name it.
pub(crate) fn bookmark_name(named: Option<&str>, title: Option<&str>) -> Result<String, String> {
    if let Some(name) = named {
        return Ok(name.to_string());
    }
    let title = title.ok_or("cycle --open: --title or --bookmark names the bookmark")?;
    crate::commit::refuse_suffixed(title, "cycle --open --title")
        .map_err(|e| e.replacen("commit ", "", 1))?;
    let slug = crate::validate_anchors::anchor_slug(title.trim_end());
    if slug.trim_matches(['-', '_']).is_empty() {
        return Err(format!(
            "cycle --open: the title '{title}' has no letters or digits to name the bookmark: name it, \
             --bookmark <NAME>"
        ));
    }
    Ok(slug)
}

/// Entry point: refuse a name in use, warn of a `main` that is not
/// where its `origin` was last fetched, and make the bookmark on the
/// work `@`.
pub fn open(ctx: &mut Context, params: &OpenParams) -> Result<(), Box<dyn std::error::Error>> {
    debug!("cycle --open: enter {params:?}");
    let work = params.work.as_path();
    let name = &params.bookmark;
    if jj::local_bookmark_exists(work, name)? {
        return Err(format!(
            "cycle --open: the bookmark '{name}' exists in {} already, and open moves none: name another, \
             --bookmark <NAME>",
            work.display()
        )
        .into());
    }
    if let Some(at) = jj::remote_bookmark_targets(work, "origin")?.get(name.as_str()) {
        return Err(format!(
            "cycle --open: the bookmark '{name}' is on origin at {at} already, so a new one would shadow \
             it: name another, --bookmark <NAME>"
        )
        .into());
    }
    for repo in std::iter::once(work).chain(params.agent.as_deref()) {
        warn_if_behind(repo)?;
    }
    let at = jj::chid_of(work, "@")?;
    if params.dry_run {
        info!("cycle --open: [dry-run] create '{name}' on the work @ {at}, locally");
        return Ok(());
    }
    info!("cycle --open: create '{name}' on the work @ {at}, locally");
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
            "cycle --open: 'main' in {} is at {} and origin's at {} as last fetched: `vc-x1 sync -s b` \
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
    use std::fs;

    /// The params of an open in the fixture's workspace, by a title or
    /// a bookmark's name.
    fn params(
        fx: &Fixture,
        title: Option<&str>,
        bookmark: Option<&str>,
        dry_run: bool,
    ) -> OpenParams {
        OpenParams {
            work: fx.work.clone(),
            agent: Some(fx.bot.clone()),
            bookmark: bookmark_name(bookmark, title).expect("a name"),
            dry_run,
        }
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

    /// The title's slug names the bookmark, or the name given does, a
    /// bookend's suffix refused.
    #[test]
    fn the_name_is_the_titles_slug_or_named() {
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
            &params(&fx, Some("feat: a cycle"), None, true),
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
        open(
            &mut test_ctx(),
            &params(&fx, Some("feat: a cycle"), None, false),
        )
        .expect("opened");
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
        let err = open(&mut test_ctx(), &params(&fx, None, Some("main"), false))
            .unwrap_err()
            .to_string();
        assert!(err.contains("exists") && err.contains("already"), "{err}");

        jj_ok(&fx.work, &["bookmark", "create", "taken", "-r", "main"]);
        jj_ok(&fx.work, &["git", "push", "--bookmark", "taken"]);
        jj_ok(&fx.work, &["bookmark", "delete", "taken"]);
        let err = open(&mut test_ctx(), &params(&fx, None, Some("taken"), false))
            .unwrap_err()
            .to_string();
        assert!(err.contains("on origin"), "{err}");
        assert_eq!(bookmark_at(&fx.work, "taken"), "", "nothing made");
    }
}

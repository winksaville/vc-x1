//! `RepoSession`: an in-process working session with one repo.
//!
//! The plumbing jj's own CLI wraps around every mutation
//! (`WorkspaceCommandHelper` in jj-cli), reduced to what the facade's
//! verbs need: user-config-backed settings, the working-copy snapshot
//! cycle, and transaction finish. The reference implementation is jj
//! 0.43's `cli_util.rs`; deviations are documented on each function.
//! The name is backend-neutral on purpose: what a session holds and
//! its open -> mutate -> finish lifecycle would survive a non-jj
//! backend.
//!
//! - `RepoSession::open`: settings + workspace + repo-at-head, colocation
//!   detected.
//! - `RepoSession::snapshot`: the CLI's `maybe_snapshot`: git head/refs
//!   import around a working-copy snapshot, under the same lock file
//!   the CLI takes.
//! - `RepoSession::finish_tx`: the CLI's `finish_transaction`: git HEAD
//!   reset + ref export, op-store commit, working-copy update.
//! - The publish-path verbs (`commit`, `describe`, `bookmark_set`,
//!   `git_push_bookmark`, `git_fetch`): each snapshots, mutates in
//!   one transaction, and finishes (one op per verb, so the op log
//!   keeps the shape push re-run and sync revert rely on).
//!
//! A session lives as long as its owner wants: `Context` caches one
//! per repo for a whole invocation, the facade's one-shot wrappers
//! open one per call. Either way each verb starts with `snapshot`,
//! which reloads at the op-store head, so a cached session never
//! acts on a stale view.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use jj_lib::commit::Commit;
use jj_lib::config::{ConfigLayer, ConfigSource, StackedConfig};
use jj_lib::fileset::{self, FilesetDiagnostics, FilesetParseContext};
use jj_lib::git::{self, GitImportOptions, GitSettings};
use jj_lib::gitignore::GitIgnoreFile;
use jj_lib::lock::FileLock;
use jj_lib::matchers::NothingMatcher;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::{ReadonlyRepo, Repo, StoreFactories};
use jj_lib::repo_path::{RepoPath, RepoPathUiConverter};
use jj_lib::settings::{HumanByteSize, UserSettings};
use jj_lib::transaction::Transaction;
use jj_lib::working_copy::{SnapshotOptions, WorkingCopyFreshness};
use jj_lib::workspace::{Workspace, default_working_copy_factories};
use log::debug;
use pollster::FutureExt;

/// Crate-standard boxed-error result, aliased locally for brevity.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// One repo opened for mutation: workspace, repo at head, settings,
/// and whether the workspace is colocated with git.
pub struct RepoSession {
    /// The loaded workspace (owns the working copy).
    pub workspace: Workspace,
    /// The repo, loaded at (or advanced past) the op-store head.
    pub repo: Arc<ReadonlyRepo>,
    /// Settings from the user's real jj config (see `stacked_config`).
    pub settings: UserSettings,
    /// True when `.git` and the working copy are shared with git, so
    /// mutations must import from and export to the git side.
    pub colocated: bool,
}

/// Load the user's jj configuration the way the jj CLI does.
///
/// Layers, lowest to highest: jj-lib defaults, `/etc/jj` (unix),
/// `$JJ_CONFIG` (overrides system and user paths) or the user files
/// (`~/.jjconfig.toml`, `<config dir>/jj/config.toml` + `conf.d`),
/// the repo config (`.jj/repo/config.toml`), then `JJ_USER` /
/// `JJ_EMAIL` overrides. Deviation from the CLI: no command-line
/// layer and no `EnvBase` layer (debug / color / pager knobs that
/// only affect CLI presentation).
fn stacked_config(repo_config_path: Option<&Path>) -> Result<StackedConfig> {
    let mut config = StackedConfig::with_defaults();
    // Keys the engine reads whose defaults ship with the jj CLI's
    // config files rather than jj-lib's (values match jj 0.43).
    config.add_layer(ConfigLayer::parse(
        ConfigSource::Default,
        r#"
        [snapshot]
        auto-track = "all()"
        max-new-file-size = "1MiB"
        [git]
        auto-local-bookmark = false
        "#,
    )?);
    let jj_config = std::env::var("JJ_CONFIG").ok();

    if jj_config.is_none() && cfg!(unix) {
        load_file_if_exists(&mut config, ConfigSource::System, "/etc/jj/config.toml")?;
        load_dir_if_exists(&mut config, ConfigSource::System, "/etc/jj/conf.d")?;
    }

    if let Some(paths) = &jj_config {
        for path in std::env::split_paths(paths).filter(|p| !p.as_os_str().is_empty()) {
            if path.is_dir() {
                config.load_dir(ConfigSource::User, path)?;
            } else {
                config.load_file(ConfigSource::User, path)?;
            }
        }
    } else {
        let home = std::env::var_os("HOME").map(PathBuf::from);
        let config_dir = match std::env::var_os("XDG_CONFIG_HOME") {
            Some(x) if !x.is_empty() => Some(PathBuf::from(x)),
            _ => home.as_ref().map(|h| h.join(".config")),
        };
        if let Some(home) = &home {
            load_file_if_exists(&mut config, ConfigSource::User, home.join(".jjconfig.toml"))?;
        }
        if let Some(dir) = &config_dir {
            load_file_if_exists(&mut config, ConfigSource::User, dir.join("jj/config.toml"))?;
            load_dir_if_exists(&mut config, ConfigSource::User, dir.join("jj/conf.d"))?;
        }
    }

    if let Some(repo_path) = repo_config_path {
        load_file_if_exists(
            &mut config,
            ConfigSource::Repo,
            repo_path.join("config.toml"),
        )?;
    }

    let mut env = ConfigLayer::empty(ConfigSource::EnvOverrides);
    if let Ok(value) = std::env::var("JJ_USER") {
        env.set_value("user.name", value)?;
    }
    if let Ok(value) = std::env::var("JJ_EMAIL") {
        env.set_value("user.email", value)?;
    }
    if !env.is_empty() {
        config.add_layer(env);
    }
    Ok(config)
}

/// Add one config file as a layer when it exists.
fn load_file_if_exists(
    config: &mut StackedConfig,
    source: ConfigSource,
    path: impl Into<PathBuf>,
) -> Result<()> {
    let path = path.into();
    if path.is_file() {
        config.load_file(source, path)?;
    }
    Ok(())
}

/// Add one config directory as a layer set when it exists.
fn load_dir_if_exists(
    config: &mut StackedConfig,
    source: ConfigSource,
    path: impl Into<PathBuf>,
) -> Result<()> {
    let path = path.into();
    if path.is_dir() {
        config.load_dir(source, path)?;
    }
    Ok(())
}

/// True when the workspace shares its working copy with a colocated
/// git repo (jj-cli's `is_colocated_git_workspace`, minus the
/// canonicalized `.git`-symlink fallback: vc-x1 lays repos out with
/// a real `.git` directory).
fn is_colocated(workspace: &Workspace, repo: &Arc<ReadonlyRepo>) -> bool {
    let Ok(backend) = git::get_git_backend(repo.store()) else {
        return false;
    };
    let Some(git_workdir) = backend.git_workdir() else {
        return false;
    };
    if git_workdir == workspace.workspace_root() {
        return true;
    }
    let Ok(dot_git) = workspace.workspace_root().join(".git").canonicalize() else {
        return false;
    };
    git_workdir.canonicalize().ok().as_deref() == dot_git.parent()
}

impl RepoSession {
    /// Open `path` for mutation: user settings, workspace, repo at
    /// the op-store head, colocation flag.
    pub fn open(path: &Path) -> Result<Self> {
        // Two-phase settings: the repo config layer lives under
        // `.jj/repo`, whose location the loaded workspace knows.
        let settings = UserSettings::from_config(stacked_config(None)?)?;
        let store_factories = StoreFactories::default();
        let working_copy_factories = default_working_copy_factories();
        let workspace =
            Workspace::load(&settings, path, &store_factories, &working_copy_factories)?;
        let settings = UserSettings::from_config(stacked_config(Some(workspace.repo_path()))?)?;
        let workspace =
            Workspace::load(&settings, path, &store_factories, &working_copy_factories)?;
        let repo = workspace.repo_loader().load_at_head().block_on()?;
        let colocated = is_colocated(&workspace, &repo);
        Ok(RepoSession {
            workspace,
            repo,
            settings,
            colocated,
        })
    }

    /// Take the git import/export lock the CLI takes around its
    /// import/snapshot/export cycle; `None` when not colocated.
    fn lock_git(&self) -> Result<Option<FileLock>> {
        if !self.colocated {
            return Ok(None);
        }
        let path = self.workspace.repo_path().join("git_import_export.lock");
        Ok(Some(FileLock::lock(path)?))
    }

    /// The CLI's `maybe_snapshot`: import git HEAD, snapshot the
    /// working copy, import git refs, each its own operation.
    pub fn snapshot(&mut self) -> Result<()> {
        let _lock = self.lock_git()?;
        // Reload at head (under the lock when colocated): sessions
        // outlive single verbs in a `Context`, and another process
        // or a spawned `jj` may have committed operations since the
        // open or the previous verb.
        self.repo = self.workspace.repo_loader().load_at_head().block_on()?;
        if self.colocated {
            self.import_git_head()?;
        }
        self.snapshot_working_copy()?;
        if self.colocated {
            self.import_git_refs()?;
        }
        Ok(())
    }

    /// Import a moved git HEAD (someone ran plain git here): check
    /// out the new head and reset the working-copy state to it.
    fn import_git_head(&mut self) -> Result<()> {
        let mut tx = self.repo.start_transaction();
        git::import_head(tx.repo_mut()).block_on()?;
        if !tx.repo().has_changes() {
            return Ok(());
        }
        debug!("git HEAD moved; importing");
        let new_git_head = tx.repo().view().git_head().clone();
        if let Some(new_git_head_id) = new_git_head.as_normal() {
            let name = self.workspace.workspace_name().to_owned();
            let head_commit = tx.repo().store().get_commit(new_git_head_id)?;
            let wc_commit = tx.repo_mut().check_out(name, &head_commit).block_on()?;
            let mut locked_ws = self.workspace.start_working_copy_mutation().block_on()?;
            // The git command that moved HEAD updated the files too,
            // so reset the state without touching the working copy.
            locked_ws.locked_wc().reset(&wc_commit).block_on()?;
            tx.repo_mut().rebase_descendants().block_on()?;
            self.repo = tx.commit("import git head").block_on()?;
            locked_ws.finish(self.repo.op_id().clone()).block_on()?;
        } else {
            tx.repo_mut().rebase_descendants().block_on()?;
            self.repo = tx.commit("import git head").block_on()?;
        }
        Ok(())
    }

    /// Snapshot the working copy into the wc commit if the tree
    /// changed (the CLI's `snapshot_working_copy`).
    ///
    /// Deviation: no immutable-wc-commit branch. vc-x1 never edits
    /// an immutable commit, and evaluating `immutable_heads()` needs
    /// the CLI's revset-alias machinery.
    fn snapshot_working_copy(&mut self) -> Result<()> {
        let name = self.workspace.workspace_name().to_owned();
        let repo = self.repo.clone();

        let auto_track_pattern = self.settings.get_string("snapshot.auto-track")?;
        let mut diagnostics = FilesetDiagnostics::new();
        let root = self.workspace.workspace_root().to_path_buf();
        let path_converter = RepoPathUiConverter::Fs {
            cwd: root.clone(),
            base: root,
        };
        let fileset_aliases = jj_lib::fileset::FilesetAliasesMap::new();
        let fileset_ctx = FilesetParseContext {
            aliases_map: &fileset_aliases,
            path_converter: &path_converter,
        };
        let auto_track =
            fileset::parse(&mut diagnostics, &auto_track_pattern, &fileset_ctx)?.to_matcher();
        let HumanByteSize(mut max_new_file_size) = self
            .settings
            .get_value_with("snapshot.max-new-file-size", TryInto::try_into)?;
        if max_new_file_size == 0 {
            max_new_file_size = u64::MAX;
        }
        let options = SnapshotOptions {
            base_ignores: self.base_ignores()?,
            progress: None,
            start_tracking_matcher: auto_track.as_ref(),
            force_tracking_matcher: &NothingMatcher,
            max_new_file_size,
        };

        let mut locked_ws = self.workspace.start_working_copy_mutation().block_on()?;
        let Some(wc_commit_id) = repo.view().get_wc_commit_id(&name).cloned() else {
            return Ok(()); // Workspace deleted from the repo view.
        };
        let mut wc_commit = repo.store().get_commit(&wc_commit_id)?;
        match WorkingCopyFreshness::check_stale(locked_ws.locked_wc(), &wc_commit, &repo)
            .block_on()?
        {
            WorkingCopyFreshness::Fresh => {}
            WorkingCopyFreshness::Updated(wc_operation) => {
                self.repo = repo.reload_at(&wc_operation).block_on()?;
                let Some(id) = self.repo.view().get_wc_commit_id(&name).cloned() else {
                    return Ok(());
                };
                wc_commit = self.repo.store().get_commit(&id)?;
            }
            stale => {
                return Err(format!(
                    "the working copy is stale ({stale:?}); run `jj workspace update-stale` \
                     in {}",
                    self.workspace.workspace_root().display()
                )
                .into());
            }
        }

        let (new_tree, _stats) = locked_ws.locked_wc().snapshot(&options).block_on()?;
        if new_tree.tree_ids_and_labels() != wc_commit.tree().tree_ids_and_labels() {
            debug!("working-copy tree changed; committing snapshot operation");
            let mut tx = self.repo.start_transaction();
            tx.set_is_snapshot(true);
            let mut_repo = tx.repo_mut();
            let new_wc_commit = mut_repo
                .rewrite_commit(&wc_commit)
                .set_tree(new_tree.clone())
                .write()
                .block_on()?;
            mut_repo.set_wc_commit(name.clone(), new_wc_commit.id().clone())?;
            mut_repo.rebase_descendants().block_on()?;
            if self.colocated {
                git::update_intent_to_add(
                    mut_repo.base_repo().as_ref(),
                    &wc_commit.tree(),
                    &new_wc_commit.tree(),
                )
                .block_on()?;
                git::export_refs(mut_repo)?;
            }
            self.repo = tx.commit("snapshot working copy").block_on()?;
        }
        locked_ws.finish(self.repo.op_id().clone()).block_on()?;
        Ok(())
    }

    /// Import git refs (fetch results, refs moved by plain git) and
    /// rebase descendants of rewritten commits.
    fn import_git_refs(&mut self) -> Result<()> {
        let import_options = self.import_options()?;
        let mut tx = self.repo.start_transaction();
        git::import_refs(tx.repo_mut(), &import_options).block_on()?;
        if !tx.repo().has_changes() {
            return Ok(());
        }
        debug!("git refs changed; importing");
        tx.repo_mut().rebase_descendants().block_on()?;
        self.finish_tx(tx, "import git refs")
    }

    /// Build `GitImportOptions` from settings.
    ///
    /// Deviation from the CLI: the per-remote auto-track map is
    /// driven by `git.auto-local-bookmark` alone, not the per-remote
    /// `remotes.<name>.auto-track-bookmarks` patterns.
    pub fn import_options(&self) -> Result<GitImportOptions> {
        let git_settings = GitSettings::from_settings(&self.settings)?;
        let auto_local_bookmark = self.settings.get_bool("git.auto-local-bookmark")?;
        let mut remote_auto_track_bookmarks = HashMap::new();
        if auto_local_bookmark {
            for remote in git::get_all_remote_names(self.repo.store())? {
                remote_auto_track_bookmarks.insert(
                    remote,
                    jj_lib::str_util::StringExpression::all().to_matcher(),
                );
            }
        }
        Ok(GitImportOptions {
            abandon_unreachable_commits: git_settings.abandon_unreachable_commits,
            record_synthetic_predecessors: git_settings.record_synthetic_predecessors,
            remote_auto_track_bookmarks,
        })
    }

    /// The gitignore chain snapshots respect: the user's global
    /// excludes file, then the git dir's `info/exclude`.
    fn base_ignores(&self) -> Result<Arc<GitIgnoreFile>> {
        let mut ignores = GitIgnoreFile::empty();
        let Ok(backend) = git::get_git_backend(self.repo.store()) else {
            return Ok(ignores);
        };
        let git_repo = backend.git_repo();
        let excludes = {
            let config = git_repo.config_snapshot();
            match config.string("core.excludesFile") {
                Some(value) => str::from_utf8(&value)
                    .ok()
                    .map(jj_lib::file_util::expand_home_path)
                    .map(|p| self.workspace.workspace_root().join(p)),
                None => {
                    let config_dir = match std::env::var_os("XDG_CONFIG_HOME") {
                        Some(x) if !x.is_empty() => Some(PathBuf::from(x)),
                        _ => std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")),
                    };
                    config_dir.map(|d| d.join("git").join("ignore"))
                }
            }
        };
        if let Some(path) = excludes {
            ignores = ignores.chain_with_file(RepoPath::root(), path)?;
        }
        ignores = ignores.chain_with_file(
            RepoPath::root(),
            backend.git_repo_path().join("info/exclude"),
        )?;
        Ok(ignores)
    }

    /// Start a transaction stamped with this workspace's name.
    pub fn start_tx(&self) -> Transaction {
        let mut tx = self.repo.start_transaction();
        tx.set_workspace_name(self.workspace.workspace_name());
        tx
    }

    /// The CLI's `finish_transaction`: reset git HEAD and export
    /// refs (colocated), commit the operation, update the working
    /// copy to the possibly-rewritten wc commit.
    pub fn finish_tx(&mut self, mut tx: Transaction, desc: &str) -> Result<()> {
        if !tx.repo().has_changes() {
            debug!("{desc}: nothing changed");
            return Ok(());
        }
        tx.repo_mut().rebase_descendants().block_on()?;
        let name = self.workspace.workspace_name().to_owned();
        let old_wc_commit = tx
            .base_repo()
            .view()
            .get_wc_commit_id(&name)
            .map(|id| tx.base_repo().store().get_commit(id))
            .transpose()?;
        let new_wc_commit_id = tx.repo().view().get_wc_commit_id(&name).cloned();

        if self.colocated {
            if let Some(id) = &new_wc_commit_id {
                let wc_commit = tx.repo().store().get_commit(id)?;
                git::reset_head(tx.repo_mut(), &wc_commit).block_on()?;
            }
            git::export_refs(tx.repo_mut())?;
        }

        self.repo = tx.commit(desc).block_on()?;

        if let Some(id) = &new_wc_commit_id {
            let new_wc_commit = self.repo.store().get_commit(id)?;
            let old_tree = old_wc_commit.as_ref().map(|c| c.tree());
            self.workspace
                .check_out(self.repo.op_id().clone(), old_tree.as_ref(), &new_wc_commit)
                .block_on()?;
        }
        Ok(())
    }

    /// The working-copy commit of this workspace.
    pub fn wc_commit(&self) -> Result<Commit> {
        let name = self.workspace.workspace_name();
        let id = self
            .repo
            .view()
            .get_wc_commit_id(name)
            .ok_or_else(|| format!("workspace {name:?} has no working-copy commit"))?;
        Ok(self.repo.store().get_commit(id)?)
    }

    /// Resolve `rev` against this session's view to exactly one
    /// commit.
    fn one_commit(&self, rev: &str) -> Result<Commit> {
        let ids = crate::common::resolve_revset(&self.workspace, &self.repo, rev)?;
        match ids.as_slice() {
            [id] => Ok(self.repo.store().get_commit(id)?),
            other => Err(format!(
                "expected exactly one commit for {rev:?}, got {}",
                other.len()
            )
            .into()),
        }
    }

    /// Snapshot the working copy, then update the wc commit's
    /// description and start a new empty change on top (`jj commit`).
    pub fn commit(&mut self, desc: &str) -> Result<()> {
        self.snapshot()?;
        let wc_commit = self.wc_commit()?;
        let name = self.workspace.workspace_name().to_owned();
        let mut tx = self.start_tx();
        let new_commit = tx
            .repo_mut()
            .rewrite_commit(&wc_commit)
            .set_description(complete_newline(desc))
            .write()
            .block_on()?;
        let new_wc_commit = tx
            .repo_mut()
            .new_commit(vec![new_commit.id().clone()], wc_commit.tree())
            .write()
            .block_on()?;
        tx.repo_mut().edit(name, &new_wc_commit).block_on()?;
        self.finish_tx(tx, &format!("commit {}", wc_commit.id().hex()))
    }

    /// Snapshot, then rewrite `rev`'s description and rebase its
    /// descendants (`jj describe -r <rev> -m <desc>`).
    pub fn describe(&mut self, rev: &str, desc: &str) -> Result<()> {
        self.snapshot()?;
        let commit = self.one_commit(rev)?;
        let desc = complete_newline(desc);
        if commit.description() == desc {
            return Ok(());
        }
        let mut tx = self.start_tx();
        tx.repo_mut()
            .rewrite_commit(&commit)
            .set_description(desc)
            .write()
            .block_on()?;
        tx.repo_mut().rebase_descendants().block_on()?;
        self.finish_tx(tx, &format!("describe commit {}", commit.id().hex()))
    }

    /// Snapshot, then point local bookmark `name` at `rev`
    /// (`jj bookmark set <name> -r <rev>`).
    pub fn bookmark_set(&mut self, name: &str, rev: &str) -> Result<()> {
        use jj_lib::op_store::RefTarget;
        self.snapshot()?;
        let commit = self.one_commit(rev)?;
        let mut tx = self.start_tx();
        tx.repo_mut()
            .set_local_bookmark_target(name.as_ref(), RefTarget::normal(commit.id().clone()));
        self.finish_tx(
            tx,
            &format!("point bookmark {} to commit {}", name, commit.id().hex()),
        )
    }

    /// Snapshot, then push local bookmark `name` to `origin`
    /// (`jj git push --bookmark <name>`), force-with-lease against
    /// the last-seen remote position.
    ///
    /// Deviation from the CLI: no conflicted / no-description /
    /// private preflights on the pushed commits; the push callers
    /// validate the commit they publish before calling.
    pub fn git_push_bookmark(&mut self, name: &str) -> Result<()> {
        use jj_lib::git::{GitPushOptions, GitPushRefTargets, GitSettings};
        use jj_lib::merge::Diff;
        use jj_lib::ref_name::{RefNameBuf, RemoteName};

        self.snapshot()?;
        let remote = RemoteName::new("origin");
        let ref_name: RefNameBuf = name.into();
        let local = self
            .repo
            .view()
            .get_local_bookmark(&ref_name)
            .as_normal()
            .cloned()
            .ok_or_else(|| format!("bookmark '{name}' is absent or conflicted"))?;
        let remote_ref = self
            .repo
            .view()
            .get_remote_bookmark(ref_name.to_remote_symbol(remote));
        let before = remote_ref.target.as_normal().cloned();
        if before.as_ref() == Some(&local) {
            return Ok(()); // Already in sync; matches `jj git push` "Nothing changed".
        }
        let ref_updates = GitPushRefTargets {
            bookmarks: vec![(
                ref_name.clone(),
                Diff {
                    before,
                    after: Some(local),
                },
            )],
            tags: vec![],
        };
        let git_settings = GitSettings::from_settings(&self.settings)?;
        let mut tx = self.start_tx();
        let stats = git::push_refs(
            tx.repo_mut(),
            git_settings.to_subprocess_options(),
            remote,
            &ref_updates,
            &mut DebugCallback,
            &GitPushOptions::default(),
        )?;
        if !stats.all_ok() {
            return Err(format!(
                "git push of bookmark '{name}' failed: rejected {:?}, remote rejected {:?}",
                stats.rejected, stats.remote_rejected
            )
            .into());
        }
        self.finish_tx(tx, &format!("push bookmark {name} to git remote origin"))
    }

    /// Snapshot, then fetch all branches from `remote` and import
    /// the results (`jj git fetch --remote <remote>`). Returns one
    /// line per changed remote bookmark, empty when the fetch
    /// changed nothing.
    ///
    /// Deviation from the CLI: the branch expression is always "all
    /// branches" rather than the remote's configured fetch refspecs,
    /// which is what a standard `origin` resolves to anyway.
    pub fn git_fetch(&mut self, remote: &str) -> Result<Vec<String>> {
        use jj_lib::git::{GitFetch, GitFetchRefExpression, GitSettings, expand_fetch_refspecs};
        use jj_lib::ref_name::RemoteName;
        use jj_lib::str_util::StringExpression;

        self.snapshot()?;
        let remote = RemoteName::new(remote);
        let expanded = expand_fetch_refspecs(
            remote,
            GitFetchRefExpression {
                bookmark: StringExpression::all(),
                tag: StringExpression::none(),
            },
        )?;
        let git_settings = GitSettings::from_settings(&self.settings)?;
        let import_options = self.import_options()?;
        let mut tx = self.start_tx();
        let mut fetch = GitFetch::new(
            tx.repo_mut(),
            git_settings.to_subprocess_options(),
            &import_options,
        )?;
        fetch.fetch(remote, expanded, &mut DebugCallback, None, None)?;
        let stats = fetch.import_refs().block_on()?;
        drop(fetch);
        let lines: Vec<String> = stats
            .changed_remote_bookmarks
            .iter()
            .map(|(symbol, _)| format!("bookmark {symbol} updated"))
            .collect();
        if !tx.repo().has_changes() {
            return Ok(lines);
        }
        tx.repo_mut().rebase_descendants().block_on()?;
        self.finish_tx(tx, &format!("fetch from git remote {}", remote.as_str()))?;
        Ok(lines)
    }
}

/// `desc` with the trailing newline jj normalizes descriptions to.
fn complete_newline(desc: &str) -> String {
    if desc.is_empty() || desc.ends_with('\n') {
        desc.to_string()
    } else {
        format!("{desc}\n")
    }
}

/// Callback for jj-lib's git subprocess: no progress requests,
/// sideband chatter forwarded to `debug!` (the same place `run`
/// put spawned jj's stderr).
struct DebugCallback;

impl git::GitSubprocessCallback for DebugCallback {
    /// Progress reporting is not requested.
    fn needs_progress(&self) -> bool {
        false
    }

    /// Ignore progress events (never requested).
    fn progress(&mut self, _progress: &git::GitProgress) -> std::io::Result<()> {
        Ok(())
    }

    /// Log local git messages at debug, like spawned-jj stderr was.
    fn local_sideband(
        &mut self,
        message: &[u8],
        _term: Option<git::GitSidebandLineTerminator>,
    ) -> std::io::Result<()> {
        debug!("git: {}", String::from_utf8_lossy(message));
        Ok(())
    }

    /// Log remote sideband messages at debug.
    fn remote_sideband(
        &mut self,
        message: &[u8],
        _term: Option<git::GitSidebandLineTerminator>,
    ) -> std::io::Result<()> {
        debug!("remote: {}", String::from_utf8_lossy(message));
        Ok(())
    }
}

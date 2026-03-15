use std::path::Path;
use std::sync::Arc;

use jj_lib::config::StackedConfig;
use jj_lib::repo::{ReadonlyRepo, StoreFactories};
use jj_lib::settings::UserSettings;
use jj_lib::workspace::{Workspace, default_working_copy_factories};
use pollster::FutureExt;

pub fn load_repo(
    path: &Path,
) -> Result<(Workspace, Arc<ReadonlyRepo>), Box<dyn std::error::Error>> {
    let config = StackedConfig::with_defaults();
    let settings = UserSettings::from_config(config)?;
    let store_factories = StoreFactories::default();
    let working_copy_factories = default_working_copy_factories();

    let workspace = Workspace::load(&settings, path, &store_factories, &working_copy_factories)?;
    let repo = workspace.repo_loader().load_at_head().block_on()?;

    Ok((workspace, repo))
}

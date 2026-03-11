use std::path::Path;
use std::process::ExitCode;

use jj_lib::config::StackedConfig;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::{Repo, StoreFactories};
use jj_lib::revset::{RevsetExpression, SymbolResolver};
use jj_lib::settings::UserSettings;
use jj_lib::workspace::{Workspace, default_working_copy_factories};
use pollster::FutureExt;

fn run(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let config = StackedConfig::with_defaults();
    let settings = UserSettings::from_config(config)?;
    let store_factories = StoreFactories::default();
    let working_copy_factories = default_working_copy_factories();

    let workspace = Workspace::load(&settings, path, &store_factories, &working_copy_factories)?;
    let repo = workspace.repo_loader().load_at_head().block_on()?;

    let expression = RevsetExpression::all();
    let no_extensions: &[Box<dyn jj_lib::revset::SymbolResolverExtension>] = &[];
    let symbol_resolver = SymbolResolver::new(repo.as_ref(), no_extensions);
    let resolved = expression.resolve_user_expression(repo.as_ref(), &symbol_resolver)?;
    let revset = resolved.evaluate(repo.as_ref())?;

    let root_commit_id = repo.store().root_commit_id().clone();

    for result in revset.iter() {
        let commit_id = result?;
        if commit_id == root_commit_id {
            continue;
        }
        let commit = repo.store().get_commit(&commit_id)?;

        let change_hex = encode_reverse_hex(commit.change_id().as_bytes());
        let change_short = &change_hex[..change_hex.len().min(12)];

        let commit_hex = commit.id().hex();
        let commit_short = &commit_hex[..commit_hex.len().min(12)];

        let first_line = commit.description().lines().next().unwrap_or("");

        println!("{} {} {}", change_short, commit_short, first_line);
    }

    Ok(())
}

fn main() -> ExitCode {
    let path = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("cannot get current directory"));

    if let Err(e) = run(&path) {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

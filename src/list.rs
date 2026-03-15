use std::path::Path;

use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::Repo;
use jj_lib::revset::{RevsetExpression, SymbolResolver};

use crate::common;

pub fn list(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let (_workspace, repo) = common::load_repo(path)?;

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

use std::path::Path;

use jj_lib::object_id::HexPrefix;
use jj_lib::object_id::PrefixResolution;
use jj_lib::repo::Repo;

use crate::common;

pub fn desc(chid: &str, repo_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let (_workspace, repo) = common::load_repo(repo_path)?;

    let prefix =
        HexPrefix::try_from_reverse_hex(chid).ok_or_else(|| format!("invalid changeID: {chid}"))?;

    let resolution = repo.resolve_change_id_prefix(&prefix)?;

    match resolution {
        PrefixResolution::SingleMatch(targets) => {
            let commit_ids = targets
                .into_visible()
                .ok_or_else(|| format!("changeID {chid} has no visible commits"))?;
            for commit_id in &commit_ids {
                let commit = repo.store().get_commit(commit_id)?;
                let desc = commit.description();
                if desc.is_empty() {
                    println!("(no description set)");
                } else {
                    print!("{desc}");
                }
            }
            Ok(())
        }
        PrefixResolution::AmbiguousMatch => {
            Err(format!("changeID prefix {chid} is ambiguous").into())
        }
        PrefixResolution::NoMatch => Err(format!("no commit found for changeID {chid}").into()),
    }
}

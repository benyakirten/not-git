use std::fs;
use std::path::PathBuf;

use crate::file_hash::FileHash;

pub struct UpdateRefsConfig {
    pub commit_hash: FileHash,
    pub path: PathBuf,
}

pub fn update_refs_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_update_refs_config(args)?;
    update_refs(config)
}

pub fn update_refs(config: UpdateRefsConfig) -> Result<(), anyhow::Error> {
    if let Some(parent) = config.path.parent() {
        fs::create_dir_all(parent)?;
    }

    // TODO: Validate hash points to a commit object.
    fs::write(&config.path, config.commit_hash.full_hash())?;

    Ok(())
}

pub fn parse_update_refs_config(args: &[String]) -> Result<UpdateRefsConfig, anyhow::Error> {
    if args.len() < 2 {
        return Err(anyhow::anyhow!("Usage: update-refs <ref> <hash>"));
    }

    let path = PathBuf::from(&args[0]);
    let path = ["not-git", "refs", "heads"]
        .iter()
        .collect::<PathBuf>()
        .join(path);
    let commit_hash = FileHash::from_sha(args[1].to_string())?;

    Ok(UpdateRefsConfig { commit_hash, path })
}

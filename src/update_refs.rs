use std::fs;
use std::path::PathBuf;

use crate::file_hash::FileHash;

pub struct UpdateRefsConfig {
    commit_hash: FileHash,
    path: PathBuf,
}

impl UpdateRefsConfig {
    pub fn new(commit_hash: FileHash, path: PathBuf) -> Self {
        Self { commit_hash, path }
    }

    pub fn hash(&self) -> &FileHash {
        &self.commit_hash
    }

    pub fn path(&self) -> PathBuf {
        ["not-git", "refs", "heads"]
            .iter()
            .collect::<PathBuf>()
            .join(self.path)
    }
}

pub fn update_refs_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_update_refs_config(args)?;
    update_refs(config)
}

pub fn update_refs(config: UpdateRefsConfig) -> Result<(), anyhow::Error> {
    let path = config.path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // TODO: Validate hash points to a commit object.
    fs::write(&path, config.commit_hash.full_hash())?;

    Ok(())
}

pub fn parse_update_refs_config(args: &[String]) -> Result<UpdateRefsConfig, anyhow::Error> {
    if args.len() < 2 {
        return Err(anyhow::anyhow!("Usage: update-refs <ref> <hash>"));
    }

    let path = PathBuf::from(&args[0]);
    let commit_hash = FileHash::new(&args[1])?;

    let config = UpdateRefsConfig::new(commit_hash, path);
    Ok(config)
}

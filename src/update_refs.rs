use std::fs;
use std::path::PathBuf;

use crate::objects::ObjectHash;

pub struct UpdateRefsConfig {
    commit_hash: ObjectHash,
    path: PathBuf,
}

impl UpdateRefsConfig {
    pub fn new(commit_hash: ObjectHash, path: PathBuf) -> Self {
        Self { commit_hash, path }
    }

    pub fn hash(&self) -> &ObjectHash {
        &self.commit_hash
    }

    pub fn path(&self) -> PathBuf {
        ["not-git", "refs", "heads"]
            .iter()
            .collect::<PathBuf>()
            .join(self.path)
    }
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

fn validate_hash_as_commit(hash: &ObjectHash) -> Result<(), anyhow::Error> {
    todo!()
}

use std::fs;
use std::path::PathBuf;

use crate::objects::{ObjectFile, ObjectHash, ObjectType};

#[derive(Debug)]
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
            .join(&self.path)
    }
}

pub fn update_refs(
    base_path: Option<&PathBuf>,
    config: UpdateRefsConfig,
) -> Result<(), anyhow::Error> {
    validate_hash_as_commit(base_path, config.hash())?;

    let path = config.path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&path, config.commit_hash.full_hash())?;

    Ok(())
}

fn validate_hash_as_commit(
    base_path: Option<&PathBuf>,
    hash: &ObjectHash,
) -> Result<(), anyhow::Error> {
    match ObjectFile::new(base_path, hash)? {
        ObjectFile::Tree(_) => Err(anyhow::anyhow!("Expected commit object")),
        ObjectFile::Other(object_contents) => match object_contents.object_type {
            ObjectType::Commit => Ok(()),
            _ => Err(anyhow::anyhow!("Expected commit object")),
        },
    }
}

use std::io::Write;
use std::path::PathBuf;

use crate::hash_object;
use crate::objects::{ObjectFile, ObjectHash, ObjectType};

pub struct CommitTreeConfig<'a> {
    pub tree_hash: &'a ObjectHash,
    pub message: String,
    pub parent_hash: Option<ObjectHash>,
}

impl<'a> CommitTreeConfig<'a> {
    pub fn new(
        tree_hash: &'a ObjectHash,
        message: String,
        parent_hash: Option<ObjectHash>,
    ) -> Self {
        CommitTreeConfig {
            tree_hash,
            message,
            parent_hash,
        }
    }
}

pub fn create_commit(
    base_path: Option<&PathBuf>,
    config: CommitTreeConfig,
) -> Result<ObjectHash, anyhow::Error> {
    let mut contents = create_commit_contents(base_path, config)?;
    let hash = hash_object::hash_and_write_object(base_path, &ObjectType::Commit, &mut contents)?;

    Ok(hash)
}

fn create_commit_contents(
    base_path: Option<&PathBuf>,
    config: CommitTreeConfig,
) -> Result<Vec<u8>, anyhow::Error> {
    validate_tree_hash(base_path, &config.tree_hash)?;

    let mut contents = Vec::new();
    writeln!(&mut contents, "tree {}", config.tree_hash.full_hash())?;

    if let Some(parent_hash) = config.parent_hash {
        validate_tree_hash(base_path, &parent_hash)?;
        writeln!(&mut contents, "parent {}", parent_hash.full_hash())?;
    }

    writeln!(
        &mut contents,
        "author Ben Horowitz <benyakir.horowitz@gmail.com>",
    )?;
    writeln!(
        &mut contents,
        "committer Ben Horowitz <benyakir.horowitz@gmail.com>",
    )?;
    writeln!(&mut contents, "{}", config.message)?;
    Ok(contents)
}

fn validate_tree_hash(
    base_path: Option<&PathBuf>,
    tree_hash: &ObjectHash,
) -> Result<(), anyhow::Error> {
    let object_file = ObjectFile::new(base_path, tree_hash)?;
    match object_file {
        ObjectFile::Tree(_) => Ok(()),
        _ => Err(anyhow::anyhow!("Expected tree object")),
    }
}

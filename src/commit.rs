use std::{fs, path::PathBuf};

use anyhow::Context;

use crate::objects::{ObjectFile, ObjectHash, ObjectType};
use crate::utils::get_head_ref;
use crate::{commit_tree, update_refs, write_tree};

pub struct CommitConfig {
    message: String,
}

impl CommitConfig {
    pub fn new(message: String) -> Self {
        CommitConfig { message }
    }
}

pub fn commit_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_commit_config(args)?;
    commit(None, config)?;

    // TODO: Add proper output message.
    // I believe this has to do with packfiles
    // Output might look like:
    // 2 files changed, 9 insertions(+)
    //  create mode 100644 src/commit.rs
    println!("Commit successful.");

    Ok(())
}

pub fn commit(base_path: Option<&PathBuf>, config: CommitConfig) -> Result<(), anyhow::Error> {
    let head_ref: String = get_head_ref(None)?;
    let head_hash = get_parent_hash(&head_ref)?;
    let parent_hash = match head_hash {
        Some(hash) => get_parent_commit(base_path, hash)?,
        None => None,
    };

    let tree_hash = write_tree::write_tree(None)?;

    let commit_tree_config =
        commit_tree::CommitTreeConfig::new(tree_hash, config.message, parent_hash);
    let commit_hash = commit_tree::create_commit(commit_tree_config)?;

    let update_path = PathBuf::from(head_ref);
    let update_refs_config = update_refs::UpdateRefsConfig::new(&commit_hash, &update_path);
    update_refs::update_refs(base_path, update_refs_config)?;

    Ok(())
}

fn parse_commit_config(args: &[String]) -> Result<CommitConfig, anyhow::Error> {
    if args.len() < 2 || args[0] != "-m" {
        return Err(anyhow::anyhow!("Usage: commit -m <message>"));
    }

    let message = args[1].to_string();

    let config = CommitConfig::new(message);
    Ok(config)
}

fn get_parent_hash(head_ref: &str) -> Result<Option<ObjectHash>, anyhow::Error> {
    let head_file_path = PathBuf::from(head_ref);
    let head_file_path = ["not-git", "refs", "heads"]
        .iter()
        .collect::<PathBuf>()
        .join(head_file_path);

    if !head_file_path.exists() {
        return Ok(None);
    }

    let current_commit_hash = fs::read_to_string(head_file_path)?;
    let hash = ObjectHash::new(current_commit_hash.trim())?;
    Ok(Some(hash))
}

fn get_parent_commit(
    base_path: Option<&PathBuf>,
    object_hash: ObjectHash,
) -> Result<Option<ObjectHash>, anyhow::Error> {
    let commit = ObjectFile::new(base_path, &object_hash)?;
    let commit_content = match commit {
        ObjectFile::Tree(_) => Err(anyhow::anyhow!("Expected commit object")),
        ObjectFile::Other(object_contents) => match object_contents.object_type {
            ObjectType::Commit => String::from_utf8(object_contents.contents)
                .context("Parsing commit content to string"),
            _ => {
                return Err(anyhow::anyhow!("Expected commit object"));
            }
        },
    }?;

    for line in commit_content.lines() {
        if let Some((_, hash)) = line.split_once("parent ") {
            let hash = ObjectHash::new(hash)?;
            return Ok(Some(hash));
        }
    }

    Ok(None)
}

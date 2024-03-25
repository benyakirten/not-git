use std::{fs, path::PathBuf};

use crate::{utils::get_head_ref, write_tree};

pub struct CommitConfig {
    pub message: String,
}

pub fn commit_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_commit_config(args)?;
    commit(config)?;

    // TODO: Add proper output message.
    // I believe this has to do with packfiles
    // Output might look like:
    // 2 files changed, 9 insertions(+)
    //  create mode 100644 src/commit.rs
    println!("Commit successful.");

    Ok(())
}

pub fn commit(config: CommitConfig) -> Result<(), anyhow::Error> {
    let head_ref: String = get_head_ref()?;
    let head_hash = get_parent_hash(&head_ref)?;
    let parent_hash = match head_hash {
        Some(hash) => get_parent_commit(hash)?,
        None => None,
    };

    let tree_hash = write_tree::write_tree()?;

    let commit_hash = commit_tree::commit_tree(CommitTreeConfig {
        tree_hash,
        message: config.message,
        parent_hash,
    })?;

    let update_refs_config = UpdateRefsConfig {
        commit_hash,
        path: PathBuf::from(head_ref),
    };
    update_refs::update_refs(update_refs_config)?;

    Ok(())
}

fn parse_commit_config(args: &[String]) -> Result<CommitConfig, anyhow::Error> {
    if args.len() < 2 || args[0] != "-m" {
        return Err(anyhow::anyhow!("Usage: commit -m <message>"));
    }

    let message = args[1].to_string();

    Ok(CommitConfig { message })
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
    let hash = current_commit_hash.trim().parse()?;
    Ok(Some(hash))
}

fn get_parent_commit(file_hash: ObjectHash) -> Result<Option<ObjectHash>, anyhow::Error> {
    let cat_file_config = CatFileConfig {
        dir: file_hash.prefix,
        file_name: file_hash.hash,
    };

    let commit_content = cat_file::decode_file(cat_file_config)?;

    for line in commit_content.lines() {
        if let Some(hash_parts) = line.split_once("parent ") {
            let hash = hash_parts.1;
            let hash = hash.parse();
            return Ok(Some(hash));
        }
    }

    Ok(None)
}

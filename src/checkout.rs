use std::fs;
use std::path::PathBuf;

use anyhow::Context;

use crate::objects::{ObjectFile, ObjectHash, ObjectType, TreeObject};

pub struct CheckoutConfig {
    branch_name: String,
}

impl CheckoutConfig {
    pub fn new(branch_name: String) -> Self {
        Self { branch_name }
    }
}

pub fn checkout_branch(
    base_path: Option<&PathBuf>,
    config: &CheckoutConfig,
) -> Result<usize, anyhow::Error> {
    let initial_tree = get_initial_tree(base_path, config)?;

    let starting_path: Vec<&str> = match base_path {
        Some(base_path) => base_path.iter().map(|p| p.to_str().unwrap()).collect(),
        None => vec![],
    };
    create_tree(base_path, initial_tree, starting_path)
}

fn create_tree(
    base_path: Option<&PathBuf>,
    tree_objects: Vec<TreeObject>,
    path_until_now: Vec<&str>,
) -> Result<usize, anyhow::Error> {
    let mut num_files_written = 0;

    for tree_object in tree_objects {
        let object: ObjectFile = ObjectFile::new(base_path, &tree_object.hash)?;

        match object {
            ObjectFile::Tree(object_contents) => {
                let mut new_path = path_until_now.clone();
                new_path.push(&tree_object.file_name);

                fs::create_dir_all(new_path.iter().collect::<PathBuf>())?;

                num_files_written += create_tree(base_path, object_contents.contents, new_path)?;
            }
            ObjectFile::Other(object_contents) => {
                let file_path = path_until_now
                    .iter()
                    .collect::<PathBuf>()
                    .join(&tree_object.file_name);

                fs::write(file_path, object_contents.contents)?;
                num_files_written += 1;
            }
        }
    }

    Ok(num_files_written)
}

fn get_initial_tree(
    base_path: Option<&PathBuf>,
    config: &CheckoutConfig,
) -> Result<Vec<TreeObject>, anyhow::Error> {
    let path: PathBuf = if config.branch_name.starts_with("remote") {
        ["not-git", "refs"].iter().collect()
    } else {
        ["not-git", "refs", "heads"].iter().collect()
    };
    let path = match base_path {
        Some(base_path) => base_path.join(path),
        None => path,
    };

    let branch_path = path.join(&config.branch_name);
    let commit_hash = fs::read_to_string(branch_path)?;
    let commit_hash = ObjectHash::new(&commit_hash)?;

    let object_file = ObjectFile::new(base_path, &commit_hash).context(format!(
        "Unable to find commit associated with branch {}",
        config.branch_name
    ))?;

    let readable_contents = match object_file {
        ObjectFile::Other(object_contents) if object_contents.object_type == ObjectType::Commit => {
            String::from_utf8(object_contents.contents)
                .context("Parsing commit object contents as utf8")
        }
        _ => return Err(anyhow::anyhow!("Expected commit object"))?,
    }?;

    let tree_hash = readable_contents
        .lines()
        .find(|line| line.starts_with("tree "))
        .ok_or_else(|| anyhow::anyhow!("No tree hash found in commit"))?
        .split_ascii_whitespace()
        .last()
        .ok_or_else(|| anyhow::anyhow!("No tree hash found in commit"))?;

    let tree_hash = ObjectHash::new(tree_hash)?;

    let tree_object =
        ObjectFile::new(base_path, &tree_hash).context("Unable to find tree object")?;
    match tree_object {
        ObjectFile::Tree(tree_object) => Ok(tree_object.contents),
        ObjectFile::Other(_) => Err(anyhow::anyhow!("Expected tree object")),
    }
}

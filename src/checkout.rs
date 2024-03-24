use std::{
    fs::{self, read_to_string},
    path::PathBuf,
};

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
    config: &CheckoutConfig,
    initial_folder: Option<&str>,
) -> Result<usize, anyhow::Error> {
    let initial_tree = get_initial_tree(config)?;

    let starting_path = match initial_folder {
        Some(folder) => {
            let path: PathBuf = [folder].iter().collect();
            fs::create_dir(&path)?;
            vec![folder]
        }
        None => vec![],
    };
    create_tree(initial_tree, starting_path)
}

fn create_tree(
    tree_objects: Vec<TreeObject>,
    path_until_now: Vec<&str>,
) -> Result<usize, anyhow::Error> {
    let mut num_files_written = 0;

    for tree_object in tree_objects {
        let object: ObjectFile = ObjectFile::new(&tree_object.hash)?;

        match object {
            ObjectFile::Tree(tree_object) => {
                let mut new_path = path_until_now.clone();
                new_path.push(&tree_object.file_name);

                num_files_written += create_tree(tree_object.contents, new_path)?;
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

fn get_initial_tree(config: &CheckoutConfig) -> Result<Vec<TreeObject>, anyhow::Error> {
    let path: PathBuf = if config.branch_name.starts_with("remote") {
        ["not-git", "refs"].iter().collect()
    } else {
        ["not-git", "refs", "heads"].iter().collect()
    };

    let branch_path = path.join(&config.branch_name);
    let commit_hash = read_to_string(branch_path)?;
    let commit_hash = ObjectHash::new(&commit_hash)?;

    let object_file = ObjectFile::new(&commit_hash).context(format!(
        "Unable to find commit associated with branch {}",
        config.branch_name
    ))?;

    let readable_contents = match object_file {
        ObjectFile::Other(object_contents) => {
            if object_contents.object_type != ObjectType::Commit {
                return Err(anyhow::anyhow!("Expected commit object"));
            }

            let contents = String::from_utf8(object_contents.contents)
                .context("Parsing commit object contents as utf8")?;

            contents
        }
        ObjectFile::Tree(tree) => {
            return Err(anyhow::anyhow!("Expected commit to be a tree object"))?
        }
    };

    let tree_hash = readable_contents
        .lines()
        .find(|line| line.starts_with("tree "));
    if tree_hash.is_none() {
        return Err(anyhow::anyhow!("No tree hash found in commit"));
    }

    let tree_hash = tree_hash
        .unwrap()
        .split_ascii_whitespace()
        .last()
        .ok_or_else(|| anyhow::anyhow!("No tree hash found in commit"))?;
    let tree_hash = ObjectHash::new(tree_hash)?;

    let tree_object = ObjectFile::new(&tree_hash).context("Unable to find tree object")?;
    match tree_object {
        ObjectFile::Tree(tree_object) => Ok(tree_object.contents),
        ObjectFile::Other(_) => Err(anyhow::anyhow!("Expected tree object")),
    }
}

fn parse_checkout_config(args: &[String]) -> Result<CheckoutConfig, anyhow::Error> {
    if args.is_empty() {
        return Err(anyhow::anyhow!("Usage: checkout <branch-name>"));
    }

    Ok(CheckoutConfig {
        branch_name: args[0].clone(),
    })
}

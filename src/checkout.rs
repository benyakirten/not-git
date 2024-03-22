use std::{
    fs::{self, read_to_string},
    path::PathBuf,
};

use anyhow::Context;

use crate::{
    cat_file::{self, CatFileConfig},
    file_hash::FileHash,
    ls_tree, utils,
};

pub struct CheckoutConfig {
    pub branch_name: String,
}

pub fn checkout_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_checkout_config(args)?;
    checkout_branch(&config)?;

    println!("Switched to branch '{}'", config.branch_name);
    Ok(())
}

pub fn checkout_branch(config: &CheckoutConfig) -> Result<usize, anyhow::Error> {
    let initial_tree = get_initial_tree(config)?;

    // TODO: Add parameter to copy all data to a folder.
    fs::create_dir("copy_folder")?;
    create_tree(initial_tree, vec!["copy_folder"])
}

fn create_tree(
    tree_files: Vec<ls_tree::TreeFile>,
    path_until_now: Vec<&str>,
) -> Result<usize, anyhow::Error> {
    let mut num_files_written = 0;

    for tree_file in tree_files {
        match tree_file.file_type {
            ls_tree::FileType::Tree => {
                let mut new_path = path_until_now.clone();
                new_path.push(&tree_file.file_name);

                let folder_path: PathBuf = new_path.iter().collect();
                fs::create_dir(folder_path)?;

                let decoded_content = utils::decode_file(
                    [
                        "not-git",
                        "objects",
                        &tree_file.hash.prefix,
                        &tree_file.hash.hash,
                    ]
                    .iter()
                    .collect(),
                )?;
                let new_tree_files = ls_tree::parse_tree_files(decoded_content)?;

                num_files_written += create_tree(new_tree_files, new_path)?;
            }
            _ => {
                let file_path = path_until_now
                    .iter()
                    .collect::<PathBuf>()
                    .join(&tree_file.file_name);
                let cat_config = CatFileConfig {
                    dir: tree_file.hash.prefix,
                    file_name: tree_file.hash.hash,
                };
                let decoded_content = cat_file::decode_file(cat_config)?;
                fs::write(file_path, decoded_content)?;

                num_files_written += 1;
            }
        }
    }

    Ok(num_files_written)
}

fn get_initial_tree(config: &CheckoutConfig) -> Result<Vec<ls_tree::TreeFile>, anyhow::Error> {
    let path: PathBuf = if config.branch_name.starts_with("remote") {
        ["not-git", "refs"].iter().collect()
    } else {
        ["not-git", "refs", "heads"].iter().collect()
    };

    let commit_sha = read_to_string(path.join(config.branch_name.to_string()))?;
    let commit_sha = FileHash::from_sha(commit_sha)?;

    let cat_config = CatFileConfig {
        dir: commit_sha.prefix,
        file_name: commit_sha.hash,
    };
    let file_contents = cat_file::decode_file(cat_config).context(format!(
        "Unable to find commit associated with branch {}",
        config.branch_name
    ))?;

    let tree_hash = file_contents.lines().find(|line| line.starts_with("tree "));
    if tree_hash.is_none() {
        return Err(anyhow::anyhow!("No tree hash found in commit"));
    }
    let tree_hash = tree_hash
        .unwrap()
        .split_ascii_whitespace()
        .last()
        .ok_or_else(|| anyhow::anyhow!("No tree hash found in commit"))?;
    let tree_hash = FileHash::from_sha(tree_hash.to_string())?;

    let tree_path = ["not-git", "objects", &tree_hash.prefix, &tree_hash.hash]
        .iter()
        .collect::<PathBuf>();
    let decoded_tree = utils::decode_file(tree_path)?;

    ls_tree::parse_tree_files(decoded_tree)
}

fn parse_checkout_config(args: &[String]) -> Result<CheckoutConfig, anyhow::Error> {
    if args.len() < 1 {
        return Err(anyhow::anyhow!("Usage: checkout <branch-name>"));
    }

    Ok(CheckoutConfig {
        branch_name: args[0].clone(),
    })
}

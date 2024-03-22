use std::{fs::read_to_string, path::PathBuf};

use crate::{
    cat_file::{self, CatFileConfig},
    file_hash::FileHash,
    ls_tree,
};

pub struct CheckoutConfig {
    pub branch_name: String,
}

pub fn checkout_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_checkout_config(args)?;
    checkout_files(&config)?;

    println!("Switched to branch '{}'", config.branch_name);
    Ok(())
}

pub fn checkout_files(config: &CheckoutConfig) -> Result<(), anyhow::Error> {
    let path: PathBuf = if config.branch_name.starts_with("remote") {
        ["not-git", "refs"].iter().collect()
    } else {
        ["not-git", "refs", "heads"].iter().collect()
    };

    let path = path.join(&config.branch_name);
    let commit_sha = read_to_string(path)?;
    let commit_sha = FileHash::from_sha(commit_sha)?;

    let cat_config = CatFileConfig {
        dir: commit_sha.prefix,
        file_name: commit_sha.hash,
    };
    let file_contents = cat_file::decode_file(cat_config)?;
    println!("{}", file_contents);

    Ok(())
}

fn parse_checkout_config(args: &[String]) -> Result<CheckoutConfig, anyhow::Error> {
    if args.len() < 1 {
        return Err(anyhow::anyhow!("Usage: checkout <branch-name>"));
    }

    Ok(CheckoutConfig {
        branch_name: args[0].clone(),
    })
}

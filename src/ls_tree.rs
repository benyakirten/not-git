use std::path::PathBuf;

use crate::{file_hash::FileHash, utils};

pub fn list_tree(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_config(args)?;
    let decoded_content = decode_file(&config)?;
    let file_names = get_file_names(&decoded_content);

    for file_name in file_names {
        println!("{}", file_name);
    }
    Ok(())
}

fn get_file_names(decoded_content: &str) -> Vec<String> {
    decoded_content
        .split("\0")
        .skip(1)
        .filter_map(|line| {
            let file_name = line.split_whitespace().skip(1).last();
            match file_name {
                Some(name) => Some(name.to_string()),
                None => None,
            }
        })
        .collect()
}

fn decode_file(config: &FileHash) -> Result<String, anyhow::Error> {
    let path = [".git", "objects", &config.prefix, &config.hash]
        .iter()
        .collect::<PathBuf>();

    let decoded_content = utils::decode_file(path)?;
    let decoded_string = String::from_utf8_lossy(&decoded_content);
    Ok(decoded_string.to_string())
}

fn parse_config(args: &[String]) -> Result<FileHash, anyhow::Error> {
    if args.len() < 2 || args[0] != "--name-only" {
        return Err(anyhow::anyhow!("Usage: ls-tree --name-only <tree_sha>"));
    }

    let sha = args[1].to_string();
    FileHash::from_sha(sha)
}

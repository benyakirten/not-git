use std::path::PathBuf;

use crate::{ls_tree, utils};

pub struct CatFileConfig {
    pub dir: String,
    pub file_name: String,
}

impl Into<PathBuf> for CatFileConfig {
    fn into(self) -> PathBuf {
        ["not-git", "objects", &self.dir, &self.file_name]
            .iter()
            .collect()
    }
}

pub fn cat(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_cat_file_cmds(args)?;
    let contents = decode_file(config)?;

    print!("{}", &contents);

    Ok(())
}

pub fn decode_file(config: CatFileConfig) -> Result<String, anyhow::Error> {
    let decoded_content = utils::decode_file(config.into())?;
    let decoded_string = String::from_utf8_lossy(&decoded_content);

    // Git begins the file with {file_type} <size>\0, so we need to remove it.
    if decoded_string.starts_with("tree") {
        return ls_tree::stringify_list_tree(decoded_content, ls_tree::LsTreeFlag::Long);
    }

    match decoded_string.split("\x00").last() {
        Some(content) => Ok(content.to_string()),
        None => Ok(decoded_string.to_string()),
    }
}

fn parse_cat_file_cmds(args: &[String]) -> Result<CatFileConfig, anyhow::Error> {
    if args.len() == 0 {
        return Err(anyhow::anyhow!("Usage: cat-file <hash>"));
    }

    let full_hash = get_full_hash(args)?;
    Ok(CatFileConfig {
        dir: full_hash[..2].to_string(),
        file_name: full_hash[2..].to_string(),
    })
}

fn get_full_hash(args: &[String]) -> Result<String, anyhow::Error> {
    let initial_index;
    if args[0] == "-p" {
        initial_index = 1;

        if args.len() < 2 {
            return Err(anyhow::anyhow!("Usage: cat-file -p <hash>",));
        }
    } else {
        return Err(anyhow::anyhow!("-p flag must be provided"));
    }

    let hash = args[initial_index].as_str();
    Ok(hash.to_string())
}

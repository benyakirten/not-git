use std::io::Read;
use std::path::PathBuf;

use flate2::bufread::ZlibDecoder;

use crate::utils::{self, print_string};

struct CatFileConfig {
    dir: String,
    file_name: String,
}

impl Into<PathBuf> for CatFileConfig {
    fn into(self) -> PathBuf {
        [".git", "objects", &self.dir, &self.file_name]
            .iter()
            .collect()
    }
}

pub fn cat(args: Vec<String>) -> Result<(), anyhow::Error> {
    let config = parse_cat_file_cmds(args)?;
    let contents = decode_file(config)?;

    print_string(&contents);

    Ok(())
}

fn decode_file(config: CatFileConfig) -> Result<String, anyhow::Error> {
    let path: PathBuf = config.into();
    let encoded_content = utils::read_from_file(path)?;

    let mut decoded_content = String::new();
    let mut decoder = ZlibDecoder::new(encoded_content.as_slice());
    decoder.read_to_string(&mut decoded_content)?;

    // Git begins the file with blob <size>\x00, so we need to remove that
    match decoded_content.split("\x00").last() {
        Some(content) => Ok(content.to_string()),
        None => Ok(decoded_content),
    }
}

fn parse_cat_file_cmds(args: Vec<String>) -> Result<CatFileConfig, anyhow::Error> {
    if args.len() == 0 {
        return Err(anyhow::anyhow!("Usage: cat-file <hash>"));
    }

    let full_hash = get_full_hash(args)?;
    Ok(CatFileConfig {
        dir: full_hash[..2].to_string(),
        file_name: full_hash[2..].to_string(),
    })
}

fn get_full_hash(args: Vec<String>) -> Result<String, anyhow::Error> {
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

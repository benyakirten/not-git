use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use flate2::bufread::ZlibDecoder;

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

pub fn file(args: Vec<String>) -> Result<(), io::Error> {
    let config = parse_cat_file_cmds(args)?;
    let contents = decode_file(config)?;

    print!("{}", contents);

    Ok(())
}

fn decode_file(config: CatFileConfig) -> Result<String, io::Error> {
    let path: PathBuf = config.into();
    let mut file = fs::File::open(path)?;
    let mut encoded_content = vec![];
    file.read_to_end(&mut encoded_content)?;

    let mut decoded_content = String::new();
    let mut decoder = ZlibDecoder::new(encoded_content.as_slice());
    decoder.read_to_string(&mut decoded_content)?;

    match decoded_content.split("\x00").last() {
        Some(content) => Ok(content.to_string()),
        None => Ok(decoded_content),
    }
}

fn parse_cat_file_cmds(args: Vec<String>) -> Result<CatFileConfig, io::Error> {
    if args.len() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Usage: cat-file <hash>",
        ));
    }

    let full_hash = get_full_hash(args)?;
    Ok(CatFileConfig {
        dir: full_hash[..2].to_string(),
        file_name: full_hash[2..].to_string(),
    })
}

fn get_full_hash(args: Vec<String>) -> Result<String, io::Error> {
    let initial_index;
    if args[0] == "-p" {
        initial_index = 1;

        if args.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Usage: cat-file -p <hash>",
            ));
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "-p flag must be provided",
        ));
    }

    let hash = args[initial_index].as_str();
    Ok(hash.to_string())
}

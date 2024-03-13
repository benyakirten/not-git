use std::{fs, io::Write, path::PathBuf};

use flate2::{write::ZlibEncoder, Compression};
use hex::ToHex;
use sha1::{Digest, Sha1};

use crate::utils::{print_string, read_from_file};

struct HashObjectConfig {
    file: String,
}

struct FileHash {
    prefix: String,
    hash: String,
}

impl FileHash {
    fn new(prefix: String, hash: String) -> Self {
        FileHash { prefix, hash }
    }

    fn full_hash(&self) -> String {
        self.prefix.to_string() + &self.hash.to_string()
    }
}

pub fn hash(args: Vec<String>) -> Result<(), anyhow::Error> {
    let config = parse_config(args)?;
    let file_contents = read_from_file(config.file)?;

    let hash = hash_file(&file_contents)?;
    let encoded_contents = encode_file_contents(&file_contents);

    write_encoded_object(&hash, encoded_contents)?;
    print_string(&hash.full_hash());

    Ok(())
}

fn write_encoded_object(hash: &FileHash, encoded_contents: Vec<u8>) -> Result<(), anyhow::Error> {
    let path: PathBuf = [".git", "objects", &hash.prefix].iter().collect();

    if !path.exists() {
        fs::create_dir(&path)?;
    }

    fs::write(path.join(&hash.hash), encoded_contents)?;
    Ok(())
}

fn hash_file(file: &Vec<u8>) -> Result<FileHash, anyhow::Error> {
    let mut hasher = Sha1::new();
    hasher.update(file);
    let result: String = hasher.finalize().encode_hex();

    if result.len() < 40 {
        return Err(anyhow::anyhow!(
            "Expected sha1 hash to be 40 characters long"
        ));
    }

    Ok(FileHash::new(
        result[..2].to_string(),
        result[2..].to_string(),
    ))
}

fn encode_file_contents(file: &Vec<u8>) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(file).unwrap();
    encoder.finish().unwrap()
}

fn parse_config(args: Vec<String>) -> Result<HashObjectConfig, anyhow::Error> {
    if args.len() < 2 || args[0] != "-w" {
        return Err(anyhow::anyhow!("Usage: hash-object -w <file>"));
    }

    Ok(HashObjectConfig {
        file: args[1].to_string(),
    })
}

use std::{fs, io::Write, path::PathBuf};

use flate2::{write::ZlibEncoder, Compression};
use hex::ToHex;
use sha1::{Digest, Sha1};

use crate::{
    file_hash::FileHash,
    ls_tree::FileType,
    utils::{create_header, read_from_file},
};

struct HashObjectConfig {
    file: String,
}

pub fn write_and_output(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_config(args)?;
    let mut file_contents = read_from_file(config.file.as_str())?;

    let hash = hash_and_write(&FileType::Blob, &mut file_contents)?;
    print!("{}", &hash.full_hash());

    Ok(())
}

pub fn hash_and_write(
    file_type: &FileType,
    file_contents: &mut Vec<u8>,
) -> Result<FileHash, anyhow::Error> {
    let mut header = create_header(file_type, &file_contents);

    header.append(file_contents);
    let hash = hash_file(&header)?;
    let encoded_contents = encode_file_contents(header)?;

    write_encoded_object(&hash, encoded_contents)?;

    Ok(hash)
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

fn encode_file_contents(file_contents: Vec<u8>) -> Result<Vec<u8>, anyhow::Error> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&file_contents)?;
    encoder.finish().map_err(|e| anyhow::anyhow!(e))
}

fn parse_config(args: &[String]) -> Result<HashObjectConfig, anyhow::Error> {
    if args.len() < 2 || args[0] != "-w" {
        return Err(anyhow::anyhow!("Usage: hash-object -w <file>"));
    }

    Ok(HashObjectConfig {
        file: args[1].to_string(),
    })
}

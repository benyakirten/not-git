use std::fs;
use std::io::Write;
use std::path::PathBuf;

use flate2::write::ZlibEncoder;
use flate2::Compression;
use hex::ToHex;
use sha1::{Digest, Sha1};

use crate::{
    objects::{ObjectHash, ObjectType},
    utils::create_header,
};

struct HashObjectConfig {
    file: String,
}

pub fn hash_object_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_config(args)?;
    let mut file_contents = fs::read(config.file.as_str())?;

    let hash = hash_and_write_object(None, &ObjectType::Blob, &mut file_contents)?;
    print!("{}", &hash.full_hash());

    Ok(())
}

pub fn hash_and_write_object(
    base_path: Option<&PathBuf>,
    object_type: &ObjectType,
    file_contents: &mut Vec<u8>,
) -> Result<ObjectHash, anyhow::Error> {
    let mut header = create_header(object_type, file_contents);

    header.append(file_contents);
    let hash = hash_file(&header)?;
    let encoded_contents = encode_file_contents(header)?;

    write_encoded_object(base_path, &hash, encoded_contents)?;

    Ok(hash)
}

fn write_encoded_object(
    base_path: Option<&PathBuf>,
    hash: &ObjectHash,
    encoded_contents: Vec<u8>,
) -> Result<(), anyhow::Error> {
    let path: PathBuf = match base_path {
        Some(base_path) => base_path.join(hash.path()),
        None => hash.path(),
    };

    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Unable to find parent path to {:?}", path))?;
    if !parent.exists() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, encoded_contents)?;
    Ok(())
}

fn hash_file(file: &Vec<u8>) -> Result<ObjectHash, anyhow::Error> {
    let mut hasher = Sha1::new();
    hasher.update(file);
    let result: String = hasher.finalize().encode_hex();

    if result.len() < 40 {
        return Err(anyhow::anyhow!(
            "Expected sha1 hash to be 40 characters long"
        ));
    }

    let hash = ObjectHash::new(&result)?;
    Ok(hash)
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

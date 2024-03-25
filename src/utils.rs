use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::Context;
use flate2::read::ZlibDecoder;

use crate::objects::ObjectType;

/// Utiltiy function for reading the contents of a file.
pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, anyhow::Error> {
    let mut file = fs::File::open(path)?;
    let mut content = vec![];
    file.read_to_end(&mut content)?;

    Ok(content)
}

/// Utility function for decoding a file that has been encoded with zlib.
pub fn decode_file(path: PathBuf) -> Result<Vec<u8>, anyhow::Error> {
    let encoded_content = read_from_file(path)?;

    let mut decoder = ZlibDecoder::new(encoded_content.as_slice());

    let mut decoded_vec = vec![];
    decoder.read_to_end(&mut decoded_vec)?;
    Ok(decoded_vec)
}

/// Given a file type and the
pub fn create_header(object_type: &ObjectType, file: &[u8]) -> Vec<u8> {
    let header = format!("{} {}\0", object_type.as_str(), file.len());
    header.as_bytes().to_vec()
}

pub fn get_head_ref() -> Result<String, anyhow::Error> {
    let head = ["not-git", "HEAD"].iter().collect::<PathBuf>();
    let head = fs::read_to_string(head)?;

    let head_ref = head
        .split("refs/heads/")
        .last()
        .ok_or_else(|| {
            anyhow::anyhow!(format!(
                "Invalid HEAD file. Expected ref: refs/heads<branch_name>, got {}",
                head
            ))
        })?
        .trim();

    Ok(head_ref.to_string())
}

pub fn split_header_from_contents(content: &[u8]) -> Result<(&[u8], &[u8]), anyhow::Error> {
    let mut split_content = content.splitn(2, |&x| x == 0);

    let header = split_content.next().context("Getting header")?;
    let body = split_content.next().context("Getting body")?;
    Ok((header, body))
}

pub fn copy_dir(src: &PathBuf, dest: &PathBuf) -> Result<(), anyhow::Error> {
    if !src.is_dir() {
        return Err(anyhow::anyhow!("Source is not a directory"));
    }

    if !dest.exists() {
        fs::create_dir_all(dest)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if entry_path.is_dir() {
            copy_dir(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path)?;
        }
    }

    Ok(())
}

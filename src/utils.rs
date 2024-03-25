use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

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

pub fn get_head_ref(base_path: Option<&PathBuf>) -> Result<String, anyhow::Error> {
    let head = match base_path {
        Some(path) => PathBuf::from(path),
        None => std::env::current_dir()?,
    };
    let head = head
        .join("not-git")
        .join("HEAD")
        .iter()
        .collect::<PathBuf>();

    let head = fs::read_to_string(head)?;

    if !head.starts_with("ref: ") {
        return Err(anyhow::anyhow!(format!(
            "Invalid HEAD file. Expected ref: refs/heads/<branch_name>, got {}",
            head
        )));
    }

    let head_ref = head
        .split("refs/heads/")
        .last()
        .ok_or_else(|| {
            anyhow::anyhow!(format!(
                "Invalid HEAD file. Expected ref: refs/heads/<branch_name>, got {}",
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

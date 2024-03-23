use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use flate2::read::ZlibDecoder;

use crate::ls_tree::FileType;

pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut content = vec![];
    file.read_to_end(&mut content)?;

    Ok(content)
}

pub fn decode_file(path: PathBuf) -> Result<Vec<u8>, anyhow::Error> {
    let encoded_content = read_from_file(path)?;

    let mut decoder = ZlibDecoder::new(encoded_content.as_slice());

    let mut decoded_vec = vec![];
    decoder.read_to_end(&mut decoded_vec)?;
    Ok(decoded_vec)
}

pub fn create_header(file_type: &FileType, file: &[u8]) -> Vec<u8> {
    let header = format!("{} {}\0", file_type.to_readable_string(), file.len());
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

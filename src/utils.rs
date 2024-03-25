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

pub fn get_head_ref() -> Result<String, anyhow::Error> {
    let head = ["not-git", "HEAD"].iter().collect::<PathBuf>();
    let head = fs::read_to_string(head)?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_header_from_contents_returns_header_and_body_success() {
        let content = b"blob 5\0hello";
        let (header, body) = split_header_from_contents(content).unwrap();
        assert_eq!(header, b"blob 5");
        assert_eq!(body, b"hello");
    }

    #[test]
    fn split_header_from_body_error_no_null_byte() {
        let content = b"blob 5hello";
        let result = split_header_from_contents(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_header_success() {
        let object_type = ObjectType::Blob;
        let file = b"file content";
        let header = create_header(&object_type, file);
        assert_eq!(header, b"blob 12\0");
    }

    #[test]
    fn test_split_header_from_contents_success() {
        let content = b"blob 5\0hello";
        let (header, body) = split_header_from_contents(content).unwrap();
        assert_eq!(header, b"blob 5");
        assert_eq!(body, b"hello");
    }

    #[test]
    fn test_split_header_from_contents_error() {
        let content = b"blob 5hello";
        let result = split_header_from_contents(content);
        assert!(result.is_err());
    }

    // #[test]
    // fn test_get_head_ref() {
    //     let head_ref = get_head_ref().unwrap();
    //     assert_eq!(head_ref, "branch_name");
    // }

    // #[test]
    // fn test_read_from_file_success() {
    //     let content = read_from_file("/path/to/file.txt").unwrap();
    //     assert_eq!(content, b"file content");
    // }

    // fn test_read_from_file_error_if_file_missing() {}
}

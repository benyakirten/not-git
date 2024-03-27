use std::fs;

use common::TestPath;

use not_git::objects::ObjectType;
use not_git::utils::{create_header, get_head_ref, split_header_from_contents};

mod common;

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
fn create_header_success() {
    let object_type = ObjectType::Blob;
    let file = b"file content";
    let header = create_header(&object_type, file);
    assert_eq!(header, b"blob 12\0");
}

#[test]
fn split_header_from_contents_success() {
    let content = b"blob 5\0hello";
    let (header, body) = split_header_from_contents(content).unwrap();
    assert_eq!(header, b"blob 5");
    assert_eq!(body, b"hello");
}

#[test]
fn split_header_from_contents_error() {
    let content = b"blob 5hello";
    let result = split_header_from_contents(content);
    assert!(result.is_err());
}

fn write_head_file(contents: &str) -> TestPath {
    let path = common::TestPath::new();
    let head = path.join(&"not-git").join("HEAD");

    fs::create_dir_all(head.parent().unwrap()).unwrap();
    fs::write(head, contents).unwrap();

    path
}

#[test]
fn get_head_ref_success() {
    let branch_name: &str = "test_branch_name";
    let path = write_head_file(&format!("ref: refs/heads/{}\n", branch_name));

    let head_ref = get_head_ref(Some(&path.0)).unwrap();
    assert_eq!(head_ref, branch_name);
}

#[test]
fn get_head_ref_error_no_file() {
    let path = common::TestPath::new();

    let result = get_head_ref(Some(&path.0));
    assert!(result.is_err());
}

#[test]
fn get_head_ref_error_improper_format() {
    let branch_name: &str = "test_branch_name";
    let path = write_head_file(&format!("refs/heads/{}\n", branch_name));

    let result = get_head_ref(Some(&path.0));
    assert!(result.is_err());
}

#[test]
fn decode_file_success() {
    let path = common::TestPath::new();
    let file_path = path.join(&"test_file");

    let contents = b"these are the file contents".to_vec();
    let encoded_contents = common::encode_to_zlib(contents.as_slice());

    fs::write(&file_path, encoded_contents).unwrap();

    let decoded = not_git::utils::decode_file(file_path).unwrap();
    assert_eq!(decoded, contents);
}

#[test]
fn decode_file_error_no_file() {
    let path = common::TestPath::new();
    let file_path = path.join(&"test_file");

    let decoded = not_git::utils::decode_file(file_path);
    assert!(decoded.is_err());
}

#[test]
fn decode_file_error_not_encoded() {
    let path = common::TestPath::new();
    let file_path = path.join(&"test_file");

    let contents = b"these are the file contents".to_vec();
    fs::write(&file_path, contents).unwrap();

    let decoded = not_git::utils::decode_file(file_path);
    assert!(decoded.is_err());
}

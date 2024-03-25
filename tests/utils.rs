use std::fs;
use std::path::PathBuf;

use not_git::objects::ObjectType;
use not_git::utils::{create_header, get_head_ref, read_from_file, split_header_from_contents};

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

fn write_head_file(contents: &str) -> PathBuf {
    let path = common::setup();
    let head = path.join("not-git").join("HEAD");

    fs::create_dir_all(head.parent().unwrap()).unwrap();
    fs::write(head, contents).unwrap();

    path
}

#[test]
fn test_get_head_ref_success() {
    let branch_name: &str = "test_branch_name";
    let path = write_head_file(&format!("ref: refs/heads/{}\n", branch_name));

    let head_ref = get_head_ref(Some(&path)).unwrap();
    assert_eq!(head_ref, branch_name);

    common::cleanup(path);
}

#[test]
fn test_get_head_ref_error_no_file() {
    let path = common::setup();

    let result = get_head_ref(None);
    assert!(result.is_err());

    common::cleanup(path);
}

#[test]
fn test_get_head_ref_error_improper_format() {
    let branch_name: &str = "test_branch_name";
    let path = write_head_file(&format!("refs/heads/{}\n", branch_name));

    let result = get_head_ref(None);
    assert!(result.is_err());

    common::cleanup(path);
}

#[test]
fn test_read_from_file_success() {
    let path = common::setup();
    let content = b"test file content";

    let file_path = path.join("test_file");
    fs::write(&file_path, content).unwrap();

    let read_content = read_from_file(&file_path).unwrap();
    assert_eq!(read_content, content);

    common::cleanup(path);
}

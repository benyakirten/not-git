use std::fs;

use not_git::{init, objects::ObjectFile, write_tree};

mod common;

#[test]
fn test_write_tree_success() {
    let path = common::TestPath::new();
    init::create_directories(init::InitConfig::new("main", path.to_str())).unwrap();

    let contents_1 = b"Test 1".to_vec();
    let file_name_1 = "file1".to_string();

    fs::write(path.join(&file_name_1), &contents_1).unwrap();

    let contents_2 = b"Test 2".to_vec();
    let file_name_2 = "file2".to_string();

    fs::write(path.join(&file_name_2), &contents_2).unwrap();

    let contents_3 = b"Test 3".to_vec();
    let file_name_3 = "file3".to_string();

    let sub_dir = path.join(&"sub_dir");
    fs::create_dir(&sub_dir).unwrap();
    fs::write(sub_dir.join(&file_name_3), &contents_3).unwrap();

    let path_str = path.0.to_str();
    let tree_hash = write_tree::write_tree(Some(&path.0), path_str).unwrap();

    assert_eq!(
        tree_hash.full_hash(),
        "7abecc23db04e758fd76dd97a97901597ced79cf"
    );

    let tree_file = ObjectFile::new(Some(&path.0), &tree_hash);
}

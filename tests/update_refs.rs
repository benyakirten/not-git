use std::{fs, path::PathBuf};

use not_git::objects::ObjectHash;
use not_git::update_refs;

mod common;

#[test]
fn update_refs_success() {
    let path = common::TestPath::new();

    let old_commit_hash = "0123456789abcdef0123456789abcdef01234567";
    let old_commit_hash = ObjectHash::new(old_commit_hash).unwrap();

    let ref_path = path.join(&"not-git/refs/heads/abc/def/ghi");
    fs::create_dir_all(ref_path.parent().unwrap()).unwrap();
    fs::write(&ref_path, old_commit_hash.full_hash()).unwrap();

    let tree_hash: [u8; 20] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    ];
    let tree_hash = ObjectHash::from_bytes(&tree_hash).unwrap();

    let new_commit_hash = common::write_commit(&path, &tree_hash, None, "test-commit");
    let update_path = PathBuf::from("abc/def/ghi");

    let config = update_refs::UpdateRefsConfig::new(&new_commit_hash, &update_path);
    update_refs::update_refs(Some(&path.0), config).unwrap();

    let got_commit_hash = fs::read_to_string(&ref_path).unwrap();
    assert_eq!(got_commit_hash, new_commit_hash.full_hash());
}

#[test]
fn update_refs_create_branch_if_not_exists() {
    let path = common::TestPath::new();

    let tree_hash: [u8; 20] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    ];
    let tree_hash = ObjectHash::from_bytes(&tree_hash).unwrap();

    let commit_hash = common::write_commit(&path, &tree_hash, None, "test-commit");
    let update_path = PathBuf::from("abc/def/ghi");

    let config = update_refs::UpdateRefsConfig::new(&commit_hash, &update_path);
    update_refs::update_refs(Some(&path.0), config).unwrap();

    let ref_path = path.join(&"not-git/refs/heads/abc/def/ghi");
    let got_commit_hash = fs::read_to_string(ref_path).unwrap();
    assert_eq!(got_commit_hash, commit_hash.full_hash());
}

#[test]
fn update_refs_failure_not_commit_hash() {
    let path = common::TestPath::new();

    let commit_hash: [u8; 20] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    ];
    let commit_hash = ObjectHash::from_bytes(&commit_hash).unwrap();
    let update_path = PathBuf::from("abc/def/ghi");

    let config = update_refs::UpdateRefsConfig::new(&commit_hash, &update_path);
    let update_refs_result = update_refs::update_refs(Some(&path.0), config);

    assert!(update_refs_result.is_err());
}

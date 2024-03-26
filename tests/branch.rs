use std::{fs, path::PathBuf};

use not_git::objects::ObjectHash;
use not_git::{branch, init, update_refs};

mod common;

#[test]
fn delete_branch_fails_if_branch_does_not_exist() {
    let path = common::TestPath::new();

    init::create_directories(init::InitConfig::new("main", path.0.to_str())).unwrap();

    let got = branch::delete_branch(Some(&path.0), "nonexistent");
    assert!(got.is_err());
}

#[test]
fn delete_branch_fails_if_branch_is_current() {
    let path = common::TestPath::new();

    init::create_directories(init::InitConfig::new("main", path.0.to_str())).unwrap();

    let got = branch::delete_branch(Some(&path.0), "main");
    assert!(got.is_err());
}

#[test]
fn delete_branch_success() {
    let path = common::TestPath::new();

    init::create_directories(init::InitConfig::new("main", path.0.to_str())).unwrap();

    let tree_hash: [u8; 20] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    ];
    let tree_hash = ObjectHash::from_bytes(&tree_hash).unwrap();

    let commit_hash = common::write_commit(&path, &tree_hash, None, "test-commit");
    let update_path = PathBuf::from("abc/def/ghi");

    let config = update_refs::UpdateRefsConfig::new(&commit_hash, &update_path);
    update_refs::update_refs(Some(&path.0), config).unwrap();

    let got = branch::delete_branch(Some(&path.0), "abc/def/ghi");
    assert!(got.is_ok());

    let ref_path = path.join(&"not-git/refs/heads/main");
    assert!(!ref_path.exists());
}

#[test]
fn list_branches_lists_all_branches() {
    let path = common::TestPath::new();

    init::create_directories(init::InitConfig::new("main", path.0.to_str())).unwrap();

    let tree_hash: [u8; 20] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    ];
    let tree_hash = ObjectHash::from_bytes(&tree_hash).unwrap();

    let commit_hash = common::write_commit(&path, &tree_hash, None, "test-commit");

    let ref_path_1 = PathBuf::from("abc/def/ghi");
    let ref_path_2 = PathBuf::from("main");
    let ref_path_3 = PathBuf::from("jkl/mno/pqr");

    let config_1 = update_refs::UpdateRefsConfig::new(&commit_hash, &ref_path_1);
    update_refs::update_refs(Some(&path.0), config_1).unwrap();

    let config_2 = update_refs::UpdateRefsConfig::new(&commit_hash, &ref_path_2);
    update_refs::update_refs(Some(&path.0), config_2).unwrap();

    let config_3 = update_refs::UpdateRefsConfig::new(&commit_hash, &ref_path_3);
    update_refs::update_refs(Some(&path.0), config_3).unwrap();

    let branch_options = branch::list_branches(Some(&path.0), true).unwrap();
    assert_eq!(branch_options.branches.len(), 3);
    assert_eq!(
        branch_options.branches,
        vec![
            ref_path_1.to_str().unwrap(),
            ref_path_3.to_str().unwrap(),
            ref_path_2.to_str().unwrap(),
        ]
    );
    assert_eq!(branch_options.head_ref, "main");
}

#[test]
fn list_branches_fails_if_no_head_branch() {
    let path = common::TestPath::new();

    let tree_hash: [u8; 20] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    ];
    let tree_hash = ObjectHash::from_bytes(&tree_hash).unwrap();

    let commit_hash = common::write_commit(&path, &tree_hash, None, "test-commit");

    let ref_path_1 = PathBuf::from("abc/def/ghi");
    let ref_path_2 = PathBuf::from("main");
    let ref_path_3 = PathBuf::from("jkl/mno/pqr");

    let config_1 = update_refs::UpdateRefsConfig::new(&commit_hash, &ref_path_1);
    update_refs::update_refs(Some(&path.0), config_1).unwrap();

    let config_2 = update_refs::UpdateRefsConfig::new(&commit_hash, &ref_path_2);
    update_refs::update_refs(Some(&path.0), config_2).unwrap();

    let config_3 = update_refs::UpdateRefsConfig::new(&commit_hash, &ref_path_3);
    update_refs::update_refs(Some(&path.0), config_3).unwrap();

    let branch_options = branch::list_branches(Some(&path.0), true);
    assert!(branch_options.is_err());
}

#[test]
fn create_branch_fails_if_branch_already_exists() {
    let path = common::TestPath::new();

    init::create_directories(init::InitConfig::new("main", path.0.to_str())).unwrap();

    let got = branch::create_branch(Some(&path.0), "main");
    assert!(got.is_err());
}

#[test]
fn create_branch_fails_if_no_head_branch() {
    let path = common::TestPath::new();

    let got = branch::create_branch(Some(&path.0), "new/branch");
    assert!(got.is_err());
}

#[test]
fn create_branch_creates_new_branch() {
    let path = common::TestPath::new();

    init::create_directories(init::InitConfig::new("main", path.0.to_str())).unwrap();

    let tree_hash: [u8; 20] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    ];
    let tree_hash = ObjectHash::from_bytes(&tree_hash).unwrap();

    let commit_hash = common::write_commit(&path, &tree_hash, None, "test-commit");
    let ref_path = PathBuf::from("main");

    let update_refs_config = update_refs::UpdateRefsConfig::new(&commit_hash, &ref_path);
    update_refs::update_refs(Some(&path.0), update_refs_config).unwrap();

    let got = branch::create_branch(Some(&path.0), "new/branch");
    assert!(got.is_ok());

    let ref_path = path.join(&"not-git/refs/heads/new/branch");
    assert!(ref_path.exists());

    let ref_contents = fs::read(&ref_path).unwrap();
    let ref_contents = String::from_utf8(ref_contents).unwrap();
    assert_eq!(ref_contents, commit_hash.full_hash());
}

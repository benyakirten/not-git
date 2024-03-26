use std::{fs, path::PathBuf};

use not_git::{
    commit, init,
    objects::{ObjectFile, ObjectHash, ObjectType},
    utils::{self, get_head_ref},
};

mod common;

#[test]
fn parent_commit_preserved_on_new_commit() {
    let path = common::TestPath::new();
    // Steps
    // 1. Init repository
    // 2. Create a commit for parent
    // 3. Update refs to commit
    // 4. Write some files
    // 5. Run command
    // 6. Examine files
}

#[test]
fn commit_successful_without_parent_commit() {
    let path = common::TestPath::new();
    // Steps
    // 1. Init repository
    let init_config = init::InitConfig::new("main", path.to_str());
    init::create_directories(init_config).unwrap();

    let contents_1 = b"Test 1".to_vec();
    let file_name_1 = "file_1.txt";

    let contents_2 = b"Test 2".to_vec();
    let file_name_2 = "file_2.txt";

    let contents_3 = b"Test 3".to_vec();
    let file_name_3 = "file_3.txt";

    fs::write(path.join(&file_name_1), contents_1).unwrap();
    fs::write(path.join(&file_name_2), contents_2).unwrap();
    fs::write(path.join(&file_name_3), contents_3).unwrap();

    let commit_config = commit::CommitConfig::new("Test commit".to_string());
    commit::commit(Some(&path.0), commit_config).unwrap();

    let head_ref = get_head_ref(Some(&path.0)).unwrap();
    let head_path = PathBuf::from("not-git/refs/heads").join(head_ref);
    let head_path = path.join(&head_path);

    let head_hash = fs::read_to_string(head_path).unwrap();
    let head_hash = ObjectHash::new(&head_hash).unwrap();

    let head_commit = ObjectFile::new(Some(&path.0), &head_hash).unwrap();

    let head_commit = match head_commit {
        ObjectFile::Other(commit) => commit,
        ObjectFile::Tree(_) => panic!("Expected commit object"),
    };

    assert_eq!(head_commit.object_type, ObjectType::Commit);
}

#[test]
fn commit_error_if_head_ref_not_found() {
    let path = common::TestPath::new();

    let config = commit::CommitConfig::new("Test commit".to_string());
    let result = commit::commit(Some(&path.0), config);
    assert!(result.is_err());
}

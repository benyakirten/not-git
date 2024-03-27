use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use not_git::{checkout, init, update_refs};

mod common;

#[test]
fn checkout_creates_files_in_new_branch() {
    let path = common::TestPath::new();

    let init_config = init::InitConfig::new("main", path.to_str());
    init::create_directories(init_config).unwrap();

    let tree_hash = common::create_valid_tree_hash(&path);
    let commit_hash = common::write_commit(&path, &tree_hash, None, "Initial commit");

    let branch_name = PathBuf::from("abc/def/ghi");
    let update_refs_config = update_refs::UpdateRefsConfig::new(&commit_hash, &branch_name);
    update_refs::update_refs(Some(&path.0), update_refs_config).unwrap();

    let checkout_config = checkout::CheckoutConfig::new(branch_name.to_str().unwrap().to_string());
    checkout::checkout_branch(Some(&path.0), &checkout_config).unwrap();

    let mut entries: Vec<DirEntry> = path
        .0
        .read_dir()
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            match entry.file_name().to_str() {
                Some(file_name) => {
                    if file_name == "not-git" {
                        None
                    } else {
                        Some(entry)
                    }
                }
                None => None,
            }
        })
        .collect();
    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    let file_1 = &entries[0];
    assert_eq!(file_1.file_name(), "file1");
    assert!(file_1.metadata().unwrap().is_file());
    let file_1_contents = fs::read(file_1.path()).unwrap();
    assert_eq!(file_1_contents, b"Test 1");

    let file_2 = &entries[1];
    assert_eq!(file_2.file_name(), "file2");
    assert!(file_2.metadata().unwrap().is_file());
    let file_2_contents = fs::read(file_2.path()).unwrap();
    assert_eq!(file_2_contents, b"Test 2");

    let file_3 = &entries[2];
    assert_eq!(file_3.file_name(), "tree1");
    assert!(file_3.metadata().unwrap().is_dir());

    let mut tree_1_entries: Vec<DirEntry> = file_3
        .path()
        .read_dir()
        .unwrap()
        .map(|entry| entry.unwrap())
        .collect();
    tree_1_entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    let file_3_1 = &tree_1_entries[0];
    assert_eq!(file_3_1.file_name(), "file3");
    assert!(file_3_1.metadata().unwrap().is_file());
    let file_3_1_contents = fs::read(file_3_1.path()).unwrap();
    assert_eq!(file_3_1_contents, b"Test 3");

    let file_3_2 = &tree_1_entries[1];
    assert_eq!(file_3_2.file_name(), "file4");
    assert!(file_3_2.metadata().unwrap().is_file());
    let file_3_2_contents = fs::read(file_3_2.path()).unwrap();
    assert_eq!(file_3_2_contents, b"Test 4");
}

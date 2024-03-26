use std::fs;

mod common;

#[test]
fn create_repo_init() {
    let branch_name = "test_branch_name";

    let path = common::setup();
    let head_file = path.0.join("not-git").join("HEAD");
    let packed_refs_file = path.0.join("not-git").join("packed-refs");
    let objects_dir = path.0.join("not-git").join("objects");

    assert!(!head_file.exists());
    assert!(!packed_refs_file.exists());
    assert!(!objects_dir.exists());

    let config = not_git::init::InitConfig::new(branch_name, path.0.to_str());
    not_git::init::create_directories(config).unwrap();

    assert!(head_file.exists());
    assert!(packed_refs_file.exists());
    assert!(objects_dir.exists());

    let head = fs::read_to_string(head_file).unwrap();
    assert_eq!(head, format!("ref: refs/heads/{}\n", branch_name));

    let packed_refs = fs::read_to_string(packed_refs_file).unwrap();
    assert_eq!(
        packed_refs,
        "# pack-refs with: peeled fully-peeled sorted\n"
    );
}

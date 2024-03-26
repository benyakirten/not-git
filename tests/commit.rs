use not_git::commit;

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
    // 2. Write some files
    // 3. Run command
    // 4. Examine files
}

#[test]
fn commit_error_if_head_ref_not_found() {
    let path = common::TestPath::new();

    let config = commit::CommitConfig::new("Test commit".to_string());
    let result = commit::commit(Some(&path.0), config);
    assert!(result.is_err());
}

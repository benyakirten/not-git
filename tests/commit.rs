use std::fs;
use std::path::PathBuf;

use not_git::objects::{ObjectFile, ObjectHash, ObjectType};
use not_git::utils::get_head_ref;
use not_git::{commit, commit_tree, init};

mod common;

#[test]
fn parent_commit_preserved_on_new_commit() {
    let path = common::TestPath::new();

    let init_config = init::InitConfig::new("main", path.to_optional_path());
    init::create_directories(init_config).unwrap();

    let tree_hash = common::create_valid_tree_hash(&path);
    let parent_config = commit_tree::CommitTreeConfig::new(&tree_hash, "message".to_string(), None);
    let parent_hash = commit_tree::create_commit(path.to_optional_path(), parent_config).unwrap();

    let commit_config = commit_tree::CommitTreeConfig::new(
        &tree_hash,
        "message".to_string(),
        Some(parent_hash.clone()),
    );
    let commit_hash = commit_tree::create_commit(path.to_optional_path(), commit_config).unwrap();

    let head_ref_path = path.join(&"not-git/refs/heads/main");
    fs::create_dir_all(head_ref_path.parent().unwrap()).unwrap();
    fs::write(head_ref_path, commit_hash.full_hash()).unwrap();

    let contents_1 = b"Test 1".to_vec();
    let file_name_1 = "file_1.txt";

    let contents_2 = b"Test 2".to_vec();
    let file_name_2 = "file_2.txt";

    let contents_3 = b"Test 3".to_vec();
    let file_name_3 = "file_3.txt";

    let files = vec![
        (file_name_1, contents_1),
        (file_name_2, contents_2),
        (file_name_3, contents_3),
    ];

    let commit_content = create_commit_with_files(&path, files, "Test commit");
    let mut commit_lines = commit_content.lines();

    let _tree = commit_lines.next().unwrap();
    let parent = commit_lines.next().unwrap();
    assert_eq!(parent, format!("parent {}", &parent_hash.full_hash()));
}

#[test]
fn commit_successful_without_parent_commit() {
    let path = common::TestPath::new();

    let init_config = init::InitConfig::new("main", path.to_optional_path());
    init::create_directories(init_config).unwrap();

    let contents_1 = b"Test 1".to_vec();
    let file_name_1 = "file_1.txt";

    let contents_2 = b"Test 2".to_vec();
    let file_name_2 = "file_2.txt";

    let contents_3 = b"Test 3".to_vec();
    let file_name_3 = "file_3.txt";

    let files = vec![
        (file_name_1, contents_1),
        (file_name_2, contents_2),
        (file_name_3, contents_3),
    ];

    let commit_content = create_commit_with_files(&path, files, "Test commit");
    let mut commit_lines = commit_content.lines();

    let tree = commit_lines.next().unwrap();
    let tree_commit = ObjectHash::new(&tree[5..]).unwrap();

    let author = commit_lines.next().unwrap();
    assert_eq!(author, "author Ben Horowitz <benyakir.horowitz@gmail.com>");

    let committer = commit_lines.next().unwrap();
    assert_eq!(
        committer,
        "committer Ben Horowitz <benyakir.horowitz@gmail.com>"
    );

    let message = commit_lines.next().unwrap();
    assert_eq!(message, "Test commit");

    let tree_object = ObjectFile::new(path.to_optional_path(), &tree_commit).unwrap();
    let tree_object = match tree_object {
        ObjectFile::Other(_) => panic!("Expected tree object"),
        ObjectFile::Tree(tree) => tree,
    };

    let tree_file_1 = &tree_object.contents[0];
    let tree_file_2 = &tree_object.contents[1];
    let tree_file_3 = &tree_object.contents[2];

    assert_eq!(tree_file_1.file_name, file_name_1);
    assert_eq!(tree_file_2.file_name, file_name_2);
    assert_eq!(tree_file_3.file_name, file_name_3);
}

#[test]
fn commit_error_if_head_ref_not_found() {
    let path = common::TestPath::new();

    let config = commit::CommitConfig::new("Test commit".to_string());
    let result = commit::commit(path.to_optional_path(), config);
    assert!(result.is_err());
}

fn create_commit_with_files(
    path: &common::TestPath,
    files: Vec<(&str, Vec<u8>)>,
    message: &str,
) -> String {
    for (file_name, contents) in files {
        fs::write(path.join(&file_name), contents).unwrap();
    }

    let commit_config = commit::CommitConfig::new(message.to_string());
    commit::commit(path.to_optional_path(), commit_config).unwrap();

    let head_ref = get_head_ref(path.to_optional_path()).unwrap();
    let head_path = PathBuf::from("not-git/refs/heads").join(head_ref);
    let head_path = path.join(&head_path);

    let head_hash = fs::read_to_string(head_path).unwrap();
    let head_hash = ObjectHash::new(&head_hash).unwrap();

    let head_commit = ObjectFile::new(path.to_optional_path(), &head_hash).unwrap();

    let head_commit = match head_commit {
        ObjectFile::Other(commit) => commit,
        ObjectFile::Tree(_) => panic!("Expected commit object"),
    };

    assert_eq!(head_commit.object_type, ObjectType::Commit);

    String::from_utf8(head_commit.contents).unwrap()
}

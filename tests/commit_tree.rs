use not_git::commit_tree;
use not_git::objects::{ObjectFile, ObjectHash, ObjectType, TreeObject};

mod common;

#[test]
fn commit_tree_error_if_tree_hash_invalid() {
    let path = common::TestPath::new();

    let tree_hash = "0123456789abcdef0123456789abcdef01234567";
    let tree_hash = ObjectHash::new(tree_hash).unwrap();
    let commit_config = commit_tree::CommitTreeConfig::new(&tree_hash, "message".to_string(), None);

    let result = commit_tree::create_commit(Some(&path.0), commit_config);
    assert!(result.is_err());
}

#[test]
fn commit_tree_error_if_parent_hash_present_but_invalid() {
    let path = common::TestPath::new();

    let tree_hash = create_valid_tree_hash(&path);
    let parent_tree_hash = "0123456789abcdef0123456789abcdef01234567";
    let parent_tree_hash = ObjectHash::new(parent_tree_hash).unwrap();

    let commit_config = commit_tree::CommitTreeConfig::new(
        &tree_hash,
        "message".to_string(),
        Some(parent_tree_hash),
    );

    let result = commit_tree::create_commit(Some(&path.0), commit_config);
    assert!(result.is_err());
}

#[test]
fn commit_tree_no_parent_if_no_parent_hash() {
    let path = common::TestPath::new();

    let tree_hash = create_valid_tree_hash(&path);
    let commit_config =
        commit_tree::CommitTreeConfig::new(&tree_hash, "test message".to_string(), None);

    let got = commit_tree::create_commit(Some(&path.0), commit_config).unwrap();
    assert_eq!(got.full_hash(), "06f5feac8532041dd7dc6ac23af6ae7cdfdc5d76");

    let object_file = ObjectFile::new(Some(&path.0), &got).unwrap();
    let object_file = match object_file {
        ObjectFile::Other(contents) => contents,
        ObjectFile::Tree(_) => panic!("Expected commit object"),
    };

    assert_eq!(object_file.object_type, ObjectType::Commit);

    let content = String::from_utf8(object_file.contents).unwrap();
    let mut lines = content.lines();

    let tree = lines.next().unwrap();
    assert_eq!(tree, format!("tree {}", tree_hash.full_hash()));

    let author = lines.next().unwrap();
    assert_eq!(author, "author Ben Horowitz <benyakir.horowitz@gmail.com>");

    let committer = lines.next().unwrap();
    assert_eq!(
        committer,
        "committer Ben Horowitz <benyakir.horowitz@gmail.com>"
    );

    let message = lines.next().unwrap();
    assert_eq!(message, "test message");
}

#[test]
fn commit_tree_has_parent_if_valid_parent_hash() {
    let path = common::TestPath::new();

    let tree_hash = create_valid_tree_hash(&path);
    let parent_tree_hash = common::write_tree(&path, vec![]);

    let commit_config = commit_tree::CommitTreeConfig::new(
        &tree_hash,
        "test message".to_string(),
        Some(parent_tree_hash.clone()),
    );

    let got = commit_tree::create_commit(Some(&path.0), commit_config).unwrap();
    assert_eq!(got.full_hash(), "fae075ab36729e330ea06385bfb4b0aa2e47d202");

    let object_file = ObjectFile::new(Some(&path.0), &got).unwrap();
    let object_file = match object_file {
        ObjectFile::Other(contents) => contents,
        ObjectFile::Tree(_) => panic!("Expected commit object"),
    };

    assert_eq!(object_file.object_type, ObjectType::Commit);

    let content = String::from_utf8(object_file.contents).unwrap();
    let mut lines = content.lines();

    let tree = lines.next().unwrap();
    assert_eq!(tree, format!("tree {}", tree_hash.full_hash()));

    let parent = lines.next().unwrap();
    assert_eq!(parent, format!("parent {}", parent_tree_hash.full_hash()));

    let author = lines.next().unwrap();
    assert_eq!(author, "author Ben Horowitz <benyakir.horowitz@gmail.com>");

    let committer = lines.next().unwrap();
    assert_eq!(
        committer,
        "committer Ben Horowitz <benyakir.horowitz@gmail.com>"
    );

    let message = lines.next().unwrap();
    assert_eq!(message, "test message");
}

fn create_valid_tree_hash(path: &common::TestPath) -> ObjectHash {
    let mut contents_1 = b"Test 1".to_vec();
    let object_hash_1 = common::write_object(&path, &ObjectType::Blob, &mut contents_1);

    let mut contents_2 = b"Test 2".to_vec();
    let object_hash_2 = common::write_object(&path, &ObjectType::Blob, &mut contents_2);

    let mut contents_3 = b"Test 1".to_vec();
    let object_hash_3 = common::write_object(&path, &ObjectType::Tree, &mut contents_3);

    let tree_objects: Vec<TreeObject> = vec![
        TreeObject::new(ObjectType::Blob, "file1".to_string(), object_hash_1.clone()),
        TreeObject::new(ObjectType::Blob, "file2".to_string(), object_hash_2.clone()),
        TreeObject::new(ObjectType::Tree, "tree1".to_string(), object_hash_3.clone()),
    ];

    common::write_tree(&path, tree_objects)
}

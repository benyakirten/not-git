use std::fs;

use not_git::objects::{ObjectFile, ObjectHash, ObjectType, TreeObject};

mod common;

#[test]
fn parse_blob_file() {
    let path = common::TestPath::new();
    let mut contents = b"Hello, World!".to_vec();
    let object_hash = common::write_object(&path, &ObjectType::Blob, &mut contents);

    let object_file = ObjectFile::new(Some(&path.0), &object_hash).unwrap();
    match object_file {
        ObjectFile::Other(contents) => {
            assert_eq!(contents.size, 13);
            assert_eq!(contents.contents, b"Hello, World!".to_vec());
            assert_eq!(contents.object_type, ObjectType::Blob);
        }
        _ => panic!(),
    };
}

#[test]
fn parse_object_tree_file() {
    let path = common::TestPath::new();

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

    let tree_hash = common::write_tree(&path, tree_objects);

    let tree_file = ObjectFile::new(Some(&path.0), &tree_hash).unwrap();
    match tree_file {
        ObjectFile::Tree(object_contents) => {
            assert_eq!(object_contents.object_type, ObjectType::Tree);

            let contents = object_contents.contents;
            assert_eq!(contents.len(), 3);

            let tree_object_1 = &contents[0];
            assert_eq!(tree_object_1.object_type, ObjectType::Blob);
            assert_eq!(tree_object_1.file_name, "file1");
            assert_eq!(tree_object_1.hash.full_hash(), object_hash_1.full_hash());

            let tree_object_2 = &contents[1];
            assert_eq!(tree_object_2.object_type, ObjectType::Blob);
            assert_eq!(tree_object_2.file_name, "file2");
            assert_eq!(tree_object_2.hash.full_hash(), object_hash_2.full_hash());

            let tree_object_3 = &contents[2];
            assert_eq!(tree_object_3.object_type, ObjectType::Tree);
            assert_eq!(tree_object_3.file_name, "tree1");
            assert_eq!(tree_object_3.hash.full_hash(), object_hash_3.full_hash());
        }
        _ => panic!(),
    };
}

#[test]
fn parse_fails_on_file_improper_format() {
    let path = common::TestPath::new();

    let hash = "0123456789abcdef0123456789abcdef01234567";
    let object_hash = ObjectHash::new(hash).unwrap();

    let file_path = path.join(&object_hash.path());

    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, b"improper format").unwrap();

    let object_file = ObjectFile::new(Some(&path.0), &object_hash);
    assert!(object_file.is_err());
}

#[test]
fn parse_fails_on_file_not_found() {
    let path = common::TestPath::new();

    let hash = "0123456789abcdef0123456789abcdef01234567";
    let object_hash = ObjectHash::new(hash).unwrap();
    let file_path = path.join(&object_hash.path());

    let object_file = ObjectFile::new(Some(&file_path), &object_hash);
    assert!(object_file.is_err());
}

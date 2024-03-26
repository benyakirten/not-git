use not_git::objects::{ObjectFile, ObjectType};

mod common;

#[test]
fn test_parse_blob_file() {
    let path = common::setup();
    let mut contents = b"Hello, World!".to_vec();
    let object_hash = common::write_object(&path, &ObjectType::Blob, &mut contents);

    let object_file = ObjectFile::new(Some(&path), &object_hash).unwrap();
    match object_file {
        ObjectFile::Other(contents) => {
            assert_eq!(contents.size, 13);
            assert_eq!(contents.contents, b"Hello, World!".to_vec());
            assert_eq!(contents.object_type, ObjectType::Blob);
        }
        _ => panic!(),
    }

    common::cleanup(path);
}

#[test]
fn test_parse_object_tree_file() {
    let path = common::setup();
    // TODO
    common::cleanup(path);
}

#[test]
fn test_parse_fails_on_invalid_file() {
    let path = common::setup();
    // TODO
    common::cleanup(path);
}

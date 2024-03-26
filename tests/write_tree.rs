use std::fs;

use not_git::{
    init,
    objects::{ObjectFile, ObjectType},
    write_tree,
};

mod common;

#[test]
fn test_write_tree_success() {
    let path = common::TestPath::new();
    init::create_directories(init::InitConfig::new("main", path.to_str())).unwrap();

    let contents_1 = b"Test 1".to_vec();
    let file_name_1 = "file1".to_string();

    fs::write(path.join(&file_name_1), &contents_1).unwrap();

    let contents_2 = b"Test 2".to_vec();
    let file_name_2 = "file2".to_string();

    fs::write(path.join(&file_name_2), &contents_2).unwrap();

    let contents_3 = b"Test 3".to_vec();
    let file_name_3 = "file3".to_string();

    let sub_dir_name = "sub_dir";
    let sub_dir = path.join(&sub_dir_name);
    fs::create_dir(&sub_dir).unwrap();
    fs::write(sub_dir.join(&file_name_3), &contents_3).unwrap();

    let path_str = path.0.to_str();
    let tree_hash = write_tree::write_tree(path_str).unwrap();

    assert_eq!(
        tree_hash.full_hash(),
        "7abecc23db04e758fd76dd97a97901597ced79cf"
    );

    let tree_file = ObjectFile::new(Some(&path.0), &tree_hash).unwrap();
    match tree_file {
        ObjectFile::Tree(tree) => {
            assert_eq!(tree.object_type, ObjectType::Tree);
            assert_eq!(tree.contents.len(), 3);

            let file1 = &tree.contents[0];
            assert_eq!(file1.object_type, ObjectType::Blob);
            assert_eq!(file1.file_name, file_name_1);

            let file_hash_1 = &file1.hash;
            let object_file_1 = ObjectFile::new(Some(&path.0), file_hash_1).unwrap();
            match object_file_1 {
                ObjectFile::Other(blob) => {
                    assert_eq!(blob.object_type, ObjectType::Blob);
                    assert_eq!(blob.contents, contents_1);
                }
                _ => unreachable!(),
            }

            let file2 = &tree.contents[1];
            assert_eq!(file2.object_type, ObjectType::Blob);

            let file_hash_2 = &file2.hash;
            let object_file_2 = ObjectFile::new(Some(&path.0), file_hash_2).unwrap();
            match object_file_2 {
                ObjectFile::Other(blob) => {
                    assert_eq!(blob.object_type, ObjectType::Blob);
                    assert_eq!(blob.contents, contents_2);
                }
                _ => unreachable!(),
            }

            let file3 = &tree.contents[2];
            assert_eq!(file3.object_type, ObjectType::Tree);
            assert_eq!(file3.file_name, sub_dir_name);

            let sub_dir_hash = &file3.hash;
            let sub_dir_hash = ObjectFile::new(Some(&path.0), sub_dir_hash).unwrap();
            match sub_dir_hash {
                ObjectFile::Tree(sub_tree) => {
                    assert_eq!(sub_tree.object_type, ObjectType::Tree);
                    assert_eq!(sub_tree.contents.len(), 1);

                    let sub_file = &sub_tree.contents[0];
                    assert_eq!(sub_file.object_type, ObjectType::Blob);
                    assert_eq!(sub_file.file_name, file_name_3);

                    let sub_file_hash = &sub_file.hash;
                    let object_file_3 = ObjectFile::new(Some(&path.0), sub_file_hash).unwrap();
                    match object_file_3 {
                        ObjectFile::Other(blob) => {
                            assert_eq!(blob.object_type, ObjectType::Blob);
                            assert_eq!(blob.contents, contents_3);
                        }
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}

use not_git::{hash_object, objects::ObjectType, utils::decode_file};

mod common;

#[test]
fn hash_object_encodes_file_to_zlib_with_header() {
    let path = common::TestPath::new();

    let contents = b"hello world";
    let got_hash = hash_object::hash_and_write_object(
        Some(&path.0),
        &ObjectType::Blob,
        &mut contents.to_vec(),
    )
    .unwrap();

    let file_path = path.join(&got_hash.path());
    assert!(file_path.exists());

    let decoded_content = decode_file(file_path).unwrap();

    let split_index = decoded_content.iter().position(|&x| x == 0).unwrap();
    let (header, body) = decoded_content.split_at(split_index + 1);

    assert_eq!(header, b"blob 11\0");
    assert_eq!(body, contents);
}

// Almost all errors are OS-based, and I'm not sure how to trigger them for a test

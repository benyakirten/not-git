use std::io::Cursor;

use not_git::{
    objects::{ObjectFile, ObjectType},
    packfile,
};

mod common;

#[test]
fn decode_undeltified_data_creates_object_from_zlib_compressed_data() {
    let path = common::TestPath::new();

    let file_1_contents = b"Hello, world!";
    let mut data = common::encode_to_zlib(file_1_contents);

    let next_metadata = b"Next metadata";
    data.extend(next_metadata);

    let mut cursor = Cursor::new(data.as_slice());

    let (data, object_hash, object_type) =
        packfile::decode_undeltified_data(&path.0, ObjectType::Blob, &mut cursor).unwrap();

    assert_eq!(data, file_1_contents);
    assert_eq!(object_type, ObjectType::Blob);

    let object_file = ObjectFile::new(path.to_optional_path(), &object_hash).unwrap();
    match object_file {
        ObjectFile::Other(contents) => {
            assert_eq!(contents.contents, file_1_contents);
            assert_eq!(contents.size, file_1_contents.len());
            assert_eq!(contents.object_type, ObjectType::Blob);
        }
        _ => panic!("Expected ObjectFile::Other"),
    }
}

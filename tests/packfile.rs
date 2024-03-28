use std::io::Cursor;

use common::RenderContent;
use not_git::{
    objects::{ObjectFile, ObjectHash, ObjectType},
    packfile::{
        self, CopyInstruction, DeltaInstruction, InsertInstruction, PackfileObject,
        PackfileObjectType,
    },
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

#[test]
fn decode_undeltified_data_errors_if_data_not_zlib_encoded() {
    let path = common::TestPath::new();

    let mut data = b"Hello, world!".to_vec();

    let next_metadata = b"Next metadata";
    data.extend(next_metadata);

    let mut cursor = Cursor::new(data.as_slice());
    let got = packfile::decode_undeltified_data(&path.0, ObjectType::Blob, &mut cursor);

    assert!(got.is_err());
}

#[test]
fn read_obj_offset_data_creates_new_object_file() {
    let path = common::TestPath::new();

    let packfile_object = common::packfile_file_object();
    let packfile_objects = vec![packfile_object];
    let packfile_objects = packfile_objects.as_slice();

    let copy_instruction = CopyInstruction { offset: 3, size: 5 };
    let insert_instruction = InsertInstruction { size: 4 };

    let instructions = vec![
        DeltaInstruction::Copy(copy_instruction),
        DeltaInstruction::Insert(insert_instruction),
    ];

    let delta_data = b"abcdefghijklmnopqrstuvwxyz".to_vec();

    let offset_instruction = common::OffsetDeltaData::new(delta_data, instructions, 10);

    let mut data = b"123456789012345678901234567890".to_vec();
    data.extend(offset_instruction.render());

    let mut cursor = Cursor::new(data.as_slice());
    cursor.set_position(30);

    let got = packfile::read_obj_offset_data(&path.0, packfile_objects, &mut cursor).unwrap();
}

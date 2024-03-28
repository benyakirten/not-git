use std::io::Cursor;

use not_git::objects::{ObjectFile, ObjectType};
use not_git::packfile;

mod common;
use common::RenderContent;

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
fn read_obj_offset_delta_creates_new_object_file() {
    let path = common::TestPath::new();

    let packfile_object = common::packfile_file_object();
    let packfile_objects = vec![packfile_object];
    let packfile_objects = packfile_objects.as_slice();

    let copy_instruction = common::TestCopyInstruction { offset: 3, size: 5 };
    let insert_instruction = common::TestInsertInstruction {
        content: b"testcontent".to_vec(),
    };

    let instructions = vec![
        common::TestDeltaInstruction::Copy(copy_instruction),
        common::TestDeltaInstruction::Insert(insert_instruction),
    ];

    let offset_instruction = common::OffsetDeltaData::new(instructions, 10);

    let mut data = b"123456789012345678901234567890".to_vec();
    data.extend(offset_instruction.render());

    let mut cursor = Cursor::new(data.as_slice());
    cursor.set_position(30);

    let (content, hash, object_type) =
        packfile::read_obj_offset_data(&path.0, packfile_objects, &mut cursor).unwrap();

    let content = String::from_utf8(content).unwrap();
    assert_eq!(content, "lo, wtestcontent");
    assert_eq!(object_type, ObjectType::Blob);
    let object_file = ObjectFile::new(path.to_optional_path(), &hash).unwrap();
    match object_file {
        ObjectFile::Other(contents) => {
            assert_eq!(contents.contents, content.as_bytes());
            assert_eq!(contents.size, content.len());
            assert_eq!(contents.object_type, ObjectType::Blob);
        }
        _ => panic!("Expected ObjectFile::Other"),
    };
}

#[test]
fn read_obj_ref_delta_creates_new_object_file() {
    let path = common::TestPath::new();

    let packfile_object = common::packfile_file_object();
    let hash = packfile_object.file_hash.clone();

    let packfile_objects = vec![packfile_object];
    let packfile_objects = packfile_objects.as_slice();

    let copy_instruction = common::TestCopyInstruction { offset: 3, size: 5 };
    let insert_instruction = common::TestInsertInstruction {
        content: b"testcontent".to_vec(),
    };

    let instructions = vec![
        common::TestDeltaInstruction::Copy(copy_instruction),
        common::TestDeltaInstruction::Insert(insert_instruction),
    ];

    let offset_instruction = common::RefDeltaData::new(instructions, hash);

    let mut data = b"123456789012345678901234567890".to_vec();
    data.extend(offset_instruction.render());

    let mut cursor = Cursor::new(data.as_slice());
    cursor.set_position(30);

    let (content, hash, object_type) =
        packfile::read_obj_ref_data(&path.0, packfile_objects, &mut cursor).unwrap();

    let content = String::from_utf8(content).unwrap();
    assert_eq!(content, "lo, wtestcontent");
    assert_eq!(object_type, ObjectType::Blob);
    let object_file = ObjectFile::new(path.to_optional_path(), &hash).unwrap();
    match object_file {
        ObjectFile::Other(contents) => {
            assert_eq!(contents.contents, content.as_bytes());
            assert_eq!(contents.size, content.len());
            assert_eq!(contents.object_type, ObjectType::Blob);
        }
        _ => panic!("Expected ObjectFile::Other"),
    };
}

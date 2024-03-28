use std::io::{BufRead, Cursor, ErrorKind, Read};

use super::{ObjectHash, ObjectType};

#[derive(Debug)]
pub struct TreeObject {
    pub object_type: ObjectType,
    pub file_name: String,
    pub hash: ObjectHash,
}

impl TreeObject {
    /// Parse a tree object from an object that has already been decoded from zlib.
    /// It shoudl already have the header removed.
    pub fn from_object(body: &[u8]) -> Result<Vec<Self>, anyhow::Error> {
        let mut cursor = Cursor::new(body);
        let mut tree_files = vec![];

        loop {
            let tree_file = match read_next_tree_file(&mut cursor)? {
                None => break,
                Some(tree_file) => tree_file,
            };

            tree_files.push(tree_file);
        }

        Ok(tree_files)
    }

    pub fn new(object_type: ObjectType, file_name: String, hash: ObjectHash) -> Self {
        Self {
            object_type,
            file_name,
            hash,
        }
    }
}

fn read_next_tree_file(cursor: &mut Cursor<&[u8]>) -> Result<Option<TreeObject>, anyhow::Error> {
    let mut mode_file_name = vec![];

    match cursor.read_until(0, &mut mode_file_name) {
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
        _ => {}
    };

    if mode_file_name.is_empty() {
        return Ok(None);
    }

    let readable_mode_file_name = String::from_utf8(mode_file_name)?;
    let (mode, file_name) = readable_mode_file_name
        .split_once(' ')
        .ok_or_else(|| anyhow::anyhow!("Invalid tree object: Unable to find mode and file name"))?;

    let file_name = file_name.replace('\0', "");

    let mut hash_bytes = vec![0; 20];
    cursor.read_exact(&mut hash_bytes)?;
    let hash = ObjectHash::from_bytes(hash_bytes.as_slice())?;

    let object_type = ObjectType::from_mode(mode)?;

    let tree_object = TreeObject::new(object_type, file_name, hash);
    Ok(Some(tree_object))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_object_success() {
        let mut file_contents: Vec<u8> = vec![];

        let file_name1 = "file1";
        let file_mode1 = "100644";
        let file_hash1: [u8; 20] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        ];
        let hex_file_hash1 = hex::encode(file_hash1);

        let file_name2 = "file2";
        let file_mode2 = "040000";
        let file_hash2: [u8; 20] = [
            10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160, 170, 180, 190,
            200,
        ];
        let hex_file_hash2 = hex::encode(file_hash2);

        let mode_file_name1 = format!("{} {}\0", file_mode1, file_name1);
        let mode_file_name1 = mode_file_name1.as_bytes();
        file_contents.append(&mut mode_file_name1.to_vec());
        file_contents.append(&mut file_hash1.to_vec());

        let mode_file_name2 = format!("{} {}\0", file_mode2, file_name2);
        let mode_file_name2 = mode_file_name2.as_bytes();
        file_contents.append(&mut mode_file_name2.to_vec());
        file_contents.append(&mut file_hash2.to_vec());

        let tree_objects = TreeObject::from_object(&file_contents).unwrap();

        assert_eq!(tree_objects.len(), 2);
        let tree_object1 = &tree_objects[0];

        assert_eq!(tree_object1.object_type, ObjectType::Blob);
        assert_eq!(tree_object1.file_name, file_name1);
        assert_eq!(tree_object1.hash.full_hash(), hex_file_hash1);

        let tree_object2 = &tree_objects[1];
        assert_eq!(tree_object2.object_type, ObjectType::Tree);
        assert_eq!(tree_object2.file_name, file_name2);
        assert_eq!(tree_object2.hash.full_hash(), hex_file_hash2);
    }

    #[test]
    fn from_object_error_if_invalid_object() {
        let mut file_contents: Vec<u8> = vec![];

        let file_name1 = "file1";
        let file_mode1 = "100644";

        let mode_file_name1 = format!("{} {}\0", file_mode1, file_name1);
        let mode_file_name1 = mode_file_name1.as_bytes();
        file_contents.append(&mut mode_file_name1.to_vec());

        let tree_object = TreeObject::from_object(&file_contents);
        assert!(tree_object.is_err());
    }
}

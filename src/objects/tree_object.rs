use std::io::{Cursor, ErrorKind};

use super::{ObjectHash, ObjectType};

#[derive(Debug)]
pub struct TreeObject {
    pub object_type: ObjectType,
    pub file_name: String,
    pub hash: ObjectHash,
}

impl TreeObject {
    pub fn from_object(body: [u8]) -> Result<Vec<Self>, anyhow::Error> {
        let mut cursor = Cursor::new(body);
        let mut tree_files = vec![];

        loop {
            let tree_file = read_next_tree_file(&mut cursor)?;
            if tree_file.is_none() {
                break;
            }
            let tree_file = tree_file.unwrap();
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
    }

    let (mode, file_name) = String::from_utf8(mode_file_name)?
        .split_once(' ')
        .ok_or_else(|| anyhow::anyhow!("Invalid tree object: Unable to find mode and file name"))?;

    let file_name = file_name.replace('\0', "");

    let mut hash_bytes = vec![0; 20];
    cursor.read_exact(&mut hash_bytes)?;
    let hash = ObjectHash::from_bytes(hash_bytes)?;

    let object_type = ObjectType::from_mode(mode)?;

    let tree_object = TreeObject::new(object_type, file_name, hash);
    Ok(Some(tree_object))
}

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use flate2::write::ZlibEncoder;
use flate2::Compression;
use hex::ToHex;
use not_git::objects::{ObjectHash, ObjectType, TreeObject};
use not_git::packfile::{CopyInstruction, DeltaInstruction, InsertInstruction, PackfileObject};
use sha1::{Digest, Sha1};

// TODO: Figure out why some functions are marked as not being used.

pub struct TestPath(pub PathBuf);

impl TestPath {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let test_dir_name = uuid::Uuid::new_v4().to_string();
        let test_dir = [".test", &test_dir_name].iter().collect::<PathBuf>();
        if !test_dir.exists() {
            fs::create_dir_all(&test_dir).unwrap();
        }

        Self(test_dir)
    }

    pub fn join<P>(&self, path: &P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        self.0.join(path)
    }

    #[allow(dead_code)]
    pub fn to_str(&self) -> Option<&str> {
        self.0.to_str()
    }

    #[allow(dead_code)]
    pub fn to_optional_path(&self) -> Option<&PathBuf> {
        Some(&self.0)
    }
}

impl Drop for TestPath {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[allow(dead_code)]
pub fn write_object(
    path: &TestPath,
    object_type: &ObjectType,
    contents: &mut Vec<u8>,
) -> ObjectHash {
    let mut hasher = Sha1::new();
    hasher.update(&contents);
    let hash: String = hasher.finalize().encode_hex();
    let hash = ObjectHash::new(&hash).unwrap();

    let mut header = format!("{} {}\0", object_type.as_str(), contents.len())
        .as_bytes()
        .to_vec();

    let mut contents = contents.to_vec();
    header.append(&mut contents);

    let encoded_contents = encode_to_zlib(header.as_slice());
    let object_path = path.join(&hash.path());

    fs::create_dir_all(object_path.parent().unwrap()).unwrap();
    fs::write(object_path, encoded_contents).unwrap();

    hash
}

#[allow(dead_code)]
pub fn write_tree(path: &TestPath, tree: Vec<TreeObject>) -> ObjectHash {
    let mut tree_contents = vec![];
    for tree_object in tree {
        let mode_file_name = format!(
            "{} {}\0",
            tree_object.object_type.to_mode(),
            tree_object.file_name
        );
        tree_contents.extend(mode_file_name.as_bytes());

        let hash = hex::decode(tree_object.hash.full_hash()).unwrap();
        tree_contents.extend(hash);
    }

    write_object(path, &ObjectType::Tree, &mut tree_contents)
}

#[allow(dead_code)]
pub fn write_commit(
    path: &TestPath,
    tree_hash: &ObjectHash,
    parent_hash: Option<&ObjectHash>,
    message: &str,
) -> ObjectHash {
    let mut commit_contents = vec![];
    writeln!(&mut commit_contents, "tree {}", tree_hash.full_hash()).unwrap();

    if let Some(parent_hash) = parent_hash {
        writeln!(&mut commit_contents, "parent {}", parent_hash.full_hash()).unwrap();
    }

    writeln!(
        &mut commit_contents,
        "author Ben Horowitz <benyakir.horowitz@gmail.com>"
    )
    .unwrap();

    writeln!(
        &mut commit_contents,
        "committer Ben Horowitz <benyakir.horowitz@gmail.com>"
    )
    .unwrap();

    writeln!(&mut commit_contents, "{}", message).unwrap();

    write_object(path, &ObjectType::Commit, &mut commit_contents)
}

#[allow(dead_code)]
pub fn create_valid_tree_hash(path: &TestPath) -> ObjectHash {
    let mut contents_1 = b"Test 1".to_vec();
    let object_hash_1 = write_object(path, &ObjectType::Blob, &mut contents_1);

    let mut contents_2 = b"Test 2".to_vec();
    let object_hash_2 = write_object(path, &ObjectType::Blob, &mut contents_2);

    let mut child_contents_1 = b"Test 3".to_vec();
    let object_hash_3 = write_object(path, &ObjectType::Blob, &mut child_contents_1);

    let mut child_contents_2 = b"Test 4".to_vec();
    let object_hash_4 = write_object(path, &ObjectType::Blob, &mut child_contents_2);

    let child_tree_objects: Vec<TreeObject> = vec![
        TreeObject::new(ObjectType::Blob, "file3".to_string(), object_hash_3.clone()),
        TreeObject::new(ObjectType::Blob, "file4".to_string(), object_hash_4.clone()),
    ];
    let child_tree_hash = write_tree(path, child_tree_objects);

    let tree_objects: Vec<TreeObject> = vec![
        TreeObject::new(ObjectType::Blob, "file1".to_string(), object_hash_1.clone()),
        TreeObject::new(ObjectType::Blob, "file2".to_string(), object_hash_2.clone()),
        TreeObject::new(
            ObjectType::Tree,
            "tree1".to_string(),
            child_tree_hash.clone(),
        ),
    ];

    write_tree(path, tree_objects)
}

#[allow(dead_code)]
pub fn create_packfile_from_objects(_instructions: Vec<PackfileObject>) -> Vec<u8> {
    todo!()
}

#[allow(dead_code)]
pub fn create_discover_references_response() {
    todo!()
}

#[allow(dead_code)]
pub fn create_packfile_header(num_instruction: usize) -> Vec<u8> {
    let mut header = vec![];
    header.extend(b"0008NAK\n");
    header.extend(b"PACK");
    header.extend(&[0, 0, 0, 2]);
    header.extend(&(num_instruction as u32).to_be_bytes());
    header
}

#[allow(dead_code)]
pub fn render_delta_instruction_bytes(instructions: Vec<(DeltaInstruction, Vec<u8>)>) -> Vec<u8> {
    for (instruction, contents) in instructions {
        let instruction_bytes = match instruction {
            DeltaInstruction::Copy(copy_instruction) => {
                render_copy_delta_instruction_bytes(copy_instruction)
            }
            DeltaInstruction::Insert(insert_instruction) => {
                render_insert_delta_instruction_bytes(insert_instruction)
            }
            _ => {
                panic!("DeltaInstruction::End not supported. The cycle will end when the last instruction is reached.")
            }
        };
    }

    todo!()
}

#[allow(dead_code)]
pub fn render_copy_delta_instruction_bytes(instruction: CopyInstruction) -> Vec<u8> {
    if instruction.offset > 0xFFFFFFFF {
        panic!("Offset must be less than 2^32")
    }

    if instruction.size > 0xFFFFFF {
        panic!("Size must be less than 2^24")
    }

    let mut following_bytes: Vec<u8> = vec![];

    let mut leading_byte = 0b1000_0000;

    // The order will be:
    // 1. Instruction byte
    // 2. Offset bytes
    // 3. Size bytes
    // The tricky thing is that the instruction byte's bits minus the most significant bit
    // are in reverse order. As in 0b1010_1010 means that bytes `0b010` (first three digits minus the MSB)
    // correspond to the size, and `1010` corresponds to the offset, but then you need to flip the order.
    // So the offset is still `010`, but the size is now `0101`.`What this means is that the first byte
    // you read corresponds to the first byte of the offset, the second byte the third byte of the offset
    // the intermediate byte will be `0b0000_0000`. So if you have the size bytes as `0b1010_1010 0b0101_0101`
    // then the final value of the offset will be `0b0000_0000 0b1010_1010 0b0000_0000 0b0101_0101`
    let mut total_bytes = instruction.offset.to_be_bytes().to_vec();
    let size_bytes = instruction.size.to_be_bytes().to_vec();
    total_bytes.extend(size_bytes);

    for (index, &byte) in total_bytes.iter().enumerate() {
        let mut bit_validity: u8 = if byte == 0 {
            0
        } else {
            following_bytes.push(byte);
            1
        };

        // Shift the bit to the left based on how far we've progressed through the bytes
        // As we progress further, we will shift further, meaning that if
        bit_validity <<= index;
        leading_byte |= bit_validity;
    }

    let mut final_bytes = vec![leading_byte];
    final_bytes.extend(following_bytes);

    final_bytes
}

#[allow(dead_code)]
pub fn render_insert_delta_instruction_bytes(instruction: InsertInstruction) -> Vec<u8> {
    // If only copy instructions were this simple
    if instruction.size >= 128 {
        panic!("Size must be less than 128")
    }

    vec![instruction.size]
}

pub fn encode_to_zlib(contents: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&contents).unwrap();
    encoder.finish().unwrap()
}

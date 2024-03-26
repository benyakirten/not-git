use std::path::PathBuf;
use std::sync::Once;
use std::{fs, io::Read};

use flate2::bufread::ZlibEncoder;
use hex::ToHex;
use not_git::objects::{ObjectHash, ObjectType, TreeObject};
use sha1::{Digest, Sha1};

// TODO: Figure out if we can make this into a macro

static CLEAR_TEST_DIR: Once = Once::new();

pub fn setup() -> PathBuf {
    CLEAR_TEST_DIR.call_once(|| {
        let test_dir = PathBuf::from(".test");
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).unwrap();
        }
    });

    let test_dir_name = uuid::Uuid::new_v4().to_string();
    let test_dir = [".test", &test_dir_name].iter().collect::<PathBuf>();
    if !test_dir.exists() {
        fs::create_dir_all(&test_dir).unwrap();
    }

    test_dir
}

pub fn cleanup(path: PathBuf) {
    fs::remove_dir_all(path).unwrap();
}

pub fn write_object(
    path: &PathBuf,
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

    let mut encoded_contents: Vec<u8> = vec![];
    let mut encoder = ZlibEncoder::new(header.as_slice(), flate2::Compression::default());
    encoder.read_to_end(&mut encoded_contents).unwrap();

    let object_path = path.join(hash.path());

    fs::create_dir_all(object_path.parent().unwrap()).unwrap();
    fs::write(object_path, encoded_contents).unwrap();

    hash
}

pub fn write_tree(path: &PathBuf, tree: Vec<TreeObject>) -> ObjectHash {
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

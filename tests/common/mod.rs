use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io::Read};

use flate2::bufread::ZlibEncoder;
use hex::ToHex;
use not_git::objects::{ObjectHash, ObjectType, TreeObject};
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

    let mut encoded_contents: Vec<u8> = vec![];
    let mut encoder = ZlibEncoder::new(header.as_slice(), flate2::Compression::default());
    encoder.read_to_end(&mut encoded_contents).unwrap();

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

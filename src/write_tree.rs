use std::path::PathBuf;
use std::{env, fs};

use crate::hash_object;
use crate::objects::{ObjectHash, ObjectType};

enum TreeFileType {
    Tree(Vec<TreeFile>),
    Other(ObjectType, PathBuf),
    Error(anyhow::Error),
}

struct TreeFile {
    object_type: ObjectType,
    file_name: String,
    hash: String,
}

impl TreeFile {
    fn new(object_type: ObjectType, file_name: String, hash: String) -> Self {
        Self {
            object_type,
            file_name,
            hash,
        }
    }
}

pub fn write_tree_command(_: &[String]) -> Result<(), anyhow::Error> {
    let hash = create_tree(None)?;
    println!("{}", hash.full_hash());

    Ok(())
}

pub fn create_tree(path: Option<&str>) -> Result<ObjectHash, anyhow::Error> {
    let path = match path {
        None => env::current_dir()?,
        Some(path) => PathBuf::from(path),
    };

    let base_path = &PathBuf::from(&path);
    let mut root_tree = build_tree_from_path(base_path, path)?;

    let hash = hash_tree(base_path, &mut root_tree)?;
    let hash = ObjectHash::new(&hash)?;

    Ok(hash)
}

fn build_tree_from_path(
    base_path: &PathBuf,
    path: PathBuf,
) -> Result<Vec<TreeFile>, anyhow::Error> {
    let mut tree_files: Vec<TreeFile> = Vec::new();

    for entry in path.read_dir()? {
        let entry = entry?;

        let object_type = ObjectType::from_entry(&entry)?;
        let file_name = entry
            .file_name()
            .to_str()
            .ok_or_else(|| {
                anyhow::anyhow!(format!(
                    "File {:?} name cannot be converted to utf-8",
                    entry.file_name()
                ))
            })?
            .to_string();

        if file_name == "target" || file_name == ".git" || file_name == "not-git" {
            continue;
        }

        let tree_file_type = match object_type {
            ObjectType::Tree => match build_tree_from_path(base_path, entry.path()) {
                Ok(tree_file) => TreeFileType::Tree(tree_file),
                Err(e) => TreeFileType::Error(e),
            },
            _ => TreeFileType::Other(object_type.clone(), entry.path()),
        };

        let hash = match tree_file_type {
            TreeFileType::Error(e) => return Err(e),
            TreeFileType::Other(object_type, path) => {
                let mut file_contents = fs::read(path)?;
                let hash = hash_object::hash_and_write_object(
                    Some(base_path),
                    &object_type,
                    &mut file_contents,
                )?;
                hash.full_hash()
            }
            TreeFileType::Tree(mut tree_files) => hash_tree(base_path, &mut tree_files)?,
        };

        let tree_file = TreeFile::new(object_type, file_name, hash);
        tree_files.push(tree_file);
    }

    Ok(tree_files)
}

fn hash_tree(base_path: &PathBuf, tree_files: &mut Vec<TreeFile>) -> Result<String, anyhow::Error> {
    tree_files.sort_by(|a, b| a.file_name.cmp(&b.file_name));

    let mut tree_content = Vec::new();
    for tree_file in tree_files {
        let file_type = tree_file.object_type.to_mode();
        let file_name = &tree_file.file_name;
        let hash = &tree_file.hash;

        if hash.len() != 40 {
            return Err(anyhow::anyhow!(
                "Invalid tree object: hash must be exactly 40 characters long"
            ));
        }

        let mut hash_bytes: Vec<u8> = hash
            .as_bytes()
            .chunks(2)
            .filter_map(|chunk| {
                let hex = std::str::from_utf8(chunk);
                if hex.is_err() {
                    return None;
                }

                let byte = u8::from_str_radix(hex.unwrap(), 16);
                if byte.is_err() {
                    return None;
                }

                Some(byte.unwrap())
            })
            .collect();

        if hash_bytes.len() != 20 {
            return Err(anyhow::anyhow!(
                "Invalid tree object: hash cannot be parsed correctly"
            ));
        }

        let line = format!("{} {}", file_type, file_name);
        tree_content.append(&mut line.as_bytes().to_vec());

        tree_content.push(0);
        tree_content.append(&mut hash_bytes);
    }

    let hash =
        hash_object::hash_and_write_object(Some(base_path), &ObjectType::Tree, &mut tree_content)?;
    Ok(hash.full_hash())
}

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
    sha: String,
}

impl TreeFile {
    fn new(object_type: ObjectType, file_name: String, sha: String) -> Self {
        Self {
            object_type,
            file_name,
            sha,
        }
    }
}

pub fn write_tree(path: Option<&str>) -> Result<ObjectHash, anyhow::Error> {
    let path = match path {
        None => env::current_dir()?,
        Some(path) => PathBuf::from(path),
    };
    let mut root_tree = build_tree_from_path(path)?;

    let sha = hash_tree(&mut root_tree)?;
    let sha = ObjectHash::new(&sha)?;

    Ok(sha)
}

fn build_tree_from_path(path: PathBuf) -> Result<Vec<TreeFile>, anyhow::Error> {
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
            ObjectType::Tree => match build_tree_from_path(entry.path()) {
                Ok(tree_file) => TreeFileType::Tree(tree_file),
                Err(e) => TreeFileType::Error(e),
            },
            _ => TreeFileType::Other(object_type.clone(), entry.path()),
        };

        let sha = match tree_file_type {
            TreeFileType::Error(e) => return Err(e),
            TreeFileType::Other(object_type, path) => {
                let mut file_contents = fs::read(path)?;
                let hash = hash_object::hash_and_write_object(&object_type, &mut file_contents)?;
                hash.full_hash()
            }
            TreeFileType::Tree(mut tree_files) => hash_tree(&mut tree_files)?,
        };

        let tree_file = TreeFile::new(object_type, file_name, sha);
        tree_files.push(tree_file);
    }

    Ok(tree_files)
}

fn hash_tree(tree_files: &mut Vec<TreeFile>) -> Result<String, anyhow::Error> {
    tree_files.sort_by(|a, b| a.file_name.cmp(&b.file_name));

    let mut tree_content = Vec::new();
    for tree_file in tree_files {
        let file_type = tree_file.object_type.to_mode();
        let file_name = &tree_file.file_name;
        let sha = &tree_file.sha;

        if sha.len() != 40 {
            return Err(anyhow::anyhow!(
                "Invalid tree object: sha must be exactly 40 characters long"
            ));
        }

        let mut sha_bytes: Vec<u8> = sha
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

        if sha_bytes.len() != 20 {
            return Err(anyhow::anyhow!(
                "Invalid tree object: sha hex cannot be parsed correctly"
            ));
        }

        let line = format!("{} {}", file_type, file_name);
        tree_content.append(&mut line.as_bytes().to_vec());

        tree_content.push(0);
        tree_content.append(&mut sha_bytes);
    }

    let hash = hash_object::hash_and_write_object(&ObjectType::Tree, &mut tree_content)?;
    Ok(hash.full_hash())
}

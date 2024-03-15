use std::{env, path::PathBuf};

use crate::ls_tree::FileType;
use crate::{hash_object, utils};

enum TreeFileType {
    Tree(Vec<TreeFile>),
    Other(FileType, PathBuf),
    Error(anyhow::Error),
}

struct TreeFile {
    file_type: FileType,
    file_name: String,
    sha: String,
}

pub fn write(_: &[String]) -> Result<(), anyhow::Error> {
    let path = env::current_dir()?;
    build_tree_from_path(path)?;
    Ok(())
}

fn build_tree_from_path(path: PathBuf) -> Result<Vec<TreeFile>, anyhow::Error> {
    let mut tree_files: Vec<TreeFile> = Vec::new();

    for entry in path.read_dir()? {
        let entry = entry?;
        entry.metadata()?;
        let file_type = FileType::from_entry(&entry)?;
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

        let tree_file_type = match file_type {
            FileType::Tree => match build_tree_from_path(entry.path()) {
                Ok(tree_file) => TreeFileType::Tree(tree_file),
                Err(e) => TreeFileType::Error(e),
            },
            _ => TreeFileType::Other(file_type.clone(), entry.path()),
        };

        let sha = match tree_file_type {
            TreeFileType::Error(_) => "".to_string(),
            TreeFileType::Other(_, path) => {
                let mut file_contents = utils::read_from_file(path)?;
                let hash = hash_object::hash_and_write(&mut file_contents)?;
                hash.full_hash()
            }
            TreeFileType::Tree(mut tree_files) => hash_tree(&mut tree_files)?,
        };

        tree_files.push(TreeFile {
            file_type,
            file_name: file_name.to_string(),
            sha,
        });
    }

    Ok(tree_files)
}

fn hash_tree(tree_files: &mut Vec<TreeFile>) -> Result<String, anyhow::Error> {
    tree_files.sort_by(|a, b| a.file_name.cmp(&b.file_name));

    let mut tree_content = Vec::new();
    for tree_file in tree_files {
        let file_type = tree_file.file_type.to_number_string();
        let file_name = &tree_file.file_name;
        let sha = &tree_file.sha;

        let line = format!("{} {}\0{}", file_type, sha, file_name);
        tree_content.append(&mut line.as_bytes().to_vec());
    }

    let hash = hash_object::hash_and_write(&mut tree_content)?;
    Ok(hash.full_hash())
}

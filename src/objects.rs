use std::{os::unix::fs::PermissionsExt, path::PathBuf, str::FromStr};

use anyhow::Context;

use crate::{
    file_hash::FileHash,
    ls_tree::{self, TreeFile},
    utils::{decode_file, split_at_empty_byte},
};

#[derive(Debug, Clone)]
pub enum ObjectType {
    Blob,
    Tree,
    Executable,
    Symlink,
    Commit,
    Tag,
}

impl ObjectType {
    pub fn from_mode(mode: &str) -> Result<ObjectType, anyhow::Error> {
        match mode {
            "100644" => Ok(ObjectType::Blob),
            "040000" | "40000" => Ok(ObjectType::Tree),
            "100755" => Ok(ObjectType::Executable),
            "120000" => Ok(ObjectType::Symlink),
            _ => Err(anyhow::anyhow!(format!(
                "Invalid file type, unable to parse {}",
                mode
            ))),
        }
    }

    pub fn to_mode(&self) -> &str {
        match self {
            ObjectType::Blob => "100644",
            ObjectType::Tree => "040000",
            ObjectType::Executable => "100755",
            ObjectType::Symlink => "120000",
            ObjectType::Commit => "160000",
            ObjectType::Tag => "",
        }
    }

    pub fn from_entry(entry: &std::fs::DirEntry) -> Result<ObjectType, anyhow::Error> {
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            Ok(ObjectType::Blob)
        } else if metadata.is_dir() {
            Ok(ObjectType::Tree)
        } else if metadata.permissions().mode() & 0o111 != 0 {
            Ok(ObjectType::Executable)
        } else if metadata.file_type().is_symlink() {
            Ok(ObjectType::Symlink)
        } else {
            Err(anyhow::anyhow!("Unable to determine file type"))
        }
    }
}

impl From<&ObjectType> for &str {
    fn from(value: &ObjectType) -> Self {
        match value {
            ObjectType::Blob => "blob",
            ObjectType::Tree => "tree",
            ObjectType::Executable => "executable",
            ObjectType::Symlink => "symlink",
            ObjectType::Commit => "commit",
            ObjectType::Tag => "tag",
        }
    }
}

impl FromStr for ObjectType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blob" => Ok(ObjectType::Blob),
            "tree" => Ok(ObjectType::Tree),
            "executable" => Ok(ObjectType::Executable),
            "symlink" => Ok(ObjectType::Symlink),
            "commit" => Ok(ObjectType::Commit),
            "tag" => Ok(ObjectType::Tag),
            _ => Err(anyhow::anyhow!("Invalid file type")),
        }
    }
}

pub enum ObjectFile {
    Tree(ObjectContents<TreeFile>),
    Other(ObjectContents<u8>),
}

pub struct ObjectContents<T> {
    pub file_type: ObjectType,
    pub content: Vec<T>,
    pub size: usize,
}

impl ObjectFile {
    pub fn new(hash: &FileHash) -> Result<Self, anyhow::Error> {
        let path: PathBuf = hash.into();
        let content = decode_file(path).context("Decoding file from hash")?;

        let (header, body) = split_at_empty_byte(&content).context("Splitting header and body")?;
        let header = String::from_utf8(header.to_vec()).context("Parsing header")?;
        let (file_type, size) = header
            .split_once(' ')
            .context("Splitting file type and size")?;

        let file_type = ObjectType::from_str(file_type).context("Parsing file type")?;
        let size: usize = size.parse().context("Parsing size")?;

        match file_type {
            ObjectType::Tree => {
                let tree_content = ls_tree::parse_tree_files(body).context("Parsing tree files")?;
                let tree = ObjectContents {
                    file_type,
                    content: tree_content,
                    size,
                };
                Ok(ObjectFile::Tree(tree))
            }
            _ => {
                let other = ObjectContents {
                    file_type,
                    content: body.to_vec(),
                    size,
                };
                Ok(ObjectFile::Other(other))
            }
        }
    }
}

impl TryFrom<&FileHash> for ObjectFile {
    type Error = anyhow::Error;

    fn try_from(value: &FileHash) -> Result<Self, Self::Error> {
        ObjectFile::new(value)
    }
}

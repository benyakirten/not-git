use std::{path::PathBuf, str::FromStr};

use anyhow::Context;

use crate::utils::{decode_file, split_header_from_contents};

use super::{ObjectHash, ObjectType, TreeObject};

/// A representation of a git object file. Trees are unique in that the contents
/// cannot be parsed to a string once decoded from Zlib since the file shas will
/// need to be decoded from the bytes in a secondary step.
/// TODO: Examples + Links to documentation
pub enum ObjectFile {
    Tree(ObjectContents<TreeObject>),
    Other(ObjectContents<u8>),
}

/// A simplistic representation of the relevant parts of a file. All git objects,
/// once decoded, begin with `<file_type> <size>\0`, which can be relevant for checking
/// for verifying object file/size integrity or for general information (such as ls-tree).
/// TODO: Examples + Links to documentation
pub struct ObjectContents<T> {
    pub object_type: ObjectType,
    pub contents: Vec<T>,
    pub size: usize,
}

impl<T> ObjectContents<T> {
    pub fn new(object_type: ObjectType, contents: Vec<T>, size: usize) -> Self {
        Self {
            object_type,
            contents,
            size,
        }
    }
}

impl ObjectFile {
    /// Read a file and generate a new ObjectFile from the contents. This will fail if
    /// the file does not have the expected properties, such as a header that contains
    /// `<file_type> <size>\0`. If the file type is a tree, it requires to be parsed
    /// further, which is handled by the `ls_tree` commands.
    /// TODO: Examples + Links to documentation
    pub fn new(hash: &ObjectHash) -> Result<Self, anyhow::Error> {
        let path: PathBuf = hash.into();
        let contents = decode_file(path).context("Decoding file from hash")?;

        let (header, body) =
            split_header_from_contents(&contents).context("Splitting header and body")?;
        let header = String::from_utf8(header.to_vec()).context("Parsing header")?;
        let (object_type, size) = header
            .split_once(' ')
            .context("Splitting file type and size")?;

        let object_type = ObjectType::from_str(object_type).context("Parsing file type")?;
        let size: usize = size.parse().context("Parsing size")?;

        match object_type {
            ObjectType::Tree => {
                let tree_content = TreeObject::from_object(body).context("Parsing tree object")?;
                let tree = ObjectContents::new(object_type, tree_content, size);
                Ok(Self::Tree(tree))
            }
            _ => {
                let other = ObjectContents::new(object_type, body.to_vec(), size);
                Ok(Self::Other(other))
            }
        }
    }
}

impl TryFrom<&ObjectHash> for ObjectFile {
    type Error = anyhow::Error;

    fn try_from(value: &ObjectHash) -> Result<Self, Self::Error> {
        ObjectFile::new(value)
    }
}

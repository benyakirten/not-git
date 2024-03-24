use std::{io::BufRead, path::PathBuf, str::FromStr};

use crate::object_type::ObjectType;

#[derive(Debug)]
pub struct FileHash {
    prefix: String,
    hash: String,
}

impl FileHash {
    pub fn new(prefix: String, hash: String) -> Self {
        FileHash { prefix, hash }
    }

    pub fn full_hash(&self) -> String {
        self.prefix.to_string() + &self.hash.to_string()
    }

    pub fn get_header(&self) -> Result<(ObjectType, usize), anyhow::Error> {
        let path: PathBuf = self.into();
        let f = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(f);

        let mut header = vec![];
        reader.read_until('\0' as u8, &mut header)?;

        let header = String::from_utf8(header)?;
        let (file_type, size) = header
            .split_once(' ')
            .ok_or_else(|| anyhow::anyhow!("Invalid header"))?;

        let file_type: ObjectType = file_type.parse()?;
        let size: usize = size.parse()?;
        Ok((file_type, size))
    }
}

impl From<&FileHash> for PathBuf {
    fn from(value: &FileHash) -> Self {
        ["not-git", "objects", &value.prefix, &value.hash]
            .iter()
            .collect()
    }
}

impl FromStr for FileHash {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 40 {
            return Err(anyhow::anyhow!(
                "Expected sha1 hash to be 40 characters long"
            ));
        }

        let prefix = s[..2].to_string();
        let hash = s[2..].to_string();

        Ok(FileHash::new(prefix, hash))
    }
}

use std::path::PathBuf;

#[derive(Debug)]
pub struct ObjectHash {
    prefix: String,
    hash: String,
}

impl ObjectHash {
    pub fn new(hash: &str) -> Result<Self, anyhow::Error> {
        if hash.len() != 40 {
            return Err(anyhow::anyhow!(
                "Expected sha1 hash to be 40 characters long"
            ));
        }

        let prefix = hash[..2].to_string();
        let hash = hash[2..].to_string();

        Ok(ObjectHash { prefix, hash })
    }

    pub fn from_bytes(nums: &[u8]) -> Result<Self, anyhow::Error> {
        if nums.len() != 20 {
            return Err(anyhow::anyhow!("Expected sha1 hash to be 20 bytes long"));
        }
        let hex: String = nums.encode_hex();

        ObjectHash::new(&hex)
    }

    pub fn full_hash(&self) -> String {
        self.prefix.to_string() + &self.hash.to_string()
    }

    pub fn path(&self) -> PathBuf {
        self.into()
    }
}

impl From<&ObjectHash> for PathBuf {
    fn from(value: &ObjectHash) -> Self {
        ["not-git", "objects", &value.prefix, &value.hash]
            .iter()
            .collect()
    }
}

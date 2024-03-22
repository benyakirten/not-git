#[derive(Debug)]
pub struct FileHash {
    pub prefix: String,
    pub hash: String,
}

impl FileHash {
    pub fn from_sha(hash: String) -> Result<Self, anyhow::Error> {
        if hash.len() < 40 {
            return Err(anyhow::anyhow!(
                "Expected sha1 hash to be 40 characters long"
            ));
        }

        let prefix = hash[..2].to_string();
        let hash = hash[2..].to_string();

        Ok(FileHash::new(prefix, hash))
    }

    pub fn new(prefix: String, hash: String) -> Self {
        FileHash { prefix, hash }
    }

    pub fn full_hash(&self) -> String {
        self.prefix.to_string() + &self.hash.to_string()
    }
}

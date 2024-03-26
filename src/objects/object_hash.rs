use std::path::PathBuf;

use hex::ToHex;

// TODO: Remove clone derive when tree objects take a reference to the hash.
#[derive(Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_valid_hash() {
        let hash = "0123456789abcdef0123456789abcdef01234567";
        let object_hash = ObjectHash::new(hash).unwrap();
        assert_eq!(object_hash.prefix, "01");
        assert_eq!(object_hash.hash, "23456789abcdef0123456789abcdef01234567");
    }

    #[test]
    fn new_invalid_hash() {
        let hash = "0123456789abcdef0123456789abcdef0123456"; // Invalid length
        let result = ObjectHash::new(hash);
        assert!(result.is_err());
    }

    #[test]
    fn from_bytes_valid_hash() {
        let bytes = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        ];
        let object_hash = ObjectHash::from_bytes(&bytes).unwrap();
        assert_eq!(object_hash.prefix, "01");
        assert_eq!(object_hash.hash, "02030405060708090a0b0c0d0e0f1011121314");
    }

    #[test]
    fn from_bytes_invalid_hash() {
        let bytes = [
            1, 35, 69, 103, 137, 171, 205, 239, 1, 35, 69, 103, 137, 171, 205, 239, 1,
        ]; // Invalid length
        let result = ObjectHash::from_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn full_hash() {
        let object_hash = ObjectHash {
            prefix: "01".to_string(),
            hash: "23456789abcdef0123456789abcdef01234567".to_string(),
        };
        assert_eq!(
            object_hash.full_hash(),
            "0123456789abcdef0123456789abcdef01234567"
        );
    }

    #[test]
    fn path() {
        let object_hash = ObjectHash {
            prefix: "01".to_string(),
            hash: "23456789abcdef0123456789abcdef01234567".to_string(),
        };
        let path = object_hash.path();
        assert_eq!(
            path,
            PathBuf::from("not-git/objects/01/23456789abcdef0123456789abcdef01234567")
        );
    }
}

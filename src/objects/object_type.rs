use std::os::unix::fs::PermissionsExt;
use std::str::FromStr;

/// A representation of the type of git object. It contains the possible types of objects that
/// git will use. The enum also contains relevant methods that will be used, such
/// as converting back and forth to modes and to string.
/// TODO: Examples + Links to documentation
#[derive(Debug, Clone, PartialEq)]
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
            ObjectType::Tag => "100644",
        }
    }

    pub fn from_entry(entry: &std::fs::DirEntry) -> Result<ObjectType, anyhow::Error> {
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            Ok(ObjectType::Blob)
        } else if metadata.is_dir() {
            Ok(ObjectType::Tree)
        } else if metadata.permissions().mode() & 0o111 != 0 {
            // Detecting an executable depends on the operating system. This
            // method is not reliable.
            Ok(ObjectType::Executable)
        } else if metadata.file_type().is_symlink() {
            Ok(ObjectType::Symlink)
        } else {
            Err(anyhow::anyhow!("Unable to determine file type"))
        }
    }

    pub fn as_str(&self) -> &str {
        self.into()
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

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Use macros to make these into parametrized tests
    #[test]
    fn test_from_mode() {
        assert_eq!(ObjectType::from_mode("100644").unwrap(), ObjectType::Blob);
        assert_eq!(ObjectType::from_mode("040000").unwrap(), ObjectType::Tree);
        assert_eq!(
            ObjectType::from_mode("100755").unwrap(),
            ObjectType::Executable
        );
        assert_eq!(
            ObjectType::from_mode("120000").unwrap(),
            ObjectType::Symlink
        );
        assert!(ObjectType::from_mode("invalid_mode").is_err());
    }

    #[test]
    fn test_to_mode() {
        assert_eq!(ObjectType::Blob.to_mode(), "100644");
        assert_eq!(ObjectType::Tree.to_mode(), "040000");
        assert_eq!(ObjectType::Executable.to_mode(), "100755");
        assert_eq!(ObjectType::Symlink.to_mode(), "120000");
        assert_eq!(ObjectType::Commit.to_mode(), "160000");
        assert_eq!(ObjectType::Tag.to_mode(), "");
    }

    #[test]
    fn test_as_str() {
        assert_eq!(ObjectType::Blob.as_str(), "blob");
        assert_eq!(ObjectType::Tree.as_str(), "tree");
        assert_eq!(ObjectType::Executable.as_str(), "executable");
        assert_eq!(ObjectType::Symlink.as_str(), "symlink");
        assert_eq!(ObjectType::Commit.as_str(), "commit");
        assert_eq!(ObjectType::Tag.as_str(), "tag");
    }

    #[test]
    fn test_from_str() {
        assert_eq!(ObjectType::from_str("blob").unwrap(), ObjectType::Blob);
        assert_eq!(ObjectType::from_str("tree").unwrap(), ObjectType::Tree);
        assert_eq!(
            ObjectType::from_str("executable").unwrap(),
            ObjectType::Executable
        );
        assert_eq!(
            ObjectType::from_str("symlink").unwrap(),
            ObjectType::Symlink
        );
        assert_eq!(ObjectType::from_str("commit").unwrap(), ObjectType::Commit);
        assert_eq!(ObjectType::from_str("tag").unwrap(), ObjectType::Tag);
        assert!(ObjectType::from_str("invalid_type").is_err());
    }

    #[test]
    fn test_from_entry() {
        // TODO
    }
}

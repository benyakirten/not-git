use std::{os::unix::fs::PermissionsExt, path::PathBuf};

use crate::{file_hash::FileHash, utils};

struct LsTreeConfig {
    file_hash: FileHash,
    flag: LsTreeFlag,
}

pub enum LsTreeFlag {
    NameOnly,
    Long,
}

impl LsTreeFlag {
    fn from_string(flag: &str) -> Result<LsTreeFlag, anyhow::Error> {
        match flag {
            "--name-only" => Ok(LsTreeFlag::NameOnly),
            "-l" => Ok(LsTreeFlag::Long),
            _ => Err(anyhow::anyhow!(format!("Invalid flag {}", flag))),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FileType {
    Blob,
    Tree,
    Executable,
    Symlink,
    Commit,
}

impl FileType {
    pub fn from_mode(mode: &str) -> Result<FileType, anyhow::Error> {
        match mode {
            "100644" => Ok(FileType::Blob),
            "040000" | "40000" => Ok(FileType::Tree),
            "100755" => Ok(FileType::Executable),
            "120000" => Ok(FileType::Symlink),
            _ => Err(anyhow::anyhow!(format!(
                "Invalid file type, unable to parse {}",
                mode
            ))),
        }
    }

    pub fn to_number_string(&self) -> &str {
        match self {
            FileType::Blob => "100644",
            FileType::Tree => "040000",
            FileType::Executable => "100755",
            FileType::Symlink => "120000",
            FileType::Commit => "160000",
        }
    }

    pub fn to_readable_string(&self) -> String {
        match self {
            FileType::Blob => "blob".to_string(),
            FileType::Tree => "tree".to_string(),
            FileType::Executable => "executable".to_string(),
            FileType::Symlink => "symlink".to_string(),
            FileType::Commit => "commit".to_string(),
        }
    }

    pub fn from_entry(entry: &std::fs::DirEntry) -> Result<FileType, anyhow::Error> {
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            Ok(FileType::Blob)
        } else if metadata.is_dir() {
            Ok(FileType::Tree)
        } else if metadata.permissions().mode() & 0o111 != 0 {
            Ok(FileType::Executable)
        } else if metadata.file_type().is_symlink() {
            Ok(FileType::Symlink)
        } else {
            Err(anyhow::anyhow!("Unable to determine file type"))
        }
    }
}

#[derive(Debug)]
pub struct TreeFile {
    file_type: FileType,
    file_name: String,
    sha: String,
}

pub fn list_tree_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_config(args)?;
    let decoded_content = decode_file(&config.file_hash)?;
    let output = stringify_list_tree(decoded_content, config.flag)?;
    println!("{}", output);

    Ok(())
}

pub fn stringify_list_tree(
    decoded_content: Vec<u8>,
    flag: LsTreeFlag,
) -> Result<String, anyhow::Error> {
    let tree_files = parse_tree_files(decoded_content)?;

    let output_string: String = match flag {
        LsTreeFlag::NameOnly => tree_files
            .iter()
            .map(|tree_file| format!("{}\n", tree_file.file_name))
            .collect(),
        LsTreeFlag::Long => {
            tree_files
                .iter()
                .map(|tree_file| {
                    // To not make this harder on myself, I'm leaving off the file size
                    format!(
                        "{} {} {}\t{}\n",
                        tree_file.file_type.to_number_string(),
                        tree_file.file_type.to_readable_string(),
                        tree_file.sha,
                        tree_file.file_name,
                    )
                })
                .collect()
        }
    };

    Ok(output_string)
}

pub fn parse_tree_files(decoded_content: Vec<u8>) -> Result<Vec<TreeFile>, anyhow::Error> {
    let mut tree_files = vec![];

    let (_, mut body) = split_at_next_empty_byte(decoded_content)?;

    while body.len() > 0 {
        let (tree_file, rest) = parse_until_next_file(body)?;
        tree_files.push(tree_file);
        body = rest;
    }

    Ok(tree_files)
}

fn split_at_next_empty_byte(contents: Vec<u8>) -> Result<(String, Vec<u8>), anyhow::Error> {
    let split_index = match contents.iter().position(|val| *val == 0) {
        Some(index) => index,
        None => {
            return Err(anyhow::anyhow!(
                "Invalid tree object: Unable to find null byte in {contents:?}"
            ));
        }
    };
    let (parseable, body) = contents.split_at(split_index + 1);
    let parseable_string = String::from_utf8(parseable.into())?;
    Ok((parseable_string, body.to_vec()))
}

fn parse_until_next_file(body: Vec<u8>) -> Result<(TreeFile, Vec<u8>), anyhow::Error> {
    let (mode_file_name, rest) = split_at_next_empty_byte(body)?;
    let (mode, file_name) = mode_file_name
        .split_once(' ')
        .ok_or_else(|| anyhow::anyhow!("Invalid tree object: Unable to find mode and file name"))?;
    let file_name = file_name.replace('\0', "");

    if rest.len() < 20 {
        return Err(anyhow::anyhow!("Invalid tree object: Unable to find sha"));
    }

    // Sha will always be 20 bytes long - after that the next entry begins.
    let (sha_bytes, rest) = if rest.len() > 20 {
        rest.split_at(20)
    } else {
        let empty_array: &[u8] = &[];
        (rest.as_slice(), empty_array)
    };

    let sha = hex::encode(sha_bytes);
    let file_type: FileType = FileType::from_mode(mode)?;

    let tree_file = TreeFile {
        file_type,
        file_name: file_name.to_string(),
        sha,
    };

    Ok((tree_file, rest.to_vec()))
}

fn decode_file(config: &FileHash) -> Result<Vec<u8>, anyhow::Error> {
    let path = ["not-git", "objects", &config.prefix, &config.hash]
        .iter()
        .collect::<PathBuf>();

    let decoded_content = utils::decode_file(path)?;
    Ok(decoded_content)
}

fn parse_config(args: &[String]) -> Result<LsTreeConfig, anyhow::Error> {
    if args.len() < 2 {
        return Err(anyhow::anyhow!("Usage: ls-tree --name-only|-l <tree_sha>"));
    }

    let file_hash = FileHash::from_sha(args[1].to_string())?;
    let flag = LsTreeFlag::from_string(&args[0])?;

    Ok(LsTreeConfig { file_hash, flag })
}

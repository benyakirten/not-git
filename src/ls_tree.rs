use std::path::PathBuf;

use crate::{file_hash::FileHash, utils};

struct LsTreeConfig {
    file_hash: FileHash,
    flag: LsTreeFlag,
}

enum LsTreeFlag {
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

#[derive(Debug)]
enum FileType {
    Blob,
    Tree,
    Executable,
    Symlink,
}

impl FileType {
    fn from_mode(mode: &str) -> Result<FileType, anyhow::Error> {
        match mode {
            "100644" => Ok(FileType::Blob),
            "040000" => Ok(FileType::Tree),
            "100755" => Ok(FileType::Executable),
            "120000" => Ok(FileType::Symlink),
            _ => Err(anyhow::anyhow!(format!(
                "Invalid file type, unable to parse {}",
                mode
            ))),
        }
    }

    fn to_number_string(&self) -> &str {
        match self {
            FileType::Blob => "100644",
            FileType::Tree => "040000",
            FileType::Executable => "100755",
            FileType::Symlink => "120000",
        }
    }

    fn to_readable_string(&self) -> String {
        match self {
            FileType::Blob => "blob".to_string(),
            FileType::Tree => "tree".to_string(),
            FileType::Executable => "executable".to_string(),
            FileType::Symlink => "symlink".to_string(),
        }
    }
}

#[derive(Debug)]
struct TreeFile {
    file_type: FileType,
    file_name: String,
    sha: String,
}

pub fn list_tree(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_config(args)?;
    let decoded_content = decode_file(&config.file_hash)?;
    let tree_files = parse_tree_files(decoded_content)?;

    match config.flag {
        LsTreeFlag::NameOnly => {
            for tree_file in tree_files {
                println!("{}", tree_file.file_name);
            }
        }
        LsTreeFlag::Long => {
            for tree_file in tree_files {
                // To not make this harder on myself, I'm leaving off the file size
                println!(
                    "{} {} {}\t{}",
                    tree_file.file_type.to_number_string(),
                    tree_file.file_type.to_readable_string(),
                    tree_file.sha,
                    tree_file.file_name,
                );
            }
        }
    }
    Ok(())
}

fn parse_tree_files(decoded_content: Vec<u8>) -> Result<Vec<TreeFile>, anyhow::Error> {
    // https://stackoverflow.com/questions/14790681/what-is-the-internal-format-of-a-git-tree-object
    // https://git-scm.com/book/en/v2/Git-Internals-Git-Objects
    // decoded_content might look like:
    // tree 221\0100644 cat_file.rs\07��fh�M�6:c\u{8}����Ę�.100644 hash_object.rs\0y�Jϭj�N�[�Gdi�3�=Z�100644 init.rs\0ڜ��-u[�G{U�?\u{18}m�R�\u{4}�100644 lib.rs\0\u{f}����LmiP��Q���\0\u{1a}�100644 main.rs\0��X\u{b} y�\u{9d}��3V��g�\u{1c}��100644 utils.rs\0���֎��f#f.�$Ty'@�\u{6}�
    // If you look right before main.rs, the sha contains a nulll byte \0, which makes splitting by null bytes
    // not always have the file name at the end of the split. This is a workaround that might not be fullproof
    // On the final version - we should parse each line into a struct
    // Each line is in the format {numbers-mode} {file_name}\n{sha}
    // Note that there's no separator beftween the sha and the numbers for the mode corresponding to the next line
    // Therefore we may want to split for the first two whitespaces after the header

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
                "Invalid tree object: Unable to find header"
            ))
        }
    };
    let (parseable, body) = contents.split_at(split_index + 1);
    let parseable_string = String::from_utf8(parseable.into())?;
    Ok((parseable_string, body.to_vec()))
}

fn parse_until_next_file(body: Vec<u8>) -> Result<(TreeFile, Vec<u8>), anyhow::Error> {
    println!("RECEIVING A BODY");
    println!("BODY: {:?}", body);
    let (mode_file_name, rest) = split_at_next_empty_byte(body)?;
    let (mode, file_name) = mode_file_name
        .split_once(' ')
        .ok_or_else(|| anyhow::anyhow!("Invalid tree object: Unable to find mode and file name"))?;

    let mut digits_acc: Vec<char> = vec![];
    let mut final_index = 0;
    for (i, byte) in rest.iter().enumerate() {
        if digits_acc.len() == 6 {
            final_index = i;
            break;
        }

        if byte.is_ascii_digit() {
            digits_acc.push(*byte as char);
        } else {
            digits_acc.clear()
        }
    }

    let (sha_bytes, rest) = rest.split_at(final_index);
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
    let path = [".git", "objects", &config.prefix, &config.hash]
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
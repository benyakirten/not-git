use std::path::PathBuf;

use crate::{file_hash::FileHash, utils};

pub fn list_tree(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_config(args)?;
    let decoded_content = decode_file(&config)?;
    let file_names = get_file_names(&decoded_content);

    for file_name in file_names {
        println!("{}", file_name);
    }
    Ok(())
}

fn get_file_names(decoded_content: &str) -> Vec<String> {
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
    decoded_content
        .split("\0")
        .skip(1)
        .filter_map(|line| {
            let file_name = line.split_whitespace().skip(1).last();
            match file_name {
                Some(name) => Some(name.to_string()),
                None => None,
            }
        })
        .collect()
}

fn decode_file(config: &FileHash) -> Result<String, anyhow::Error> {
    let path = [".git", "objects", &config.prefix, &config.hash]
        .iter()
        .collect::<PathBuf>();

    let decoded_content = utils::decode_file(path)?;
    let decoded_string = String::from_utf8_lossy(&decoded_content);
    Ok(decoded_string.to_string())
}

fn parse_config(args: &[String]) -> Result<FileHash, anyhow::Error> {
    if args.len() < 2 || args[0] != "--name-only" {
        return Err(anyhow::anyhow!("Usage: ls-tree --name-only <tree_sha>"));
    }

    let sha = args[1].to_string();
    FileHash::from_sha(sha)
}

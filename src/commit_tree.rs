use std::io::Write;

use crate::hash_object;
use crate::objects::{ObjectHash, ObjectType};

pub struct CommitTreeConfig {
    pub tree_hash: ObjectHash,
    pub message: String,
    pub parent_hash: Option<ObjectHash>,
}

impl CommitTreeConfig {
    pub fn new(tree_hash: ObjectHash, message: String, parent_hash: Option<ObjectHash>) -> Self {
        CommitTreeConfig {
            tree_hash,
            message,
            parent_hash,
        }
    }
}

pub fn commit_tree_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_commit_tree_config(args)?;
    let hash = create_commit(config)?;

    println!("{}", hash.full_hash());

    Ok(())
}

pub fn create_commit(config: CommitTreeConfig) -> Result<ObjectHash, anyhow::Error> {
    let mut contents = create_file_contents(config)?;
    let hash = hash_object::hash_and_write_object(None, &ObjectType::Commit, &mut contents)?;

    Ok(hash)
}

fn create_file_contents(config: CommitTreeConfig) -> Result<Vec<u8>, anyhow::Error> {
    let mut contents = Vec::new();
    writeln!(&mut contents, "tree {}", config.tree_hash.full_hash())?;

    if let Some(parent_hash) = config.parent_hash {
        writeln!(&mut contents, "parent {}", parent_hash.full_hash())?;
    }

    writeln!(
        &mut contents,
        "author Ben Horowitz >benyakir.horowitz@gmail.com>",
    )?;
    writeln!(
        &mut contents,
        "committer Ben Horowitz <benyakir.horowitz@gmail.com>",
    )?;
    writeln!(&mut contents, "{}", config.message)?;
    Ok(contents)
}

fn parse_commit_tree_config(args: &[String]) -> Result<CommitTreeConfig, anyhow::Error> {
    if args.is_empty() {
        return Err(anyhow::anyhow!("Usage: commit-tree <tree-hash>"));
    }

    let parent_hash = get_parent_hash(args)?;
    let message = get_flag_arg("-m", args)?;
    let tree_hash = get_tree_hash(args)?;

    Ok(CommitTreeConfig {
        tree_hash,
        message,
        parent_hash,
    })
}

fn get_tree_hash(args: &[String]) -> Result<ObjectHash, anyhow::Error> {
    // We have already checked that args.len() >= 1
    let tree_hash = if args[0].starts_with('-') {
        args.last().unwrap().to_string()
    } else {
        args[0].to_string()
    };

    let tree_hash = ObjectHash::new(&tree_hash)?;
    Ok(tree_hash)
}

fn get_parent_hash(args: &[String]) -> Result<Option<ObjectHash>, anyhow::Error> {
    let parent_hash = get_flag_arg_optional("-p", args)?;
    match parent_hash {
        Some(hash) => {
            let hash = ObjectHash::new(&hash)?;
            Ok(Some(hash))
        }
        None => Ok(None),
    }
}

fn get_flag_arg(flag: &str, args: &[String]) -> Result<String, anyhow::Error> {
    let result = get_flag_arg_optional(flag, args)?;
    if result.is_none() {
        return Err(anyhow::anyhow!(format!(
            "Argument {} is required for this command",
            flag
        )));
    }

    Ok(result.unwrap())
}

fn get_flag_arg_optional(flag: &str, args: &[String]) -> Result<Option<String>, anyhow::Error> {
    let flag_index = args.iter().position(|x| x == flag);
    if flag_index.is_none() {
        return Ok(None);
    }

    if args.len() < flag_index.unwrap() + 2 {
        return Err(anyhow::anyhow!(
            "Usage: commit-tree <tree-hash> -p <parent-hash> -m <message>"
        ));
    }

    let val = args[flag_index.unwrap() + 1].to_string();
    Ok(Some(val))
}

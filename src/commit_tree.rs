use crate::{file_hash::FileHash, hash_object, ls_tree::FileType};

struct CommitTreeConfig {
    tree_hash: FileHash,
    message: String,
    parent_hash: Option<FileHash>,
    // TODO: Add author and committer - figure out how git gets the values.
}

pub fn commit_tree(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_commit_tree_config(args)?;
    let mut contents = create_file_contents(config);
    let mut header = get_commit_header(&contents);

    header.append(&mut contents);
    let hash = hash_object::hash_and_write(&FileType::Commit, &mut header)?;

    println!("{}", &hash.full_hash());
    Ok(())
}

fn create_file_contents(config: CommitTreeConfig) -> Vec<u8> {
    let mut contents: Vec<u8> = Vec::new();
    let mut tree_hash = format!("tree {}\n", config.tree_hash.full_hash())
        .as_bytes()
        .to_vec();
    contents.append(&mut tree_hash);
    if config.parent_hash.is_some() {
        let mut parent_hash = format!("parent {}\n", config.parent_hash.unwrap().full_hash())
            .as_bytes()
            .to_vec();
        contents.append(&mut parent_hash);
    }

    let mut author = format!("author Ben Horowitz >benyakir.horowitz@gmail.com>\n")
        .as_bytes()
        .to_vec();
    contents.append(&mut author);

    let mut committer = format!("committer Ben Horowitz <benyakir.horowitz@gmail.com>\n")
        .as_bytes()
        .to_vec();
    contents.append(&mut committer);

    let mut message = format!("{}\n", config.message).as_bytes().to_vec();
    contents.append(&mut message);

    contents
}

fn get_commit_header(contents: &[u8]) -> Vec<u8> {
    let header = format!("commit {}\0", contents.len());
    header.as_bytes().to_vec()
}

fn parse_commit_tree_config(args: &[String]) -> Result<CommitTreeConfig, anyhow::Error> {
    if args.len() < 1 {
        return Err(anyhow::anyhow!("Usage: commit-tree <tree-hash>"));
    }

    let parent_hash = get_parent_hash(args)?;
    let message = get_flag_arg("-m", args)?;
    let tree_hash = get_tree_hash(args)?;

    Ok(CommitTreeConfig {
        tree_hash,
        message: message,
        parent_hash,
    })
}

fn get_tree_hash(args: &[String]) -> Result<FileHash, anyhow::Error> {
    // We have already checked that args.len() >= 1
    let tree_hash = if args[0].starts_with("-") {
        args.last().unwrap().to_string()
    } else {
        args[0].to_string()
    };

    let tree_hash = FileHash::from_sha(tree_hash)?;
    Ok(tree_hash)
}

fn get_parent_hash(args: &[String]) -> Result<Option<FileHash>, anyhow::Error> {
    let parent_hash = get_flag_arg_optional("-p", args)?;
    match parent_hash {
        Some(hash) => {
            let hash = FileHash::from_sha(hash)?;
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

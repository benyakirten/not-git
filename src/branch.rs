use std::path::PathBuf;

pub struct BranchConfig {
    all: bool,
}

pub fn branch_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_branch_config(args);
    branch(config)?;

    Ok(())
}

pub fn branch(config: BranchConfig) -> Result<Vec<String>, anyhow::Error> {
    // TODO: Handle tags

    let head_path: PathBuf = ["not-git", "refs", "heads"].iter().collect();
    let mut branches = collect_branches(head_path)?;

    if config.all {
        let remotes_path: PathBuf = ["not-git", "refs", "remotes"].iter().collect();
        let remotes_branches = collect_branches(remotes_path)?;
        branches.extend(remotes_branches);
    }

    Ok(branches)
}

fn collect_branches(path: PathBuf) -> Result<Vec<String>, anyhow::Error> {
    let mut files: Vec<String> = vec![];
    for p in path.read_dir()? {
        if p.is_err() {
            // TODO: Log error
            continue;
        }

        let p = p.unwrap();
        let file_type = p.file_type();

        if file_type.is_err() {
            // TODO: Log error
            continue;
        }

        if p.file_type().unwrap().is_dir() {
            let dir_files = collect_branches(p.path());
            match dir_files {
                Ok(mut f) => files.append(&mut f),
                Err(e) => return Err(e),
            }
        } else {
            files.push(p.file_name().to_string_lossy().to_string());
        }
    }

    Ok(files)
}

fn parse_branch_config(args: &[String]) -> BranchConfig {
    let all = args.contains(&"-a".to_string());
    BranchConfig { all }
}

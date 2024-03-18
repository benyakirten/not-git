use std::path::PathBuf;

use crate::utils::get_head_ref;

pub struct BranchConfig {
    all: bool,
}

pub struct BranchOptions {
    branches: Vec<String>,
    head_ref: String,
}

pub fn branch_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_branch_config(args);
    let branches = list_branches(config)?;

    for file_name in branches.branches {
        if file_name == branches.head_ref {
            println!("\x1b[92m * {}\x1b[0m", file_name);
        } else {
            println!("   {}", file_name);
        }
    }

    Ok(())
}

pub fn list_branches(config: BranchConfig) -> Result<BranchOptions, anyhow::Error> {
    // TODO: Handle tags

    let head_path: PathBuf = ["not-git", "refs", "heads"].iter().collect();
    let mut branches = collect_branches(vec![], head_path)?;

    if config.all {
        let remotes_path: PathBuf = ["not-git", "refs", "remotes"].iter().collect();
        let remotes_branches = collect_branches(vec![], remotes_path)?;
        branches.extend(remotes_branches);
    }

    let head_ref = get_head_ref()?;

    let branch_options = BranchOptions { branches, head_ref };
    Ok(branch_options)
}

fn collect_branches(
    preceding_dirs: Vec<String>,
    path: PathBuf,
) -> Result<Vec<String>, anyhow::Error> {
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
            let mut new_dir = preceding_dirs.to_vec();
            new_dir.push(p.file_name().to_string_lossy().to_string());
            let dir_files = collect_branches(new_dir, p.path());
            match dir_files {
                Ok(mut f) => files.append(&mut f),
                Err(e) => return Err(e),
            }
        } else {
            let file_name = p.file_name().to_string_lossy().to_string();
            let branch_name = preceding_dirs.join("/") + "/" + &file_name;
            files.push(branch_name);
        }
    }

    Ok(files)
}

fn parse_branch_config(args: &[String]) -> BranchConfig {
    let all = args.contains(&"-a".to_string());
    BranchConfig { all }
}

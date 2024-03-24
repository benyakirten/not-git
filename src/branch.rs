use std::path::PathBuf;

use crate::{
    objects::ObjectHash,
    update_refs::{self, UpdateRefsConfig},
    utils::{get_head_ref, read_from_file},
};

pub enum BranchConfig {
    List(bool),
    Delete(String),
    Create(String),
}

pub struct ListBranchOptions {
    branches: Vec<String>,
    head_ref: String,
}

pub struct DeleteBranchResults {
    path: PathBuf,
    sha: String,
}

pub fn branch_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_branch_config(args)?;
    match config {
        BranchConfig::List(list_all_branches) => {
            let branches = list_branches(list_all_branches)?;
            print_branches(branches)
        }
        BranchConfig::Delete(branch_name) => {
            let results = delete_branch(branch_name)?;
            println!("Deleted branch {:?} (was {})", results.path, results.sha);
            Ok(())
        }
        BranchConfig::Create(branch_name) => create_branch(branch_name),
    }
}

fn delete_branch(branch_name: String) -> Result<DeleteBranchResults, anyhow::Error> {
    let head_ref = get_head_ref()?;
    if branch_name == head_ref {
        return Err(anyhow::anyhow!(
            "Cannot delete the branch you are currently on"
        ));
    }

    let path: PathBuf = ["not-git", "refs", "heads", branch_name.as_str()]
        .iter()
        .collect();
    if !path.exists() {
        return Err(anyhow::anyhow!("Branch {} does not exist", branch_name));
    }

    let contents = std::fs::read_to_string(&path)?;
    let contents = ObjectHash::from_sha(contents)?;

    std::fs::remove_file(&path)?;

    Ok(DeleteBranchResults {
        path: path.iter().skip(3).collect(),
        sha: contents.full_hash(),
    })
}

// TODO: Does this functionality need to be revisited?
fn create_branch(branch_name: String) -> Result<(), anyhow::Error> {
    let path: PathBuf = ["not-git", "refs", "heads", &branch_name].iter().collect();
    if path.exists() {
        return Err(anyhow::anyhow!("Branch {} already exists", branch_name));
    }

    let head_ref = get_head_ref()?;
    let head_path: PathBuf = ["not-git", "refs", "heads", &head_ref].iter().collect();

    if !head_path.exists() {
        return Err(anyhow::anyhow!("HEAD does not point to a valid branch"));
    }

    let head_commit = {
        let head_commit = read_from_file(&head_path)?;
        let head_commit = String::from_utf8(head_commit)?;
        ObjectHash::from_sha(head_commit)?
    };

    let config = UpdateRefsConfig::new(head_commit, PathBuf::from(branch_name));
    update_refs::update_refs(config)?;

    Ok(())
}

fn print_branches(branches: ListBranchOptions) -> Result<(), anyhow::Error> {
    for file_name in branches.branches {
        if file_name == branches.head_ref {
            println!("\x1b[92m * {}\x1b[0m", file_name);
        } else {
            println!("   {}", file_name);
        }
    }

    Ok(())
}

pub fn list_branches(_list_all_branches: bool) -> Result<ListBranchOptions, anyhow::Error> {
    // TODO: Handle tags

    let head_path: PathBuf = ["not-git", "refs", "heads"].iter().collect();
    let branches = collect_branches(vec![], head_path)?;

    // TODO: List all branches if -a tag - decode packed-refs

    let head_ref = get_head_ref()?;

    let branch_options = ListBranchOptions { branches, head_ref };
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
            let branch_name_base = preceding_dirs.join("/");
            let branch_name = if branch_name_base.is_empty() {
                file_name
            } else {
                format!("{}/{}", branch_name_base, file_name)
            };
            files.push(branch_name);
        }
    }

    Ok(files)
}

fn parse_branch_config(args: &[String]) -> Result<BranchConfig, anyhow::Error> {
    if args.is_empty() || args[0] == "--list" {
        return Ok(BranchConfig::List(false));
    }

    if args[0] == "-a" {
        return Ok(BranchConfig::List(true));
    }

    if args[0] == "-d" {
        if args.len() < 2 {
            return Err(anyhow::anyhow!("Branch name not provided"));
        }
        return Ok(BranchConfig::Delete(args[1].to_string()));
    }

    Ok(BranchConfig::Create(args[0].to_string()))
}

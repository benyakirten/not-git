use std::fs;
use std::path::PathBuf;

pub struct InitConfig<'a> {
    branch_name: &'a str,
    directory: Option<&'a str>,
}

impl<'a> InitConfig<'a> {
    pub fn new(branch_name: &'a str, directory: Option<&'a str>) -> Self {
        InitConfig {
            branch_name,
            directory,
        }
    }
}

// TODO: Allow the parent directory to be customized
pub fn create_directories(config: InitConfig) -> Result<(), anyhow::Error> {
    let base_path: PathBuf = match config.directory {
        Some(directory) => [directory, "not-git"].iter().collect(),
        None => PathBuf::from("not-git"),
    };

    fs::create_dir_all(base_path.join("objects"))?;
    fs::create_dir_all(base_path.join("refs/heads"))?;
    fs::write(
        base_path.join("HEAD"),
        format!("ref: refs/heads/{}\n", config.branch_name),
    )?;
    fs::write(
        base_path.join("packed-refs"),
        "# pack-refs with: peeled fully-peeled sorted\n",
    )?;

    Ok(())
}

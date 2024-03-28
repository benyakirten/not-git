use std::fs;
use std::path::PathBuf;

pub struct InitConfig<'a> {
    path: Option<&'a PathBuf>,
    branch_name: &'a str,
}

impl<'a> InitConfig<'a> {
    pub fn new(branch_name: &'a str, path: Option<&'a PathBuf>) -> Self {
        InitConfig { path, branch_name }
    }
}

// TODO: Allow the parent directory to be customized
pub fn create_directories(config: InitConfig) -> Result<(), anyhow::Error> {
    let base_path: PathBuf = match config.path {
        Some(path) => path.join("not-git"),
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

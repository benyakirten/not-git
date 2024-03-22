use std::{fs, path::PathBuf};

pub struct InitConfig {
    pub commit_name: String,
}

pub fn init_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_options(args);
    create_directories(config)?;

    println!("Initialized git directory.");
    Ok(())
}

// TODO: Allow the parent directory to be customized
pub fn create_directories(config: InitConfig) -> Result<(), anyhow::Error> {
    fs::create_dir_all(PathBuf::from("not-git/objects"))?;
    fs::create_dir_all(PathBuf::from("not-git/refs/heads"))?;
    fs::write(
        PathBuf::from("not-git/HEAD"),
        format!("ref: refs/heads/{}\n", config.commit_name),
    )?;
    fs::write(
        PathBuf::from("not-git/packed-refs"),
        "# pack-refs with: peeled fully-peeled sorted\n",
    )?;

    Ok(())
}

fn parse_options(args: &[String]) -> InitConfig {
    let commit_name = if args.len() == 2 && args[0] == "-b" {
        args[1].to_string()
    } else {
        "main".to_string()
    };

    InitConfig { commit_name }
}

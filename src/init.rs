use std::fs;

struct InitConfig {
    commit_name: String,
}

pub fn create_directories(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_options(args);

    // TODO - Check for already existing git directory/partially initialized git directory
    fs::create_dir("not-git")?;
    fs::create_dir("not-git/objects")?;
    fs::create_dir("not-git/refs")?;
    fs::create_dir("not-git/refs/heads")?;
    fs::write(
        "not-git/HEAD",
        format!("ref: refs/heads/{}\n", config.commit_name),
    )?;

    println!("Initialized git directory.");
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

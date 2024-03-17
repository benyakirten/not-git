use std::fs;

struct InitConfig {
    commit_name: String,
}

pub fn create_directories(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_options(args);

    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::create_dir(".git/refs/heads")?;
    fs::write(
        ".git/HEAD",
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

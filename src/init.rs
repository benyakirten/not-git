use std::fs;

pub fn create_directories() -> Result<(), anyhow::Error> {
    println!("Initialized git directory.");
    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/master\n")?;

    Ok(())
}

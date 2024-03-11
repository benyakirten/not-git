use std::fs;

pub fn create_directories() -> Result<(), std::io::Error> {
    println!("Initialized not-git directory.");
    fs::create_dir(".not-git")?;
    fs::create_dir(".not-git/objects")?;
    fs::create_dir(".not-git/refs")?;
    fs::write(".not-git/HEAD", "ref: refs/heads/master\n")?;

    Ok(())
}

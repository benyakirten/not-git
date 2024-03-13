use std::{fs, io::Read, path::Path};

pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut content = vec![];
    file.read_to_end(&mut content)?;

    Ok(content)
}

pub fn print_string(content: &str) {
    print!("{}", content);

    if !content.ends_with("\n") {
        print!("\n");
    }
}

use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use flate2::read::ZlibDecoder;

pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut content = vec![];
    file.read_to_end(&mut content)?;

    Ok(content)
}

pub fn decode_file(path: PathBuf) -> Result<Vec<u8>, anyhow::Error> {
    let encoded_content = read_from_file(path)?;

    let mut decoder = ZlibDecoder::new(encoded_content.as_slice());

    let mut decoded_vec = vec![];
    decoder.read_to_end(&mut decoded_vec)?;
    Ok(decoded_vec)
}

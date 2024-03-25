use std::fs;
use std::path::PathBuf;

pub fn setup() -> PathBuf {
    let test_dir_name = uuid::Uuid::new_v4().to_string();
    let test_dir = [".test", &test_dir_name].iter().collect::<PathBuf>();
    if !test_dir.exists() {
        fs::create_dir_all(&test_dir).unwrap();
    }

    test_dir
}

pub fn cleanup(path: PathBuf) {
    fs::remove_dir_all(path).unwrap();
}

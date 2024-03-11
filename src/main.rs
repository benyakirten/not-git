use std::env;

use not_git::{cat_file, init};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: not-git <command>");
        return;
    }

    let command = args[1].clone();
    let result = match command.as_str() {
        "init" => init::create_directories(),
        "cat-file" => cat_file::file(args[2..].to_vec()),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Unknown command {}", command),
        )),
    };

    match result {
        Ok(()) => {}
        Err(e) => println!("Error: {}", e),
    }
}

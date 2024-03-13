use std::env;

use not_git::{cat_file, hash_object, init};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: not-git <command>");
        return;
    }

    let command = args[1].to_string();
    let result = match command.as_str() {
        "init" => init::create_directories(),
        "cat-file" => cat_file::cat(args[2..].to_vec()),
        "hash-object" => hash_object::hash(args[2..].to_vec()),
        _ => Err(anyhow::anyhow!(format!("Unknown command {}", command))),
    };

    match result {
        Ok(()) => {}
        Err(e) => println!("Error: {}", e),
    }
}

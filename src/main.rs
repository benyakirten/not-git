use std::env;

use not_git::{cat_file, hash_object, init, ls_tree, write_tree};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: not-git <command>");
        return;
    }

    let command = args[1].to_string();
    let result = match command.as_str() {
        "init" => init::create_directories(),
        "cat-file" => cat_file::cat(&args[2..]),
        "hash-object" => hash_object::hash(&args[2..]),
        "ls-tree" => ls_tree::list_tree(&args[2..]),
        "write-tree" => write_tree::write(&args[2..]),
        _ => Err(anyhow::anyhow!(format!("Unknown command {}", command))),
    };

    match result {
        Ok(()) => {}
        Err(e) => println!("Error: {}", e),
    }
}

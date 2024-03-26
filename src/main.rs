use std::env;

use not_git::{branch, clone, commit, hash_object, write_tree};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: not-git <command>");
        return;
    }

    let command = args[1].to_string();
    let result = match command.as_str() {
        "hash-object" => hash_object::hash_object_command(&args[2..]),
        "branch" => branch::branch_command(&args[2..]),
        "commit" => commit::commit_command(&args[2..]),
        "clone" => clone::clone_command(&args[2..]),
        "write-tree" => write_tree::write_tree_command(&args[2..]),
        _ => Err(anyhow::anyhow!(format!("Unknown command {}", command))),
    };

    match result {
        Ok(()) => {}
        Err(e) => println!("Error: {}", e),
    }
}

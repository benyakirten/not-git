use std::env;

use not_git::{
    branch, cat_file, clone, commit, commit_tree, hash_object, init, ls_tree, update_refs,
    write_tree,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: not-git <command>");
        return;
    }

    let command = args[1].to_string();
    let result = match command.as_str() {
        "init" => init::init_command(&args[2..]),
        "cat-file" => cat_file::cat(&args[2..]),
        "hash-object" => hash_object::hash_object_command(&args[2..]),
        "ls-tree" => ls_tree::list_tree_command(&args[2..]),
        "write-tree" => write_tree::write_tree_command(&args[2..]),
        "commit-tree" => commit_tree::commit_tree_command(&args[2..]),
        "update-refs" => update_refs::update_refs_command(&args[2..]),
        "branch" => branch::branch_command(&args[2..]),
        "commit" => commit::commit_command(&args[2..]),
        "clone" => clone::clone_command(&args[2..]),
        _ => Err(anyhow::anyhow!(format!("Unknown command {}", command))),
    };

    match result {
        Ok(()) => {}
        Err(e) => println!("Error: {}", e),
    }
}

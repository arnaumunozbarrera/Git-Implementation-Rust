// Import libraries
use std::{
    env
};

// Import functions
mod cli;
mod utils;

// Constant values
const MIN_ARGS_LEN_CLI: usize = 4;

// CLI Entry point
fn main() {
    // Argument extraction from cli command (if available)
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        // Case: there is a command
        let command = &args[1];

        // Switch case: choose command response / behaviour
        match command.as_str() {
            // Initialization section of `.voor` folder
            "init" => { 
                cli::init::init_command();
            }
            // Creation of cat-file
            "cat-file" => {
                if args.len() >= MIN_ARGS_LEN_CLI {
                    let argument = &args[2];
                    let hash = &args[3].clone();

                    cli::cat_file::cat_file_command(&argument, &hash);
                } else {
                    println!("[ERROR] Not enough arguments provided to execute the `cat-file` command");
                }
            }
            // Creation of hash-object
            "hash-object" => {
                if args.len() >= MIN_ARGS_LEN_CLI {
                    let argument = &args[2];
                    let file_path = &args[3].clone();

                    cli::hash_object::hash_object_command(&argument, &file_path);
                } else {
                    println!("[ERROR] Not enough arguments provided to execute the `hash-object` command");
                }
            }
            "diff" => {
                // TODO: diff --staged || HEAD
                // if args.len() == 3 {
                //     let argument = &args[2];

                //     if argument == "--staged" {

                //     } else if argument == "HEAD" {
                //         // HEAD
                //     }
                // } else if args.len() > 3 {
                //     let ref1 = &args[3];
                //     let ref4 = &args[4];

                //     // TODO: first, implement diff for branches

                //     // TODO: second, implement diff for commits0
                // } else {
                //     // Not prepared changes

                // }

                let old_hash = &args[2];
                let file_path = &args[3].clone();

                cli::diff::diff_by_hash(&old_hash, &file_path);
            }
            "add" => {
                let file_path = &args[2].clone();

                cli::add::add_by_hash(&file_path);
            }
            // Default response for unknown command
            _ => {
                println!("[EXIT] Unknown command.\nTry one of this list:");
                println!("\t· init\n\t· cat-file\n\t· hash-object\n");
            }
        }
    } else {
        // Case: no command typed
        println!("[EXIT] No command typed.\n Try one of this list:");
        println!("\t· init\n\t· cat-file\n\t· hash-object\n");
    }
}
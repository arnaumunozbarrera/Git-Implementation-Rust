// Import libraries
use std::{
    env
};
use std::path::Path;
use crate::cli::status::display_status;

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
                if args.len() != 3 {
                    println!("[EXIT] Unknown argument.\nTry one of these:");
                    println!("\t· <file_name>\n\t· . (add all)\n");
                    return;
                }

                let argument = &args[2];

                if argument == "." {
                    cli::add::add_all(std::path::Path::new("."));
                } else {
                    cli::add::add_by_hash(std::path::Path::new(argument));
                }
            }
            "status" => {
                if args.len() != 2 {
                    println!("[EXIT] Unknown argument.\nTry without one:");
                    return;
                }

                let root_path = Path::new(".");

                display_status(root_path);
            }
            "commit" => {
                if args.len() != 4 {
                    println!("[EXIT] Unknown argument.\nTry this one:");
                    println!("\t· -m <commit_message>");
                    return;
                }

                let message = &args[2];
                cli::commit::commit(message);
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
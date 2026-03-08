// Import libraries
use std::{
    env, fs::{self}
};

// Import functions
mod cli;

// CLI Entry point
fn main() {
    let title = fs::read_to_string("src/cli/title.txt")
        .expect("[ERROR] Could not read ASCII inside file");
    let subtitle = fs::read_to_string("src/cli/subtitle.txt")
        .expect("[ERROR] Could not read ASCII inside file");

    println!("{}\n{}\n", title, subtitle);

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
                if args.len() >= 3 {
                    let argument = &args[2];
                    let hash = &args[3].clone();

                    cli::cat_file::cat_file_command(&argument, &hash);
                } else {
                    println!("[ERROR] Not enough arguments provided to execute the `cat-file` command");
                }
            }
            // Creation of hash-object
            "hash-object" => {
                if args.len() >= 3 {
                    let argument = &args[2];
                    let file_path = &args[3].clone();

                    cli::hash_object::hash_object_command(&argument, &file_path);
                } else {
                    println!("[ERROR] Not enough arguments provided to execute the `hash-object` command");
                }
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
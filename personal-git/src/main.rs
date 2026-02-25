#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::path::Path;

fn main() {
    println!("Hello, I'm Arnau and this will be my personal implementation of Git as a version controller with Rust! \n");
    
    let args: Vec<String> = env::args().collect();

    let command = &args[1];

    // Refactor of if-else to switch-case / match
    match command.as_str() {
        // Initialization section of `.voor` folder
        "init" => { 
            if Path::new(".voor").exists() {
                println!("[INFO] `.voor` directory already initialized\n");
            } else {
                fs::create_dir(".voor").unwrap();
                fs::create_dir(".voor/objects").unwrap();
                fs::create_dir(".voor/ref").unwrap();
                fs::write(".voor/HEAD", "ref: refs/heads/master\n").unwrap();
                println!("[INFO] `.voor` directory initialized successfully!\n");
            }
        }
        // Creation of cat-file
        "cat-file" => {
            let argument = &args[2];
            println!("[DEBUG] Argument typed: {}", argument);

            if argument == "-p" {
                
            } else {
                println!("[INFO] Unknown argument. Did you mean `-p`?\n");
            }
        }
        // Default response for unknown command
        _ => {
            println!("[INFO] Unknown command. Did you mean `init`?\n");
        }
    }
    
}

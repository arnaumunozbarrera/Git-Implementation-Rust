#[allow(unused_imports)]
use std::env;
use std::{fmt::format, io::Read};
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
                let hash = &args[3].clone();
                // println!("[DEBUG] Hash value: {}", hash);
                
                let folder_name = &hash[0..2];
                let file_name = &hash[2..];

                println!("[DEBUG] Folder name: {}, File name: {}", folder_name, file_name);

                let path = format!(".voor/objects/{folder_name}/{file_name}");
                let mut file = fs::File::open(path).expect("[WARN] Unable to open file");
                let mut contents = String::new();

                file.read_to_string(&mut contents).expect("[WARN] Unable to read content from file");

                println!("[DEBUG] File content: {}", contents);
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

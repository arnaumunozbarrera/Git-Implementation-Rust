use std::{
    fs,
    path::Path,
    env,
    io::{self, Read, Write}
};
use flate2::read::ZlibDecoder;

fn main() {
    println!("Hello, I'm Arnau and this will be my personal implementation of Git as a version controller with Rust! \n");
    
    let args: Vec<String> = env::args().collect();

    let command = &args[1];

    // Refactor of if-else to switch-case / match
    match command.as_str() {
        // Initialization section of `.voor` folder
        "init" => { 
            init_command();
        }
        // Creation of cat-file
        "cat-file" => {
            let argument = &args[2];
            let hash = &args[3].clone();

            cat_file_command(&argument, &hash);
        }
        // Default response for unknown command
        _ => {
            println!("[INFO] Unknown command. Did you mean `init`?\n");
        }
    }
    
}

fn init_command() {
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

fn cat_file_command(argument: &str, hash: &str) {
    // println!("[DEBUG] Argument typed: {}", argument);

    if argument == "-p" {
        // println!("[DEBUG] Hash value: {}", hash);
        
        let folder_name = &hash[0..2];
        let file_name = &hash[2..];

        println!("[DEBUG] Folder name: {}, File name: {}", folder_name, file_name);

        let path = format!(".voor/objects/{folder_name}/{file_name}");
        let mut object = fs::File::open(path).expect("[WARN] Unable to open file\n");
        let mut content: Vec<u8> = vec![];
        let mut extracted_content = String::new();

        object.read_to_end(&mut content).expect("[WARN] Unable to read content from file\n");

        let mut decoder = ZlibDecoder::new(content.as_slice());

        decoder.read_to_string(&mut extracted_content).unwrap();
        let split = extracted_content.split("\x00");
        let extracted_content = split.last().unwrap();

        println!("[DEBUG] Extracted file content: {}", extracted_content);
        io::stdout().flush().unwrap();
    } else {
        println!("[INFO] Unknown argument. Did you mean `-p`?\n");
    }
}
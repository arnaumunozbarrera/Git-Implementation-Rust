use core::hash;
use std::{
    env, fs::{self, File}, io::{self, Read, Write}, path::Path
};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Sha1, Digest};

// CLI Entry point
fn main() {
    let title = fs::read_to_string("src/cli/title.txt")
        .expect("[ERROR] Could not read ASCII inside file");
    let subtitle = fs::read_to_string("src/cli/subtitle.txt")
        .expect("[ERROR] Could not read ASCII inside file");

    println!("{}\n{}", title, subtitle);
    
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
        // Creation of hash-object
        "hash-object" => {
            let argument = &args[2];
            let file_path = &args[3].clone();

            hash_object_command(&argument, &file_path);
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

fn hash_object_command(argument: &str, file_path: &str) {
    if argument == "-w" {
        // Read content from file_path
        let mut object = fs::File::open(file_path).expect("[WARN] Unable to read content from file\n");
        let mut content = Vec::new();
        object.read_to_end(&mut content).expect("[WARN] Unable to read content from file\n");

        // Hash creation & apply blob format
        let header = format!("blob {}\0", content.len());
        let mut full = header.into_bytes();
        full.extend(content);

        let mut hasher = Sha1::new();
        hasher.update(&full);
        let hash = format!("{:x}", hasher.finalize());

        let (dir, file) = hash.split_at(2);
        fs::create_dir_all(format!(".voor/objects/{}", dir)).unwrap();

        // Compress
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&full).unwrap();
        let compressed = encoder.finish().unwrap();

        fs::write(format!(".voor/objects/{}/{}", dir, file), compressed).unwrap();

        println!("[INFO] Blob created successfully at folder: ./voor/objects/{} with a hash value of: {}", dir, hash);
    } else {
        println!("[INFO] Unknown argument. Did you mean `-w`?\n");
    }
}
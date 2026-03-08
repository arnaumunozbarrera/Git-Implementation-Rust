// Import libraries
use std::{
    fs::{self}, io::{self, Read, Write}
};
use flate2::read::ZlibDecoder;

pub fn cat_file_command(argument: &str, hash: &str) {
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
// Import libraries
use std::{
    fs::{self}, io::{self, Read, Write}
};
use flate2::read::ZlibDecoder;
use crate::utils::read_file;

pub fn cat_file_command(argument: &str, hash: &str) {
    // println!("[DEBUG] Argument typed: {}", argument);
    if argument == "-p" {
        // println!("[DEBUG] Hash value: {}", hash);

        if hash.len() > 2 {
            let folder_name = &hash[0..2];
            let file_name = &hash[2..];

            // println!("[DEBUG] Folder name: {}, File name: {}", folder_name, file_name);

            let path = format!(".voor/objects/{folder_name}/{file_name}");
            let extracted_content = read_file::read_file(&path);
            
            io::stdout().flush().unwrap();
        } else {
            println!("[ERROR] Specified hash is not long enough")
        }
    } else {
        println!("[INFO] Unknown argument. Did you mean `-p`?\n");
    }
}
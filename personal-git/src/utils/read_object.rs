// Import libraries
use std::{
    fs::{self}, io::{Read}
};
use flate2::read::ZlibDecoder;

pub fn read_object(path: &str) -> String {
    let mut object = fs::File::open(path).expect("[WARN] Unable to open file from path:\n");
    let mut content: Vec<u8> = vec![];
    let mut extracted_content = String::new();

    object.read_to_end(&mut content).expect("[WARN] Unable to read content from file\n");

    let mut decoder = ZlibDecoder::new(content.as_slice());

    decoder.read_to_string(&mut extracted_content).unwrap();
    let split = extracted_content.split("\x00");
    let extracted_content = split.last().unwrap();

    let file_content = extracted_content
        .split('\0')
        .last()
        .unwrap_or("")
        .to_string();

    println!("[DEBUG] Extracted file content: {}", file_content);

    file_content
}
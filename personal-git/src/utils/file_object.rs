// Import libraries
use std::fs::File;
use std::io::Read;

use flate2::read::ZlibDecoder;

pub fn read_object_bytes(path: &str) -> Vec<u8> {
    let mut object =
        File::open(path).expect("[ERROR] Unable to open object file");

    let mut compressed = Vec::new();
    object
        .read_to_end(&mut compressed)
        .expect("[ERROR] Unable to read compressed object");

    let mut decoder = ZlibDecoder::new(compressed.as_slice());
    let mut decompressed = Vec::new();

    decoder
        .read_to_end(&mut decompressed)
        .expect("[ERROR] Unable to decompress object");

    decompressed
}

pub fn extract_blob_content(object_bytes: &[u8]) -> &[u8] {
    let null_pos = object_bytes
        .iter()
        .position(|&b| b == 0)
        .expect("[ERROR] Invalid object format: missing header separator");

    &object_bytes[null_pos + 1..]
}

pub fn read_blob_content(path: &str) -> Vec<u8> {
    let object_bytes = read_object_bytes(path);
    extract_blob_content(&object_bytes).to_vec()
}

pub fn print_blob_content(path: &str) {
    let content = read_blob_content(path);

    match std::str::from_utf8(&content) {
        Ok(text) => {
            print!("{}", text);
        }
        Err(_) => {
            eprintln!("[ERROR] Blob content is binary, cannot print safely as UTF-8 text");
        }
    }
}
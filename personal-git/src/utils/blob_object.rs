// Import libraries
use std::fs;
use std::io::Write;

use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};

pub fn get_header(extracted_content: &[u8]) -> String {
    format!("blob {}\0", extracted_content.len())
}

pub fn get_hash(header: &str, extracted_content: &[u8]) -> (String, Vec<u8>) {
    let mut full = header.as_bytes().to_vec();
    full.extend_from_slice(extracted_content);

    let mut hasher = Sha1::new();
    hasher.update(&full);

    let hash = format!("{:x}", hasher.finalize());
    (hash, full)
}

pub fn save_compressed_object(dir: &str, file: &str, full: &[u8]) {
    fs::create_dir_all(format!(".voor/objects/{}", dir)).unwrap();

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(full).unwrap();
    let compressed = encoder.finish().unwrap();

    fs::write(format!(".voor/objects/{}/{}", dir, file), compressed).unwrap();

    println!("[INFO] Blob created successfully at folder: .voor/objects/{}", dir);
}
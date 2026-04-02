use std::fs;
use std::io::Write;

use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest as Sha1Digest, Sha1};
use sha2::Sha256;

#[derive(Debug, Clone, Copy)]
pub enum HashAlgorithm {
    Sha1,
    Sha256,
}

pub fn get_header(extracted_content: &[u8]) -> Vec<u8> {
    format!("blob {}\0", extracted_content.len()).into_bytes()
}

pub fn serialize_blob(extracted_content: &[u8]) -> Vec<u8> {
    let mut full = get_header(extracted_content);
    full.extend_from_slice(extracted_content);
    full
}

pub fn compute_hash(full: &[u8], algorithm: HashAlgorithm) -> String {
    match algorithm {
        HashAlgorithm::Sha1 => {
            let mut hasher = Sha1::new();
            hasher.update(full);
            format!("{:x}", hasher.finalize())
        }
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(full);
            format!("{:x}", hasher.finalize())
        }
    }
}

pub fn get_hash(extracted_content: &[u8], algorithm: HashAlgorithm) -> (String, Vec<u8>) {
    let full = serialize_blob(extracted_content);
    let hash = compute_hash(&full, algorithm);
    (hash, full)
}

pub fn save_compressed_object(dir: &str, file: &str, full: &[u8]) {
    fs::create_dir_all(format!(".voor/objects/{}", dir))
        .expect("[ERROR] Unable to create object directory");

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(full)
        .expect("[ERROR] Unable to compress object");
    let compressed = encoder.finish().expect("[ERROR] Unable to finalize compression");

    fs::write(format!(".voor/objects/{}/{}", dir, file), compressed)
        .expect("[ERROR] Unable to write object file");

    // println!("[INFO] Blob created successfully at folder: .voor/objects/{}", dir);
}
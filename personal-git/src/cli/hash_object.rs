// Import libraries
use std::fs;
use std::io::{Write};
use sha1::{Sha1, Digest};
use flate2::write::ZlibEncoder;
use flate2::Compression;

pub fn hash_object_command(argument: &str, file_path: &str) {
    if argument == "-w" {
        // Read content from file_path
        let extracted_content = fs::read(file_path).expect("[WARN] Unable to read content from file");

        // Hash creation & apply blob format
        let header = format!("blob {}\0", extracted_content.len());
        let mut full = header.into_bytes();
        full.extend(extracted_content);

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
// Import libraries
use std::fs;

use crate::utils::blob_object::{self, HashAlgorithm};

pub fn hash_object_command(argument: &str, file_path: &str) -> String {
    let algorithm = match argument {
        "-w" | "--sha1" => HashAlgorithm::Sha1,
        "--sha256" => HashAlgorithm::Sha256,
        _ => {
            println!("[INFO] Unknown argument. Use `-w`, `--sha1`, or `--sha256`\n");
            return String::new();
        }
    };

    let extracted_content =
        fs::read(file_path).expect("[WARN] Unable to read content from file");

    let (hash, full) = blob_object::get_hash(&extracted_content, algorithm);

    let (dir, file) = hash.split_at(2);
    blob_object::save_compressed_object(dir, file, &full);

    println!("{}", hash);

    hash
}
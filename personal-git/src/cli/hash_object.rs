// Import libraries
use std::fs;

use crate::utils::blob_object;

pub fn hash_object_command(argument: &str, file_path: &str) {
    if argument == "-w" {
        // Read content from file_path
        let extracted_content = fs::read(file_path).expect("[WARN] Unable to read content from file");

        // Hash creation & apply blob format
        let header = blob_object::get_header(&extracted_content);
        let (hash, full) = blob_object::get_hash(&header, &extracted_content);

        // Compress & save inside `/objects`
        let (dir, file) = hash.split_at(2);
        blob_object::save_compressed_object(dir, file, &full);
    } else {
        println!("[INFO] Unknown argument. Did you mean `-w`?\n");
    }
}
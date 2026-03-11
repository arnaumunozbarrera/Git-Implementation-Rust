use std::fs;
use std::path::Path;

use crate::{cli::diff::diff_by_hash, utils::blob_object::{self, HashAlgorithm}};

pub fn display_status(root_path: &Path) {
    let entries = fs::read_dir(root_path).unwrap();

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            // Read file
            let full = fs::read(&path).unwrap();

            // Compute hash
            let (hash, _) = blob_object::get_hash(&full, HashAlgorithm::Sha256);
            // Split hash
            let dir = &hash[..2];
            let file = &hash[2..];

            let object_path = Path::new(".voor")
                .join("objects")
                .join(dir)
                .join(file);

            // Check if object exists
            if !object_path.exists() {
                println!("Untracked: {}", path.display());
            } else {
                let old_bytes: Vec<u8> =
                    crate::utils::file_object::read_blob_content(object_path.to_str().unwrap());

                if old_bytes == full {
                    println!("Tracked: {}, not modified", path.display());
                } else {
                    println!("Tracked changes in: {}", path.display());
                    diff_by_hash(&hash, path.to_str().unwrap());
                }
            }

        } else if path.is_dir() {
            // Avoid scanning .voor itself
            if path.file_name().unwrap() == ".voor" {
                continue;
            }

            display_status(&path);
        }
    }
}
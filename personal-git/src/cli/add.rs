use std::fs;
use std::path::Path;

use crate::cli::hash_object;

pub fn add_by_hash(path: &Path) {
    hash_object::hash_object_command("--sha256", path.to_str().unwrap());
    println!("[INFO] Staged file: {}\n", path.display());
}

pub fn add_all(root_path: &Path) {
    let entries = fs::read_dir(root_path).unwrap();

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            add_by_hash(&path);
        } 
        else if path.is_dir() {
            add_all(&path); // Recursion
        }
    }
}
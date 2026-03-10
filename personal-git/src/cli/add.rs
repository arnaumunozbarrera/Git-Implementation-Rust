// Import libraries
use crate::cli::hash_object;

pub fn stage_by_hash(path: &str) {
    hash_object::hash_object_command("--sha256", &path);
    println!("[INFO] Object staged successfully");
}
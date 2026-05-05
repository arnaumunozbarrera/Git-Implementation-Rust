// Import libraries
use std::fs;

use crate::utils::file_object;

pub fn diff_by_hash(old_hash: &str, path: &str) {
    if old_hash.len() <= 2 {
        println!("[ERROR] Specified hash is not long enough");
        return;
    }

    let folder_name = &old_hash[0..2];
    let file_name = &old_hash[2..];
    let object_path = format!(".voor/objects/{}/{}", folder_name, file_name);

    // Read old stored blob content
    let old_bytes = file_object::read_blob_content(&object_path);

    // Read current working file
    let current_bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => {
            println!("[ERROR] Unable to read working file");
            return;
        }
    };

    // Convert both contents to text
    let old_text = match String::from_utf8(old_bytes) {
        Ok(text) => text,
        Err(_) => {
            println!("[ERROR] Stored object is not valid UTF-8 text");
            return;
        }
    };

    let current_text = match String::from_utf8(current_bytes) {
        Ok(text) => text,
        Err(_) => {
            println!("[ERROR] Working file is not valid UTF-8 text");
            return;
        }
    };

    // Split into lines
    let old_lines: Vec<&str> = old_text.lines().collect();
    let current_lines: Vec<&str> = current_text.lines().collect();

    let max_len = old_lines.len().max(current_lines.len());

    // Compare line by line
    for i in 0..max_len {
        match (old_lines.get(i), current_lines.get(i)) {
            (Some(old), Some(new)) if old == new => {
                println!("No changes in the content:\n`{}`", old);
            }
            (Some(old), Some(new)) => {
                println!("- {}", old);
                println!("+ {}", new);
            }
            (Some(old), None) => {
                println!("- {}", old);
            }
            (None, Some(new)) => {
                println!("+ {}", new);
            }
            (None, None) => {}
        }
    }
}

// Changes not prepared
// pub fn diff() {

// }

// Changes prepared for commit: --staged
// pub fn diff_staged() {

// }

// All unconfirmed changes
// pub fn diff_head() {

// }

// Changes between branches
// pub fn diff_branches() {

// }

// Changes between commits
// pub fn diff_commits() {

// }
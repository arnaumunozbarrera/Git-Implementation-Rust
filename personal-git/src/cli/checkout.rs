use std::fs;
use std::path::Path;

use crate::cli::branch;

pub fn checkout_to_branch(branch_name: &str) {
    let branch_path = format!("{}/{}", ".voor/refs/heads", branch_name);

    // Check if branch exists
    if !Path::new(&branch_path).exists() {
        println!("[ERROR] Branch '{}' does not exist", branch_name);
        return;
    }

    // Update HEAD to point to the new branch
    let new_head_content = format!("ref: refs/heads/{}", branch_name);

    fs::write(".voor/refs/HEAD", new_head_content)
        .expect("[ERROR] Failed to update HEAD");

    println!("[INFO] Switched to branch '{}'", branch_name);
}

pub fn create_branch_and_checkout(branch_name: &str) {
    let branch_path = format!("{}/{}", ".voor/refs/heads", branch_name);

    // If branch already exists → just checkout
    if Path::new(&branch_path).exists() {
        println!("[INFO] Branch '{}' already exists, switching to it", branch_name);
        checkout_to_branch(branch_name);
        return;
    }

    // Create branch first
    branch::create_branch(branch_name);

    // Then switch to it
    checkout_to_branch(branch_name);
}
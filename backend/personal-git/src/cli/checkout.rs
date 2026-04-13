use std::fs;
use std::path::Path;

use crate::cli::branch;
use crate::cli::status;
use crate::utils::refs;
use crate::utils::sync;

pub fn checkout_to_branch(branch_name: &str) {
    let root_path = Path::new(".");
    let has_changes = status::changes_not_commited(root_path);

    if has_changes {
        println!("[WARN] There is changes in branch '{}' to commit.\nCommit them before changing branches.", branch_name);
    } else {
        let branch_path = format!("{}/{}", ".voor/refs/heads", branch_name);

        // Check if branch exists
        if !Path::new(&branch_path).exists() {
            println!("[ERROR] Branch '{}' does not exist", branch_name);
            return;
        }

        let previous_commit = refs::read_head_target();
        let target_commit = fs::read_to_string(&branch_path).unwrap_or_default();

        // Update HEAD to point to the new branch
        let new_head_content = format!("ref: refs/heads/{}", branch_name);

        fs::write(".voor/HEAD", new_head_content)
            .expect("[ERROR] Failed to update HEAD");

        if let Err(error) = sync::restore_working_tree(&previous_commit, target_commit.trim()) {
            println!("{}", error);
            return;
        }

        println!("[INFO] Switched to branch '{}'", branch_name);
    }
}

pub fn create_branch_and_checkout(branch_name: &str) {
    let root_path = Path::new(".");
    let has_changes = status::changes_not_commited(root_path);

    if has_changes {
        println!("[WARN] There is changes in branch '{}' to commit.\nCommit them before changing branches.", branch_name);
    } else {
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
}

use std::fs;
use std::path::Path;

use crate::cli::branch;
use crate::cli::status;
use crate::utils::fs_ops;
use crate::utils::refs;
use crate::utils::sync;

pub fn checkout_to_branch(branch_name: &str) {
    if let Err(error) = fs_ops::with_repo_lock("checkout", || checkout_to_branch_locked(branch_name)) {
        println!("{}", error);
    }
}

pub fn create_branch_and_checkout(branch_name: &str) {
    let result = fs_ops::with_repo_lock("checkout-create-branch", || {
        let root_path = Path::new(".");
        let has_changes = status::changes_not_commited(root_path);

        if has_changes {
            println!("[WARN] There is changes in branch '{}' to commit.\nCommit them before changing branches.", branch_name);
            return Ok(());
        }

        let branch_path = format!("{}/{}", ".voor/refs/heads", branch_name);
        if Path::new(&branch_path).exists() {
            println!("[INFO] Branch '{}' already exists, switching to it", branch_name);
            return checkout_to_branch_locked(branch_name);
        }

        branch::create_branch_locked(branch_name)?;
        checkout_to_branch_locked(branch_name)
    });

    if let Err(error) = result {
        println!("{}", error);
    }
}

fn checkout_to_branch_locked(branch_name: &str) -> Result<(), String> {
    let root_path = Path::new(".");
    let has_changes = status::changes_not_commited(root_path);

    if has_changes {
        println!("[WARN] There is changes in branch '{}' to commit.\nCommit them before changing branches.", branch_name);
        return Ok(());
    }

    let branch_path = format!("{}/{}", ".voor/refs/heads", branch_name);
    if !Path::new(&branch_path).exists() {
        println!("[ERROR] Branch '{}' does not exist", branch_name);
        return Ok(());
    }

    let previous_commit = refs::read_head_target();
    let target_commit = fs::read_to_string(&branch_path).unwrap_or_default();
    let new_head_content = format!("ref: refs/heads/{}", branch_name);
    fs_ops::write_file_atomic(".voor/HEAD", new_head_content.as_bytes())?;

    sync::restore_working_tree(&previous_commit, target_commit.trim())?;
    println!("[INFO] Switched to branch '{}'", branch_name);
    Ok(())
}

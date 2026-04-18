use std::fs;
use std::path::Path;

use crate::utils::fs_ops;

pub fn get_current_branch() -> String {
    let head_content = fs::read_to_string(".voor/HEAD")
        .expect("[ERROR] Failed to read HEAD");

    head_content
        .strip_prefix("ref: refs/heads/")
        .expect("[ERROR] Invalid HEAD format")
        .trim()
        .to_string()
}

pub fn current_branch_or(value: Option<&str>) -> String {
    value
        .map(|branch| branch.trim().to_string())
        .filter(|branch| !branch.is_empty())
        .unwrap_or_else(get_current_branch)
}

pub fn display_branches() {
    if !Path::new(".voor/refs/heads").exists() {
        println!("[INFO] No branches found");
        return;
    }

    let head_content = fs::read_to_string(".voor/HEAD")
        .expect("[ERROR] Failed to read HEAD");

    let current_branch = head_content
        .strip_prefix("ref: refs/heads/")
        .unwrap_or("")
        .trim();

    let entries = fs::read_dir(".voor/refs/heads")
        .expect("[ERROR] Failed to read branches directory");

    println!("[INFO] Available branches:");

    for entry in entries {
        if let Ok(entry) = entry {
            if let Some(branch_name) = entry.file_name().to_str() {
                if branch_name == current_branch {
                    println!("> {} (current branch)", branch_name);
                } else {
                    println!("  {}", branch_name);
                }
            }
        }
    }
}

pub fn create_branch(branch_name: &str) {
    if let Err(error) = fs_ops::with_repo_lock("branch-create", || create_branch_locked(branch_name)) {
        println!("{}", error);
    }
}

pub fn delete_branch(branch_name: &str) {
    if let Err(error) = fs_ops::with_repo_lock("branch-delete", || delete_branch_locked(branch_name)) {
        println!("{}", error);
    }
}

pub(crate) fn create_branch_locked(branch_name: &str) -> Result<(), String> {
    let branch_path = format!("{}/{}", ".voor/refs/heads", branch_name);

    if Path::new(&branch_path).exists() {
        println!("[INFO] Branch '{}' already exists", branch_name);
        return Ok(());
    }

    let head_content = fs::read_to_string(".voor/HEAD")
        .expect("[ERROR] Failed to read HEAD");

    let current_ref = head_content
        .strip_prefix("ref: ")
        .expect("[ERROR] Invalid HEAD format")
        .trim();

    let current_commit = fs::read_to_string(format!("{}/{}", ".voor", current_ref))
        .expect("[ERROR] Failed to read current branch");

    fs_ops::write_file_atomic(&branch_path, current_commit.as_bytes())?;
    println!("[INFO] Branch '{}' created", branch_name);
    Ok(())
}

fn delete_branch_locked(branch_name: &str) -> Result<(), String> {
    let branch_path = format!("{}/{}", ".voor/refs/heads", branch_name);

    if !Path::new(&branch_path).exists() {
        println!("[INFO] Branch '{}' does not exist", branch_name);
        return Ok(());
    }

    let head_content = fs::read_to_string(".voor/HEAD")
        .expect("[ERROR]Failed to read HEAD");

    if head_content.contains(branch_name) {
        println!("[WARN] Cannot delete the current branch '{}'", branch_name);
        println!("\t Try checking out to another branch and re-run the command");
        return Ok(());
    }

    fs::remove_file(branch_path)
        .map_err(|error| format!("[ERROR] Failed to delete branch '{}': {}", branch_name, error))?;

    println!("Branch '{}' deleted", branch_name);
    Ok(())
}

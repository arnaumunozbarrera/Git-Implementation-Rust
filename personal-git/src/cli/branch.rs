use std::fs;
use std::path::Path;

pub fn display_branches() {
    if !Path::new(".voor/refs/heads").exists() {
        println!("[INFO] No branches found");
        return;
    }

    // Read HEAD to know current branch
    let head_content = fs::read_to_string(".voor/refs/HEAD")
        .expect("[ERROR] Failed to read HEAD");

    let current_branch = head_content
        .strip_prefix("ref: refs/heads/")
        .unwrap_or("")
        .trim();

    // Read directory entries
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
    let branch_path = format!("{}/{}", ".voor/refs/heads", branch_name);

    if Path::new(&branch_path).exists() {
        println!("[INFO] Branch '{}' already exists", branch_name);
        return;
    }

    // Read HEAD to find current branch
    let head_content = fs::read_to_string(".voor/HEAD")
        .expect("[ERROR] Failed to read HEAD");

    let current_ref = head_content
        .strip_prefix("ref: ")
        .expect("[ERROR] Invalid HEAD format")
        .trim();

    // Read current commit hash
    let current_commit = fs::read_to_string(format!("{}/{}", ".voor", current_ref))
        .expect("[ERROR] Failed to read current branch");

    // Create new branch with same commit
    fs::write(branch_path, current_commit)
        .expect("[ERROR] Failed to create branch");

    println!("[INFO] Branch '{}' created", branch_name);
}   

pub fn delete_branch(branch_name: &str) {
    let branch_path = format!("{}/{}", ".voor/refs/heads", branch_name);

    // Check if branch exists
    if !Path::new(&branch_path).exists() {
        println!("[INFO] Branch '{}' does not exist", branch_name);
        return;
    }

    // Prevent deleting current branch
    let head_content = fs::read_to_string(".voor/HEAD")
        .expect("[ERROR]Failed to read HEAD");

    if head_content.contains(branch_name) {
        println!("[WARN] Cannot delete the current branch '{}'", branch_name);
        println!("\t Try checking out to another branch and re-run the command");
        return;
    }

    // Delete branch file
    fs::remove_file(branch_path)
        .expect("Failed to delete branch");

    println!("Branch '{}' deleted", branch_name);
}
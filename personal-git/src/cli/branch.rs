use std::fs;
use std::path::Path;

pub fn create_branch(branch_name: &str) {
    let branch_path = format!("{}/{}", ".voor/refs/heads", branch_name);

    if Path::new(&branch_path).exists() {
        println!("Branch '{}' already exists", branch_name);
        return;
    }

    // Read HEAD to find current branch
    let head_content = fs::read_to_string(".voor/refs/HEAD")
        .expect("Failed to read HEAD");

    let current_ref = head_content
        .strip_prefix("ref: ")
        .expect("Invalid HEAD format")
        .trim();

    // Read current commit hash
    let current_commit = fs::read_to_string(format!("{}/{}", ".voor", current_ref))
        .expect("Failed to read current branch");

    // Create new branch with same commit
    fs::write(branch_path, current_commit)
        .expect("Failed to create branch");

    println!("Branch '{}' created", branch_name);
}

pub fn delete_branch(branch_name: &str) {

}
use crate::utils::index;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use sha1::{Sha1};
use crate::utils::blob_object::{get_hash, HashAlgorithm, save_compressed_object}; 


// Verifies that there are staged files before committing
pub fn verify_staged_files() {
    let index = index::read_index();
    if index.is_empty() {
        println!("No changes added to commit");
        return;
    }
}

// Creates a commit object content and returns its SHA-1 hash
pub fn create_commit_object(tree_hash: String, message: &str) -> String {
    // Read parent commit from HEAD
    let parent_hash = match fs::read_to_string(".voor/HEAD") {
        Ok(p) if !p.trim().is_empty() => format!("parent {}\n", p.trim()),
        _ => String::new(),
    };

    // Timestamp for author/committer
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)
        .expect("Time went backwards").as_secs();

    let commit_content = format!(
        "tree {}\n{}author {} <{}> {}\ncommitter {} <{}> {}\n\n{}",
        tree_hash,
        parent_hash,
        "Your Name", "you@example.com", timestamp,
        "Arnau Muñoz Barrera", "arnaumunozbarrera@gmail.com", timestamp,
        message
    );

    // Compute the commit hash and get serialized content
    let (commit_hash, full_commit_content) = get_hash(commit_content.as_bytes(), HashAlgorithm::Sha1);

    // Save the commit object (compressed) in the objects directory
    save_compressed_object("commit", &commit_hash, &full_commit_content);

    // commit_hash now contains the SHA-1 hex string
    commit_hash
}

// Stores the commit object to .voor/objects
pub fn store_commit_object(commit_hash: String, tree_hash: String, message: &str) {
    let parent_hash = match fs::read_to_string(".voor/HEAD") {
        Ok(p) if !p.trim().is_empty() => format!("parent {}\n", p.trim()),
        _ => String::new(),
    };

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)
        .expect("Time went backwards").as_secs();

    let commit_content = format!(
        "tree {}\n{}author {} <{}> {}\ncommitter {} <{}> {}\n\n{}",
        tree_hash,
        parent_hash,
        "Your Name", "you@example.com", timestamp,
        "Arnau Muñoz Barrera", "arnaumunozbarrera@gmail.com", timestamp,
        message
    );

    fs::write(format!(".voor/objects/{}", commit_hash), commit_content)
        .expect("[ERROR] Unable to write commit object");
}

// Clears the staging area
pub fn clear_index() {
    fs::write(".voor/index", "").expect("[ERROR] Unable to clear index");
}

// Builds a tree object for all staged files and returns its SHA-1 hash
pub fn build_tree_object() -> String {
    let index = index::read_index();
    let mut tree_content = String::new();

    for (hash, path) in index {
        tree_content.push_str(&format!("{} {}\n", hash, path));
    }

    // Compute tree hash and serialized content
    let (tree_hash, full_tree_content) = get_hash(tree_content.as_bytes(), HashAlgorithm::Sha1);

    // Save the tree object compressed under "tree" folder
    save_compressed_object("tree", &tree_hash, &full_tree_content);

    tree_hash
}
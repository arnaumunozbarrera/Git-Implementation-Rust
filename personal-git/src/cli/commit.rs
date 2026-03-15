use crate::utils::tree_builder;
use crate::utils::refs;

pub fn commit(message: &str) {
    // Make sure there are staged files
    tree_builder::verify_staged_files();

    // Build the tree object and store it
    let tree_hash = tree_builder::build_tree_object();

    // Create the commit object and store it, returning the commit hash
    let commit_hash = tree_builder::create_commit_object(tree_hash, message);

    // Clear the staging area
    tree_builder::clear_index();

    // Update HEAD to point to the new commit
    refs::update_ref("HEAD", &commit_hash);

    println!("[INFO] Commit {} created successfully", commit_hash);
}
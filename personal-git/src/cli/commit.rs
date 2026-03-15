use crate::utils::refs;
use crate::utils::tree_builder;

pub fn commit(message: &str) {
    // Abort early if there is nothing staged
    if !tree_builder::verify_staged_files() {
        return;
    }

    // Build and store the tree object
    let tree_hash = tree_builder::build_tree_object();

    // Build and store the commit object
    let commit_hash = tree_builder::create_commit_object(tree_hash, message);

    // Move the current branch (or detached HEAD) to the new commit
    refs::update_head_target(&commit_hash);

    // Clear staging only after commit/ref update succeeded
    tree_builder::clear_index();

    println!("[INFO] Commit {} created successfully", commit_hash);
}
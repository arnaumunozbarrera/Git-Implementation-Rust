use crate::utils::fs_ops;
use crate::utils::refs;
use crate::utils::tree_builder;

pub fn commit(message: &str) {
    let result = fs_ops::with_repo_lock("commit", || {
        if !tree_builder::verify_staged_files() {
            return Ok(());
        }

        let tree_hash = tree_builder::build_tree_object();
        let commit_hash = tree_builder::create_commit_object(tree_hash, message);
        refs::update_head_target(&commit_hash);
        tree_builder::clear_index();

        println!("[INFO] Commit {} created successfully", commit_hash);
        Ok(())
    });

    if let Err(error) = result {
        println!("{}", error);
    }
}

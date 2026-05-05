use crate::utils::index;
use crate::utils::fs_ops;
use crate::utils::object_store::{self, ObjectType};
use crate::utils::refs;
use crate::utils::sync;
use std::time::{SystemTime, UNIX_EPOCH};

fn normalize_stored_path(path: &str) -> String {
    let mut normalized = path.trim().replace('\\', "/");

    while normalized.starts_with("./") {
        normalized = normalized[2..].to_string();
    }

    normalized
}

fn build_parent_line() -> String {
    let parent = refs::read_head_target();

    if parent.is_empty() {
        String::new()
    } else {
        format!("parent {}\n", parent)
    }
}

/// Returns true if there are staged files, false otherwise.
pub fn verify_staged_files() -> bool {
    let index_map = index::read_index();

    if index_map.is_empty() {
        println!("No changes added to commit");
        return false;
    }

    true
}

/// Creates and stores a commit object, returning its SHA-1 hash.
pub fn create_commit_object(tree_hash: String, message: &str) -> String {
    let parent_line = build_parent_line();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let commit_content = format!(
        "tree {}\n{}author {} <{}> {}\ncommitter {} <{}> {}\n\n{}",
        tree_hash.trim(),
        parent_line,
        "Your Name",
        "you@example.com",
        timestamp,
        "Arnau Muñoz Barrera",
        "arnaumunozbarrera@gmail.com",
        timestamp,
        message
    );

    object_store::write_object(ObjectType::Commit, commit_content.as_bytes())
        .expect("[ERROR] Unable to store commit object")
}

/// Stores a specific commit object under the provided hash.
/// Kept for compatibility/debug usage.
#[allow(dead_code)]
pub fn store_commit_object(commit_hash: String, tree_hash: String, message: &str) {
    let parent_line = build_parent_line();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let commit_content = format!(
        "tree {}\n{}author {} <{}> {}\ncommitter {} <{}> {}\n\n{}",
        tree_hash.trim(),
        parent_line,
        "Your Name",
        "you@example.com",
        timestamp,
        "Arnau Muñoz Barrera",
        "arnaumunozbarrera@gmail.com",
        timestamp,
        message
    );

    let (computed_hash, full_commit_content) =
        object_store::hash_object(ObjectType::Commit, commit_content.as_bytes());

    if computed_hash != commit_hash.trim() {
        eprintln!(
            "[WARNING] Provided commit hash does not match computed hash. Writing provided hash anyway: {}",
            commit_hash.trim()
        );
    }

    object_store::write_full_object(commit_hash.trim(), &full_commit_content)
        .expect("[ERROR] Unable to store commit object");
}

pub fn clear_index() {
    fs_ops::write_file_atomic(".voor/index", b"").expect("[ERROR] Unable to clear index");
}

/// Builds a tree object from the staged index and returns its SHA-1 hash.
///
/// Tree entry format:
///     <blob_hash>\t<path>\n
///
/// Using '\t' instead of ' ' avoids breaking paths that contain spaces.
pub fn build_tree_object() -> String {
    let index_map = index::read_index();

    // Assumes read_index() returns: path -> hash
    let mut entries: Vec<(String, String)> = index_map
        .into_iter()
        .map(|(path, hash)| (normalize_stored_path(&path), hash.trim().to_string()))
        .collect();

    // Deterministic ordering is required so the same logical tree always hashes the same.
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    sync::build_tree_from_index(&entries).expect("[ERROR] Unable to build tree object")
}

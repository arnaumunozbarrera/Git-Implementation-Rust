use std::fs;
use std::path::Path;

use crate::utils::fs_ops;

pub fn read_head() -> String {
    fs::read_to_string(".voor/HEAD").expect("[ERROR] Unable to read HEAD")
}

pub fn get_head_ref() -> String {
    let head = read_head();

    head.strip_prefix("ref: ")
        .unwrap_or("")
        .trim()
        .to_string()
}

pub fn read_head_target() -> String {
    let head = read_head();
    let trimmed = head.trim();

    if let Some(head_ref) = trimmed.strip_prefix("ref: ") {
        let path = format!(".voor/{}", head_ref.trim());

        fs::read_to_string(path)
            .unwrap_or_default()
            .trim()
            .to_string()
    } else {
        // Detached HEAD: HEAD itself stores the commit hash
        trimmed.to_string()
    }
}

/// Updates a reference file (e.g. refs/heads/main) to point to a commit hash
pub fn update_ref(reference: &str, hash_content: &str) {
    let path = format!(".voor/{}", reference.trim());

    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent).expect("[ERROR] Unable to create ref directory");
    }

    fs_ops::write_file_atomic(&path, hash_content.trim().as_bytes())
        .expect("[ERROR] Unable to update ref");
}

/// Updates whatever HEAD currently points to:
/// - if HEAD is symbolic, move that branch ref
/// - if HEAD is detached, write the hash directly into HEAD
pub fn update_head_target(hash_content: &str) {
    let head = read_head();
    let trimmed = head.trim();

    if let Some(head_ref) = trimmed.strip_prefix("ref: ") {
        update_ref(head_ref.trim(), hash_content);
    } else {
        fs_ops::write_file_atomic(".voor/HEAD", hash_content.trim().as_bytes())
            .expect("[ERROR] Unable to update HEAD");
    }
}

#[allow(dead_code)]
pub fn update_head_branch(branch: &str) {
    let content = format!("ref: refs/heads/{}", branch.trim());
    fs_ops::write_file_atomic(".voor/HEAD", content.as_bytes()).expect("[ERROR] Unable to update HEAD");
}

use crate::utils::blob_object::{self, HashAlgorithm};
use crate::utils::index;
use crate::utils::refs;
use crate::utils::sync;
use ignore::WalkBuilder;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::cli::branch;

fn normalize_repo_path(root_path: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root_path).unwrap_or(path);

    let mut normalized = relative.to_string_lossy().replace('\\', "/");

    while normalized.starts_with("./") {
        normalized = normalized[2..].to_string();
    }

    normalized
}

fn normalize_stored_path(path: &str) -> String {
    let mut normalized = path.trim().replace('\\', "/");

    while normalized.starts_with("./") {
        normalized = normalized[2..].to_string();
    }

    normalized
}

fn read_commit_tree(commit_hash: &str) -> HashMap<String, String> {
    sync::read_commit_tree(commit_hash)
        .unwrap_or_default()
        .into_iter()
        .map(|(path, hash)| (normalize_stored_path(&path), hash))
        .collect()
}

pub fn display_status(root_path: &Path) {
    let current_branch = branch::get_current_branch();
    println!("[INFO] On branch: {}\n\nFile status:", current_branch);

    let head_hash = refs::read_head_target();
    let commit_tree = read_commit_tree(&head_hash);

    // Assumes read_index() returns: path -> hash
    let index_map: HashMap<String, String> = index::read_index()
        .into_iter()
        .map(|(path, hash)| (normalize_stored_path(&path), hash.trim().to_string()))
        .collect();

    let walker = WalkBuilder::new(root_path)
        .add_custom_ignore_filename(".voorignore")
        .ignore(false)
        .build();

    let mut seen_files: HashSet<String> = HashSet::new();
    let mut has_changes = false;

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        let path_str = normalize_repo_path(root_path, path);

        if path_str.is_empty() || path_str == ".voor" || path_str.starts_with(".voor/") {
            continue;
        }

        seen_files.insert(path_str.clone());

        let full = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };

        let (current_hash, _) = blob_object::get_hash(&full, HashAlgorithm::Sha1);

        match (index_map.get(&path_str), commit_tree.get(&path_str)) {
            // Tracked in index and HEAD
            (Some(index_hash), Some(commit_hash)) => {
                if index_hash == commit_hash {
                    if current_hash != *index_hash {
                        has_changes = true;
                        println!("\t+ Modified: {} (not staged)", path_str);
                    }
                } else {
                    has_changes = true;
                    println!("\t~ Staged: {} (changes to be committed)", path_str);

                    if current_hash != *index_hash {
                        println!("\t+ Modified: {} (not staged)", path_str);
                    }
                }
            }

            // Staged new file
            (Some(index_hash), None) => {
                has_changes = true;
                println!("\t~ Staged: {} (new file staged)", path_str);

                if current_hash != *index_hash {
                    println!("\t+ Modified: {} (not staged)", path_str);
                }
            }

            // Tracked in HEAD but not staged
            (None, Some(commit_hash)) => {
                if current_hash != *commit_hash {
                    has_changes = true;
                    println!("\t+ Modified: {} (not staged)", path_str);
                }
            }

            // Not tracked anywhere
            (None, None) => {
                has_changes = true;
                println!("\t? Untracked: {}", path_str);
            }
        }
    }

    for path in commit_tree.keys() {
        if !seen_files.contains(path) {
            has_changes = true;
            println!("\t- Deleted: {}", path);
        }
    }

    if !has_changes {
        println!("\t= No changes to commit");
    }

}

pub fn changes_not_commited(root_path: &Path) -> bool {
    let head_hash = refs::read_head_target();
    let commit_tree = read_commit_tree(&head_hash);

    // Assumes read_index() returns: path -> hash
    let index_map: HashMap<String, String> = index::read_index()
        .into_iter()
        .map(|(path, hash)| (normalize_stored_path(&path), hash.trim().to_string()))
        .collect();

    let walker = WalkBuilder::new(root_path)
        .add_custom_ignore_filename(".voorignore")
        .ignore(false)
        .build();

    let mut seen_files: HashSet<String> = HashSet::new();
    let mut has_changes = false;

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        let path_str = normalize_repo_path(root_path, path);

        if path_str.is_empty() || path_str == ".voor" || path_str.starts_with(".voor/") {
            continue;
        }

        seen_files.insert(path_str.clone());

        let full = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };

        let (current_hash, _) = blob_object::get_hash(&full, HashAlgorithm::Sha1);

        match (index_map.get(&path_str), commit_tree.get(&path_str)) {
            // Tracked in index and HEAD
            (Some(index_hash), Some(commit_hash)) => {
                if index_hash == commit_hash {
                    if current_hash != *index_hash {
                        has_changes = true;
                    }
                } else {
                    has_changes = true;
                }
            }

            // Staged new file
            (Some(index_hash), None) => {
                has_changes = true;
            }

            // Tracked in HEAD but not staged
            (None, Some(commit_hash)) => {
                if current_hash != *commit_hash {
                    has_changes = true;
                }
            }

            // Not tracked anywhere
            (None, None) => {
                has_changes = true;
            }
        }
    }

    for path in commit_tree.keys() {
        if !seen_files.contains(path) {
            has_changes = true;
        }
    }

    has_changes
}

use crate::utils::blob_object::{self, HashAlgorithm};
use crate::utils::index;
use crate::utils::refs;
use flate2::read::ZlibDecoder;
use ignore::WalkBuilder;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::Path;

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

fn read_loose_object_text(object_type: &str, object_hash: &str) -> String {
    let path = format!(".voor/objects/{}/{}", object_type, object_hash.trim());
    let compressed = fs::read(&path)
        .unwrap_or_else(|_| panic!("Unable to read {} object: {}", object_type, path));

    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();

    decoder
        .read_to_end(&mut decompressed)
        .expect("Unable to decompress object");

    // Remove "<type> <size>\0" header if present
    let content_start = decompressed
        .iter()
        .position(|b| *b == 0)
        .map(|idx| idx + 1)
        .unwrap_or(0);

    String::from_utf8_lossy(&decompressed[content_start..]).into_owned()
}

/// Read the tree referenced by a commit and return:
///     path -> blob_hash
fn read_commit_tree(commit_hash: &str) -> HashMap<String, String> {
    if commit_hash.trim().is_empty() {
        return HashMap::new();
    }

    let commit_content = read_loose_object_text("commit", commit_hash);

    let tree_hash = commit_content
        .lines()
        .find_map(|line| line.strip_prefix("tree "))
        .map(str::trim)
        .unwrap_or("");

    if tree_hash.is_empty() {
        return HashMap::new();
    }

    let tree_content = read_loose_object_text("tree", tree_hash);

    tree_content
        .lines()
        .filter_map(|line| {
            let line = line.trim_end_matches('\r');

            if line.is_empty() {
                return None;
            }

            // New format: "<hash>\t<path>"
            if let Some((hash, path)) = line.split_once('\t') {
                return Some((normalize_stored_path(path), hash.trim().to_string()));
            }

            // Backward compatibility with old format: "<hash> <path>"
            line.split_once(' ')
                .map(|(hash, path)| (normalize_stored_path(path), hash.trim().to_string()))
        })
        .collect()
}

pub fn display_status(root_path: &Path) {
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
                        println!("Modified: {} (not staged)", path_str);
                    }
                } else {
                    has_changes = true;
                    println!("Staged: {} (changes to be committed)", path_str);

                    if current_hash != *index_hash {
                        println!("Modified: {} (not staged)", path_str);
                    }
                }
            }

            // Staged new file
            (Some(index_hash), None) => {
                has_changes = true;
                println!("Staged: {} (new file staged)", path_str);

                if current_hash != *index_hash {
                    println!("Modified: {} (not staged)", path_str);
                }
            }

            // Tracked in HEAD but not staged
            (None, Some(commit_hash)) => {
                if current_hash != *commit_hash {
                    has_changes = true;
                    println!("Modified: {} (not staged)", path_str);
                }
            }

            // Not tracked anywhere
            (None, None) => {
                has_changes = true;
                println!("Untracked: {}", path_str);
            }
        }
    }

    for path in commit_tree.keys() {
        if !seen_files.contains(path) {
            has_changes = true;
            println!("Deleted: {}", path);
        }
    }

    if !has_changes {
        println!("No changes to commit");
    }
}
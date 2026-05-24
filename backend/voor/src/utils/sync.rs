use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};

use crate::utils::fs_ops;
use crate::utils::object_store::{self, ObjectType, TreeEntry};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodedObject {
    pub hash: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRequest {
    pub repo_id: String,
    pub user_id: Option<String>,
    pub branch: String,
    pub head: String,
    pub objects: Vec<EncodedObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResponse {
    pub message: String,
    pub object_count: usize,
    pub database_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub repo_id: String,
    pub user_id: Option<String>,
    pub branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResponse {
    pub branch: String,
    pub head: String,
    pub objects: Vec<EncodedObject>,
    pub database_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDbRequest {
    pub repo_id: String,
    pub user_id: Option<String>,
    pub branch: String,
    pub head: String,
    pub objects: Vec<EncodedObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDbResponse {
    pub message: String,
    pub database_action: Option<String>,
    pub branch_status: Option<String>,
}

pub fn repo_id_from_cwd() -> Result<String, String> {
    let current = std::env::current_dir()
        .map_err(|err| format!("[ERROR] Unable to read current directory: {}", err))?;
    current
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| "[ERROR] Unable to determine repository id from current directory".to_string())
}

pub fn encode_object_for_network(hash: &str) -> Result<EncodedObject, String> {
    let parsed = object_store::read_object(hash)?;
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&parsed.full_bytes)
        .map_err(|err| format!("[ERROR] Unable to compress object '{}': {}", hash, err))?;
    let compressed = encoder
        .finish()
        .map_err(|err| format!("[ERROR] Unable to finalize compression for '{}': {}", hash, err))?;

    Ok(EncodedObject {
        hash: hash.trim().to_string(),
        data: STANDARD.encode(compressed),
    })
}

pub fn decode_object_from_network(encoded: &EncodedObject) -> Result<Vec<u8>, String> {
    let compressed = STANDARD
        .decode(encoded.data.as_bytes())
        .map_err(|err| format!("[ERROR] Unable to decode object '{}': {}", encoded.hash, err))?;

    let mut decoder = ZlibDecoder::new(compressed.as_slice());
    let mut full_bytes = Vec::new();
    decoder
        .read_to_end(&mut full_bytes)
        .map_err(|err| format!("[ERROR] Unable to decompress object '{}': {}", encoded.hash, err))?;

    let computed_hash = object_store::compute_hash(&full_bytes);
    if computed_hash != encoded.hash.trim() {
        return Err(format!(
            "[ERROR] Object hash mismatch for '{}': computed '{}'",
            encoded.hash, computed_hash
        ));
    }

    Ok(full_bytes)
}

pub fn save_received_objects(objects: &[EncodedObject]) -> Result<(), String> {
    for object in objects {
        let full_bytes = decode_object_from_network(object)?;
        object_store::write_full_object(&object.hash, &full_bytes)?;
    }

    Ok(())
}

pub fn collect_related_objects(commit_hash: &str) -> Result<Vec<String>, String> {
    if commit_hash.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut seen = HashSet::new();
    let mut ordered = Vec::new();
    collect_commit_graph(commit_hash.trim(), &mut seen, &mut ordered)?;
    Ok(ordered)
}

fn collect_commit_graph(
    commit_hash: &str,
    seen: &mut HashSet<String>,
    ordered: &mut Vec<String>,
) -> Result<(), String> {
    if !seen.insert(commit_hash.to_string()) {
        return Ok(());
    }

    let parsed = object_store::read_object(commit_hash)?;
    if parsed.object_type != ObjectType::Commit {
        return Err(format!("[ERROR] Expected commit object '{}'", commit_hash));
    }

    ordered.push(commit_hash.to_string());

    let commit_text = String::from_utf8(parsed.content)
        .map_err(|err| format!("[ERROR] Invalid commit '{}': {}", commit_hash, err))?;

    let tree_hash = commit_text
        .lines()
        .find_map(|line| line.strip_prefix("tree "))
        .map(str::trim)
        .ok_or_else(|| format!("[ERROR] Commit '{}' missing tree", commit_hash))?;

    collect_tree_graph(tree_hash, seen, ordered)?;

    for parent_hash in commit_text
        .lines()
        .filter_map(|line| line.strip_prefix("parent "))
        .map(str::trim)
    {
        collect_commit_graph(parent_hash, seen, ordered)?;
    }

    Ok(())
}

fn collect_tree_graph(
    tree_hash: &str,
    seen: &mut HashSet<String>,
    ordered: &mut Vec<String>,
) -> Result<(), String> {
    if !seen.insert(tree_hash.to_string()) {
        return Ok(());
    }

    let parsed = object_store::read_object(tree_hash)?;
    if parsed.object_type != ObjectType::Tree {
        return Err(format!("[ERROR] Expected tree object '{}'", tree_hash));
    }

    ordered.push(tree_hash.to_string());

    for entry in object_store::parse_tree(&parsed.content)? {
        match entry.object_type {
            ObjectType::Blob => {
                if !seen.contains(&entry.hash) {
                    let blob = object_store::read_object(&entry.hash)?;
                    if blob.object_type != ObjectType::Blob {
                        return Err(format!("[ERROR] Expected blob object '{}'", entry.hash));
                    }
                    seen.insert(entry.hash.clone());
                    ordered.push(entry.hash);
                }
            }
            ObjectType::Tree => collect_tree_graph(&entry.hash, seen, ordered)?,
            ObjectType::Commit => {}
        }
    }

    Ok(())
}

pub fn collect_encoded_objects(commit_hash: &str) -> Result<Vec<EncodedObject>, String> {
    collect_related_objects(commit_hash)?
        .into_iter()
        .map(|hash| encode_object_for_network(&hash))
        .collect()
}

pub fn read_commit_tree(commit_hash: &str) -> Result<Vec<(String, String)>, String> {
    if commit_hash.trim().is_empty() {
        return Ok(Vec::new());
    }

    let parsed = object_store::read_object(commit_hash)?;
    if parsed.object_type != ObjectType::Commit {
        return Err(format!("[ERROR] Expected commit object '{}'", commit_hash));
    }

    let commit_text = String::from_utf8(parsed.content)
        .map_err(|err| format!("[ERROR] Invalid commit '{}': {}", commit_hash, err))?;

    let tree_hash = commit_text
        .lines()
        .find_map(|line| line.strip_prefix("tree "))
        .map(str::trim)
        .ok_or_else(|| format!("[ERROR] Commit '{}' missing tree", commit_hash))?;

    let mut files = Vec::new();
    flatten_tree(tree_hash, Path::new(""), &mut files)?;
    Ok(files)
}

fn flatten_tree(
    tree_hash: &str,
    prefix: &Path,
    files: &mut Vec<(String, String)>,
) -> Result<(), String> {
    let parsed = object_store::read_object(tree_hash)?;
    for entry in object_store::parse_tree(&parsed.content)? {
        let path = prefix.join(&entry.name);
        match entry.object_type {
            ObjectType::Blob => files.push((normalize_path(&path), entry.hash)),
            ObjectType::Tree => flatten_tree(&entry.hash, &path, files)?,
            ObjectType::Commit => {}
        }
    }

    Ok(())
}

pub fn restore_working_tree(current_commit: &str, target_commit: &str) -> Result<(), String> {
    let current_files = read_commit_tree(current_commit)?;
    let target_files = read_commit_tree(target_commit)?;

    let current_set: HashSet<String> = current_files.into_iter().map(|(path, _)| path).collect();
    let target_map: HashMap<String, String> = target_files.into_iter().collect();

    for path in current_set {
        if !target_map.contains_key(&path) && Path::new(&path).exists() {
            fs::remove_file(&path)
                .map_err(|err| format!("[ERROR] Unable to remove '{}': {}", path, err))?;
            remove_empty_parents(Path::new(&path));
        }
    }

    for (path, hash) in target_map {
        let blob = object_store::read_object(&hash)?;
        if blob.object_type != ObjectType::Blob {
            return Err(format!("[ERROR] Expected blob object '{}'", hash));
        }

        if let Some(parent) = Path::new(&path).parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("[ERROR] Unable to create '{}': {}", parent.display(), err))?;
        }

        fs_ops::write_file_atomic(&path, &blob.content)
            .map_err(|err| format!("[ERROR] Unable to restore '{}': {}", path, err))?;
    }

    fs_ops::write_file_atomic(".voor/index", b"")
        .map_err(|err| format!("[ERROR] Unable to clear index: {}", err))?;
    Ok(())
}

fn remove_empty_parents(path: &Path) {
    let mut current = path.parent();

    while let Some(directory) = current {
        if directory == Path::new(".") || directory == Path::new("") || directory.starts_with(".voor") {
            break;
        }

        if fs::remove_dir(directory).is_err() {
            break;
        }

        current = directory.parent();
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn build_tree_from_index(entries: &[(String, String)]) -> Result<String, String> {
    build_tree_recursive(Path::new(""), entries)
}

fn build_tree_recursive(prefix: &Path, entries: &[(String, String)]) -> Result<String, String> {
    let mut tree_entries = Vec::new();
    let mut directories = HashSet::new();

    for (path, hash) in entries {
        let full_path = Path::new(path);
        let relative = if prefix.as_os_str().is_empty() {
            full_path
        } else {
            match full_path.strip_prefix(prefix) {
                Ok(relative) => relative,
                Err(_) => continue,
            }
        };

        let mut components = relative.components();
        let Some(first) = components.next() else {
            return Err(format!("[ERROR] Invalid staged path '{}'", path));
        };
        let name = first.as_os_str().to_string_lossy().to_string();

        if components.next().is_some() {
            directories.insert(name);
        } else {
            tree_entries.push(TreeEntry {
                mode: "100644".to_string(),
                name,
                hash: hash.clone(),
                object_type: ObjectType::Blob,
            });
        }
    }

    for directory in directories {
        let child_prefix = prefix.join(&directory);
        let child_hash = build_tree_recursive(&child_prefix, entries)?;
        tree_entries.push(TreeEntry {
            mode: "40000".to_string(),
            name: directory,
            hash: child_hash,
            object_type: ObjectType::Tree,
        });
    }

    tree_entries.sort_by(|left, right| left.name.cmp(&right.name));
    let content = object_store::serialize_tree(&tree_entries)?;
    object_store::write_object(ObjectType::Tree, &content)
}

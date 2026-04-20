use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::utils::fs_ops;

fn normalize_path_str(path: &str) -> String {
    let mut normalized = path.trim().replace('\\', "/");

    while normalized.starts_with("./") {
        normalized = normalized[2..].to_string();
    }

    normalized
}

pub fn normalize_path(path: &Path) -> String {
    normalize_path_str(&path.to_string_lossy())
}

/// Writes/updates one index entry.
/// On-disk format:
///     <hash>\t<path>\n
///
/// Using '\t' avoids breaking paths that contain spaces.
pub fn write_index(hash: &str, path: &Path) {
    let mut index_map = read_index();
    index_map.insert(normalize_path(path), hash.trim().to_string());

    let mut entries: Vec<(String, String)> = index_map.into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut content = String::new();
    for (path, hash) in entries {
        content.push_str(&format!("{}\t{}\n", hash.trim(), path));
    }

    fs_ops::write_file_atomic(".voor/index", content.as_bytes()).expect("[ERROR] Unable to write index");
}

/// Returns:
///     path -> hash
pub fn read_index() -> HashMap<String, String> {
    let mut map = HashMap::new();
    let content = fs::read_to_string(".voor/index").unwrap_or_default();

    for raw_line in content.lines() {
        let line = raw_line.trim_end_matches('\r');

        if line.is_empty() {
            continue;
        }

        // Preferred format: "<hash>\t<path>"
        if let Some((hash, path)) = line.split_once('\t') {
            map.insert(normalize_path_str(path), hash.trim().to_string());
            continue;
        }

        // Backward compatibility with old format: "<hash> <path>"
        // split only on the first whitespace so paths with spaces survive
        if let Some(idx) = line.find(char::is_whitespace) {
            let hash = line[..idx].trim();
            let path = line[idx..].trim();

            if !hash.is_empty() && !path.is_empty() {
                map.insert(normalize_path_str(path), hash.to_string());
            }
        }
    }

    map
}

use std::path::Path;
use std::fs::OpenOptions;
use std::io::Write;
use std::collections::HashMap;
use std::fs;

pub fn write_index(hash: &str, path: &Path) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(".voor/index")
        .unwrap();

    writeln!(file, "{} {}", hash, path.display()).unwrap();
}

pub fn read_index() -> HashMap<String, String> {
    let mut map = HashMap::new();

    let content = fs::read_to_string(".voor/index").unwrap_or_default();

    for line in content.lines() {
        let mut parts = line.split_whitespace();

        if let (Some(hash), Some(path)) = (parts.next(), parts.next()) {
            map.insert(path.to_string(), hash.to_string());
        }
    }

    map
}
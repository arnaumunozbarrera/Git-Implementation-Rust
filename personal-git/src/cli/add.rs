use std::path::Path;
use ignore::WalkBuilder;

use crate::cli::hash_object;
use crate::utils::add;

pub fn add_by_hash(path: &Path) {
    let hash = hash_object::hash_object_command("--sha256", path.to_str().unwrap());

    add::write_index(&hash, path);

    println!("[INFO] File staged to .voor/index: {}\n", path.display());
}

pub fn add_all(root_path: &Path) {
    let walker = WalkBuilder::new(root_path)
        .add_custom_ignore_filename(".voorignore")
        .build();

    for entry in walker {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.starts_with(".voor") {
            continue;
        }

        if path.is_file() {
            add_by_hash(path);
        }
    }
}
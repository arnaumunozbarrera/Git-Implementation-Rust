use std::fs;
use std::path::Path;
use ignore::WalkBuilder;

use crate::utils::blob_object::{self, HashAlgorithm};
use crate::utils::add;

pub fn display_status(root_path: &Path) {

    let walker = WalkBuilder::new(root_path)
        .add_custom_ignore_filename(".voorignore")
        .ignore(false)
        .build();

    let index = add::read_index();

    for entry in walker {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.starts_with(".voor") {
            continue;
        }

        if path.is_dir() {
            continue;
        }

       let path_str = path.to_string_lossy();

        if let Some(index_hash) = index.get(path_str.as_ref()) {

            let full = fs::read(path).unwrap();
            let (current_hash, _) = blob_object::get_hash(&full, HashAlgorithm::Sha256);

            if &current_hash == index_hash {
                println!("Tracked: {} (clean)", path.display());
            } else {
                println!("Modified: {}", path.display());
            }

        } else {
            println!("Untracked: {}", path.display());
        }
    }

    for (path, _) in &index {
    if !Path::new(path).exists() {
        println!("Deleted: {}", path);
    }
}
}
use std::fs;
use std::path::Path;
use ignore::WalkBuilder;

use crate::utils::blob_object::{self, HashAlgorithm};

pub fn display_status(root_path: &Path) {

    let walker = WalkBuilder::new(root_path)
        .add_custom_ignore_filename(".voorignore")
        .ignore(false)
        .build();

    for entry in walker {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.starts_with(".voor") {
            continue;
        }

        if path.is_file() {

            let full = fs::read(path).unwrap();

            let (hash, _) = blob_object::get_hash(&full, HashAlgorithm::Sha256);

            let dir = &hash[..2];
            let file = &hash[2..];

            let object_path = Path::new(".voor")
                .join("objects")
                .join(dir)
                .join(file);

            if !object_path.exists() {
                println!("Untracked: {}", path.display());
            } else {

                let old_bytes =
                    crate::utils::file_object::read_blob_content(object_path.to_str().unwrap());

                if old_bytes == full {
                    println!("Tracked: {} (clean)", path.display());
                } else {
                    println!("Modified: {}", path.display());
                }
            }
        }
    }
}
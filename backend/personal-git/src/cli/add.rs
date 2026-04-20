use std::fs;
use std::path::Path;

use ignore::WalkBuilder;

use crate::cli::branch;
use crate::utils::blob_object::{self, HashAlgorithm};
use crate::utils::fs_ops;
use crate::utils::index;

pub fn add_by_hash(path: &Path) {
    if let Err(error) = fs_ops::with_repo_lock("add", || add_by_hash_locked(path)) {
        println!("{}", error);
    }
}

pub fn add_all(root_path: &Path) {
    let result = fs_ops::with_repo_lock("add-all", || {
        let current_branch = branch::get_current_branch();
        println!("[INFO] On branch: {:?}\n\nFile status:", current_branch);

        let walker = WalkBuilder::new(root_path)
            .add_custom_ignore_filename(".voorignore")
            .build();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let normalized = index::normalize_path(path);
            if normalized == ".voor" || normalized.starts_with(".voor/") {
                continue;
            }

            add_by_hash_locked(path)?;
        }

        Ok(())
    });

    if let Err(error) = result {
        println!("{}", error);
    }
}

fn add_by_hash_locked(path: &Path) -> Result<(), String> {
    let file_bytes = fs::read(path).expect("[ERROR] Unable to read file for staging");
    let (hash, full_blob_content) = blob_object::get_hash(&file_bytes, HashAlgorithm::Sha1);
    let (dir, file) = hash.split_at(2);
    blob_object::save_compressed_object(dir, file, &full_blob_content);
    index::write_index(&hash, path);

    println!("[INFO] File staged to .voor/index: {}", index::normalize_path(path));
    Ok(())
}

use std::fs;
use std::path::Path;

use crate::utils::fs_ops;

pub fn init_command() {
    let result = fs_ops::with_repo_lock("init", || ensure_repo_layout());

    match result {
        Ok(InitStatus::Created) => println!("[INFO] `.voor` directory initialized successfully!\n"),
        Ok(InitStatus::Repaired) => println!("[INFO] `.voor` directory repaired successfully!\n"),
        Ok(InitStatus::AlreadyInitialized) => println!("[INFO] `.voor` directory already initialized successfully\n"),
        Err(error) => println!("{}", error),
    }
}

enum InitStatus {
    Created,
    Repaired,
    AlreadyInitialized,
}

fn ensure_repo_layout() -> Result<InitStatus, String> {
    let repo_exists = Path::new(".voor").exists();
    let mut repaired = false;

    if !repo_exists {
        fs::create_dir(".voor").map_err(|error| format!("[ERROR] Unable to create .voor: {}", error))?;
    }

    for directory in [".voor/objects", ".voor/refs", ".voor/refs/heads", ".voor/locks"] {
        if !Path::new(directory).exists() {
            fs::create_dir_all(directory)
                .map_err(|error| format!("[ERROR] Unable to create '{}': {}", directory, error))?;
            if repo_exists {
                repaired = true;
            }
        }
    }

    repaired |= ensure_file(".voor/refs/heads/master", b"")?;
    repaired |= ensure_file(".voor/HEAD", b"ref: refs/heads/master")?;
    repaired |= ensure_file(".voor/index", b"")?;
    repaired |= ensure_file(".voor/config", b"[remote \"origin\"]\nurl = http://localhost:3000\n")?;
    repaired |= ensure_file(".voorignore", b".env\n\n.voor/\n/.voor/\n\nCargo.lock\nCargo.toml")?;

    if !repo_exists {
        Ok(InitStatus::Created)
    } else if repaired {
        Ok(InitStatus::Repaired)
    } else {
        Ok(InitStatus::AlreadyInitialized)
    }
}

fn ensure_file(path: &str, content: &[u8]) -> Result<bool, String> {
    if Path::new(path).exists() {
        return Ok(false);
    }

    fs_ops::write_file_atomic(path, content)?;
    Ok(true)
}

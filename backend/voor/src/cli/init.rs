use std::fs;
use std::path::Path;

use crate::utils::fs_ops;

const DEFAULT_REMOTE_URL: &str = "http://localhost:3000";

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
    repaired |= ensure_default_remote_config()?;
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

fn ensure_default_remote_config() -> Result<bool, String> {
    let path = ".voor/config";
    if !Path::new(path).exists() {
        fs_ops::write_file_atomic(
            path,
            format!("[remote \"origin\"]\nurl = {}\n", DEFAULT_REMOTE_URL).as_bytes(),
        )?;
        return Ok(true);
    }

    let content = fs::read_to_string(path)
        .map_err(|error| format!("[ERROR] Unable to read '{}': {}", path, error))?;
    let mut changed = false;
    let mut saw_url = false;
    let mut lines = Vec::new();

    for line in content.lines() {
        if line.trim_start().starts_with("url = ") {
            saw_url = true;
            let replacement = format!("url = {}", DEFAULT_REMOTE_URL);
            if line.trim() != replacement {
                changed = true;
            }
            lines.push(replacement);
        } else {
            lines.push(line.to_string());
        }
    }

    if !saw_url {
        lines.insert(1.min(lines.len()), format!("url = {}", DEFAULT_REMOTE_URL));
        changed = true;
    }

    if changed {
        let mut normalized = lines.join("\n");
        normalized.push('\n');
        fs_ops::write_file_atomic(path, normalized.as_bytes())?;
    }

    Ok(changed)
}

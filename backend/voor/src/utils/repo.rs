use std::env;
use std::path::{Path, PathBuf};

pub fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut current = if start.is_absolute() {
        start.to_path_buf()
    } else {
        env::current_dir().ok()?.join(start)
    };

    if current.is_file() {
        current = current.parent()?.to_path_buf();
    }

    loop {
        if current.join(".voor").is_dir() {
            return Some(current);
        }

        if !current.pop() {
            return None;
        }
    }
}

pub fn switch_to_repo_root() -> Result<PathBuf, String> {
    let current = env::current_dir()
        .map_err(|error| format!("[ERROR] Unable to read current directory: {}", error))?;
    let repo_root = find_repo_root(&current).ok_or_else(|| {
        "[ERROR] Not a voor repository (or any parent directory up to the filesystem root)".to_string()
    })?;

    env::set_current_dir(&repo_root)
        .map_err(|error| format!("[ERROR] Unable to switch to repository root '{}': {}", repo_root.display(), error))?;

    Ok(repo_root)
}

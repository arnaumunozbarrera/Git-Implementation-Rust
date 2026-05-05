use std::fs;
use std::path::PathBuf;

use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use crate::utils::fs_ops;

const APP_NAME: &str = "voor";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserConfig {
    pub auth_token: Option<String>,
    pub default_remote: Option<String>,
}

pub fn load_user_config() -> Result<UserConfig, String> {
    let path = user_config_path()?;
    if !path.exists() {
        return Ok(UserConfig::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("[ERROR] Unable to read global config '{}': {}", path.display(), error))?;

    toml::from_str::<UserConfig>(&content)
        .map_err(|error| format!("[ERROR] Invalid global config '{}': {}", path.display(), error))
}

pub fn save_user_config(config: &UserConfig) -> Result<(), String> {
    let path = user_config_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| "[ERROR] Unable to determine global config directory".to_string())?;

    fs::create_dir_all(parent)
        .map_err(|error| format!("[ERROR] Unable to create global config directory '{}': {}", parent.display(), error))?;

    let content = toml::to_string(config)
        .map_err(|error| format!("[ERROR] Unable to serialize global config: {}", error))?;

    fs_ops::write_file_atomic(&path, content.as_bytes())
        .map_err(|error| format!("[ERROR] Unable to write global config '{}': {}", path.display(), error))
}

pub fn user_config_path() -> Result<PathBuf, String> {
    let dirs = BaseDirs::new()
        .ok_or_else(|| "[ERROR] Unable to determine platform config directory".to_string())?;

    Ok(dirs.config_dir().join(APP_NAME).join("config.toml"))
}

use std::env;
use std::fs;

use reqwest::blocking::Client;
use reqwest::StatusCode;

use crate::api::models::{InitRepoRequest, InitRepoResponse};
use crate::cli::branch;
use crate::utils::fs_ops;
use crate::utils::refs;
use crate::utils::sync::{
    self, PullRequest, PullResponse, PushRequest, PushResponse, SyncDbRequest, SyncDbResponse,
};

const CONFIG_PATH: &str = ".voor/config";

#[derive(Default)]
struct RemoteConfig {
    url: Option<String>,
    repo_id: Option<String>,
    user_id: Option<String>,
}

pub fn set_remote(url: &str) {
    let result = fs_ops::with_repo_lock("remote-set", || {
        let mut config = read_config().unwrap_or_default();
        config.url = Some(url.trim().trim_end_matches('/').to_string());
        write_config(&config)
    });

    match result {
        Ok(_) => println!("[INFO] Remote 'origin' set to {}", url.trim()),
        Err(error) => println!("{}", error),
    }
}

pub fn init_remote(user_id: &str, branch_name: Option<&str>) {
    let result = fs_ops::with_repo_lock("init-remote", || init_remote_locked(user_id, branch_name));
    if let Err(error) = result {
        println!("{}", error);
    }
}

pub fn push_branch(branch_name: &str) {
    let result = fs_ops::with_repo_lock("push-snapshot", || push_branch_locked(branch_name));
    if let Err(error) = result {
        println!("{}", error);
    }
}

pub fn pull_branch(branch_name: &str) {
    let result = fs_ops::with_repo_lock("pull", || pull_branch_locked(branch_name));
    if let Err(error) = result {
        println!("{}", error);
    }
}

pub fn sync_db(branch_name: Option<&str>, user_id: Option<&str>) {
    let result = fs_ops::with_repo_lock("sync-db", || {
        let user_id = resolve_user_id(user_id)?;
        let branch_name = branch::current_branch_or(branch_name);
        sync_db_internal(&branch_name, &user_id, true)
    });

    if let Err(error) = result {
        println!("{}", error);
    }
}

fn init_remote_locked(user_id: &str, branch_name: Option<&str>) -> Result<(), String> {
    let repo_id = repo_id_from_config_or_cwd()?;
    let remote = get_remote_url()?;
    let branch_name = branch::current_branch_or(branch_name);
    let head = read_branch_head(&branch_name).unwrap_or_default();
    let objects = if head.is_empty() {
        Vec::new()
    } else {
        sync::collect_encoded_objects(&head)?
    };

    let response = Client::new()
        .post(format!("{}/repos/init", remote))
        .json(&InitRepoRequest {
            repo_id: repo_id.clone(),
            name: repo_id.clone(),
            owner_id: user_id.trim().to_string(),
            default_branch: branch_name.clone(),
            is_private: false,
            description: Some(format!("Remote repository for {}", repo_id)),
            readme_path: Some("README.md".to_string()),
            tags: Some(vec!["rust".to_string(), "git".to_string()]),
            theme: None,
            head: if head.is_empty() { None } else { Some(head.clone()) },
            objects: if objects.is_empty() { None } else { Some(objects.clone()) },
        })
        .send()
        .map_err(|_| "[ERROR] Remote repository initialization request failed".to_string())?;

    if response.status().is_success() {
        match response.json::<InitRepoResponse>() {
            Ok(result) => {
                println!("[INFO] {}", result.message);
                print_database_action(result.database_action.as_deref());
            }
            Err(error) => {
                return Err(format!("[ERROR] Invalid init-remote response: {}", error));
            }
        }
    } else {
        let status = response.status();
        let body = response.text().unwrap_or_default();

        if status != StatusCode::CONFLICT {
            return Err(format!("[ERROR] init-remote failed ({}): {}", status, body));
        }

        println!("[WARN] {}", body);
    }

    persist_repo_mapping(&repo_id, user_id)?;
    println!("[INFO] Saved repo_id '{}' and user_id '{}' in .voor/config", repo_id, user_id.trim());

    if !head.is_empty() {
        sync_db_internal(&branch_name, user_id, false)?;
    } else {
        println!(
            "[INFO] Branch '{}' has no commits yet; only remote repository metadata was initialized",
            branch_name
        );
    }

    Ok(())
}

fn push_branch_locked(branch_name: &str) -> Result<(), String> {
    let branch_name = branch::current_branch_or(Some(branch_name));
    let head = read_branch_head(&branch_name)?;
    let remote = get_remote_url()?;
    let repo_id = repo_id_from_config_or_cwd()?;
    let user_id = resolve_user_id(None)?;
    let objects = sync::collect_encoded_objects(&head)?;

    let response = Client::new()
        .post(format!("{}/push", remote))
        .json(&PushRequest {
            repo_id,
            user_id,
            branch: branch_name.clone(),
            head,
            objects,
        })
        .send()
        .map_err(|_| "[ERROR] Push request failed".to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(format!("[ERROR] Push failed ({}): {}", status, body));
    }

    match response.json::<PushResponse>() {
        Ok(result) => {
            println!("[INFO] {}", result.message);
            println!("[INFO] Sent {} objects", result.object_count);
            print_database_action(result.database_action.as_deref());
            Ok(())
        }
        Err(error) => Err(format!("[ERROR] Invalid push response: {}", error)),
    }
}

fn pull_branch_locked(branch_name: &str) -> Result<(), String> {
    let branch_name = branch::current_branch_or(Some(branch_name));
    let remote = get_remote_url()?;
    let repo_id = repo_id_from_config_or_cwd()?;
    let user_id = resolve_user_id(None)?;
    let current_head = refs::read_head_target();

    let response = Client::new()
        .post(format!("{}/pull", remote))
        .json(&PullRequest {
            repo_id,
            user_id,
            branch: branch_name,
        })
        .send()
        .map_err(|_| "[ERROR] Pull request failed".to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(format!("[ERROR] Pull failed ({}): {}", status, body));
    }

    let result = response
        .json::<PullResponse>()
        .map_err(|error| format!("[ERROR] Invalid pull response: {}", error))?;

    sync::save_received_objects(&result.objects)?;
    refs::update_ref(&format!("refs/heads/{}", result.branch), &result.head);

    if refs::get_head_ref() == format!("refs/heads/{}", result.branch) {
        sync::restore_working_tree(&current_head, &result.head)?;
    }

    println!("[INFO] Pulled branch '{}' at {}", result.branch, result.head);
    println!("[INFO] Received {} objects", result.objects.len());
    print_database_action(result.database_action.as_deref());
    Ok(())
}

fn sync_db_internal(branch_name: &str, user_id: &str, print_prefix: bool) -> Result<(), String> {
    let head = read_branch_head(branch_name)?;
    let remote = get_remote_url()?;
    let repo_id = repo_id_from_config_or_cwd()?;
    let objects = sync::collect_encoded_objects(&head)?;

    let response = Client::new()
        .post(format!("{}/sync-db", remote))
        .json(&SyncDbRequest {
            repo_id,
            user_id: user_id.trim().to_string(),
            branch: branch_name.to_string(),
            head,
            objects,
        })
        .send()
        .map_err(|_| "[ERROR] sync-db request failed".to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(format!("[ERROR] sync-db failed ({}): {}", status, body));
    }

    match response.json::<SyncDbResponse>() {
        Ok(result) => {
            if print_prefix {
                println!("[INFO] {}", result.message);
            } else {
                println!("[INFO] {}", result.message);
            }
            print_database_action(result.database_action.as_deref());
            if let Some(branch_status) = result.branch_status.as_deref() {
                println!("[WARN] {}", branch_status);
            }
            Ok(())
        }
        Err(error) => Err(format!("[ERROR] Invalid sync-db response: {}", error)),
    }
}

fn get_remote_url() -> Result<String, String> {
    read_config()?
        .url
        .ok_or_else(|| "[ERROR] Missing remote url in .voor/config".to_string())
}

fn repo_id_from_config_or_cwd() -> Result<String, String> {
    if let Some(repo_id) = read_config()?.repo_id {
        return Ok(repo_id);
    }

    sync::repo_id_from_cwd()
}

fn resolve_user_id(explicit_user_id: Option<&str>) -> Result<String, String> {
    if let Some(user_id) = explicit_user_id.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(user_id.to_string());
    }

    if let Some(user_id) = read_config()?.user_id {
        return Ok(user_id);
    }

    if let Ok(user_id) = env::var("SYNC_LOG_USER_ID") {
        if !user_id.trim().is_empty() {
            return Ok(user_id.trim().to_string());
        }
    }

    Err("[ERROR] Missing user_id. Run `cargo run -- init-remote <user_id>` or set SYNC_LOG_USER_ID".to_string())
}

fn persist_repo_mapping(repo_id: &str, user_id: &str) -> Result<(), String> {
    let mut config = read_config().unwrap_or_default();
    if config.url.is_none() {
        config.url = Some("http://localhost:3000".to_string());
    }
    config.repo_id = Some(repo_id.trim().to_string());
    config.user_id = Some(user_id.trim().to_string());
    write_config(&config)
}

fn read_config() -> Result<RemoteConfig, String> {
    let content = fs::read_to_string(CONFIG_PATH)
        .map_err(|_| "[ERROR] Missing remote config in .voor/config".to_string())?;

    let mut config = RemoteConfig::default();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("url = ") {
            config.url = Some(value.trim().trim_end_matches('/').to_string());
        } else if let Some(value) = trimmed.strip_prefix("repo_id = ") {
            config.repo_id = Some(value.trim().to_string());
        } else if let Some(value) = trimmed.strip_prefix("user_id = ") {
            config.user_id = Some(value.trim().to_string());
        }
    }

    Ok(config)
}

fn write_config(config: &RemoteConfig) -> Result<(), String> {
    let url = config
        .url
        .as_deref()
        .unwrap_or("http://localhost:3000")
        .trim()
        .trim_end_matches('/')
        .to_string();

    let mut content = format!("[remote \"origin\"]\nurl = {}\n", url);
    if let Some(repo_id) = config.repo_id.as_deref() {
        content.push_str(&format!("repo_id = {}\n", repo_id.trim()));
    }
    if let Some(user_id) = config.user_id.as_deref() {
        content.push_str(&format!("user_id = {}\n", user_id.trim()));
    }

    fs_ops::write_file_atomic(CONFIG_PATH, content.as_bytes())
        .map_err(|error| format!("[ERROR] Unable to write config: {}", error))
}

fn read_branch_head(branch_name: &str) -> Result<String, String> {
    let path = format!(".voor/refs/heads/{}", branch_name.trim());
    let content = fs::read_to_string(&path)
        .map_err(|_| format!("[ERROR] Missing branch '{}'", branch_name.trim()))?;
    Ok(content.trim().to_string())
}

fn print_database_action(database_action: Option<&str>) {
    if let Some(action) = database_action {
        println!("[INFO] {}", action);
    }
}

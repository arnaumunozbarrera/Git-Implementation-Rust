use std::env;
use std::fs;

use reqwest::blocking::Client;
use reqwest::StatusCode;

use crate::api::models::{InitRepoRequest, InitRepoResponse};
use crate::cli::branch;
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
    let mut config = read_config().unwrap_or_default();
    config.url = Some(url.trim().trim_end_matches('/').to_string());

    match write_config(&config) {
        Ok(_) => println!("[INFO] Remote 'origin' set to {}", url.trim()),
        Err(error) => println!("{}", error),
    }
}

pub fn init_remote(user_id: &str, branch_name: Option<&str>) {
    let repo_id = match repo_id_from_config_or_cwd() {
        Ok(repo_id) => repo_id,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };
    let remote = match get_remote_url() {
        Ok(remote) => remote,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };
    let branch_name = branch::current_branch_or(branch_name);
    let head = read_branch_head(&branch_name).unwrap_or_default();
    let objects = if head.is_empty() {
        Vec::new()
    } else {
        match sync::collect_encoded_objects(&head) {
            Ok(objects) => objects,
            Err(error) => {
                println!("{}", error);
                return;
            }
        }
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
            objects: if objects.is_empty() {
                None
            } else {
                Some(objects.clone())
            },
        })
        .send();

    let Ok(response) = response else {
        println!("[ERROR] Remote repository initialization request failed");
        return;
    };

    if response.status().is_success() {
        match response.json::<InitRepoResponse>() {
            Ok(result) => {
                println!("[INFO] {}", result.message);
                print_database_action(result.database_action.as_deref());
            }
            Err(error) => {
                println!("[ERROR] Invalid init-remote response: {}", error);
                return;
            }
        }
    } else {
        let status = response.status();
        let body = response.text().unwrap_or_default();

        if status != StatusCode::CONFLICT {
            println!("[ERROR] init-remote failed ({}): {}", status, body);
            return;
        }

        println!("[WARN] {}", body);
    }

    if let Err(error) = persist_repo_mapping(&repo_id, user_id) {
        println!("{}", error);
        return;
    }

    println!("[INFO] Saved repo_id '{}' and user_id '{}' in .voor/config", repo_id, user_id.trim());

    if !head.is_empty() {
        sync_db_internal(&branch_name, user_id, false);
    } else {
        println!(
            "[INFO] Branch '{}' has no commits yet; only remote repository metadata was initialized",
            branch_name
        );
    }
}

pub fn push_branch(branch_name: &str) {
    let branch_name = branch::current_branch_or(Some(branch_name));
    let head = match read_branch_head(&branch_name) {
        Ok(head) => head,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };
    let remote = match get_remote_url() {
        Ok(remote) => remote,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };
    let repo_id = match repo_id_from_config_or_cwd() {
        Ok(repo_id) => repo_id,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };
    let user_id = match resolve_user_id(None) {
        Ok(user_id) => user_id,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };

    let objects = match sync::collect_encoded_objects(&head) {
        Ok(objects) => objects,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };

    let response = Client::new()
        .post(format!("{}/push", remote))
        .json(&PushRequest {
            repo_id,
            user_id,
            branch: branch_name.clone(),
            head,
            objects,
        })
        .send();

    let Ok(response) = response else {
        println!("[ERROR] Push request failed");
        return;
    };

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        println!("[ERROR] Push failed ({}): {}", status, body);
        return;
    }

    match response.json::<PushResponse>() {
        Ok(result) => {
            println!("[INFO] {}", result.message);
            println!("[INFO] Sent {} objects", result.object_count);
            print_database_action(result.database_action.as_deref());
        }
        Err(error) => println!("[ERROR] Invalid push response: {}", error),
    }
}

pub fn pull_branch(branch_name: &str) {
    let branch_name = branch::current_branch_or(Some(branch_name));
    let remote = match get_remote_url() {
        Ok(remote) => remote,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };
    let repo_id = match repo_id_from_config_or_cwd() {
        Ok(repo_id) => repo_id,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };
    let user_id = match resolve_user_id(None) {
        Ok(user_id) => user_id,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };

    let branch_dbg = branch_name.clone();
    let repo_dbg = repo_id.clone();

    let current_head = refs::read_head_target();
    let response = Client::new()
        .post(format!("{}/pull", remote))
        .json(&PullRequest {
            repo_id,
            user_id,
            branch: branch_name,
        })
        .send();

    println!("[DEBUG] branch = {:?} (len={})", branch_dbg, branch_dbg.len());
    println!("[DEBUG] repo_id = {:?} (len={})", repo_dbg, repo_dbg.len());

    let Ok(response) = response else {
        println!("[ERROR] Pull request failed");
        return;
    };

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        println!("[ERROR] Pull failed ({}): {}", status, body);
        return;
    }

    let result = match response.json::<PullResponse>() {
        Ok(result) => result,
        Err(error) => {
            println!("[ERROR] Invalid pull response: {}", error);
            return;
        }
    };

    if let Err(error) = sync::save_received_objects(&result.objects) {
        println!("{}", error);
        return;
    }

    refs::update_ref(&format!("refs/heads/{}", result.branch), &result.head);

    if refs::get_head_ref() == format!("refs/heads/{}", result.branch) {
        if let Err(error) = sync::restore_working_tree(&current_head, &result.head) {
            println!("{}", error);
            return;
        }
    }

    println!("[INFO] Pulled branch '{}' at {}", result.branch, result.head);
    println!("[INFO] Received {} objects", result.objects.len());
    print_database_action(result.database_action.as_deref());
}

pub fn sync_db(branch_name: Option<&str>, user_id: Option<&str>) {
    let user_id = match resolve_user_id(user_id) {
        Ok(user_id) => user_id,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };

    let branch_name = branch::current_branch_or(branch_name);
    sync_db_internal(&branch_name, &user_id, true);
}

fn sync_db_internal(branch_name: &str, user_id: &str, print_prefix: bool) {
    let head = match read_branch_head(branch_name) {
        Ok(head) => head,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };
    let remote = match get_remote_url() {
        Ok(remote) => remote,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };
    let repo_id = match repo_id_from_config_or_cwd() {
        Ok(repo_id) => repo_id,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };
    let objects = match sync::collect_encoded_objects(&head) {
        Ok(objects) => objects,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };

    let response = Client::new()
        .post(format!("{}/sync-db", remote))
        .json(&SyncDbRequest {
            repo_id,
            user_id: user_id.trim().to_string(),
            branch: branch_name.to_string(),
            head,
            objects,
        })
        .send();

    let Ok(response) = response else {
        println!("[ERROR] sync-db request failed");
        return;
    };

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        println!("[ERROR] sync-db failed ({}): {}", status, body);
        return;
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
        }
        Err(error) => println!("[ERROR] Invalid sync-db response: {}", error),
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

    fs::write(CONFIG_PATH, content).map_err(|error| format!("[ERROR] Unable to write config: {}", error))
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

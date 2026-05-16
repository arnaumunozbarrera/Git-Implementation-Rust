use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use reqwest::blocking::{Client, RequestBuilder};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::api::models::{InitRepoRequest, InitRepoResponse};
use crate::cli::branch;
use crate::utils::app_config;
use crate::utils::fs_ops;
use crate::utils::refs;
use crate::utils::sync::{
    self, PullRequest, PullResponse, PushRequest, PushResponse, SyncDbRequest, SyncDbResponse,
};

const CONFIG_PATH: &str = ".voor/config";
const DEFAULT_REMOTE_URL: &str = "http://localhost:3000";
const DEFAULT_FRONTEND_URL: &str = "http://localhost:5173";

#[derive(Default)]
struct RemoteConfig {
    url: Option<String>,
    repo_id: Option<String>,
    user_id: Option<String>,
    auth_token: Option<String>,
}

pub fn set_remote(url: &str) {
    let result = fs_ops::with_repo_lock("remote-set", || {
        let mut config = read_config().unwrap_or_default();
        config.url = Some(normalize_remote_url(url));
        write_config(&config)
    });

    match result {
        Ok(_) => println!("[INFO] Remote 'origin' set to {}", url.trim()),
        Err(error) => println!("{}", error),
    }
}

pub fn login(token: Option<&str>) {
    let result = match token {
        Some(token) => persist_auth_token(token),
        None => login_with_browser(true).and_then(|token| persist_auth_token(&token)),
    };

    match result {
        Ok(path) => println!("[INFO] Stored auth token in {}", path),
        Err(error) => println!("{}", error),
    }
}

pub fn logout() {
    let result = clear_auth_token();

    match result {
        Ok(path) => println!("[INFO] Removed auth token from {}", path),
        Err(error) => println!("{}", error),
    }
}

pub fn init_remote(branch_name: Option<&str>) {
    if let Err(error) = ensure_auth_token_available() {
        println!("{}", error);
        return;
    }

    let result = fs_ops::with_repo_lock("init-remote", || init_remote_locked(branch_name));
    if let Err(error) = result {
        println!("{}", error);
    }
}

pub fn push_branch(branch_name: &str) {
    if let Err(error) = ensure_auth_token_available() {
        println!("{}", error);
        return;
    }

    let result = fs_ops::with_repo_lock("push-snapshot", || push_branch_locked(branch_name));
    if let Err(error) = result {
        println!("{}", error);
    }
}

pub fn pull_branch(branch_name: &str) {
    if let Err(error) = ensure_auth_token_available() {
        println!("{}", error);
        return;
    }

    let result = fs_ops::with_repo_lock("pull", || pull_branch_locked(branch_name));
    if let Err(error) = result {
        println!("{}", error);
    }
}

pub fn sync_db(branch_name: Option<&str>) {
    if let Err(error) = ensure_auth_token_available() {
        println!("{}", error);
        return;
    }

    let result = fs_ops::with_repo_lock("sync-db", || {
        let branch_name = branch::current_branch_or(branch_name);
        sync_db_internal(&branch_name, true)
    });

    if let Err(error) = result {
        println!("{}", error);
    }
}

fn init_remote_locked(branch_name: Option<&str>) -> Result<(), String> {
    let repo_id = repo_id_from_config_or_cwd()?;
    let remote = get_remote_url()?;
    let token = get_auth_token()?;
    let branch_name = branch::current_branch_or(branch_name);
    let head = read_branch_head(&branch_name).unwrap_or_default();
    let objects = if head.is_empty() {
        Vec::new()
    } else {
        sync::collect_encoded_objects(&head)?
    };

    let response = authorized(Client::new().post(format!("{}/repos/init", remote)), &token)
        .json(&InitRepoRequest {
            repo_id: repo_id.clone(),
            name: repo_id.clone(),
            owner_id: "self".to_string(),
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
            Err(error) => return Err(format!("[ERROR] Invalid init-remote response: {}", error)),
        }
    } else {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        if status != StatusCode::CONFLICT {
            return Err(format!("[ERROR] init-remote failed ({}): {}", status, body));
        }

        println!("[WARN] {}", body);
    }

    persist_repo_mapping(&repo_id)?;
    println!("[INFO] Saved repo_id '{}' in .voor/config", repo_id);

    if !head.is_empty() {
        sync_db_internal(&branch_name, false)?;
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
    let token = get_auth_token()?;
    let repo_id = repo_id_from_config_or_cwd()?;
    let objects = sync::collect_encoded_objects(&head)?;

    let response = authorized(Client::new().post(format!("{}/push", remote)), &token)
        .json(&PushRequest {
            repo_id,
            user_id: None,
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
    let token = get_auth_token()?;
    let repo_id = repo_id_from_config_or_cwd()?;
    let current_head = refs::read_head_target();

    let response = authorized(Client::new().post(format!("{}/pull", remote)), &token)
        .json(&PullRequest {
            repo_id,
            user_id: None,
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

fn sync_db_internal(branch_name: &str, print_prefix: bool) -> Result<(), String> {
    let head = read_branch_head(branch_name)?;
    let remote = get_remote_url()?;
    let token = get_auth_token()?;
    let repo_id = repo_id_from_config_or_cwd()?;
    let objects = sync::collect_encoded_objects(&head)?;

    let response = authorized(Client::new().post(format!("{}/sync-db", remote)), &token)
        .json(&SyncDbRequest {
            repo_id,
            user_id: None,
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

fn authorized(builder: RequestBuilder, token: &str) -> RequestBuilder {
    builder.bearer_auth(token.trim())
}

fn get_remote_url() -> Result<String, String> {
    let mut config = read_config().unwrap_or_default();
    let remote = config
        .url
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_REMOTE_URL.to_string());

    if config.url.as_deref() != Some(remote.as_str()) {
        config.url = Some(remote.clone());
        write_config(&config)?;
    }

    Ok(remote)
}

fn get_auth_token() -> Result<String, String> {
    if let Ok(value) = std::env::var("VOOR_AUTH_TOKEN") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    if let Some(value) = app_config::load_user_config()?.auth_token {
        if !value.trim().is_empty() {
            return Ok(value.trim().to_string());
        }
    }

    if let Some(value) = read_config().unwrap_or_default().auth_token {
        if !value.trim().is_empty() {
            return Ok(value.trim().to_string());
        }
    }

    println!("[INFO] No saved Clerk token found. Opening browser login...");
    let token = login_with_browser(false)?;
    persist_auth_token_global(&token)?;
    Ok(token)
}

fn ensure_auth_token_available() -> Result<(), String> {
    get_auth_token().map(|_| ())
}

fn repo_id_from_config_or_cwd() -> Result<String, String> {
    if let Some(repo_id) = read_config()?.repo_id {
        return Ok(repo_id);
    }

    sync::repo_id_from_cwd()
}

fn persist_repo_mapping(repo_id: &str) -> Result<(), String> {
    let mut config = read_config().unwrap_or_default();
    if config.url.is_none() {
        config.url = Some(DEFAULT_REMOTE_URL.to_string());
    }
    config.repo_id = Some(repo_id.trim().to_string());
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
        } else if let Some(value) = trimmed.strip_prefix("auth_token = ") {
            config.auth_token = Some(value.trim().to_string());
        }
    }

    Ok(config)
}

fn write_config(config: &RemoteConfig) -> Result<(), String> {
    let url = config
        .url
        .as_deref()
        .map(normalize_remote_url)
        .unwrap_or_else(|| DEFAULT_REMOTE_URL.to_string());

    let mut content = format!("[remote \"origin\"]\nurl = {}\n", url);
    if let Some(repo_id) = config.repo_id.as_deref() {
        content.push_str(&format!("repo_id = {}\n", repo_id.trim()));
    }
    if let Some(user_id) = config.user_id.as_deref() {
        content.push_str(&format!("user_id = {}\n", user_id.trim()));
    }
    if let Some(auth_token) = config.auth_token.as_deref() {
        content.push_str(&format!("auth_token = {}\n", auth_token.trim()));
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

fn persist_auth_token(token: &str) -> Result<String, String> {
    let path = persist_auth_token_global(token)?;
    migrate_local_auth_token(token.trim())?;
    Ok(path)
}

fn persist_auth_token_global(token: &str) -> Result<String, String> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return Err("[ERROR] Refusing to store an empty auth token".to_string());
    }

    let mut config = app_config::load_user_config().unwrap_or_default();
    config.auth_token = Some(trimmed.to_string());
    let path = app_config::user_config_path()?;
    app_config::save_user_config(&config)?;
    Ok(path.display().to_string())
}

fn clear_auth_token() -> Result<String, String> {
    let mut config = app_config::load_user_config().unwrap_or_default();
    config.auth_token = None;
    let path = app_config::user_config_path()?;
    app_config::save_user_config(&config)?;

    clear_local_auth_token()?;
    Ok(path.display().to_string())
}

fn migrate_local_auth_token(token: &str) -> Result<(), String> {
    if !Path::new(CONFIG_PATH).exists() {
        return Ok(());
    }

    let result = fs_ops::with_repo_lock("auth-login", || {
        let mut config = read_config().unwrap_or_default();
        config.auth_token = Some(token.to_string());
        write_config(&config)
    });

    match result {
        Ok(_) => Ok(()),
        Err(error) if error.contains("Timed out waiting for repository lock") => Err(error),
        Err(_) => Ok(()),
    }
}

fn clear_local_auth_token() -> Result<(), String> {
    if !Path::new(CONFIG_PATH).exists() {
        return Ok(());
    }

    let result = fs_ops::with_repo_lock("auth-logout", || {
        let mut config = read_config().unwrap_or_default();
        config.auth_token = None;
        write_config(&config)
    });

    match result {
        Ok(_) => Ok(()),
        Err(error) if error.contains("Timed out waiting for repository lock") => Err(error),
        Err(_) => Ok(()),
    }
}

fn normalize_remote_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        DEFAULT_REMOTE_URL.to_string()
    } else {
        trimmed.to_string()
    }
}

#[derive(Deserialize)]
struct CliLoginPayload {
    token: String,
}

fn login_with_browser(print_success_page_hint: bool) -> Result<String, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|error| format!("[ERROR] Unable to start local login callback: {}", error))?;
    let port = listener
        .local_addr()
        .map_err(|error| format!("[ERROR] Unable to read local login callback address: {}", error))?
        .port();
    listener
        .set_nonblocking(false)
        .map_err(|error| format!("[ERROR] Unable to configure login callback: {}", error))?;

    let login_url = format!("{}/?cli_login_port={}", frontend_url(), port);
    open_browser(&login_url)?;
    println!("[INFO] Browser opened for Clerk login: {}", login_url);
    println!("[INFO] Waiting for Clerk to return a session token...");

    for stream in listener.incoming() {
        let mut stream = stream.map_err(|error| format!("[ERROR] Login callback failed: {}", error))?;
        match read_login_callback(&mut stream)? {
            LoginCallback::Options => {
                write_http_response(&mut stream, "204 No Content", "", "text/plain")?;
            }
            LoginCallback::Token(token) => {
                write_http_response(
                    &mut stream,
                    "200 OK",
                    "{\"ok\":true}",
                    "application/json",
                )?;
                if print_success_page_hint {
                    println!("[INFO] Clerk login completed");
                }
                return Ok(token);
            }
            LoginCallback::Ignored => {
                write_http_response(
                    &mut stream,
                    "404 Not Found",
                    "Voor CLI login callback is waiting for a Clerk token.",
                    "text/plain",
                )?;
            }
        }
    }

    Err("[ERROR] Login callback stopped before receiving a Clerk token".to_string())
}

enum LoginCallback {
    Options,
    Token(String),
    Ignored,
}

fn read_login_callback(stream: &mut TcpStream) -> Result<LoginCallback, String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .map_err(|error| format!("[ERROR] Unable to configure login callback timeout: {}", error))?;

    let mut buffer = Vec::new();
    let mut temp = [0_u8; 1024];
    let mut header_end = None;
    let mut content_length = 0_usize;

    loop {
        let read = stream
            .read(&mut temp)
            .map_err(|error| format!("[ERROR] Unable to read login callback: {}", error))?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..read]);

        if header_end.is_none() {
            if let Some(position) = find_header_end(&buffer) {
                header_end = Some(position);
                let headers = String::from_utf8_lossy(&buffer[..position]);
                content_length = parse_content_length(&headers);
            }
        }

        if let Some(position) = header_end {
            if buffer.len() >= position + 4 + content_length {
                break;
            }
        }
    }

    let position = header_end.ok_or_else(|| "[ERROR] Invalid login callback request".to_string())?;
    let headers = String::from_utf8_lossy(&buffer[..position]);
    let request_line = headers.lines().next().unwrap_or_default();

    if request_line.starts_with("OPTIONS ") {
        return Ok(LoginCallback::Options);
    }

    if !request_line.starts_with("POST /auth-token ") {
        return Ok(LoginCallback::Ignored);
    }

    let body_start = position + 4;
    let body_end = body_start + content_length;
    let body = std::str::from_utf8(&buffer[body_start..body_end])
        .map_err(|error| format!("[ERROR] Login callback body is not UTF-8: {}", error))?;
    let payload: CliLoginPayload = serde_json::from_str(body)
        .map_err(|error| format!("[ERROR] Invalid login callback payload: {}", error))?;
    let token = payload.token.trim();
    if token.is_empty() {
        return Err("[ERROR] Clerk returned an empty token".to_string());
    }

    Ok(LoginCallback::Token(token.to_string()))
}

fn write_http_response(
    stream: &mut TcpStream,
    status: &str,
    body: &str,
    content_type: &str,
) -> Result<(), String> {
    let response = format!(
        "HTTP/1.1 {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: content-type\r\nAccess-Control-Allow-Methods: POST, OPTIONS\r\nAccess-Control-Allow-Private-Network: true\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| format!("[ERROR] Unable to write login callback response: {}", error))
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
}

fn parse_content_length(headers: &str) -> usize {
    headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0)
}

fn frontend_url() -> String {
    std::env::var("VOOR_FRONTEND_URL")
        .ok()
        .map(|value| normalize_remote_url(&value))
        .unwrap_or_else(|| DEFAULT_FRONTEND_URL.to_string())
}

fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let result = Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .status();

    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(url).status();

    #[cfg(all(unix, not(target_os = "macos")))]
    let result = Command::new("xdg-open").arg(url).status();

    match result {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("[ERROR] Browser opener exited with status {}", status)),
        Err(error) => Err(format!("[ERROR] Unable to open browser: {}", error)),
    }
}

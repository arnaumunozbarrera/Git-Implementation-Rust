use std::fs;

use reqwest::blocking::Client;

use crate::cli::branch;
use crate::utils::refs;
use crate::utils::sync::{self, PullRequest, PullResponse, PushRequest, PushResponse};

const CONFIG_PATH: &str = ".voor/config";

pub fn set_remote(url: &str) {
    let content = format!("[remote \"origin\"]\nurl = {}\n", url.trim());
    match fs::write(CONFIG_PATH, content) {
        Ok(_) => println!("[INFO] Remote 'origin' set to {}", url.trim()),
        Err(error) => println!("[ERROR] Unable to write config: {}", error),
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
    let repo_id = match sync::repo_id_from_cwd() {
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
        .post(format!("{}/push", remote))
        .json(&PushRequest {
            repo_id,
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
    let repo_id = match sync::repo_id_from_cwd() {
        Ok(repo_id) => repo_id,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };

    let current_head = refs::read_head_target();
    let response = Client::new()
        .post(format!("{}/pull", remote))
        .json(&PullRequest {
            repo_id,
            branch: branch_name,
        })
        .send();

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

fn get_remote_url() -> Result<String, String> {
    let content = fs::read_to_string(CONFIG_PATH)
        .map_err(|_| "[ERROR] Missing remote config in .voor/config".to_string())?;

    content
        .lines()
        .find_map(|line| line.trim().strip_prefix("url = "))
        .map(|url| url.trim().trim_end_matches('/').to_string())
        .ok_or_else(|| "[ERROR] Missing remote url in .voor/config".to_string())
}

fn read_branch_head(branch_name: &str) -> Result<String, String> {
    let path = format!(".voor/refs/heads/{}", branch_name.trim());
    let content = fs::read_to_string(&path)
        .map_err(|_| format!("[ERROR] Missing branch '{}'", branch_name.trim()))?;
    let head = content.trim().to_string();

    if head.is_empty() {
        return Err(format!("[ERROR] Branch '{}' has no commits to sync", branch_name.trim()));
    }

    Ok(head)
}

fn print_database_action(database_action: Option<&str>) {
    if let Some(action) = database_action {
        println!("[INFO] {}", action);
    }
}

use std::env;

use serde_json::json;

use crate::api::clients::supabase::SupabaseClient;
use crate::utils::refs;
use crate::utils::sync::{self, PullRequest, PullResponse, PushRequest, PushResponse};

pub async fn push_branch(
    client: Option<&SupabaseClient>,
    payload: PushRequest,
) -> Result<PushResponse, String> {
    let expected_repo = sync::repo_id_from_cwd()?;
    if payload.repo_id.trim() != expected_repo {
        return Err(format!(
            "[ERROR] Unknown repo '{}', expected '{}'",
            payload.repo_id.trim(),
            expected_repo
        ));
    }

    if payload.branch.trim().is_empty() {
        return Err("[ERROR] Missing branch name".to_string());
    }

    if payload.head.trim().is_empty() {
        return Err("[ERROR] Missing commit hash".to_string());
    }

    sync::save_received_objects(&payload.objects)?;
    refs::update_ref(&format!("refs/heads/{}", payload.branch.trim()), &payload.head);
    let object_count = payload.objects.len();
    let database_action = log_sync_action(
        client,
        payload.repo_id.trim(),
        "push",
        json!({
            "branch": payload.branch.trim(),
            "head": payload.head.trim(),
            "object_count": object_count
        }),
    )
    .await?;

    Ok(PushResponse {
        message: format!(
            "Pushed branch '{}' at {}",
            payload.branch.trim(),
            payload.head.trim()
        ),
        object_count,
        database_action,
    })
}

pub async fn pull_branch(
    client: Option<&SupabaseClient>,
    payload: PullRequest,
) -> Result<PullResponse, String> {
    let expected_repo = sync::repo_id_from_cwd()?;
    if payload.repo_id.trim() != expected_repo {
        return Err(format!(
            "[ERROR] Unknown repo '{}', expected '{}'",
            payload.repo_id.trim(),
            expected_repo
        ));
    }

    if payload.branch.trim().is_empty() {
        return Err("[ERROR] Missing branch name".to_string());
    }

    let branch_ref = format!(".voor/refs/heads/{}", payload.branch.trim());
    let head = std::fs::read_to_string(&branch_ref)
        .map_err(|_| format!("[ERROR] Missing branch '{}'", payload.branch.trim()))?
        .trim()
        .to_string();

    if head.is_empty() {
        return Err(format!("[ERROR] Missing branch '{}'", payload.branch.trim()));
    }

    let objects = sync::collect_encoded_objects(&head)?;
    let object_count = objects.len();
    let database_action = log_sync_action(
        client,
        payload.repo_id.trim(),
        "pull",
        json!({
            "branch": payload.branch.trim(),
            "head": head,
            "object_count": object_count
        }),
    )
    .await?;

    Ok(PullResponse {
        branch: payload.branch.trim().to_string(),
        head,
        objects,
        database_action,
    })
}

async fn log_sync_action(
    client: Option<&SupabaseClient>,
    repo_id: &str,
    action: &str,
    metadata: serde_json::Value,
) -> Result<Option<String>, String> {
    let Some(client) = client else {
        return Ok(Some("Skipped database log: Supabase client not configured".to_string()));
    };

    let user_id = match env::var("SYNC_LOG_USER_ID") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            return Ok(Some(
                "Skipped database log: SYNC_LOG_USER_ID not set".to_string(),
            ));
        }
    };

    let repo_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM repositories WHERE id = $1)")
            .bind(repo_id)
            .fetch_one(&client.pool)
            .await
            .map_err(|error| format!("[ERROR] Failed to verify repository for sync log: {}", error))?;

    if !repo_exists {
        return Ok(Some(format!(
            "Skipped database log: repository '{}' not found in database",
            repo_id
        )));
    }

    let trimmed_user_id = user_id.trim();
    let user_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
            .bind(trimmed_user_id)
            .fetch_one(&client.pool)
            .await
            .map_err(|error| format!("[ERROR] Failed to verify user for sync log: {}", error))?;

    if !user_exists {
        return Ok(Some(format!(
            "Skipped database log: user '{}' not found in database",
            trimmed_user_id
        )));
    }

    match sqlx::query(
        "INSERT INTO repo_access_logs (repo_id, user_id, action, metadata) VALUES ($1, $2, $3, $4)",
    )
    .bind(repo_id)
    .bind(trimmed_user_id)
    .bind(action)
    .bind(metadata)
    .execute(&client.pool)
    .await
    {
        Ok(_) => {}
        Err(error) => {
            return Ok(Some(format!("Skipped database log: {}", error)));
        }
    }

    Ok(Some(format!(
        "Logged {} action into repo_access_logs",
        action
    )))
}

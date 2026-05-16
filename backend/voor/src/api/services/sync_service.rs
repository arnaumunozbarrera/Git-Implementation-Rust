use std::collections::HashMap;

use serde_json::json;

use crate::api::clients::supabase::SupabaseClient;
use crate::api::auth::AuthenticatedUser;
use crate::utils::fs_ops;
use crate::utils::object_store::{self, ObjectType, ParsedObject};
use crate::utils::refs;
use crate::utils::sync::{
    self, PullRequest, PullResponse, PushRequest, PushResponse, SyncDbRequest, SyncDbResponse,
};
use uuid::Uuid;
use sqlx::Row;

pub async fn push_branch(
    client: Option<&SupabaseClient>,
    user: &AuthenticatedUser,
    payload: PushRequest,
) -> Result<PushResponse, String> {
    let _repo_lock = fs_ops::acquire_repo_lock("api-push", 15_000)?;
    validate_repo_and_branch(&payload.repo_id, &payload.branch, &payload.head)?;

    sync::save_received_objects(&payload.objects)?;
    refs::update_ref(&format!("refs/heads/{}", payload.branch.trim()), &payload.head);
    let object_count = payload.objects.len();

    let sync_outcome = sync_objects_to_database(
        client,
        payload.repo_id.trim(),
        user.user_id.trim(),
        payload.branch.trim(),
        payload.head.trim(),
        &payload.objects,
        true,
    )
    .await?;

    Ok(PushResponse {
        message: format!(
            "Pushed branch '{}' at {}",
            payload.branch.trim(),
            payload.head.trim()
        ),
        object_count,
        database_action: sync_outcome.database_action,
    })
}

pub async fn pull_branch(
    client: Option<&SupabaseClient>,
    user: &AuthenticatedUser,
    payload: PullRequest,
) -> Result<PullResponse, String> {
    let _repo_lock = fs_ops::acquire_repo_lock("api-pull", 15_000)?;
    validate_repo_and_branch(&payload.repo_id, &payload.branch, "pull")?;

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
        user.user_id.trim(),
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

pub async fn sync_db(
    client: Option<&SupabaseClient>,
    user: &AuthenticatedUser,
    payload: SyncDbRequest,
) -> Result<SyncDbResponse, String> {
    let _repo_lock = fs_ops::acquire_repo_lock("api-sync-db", 15_000)?;
    validate_repo_and_branch(&payload.repo_id, &payload.branch, &payload.head)?;

    sync::save_received_objects(&payload.objects)?;
    refs::update_ref(&format!("refs/heads/{}", payload.branch.trim()), &payload.head);

    let sync_outcome = sync_objects_to_database(
        client,
        payload.repo_id.trim(),
        user.user_id.trim(),
        payload.branch.trim(),
        payload.head.trim(),
        &payload.objects,
        false,
    )
    .await?;

    Ok(SyncDbResponse {
        message: format!(
            "Synchronized database state for branch '{}' at {}",
            payload.branch.trim(),
            payload.head.trim()
        ),
        database_action: sync_outcome.database_action,
        branch_status: sync_outcome.branch_status,
    })
}

struct DatabaseSyncOutcome {
    database_action: Option<String>,
    branch_status: Option<String>,
}

fn validate_repo_and_branch(repo_id: &str, branch: &str, head: &str) -> Result<(), String> {
    if repo_id.trim().is_empty() {
        return Err("[ERROR] Missing repo_id".to_string());
    }

    if branch.trim().is_empty() {
        return Err("[ERROR] Missing branch name".to_string());
    }

    if head.trim().is_empty() {
        return Err("[ERROR] Missing commit hash".to_string());
    }

    Ok(())
}

async fn sync_objects_to_database(
    client: Option<&SupabaseClient>,
    repo_id: &str,
    user_id: &str,
    branch: &str,
    head: &str,
    objects: &[sync::EncodedObject],
    log_push_action: bool,
) -> Result<DatabaseSyncOutcome, String> {
    let Some(client) = client else {
        return Ok(DatabaseSyncOutcome {
            database_action: Some("Skipped database sync: Supabase client not configured".to_string()),
            branch_status: None,
        });
    };

    ensure_repo_owner(client, repo_id, user_id).await?;
    ensure_user_exists(client, user_id).await?;

    let mut parsed_objects = Vec::with_capacity(objects.len());
    let mut object_cache = HashMap::with_capacity(objects.len());

    for encoded in objects {
        let full_bytes = sync::decode_object_from_network(encoded)?;
        let parsed = object_store::parse_full_object(&encoded.hash, full_bytes)?;
        object_cache.insert(encoded.hash.trim().to_string(), parsed.clone());
        parsed_objects.push((encoded.hash.trim().to_string(), parsed));
    }

    let mut blob_count = 0usize;
    let mut tree_count = 0usize;
    let mut commit_count = 0usize;

    for (hash, parsed) in &parsed_objects {
        if parsed.object_type == ObjectType::Blob {
            upsert_blob(client, hash, &parsed.content).await?;
            blob_count += 1;
        }
    }

    for (hash, parsed) in &parsed_objects {
        if parsed.object_type == ObjectType::Tree {
            upsert_tree(client, hash).await?;
            upsert_tree_entries(client, hash, &parsed.content).await?;
            tree_count += 1;
        }
    }

    for (hash, parsed) in &parsed_objects {
        if parsed.object_type == ObjectType::Commit {
            let commit_data = parse_commit_content(&parsed.content)?;
            ensure_tree_exists(client, &commit_data.tree_hash).await?;
            upsert_commit(client, hash, &commit_data, user_id).await?;
            let (additions, deletions) = calculate_commit_metrics(&commit_data, &object_cache)?;
            upsert_commit_metadata(
                client,
                repo_id,
                hash,
                user_id,
                &commit_data.message,
                additions,
                deletions,
            )
            .await?;
            commit_count += 1;
        }
    }

    let branch_status = upsert_branch(client, repo_id, branch, head).await?;
    let log_status = if log_push_action {
        log_sync_action(
            Some(client),
            repo_id,
            user_id,
            "push",
            json!({
                "branch": branch,
                "head": head,
                "object_count": objects.len(),
                "blob_count": blob_count,
                "tree_count": tree_count,
                "commit_count": commit_count
            }),
        )
        .await?
    } else {
        None
    };

    let mut status = format!(
        "Synced {} blobs, {} trees, {} commits into database",
        blob_count, tree_count, commit_count
    );
    if let Some(branch_note) = &branch_status {
        status.push_str(&format!("; {}", branch_note));
    }
    if let Some(log_note) = &log_status {
        status.push_str(&format!("; {}", log_note));
    }

    Ok(DatabaseSyncOutcome {
        database_action: Some(status),
        branch_status,
    })
}

async fn ensure_repo_owner(
    client: &SupabaseClient,
    repo_id: &str,
    user_id: &str,
) -> Result<(), String> {
    let owner_id: Option<String> = sqlx::query_scalar(
        "SELECT owner_id FROM repositories WHERE id = $1",
    )
    .bind(repo_id)
    .fetch_optional(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to verify repository '{}': {}", repo_id, error))?;

    let Some(owner_id) = owner_id else {
        return Err(format!("[ERROR] Repository '{}' not found", repo_id));
    };

    if owner_id != user_id {
        return Err(format!(
            "[ERROR] User '{}' cannot sync repository '{}'",
            user_id, repo_id
        ));
    }

    Ok(())
}

async fn ensure_user_exists(client: &SupabaseClient, user_id: &str) -> Result<(), String> {
    let user_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(user_id)
        .fetch_one(&client.pool)
        .await
        .map_err(|error| format!("[ERROR] Failed to verify user '{}': {}", user_id, error))?;

    if !user_exists {
        return Err(format!("[ERROR] User '{}' not found", user_id));
    }

    Ok(())
}

async fn upsert_blob(client: &SupabaseClient, hash: &str, content: &[u8]) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO blobs (hash, content, size) VALUES ($1, $2, $3) ON CONFLICT (hash) DO NOTHING",
    )
    .bind(hash)
    .bind(content)
    .bind(content.len() as i64)
    .execute(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to store blob '{}': {}", hash, error))?;

    Ok(())
}

async fn upsert_tree(client: &SupabaseClient, hash: &str) -> Result<(), String> {
    sqlx::query("INSERT INTO trees (hash) VALUES ($1) ON CONFLICT (hash) DO NOTHING")
        .bind(hash)
        .execute(&client.pool)
        .await
        .map_err(|error| format!("[ERROR] Failed to store tree '{}': {}", hash, error))?;

    Ok(())
}

async fn upsert_tree_entries(
    client: &SupabaseClient,
    tree_hash: &str,
    content: &[u8],
) -> Result<(), String> {
    for entry in object_store::parse_tree(content)? {
        let entry_type = entry.object_type.as_str();
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM tree_entries WHERE tree_hash = $1 AND name = $2 AND type = $3 AND hash = $4 AND mode = $5)",
        )
        .bind(tree_hash)
        .bind(&entry.name)
        .bind(entry_type)
        .bind(&entry.hash)
        .bind(&entry.mode)
        .fetch_one(&client.pool)
        .await
        .map_err(|error| format!("[ERROR] Failed to verify tree entry '{}': {}", entry.name, error))?;

        if !exists {
            sqlx::query(
                "INSERT INTO tree_entries (tree_hash, name, type, hash, mode) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(tree_hash)
            .bind(&entry.name)
            .bind(entry_type)
            .bind(&entry.hash)
            .bind(&entry.mode)
            .execute(&client.pool)
            .await
            .map_err(|error| format!(
                "[ERROR] Failed to store tree entry '{}' for tree '{}': {}",
                entry.name, tree_hash, error
            ))?;
        }
    }

    Ok(())
}

async fn ensure_tree_exists(client: &SupabaseClient, tree_hash: &str) -> Result<(), String> {
    let tree_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM trees WHERE hash = $1)")
        .bind(tree_hash)
        .fetch_one(&client.pool)
        .await
        .map_err(|error| format!("[ERROR] Failed to verify tree '{}': {}", tree_hash, error))?;

    if !tree_exists {
        return Err(format!(
            "[ERROR] Cannot insert commit because tree '{}' is missing",
            tree_hash
        ));
    }

    Ok(())
}

async fn upsert_commit(
    client: &SupabaseClient,
    hash: &str,
    commit: &ParsedCommit,
    user_id: &str,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO commits (hash, tree_hash, parent_hash, author_id, message) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (hash) DO NOTHING",
    )
    .bind(hash)
    .bind(&commit.tree_hash)
    .bind(commit.parent_hash.as_deref())
    .bind(user_id)
    .bind(&commit.message)
    .execute(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to store commit '{}': {}", hash, error))?;

    Ok(())
}

async fn upsert_commit_metadata(
    client: &SupabaseClient,
    repo_id: &str,
    commit_hash: &str,
    user_id: &str,
    message: &str,
    additions: i64,
    deletions: i64,
) -> Result<(), String> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM commits_metadata WHERE repo_id = $1 AND commit_hash = $2)",
    )
    .bind(repo_id)
    .bind(commit_hash)
    .fetch_one(&client.pool)
    .await
    .map_err(|error| format!(
        "[ERROR] Failed to verify commit metadata '{}': {}",
        commit_hash, error
    ))?;

    if !exists {
        sqlx::query(
            "INSERT INTO commits_metadata (repo_id, commit_hash, author_id, message, additions, deletions) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(repo_id)
        .bind(commit_hash)
        .bind(user_id)
        .bind(message)
        .bind(additions)
        .bind(deletions)
        .execute(&client.pool)
        .await
        .map_err(|error| format!(
            "[ERROR] Failed to store commit metadata '{}': {}",
            commit_hash, error
        ))?;
    }

    Ok(())
}

async fn upsert_branch(
    client: &SupabaseClient,
    repo_id: &str,
    branch_name: &str,
    head: &str,
) -> Result<Option<String>, String> {
    let existing = sqlx::query(
        "SELECT id, last_commit_hash 
         FROM branches 
         WHERE repo_id = $1 AND name = $2 
         ORDER BY created_at ASC 
         LIMIT 1",
    )
    .bind(repo_id)
    .bind(branch_name)
    .fetch_optional(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to verify branch '{}': {}", branch_name, error))?;

    if let Some(row) = existing {
        let branch_id: Uuid = row.get(0);
        let previous_head: Option<String> = row.get(1);

        sqlx::query("UPDATE branches SET last_commit_hash = $1 WHERE id = $2")
            .bind(head)
            .bind(branch_id)
            .execute(&client.pool)
            .await
            .map_err(|error| format!("[ERROR] Failed to update branch '{}': {}", branch_name, error))?;

        if previous_head.as_deref() != Some(head) {
            return Ok(Some(format!(
                "Updated branch '{}' from {:?} to {}",
                branch_name, previous_head, head
            )));
        }

        return Ok(Some(format!(
            "Branch '{}' already pointed to {}",
            branch_name, head
        )));
    }

    sqlx::query(
        "INSERT INTO branches (repo_id, name, last_commit_hash) VALUES ($1, $2, $3)",
    )
    .bind(repo_id)
    .bind(branch_name)
    .bind(head)
    .execute(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to create branch '{}': {}", branch_name, error))?;

    Ok(Some(format!("Created branch '{}' at {}", branch_name, head)))
}

async fn log_sync_action(
    client: Option<&SupabaseClient>,
    repo_id: &str,
    user_id: &str,
    action: &str,
    metadata: serde_json::Value,
) -> Result<Option<String>, String> {
    let Some(client) = client else {
        return Ok(Some("Skipped database log: Supabase client not configured".to_string()));
    };

    if user_id.trim().is_empty() {
        return Ok(Some("Skipped database log: user_id not provided".to_string()));
    }

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

    let user_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
            .bind(user_id)
            .fetch_one(&client.pool)
            .await
            .map_err(|error| format!("[ERROR] Failed to verify user for sync log: {}", error))?;

    if !user_exists {
        return Ok(Some(format!(
            "Skipped database log: user '{}' not found in database",
            user_id
        )));
    }

    match sqlx::query(
        "INSERT INTO repo_access_logs (repo_id, user_id, action, metadata) VALUES ($1, $2, $3, $4)",
    )
    .bind(repo_id)
    .bind(user_id)
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

#[derive(Debug, Clone)]
struct ParsedCommit {
    tree_hash: String,
    parent_hash: Option<String>,
    _author: String,
    message: String,
}

fn parse_commit_content(content: &[u8]) -> Result<ParsedCommit, String> {
    let commit_text = String::from_utf8(content.to_vec())
        .map_err(|error| format!("[ERROR] Invalid commit content: {}", error))?;
    let mut tree_hash = None;
    let mut parent_hash = None;
    let mut author = None;
    let mut message_lines = Vec::new();
    let mut in_message = false;

    for line in commit_text.lines() {
        if in_message {
            message_lines.push(line);
            continue;
        }

        if line.is_empty() {
            in_message = true;
            continue;
        }

        if let Some(value) = line.strip_prefix("tree ") {
            tree_hash = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("parent ") {
            parent_hash = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("author ") {
            author = Some(value.trim().to_string());
        }
    }

    Ok(ParsedCommit {
        tree_hash: tree_hash.ok_or_else(|| "[ERROR] Commit missing tree hash".to_string())?,
        parent_hash,
        _author: author.unwrap_or_default(),
        message: message_lines.join("\n").trim().to_string(),
    })
}

fn calculate_commit_metrics(
    commit: &ParsedCommit,
    cache: &HashMap<String, ParsedObject>,
) -> Result<(i64, i64), String> {
    let current_files = collect_tree_files(&commit.tree_hash, cache)?;
    let parent_files = match commit.parent_hash.as_deref() {
        Some(parent_hash) if !parent_hash.trim().is_empty() => {
            let parent_commit = load_object(parent_hash, cache)?;
            let parent_data = parse_commit_content(&parent_commit.content)?;
            collect_tree_files(&parent_data.tree_hash, cache)?
        }
        _ => HashMap::new(),
    };

    let mut additions = 0i64;
    let mut deletions = 0i64;

    for (path, new_hash) in &current_files {
        match parent_files.get(path) {
            Some(old_hash) if old_hash == new_hash => {}
            Some(old_hash) => {
                let (file_additions, file_deletions) = diff_blob_hashes(old_hash, new_hash, cache)?;
                additions += file_additions;
                deletions += file_deletions;
            }
            None => {
                let (file_additions, file_deletions) = diff_blob_hashes("", new_hash, cache)?;
                additions += file_additions;
                deletions += file_deletions;
            }
        }
    }

    for (path, old_hash) in &parent_files {
        if !current_files.contains_key(path) {
            let (file_additions, file_deletions) = diff_blob_hashes(old_hash, "", cache)?;
            additions += file_additions;
            deletions += file_deletions;
        }
    }

    Ok((additions, deletions))
}

fn collect_tree_files(
    tree_hash: &str,
    cache: &HashMap<String, ParsedObject>,
) -> Result<HashMap<String, String>, String> {
    let mut files = HashMap::new();
    collect_tree_files_recursive(tree_hash, "", cache, &mut files)?;
    Ok(files)
}

fn collect_tree_files_recursive(
    tree_hash: &str,
    prefix: &str,
    cache: &HashMap<String, ParsedObject>,
    files: &mut HashMap<String, String>,
) -> Result<(), String> {
    let tree = load_object(tree_hash, cache)?;
    if tree.object_type != ObjectType::Tree {
        return Err(format!("[ERROR] Expected tree object '{}'", tree_hash));
    }

    for entry in object_store::parse_tree(&tree.content)? {
        let path = if prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{}/{}", prefix, entry.name)
        };

        match entry.object_type {
            ObjectType::Blob => {
                files.insert(path, entry.hash);
            }
            ObjectType::Tree => collect_tree_files_recursive(&entry.hash, &path, cache, files)?,
            ObjectType::Commit => {}
        }
    }

    Ok(())
}

fn diff_blob_hashes(
    old_hash: &str,
    new_hash: &str,
    cache: &HashMap<String, ParsedObject>,
) -> Result<(i64, i64), String> {
    let old_lines = if old_hash.trim().is_empty() {
        Vec::new()
    } else {
        read_blob_lines(old_hash, cache)?
    };
    let new_lines = if new_hash.trim().is_empty() {
        Vec::new()
    } else {
        read_blob_lines(new_hash, cache)?
    };

    Ok(diff_line_counts(&old_lines, &new_lines))
}

fn read_blob_lines(hash: &str, cache: &HashMap<String, ParsedObject>) -> Result<Vec<String>, String> {
    let blob = load_object(hash, cache)?;
    if blob.object_type != ObjectType::Blob {
        return Err(format!("[ERROR] Expected blob object '{}'", hash));
    }

    let text = String::from_utf8_lossy(&blob.content);
    Ok(text.lines().map(|line| line.to_string()).collect())
}

fn diff_line_counts(old_lines: &[String], new_lines: &[String]) -> (i64, i64) {
    let old_len = old_lines.len();
    let new_len = new_lines.len();
    let mut lcs = vec![vec![0usize; new_len + 1]; old_len + 1];

    for old_index in 0..old_len {
        for new_index in 0..new_len {
            if old_lines[old_index] == new_lines[new_index] {
                lcs[old_index + 1][new_index + 1] = lcs[old_index][new_index] + 1;
            } else {
                lcs[old_index + 1][new_index + 1] =
                    lcs[old_index + 1][new_index].max(lcs[old_index][new_index + 1]);
            }
        }
    }

    let common = lcs[old_len][new_len] as i64;
    (new_len as i64 - common, old_len as i64 - common)
}

fn load_object(hash: &str, cache: &HashMap<String, ParsedObject>) -> Result<ParsedObject, String> {
    if let Some(object) = cache.get(hash) {
        return Ok(object.clone());
    }

    object_store::read_object(hash)
}

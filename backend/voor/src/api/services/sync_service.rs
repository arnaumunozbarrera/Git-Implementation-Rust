use std::collections::HashMap;

use serde_json::json;
use chrono::Utc;

use crate::api::clients::supabase::SupabaseClient;
use crate::api::auth::AuthenticatedUser;
use crate::api::services::email_service;
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

    let head_commit_hash = payload.head.trim();
    
    if let Ok(head_obj_result) = fetch_object_bytes(head_commit_hash) {
        if let Ok(parsed_commit) = parse_commit_content(&head_obj_result) {
            let mut object_cache = HashMap::new();
            for obj in &payload.objects {
                if let Ok(full_bytes) = sync::decode_object_from_network(obj) {
                    if let Ok(parsed) = object_store::parse_full_object(&obj.hash, full_bytes) {
                        object_cache.insert(obj.hash.trim().to_string(), parsed);
                    }
                }
            }

            if let Ok(top_files) = calculate_top_3_files_with_changes(&parsed_commit, &object_cache) {
                let _ = send_push_email_alert(
                    client,
                    payload.repo_id.trim(),
                    user.user_id.trim(),
                    payload.branch.trim(),
                    &parsed_commit.message,
                    top_files,
                )
                .await;
            }
        }
    }

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
            upsert_commit_edges(client, repo_id, hash, &commit_data.parent_hashes).await?;
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
    refresh_repository_vcs_metrics(client, repo_id).await?;
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
    .bind(commit.parent_hashes.first().map(String::as_str))
    .bind(user_id)
    .bind(&commit.message)
    .execute(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to store commit '{}': {}", hash, error))?;

    Ok(())
}

async fn upsert_commit_edges(
    client: &SupabaseClient,
    repo_id: &str,
    child_hash: &str,
    parent_hashes: &[String],
) -> Result<(), String> {
    for (index, parent_hash) in parent_hashes.iter().enumerate() {
        if parent_hash.trim().is_empty() {
            continue;
        }

        let edge_type = if index == 0 { "parent" } else { "merge" };
        sqlx::query(
            "INSERT INTO commit_edges (repo_id, child_hash, parent_hash, parent_index, edge_type)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (repo_id, child_hash, parent_hash)
             DO UPDATE SET parent_index = EXCLUDED.parent_index, edge_type = EXCLUDED.edge_type",
        )
        .bind(repo_id)
        .bind(child_hash)
        .bind(parent_hash)
        .bind(index as i32)
        .bind(edge_type)
        .execute(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to store commit edge '{} -> {}': {}",
                child_hash, parent_hash, error
            )
        })?;
    }

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

        sqlx::query(
            "UPDATE branches
             SET last_commit_hash = $1,
                 last_activity_at = now(),
                 last_analyzed_at = now(),
                 is_default_cached = name = (SELECT default_branch FROM repositories WHERE id = $3)
             WHERE id = $2",
        )
            .bind(head)
            .bind(branch_id)
            .bind(repo_id)
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
        "INSERT INTO branches (repo_id, name, last_commit_hash, last_activity_at, last_analyzed_at, is_default_cached)
         VALUES ($1, $2, $3, now(), now(), $2 = (SELECT default_branch FROM repositories WHERE id = $1))",
    )
    .bind(repo_id)
    .bind(branch_name)
    .bind(head)
    .execute(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to create branch '{}': {}", branch_name, error))?;

    Ok(Some(format!("Created branch '{}' at {}", branch_name, head)))
}

async fn refresh_repository_vcs_metrics(
    client: &SupabaseClient,
    repo_id: &str,
) -> Result<(), String> {
    sqlx::query(
        "WITH RECURSIVE
            repo AS (
                SELECT id, default_branch FROM repositories WHERE id = $1
            ),
            default_head AS (
                SELECT b.last_commit_hash AS hash
                FROM branches b
                JOIN repo r ON r.id = b.repo_id AND r.default_branch = b.name
                WHERE b.last_commit_hash IS NOT NULL
            ),
            branch_heads AS (
                SELECT b.id, b.repo_id, b.name, b.last_commit_hash
                FROM branches b
                WHERE b.repo_id = $1 AND b.last_commit_hash IS NOT NULL
            ),
            default_ancestors(hash, depth) AS (
                SELECT hash, 0 FROM default_head
                UNION
                SELECT COALESCE(ce.parent_hash, c.parent_hash), da.depth + 1
                FROM default_ancestors da
                JOIN commits c ON c.hash = da.hash
                LEFT JOIN commit_edges ce ON ce.repo_id = $1 AND ce.child_hash = c.hash
                WHERE COALESCE(ce.parent_hash, c.parent_hash) IS NOT NULL
            ),
            branch_ancestors(branch_id, branch_name, hash, depth) AS (
                SELECT id, name, last_commit_hash, 0 FROM branch_heads
                UNION
                SELECT ba.branch_id, ba.branch_name, COALESCE(ce.parent_hash, c.parent_hash), ba.depth + 1
                FROM branch_ancestors ba
                JOIN commits c ON c.hash = ba.hash
                LEFT JOIN commit_edges ce ON ce.repo_id = $1 AND ce.child_hash = c.hash
                WHERE COALESCE(ce.parent_hash, c.parent_hash) IS NOT NULL
            ),
            merge_bases AS (
                SELECT DISTINCT ON (ba.branch_id)
                    ba.branch_id,
                    ba.branch_name,
                    ba.hash AS merge_base_hash,
                    ba.depth AS ahead_count,
                    da.depth AS behind_count
                FROM branch_ancestors ba
                JOIN default_ancestors da ON da.hash = ba.hash
                ORDER BY ba.branch_id, (ba.depth + da.depth), ba.depth
            )
         INSERT INTO branch_metrics (
            repo_id,
            branch_id,
            branch_name,
            default_branch_name,
            head_commit_hash,
            default_head_hash,
            merge_base_hash,
            ahead_count,
            behind_count,
            divergence_distance,
            freshness_status,
            freshness_score,
            health_score,
            stale_days,
            computed_at
         )
         SELECT
            b.repo_id,
            b.id,
            b.name,
            r.default_branch,
            b.last_commit_hash,
            dh.hash,
            mb.merge_base_hash,
            CASE WHEN b.name = r.default_branch THEN 0 ELSE COALESCE(mb.ahead_count, 0) END,
            CASE WHEN b.name = r.default_branch THEN 0 ELSE COALESCE(mb.behind_count, 0) END,
            CASE WHEN b.name = r.default_branch THEN 0 ELSE COALESCE(mb.ahead_count, 0) + COALESCE(mb.behind_count, 0) END,
            CASE
                WHEN b.name = r.default_branch THEN 'default'
                WHEN EXTRACT(day FROM now() - COALESCE(b.last_activity_at, c.created_at, b.created_at)) < 15 THEN 'active'
                WHEN EXTRACT(day FROM now() - COALESCE(b.last_activity_at, c.created_at, b.created_at)) <= 30 THEN 'idle'
                ELSE 'outdated'
            END,
            GREATEST(0, 100 - EXTRACT(day FROM now() - COALESCE(b.last_activity_at, c.created_at, b.created_at)))::numeric,
            GREATEST(
                0,
                100
                - (CASE WHEN b.name = r.default_branch THEN 0 ELSE COALESCE(mb.ahead_count, 0) + COALESCE(mb.behind_count, 0) END * 3)
                - EXTRACT(day FROM now() - COALESCE(b.last_activity_at, c.created_at, b.created_at))
            )::numeric,
            GREATEST(0, FLOOR(EXTRACT(epoch FROM now() - COALESCE(b.last_activity_at, c.created_at, b.created_at)) / 86400))::int,
            now()
         FROM branches b
         JOIN repositories r ON r.id = b.repo_id
         LEFT JOIN commits c ON c.hash = b.last_commit_hash
         LEFT JOIN default_head dh ON true
         LEFT JOIN merge_bases mb ON mb.branch_id = b.id
         WHERE b.repo_id = $1
         ON CONFLICT (repo_id, branch_name)
         DO UPDATE SET
            branch_id = EXCLUDED.branch_id,
            default_branch_name = EXCLUDED.default_branch_name,
            head_commit_hash = EXCLUDED.head_commit_hash,
            default_head_hash = EXCLUDED.default_head_hash,
            merge_base_hash = EXCLUDED.merge_base_hash,
            ahead_count = EXCLUDED.ahead_count,
            behind_count = EXCLUDED.behind_count,
            divergence_distance = EXCLUDED.divergence_distance,
            freshness_status = EXCLUDED.freshness_status,
            freshness_score = EXCLUDED.freshness_score,
            health_score = EXCLUDED.health_score,
            stale_days = EXCLUDED.stale_days,
            computed_at = EXCLUDED.computed_at",
    )
    .bind(repo_id)
    .execute(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to refresh branch metrics for '{}': {}", repo_id, error))?;

    sqlx::query(
        "INSERT INTO branch_topology_metrics (
            repo_id,
            branch_name,
            lane_index,
            lane_color,
            start_commit_hash,
            head_commit_hash,
            merge_base_hash,
            first_seen_at,
            last_seen_at,
            commit_density,
            activity_heat
         )
         SELECT
            bm.repo_id,
            bm.branch_name,
            ROW_NUMBER() OVER (
                PARTITION BY bm.repo_id
                ORDER BY CASE WHEN bm.branch_name = bm.default_branch_name THEN 0 ELSE 1 END,
                         bm.divergence_distance DESC,
                         bm.branch_name ASC
            )::int - 1,
            NULL,
            bm.merge_base_hash,
            bm.head_commit_hash,
            bm.merge_base_hash,
            MIN(c.created_at),
            MAX(c.created_at),
            COUNT(DISTINCT c.hash)::numeric,
            LEAST(0.16, GREATEST(0.04, COUNT(DISTINCT c.hash)::numeric / 100.0))
         FROM branch_metrics bm
         LEFT JOIN commits c ON c.hash = bm.head_commit_hash OR c.hash = bm.merge_base_hash
         WHERE bm.repo_id = $1
         GROUP BY bm.repo_id, bm.branch_name, bm.default_branch_name, bm.divergence_distance, bm.merge_base_hash, bm.head_commit_hash
         ON CONFLICT (repo_id, branch_name)
         DO UPDATE SET
            lane_index = EXCLUDED.lane_index,
            start_commit_hash = EXCLUDED.start_commit_hash,
            head_commit_hash = EXCLUDED.head_commit_hash,
            merge_base_hash = EXCLUDED.merge_base_hash,
            first_seen_at = EXCLUDED.first_seen_at,
            last_seen_at = EXCLUDED.last_seen_at,
            commit_density = EXCLUDED.commit_density,
            activity_heat = EXCLUDED.activity_heat",
    )
    .bind(repo_id)
    .execute(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to refresh topology metrics for '{}': {}", repo_id, error))?;

    sqlx::query(
        "INSERT INTO timeline_aggregation (
            repo_id,
            bucket_start,
            bucket_granularity,
            commit_count,
            author_count,
            branch_count,
            additions,
            deletions,
            audit_event_count
         )
         SELECT
            $1,
            date_trunc('day', now()),
            'day',
            (SELECT COUNT(*) FROM commits_metadata WHERE repo_id = $1 AND created_at >= date_trunc('day', now())),
            (SELECT COUNT(DISTINCT author_id) FROM commits_metadata WHERE repo_id = $1 AND created_at >= date_trunc('day', now())),
            (SELECT COUNT(*) FROM branches WHERE repo_id = $1),
            (SELECT COALESCE(SUM(additions), 0) FROM commits_metadata WHERE repo_id = $1 AND created_at >= date_trunc('day', now())),
            (SELECT COALESCE(SUM(deletions), 0) FROM commits_metadata WHERE repo_id = $1 AND created_at >= date_trunc('day', now())),
            (SELECT COUNT(*) FROM repo_access_logs WHERE repo_id = $1 AND created_at >= date_trunc('day', now()))
         ON CONFLICT (repo_id, bucket_start, bucket_granularity)
         DO UPDATE SET
            commit_count = EXCLUDED.commit_count,
            author_count = EXCLUDED.author_count,
            branch_count = EXCLUDED.branch_count,
            additions = EXCLUDED.additions,
            deletions = EXCLUDED.deletions,
            audit_event_count = EXCLUDED.audit_event_count",
    )
    .bind(repo_id)
    .execute(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to refresh timeline aggregation for '{}': {}", repo_id, error))?;

    Ok(())
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
    parent_hashes: Vec<String>,
    _author: String,
    message: String,
}

fn parse_commit_content(content: &[u8]) -> Result<ParsedCommit, String> {
    let commit_text = String::from_utf8(content.to_vec())
        .map_err(|error| format!("[ERROR] Invalid commit content: {}", error))?;
    let mut tree_hash = None;
    let mut parent_hashes = Vec::new();
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
            parent_hashes.push(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("author ") {
            author = Some(value.trim().to_string());
        }
    }

    Ok(ParsedCommit {
        tree_hash: tree_hash.ok_or_else(|| "[ERROR] Commit missing tree hash".to_string())?,
        parent_hashes,
        _author: author.unwrap_or_default(),
        message: message_lines.join("\n").trim().to_string(),
    })
}

fn calculate_commit_metrics(
    commit: &ParsedCommit,
    cache: &HashMap<String, ParsedObject>,
) -> Result<(i64, i64), String> {
    let current_files = collect_tree_files(&commit.tree_hash, cache)?;
    let parent_files = match commit.parent_hashes.first().map(String::as_str) {
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

struct FileChangeInfo {
    file_path: String,
    changes: i64,
}

fn calculate_top_3_files_with_changes(
    commit: &ParsedCommit,
    cache: &HashMap<String, ParsedObject>,
) -> Result<Vec<(String, i64)>, String> {
    let current_files = collect_tree_files(&commit.tree_hash, cache)?;
    let parent_files = match commit.parent_hashes.first().map(String::as_str) {
        Some(parent_hash) if !parent_hash.trim().is_empty() => {
            let parent_commit = load_object(parent_hash, cache)?;
            let parent_data = parse_commit_content(&parent_commit.content)?;
            collect_tree_files(&parent_data.tree_hash, cache)?
        }
        _ => HashMap::new(),
    };

    let mut file_changes: Vec<FileChangeInfo> = Vec::new();

    for (path, new_hash) in &current_files {
        let change_count = match parent_files.get(path) {
            Some(old_hash) if old_hash == new_hash => 0i64,
            Some(old_hash) => {
                let (additions, deletions) = diff_blob_hashes(old_hash, new_hash, cache)?;
                additions + deletions
            }
            None => {
                let (additions, _deletions) = diff_blob_hashes("", new_hash, cache)?;
                additions
            }
        };

        if change_count > 0 {
            file_changes.push(FileChangeInfo {
                file_path: path.clone(),
                changes: change_count,
            });
        }
    }

    for (path, old_hash) in &parent_files {
        if !current_files.contains_key(path) {
            let (deletions, _) = diff_blob_hashes(old_hash, "", cache)?;
            file_changes.push(FileChangeInfo {
                file_path: path.clone(),
                changes: deletions,
            });
        }
    }

    file_changes.sort_by(|a, b| b.changes.cmp(&a.changes));
    let top_3: Vec<(String, i64)> = file_changes
        .into_iter()
        .take(3)
        .map(|fc| (fc.file_path, fc.changes))
        .collect();

    Ok(top_3)
}

async fn get_repository_name(
    client: &SupabaseClient,
    repo_id: &str,
) -> Result<String, String> {
    let name: String = sqlx::query_scalar("SELECT name FROM repositories WHERE id = $1")
        .bind(repo_id)
        .fetch_optional(&client.pool)
        .await
        .map_err(|error| format!("[ERROR] Failed to fetch repository name: {}", error))?
        .ok_or_else(|| format!("[ERROR] Repository '{}' not found", repo_id))?;

    Ok(name)
}

async fn get_owner_email(
    client: &SupabaseClient,
    repo_id: &str,
) -> Result<Option<String>, String> {
    let email: Option<String> = sqlx::query_scalar(
        "SELECT u.email FROM users u
         JOIN repositories r ON r.owner_id = u.id
         WHERE r.id = $1"
    )
    .bind(repo_id)
    .fetch_optional(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to fetch owner email: {}", error))?;

    Ok(email)
}

async fn get_user_info(
    client: &SupabaseClient,
    user_id: &str,
) -> Result<(String, Option<String>), String> {
    let (username, email): (Option<String>, Option<String>) = sqlx::query_as(
        "SELECT username, email FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to fetch user info: {}", error))?
    .ok_or_else(|| format!("[ERROR] User '{}' not found", user_id))?;

    let display_name = username.unwrap_or_else(|| user_id.to_string());
    Ok((display_name, email))
}

async fn send_push_email_alert(
    client: Option<&SupabaseClient>,
    repo_id: &str,
    user_id: &str,
    branch: &str,
    commit_message: &str,
    top_files: Vec<(String, i64)>,
) -> Result<(), String> {
    let Some(client) = client else {
        return Ok(());
    };

    let owner_email = get_owner_email(client, repo_id).await?;
    let Some(recipient_email) = owner_email else {
        return Ok(());
    };

    let repo_name = get_repository_name(client, repo_id).await?;
    let (contributor_name, _) = get_user_info(client, user_id).await?;

    let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let alert = email_service::EmailAlert {
        recipient: recipient_email,
        contributor: contributor_name,
        commit_message: commit_message.to_string(),
        repository_name: repo_name,
        branch: branch.to_string(),
        timestamp,
        top_files,
    };

    email_service::send_push_alert(alert).await.ok();
    Ok(())
}

fn fetch_object_bytes(hash: &str) -> Result<Vec<u8>, String> {
    object_store::read_object(hash)
        .map(|obj| obj.content)
}

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sqlx::Row;

use crate::api::clients::supabase::SupabaseClient;
use crate::api::models::{
    ActivityFeedItem, AnalyticsOverviewResponse, CommitGraphNode, CommitGraphResponse,
    CommitSummary, ContentEntry, ContentsResponse, FileContentResponse, PaginationResponse,
    ReadmePreview, RepoDashboardResponse, Repository, RepositoryFileSummary, UserSummary,
};
use crate::utils::object_store::{self, ObjectType};
use crate::utils::sync;

const DEFAULT_LIMIT: usize = 25;
const MAX_LIMIT: usize = 100;

pub async fn get_repo_dashboard(
    client: &SupabaseClient,
    repo_id: &str,
    user_id: &str,
) -> Result<RepoDashboardResponse, String> {
    ensure_local_repo(repo_id)?;
    let repo = load_repository(client, repo_id).await?;
    let branch_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM branches WHERE repo_id = $1")
            .bind(repo_id)
            .fetch_one(&client.pool)
            .await
            .map_err(|error| format!("[ERROR] Failed to count branches for '{}': {}", repo_id, error))?;
    let commit_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM commits_metadata WHERE repo_id = $1")
            .bind(repo_id)
            .fetch_one(&client.pool)
            .await
            .map_err(|error| format!("[ERROR] Failed to count commits for '{}': {}", repo_id, error))?;

    let latest_commit = match resolve_ref_head(client, repo_id, None).await? {
        Some((_, head)) => load_commit_summary(client, repo_id, &head).await?,
        None => None,
    };

    let activity_row = sqlx::query(
        "SELECT
            COUNT(*)::bigint AS total_events,
            COUNT(*) FILTER (WHERE action = 'push')::bigint AS push_count,
            COUNT(*) FILTER (WHERE action = 'pull')::bigint AS pull_count
         FROM repo_access_logs
         WHERE repo_id = $1",
    )
    .bind(repo_id)
    .fetch_one(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to load activity summary for '{}': {}", repo_id, error))?;

    let push_count = activity_row.get::<i64, _>("push_count");
    let pull_count = activity_row.get::<i64, _>("pull_count");
    let commit_event_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM commits_metadata WHERE repo_id = $1")
            .bind(repo_id)
            .fetch_one(&client.pool)
            .await
            .map_err(|error| format!("[ERROR] Failed to count commit events for '{}': {}", repo_id, error))?;

    let starred_by_me: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM stars WHERE repo_id = $1 AND user_id = $2)",
    )
    .bind(repo_id)
    .bind(user_id)
    .fetch_one(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to load star state for '{}': {}", repo_id, error))?;

    let file_summary = match resolve_ref_head(client, repo_id, None).await? {
        Some((_, head)) => summarize_commit_tree(&head)?,
        None => RepositoryFileSummary {
            files: 0,
            directories: 0,
        },
    };

    let readme_preview = match (
        repo.readme_path.as_deref(),
        resolve_ref_head(client, repo_id, None).await?,
    ) {
        (Some(path), Some((ref_name, head))) if !path.trim().is_empty() => {
            read_readme_preview(repo_id, &ref_name, &head, path).ok()
        }
        _ => None,
    };

    Ok(RepoDashboardResponse {
        repo,
        branch_count,
        commit_count,
        latest_commit,
        activity_summary: crate::api::models::DashboardActivitySummary {
            total_events: activity_row.get::<i64, _>("total_events") + commit_event_count,
            push_count,
            pull_count,
            commit_count: commit_event_count,
        },
        file_summary,
        starred_by_me,
        readme_preview,
    })
}

pub async fn get_commit_history(
    client: &SupabaseClient,
    repo_id: &str,
    ref_name: Option<&str>,
    limit: usize,
    offset: usize,
) -> Result<PaginationResponse<CommitSummary>, String> {
    ensure_local_repo(repo_id)?;
    let Some((_, head)) = resolve_ref_head(client, repo_id, ref_name).await? else {
        return Ok(PaginationResponse {
            items: Vec::new(),
            next_offset: None,
        });
    };

    let rows = sqlx::query(
        "WITH RECURSIVE commit_chain AS (
            SELECT c.hash, c.parent_hash
            FROM commits c
            WHERE c.hash = $2
            UNION ALL
            SELECT parent.hash, parent.parent_hash
            FROM commits parent
            JOIN commit_chain child ON parent.hash = child.parent_hash
         )
         SELECT
            c.hash,
            c.parent_hash,
            c.message,
            c.created_at,
            COALESCE(cm.additions, 0) AS additions,
            COALESCE(cm.deletions, 0) AS deletions,
            u.id AS author_id,
            u.username,
            u.email
         FROM commit_chain chain
         JOIN commits c ON c.hash = chain.hash
         LEFT JOIN commits_metadata cm ON cm.commit_hash = c.hash AND cm.repo_id = $1
         LEFT JOIN users u ON u.id = c.author_id
         ORDER BY c.created_at DESC, c.hash DESC
         LIMIT $3 OFFSET $4",
    )
    .bind(repo_id)
    .bind(&head)
    .bind((limit + 1) as i64)
    .bind(offset as i64)
    .fetch_all(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to load commit history for '{}': {}", repo_id, error))?;

    let mut items: Vec<CommitSummary> = rows
        .into_iter()
        .map(map_commit_summary_row)
        .collect::<Result<Vec<_>, _>>()?;

    let next_offset = if items.len() > limit {
        items.pop();
        Some(offset + limit)
    } else {
        None
    };

    Ok(PaginationResponse { items, next_offset })
}

pub async fn get_commit_graph(
    client: &SupabaseClient,
    repo_id: &str,
    ref_name: Option<&str>,
    limit: usize,
) -> Result<CommitGraphResponse, String> {
    ensure_local_repo(repo_id)?;
    let Some((resolved_ref, head)) = resolve_ref_head(client, repo_id, ref_name).await? else {
        return Ok(CommitGraphResponse {
            repo_id: repo_id.to_string(),
            r#ref: ref_name.unwrap_or("").to_string(),
            head: String::new(),
            nodes: Vec::new(),
        });
    };

    let branches_by_head = load_branches_by_head(client, repo_id).await?;
    let rows = sqlx::query(
        "WITH RECURSIVE commit_chain AS (
            SELECT c.hash, c.parent_hash, 0 AS depth
            FROM commits c
            WHERE c.hash = $2
            UNION ALL
            SELECT parent.hash, parent.parent_hash, child.depth + 1
            FROM commits parent
            JOIN commit_chain child ON parent.hash = child.parent_hash
            WHERE child.depth + 1 < $3
         )
         SELECT DISTINCT
            c.hash,
            c.parent_hash,
            c.message,
            c.created_at,
            u.id AS author_id,
            u.username,
            u.email
         FROM commit_chain chain
         JOIN commits c ON c.hash = chain.hash
         LEFT JOIN users u ON u.id = c.author_id
         ORDER BY c.created_at DESC, c.hash DESC",
    )
    .bind(repo_id)
    .bind(&head)
    .bind(limit as i64)
    .fetch_all(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to load commit graph for '{}': {}", repo_id, error))?;

    let mut nodes = Vec::with_capacity(rows.len());
    for row in rows {
        let hash = row.get::<String, _>("hash");
        let parent_hash = row.get::<Option<String>, _>("parent_hash");
        nodes.push(CommitGraphNode {
            hash: hash.clone(),
            parent_hashes: parent_hash.into_iter().collect(),
            message: row.get::<String, _>("message"),
            created_at: row.get::<String, _>("created_at"),
            author: UserSummary {
                id: row
                    .get::<Option<String>, _>("author_id")
                    .unwrap_or_default(),
                username: row.get::<Option<String>, _>("username"),
                email: row.get::<Option<String>, _>("email"),
            },
            branches: branches_by_head.get(&hash).cloned().unwrap_or_default(),
        });
    }

    Ok(CommitGraphResponse {
        repo_id: repo_id.to_string(),
        r#ref: resolved_ref,
        head,
        nodes,
    })
}

pub async fn get_repo_contents(
    client: &SupabaseClient,
    repo_id: &str,
    ref_name: Option<&str>,
    path: Option<&str>,
) -> Result<ContentsResponse, String> {
    ensure_local_repo(repo_id)?;
    let requested_path = normalize_repo_path(path.unwrap_or(""));
    let Some((resolved_ref, head)) = resolve_ref_head(client, repo_id, ref_name).await? else {
        return Ok(ContentsResponse {
            repo_id: repo_id.to_string(),
            r#ref: ref_name.unwrap_or("").to_string(),
            path: requested_path,
            tree_hash: None,
            items: Vec::new(),
        });
    };

    let tree_hash = resolve_tree_at_path(&head, &requested_path)?;
    let items = match tree_hash.as_deref() {
        Some(hash) => list_tree_entries(hash, &requested_path)?,
        None => Vec::new(),
    };

    Ok(ContentsResponse {
        repo_id: repo_id.to_string(),
        r#ref: resolved_ref,
        path: requested_path,
        tree_hash,
        items,
    })
}

pub async fn get_repo_file(
    client: &SupabaseClient,
    repo_id: &str,
    ref_name: Option<&str>,
    path: &str,
) -> Result<FileContentResponse, String> {
    ensure_local_repo(repo_id)?;
    let normalized_path = normalize_repo_path(path);
    let Some((resolved_ref, head)) = resolve_ref_head(client, repo_id, ref_name).await? else {
        return Err(format!("[ERROR] Repository '{}' has no commits yet", repo_id));
    };

    let (blob_hash, content) = read_blob_at_path(&head, &normalized_path)?;
    let (encoding, serialized) = match String::from_utf8(content.clone()) {
        Ok(text) => ("utf-8".to_string(), text),
        Err(_) => ("base64".to_string(), STANDARD.encode(content.as_slice())),
    };

    Ok(FileContentResponse {
        repo_id: repo_id.to_string(),
        r#ref: resolved_ref,
        path: normalized_path,
        blob_hash,
        size: content.len(),
        encoding,
        content: serialized,
    })
}

pub async fn get_activity_feed(
    client: &SupabaseClient,
    repo_id: &str,
    action: Option<&str>,
    limit: usize,
    offset: usize,
) -> Result<PaginationResponse<ActivityFeedItem>, String> {
    ensure_local_repo(repo_id)?;
    let limit_with_buffer = limit + offset + 1;
    let logs = sqlx::query(
        "SELECT l.action, l.metadata, l.created_at, u.id AS actor_id, u.username, u.email
         FROM repo_access_logs l
         JOIN users u ON u.id = l.user_id
         WHERE l.repo_id = $1
         ORDER BY l.created_at DESC
         LIMIT $2",
    )
    .bind(repo_id)
    .bind(limit_with_buffer as i64)
    .fetch_all(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to load activity logs for '{}': {}", repo_id, error))?;

    let commits = sqlx::query(
        "SELECT cm.commit_hash, cm.message, cm.additions, cm.deletions, cm.created_at,
                u.id AS actor_id, u.username, u.email
         FROM commits_metadata cm
         JOIN users u ON u.id = cm.author_id
         WHERE cm.repo_id = $1
         ORDER BY cm.created_at DESC
         LIMIT $2",
    )
    .bind(repo_id)
    .bind(limit_with_buffer as i64)
    .fetch_all(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to load commit events for '{}': {}", repo_id, error))?;

    let mut items = Vec::with_capacity(logs.len() + commits.len());
    for row in logs {
        let action_value = row
            .get::<Option<String>, _>("action")
            .unwrap_or_else(|| "activity".to_string());
        items.push(ActivityFeedItem {
            r#type: action_value.clone(),
            action: action_value.clone(),
            actor: UserSummary {
                id: row.get::<String, _>("actor_id"),
                username: row.get::<Option<String>, _>("username"),
                email: row.get::<Option<String>, _>("email"),
            },
            created_at: row.get::<String, _>("created_at"),
            message: format!("{} event", action_value),
            metadata: row
                .get::<Option<serde_json::Value>, _>("metadata")
                .unwrap_or_else(|| serde_json::json!({})),
        });
    }

    for row in commits {
        items.push(ActivityFeedItem {
            r#type: "commit".to_string(),
            action: "commit".to_string(),
            actor: UserSummary {
                id: row.get::<String, _>("actor_id"),
                username: row.get::<Option<String>, _>("username"),
                email: row.get::<Option<String>, _>("email"),
            },
            created_at: row.get::<String, _>("created_at"),
            message: row.get::<String, _>("message"),
            metadata: serde_json::json!({
                "commit_hash": row.get::<String, _>("commit_hash"),
                "additions": row.get::<Option<i64>, _>("additions").unwrap_or(0),
                "deletions": row.get::<Option<i64>, _>("deletions").unwrap_or(0)
            }),
        });
    }

    items.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    if let Some(action_filter) = action {
        items.retain(|item| item.action == action_filter);
    }

    let next_offset = if items.len() > offset + limit {
        Some(offset + limit)
    } else {
        None
    };
    let paged_items = items.into_iter().skip(offset).take(limit).collect();

    Ok(PaginationResponse {
        items: paged_items,
        next_offset,
    })
}

pub async fn get_analytics_overview(
    client: &SupabaseClient,
    repo_id: &str,
) -> Result<AnalyticsOverviewResponse, String> {
    ensure_local_repo(repo_id)?;
    load_repository(client, repo_id).await?;

    let row = sqlx::query(
        "SELECT
            (SELECT COUNT(*) FROM branches WHERE repo_id = $1)::bigint AS branches_count,
            (SELECT COUNT(*) FROM commits_metadata WHERE repo_id = $1)::bigint AS commits_count,
            (SELECT COUNT(*) FROM stars WHERE repo_id = $1)::bigint AS stars_count,
            (SELECT COUNT(*) FROM repo_access_logs WHERE repo_id = $1 AND action = 'push')::bigint AS push_count,
            (SELECT COUNT(*) FROM repo_access_logs WHERE repo_id = $1 AND action = 'pull')::bigint AS pull_count,
            (SELECT MAX(created_at)::text FROM repo_access_logs WHERE repo_id = $1 AND action = 'push') AS last_push_at,
            (SELECT MAX(created_at)::text FROM repo_access_logs WHERE repo_id = $1 AND action = 'pull') AS last_pull_at,
            (SELECT COUNT(DISTINCT author_id) FROM commits_metadata WHERE repo_id = $1)::bigint AS contributors_count",
    )
    .bind(repo_id)
    .fetch_one(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to load analytics for '{}': {}", repo_id, error))?;

    Ok(AnalyticsOverviewResponse {
        repo_id: repo_id.to_string(),
        branches_count: row.get::<i64, _>("branches_count"),
        commits_count: row.get::<i64, _>("commits_count"),
        stars_count: row.get::<i64, _>("stars_count"),
        push_count: row.get::<i64, _>("push_count"),
        pull_count: row.get::<i64, _>("pull_count"),
        last_push_at: row.get::<Option<String>, _>("last_push_at"),
        last_pull_at: row.get::<Option<String>, _>("last_pull_at"),
        contributors_count: row.get::<i64, _>("contributors_count"),
    })
}

pub fn normalize_limit(value: Option<usize>) -> usize {
    value.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
}

fn normalize_repo_path(path: &str) -> String {
    let trimmed = path.trim().replace('\\', "/");
    trimmed.trim_start_matches('/').trim_start_matches("./").to_string()
}

fn map_commit_summary_row(row: sqlx::postgres::PgRow) -> Result<CommitSummary, String> {
    Ok(CommitSummary {
        hash: row.get::<String, _>("hash"),
        parent_hash: row.get::<Option<String>, _>("parent_hash"),
        message: row.get::<String, _>("message"),
        created_at: row.get::<String, _>("created_at"),
        additions: row.get::<i64, _>("additions"),
        deletions: row.get::<i64, _>("deletions"),
        author: UserSummary {
            id: row
                .get::<Option<String>, _>("author_id")
                .unwrap_or_default(),
            username: row.get::<Option<String>, _>("username"),
            email: row.get::<Option<String>, _>("email"),
        },
    })
}

async fn load_repository(client: &SupabaseClient, repo_id: &str) -> Result<Repository, String> {
    sqlx::query_as::<_, Repository>("SELECT * FROM repositories WHERE id = $1")
        .bind(repo_id)
        .fetch_optional(&client.pool)
        .await
        .map_err(|error| format!("[ERROR] Failed to load repository '{}': {}", repo_id, error))?
        .ok_or_else(|| format!("[ERROR] Repository '{}' not found", repo_id))
}

async fn resolve_ref_head(
    client: &SupabaseClient,
    repo_id: &str,
    ref_name: Option<&str>,
) -> Result<Option<(String, String)>, String> {
    let requested_ref = ref_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let repo = load_repository(client, repo_id).await?;
    let branch_name = requested_ref
        .clone()
        .unwrap_or_else(|| repo.default_branch.clone());

    let branch = sqlx::query(
        "SELECT last_commit_hash FROM branches WHERE repo_id = $1 AND name = $2 ORDER BY created_at ASC LIMIT 1",
    )
    .bind(repo_id)
    .bind(&branch_name)
    .fetch_optional(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to resolve branch '{}' for '{}': {}", branch_name, repo_id, error))?;

    if let Some(row) = branch {
        let head = row.get::<Option<String>, _>("last_commit_hash");
        return Ok(head.map(|commit_hash| (branch_name, commit_hash)));
    }

    if let Some(raw_ref) = requested_ref {
        return Ok(Some((raw_ref.clone(), raw_ref)));
    }

    Ok(None)
}

async fn load_commit_summary(
    client: &SupabaseClient,
    repo_id: &str,
    commit_hash: &str,
) -> Result<Option<CommitSummary>, String> {
    let row = sqlx::query(
        "SELECT
            c.hash,
            c.parent_hash,
            c.message,
            c.created_at,
            COALESCE(cm.additions, 0) AS additions,
            COALESCE(cm.deletions, 0) AS deletions,
            u.id AS author_id,
            u.username,
            u.email
         FROM commits c
         LEFT JOIN commits_metadata cm ON cm.commit_hash = c.hash AND cm.repo_id = $1
         LEFT JOIN users u ON u.id = c.author_id
         WHERE c.hash = $2",
    )
    .bind(repo_id)
    .bind(commit_hash)
    .fetch_optional(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to load commit '{}' for '{}': {}", commit_hash, repo_id, error))?;

    row.map(map_commit_summary_row).transpose()
}

async fn load_branches_by_head(
    client: &SupabaseClient,
    repo_id: &str,
) -> Result<HashMap<String, Vec<String>>, String> {
    let rows = sqlx::query("SELECT name, last_commit_hash FROM branches WHERE repo_id = $1")
        .bind(repo_id)
        .fetch_all(&client.pool)
        .await
        .map_err(|error| format!("[ERROR] Failed to load branches for '{}': {}", repo_id, error))?;

    let mut map = HashMap::new();
    for row in rows {
        if let Some(hash) = row.get::<Option<String>, _>("last_commit_hash") {
            map.entry(hash)
                .or_insert_with(Vec::new)
                .push(row.get::<String, _>("name"));
        }
    }
    Ok(map)
}

fn ensure_local_repo(repo_id: &str) -> Result<(), String> {
    let expected_repo = sync::repo_id_from_cwd()?;
    if repo_id.trim() != expected_repo {
        return Err(format!(
            "[ERROR] Unknown repo '{}', expected '{}'",
            repo_id.trim(),
            expected_repo
        ));
    }
    Ok(())
}

fn summarize_commit_tree(commit_hash: &str) -> Result<RepositoryFileSummary, String> {
    if commit_hash.trim().is_empty() {
        return Ok(RepositoryFileSummary {
            files: 0,
            directories: 0,
        });
    }

    let parsed = object_store::read_object(commit_hash)?;
    let tree_hash = parse_commit_tree_hash(&parsed.content)?;
    let mut files = 0usize;
    let mut directories = 0usize;
    count_tree_entries(&tree_hash, &mut files, &mut directories)?;
    Ok(RepositoryFileSummary { files, directories })
}

fn count_tree_entries(tree_hash: &str, files: &mut usize, directories: &mut usize) -> Result<(), String> {
    let tree = object_store::read_object(tree_hash)?;
    for entry in object_store::parse_tree(&tree.content)? {
        match entry.object_type {
            ObjectType::Blob => *files += 1,
            ObjectType::Tree => {
                *directories += 1;
                count_tree_entries(&entry.hash, files, directories)?;
            }
            ObjectType::Commit => {}
        }
    }
    Ok(())
}

fn parse_commit_tree_hash(content: &[u8]) -> Result<String, String> {
    let commit_text = String::from_utf8(content.to_vec())
        .map_err(|error| format!("[ERROR] Invalid commit content: {}", error))?;
    commit_text
        .lines()
        .find_map(|line| line.strip_prefix("tree "))
        .map(str::trim)
        .map(str::to_string)
        .ok_or_else(|| "[ERROR] Commit missing tree hash".to_string())
}

fn resolve_tree_at_path(commit_hash: &str, path: &str) -> Result<Option<String>, String> {
    if commit_hash.trim().is_empty() {
        return Ok(None);
    }

    let parsed = object_store::read_object(commit_hash)?;
    let mut current_tree = parse_commit_tree_hash(&parsed.content)?;
    if path.trim().is_empty() {
        return Ok(Some(current_tree));
    }

    for component in Path::new(path).components() {
        let name = component.as_os_str().to_string_lossy();
        let tree = object_store::read_object(&current_tree)?;
        let entries = object_store::parse_tree(&tree.content)?;
        let Some(entry) = entries.into_iter().find(|entry| entry.name == name) else {
            return Err(format!("[ERROR] Path '{}' not found", path));
        };
        if entry.object_type != ObjectType::Tree {
            return Err(format!("[ERROR] Path '{}' is not a directory", path));
        }
        current_tree = entry.hash;
    }

    Ok(Some(current_tree))
}

fn list_tree_entries(tree_hash: &str, prefix: &str) -> Result<Vec<ContentEntry>, String> {
    let tree = object_store::read_object(tree_hash)?;
    let mut items = Vec::new();
    for entry in object_store::parse_tree(&tree.content)? {
        let path = join_repo_path(prefix, &entry.name);
        let size = if entry.object_type == ObjectType::Blob {
            let blob = object_store::read_object(&entry.hash)?;
            Some(blob.content.len())
        } else {
            None
        };
        items.push(ContentEntry {
            name: entry.name,
            path,
            r#type: match entry.object_type {
                ObjectType::Blob => "file".to_string(),
                ObjectType::Tree => "dir".to_string(),
                ObjectType::Commit => "commit".to_string(),
            },
            hash: entry.hash,
            mode: entry.mode,
            size,
        });
    }
    items.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(items)
}

fn read_blob_at_path(commit_hash: &str, path: &str) -> Result<(String, Vec<u8>), String> {
    let mut current_tree = {
        let parsed = object_store::read_object(commit_hash)?;
        parse_commit_tree_hash(&parsed.content)?
    };
    let normalized = PathBuf::from(path);
    let components: Vec<_> = normalized.components().collect();
    if components.is_empty() {
        return Err("[ERROR] Missing file path".to_string());
    }

    for (index, component) in components.iter().enumerate() {
        let name = component.as_os_str().to_string_lossy();
        let tree = object_store::read_object(&current_tree)?;
        let entries = object_store::parse_tree(&tree.content)?;
        let Some(entry) = entries.into_iter().find(|entry| entry.name == name) else {
            return Err(format!("[ERROR] Path '{}' not found", path));
        };

        let is_last = index + 1 == components.len();
        match (is_last, entry.object_type) {
            (true, ObjectType::Blob) => {
                let blob = object_store::read_object(&entry.hash)?;
                return Ok((entry.hash, blob.content));
            }
            (false, ObjectType::Tree) => current_tree = entry.hash,
            (true, _) => return Err(format!("[ERROR] Path '{}' is not a file", path)),
            (false, _) => return Err(format!("[ERROR] Path '{}' is not a directory", path)),
        }
    }

    Err(format!("[ERROR] Path '{}' not found", path))
}

fn read_readme_preview(
    repo_id: &str,
    ref_name: &str,
    head: &str,
    path: &str,
) -> Result<ReadmePreview, String> {
    let (blob_hash, content) = read_blob_at_path(head, path)?;
    let (encoding, serialized) = match String::from_utf8(content.clone()) {
        Ok(text) => ("utf-8".to_string(), text.chars().take(4000).collect()),
        Err(_) => ("base64".to_string(), STANDARD.encode(content.as_slice())),
    };

    Ok(ReadmePreview {
        path: format!("{}:{}", repo_id, join_repo_path(ref_name, path)),
        blob_hash,
        encoding,
        content: serialized,
    })
}

fn join_repo_path(prefix: &str, name: &str) -> String {
    if prefix.trim().is_empty() {
        name.to_string()
    } else {
        format!("{}/{}", prefix.trim_end_matches('/'), name)
    }
}

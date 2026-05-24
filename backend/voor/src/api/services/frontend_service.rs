use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sqlx::Row;

use crate::api::clients::supabase::SupabaseClient;
use crate::api::models::{
    ActivityFeedItem, AnalyticsOverviewResponse, BranchCommitDistributionItem, CommitGraphNode,
    CommitGraphResponse, CommitSummary, ContentEntry, ContentsResponse, FileContentResponse,
    PaginationResponse, ReadmePreview, RepoDashboardResponse, Repository,
    RepositoryDagMetricsResponse, RepositoryFileSummary, RepositoryStorageSummary, TopModifiedFile,
    UserSummary, VcsAnalyticsResponse, VcsBranchAnalytics, VcsTimelineBucket, VcsTopologyCacheItem,
};
use crate::utils::object_store::{self, ObjectType};

const DEFAULT_LIMIT: usize = 25;
const MAX_LIMIT: usize = 100;

pub async fn get_repo_dashboard(
    client: &SupabaseClient,
    repo_id: &str,
    user_id: &str,
) -> Result<RepoDashboardResponse, String> {
    ensure_local_repo(repo_id)?;
    let repo = load_repository(client, repo_id).await?;
    let branch_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM branches WHERE repo_id = $1")
        .bind(repo_id)
        .fetch_one(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to count branches for '{}': {}",
                repo_id, error
            )
        })?;
    let commit_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM commits_metadata WHERE repo_id = $1")
            .bind(repo_id)
            .fetch_one(&client.pool)
            .await
            .map_err(|error| {
                format!(
                    "[ERROR] Failed to count commits for '{}': {}",
                    repo_id, error
                )
            })?;

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
    .map_err(|error| {
        format!(
            "[ERROR] Failed to load activity summary for '{}': {}",
            repo_id, error
        )
    })?;

    let push_count = activity_row.get::<i64, _>("push_count");
    let pull_count = activity_row.get::<i64, _>("pull_count");
    let commit_event_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM commits_metadata WHERE repo_id = $1")
            .bind(repo_id)
            .fetch_one(&client.pool)
            .await
            .map_err(|error| {
                format!(
                    "[ERROR] Failed to count commit events for '{}': {}",
                    repo_id, error
                )
            })?;

    let starred_by_me: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM stars WHERE repo_id = $1 AND user_id = $2)",
    )
    .bind(repo_id)
    .bind(user_id)
    .fetch_one(&client.pool)
    .await
    .map_err(|error| {
        format!(
            "[ERROR] Failed to load star state for '{}': {}",
            repo_id, error
        )
    })?;

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
            c.created_at::text AS created_at,
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
    .map_err(|error| {
        format!(
            "[ERROR] Failed to load commit history for '{}': {}",
            repo_id, error
        )
    })?;

    let mut items: Vec<CommitSummary> = rows
        .into_iter()
        .map(map_commit_summary_row)
        .collect::<Result<Vec<_>, _>>()?;

    if items.is_empty() {
        let fallback_rows = sqlx::query(
            "SELECT
                cm.commit_hash AS hash,
                NULL::text AS parent_hash,
                cm.message,
                cm.created_at::text AS created_at,
                COALESCE(cm.additions, 0) AS additions,
                COALESCE(cm.deletions, 0) AS deletions,
                u.id AS author_id,
                u.username,
                u.email
             FROM commits_metadata cm
             LEFT JOIN users u ON u.id = cm.author_id
             WHERE cm.repo_id = $1
             ORDER BY cm.created_at DESC, cm.commit_hash DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(repo_id)
        .bind((limit + 1) as i64)
        .bind(offset as i64)
        .fetch_all(&client.pool)
        .await
        .map_err(|error| {
            format!(
                "[ERROR] Failed to load commit metadata history for '{}': {}",
                repo_id, error
            )
        })?;

        items = fallback_rows
            .into_iter()
            .map(map_commit_summary_row)
            .collect::<Result<Vec<_>, _>>()?;
    }

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
        "WITH RECURSIVE commit_chain(hash, depth) AS (
            SELECT $2::text AS hash, 0 AS depth
            UNION
            SELECT ce.parent_hash, chain.depth + 1
            FROM commit_edges ce
            JOIN commit_chain chain ON ce.child_hash = chain.hash
            WHERE ce.repo_id = $1 AND chain.depth + 1 < $3
            UNION
            SELECT c.parent_hash, chain.depth + 1
            FROM commits c
            JOIN commit_chain chain ON c.hash = chain.hash
            WHERE c.parent_hash IS NOT NULL AND chain.depth + 1 < $3
         )
         SELECT DISTINCT
            c.hash,
            COALESCE(
                (
                    SELECT array_agg(ce.parent_hash ORDER BY ce.parent_index)
                    FROM commit_edges ce
                    WHERE ce.repo_id = $1 AND ce.child_hash = c.hash
                ),
                CASE
                    WHEN c.parent_hash IS NULL THEN ARRAY[]::text[]
                    ELSE ARRAY[c.parent_hash]
                END
            ) AS parent_hashes,
            c.message,
            c.created_at::text AS created_at,
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
    .map_err(|error| {
        format!(
            "[ERROR] Failed to load commit graph for '{}': {}",
            repo_id, error
        )
    })?;

    let mut nodes = Vec::with_capacity(rows.len());
    for row in rows {
        let hash = row.get::<String, _>("hash");
        nodes.push(CommitGraphNode {
            hash: hash.clone(),
            parent_hashes: row.get::<Vec<String>, _>("parent_hashes"),
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
        return Err(format!(
            "[ERROR] Repository '{}' has no commits yet",
            repo_id
        ));
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
        "SELECT l.action, l.metadata, l.created_at::text AS created_at, u.id AS actor_id, u.username, u.email
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
        "SELECT cm.commit_hash, cm.message, cm.additions, cm.deletions, cm.created_at::text AS created_at,
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

    let storage_summary = summarize_repository_storage(client, repo_id)
        .await
        .unwrap_or(RepositoryStorageSummary {
            bytes: 0,
            objects: 0,
        });
    let branch_commit_distribution = summarize_branch_commit_distribution(client, repo_id)
        .await
        .unwrap_or_default();

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
        repository_size_bytes: storage_summary.bytes,
        object_count: storage_summary.objects,
        branch_commit_distribution,
    })
}

pub async fn get_vcs_analytics(
    client: &SupabaseClient,
    repo_id: &str,
) -> Result<VcsAnalyticsResponse, String> {
    ensure_local_repo(repo_id)?;
    let repo = load_repository(client, repo_id).await?;

    let branch_rows = sqlx::query(
        "SELECT
            b.id::text AS branch_id,
            b.name,
            b.last_commit_hash,
            b.created_at::text AS created_at,
            b.last_activity_at::text AS last_activity_at,
            b.last_analyzed_at::text AS last_analyzed_at,
            COALESCE(b.is_default_cached, false) OR b.name = r.default_branch AS is_default,
            COALESCE(bm.default_branch_name, r.default_branch) AS default_branch_name,
            COALESCE(bm.head_commit_hash, b.last_commit_hash) AS head_commit_hash,
            bm.default_head_hash,
            bm.merge_base_hash,
            COALESCE(bm.ahead_count, 0)::bigint AS ahead_count,
            COALESCE(bm.behind_count, 0)::bigint AS behind_count,
            COALESCE(bm.divergence_distance, 0)::bigint AS divergence_distance,
            bm.freshness_status,
            bm.freshness_score::double precision AS freshness_score,
            bm.health_score::double precision AS health_score,
            COALESCE(bm.stale_days, 0)::int AS stale_days,
            bm.computed_at::text AS computed_at,
            btm.lane_index,
            btm.lane_color,
            btm.start_commit_hash,
            btm.head_commit_hash AS topology_head_commit_hash,
            btm.merge_base_hash AS topology_merge_base_hash,
            btm.first_seen_at::text AS first_seen_at,
            btm.last_seen_at::text AS last_seen_at,
            btm.commit_density::double precision AS commit_density,
            btm.activity_heat::double precision AS activity_heat,
            COALESCE(bm.commit_count, btm.commit_count, 0)::bigint AS commit_count,
            COALESCE(bm.activity_score, 0)::double precision AS activity_score,
            bm.latest_commit_at::text AS metrics_latest_commit_at,
            bm.latest_contributor AS metrics_latest_contributor,
            c.hash AS latest_hash,
            COALESCE(
                (
                    SELECT array_agg(ce.parent_hash ORDER BY ce.parent_index)
                    FROM commit_edges ce
                    WHERE ce.repo_id = b.repo_id AND ce.child_hash = c.hash
                ),
                CASE
                    WHEN c.parent_hash IS NULL THEN ARRAY[]::text[]
                    ELSE ARRAY[c.parent_hash]
                END
            ) AS latest_parent_hashes,
            c.message AS latest_message,
            c.created_at::text AS latest_created_at,
            u.id AS latest_author_id,
            u.username AS latest_username,
            u.email AS latest_email
         FROM branches b
         JOIN repositories r ON r.id = b.repo_id
         LEFT JOIN branch_metrics bm ON bm.repo_id = b.repo_id AND bm.branch_name = b.name
         LEFT JOIN branch_topology_metrics btm ON btm.repo_id = b.repo_id AND btm.branch_name = b.name
         LEFT JOIN commits c ON c.hash = COALESCE(bm.head_commit_hash, b.last_commit_hash)
         LEFT JOIN users u ON u.id = c.author_id
         WHERE b.repo_id = $1
         ORDER BY
            CASE WHEN b.name = r.default_branch THEN 0 ELSE 1 END,
            COALESCE(btm.lane_index, 999),
            COALESCE(bm.divergence_distance, 0) DESC,
            b.name ASC",
    )
    .bind(repo_id)
    .fetch_all(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to load VCS branch analytics for '{}': {}", repo_id, error))?;

    let branches = branch_rows
        .into_iter()
        .map(|row| {
            let latest_hash = row.get::<Option<String>, _>("latest_hash");
            let latest_commit = latest_hash.map(|hash| CommitGraphNode {
                hash,
                parent_hashes: row.get::<Vec<String>, _>("latest_parent_hashes"),
                message: row
                    .get::<Option<String>, _>("latest_message")
                    .unwrap_or_default(),
                created_at: row
                    .get::<Option<String>, _>("latest_created_at")
                    .unwrap_or_default(),
                author: UserSummary {
                    id: row
                        .get::<Option<String>, _>("latest_author_id")
                        .unwrap_or_default(),
                    username: row.get::<Option<String>, _>("latest_username"),
                    email: row.get::<Option<String>, _>("latest_email"),
                },
                branches: vec![row.get::<String, _>("name")],
            });

            VcsBranchAnalytics {
                id: row.get::<Option<String>, _>("branch_id"),
                name: row.get::<String, _>("name"),
                last_commit_hash: row.get::<Option<String>, _>("last_commit_hash"),
                created_at: row.get::<String, _>("created_at"),
                last_activity_at: row.get::<Option<String>, _>("last_activity_at"),
                last_analyzed_at: row.get::<Option<String>, _>("last_analyzed_at"),
                is_default: row.get::<bool, _>("is_default"),
                default_branch_name: row.get::<String, _>("default_branch_name"),
                head_commit_hash: row.get::<Option<String>, _>("head_commit_hash"),
                default_head_hash: row.get::<Option<String>, _>("default_head_hash"),
                merge_base_hash: row
                    .get::<Option<String>, _>("merge_base_hash")
                    .or_else(|| row.get::<Option<String>, _>("topology_merge_base_hash")),
                ahead_count: row.get::<i64, _>("ahead_count"),
                behind_count: row.get::<i64, _>("behind_count"),
                divergence_distance: row.get::<i64, _>("divergence_distance"),
                freshness_status: row.get::<Option<String>, _>("freshness_status"),
                freshness_score: row.get::<Option<f64>, _>("freshness_score"),
                health_score: row.get::<Option<f64>, _>("health_score"),
                stale_days: row.get::<i32, _>("stale_days"),
                computed_at: row.get::<Option<String>, _>("computed_at"),
                lane_index: row.get::<Option<i32>, _>("lane_index"),
                lane_color: row.get::<Option<String>, _>("lane_color"),
                start_commit_hash: row.get::<Option<String>, _>("start_commit_hash"),
                first_seen_at: row.get::<Option<String>, _>("first_seen_at"),
                last_seen_at: row.get::<Option<String>, _>("last_seen_at"),
                commit_density: row.get::<Option<f64>, _>("commit_density"),
                activity_heat: row.get::<Option<f64>, _>("activity_heat"),
                commit_count: row.get::<i64, _>("commit_count"),
                activity_score: row.get::<f64, _>("activity_score"),
                latest_commit_at: row.get::<Option<String>, _>("metrics_latest_commit_at"),
                latest_contributor: row.get::<Option<String>, _>("metrics_latest_contributor"),
                latest_commit,
            }
        })
        .collect();

    let topology_rows = sqlx::query(
        "SELECT
            branch_name,
            COALESCE(MAX(head_commit_hash), '') AS head_commit_hash,
            jsonb_agg(
                jsonb_build_object(
                    'hash', commit_hash,
                    'parent_hashes', COALESCE(parent_hashes, '[]'::jsonb),
                    'message', COALESCE(message, ''),
                    'created_at', committed_at,
                    'author', jsonb_build_object(
                        'id', COALESCE(author_id, ''),
                        'username', author_username,
                        'email', NULL
                    ),
                    'branches', jsonb_build_array(branch_name),
                    'depth_from_head', depth_from_head,
                    'is_head', is_head,
                    'is_merge_base', is_merge_base,
                    'is_default_branch', is_default_branch,
                    'lane_index', lane_index,
                    'lane_color', lane_color,
                    'x_position', x_position,
                    'y_position', y_position
                )
                ORDER BY depth_from_head DESC, committed_at ASC NULLS LAST, commit_hash ASC
            ) AS nodes,
            jsonb_agg(
                jsonb_build_object(
                    'child_hash', commit_hash,
                    'parent_hashes', COALESCE(parent_hashes, '[]'::jsonb)
                )
                ORDER BY depth_from_head ASC, commit_hash ASC
            ) FILTER (WHERE parent_hashes IS NOT NULL AND jsonb_array_length(parent_hashes) > 0) AS edges,
            jsonb_build_array(
                jsonb_build_object(
                    'branch_name', branch_name,
                    'lane_index', MAX(lane_index),
                    'lane_color', MAX(lane_color)
                )
            ) AS lanes,
            '[]'::jsonb AS clusters,
            MAX(last_seen_at)::text AS computed_at
         FROM (
            SELECT
                bcm.repo_id,
                bcm.branch_name,
                bcm.commit_hash,
                bcm.depth_from_head,
                bcm.is_head,
                bcm.is_merge_base,
                bcm.is_default_branch,
                bcm.lane_index,
                bcm.lane_color,
                bcm.x_position,
                bcm.y_position,
                bcm.message,
                bcm.author_id,
                bcm.author_username,
                bcm.committed_at::text AS committed_at,
                bcm.last_seen_at,
                COALESCE(bm.head_commit_hash, b.last_commit_hash) AS head_commit_hash,
                (
                    SELECT jsonb_agg(ce.parent_hash ORDER BY ce.parent_index)
                    FROM commit_edges ce
                    WHERE ce.repo_id = bcm.repo_id AND ce.child_hash = bcm.commit_hash
                ) AS parent_hashes
            FROM branch_commit_memberships bcm
            JOIN branches b ON b.id = bcm.branch_id
            LEFT JOIN branch_metrics bm ON bm.repo_id = bcm.repo_id AND bm.branch_name = bcm.branch_name
            WHERE bcm.repo_id = $1
         ) branch_nodes
         WHERE repo_id = $1
         GROUP BY branch_name
         ORDER BY MAX(last_seen_at) DESC NULLS LAST, branch_name ASC",
    )
    .bind(repo_id)
    .fetch_all(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to load topology cache for '{}': {}", repo_id, error))?;

    let topology_cache = topology_rows
        .into_iter()
        .map(|row| VcsTopologyCacheItem {
            branch_name: row.get::<String, _>("branch_name"),
            head_commit_hash: row.get::<String, _>("head_commit_hash"),
            layout_version: Some("branch-commit-memberships-v1".to_string()),
            nodes: row.get::<serde_json::Value, _>("nodes"),
            edges: row
                .get::<Option<serde_json::Value>, _>("edges")
                .unwrap_or_else(|| serde_json::json!([])),
            lanes: row.get::<serde_json::Value, _>("lanes"),
            clusters: row.get::<serde_json::Value, _>("clusters"),
            computed_at: row.get::<Option<String>, _>("computed_at"),
        })
        .collect();

    let dag_metrics = sqlx::query(
        "SELECT
            commit_dag_complexity::double precision AS commit_dag_complexity,
            dag_complexity_status,
            longest_chain_nodes::bigint AS longest_chain_nodes,
            open_pr_count::bigint AS open_pr_count,
            open_pr_delta_24h::bigint AS open_pr_delta_24h,
            total_commits::bigint AS total_commits,
            avg_divergence::double precision AS avg_divergence,
            stale_ratio::double precision AS stale_ratio,
            merge_velocity_per_week::double precision AS merge_velocity_per_week,
            branch_count::bigint AS branch_count,
            default_branch_name,
            computed_at::text AS computed_at,
            metadata
         FROM repository_dag_metrics
         WHERE repo_id = $1",
    )
    .bind(repo_id)
    .fetch_optional(&client.pool)
    .await
    .map_err(|error| {
        format!(
            "[ERROR] Failed to load repository DAG metrics for '{}': {}",
            repo_id, error
        )
    })?
    .map(|row| RepositoryDagMetricsResponse {
        commit_dag_complexity: row.get::<f64, _>("commit_dag_complexity"),
        dag_complexity_status: row.get::<String, _>("dag_complexity_status"),
        longest_chain_nodes: row.get::<i64, _>("longest_chain_nodes"),
        open_pr_count: row.get::<i64, _>("open_pr_count"),
        open_pr_delta_24h: row.get::<i64, _>("open_pr_delta_24h"),
        total_commits: row.get::<i64, _>("total_commits"),
        avg_divergence: row.get::<f64, _>("avg_divergence"),
        stale_ratio: row.get::<f64, _>("stale_ratio"),
        merge_velocity_per_week: row.get::<f64, _>("merge_velocity_per_week"),
        branch_count: row.get::<i64, _>("branch_count"),
        default_branch_name: row.get::<Option<String>, _>("default_branch_name"),
        computed_at: row.get::<String, _>("computed_at"),
        metadata: row.get::<serde_json::Value, _>("metadata"),
    });

    let timeline_rows = sqlx::query(
        "SELECT
            bucket_start::text AS bucket_start,
            bucket_granularity,
            COALESCE(commit_count, 0)::bigint AS commit_count,
            COALESCE(author_count, 0)::bigint AS author_count,
            COALESCE(branch_count, 0)::bigint AS branch_count,
            COALESCE(additions, 0)::bigint AS additions,
            COALESCE(deletions, 0)::bigint AS deletions,
            COALESCE(audit_event_count, 0)::bigint AS audit_event_count
         FROM timeline_aggregation
         WHERE repo_id = $1
         ORDER BY bucket_start DESC
         LIMIT 64",
    )
    .bind(repo_id)
    .fetch_all(&client.pool)
    .await
    .map_err(|error| {
        format!(
            "[ERROR] Failed to load VCS timeline for '{}': {}",
            repo_id, error
        )
    })?;

    let mut timeline: Vec<VcsTimelineBucket> = timeline_rows
        .into_iter()
        .map(|row| VcsTimelineBucket {
            bucket_start: row.get::<String, _>("bucket_start"),
            bucket_granularity: row.get::<String, _>("bucket_granularity"),
            commit_count: row.get::<i64, _>("commit_count"),
            author_count: row.get::<i64, _>("author_count"),
            branch_count: row.get::<i64, _>("branch_count"),
            additions: row.get::<i64, _>("additions"),
            deletions: row.get::<i64, _>("deletions"),
            audit_event_count: row.get::<i64, _>("audit_event_count"),
        })
        .collect();

    if timeline.is_empty() {
        timeline = load_commit_metadata_timeline(client, repo_id).await?;
    }

    let top_modified_files = summarize_top_modified_files(client, repo_id)
        .await
        .unwrap_or_default();

    Ok(VcsAnalyticsResponse {
        repo_id: repo_id.to_string(),
        default_branch: repo.default_branch,
        dag_metrics,
        branches,
        topology_cache,
        timeline,
        top_modified_files,
    })
}

pub fn normalize_limit(value: Option<usize>) -> usize {
    value.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
}

fn normalize_repo_path(path: &str) -> String {
    let trimmed = path.trim().replace('\\', "/");
    trimmed
        .trim_start_matches('/')
        .trim_start_matches("./")
        .to_string()
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
    sqlx::query_as::<_, Repository>(
        "SELECT id, name, owner_id, is_private, description, tags, default_branch,
                stars_count, readme_path, theme, created_at::text AS created_at
         FROM repositories
         WHERE id = $1",
    )
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
            c.created_at::text AS created_at,
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
    .map_err(|error| {
        format!(
            "[ERROR] Failed to load commit '{}' for '{}': {}",
            commit_hash, repo_id, error
        )
    })?;

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
        .map_err(|error| {
            format!(
                "[ERROR] Failed to load branches for '{}': {}",
                repo_id, error
            )
        })?;

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
    if repo_id.trim().is_empty() {
        return Err("[ERROR] Missing repo_id".to_string());
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

async fn summarize_repository_storage(
    client: &SupabaseClient,
    repo_id: &str,
) -> Result<RepositoryStorageSummary, String> {
    let heads = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT last_commit_hash FROM branches WHERE repo_id = $1 AND last_commit_hash IS NOT NULL",
    )
    .bind(repo_id)
    .fetch_all(&client.pool)
    .await
    .map_err(|error| format!("[ERROR] Failed to load branch heads for '{}': {}", repo_id, error))?;

    let mut seen = HashSet::new();
    let mut summary = RepositoryStorageSummary {
        bytes: 0,
        objects: 0,
    };

    for head in heads {
        add_reachable_object_size(&head, &mut seen, &mut summary)?;
    }

    Ok(summary)
}

async fn summarize_branch_commit_distribution(
    client: &SupabaseClient,
    repo_id: &str,
) -> Result<Vec<BranchCommitDistributionItem>, String> {
    let rows = sqlx::query(
        "WITH RECURSIVE branch_chain AS (
            SELECT b.name AS branch, c.hash, c.parent_hash
            FROM branches b
            JOIN commits c ON c.hash = b.last_commit_hash
            WHERE b.repo_id = $1 AND b.last_commit_hash IS NOT NULL
            UNION ALL
            SELECT chain.branch, parent.hash, parent.parent_hash
            FROM commits parent
            JOIN branch_chain chain ON parent.hash = chain.parent_hash
         )
         SELECT branch, COUNT(DISTINCT hash)::bigint AS total_count
         FROM branch_chain
         GROUP BY branch
         ORDER BY total_count DESC, branch ASC",
    )
    .bind(repo_id)
    .fetch_all(&client.pool)
    .await
    .map_err(|error| {
        format!(
            "[ERROR] Failed to summarize branch commits for '{}': {}",
            repo_id, error
        )
    })?;

    let total_commits: i64 = rows
        .iter()
        .map(|row| row.get::<i64, _>("total_count"))
        .sum();

    if total_commits == 0 {
        return Ok(Vec::new());
    }

    Ok(rows
        .into_iter()
        .map(|row| {
            let total_count = row.get::<i64, _>("total_count");
            BranchCommitDistributionItem {
                branch: row.get::<String, _>("branch"),
                total_count,
                percentage: ((total_count as f64 / total_commits as f64) * 1000.0).round() / 10.0,
            }
        })
        .collect())
}

fn add_reachable_object_size(
    hash: &str,
    seen: &mut HashSet<String>,
    summary: &mut RepositoryStorageSummary,
) -> Result<(), String> {
    let trimmed = hash.trim();
    if trimmed.is_empty() || !seen.insert(trimmed.to_string()) {
        return Ok(());
    }

    let parsed = object_store::read_object(trimmed)?;
    summary.bytes += parsed.full_bytes.len();
    summary.objects += 1;

    match parsed.object_type {
        ObjectType::Blob => {}
        ObjectType::Tree => {
            for entry in object_store::parse_tree(&parsed.content)? {
                add_reachable_object_size(&entry.hash, seen, summary)?;
            }
        }
        ObjectType::Commit => {
            let commit_text = String::from_utf8(parsed.content)
                .map_err(|error| format!("[ERROR] Invalid commit content: {}", error))?;
            for line in commit_text.lines() {
                if let Some(tree_hash) = line.strip_prefix("tree ") {
                    add_reachable_object_size(tree_hash, seen, summary)?;
                } else if let Some(parent_hash) = line.strip_prefix("parent ") {
                    add_reachable_object_size(parent_hash, seen, summary)?;
                }
            }
        }
    }

    Ok(())
}

fn count_tree_entries(
    tree_hash: &str,
    files: &mut usize,
    directories: &mut usize,
) -> Result<(), String> {
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

async fn load_commit_metadata_timeline(
    client: &SupabaseClient,
    repo_id: &str,
) -> Result<Vec<VcsTimelineBucket>, String> {
    let rows = sqlx::query(
        "SELECT
            date_trunc('day', created_at)::text AS bucket_start,
            COUNT(*)::bigint AS commit_count,
            COUNT(DISTINCT author_id)::bigint AS author_count,
            COALESCE(SUM(additions), 0)::bigint AS additions,
            COALESCE(SUM(deletions), 0)::bigint AS deletions
         FROM commits_metadata
         WHERE repo_id = $1
         GROUP BY date_trunc('day', created_at)
         ORDER BY date_trunc('day', created_at) DESC
         LIMIT 64",
    )
    .bind(repo_id)
    .fetch_all(&client.pool)
    .await
    .map_err(|error| {
        format!(
            "[ERROR] Failed to build commit timeline for '{}': {}",
            repo_id, error
        )
    })?;

    Ok(rows
        .into_iter()
        .map(|row| VcsTimelineBucket {
            bucket_start: row.get::<String, _>("bucket_start"),
            bucket_granularity: "day".to_string(),
            commit_count: row.get::<i64, _>("commit_count"),
            author_count: row.get::<i64, _>("author_count"),
            branch_count: 0,
            additions: row.get::<i64, _>("additions"),
            deletions: row.get::<i64, _>("deletions"),
            audit_event_count: 0,
        })
        .collect())
}

async fn summarize_top_modified_files(
    client: &SupabaseClient,
    repo_id: &str,
) -> Result<Vec<TopModifiedFile>, String> {
    let rows = sqlx::query(
        "SELECT c.hash, c.parent_hash
         FROM commits_metadata cm
         JOIN commits c ON c.hash = cm.commit_hash
         WHERE cm.repo_id = $1
         ORDER BY cm.created_at DESC, c.hash DESC
         LIMIT 300",
    )
    .bind(repo_id)
    .fetch_all(&client.pool)
    .await
    .map_err(|error| {
        format!(
            "[ERROR] Failed to load file modification commits for '{}': {}",
            repo_id, error
        )
    })?;

    let mut counts: HashMap<String, i64> = HashMap::new();
    for row in rows {
        let hash = row.get::<String, _>("hash");
        let parent_hash = row.get::<Option<String>, _>("parent_hash");
        if let Ok(changed_paths) = changed_paths_for_commit(&hash, parent_hash.as_deref()) {
            for path in changed_paths {
                *counts.entry(path).or_insert(0) += 1;
            }
        }
    }

    let total_changes: i64 = counts.values().sum();
    if total_changes == 0 {
        return Ok(Vec::new());
    }

    let mut files: Vec<TopModifiedFile> = counts
        .into_iter()
        .map(|(path, change_count)| TopModifiedFile {
            path,
            change_count,
            percentage: ((change_count as f64 / total_changes as f64) * 1000.0).round() / 10.0,
        })
        .collect();

    files.sort_by(|left, right| {
        right
            .change_count
            .cmp(&left.change_count)
            .then_with(|| left.path.cmp(&right.path))
    });
    files.truncate(5);

    Ok(files)
}

fn changed_paths_for_commit(
    commit_hash: &str,
    parent_hash: Option<&str>,
) -> Result<Vec<String>, String> {
    let commit = object_store::read_object(commit_hash)?;
    let tree_hash = parse_commit_tree_hash(&commit.content)?;
    let current_files = flatten_tree_blobs(&tree_hash, "")?;
    let parent_files = match parent_hash.map(str::trim).filter(|value| !value.is_empty()) {
        Some(parent) => {
            let parent_commit = object_store::read_object(parent)?;
            let parent_tree = parse_commit_tree_hash(&parent_commit.content)?;
            flatten_tree_blobs(&parent_tree, "")?
        }
        None => HashMap::new(),
    };

    let mut paths = HashSet::new();
    for (path, hash) in &current_files {
        if parent_files.get(path) != Some(hash) {
            paths.insert(path.clone());
        }
    }
    for path in parent_files.keys() {
        if !current_files.contains_key(path) {
            paths.insert(path.clone());
        }
    }

    Ok(paths.into_iter().collect())
}

fn flatten_tree_blobs(tree_hash: &str, prefix: &str) -> Result<HashMap<String, String>, String> {
    let tree = object_store::read_object(tree_hash)?;
    let mut files = HashMap::new();
    for entry in object_store::parse_tree(&tree.content)? {
        let path = join_repo_path(prefix, &entry.name);
        match entry.object_type {
            ObjectType::Blob => {
                files.insert(path, entry.hash);
            }
            ObjectType::Tree => {
                files.extend(flatten_tree_blobs(&entry.hash, &path)?);
            }
            ObjectType::Commit => {}
        }
    }
    Ok(files)
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

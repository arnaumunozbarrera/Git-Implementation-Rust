use serde::{Deserialize, Serialize};

use crate::api::models::Repository;

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationResponse<T> {
    pub items: Vec<T>,
    pub next_offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct CommitHistoryQuery {
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct CommitGraphQuery {
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct RepoPathQuery {
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ActivityFeedQuery {
    pub action: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSummary {
    pub id: String,
    pub username: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitSummary {
    pub hash: String,
    pub parent_hash: Option<String>,
    pub message: String,
    pub created_at: String,
    pub additions: i64,
    pub deletions: i64,
    pub author: UserSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardActivitySummary {
    pub total_events: i64,
    pub push_count: i64,
    pub pull_count: i64,
    pub commit_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryFileSummary {
    pub files: usize,
    pub directories: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryStorageSummary {
    pub bytes: usize,
    pub objects: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadmePreview {
    pub path: String,
    pub blob_hash: String,
    pub encoding: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoDashboardResponse {
    pub repo: Repository,
    pub branch_count: i64,
    pub commit_count: i64,
    pub latest_commit: Option<CommitSummary>,
    pub activity_summary: DashboardActivitySummary,
    pub file_summary: RepositoryFileSummary,
    pub starred_by_me: bool,
    pub readme_preview: Option<ReadmePreview>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitGraphNode {
    pub hash: String,
    pub parent_hashes: Vec<String>,
    pub message: String,
    pub created_at: String,
    pub author: UserSummary,
    pub branches: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitGraphResponse {
    pub repo_id: String,
    pub r#ref: String,
    pub head: String,
    pub nodes: Vec<CommitGraphNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentEntry {
    pub name: String,
    pub path: String,
    pub r#type: String,
    pub hash: String,
    pub mode: String,
    pub size: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentsResponse {
    pub repo_id: String,
    pub r#ref: String,
    pub path: String,
    pub tree_hash: Option<String>,
    pub items: Vec<ContentEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContentResponse {
    pub repo_id: String,
    pub r#ref: String,
    pub path: String,
    pub blob_hash: String,
    pub size: usize,
    pub encoding: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityFeedItem {
    pub r#type: String,
    pub action: String,
    pub actor: UserSummary,
    pub created_at: String,
    pub message: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsOverviewResponse {
    pub repo_id: String,
    pub branches_count: i64,
    pub commits_count: i64,
    pub stars_count: i64,
    pub push_count: i64,
    pub pull_count: i64,
    pub last_push_at: Option<String>,
    pub last_pull_at: Option<String>,
    pub contributors_count: i64,
    pub repository_size_bytes: usize,
    pub object_count: usize,
    pub branch_commit_distribution: Vec<BranchCommitDistributionItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BranchCommitDistributionItem {
    pub branch: String,
    pub total_count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VcsBranchAnalytics {
    pub id: Option<String>,
    pub name: String,
    pub last_commit_hash: Option<String>,
    pub created_at: String,
    pub last_activity_at: Option<String>,
    pub last_analyzed_at: Option<String>,
    pub is_default: bool,
    pub default_branch_name: String,
    pub head_commit_hash: Option<String>,
    pub default_head_hash: Option<String>,
    pub merge_base_hash: Option<String>,
    pub ahead_count: i64,
    pub behind_count: i64,
    pub divergence_distance: i64,
    pub freshness_status: Option<String>,
    pub freshness_score: Option<f64>,
    pub health_score: Option<f64>,
    pub stale_days: i32,
    pub computed_at: Option<String>,
    pub lane_index: Option<i32>,
    pub lane_color: Option<String>,
    pub start_commit_hash: Option<String>,
    pub first_seen_at: Option<String>,
    pub last_seen_at: Option<String>,
    pub commit_density: Option<f64>,
    pub activity_heat: Option<f64>,
    pub commit_count: i64,
    pub activity_score: f64,
    pub latest_commit_at: Option<String>,
    pub latest_contributor: Option<String>,
    pub latest_commit: Option<CommitGraphNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VcsTopologyCacheItem {
    pub branch_name: String,
    pub head_commit_hash: String,
    pub layout_version: Option<String>,
    pub nodes: serde_json::Value,
    pub edges: serde_json::Value,
    pub lanes: serde_json::Value,
    pub clusters: serde_json::Value,
    pub computed_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryDagMetricsResponse {
    pub commit_dag_complexity: f64,
    pub dag_complexity_status: String,
    pub longest_chain_nodes: i64,
    pub open_pr_count: i64,
    pub open_pr_delta_24h: i64,
    pub total_commits: i64,
    pub avg_divergence: f64,
    pub stale_ratio: f64,
    pub merge_velocity_per_week: f64,
    pub branch_count: i64,
    pub default_branch_name: Option<String>,
    pub computed_at: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VcsTimelineBucket {
    pub bucket_start: String,
    pub bucket_granularity: String,
    pub commit_count: i64,
    pub author_count: i64,
    pub branch_count: i64,
    pub additions: i64,
    pub deletions: i64,
    pub audit_event_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopModifiedFile {
    pub path: String,
    pub change_count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncMonitorLogItem {
    pub source: String,
    pub action: String,
    pub severity: String,
    pub message: String,
    pub branch_name: Option<String>,
    pub commit_hash: Option<String>,
    pub created_at: String,
    pub actor: Option<UserSummary>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncAnomalyItem {
    pub level: String,
    pub message: String,
    pub event_type: String,
    pub branch_name: Option<String>,
    pub commit_hash: Option<String>,
    pub created_at: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailurePropagationBucket {
    pub bucket_start: String,
    pub warn_count: i64,
    pub critical_count: i64,
    pub info_count: i64,
    pub total_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncActionCounts {
    pub push_count: i64,
    pub pull_count: i64,
    pub merge_count: i64,
    pub sync_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncMonitorResponse {
    pub repo_id: String,
    pub logs: Vec<SyncMonitorLogItem>,
    pub anomalies: Vec<SyncAnomalyItem>,
    pub failure_propagation: Vec<FailurePropagationBucket>,
    pub action_counts: SyncActionCounts,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VcsAnalyticsResponse {
    pub repo_id: String,
    pub default_branch: String,
    pub dag_metrics: Option<RepositoryDagMetricsResponse>,
    pub branches: Vec<VcsBranchAnalytics>,
    pub topology_cache: Vec<VcsTopologyCacheItem>,
    pub timeline: Vec<VcsTimelineBucket>,
    pub top_modified_files: Vec<TopModifiedFile>,
}

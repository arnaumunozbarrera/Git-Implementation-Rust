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
}

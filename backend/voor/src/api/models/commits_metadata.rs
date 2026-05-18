use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CommitMetadata {
    pub id: String,
    pub repo_id: Option<String>,
    pub commit_hash: String,
    pub author_id: String,
    pub message: String,
    pub additions: Option<i64>,
    pub deletions: Option<i64>,
    pub created_at: String,
}

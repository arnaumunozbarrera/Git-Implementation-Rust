use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Branch {
    pub id: String,
    pub repo_id: String,
    pub name: String,
    pub last_commit_hash: Option<String>,
    pub created_at: String,
}
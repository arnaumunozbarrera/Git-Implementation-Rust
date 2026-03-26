use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoAccessLog {
    pub id: String,
    pub repo_id: String,
    pub user_id: String,
    pub action: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
}
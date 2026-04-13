use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub is_private: bool,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub default_branch: String,
    pub stars_count: Option<i64>,
    pub readme_path: Option<String>,
    pub theme: Option<serde_json::Value>,
    pub created_at: String, // o chrono si quieres subir de nivel
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitRepoRequest {
    pub repo_id: String,
    pub name: String,
    pub owner_id: String,
    pub default_branch: String,
    pub is_private: bool,
    pub description: Option<String>,
    pub readme_path: Option<String>,
    pub tags: Option<Vec<String>>,
    pub theme: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitRepoResponse {
    pub message: String,
    pub repo_id: String,
}

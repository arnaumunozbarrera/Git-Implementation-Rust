use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
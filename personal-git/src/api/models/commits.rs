use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub hash: String,
    pub tree_hash: String,
    pub parent_hash: Option<String>,
    pub author_id: String,
    pub message: String,
    pub created_at: String,
}
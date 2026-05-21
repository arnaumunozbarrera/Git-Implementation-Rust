use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TreeEntry {
    pub id: String,
    pub tree_hash: String,
    pub name: String,
    pub r#type: String,
    pub hash: String,
    pub mode: String,
    pub created_at: String,
}

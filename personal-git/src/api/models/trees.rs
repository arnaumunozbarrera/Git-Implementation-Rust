use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tree {
    pub hash: String,
    pub created_at: String,
}
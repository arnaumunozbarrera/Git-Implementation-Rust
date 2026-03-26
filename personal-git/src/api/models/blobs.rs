use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Blob {
    pub hash: String,
    pub content: Vec<u8>,
    pub size: i64,
    pub created_at: String,
}
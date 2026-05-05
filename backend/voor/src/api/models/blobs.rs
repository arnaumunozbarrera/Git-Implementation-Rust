use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Blob {
    pub hash: String,
    pub content: Vec<u8>,
    pub size: i64,
    pub created_at: String,
}
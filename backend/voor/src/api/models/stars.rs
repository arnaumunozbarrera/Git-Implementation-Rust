use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Star {
    pub user_id: String,
    pub repo_id: String,
    pub created_at: String,
}

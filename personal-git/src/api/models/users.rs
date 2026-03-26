use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub created_at: String,
}
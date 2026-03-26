use axum::{extract::State, Json};
use crate::api::clients::supabase::SupabaseClient;
use crate::api::services::user_service;

pub async fn get_users(
    State(client): State<SupabaseClient>,
) -> Json<serde_json::Value> {

    let users = user_service::get_users(&client)
        .await
        .unwrap();

    Json(serde_json::json!(users))
}
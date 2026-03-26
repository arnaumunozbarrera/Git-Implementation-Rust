use axum::{extract::State, Json};
use crate::api::clients::supabase::SupabaseClient;
use crate::api::services::repo_service;

pub async fn get_repos(
    State(client): State<SupabaseClient>,
) -> Json<serde_json::Value> {

    let repos = repo_service::get_all_repos(&client)
        .await
        .unwrap();

    Json(serde_json::json!(repos))
}
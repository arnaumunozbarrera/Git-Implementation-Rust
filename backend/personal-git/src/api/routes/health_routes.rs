use axum::{extract::State, Json};
use crate::api::clients::supabase::SupabaseClient;

pub async fn get_health(
    State(client): State<SupabaseClient>,
) -> Json<serde_json::Value> {

    Json(serde_json::json!("Status: Alive"))
}
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use crate::api::clients::supabase::SupabaseClient;
use crate::api::models::Repository;
use crate::api::services::repo_service::get_all_repos;

pub async fn get_repos(
    State(client): State<SupabaseClient>,
) -> Result<Json<Vec<Repository>>, StatusCode> {
    match get_all_repos(&client).await {
        Ok(repos) => Ok(Json(repos)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
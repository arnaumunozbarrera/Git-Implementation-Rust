use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use crate::api::clients::supabase::SupabaseClient;
use crate::api::models::User;
use crate::api::services::user_service::get_all_users;

pub async fn get_users(
    State(client): State<SupabaseClient>,
) -> Result<Json<Vec<User>>, StatusCode> {
    match get_all_users(&client).await {
        Ok(users) => Ok(Json(users)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
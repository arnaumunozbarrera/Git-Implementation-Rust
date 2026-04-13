use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use crate::api::api::AppState;
use crate::api::models::User;
use crate::api::services::user_service::get_all_users;

pub async fn get_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, StatusCode> {
    let Some(client) = state.client.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    match get_all_users(client).await {
        Ok(users) => Ok(Json(users)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

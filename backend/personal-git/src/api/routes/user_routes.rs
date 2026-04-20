use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};

use crate::api::api::AppState;
use crate::api::auth::{self, AuthenticatedUser};
use crate::api::models::User;
use crate::api::services::user_service::get_all_users;

pub async fn get_users(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<User>>, StatusCode> {
    let Some(client) = state.client.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    if let Err(error) = auth::ensure_user_exists(Some(client), &user).await {
        println!("[WARN] {}", error);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    match get_all_users(client).await {
        Ok(users) => Ok(Json(users)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};

use crate::api::api::AppState;
use crate::api::auth::{self, AuthenticatedUser};
use crate::api::models::User;
use crate::api::services::user_service::get_all_users;
use crate::utils::service_monitor::LogLevel;

pub async fn get_users(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<User>>, StatusCode> {
    let Some(client) = state.client.as_ref() else {
        state.monitor.log(
            LogLevel::Warn,
            "backend",
            "users-unavailable",
            "User listing requested without configured database client",
        );
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    if let Err(error) = auth::ensure_user_exists(Some(client), &user).await {
        state.monitor.log(LogLevel::Warn, "backend", "user-sync-failed", &error);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    match get_all_users(client).await {
        Ok(users) => {
            state.monitor.log(
                LogLevel::Info,
                "backend",
                "users-listed",
                &format!("Fetched {} users for '{}'", users.len(), user.user_id),
            );
            Ok(Json(users))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

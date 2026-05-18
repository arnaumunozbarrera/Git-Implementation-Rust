use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};

use crate::api::api::AppState;
use crate::api::auth::{self, AuthenticatedUser};
use crate::api::models::{DeleteActionResponse, UpdateUserProfileRequest, User};
use crate::api::services::user_service::{
    delete_user_account as delete_user_account_service, get_all_users, update_user_profile,
};
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
        state
            .monitor
            .log(LogLevel::Warn, "backend", "user-sync-failed", &error);
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

pub async fn update_account_profile(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<UpdateUserProfileRequest>,
) -> Result<Json<User>, (StatusCode, String)> {
    let Some(client) = state.client.as_ref() else {
        let message = "[ERROR] Supabase client not configured".to_string();
        state.monitor.log(
            LogLevel::Warn,
            "backend",
            "profile-update-unavailable",
            &message,
        );
        return Err((StatusCode::SERVICE_UNAVAILABLE, message));
    };

    auth::ensure_user_exists(Some(client), &user)
        .await
        .map_err(|message| (StatusCode::INTERNAL_SERVER_ERROR, message))?;

    match update_user_profile(client, &user.user_id, payload.username.as_deref(), payload.email.as_deref()).await {
        Ok(updated_user) => {
            state.monitor.log(
                LogLevel::Info,
                "backend",
                "profile-updated",
                &format!("Updated profile for '{}'", user.user_id),
            );
            Ok(Json(updated_user))
        }
        Err(message) => {
            state
                .monitor
                .log(LogLevel::Warn, "backend", "profile-update-failed", &message);
            Err((classify_profile_error(&message), message))
        }
    }
}

pub async fn delete_account(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<DeleteActionResponse>, (StatusCode, String)> {
    let Some(client) = state.client.as_ref() else {
        let message = "[ERROR] Supabase client not configured".to_string();
        state.monitor.log(
            LogLevel::Warn,
            "backend",
            "delete-account-unavailable",
            &message,
        );
        return Err((StatusCode::SERVICE_UNAVAILABLE, message));
    };

    if let Err(error) = auth::ensure_user_exists(Some(client), &user).await {
        state
            .monitor
            .log(LogLevel::Warn, "backend", "user-sync-failed", &error);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, error));
    }

    match delete_user_account_service(client, &user.user_id).await {
        Ok(response) => {
            state.monitor.log(
                LogLevel::Info,
                "backend",
                "delete-account-finish",
                &response.message,
            );
            Ok(Json(response))
        }
        Err(message) => {
            state
                .monitor
                .log(LogLevel::Warn, "backend", "delete-account-failed", &message);
            Err((classify_delete_error(&message), message))
        }
    }
}

fn classify_profile_error(message: &str) -> StatusCode {
    if message.contains("not configured") {
        StatusCode::SERVICE_UNAVAILABLE
    } else if message.contains("Missing ") {
        StatusCode::BAD_REQUEST
    } else if message.contains("not found") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

fn classify_delete_error(message: &str) -> StatusCode {
    if message.contains("not configured") {
        StatusCode::SERVICE_UNAVAILABLE
    } else if message.contains("Missing ") {
        StatusCode::BAD_REQUEST
    } else if message.contains("not found") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

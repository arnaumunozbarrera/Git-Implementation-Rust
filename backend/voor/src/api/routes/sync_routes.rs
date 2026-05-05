use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};

use crate::api::api::AppState;
use crate::api::auth::{self, AuthenticatedUser};
use crate::api::services::sync_service;
use crate::utils::service_monitor::LogLevel;
use crate::utils::sync::{
    PullRequest, PullResponse, PushRequest, PushResponse, SyncDbRequest, SyncDbResponse,
};

pub async fn push_branch(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<PushRequest>,
) -> Result<Json<PushResponse>, (StatusCode, String)> {
    state.monitor.log(
        LogLevel::Info,
        "backend",
        "push-start",
        &format!("Push requested for repo '{}' branch '{}'", payload.repo_id, payload.branch),
    );

    auth::ensure_user_exists(state.client.as_ref(), &user)
        .await
        .map_err(|message| (StatusCode::INTERNAL_SERVER_ERROR, message))?;

    sync_service::push_branch(state.client.as_ref(), &user, payload)
        .await
        .map(|response| {
            state.monitor.log(LogLevel::Info, "backend", "push-finish", &response.message);
            Json(response)
        })
        .map_err(|message| {
            state.monitor.log(LogLevel::Warn, "backend", "push-failed", &message);
            classify_error(message)
        })
}

pub async fn pull_branch(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<PullRequest>,
) -> Result<Json<PullResponse>, (StatusCode, String)> {
    state.monitor.log(
        LogLevel::Info,
        "backend",
        "pull-start",
        &format!("Pull requested for repo '{}' branch '{}'", payload.repo_id, payload.branch),
    );

    auth::ensure_user_exists(state.client.as_ref(), &user)
        .await
        .map_err(|message| (StatusCode::INTERNAL_SERVER_ERROR, message))?;

    sync_service::pull_branch(state.client.as_ref(), &user, payload)
        .await
        .map(|response| {
            state.monitor.log(
                LogLevel::Info,
                "backend",
                "pull-finish",
                &format!("Pulled branch '{}' at {}", response.branch, response.head),
            );
            Json(response)
        })
        .map_err(|message| {
            state.monitor.log(LogLevel::Warn, "backend", "pull-failed", &message);
            classify_error(message)
        })
}

pub async fn sync_db(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<SyncDbRequest>,
) -> Result<Json<SyncDbResponse>, (StatusCode, String)> {
    state.monitor.log(
        LogLevel::Info,
        "backend",
        "sync-db-start",
        &format!("Database sync requested for repo '{}' branch '{}'", payload.repo_id, payload.branch),
    );

    auth::ensure_user_exists(state.client.as_ref(), &user)
        .await
        .map_err(|message| (StatusCode::INTERNAL_SERVER_ERROR, message))?;

    sync_service::sync_db(state.client.as_ref(), &user, payload)
        .await
        .map(|response| {
            state.monitor.log(LogLevel::Info, "backend", "sync-db-finish", &response.message);
            Json(response)
        })
        .map_err(|message| {
            state.monitor.log(LogLevel::Warn, "backend", "sync-db-failed", &message);
            classify_error(message)
        })
}

fn classify_error(message: String) -> (StatusCode, String) {
    let status = if message.contains("Missing branch") || message.contains("Missing object") {
        StatusCode::NOT_FOUND
    } else if message.contains("auth") || message.contains("JWT") || message.contains("Authorization") {
        StatusCode::UNAUTHORIZED
    } else if message.contains("not found") {
        StatusCode::NOT_FOUND
    } else if message.contains("Missing") || message.contains("Unknown repo") {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    (status, message)
}

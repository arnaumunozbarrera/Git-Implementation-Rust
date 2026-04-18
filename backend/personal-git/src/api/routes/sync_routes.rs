use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};

use crate::api::api::AppState;
use crate::api::auth::{self, AuthenticatedUser};
use crate::api::services::sync_service;
use crate::utils::sync::{
    PullRequest, PullResponse, PushRequest, PushResponse, SyncDbRequest, SyncDbResponse,
};

pub async fn push_branch(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<PushRequest>,
) -> Result<Json<PushResponse>, (StatusCode, String)> {
    auth::ensure_user_exists(state.client.as_ref(), &user)
        .await
        .map_err(|message| (StatusCode::INTERNAL_SERVER_ERROR, message))?;

    sync_service::push_branch(state.client.as_ref(), &user, payload)
        .await
        .map(Json)
        .map_err(classify_error)
}

pub async fn pull_branch(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<PullRequest>,
) -> Result<Json<PullResponse>, (StatusCode, String)> {
    auth::ensure_user_exists(state.client.as_ref(), &user)
        .await
        .map_err(|message| (StatusCode::INTERNAL_SERVER_ERROR, message))?;

    sync_service::pull_branch(state.client.as_ref(), &user, payload)
        .await
        .map(Json)
        .map_err(classify_error)
}

pub async fn sync_db(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<SyncDbRequest>,
) -> Result<Json<SyncDbResponse>, (StatusCode, String)> {
    auth::ensure_user_exists(state.client.as_ref(), &user)
        .await
        .map_err(|message| (StatusCode::INTERNAL_SERVER_ERROR, message))?;

    sync_service::sync_db(state.client.as_ref(), &user, payload)
        .await
        .map(Json)
        .map_err(classify_error)
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

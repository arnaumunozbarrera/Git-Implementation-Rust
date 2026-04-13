use axum::{
    extract::State,
    http::StatusCode,
    Json,
};

use crate::api::api::AppState;
use crate::api::services::sync_service;
use crate::utils::sync::{PullRequest, PullResponse, PushRequest, PushResponse};

pub async fn push_branch(
    State(state): State<AppState>,
    Json(payload): Json<PushRequest>,
) -> Result<Json<PushResponse>, (StatusCode, String)> {
    sync_service::push_branch(state.client.as_ref(), payload)
        .await
        .map(Json)
        .map_err(classify_error)
}

pub async fn pull_branch(
    State(state): State<AppState>,
    Json(payload): Json<PullRequest>,
) -> Result<Json<PullResponse>, (StatusCode, String)> {
    sync_service::pull_branch(state.client.as_ref(), payload)
        .await
        .map(Json)
        .map_err(classify_error)
}

fn classify_error(message: String) -> (StatusCode, String) {
    let status = if message.contains("Missing branch") || message.contains("Missing object") {
        StatusCode::NOT_FOUND
    } else if message.contains("repo") {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    (status, message)
}

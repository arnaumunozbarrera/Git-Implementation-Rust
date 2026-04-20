use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};

use crate::api::api::AppState;
use crate::api::auth::{self, AuthenticatedUser};
use crate::api::models::{InitRepoRequest, InitRepoResponse, Repository};
use crate::api::services::repo_service::{get_all_repos, init_repo as init_repo_service};

pub async fn get_repos(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<Repository>>, StatusCode> {
    let Some(client) = state.client.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    if let Err(error) = auth::ensure_user_exists(Some(client), &user).await {
        println!("[WARN] {}", error);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    match get_all_repos(client).await {
        Ok(repos) => Ok(Json(repos)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn init_repo(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<InitRepoRequest>,
) -> Result<Json<InitRepoResponse>, (StatusCode, String)> {
    let Some(client) = state.client.as_ref() else {
        let message = "[ERROR] Supabase client not configured".to_string();
        println!("[WARN] {}", message);
        return Err((StatusCode::SERVICE_UNAVAILABLE, message));
    };

    auth::ensure_user_exists(Some(client), &user)
        .await
        .map_err(|message| (StatusCode::INTERNAL_SERVER_ERROR, message))?;

    let repo_id = payload.repo_id.trim().to_string();
    println!(
        "[INFO] Initializing remote repository '{}' for authenticated user '{}'",
        repo_id, user.user_id
    );

    match init_repo_service(client, &user.user_id, payload).await {
        Ok(response) => {
            println!("[INFO] {}", response.message);
            Ok(Json(response))
        }
        Err(message) => {
            println!("[WARN] {}", message);
            Err((classify_init_error(&message), message))
        }
    }
}

fn classify_init_error(message: &str) -> StatusCode {
    if message.contains("not configured") {
        StatusCode::SERVICE_UNAVAILABLE
    } else if message.contains("Missing ") {
        StatusCode::BAD_REQUEST
    } else if message.contains("already exists") {
        StatusCode::CONFLICT
    } else if message.contains("not found") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

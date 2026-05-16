use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};

use crate::api::api::AppState;
use crate::api::auth::{self, AuthenticatedUser};
use crate::api::models::{DeleteActionResponse, InitRepoRequest, InitRepoResponse, Repository};
use crate::api::services::repo_service::{delete_repo as delete_repo_service, get_all_repos, init_repo as init_repo_service};
use crate::utils::service_monitor::LogLevel;

pub async fn get_repos(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<Repository>>, StatusCode> {
    let Some(client) = state.client.as_ref() else {
        state.monitor.log(
            LogLevel::Warn,
            "backend",
            "repos-unavailable",
            "Repository listing requested without configured database client",
        );
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    if let Err(error) = auth::ensure_user_exists(Some(client), &user).await {
        state.monitor.log(LogLevel::Warn, "backend", "user-sync-failed", &error);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    match get_all_repos(client).await {
        Ok(repos) => {
            state.monitor.log(
                LogLevel::Info,
                "backend",
                "repos-listed",
                &format!("Fetched {} repositories for '{}'", repos.len(), user.user_id),
            );
            Ok(Json(repos))
        }
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
        state.monitor.log(LogLevel::Warn, "backend", "init-repo-unavailable", &message);
        return Err((StatusCode::SERVICE_UNAVAILABLE, message));
    };

    auth::ensure_user_exists(Some(client), &user)
        .await
        .map_err(|message| (StatusCode::INTERNAL_SERVER_ERROR, message))?;

    let repo_id = payload.repo_id.trim().to_string();
    state.monitor.log(
        LogLevel::Info,
        "backend",
        "init-repo-start",
        &format!(
            "Initializing remote repository '{}' for '{}'",
            repo_id, user.user_id
        ),
    );

    match init_repo_service(client, &user.user_id, payload).await {
        Ok(response) => {
            state.monitor.log(LogLevel::Info, "backend", "init-repo-finish", &response.message);
            Ok(Json(response))
        }
        Err(message) => {
            state.monitor.log(LogLevel::Warn, "backend", "init-repo-failed", &message);
            Err((classify_init_error(&message), message))
        }
    }
}

pub async fn delete_repo(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(repo_id): Path<String>,
) -> Result<Json<DeleteActionResponse>, (StatusCode, String)> {
    let Some(client) = state.client.as_ref() else {
        let message = "[ERROR] Supabase client not configured".to_string();
        state.monitor.log(LogLevel::Warn, "backend", "delete-repo-unavailable", &message);
        return Err((StatusCode::SERVICE_UNAVAILABLE, message));
    };

    auth::ensure_user_exists(Some(client), &user)
        .await
        .map_err(|message| (StatusCode::INTERNAL_SERVER_ERROR, message))?;

    match delete_repo_service(client, &user.user_id, &repo_id).await {
        Ok(response) => {
            state.monitor.log(LogLevel::Info, "backend", "delete-repo-finish", &response.message);
            Ok(Json(response))
        }
        Err(message) => {
            state.monitor.log(LogLevel::Warn, "backend", "delete-repo-failed", &message);
            Err((classify_delete_error(&message), message))
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

fn classify_delete_error(message: &str) -> StatusCode {
    if message.contains("not configured") {
        StatusCode::SERVICE_UNAVAILABLE
    } else if message.contains("Missing ") {
        StatusCode::BAD_REQUEST
    } else if message.contains("not found") {
        StatusCode::NOT_FOUND
    } else if message.contains("cannot delete") {
        StatusCode::FORBIDDEN
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

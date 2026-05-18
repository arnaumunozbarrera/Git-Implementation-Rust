use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
};

use crate::api::api::AppState;
use crate::api::auth::{self, AuthenticatedUser};
use crate::api::models::{
    Branch, CloneRepoRequest, CloneRepoResponse, DeleteActionResponse, InitRepoRequest,
    InitRepoResponse, Repository,
};
use crate::api::services::repo_service::{
    clone_repo_to_desktop as clone_repo_to_desktop_service, delete_repo as delete_repo_service,
    get_all_repos, get_repo_branches,
    init_repo as init_repo_service,
};
use crate::utils::service_monitor::LogLevel;

pub async fn get_repos(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<Repository>>, (StatusCode, String)> {
    let Some(client) = state.client.as_ref() else {
        state.monitor.log(
            LogLevel::Warn,
            "backend",
            "repos-unavailable",
            "Repository listing requested without configured database client",
        );
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "[ERROR] Supabase client not configured".to_string(),
        ));
    };

    if let Err(error) = auth::ensure_user_exists(Some(client), &user).await {
        state
            .monitor
            .log(LogLevel::Warn, "backend", "user-sync-failed", &error);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, error));
    }

    match get_all_repos(client, &user.user_id).await {
        Ok(repos) => {
            state.monitor.log(
                LogLevel::Info,
                "backend",
                "repos-listed",
                &format!(
                    "Fetched {} repositories for '{}'",
                    repos.len(),
                    user.user_id
                ),
            );
            Ok(Json(repos))
        }
        Err(error) => {
            let message = format!("[ERROR] Failed to list repositories: {}", error);
            state
                .monitor
                .log(LogLevel::Error, "backend", "repos-list-failed", &message);
            Err((StatusCode::INTERNAL_SERVER_ERROR, message))
        }
    }
}

pub async fn get_branches(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(repo_id): Path<String>,
) -> Result<Json<Vec<Branch>>, (StatusCode, String)> {
    let Some(client) = state.client.as_ref() else {
        let message = "[ERROR] Supabase client not configured".to_string();
        state
            .monitor
            .log(LogLevel::Warn, "backend", "branches-unavailable", &message);
        return Err((StatusCode::SERVICE_UNAVAILABLE, message));
    };

    auth::ensure_user_exists(Some(client), &user)
        .await
        .map_err(|message| (StatusCode::INTERNAL_SERVER_ERROR, message))?;

    match get_repo_branches(client, &user.user_id, &repo_id).await {
        Ok(branches) => {
            state.monitor.log(
                LogLevel::Info,
                "backend",
                "branches-listed",
                &format!("Fetched {} branches for '{}'", branches.len(), repo_id),
            );
            Ok(Json(branches))
        }
        Err(message) => {
            state
                .monitor
                .log(LogLevel::Warn, "backend", "branches-list-failed", &message);
            Err((classify_branch_error(&message), message))
        }
    }
}

pub async fn init_repo(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<InitRepoRequest>,
) -> Result<Json<InitRepoResponse>, (StatusCode, String)> {
    let Some(client) = state.client.as_ref() else {
        let message = "[ERROR] Supabase client not configured".to_string();
        state
            .monitor
            .log(LogLevel::Warn, "backend", "init-repo-unavailable", &message);
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
            state.monitor.log(
                LogLevel::Info,
                "backend",
                "init-repo-finish",
                &response.message,
            );
            Ok(Json(response))
        }
        Err(message) => {
            state
                .monitor
                .log(LogLevel::Warn, "backend", "init-repo-failed", &message);
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
        state.monitor.log(
            LogLevel::Warn,
            "backend",
            "delete-repo-unavailable",
            &message,
        );
        return Err((StatusCode::SERVICE_UNAVAILABLE, message));
    };

    auth::ensure_user_exists(Some(client), &user)
        .await
        .map_err(|message| (StatusCode::INTERNAL_SERVER_ERROR, message))?;

    match delete_repo_service(client, &user.user_id, &repo_id).await {
        Ok(response) => {
            state.monitor.log(
                LogLevel::Info,
                "backend",
                "delete-repo-finish",
                &response.message,
            );
            Ok(Json(response))
        }
        Err(message) => {
            state
                .monitor
                .log(LogLevel::Warn, "backend", "delete-repo-failed", &message);
            Err((classify_delete_error(&message), message))
        }
    }
}

pub async fn clone_repo_to_desktop(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(repo_id): Path<String>,
    Json(payload): Json<CloneRepoRequest>,
) -> Result<Json<CloneRepoResponse>, (StatusCode, String)> {
    let Some(client) = state.client.as_ref() else {
        let message = "[ERROR] Supabase client not configured".to_string();
        state
            .monitor
            .log(LogLevel::Warn, "backend", "clone-repo-unavailable", &message);
        return Err((StatusCode::SERVICE_UNAVAILABLE, message));
    };

    auth::ensure_user_exists(Some(client), &user)
        .await
        .map_err(|message| (StatusCode::INTERNAL_SERVER_ERROR, message))?;

    match clone_repo_to_desktop_service(client, &user.user_id, &repo_id, payload.default_branch.as_deref()).await {
        Ok(response) => {
            state.monitor.log(
                LogLevel::Info,
                "backend",
                "clone-repo-finish",
                &response.message,
            );
            Ok(Json(response))
        }
        Err(message) => {
            state
                .monitor
                .log(LogLevel::Warn, "backend", "clone-repo-failed", &message);
            Err((classify_clone_error(&message), message))
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

fn classify_branch_error(message: &str) -> StatusCode {
    if message.contains("not configured") {
        StatusCode::SERVICE_UNAVAILABLE
    } else if message.contains("Missing ") {
        StatusCode::BAD_REQUEST
    } else if message.contains("not found") {
        StatusCode::NOT_FOUND
    } else if message.contains("cannot access") {
        StatusCode::FORBIDDEN
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

fn classify_clone_error(message: &str) -> StatusCode {
    if message.contains("not configured") {
        StatusCode::SERVICE_UNAVAILABLE
    } else if message.contains("Missing ") {
        StatusCode::BAD_REQUEST
    } else if message.contains("not found") {
        StatusCode::NOT_FOUND
    } else if message.contains("cannot clone") {
        StatusCode::FORBIDDEN
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

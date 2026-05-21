use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
};

use crate::api::api::AppState;
use crate::api::auth::{self, AuthenticatedUser};
use crate::api::models::{
    ActivityFeedQuery, AnalyticsOverviewResponse, CommitGraphQuery, CommitGraphResponse,
    CommitHistoryQuery, CommitSummary, ContentsResponse, FileContentResponse, PaginationResponse,
    RepoDashboardResponse, RepoPathQuery, VcsAnalyticsResponse,
};
use crate::api::services::frontend_service;
use crate::utils::service_monitor::LogLevel;

pub async fn get_repo_dashboard(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(repo_id): Path<String>,
) -> Result<Json<RepoDashboardResponse>, (StatusCode, String)> {
    let client = require_client(&state)?;
    ensure_authenticated_user(&state, client, &user).await?;

    frontend_service::get_repo_dashboard(client, &repo_id, &user.user_id)
        .await
        .map(|response| {
            state.monitor.log(
                LogLevel::Info,
                "backend",
                "repo-dashboard",
                &format!("Loaded dashboard for repo '{}'", repo_id),
            );
            Json(response)
        })
        .map_err(classify_error)
}

pub async fn get_commit_history(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(repo_id): Path<String>,
    Query(query): Query<CommitHistoryQuery>,
) -> Result<Json<PaginationResponse<CommitSummary>>, (StatusCode, String)> {
    let client = require_client(&state)?;
    ensure_authenticated_user(&state, client, &user).await?;

    frontend_service::get_commit_history(
        client,
        &repo_id,
        query.ref_name.as_deref(),
        frontend_service::normalize_limit(query.limit),
        query.offset.unwrap_or(0),
    )
    .await
    .map(Json)
    .map_err(classify_error)
}

pub async fn get_commit_graph(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(repo_id): Path<String>,
    Query(query): Query<CommitGraphQuery>,
) -> Result<Json<CommitGraphResponse>, (StatusCode, String)> {
    let client = require_client(&state)?;
    ensure_authenticated_user(&state, client, &user).await?;

    frontend_service::get_commit_graph(
        client,
        &repo_id,
        query.ref_name.as_deref(),
        frontend_service::normalize_limit(query.limit),
    )
    .await
    .map(Json)
    .map_err(classify_error)
}

pub async fn get_repo_contents(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(repo_id): Path<String>,
    Query(query): Query<RepoPathQuery>,
) -> Result<Json<ContentsResponse>, (StatusCode, String)> {
    let client = require_client(&state)?;
    ensure_authenticated_user(&state, client, &user).await?;

    frontend_service::get_repo_contents(
        client,
        &repo_id,
        query.ref_name.as_deref(),
        query.path.as_deref(),
    )
    .await
    .map(Json)
    .map_err(classify_error)
}

pub async fn get_repo_file(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(repo_id): Path<String>,
    Query(query): Query<RepoPathQuery>,
) -> Result<Json<FileContentResponse>, (StatusCode, String)> {
    let client = require_client(&state)?;
    ensure_authenticated_user(&state, client, &user).await?;

    let Some(path) = query.path.as_deref() else {
        return Err((StatusCode::BAD_REQUEST, "[ERROR] Missing file path".to_string()));
    };

    frontend_service::get_repo_file(client, &repo_id, query.ref_name.as_deref(), path)
        .await
        .map(Json)
        .map_err(classify_error)
}

pub async fn get_activity_feed(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(repo_id): Path<String>,
    Query(query): Query<ActivityFeedQuery>,
) -> Result<Json<PaginationResponse<crate::api::models::ActivityFeedItem>>, (StatusCode, String)> {
    let client = require_client(&state)?;
    ensure_authenticated_user(&state, client, &user).await?;

    frontend_service::get_activity_feed(
        client,
        &repo_id,
        query.action.as_deref(),
        frontend_service::normalize_limit(query.limit),
        query.offset.unwrap_or(0),
    )
    .await
    .map(Json)
    .map_err(classify_error)
}

pub async fn get_analytics_overview(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(repo_id): Path<String>,
) -> Result<Json<AnalyticsOverviewResponse>, (StatusCode, String)> {
    let client = require_client(&state)?;
    ensure_authenticated_user(&state, client, &user).await?;

    frontend_service::get_analytics_overview(client, &repo_id)
        .await
        .map(Json)
        .map_err(classify_error)
}

pub async fn get_vcs_analytics(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(repo_id): Path<String>,
) -> Result<Json<VcsAnalyticsResponse>, (StatusCode, String)> {
    let client = require_client(&state)?;
    ensure_authenticated_user(&state, client, &user).await?;

    frontend_service::get_vcs_analytics(client, &repo_id)
        .await
        .map(Json)
        .map_err(classify_error)
}

fn require_client(state: &AppState) -> Result<&crate::api::clients::supabase::SupabaseClient, (StatusCode, String)> {
    state.client.as_ref().ok_or_else(|| {
        let message = "[ERROR] Supabase client not configured".to_string();
        state.monitor.log(LogLevel::Warn, "backend", "frontend-routes-unavailable", &message);
        (StatusCode::SERVICE_UNAVAILABLE, message)
    })
}

async fn ensure_authenticated_user(
    state: &AppState,
    client: &crate::api::clients::supabase::SupabaseClient,
    user: &AuthenticatedUser,
) -> Result<(), (StatusCode, String)> {
    auth::ensure_user_exists(Some(client), user)
        .await
        .map_err(|message| {
            state.monitor.log(LogLevel::Warn, "backend", "user-sync-failed", &message);
            (StatusCode::INTERNAL_SERVER_ERROR, message)
        })
}

fn classify_error(message: String) -> (StatusCode, String) {
    let status = if message.contains("not configured") {
        StatusCode::SERVICE_UNAVAILABLE
    } else if message.contains("not found") || message.contains("Unknown repo") {
        StatusCode::NOT_FOUND
    } else if message.contains("Missing") || message.contains("is not a") {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    (status, message)
}

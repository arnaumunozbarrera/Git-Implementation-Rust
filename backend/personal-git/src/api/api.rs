use axum::{
    extract::Request,
    middleware,
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use std::time::Instant;

use crate::api::auth::{self, AuthConfig};
use crate::api::clients::supabase::SupabaseClient;
use crate::api::routes::health_routes::get_health;
use crate::api::routes::repo_routes::{get_repos, init_repo};
use crate::api::routes::sync_routes::{pull_branch, push_branch, sync_db};
use crate::api::routes::user_routes::get_users;
use crate::utils::service_monitor::{LogLevel, ServiceMonitor};

#[derive(Clone)]
pub struct AppState {
    pub client: Option<SupabaseClient>,
    pub auth: Option<AuthConfig>,
    pub monitor: ServiceMonitor,
}

pub async fn api() {
    dotenv().ok();
    let monitor = ServiceMonitor::new();
    monitor.register_service("backend", "healthy", "booting", "Backend bootstrap started");
    monitor.register_service(
        "frontend",
        "warning",
        "not_running",
        "Frontend runtime not attached in this process; monitor via API edge",
    );
    monitor.register_service("api", "healthy", "starting", "API startup checks running");

    let client = if env::var("SUPABASE_URL").is_ok() {
        let client = SupabaseClient::new().await;
        match client.healthcheck().await {
            Ok(_) => {
                monitor.update_service("backend", "healthy", "ready", "Database connection OK");
                Some(client)
            }
            Err(error) => {
                monitor.update_service(
                    "backend",
                    "warning",
                    "degraded",
                    &format!("Database healthcheck failed: {}", error),
                );
                None
            }
        }
    } else {
        monitor.update_service(
            "backend",
            "warning",
            "degraded",
            "SUPABASE_URL not set, starting without database-backed routes",
        );
        None
    };

    let auth = match auth::AuthConfig::from_env().await {
        Ok(Some(config)) => {
            monitor.log(LogLevel::Info, "api", "auth-config", "Clerk auth configured");
            Some(config)
        }
        Ok(None) => {
            monitor.log(
                LogLevel::Warn,
                "api",
                "auth-config",
                "Clerk auth not configured; protected routes will be unavailable",
            );
            None
        }
        Err(error) => {
            monitor.log(
                LogLevel::Warn,
                "api",
                "auth-config",
                &format!("Failed to configure Clerk auth: {}", error),
            );
            None
        }
    };

    let state = AppState {
        client,
        auth,
        monitor: monitor.clone(),
    };
    let protected = Router::new()
        .route("/repos", get(get_repos))
        .route("/repos/init", post(init_repo))
        .route("/users", get(get_users))
        .route("/push", post(push_branch))
        .route("/pull", post(pull_branch))
        .route("/sync-db", post(sync_db))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth::require_auth));

    let app = Router::new()
        .route("/health", get(get_health))
        .merge(protected)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            track_api_requests,
        ))
        .with_state(state);

    let port = env::var("PORT").unwrap_or("3000".to_string());
    let addr = SocketAddr::from(([127, 0, 0, 1], port.parse().unwrap()));
    monitor.update_service("api", "healthy", "running", &format!("Server listening on {}", addr));
    monitor.update_service(
        "frontend",
        "healthy",
        "reachable-through-api",
        "Frontend activity can be monitored through HTTP traffic and health report",
    );

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    monitor.log(LogLevel::Info, "backend", "service-ready", "All service monitors attached");
    axum::serve(listener, app).await.unwrap();
}

async fn track_api_requests(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request,
    next: middleware::Next,
) -> axum::response::Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let started = Instant::now();

    state.monitor.log(
        LogLevel::Info,
        "api",
        "request-start",
        &format!("{} {}", method, path),
    );

    let response = next.run(request).await;
    let status = response.status();
    let elapsed_ms = started.elapsed().as_millis();
    let level = if status.is_server_error() {
        LogLevel::Error
    } else if status.is_client_error() {
        LogLevel::Warn
    } else {
        LogLevel::Info
    };

    state.monitor.log(
        level,
        "api",
        "request-finish",
        &format!("{} {} -> {} ({} ms)", method, path, status.as_u16(), elapsed_ms),
    );

    response
}

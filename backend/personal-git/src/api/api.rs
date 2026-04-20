use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;

use crate::api::auth::{self, AuthConfig};
use crate::api::clients::supabase::SupabaseClient;
use crate::api::routes::health_routes::get_health;
use crate::api::routes::repo_routes::{get_repos, init_repo};
use crate::api::routes::sync_routes::{pull_branch, push_branch, sync_db};
use crate::api::routes::user_routes::get_users;

#[derive(Clone)]
pub struct AppState {
    pub client: Option<SupabaseClient>,
    pub auth: Option<AuthConfig>,
}

pub async fn api() {
    dotenv().ok();

    let client = if env::var("SUPABASE_URL").is_ok() {
        let client = SupabaseClient::new().await;
        match client.healthcheck().await {
            Ok(_) => {
                println!("[INFO] Database connection OK");
                Some(client)
            }
            Err(error) => {
                println!("[WARN] Database healthcheck failed: {}", error);
                None
            }
        }
    } else {
        println!("[WARN] SUPABASE_URL not set, starting without database-backed routes");
        None
    };

    let auth = match auth::AuthConfig::from_env().await {
        Ok(Some(config)) => {
            println!("[INFO] Clerk auth configured");
            Some(config)
        }
        Ok(None) => {
            println!("[WARN] Clerk auth not configured; protected routes will be unavailable");
            None
        }
        Err(error) => {
            println!("[WARN] Failed to configure Clerk auth: {}", error);
            None
        }
    };

    let state = AppState { client, auth };
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
        .with_state(state);

    let port = env::var("PORT").unwrap_or("3000".to_string());
    let addr = SocketAddr::from(([127, 0, 0, 1], port.parse().unwrap()));
    println!("[INFO] Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

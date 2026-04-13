use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use dotenvy::dotenv;
use std::env;

use crate::api::clients::supabase::SupabaseClient;
use crate::api::routes::repo_routes::{get_repos, init_repo};
use crate::api::routes::sync_routes::{pull_branch, push_branch};
use crate::api::routes::user_routes::get_users;
use crate::api::routes::health_routes::get_health;

#[derive(Clone)]
pub struct AppState {
    pub client: Option<SupabaseClient>,
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

    let app = Router::new()
        .route("/health", get(get_health))
        .route("/repos", get(get_repos))
        .route("/repos/init", post(init_repo))
        .route("/users", get(get_users))
        .route("/push", post(push_branch))
        .route("/pull", post(pull_branch))
        .with_state(AppState { client });

    let port = env::var("PORT").unwrap_or("3000".to_string());

    let addr = SocketAddr::from(([127, 0, 0, 1], port.parse().unwrap()));
    println!("[INFO] Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

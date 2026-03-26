use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use dotenvy::dotenv;
use std::env;

use crate::api::clients::supabase::SupabaseClient;
use crate::api::routes::repo_routes::get_repos;
use crate::api::routes::user_routes::get_users;
use crate::api::routes::health_routes::get_health;

pub async fn api() {
    dotenv().ok();

    let client = SupabaseClient::new().await;

    client
        .healthcheck()
        .await
        .expect("Database healthcheck failed");

    println!("[INFO] Database connection OK");

    let app = Router::new()
        .route("/health", get(get_health))
        .route("/repos", get(get_repos))
        .route("/users", get(get_users))
        .with_state(client);

    let port = env::var("PORT").unwrap_or("3000".to_string());

    let addr = SocketAddr::from(([127, 0, 0, 1], port.parse().unwrap()));
    println!("[INFO] Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
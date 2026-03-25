use axum::{
    routing::{get, post, delete},
    Router,
    extract::{State, Path},
    Json,
};
use std::net::SocketAddr;
use dotenvy::dotenv;
use std::env;

mod supabase;
use supabase::SupabaseClient;

mod models;
use models::Repository;

#[tokio::api]
async fn api() {
    dotenv().ok();

    let client = SupabaseClient::new();

    let app = Router::new()
        .route("/repos", get(get_repos).post(create_repo))
        .route("/repos/:id", delete(delete_repo))
        .with_state(client);

    let port = env::var("PORT").unwrap_or("3000".into());

    let addr = SocketAddr::from(([127, 0, 0, 1], port.parse().unwrap()));
    println!("Server running on {}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
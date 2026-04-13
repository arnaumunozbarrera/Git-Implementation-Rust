use axum::Json;

pub async fn get_health() -> Json<serde_json::Value> {
    Json(serde_json::json!("Status: Alive"))
}

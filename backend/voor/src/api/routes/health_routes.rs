use axum::{extract::State, Json};

use crate::api::api::AppState;

pub async fn get_health(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!(state.monitor.health_report()))
}

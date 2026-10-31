//! Endpoint kiểm tra sức khỏe của máy chủ.

use axum::Json;
use axum::response::IntoResponse;
use serde_json::json;

/// Handler cho `GET /api/v1/health`.
pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "dandori-server"
    }))
}

//! Middleware kiểm tra Origin header ngăn chặn tấn công CSRF / Cross-Origin trái phép (RS-SEC-006).

use crate::error::ApiError;
use crate::middleware::request_id::RequestId;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

/// Kiểm tra tiêu đề `Origin` nếu có trong yêu cầu.
pub async fn check_origin(req: Request, next: Next) -> Response {
    let req_id = req
        .extensions()
        .get::<RequestId>()
        .map_or_else(|| uuid::Uuid::new_v4().to_string(), |r| r.0.clone());

    if let Some(origin_header) = req.headers().get("origin").and_then(|v| v.to_str().ok())
        && !is_origin_allowed(origin_header)
    {
        return ApiError::ForbiddenOrigin.into_response_with_id(&req_id);
    }

    next.run(req).await
}

fn is_origin_allowed(origin: &str) -> bool {
    let stripped = origin
        .trim_start_matches("http://")
        .trim_start_matches("https://");

    let host = stripped.split(':').next().unwrap_or(stripped);

    host == "localhost" || host == "127.0.0.1" || host == "[::1]" || host == "::1"
}

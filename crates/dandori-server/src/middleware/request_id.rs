//! Middleware gắn request_id cho mỗi yêu cầu HTTP.

use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

/// Wrapper chứa request_id để truyền qua extension.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

/// Gắn mã request_id vào mỗi yêu cầu và thêm tiêu đề phản hồi `x-request-id`.
pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    let req_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map_or_else(|| uuid::Uuid::new_v4().to_string(), ToString::to_string);

    req.extensions_mut().insert(RequestId(req_id.clone()));

    let mut response = next.run(req).await;
    if let Ok(val) = HeaderValue::from_str(&req_id) {
        response.headers_mut().insert("x-request-id", val);
    }

    response
}

//! Middleware kiểm tra Origin header ngăn chặn tấn công CSRF / Cross-Origin trái phép (RS-SEC-006).

use crate::error::ApiError;
use crate::middleware::request_id::RequestId;
use crate::state::AppState;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use sqlx::SqlitePool;

/// Kiểm tra tiêu đề `Origin` nếu có trong yêu cầu (hỗ trợ loopback mặc định và danh sách mở rộng).
pub async fn check_origin(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let req_id = req
        .extensions()
        .get::<RequestId>()
        .map_or_else(|| uuid::Uuid::new_v4().to_string(), |r| r.0.clone());

    if let Some(origin_header) = req.headers().get("origin").and_then(|v| v.to_str().ok())
        && !is_origin_allowed(origin_header)
        && !is_origin_allowed_by_mobile_config(&state.db, origin_header).await
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

/// Origin có nằm trong danh sách cho phép do admin cấu hình ở module `mobile` không (đối chiếu theo ranh giới
/// tên miền, mục cấu hình không hợp lệ như `*` hay `*.com` bị bỏ qua).
async fn is_origin_allowed_by_mobile_config(pool: &SqlitePool, origin: &str) -> bool {
    let rows: Result<Vec<(String,)>, _> =
        sqlx::query_as("SELECT allowed_origins FROM m_mobile_config")
            .fetch_all(pool)
            .await;

    let Ok(configs) = rows else {
        return false;
    };

    let host = dandori_module_api::origin_host(origin);

    for (allowed_json,) in configs {
        if let Ok(origins) = serde_json::from_str::<Vec<String>>(&allowed_json) {
            for allowed in origins {
                if dandori_module_api::origin_host_matches(&host, &allowed) {
                    return true;
                }
            }
        }
    }

    false
}

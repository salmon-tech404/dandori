//! Trích xuất thông tin người dùng đã xác thực từ cookie phiên (AuthUser extractor).

use crate::auth::hash_token;
use crate::error::ApiError;
use crate::middleware::request_id::RequestId;
use crate::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use chrono::{Duration, Utc};
use dandori_db::UserRepo;
use dandori_types::UserId;
use sqlx::Row;

/// Người dùng đã xác thực thành công từ phiên đăng nhập.
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// ID người dùng.
    pub id: UserId,
    /// Tên đăng nhập.
    pub username: String,
    /// Vai trò hệ thống (`admin` hoặc `user`).
    pub system_role: String,
}

impl AuthUser {
    /// Kiểm tra người dùng có vai trò admin hay không.
    #[must_use]
    pub fn is_admin(&self) -> bool {
        self.system_role == "admin"
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = axum::response::Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let req_id = parts
            .extensions
            .get::<RequestId>()
            .map_or_else(|| uuid::Uuid::new_v4().to_string(), |r| r.0.clone());

        let raw_token = extract_session_cookie(parts)
            .ok_or_else(|| ApiError::Unauthorized.into_response_with_id(&req_id))?;

        let now_str = Utc::now().to_rfc3339();

        let session_row = sqlx::query(
            "SELECT id, user_id, expires_at FROM session WHERE token_hash = ? AND expires_at > ?",
        )
        .bind(hash_token(&raw_token))
        .bind(&now_str)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()).into_response_with_id(&req_id))?
        .ok_or_else(|| ApiError::Unauthorized.into_response_with_id(&req_id))?;

        let session_id: String = session_row
            .try_get("id")
            .map_err(|_| ApiError::Unauthorized.into_response_with_id(&req_id))?;
        let user_id_str: String = session_row
            .try_get("user_id")
            .map_err(|_| ApiError::Unauthorized.into_response_with_id(&req_id))?;

        let user_id = user_id_str
            .parse::<UserId>()
            .map_err(|_| ApiError::Unauthorized.into_response_with_id(&req_id))?;

        // Gia hạn phiên làm việc thêm 8 giờ khi có hoạt động
        let new_expires = Utc::now() + Duration::hours(8);
        let new_expires_str = new_expires.to_rfc3339();
        let _ = sqlx::query("UPDATE session SET expires_at = ? WHERE id = ?")
            .bind(new_expires_str)
            .bind(&session_id)
            .execute(&state.db)
            .await;

        let user_repo = UserRepo::new(&state.db);
        let user = user_repo
            .get_user_by_id(&user_id)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()).into_response_with_id(&req_id))?
            .ok_or_else(|| ApiError::Unauthorized.into_response_with_id(&req_id))?;

        Ok(AuthUser {
            id: user.id,
            username: user.username,
            system_role: user.system_role,
        })
    }
}

fn extract_session_cookie(parts: &Parts) -> Option<String> {
    let cookie_header = parts.headers.get("cookie")?.to_str().ok()?;
    for pair in cookie_header.split(';') {
        let mut parts = pair.trim().splitn(2, '=');
        let key = parts.next()?.trim();
        let val = parts.next()?.trim();
        if key == "dandori_session" && !val.is_empty() {
            return Some(val.to_string());
        }
    }
    None
}

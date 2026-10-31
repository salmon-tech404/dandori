//! Các route xử lý xác thực và quản lý tài khoản người dùng (T10).

use crate::auth::{
    check_lockout, create_session_for_user, dummy_verify, hash_password, record_failed_attempt,
    reset_failed_attempts, verify_password,
};
use crate::error::ApiError;
use crate::middleware::{AuthUser, RequestId};
use crate::state::AppState;
use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use chrono::Utc;
use dandori_db::UserRepo;
use dandori_types::{User, UserId};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Tham số yêu cầu khởi tạo chủ hệ thống lần đầu.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetupOwnerRequest {
    /// Tên đăng nhập.
    pub username: String,
    /// Mật khẩu.
    pub password: String,
}

/// Tham số yêu cầu đăng nhập.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    /// Tên đăng nhập.
    pub username: String,
    /// Mật khẩu.
    pub password: String,
}

/// Tham số yêu cầu tạo người dùng mới bởi admin.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateUserRequest {
    /// Tên đăng nhập người dùng mới.
    pub username: String,
    /// Mật khẩu.
    pub password: String,
    /// Vai trò hệ thống (`admin` hoặc `user`).
    pub system_role: Option<String>,
}

/// Dữ liệu người dùng trả về cho client.
#[derive(Debug, Serialize)]
pub struct UserResponse {
    /// ID người dùng.
    pub id: String,
    /// Tên đăng nhập.
    pub username: String,
    /// Vai trò hệ thống.
    pub system_role: String,
    /// Thời điểm khởi tạo.
    pub created_at: String,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id.to_string(),
            username: u.username,
            system_role: u.system_role,
            created_at: u.created_at.to_rfc3339(),
        }
    }
}

fn attach_cookie(headers: &mut HeaderMap, cookie_str: &str) {
    if let Ok(val) = HeaderValue::from_str(cookie_str) {
        headers.insert(SET_COOKIE, val);
    }
}

/// Handler cho `POST /api/v1/auth/setup-owner`.
pub async fn setup_owner(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    Json(payload): Json<SetupOwnerRequest>,
) -> Response {
    let username = payload.username.trim();
    if username.is_empty() || payload.password.len() < 6 {
        return ApiError::BadRequest(
            "Tên đăng nhập không được rỗng và mật khẩu tối thiểu 6 ký tự".to_string(),
        )
        .into_response_with_id(&req_id.0);
    }

    let count_res = sqlx::query("SELECT COUNT(*) as count FROM user")
        .fetch_one(&state.db)
        .await;

    let count_row = match count_res {
        Ok(r) => r,
        Err(e) => return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    };

    let count: i64 = sqlx::Row::try_get(&count_row, "count").unwrap_or(0);
    if count > 0 {
        return ApiError::OwnerAlreadyExists.into_response_with_id(&req_id.0);
    }

    let password_hash = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(e) => return ApiError::Internal(e).into_response_with_id(&req_id.0),
    };

    let user_id = UserId::new();
    let new_user = User {
        id: user_id,
        username: username.to_string(),
        password_hash,
        system_role: "admin".to_string(),
        created_at: Utc::now(),
    };

    let user_repo = UserRepo::new(&state.db);
    if let Err(e) = user_repo.create_user(&new_user).await {
        return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0);
    }

    let (token, cookie_header) = match create_session_for_user(&state.db, user_id).await {
        Ok(res) => res,
        Err(e) => return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    };

    let mut headers = HeaderMap::new();
    attach_cookie(&mut headers, &cookie_header);

    (
        headers,
        Json(json!({ "user": UserResponse::from(new_user), "token": token })),
    )
        .into_response()
}

async fn authenticate_user(
    state: &AppState,
    username: &str,
    password: &str,
) -> Result<User, ApiError> {
    if let Some(locked_time) = check_lockout(&state.db, username)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        return Err(ApiError::AccountLocked {
            until: locked_time.to_rfc3339(),
        });
    }

    let user_repo = UserRepo::new(&state.db);
    let user_opt = user_repo
        .get_user_by_username(username)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let user = match user_opt {
        Some(u) => u,
        None => {
            dummy_verify(password);
            let lock_res = record_failed_attempt(&state.db, username)
                .await
                .map_err(|e| ApiError::Internal(e.to_string()))?;
            if let Some(until) = lock_res {
                return Err(ApiError::AccountLocked {
                    until: until.to_rfc3339(),
                });
            }
            return Err(ApiError::InvalidCredentials);
        }
    };

    if !verify_password(password, &user.password_hash) {
        let lock_res = record_failed_attempt(&state.db, username)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        if let Some(until) = lock_res {
            return Err(ApiError::AccountLocked {
                until: until.to_rfc3339(),
            });
        }
        return Err(ApiError::InvalidCredentials);
    }

    let _ = reset_failed_attempts(&state.db, username).await;
    Ok(user)
}

/// Handler cho `POST /api/v1/auth/login`.
pub async fn login(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    Json(payload): Json<LoginRequest>,
) -> Response {
    let username = payload.username.trim();
    let user = match authenticate_user(&state, username, &payload.password).await {
        Ok(u) => u,
        Err(err) => return err.into_response_with_id(&req_id.0),
    };

    let (token, cookie_header) = match create_session_for_user(&state.db, user.id).await {
        Ok(res) => res,
        Err(e) => return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    };

    let mut headers = HeaderMap::new();
    attach_cookie(&mut headers, &cookie_header);

    (
        headers,
        Json(json!({ "user": UserResponse::from(user), "token": token })),
    )
        .into_response()
}

/// Handler cho `POST /api/v1/auth/logout`.
pub async fn logout(State(state): State<AppState>, auth_user: AuthUser) -> Response {
    let user_id_str = auth_user.id.to_string();
    let _ = sqlx::query("DELETE FROM session WHERE user_id = ?")
        .bind(&user_id_str)
        .execute(&state.db)
        .await;

    let mut headers = HeaderMap::new();
    attach_cookie(
        &mut headers,
        "dandori_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0",
    );

    (headers, Json(json!({ "status": "logged_out" }))).into_response()
}

/// Handler cho `GET /api/v1/me`.
pub async fn get_current_user(auth_user: AuthUser) -> impl IntoResponse {
    Json(json!({
        "id": auth_user.id.to_string(),
        "username": auth_user.username,
        "system_role": auth_user.system_role
    }))
}

/// Handler cho `POST /api/v1/auth/users` (chủ hệ thống tạo người dùng mới).
pub async fn create_user(
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    auth_user: AuthUser,
    Json(payload): Json<CreateUserRequest>,
) -> Response {
    if !auth_user.is_admin() {
        return ApiError::Forbidden(
            "Chỉ tài khoản admin mới có quyền tạo người dùng mới".to_string(),
        )
        .into_response_with_id(&req_id.0);
    }

    let username = payload.username.trim();
    if username.is_empty() || payload.password.len() < 6 {
        return ApiError::BadRequest(
            "Tên đăng nhập không được rỗng và mật khẩu tối thiểu 6 ký tự".to_string(),
        )
        .into_response_with_id(&req_id.0);
    }

    let password_hash = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(e) => return ApiError::Internal(e).into_response_with_id(&req_id.0),
    };

    let role = payload.system_role.unwrap_or_else(|| "user".to_string());
    if role != "admin" && role != "user" {
        return ApiError::BadRequest("Vai trò hệ thống phải là admin hoặc user".to_string())
            .into_response_with_id(&req_id.0);
    }

    let new_user = User {
        id: UserId::new(),
        username: username.to_string(),
        password_hash,
        system_role: role,
        created_at: Utc::now(),
    };

    let user_repo = UserRepo::new(&state.db);
    if let Err(e) = user_repo.create_user(&new_user).await {
        return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0);
    }

    Json(json!({ "user": UserResponse::from(new_user) })).into_response()
}

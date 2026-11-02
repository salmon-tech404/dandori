//! Xác thực phiên và kiểm tra quyền truy cập workspace cho route của module.
//!
//! Cổng route của registry chỉ kiểm tra "có phiên hợp lệ"; việc người đó có được xem/sửa dữ liệu của
//! một workspace cụ thể là của từng module, dùng chung các hàm này (không tự chép lại ở mỗi module).

use axum::http::{HeaderMap, header};
use sqlx::SqlitePool;

/// Người dùng đã xác thực qua cookie phiên.
#[derive(Debug, Clone)]
pub struct SessionUser {
    /// ID người dùng.
    pub user_id: String,
    /// Vai trò hệ thống (`admin` hoặc `user`).
    pub system_role: String,
}

impl SessionUser {
    /// Quản trị hệ thống thấy mọi workspace.
    #[must_use]
    pub fn is_system_admin(&self) -> bool {
        self.system_role == "admin" || self.system_role == "owner"
    }
}

fn session_cookie(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    cookie_header.split(';').find_map(|pair| {
        let (key, val) = pair.trim().split_once('=')?;
        (key.trim() == "dandori_session" && !val.trim().is_empty()).then(|| val.trim().to_string())
    })
}

/// Tra người dùng từ cookie phiên (còn hạn). `None` nếu thiếu cookie, phiên sai hoặc hết hạn.
pub async fn session_user(pool: &SqlitePool, headers: &HeaderMap) -> Option<SessionUser> {
    let token = session_cookie(headers)?;
    let token_hash = dandori_db::hash_session_token(&token);
    let now = chrono::Utc::now().to_rfc3339();
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT u.id, u.system_role FROM session s JOIN user u ON u.id = s.user_id \
         WHERE s.token_hash = ? AND s.expires_at > ?",
    )
    .bind(token_hash)
    .bind(now)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    row.map(|(user_id, system_role)| SessionUser {
        user_id,
        system_role,
    })
}

/// Người dùng có được truy cập workspace này không (quản trị hệ thống hoặc thành viên).
pub async fn can_access_workspace(
    pool: &SqlitePool,
    user: &SessionUser,
    workspace_id: &str,
) -> bool {
    if user.is_system_admin() {
        return true;
    }
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM workspace_member WHERE workspace_id = ? AND user_id = ?",
    )
    .bind(workspace_id)
    .bind(&user.user_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0)
        > 0
}

/// Người dùng có quyền **quản trị** workspace không (quản trị hệ thống hoặc thành viên có vai trò `admin`).
/// Dùng cho thao tác nhạy cảm như thêm khóa/kết nối ra dịch vụ ngoài.
pub async fn can_administer_workspace(
    pool: &SqlitePool,
    user: &SessionUser,
    workspace_id: &str,
) -> bool {
    if user.is_system_admin() {
        return true;
    }
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM workspace_member WHERE workspace_id = ? AND user_id = ? AND role = 'admin'",
    )
    .bind(workspace_id)
    .bind(&user.user_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0)
        > 0
}

/// Workspace chứa dự án `project_id` (None nếu dự án không tồn tại).
pub async fn project_workspace(pool: &SqlitePool, project_id: &str) -> Option<String> {
    sqlx::query_scalar("SELECT workspace_id FROM project WHERE id = ?")
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

/// Workspace chứa task `task_id` (None nếu task không tồn tại).
pub async fn task_workspace(pool: &SqlitePool, task_id: &str) -> Option<String> {
    sqlx::query_scalar(
        "SELECT p.workspace_id FROM task t JOIN project p ON p.id = t.project_id WHERE t.id = ?",
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

/// Danh sách workspace người dùng được xem; `None` nghĩa là tất cả (quản trị hệ thống).
pub async fn accessible_workspace_ids(
    pool: &SqlitePool,
    user: &SessionUser,
) -> Option<Vec<String>> {
    if user.is_system_admin() {
        return None;
    }
    Some(
        sqlx::query_scalar("SELECT workspace_id FROM workspace_member WHERE user_id = ?")
            .bind(&user.user_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default(),
    )
}

//! Các REST API routes cho module `shared-memory` (P3-10).

use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use chrono::Utc;
use dandori_module_api::{
    ModuleContext, SessionUser, can_access_workspace, can_administer_workspace, session_user,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::model::{
    BatchUpdateStatusPayload, CreateMemoryPayload, MemoryEntry, ProposeMemoryPayload,
    UpdateMemoryStatusPayload, is_valid_category, is_valid_status,
};
use crate::redaction::{contains_secrets, redact_secrets};

#[derive(Clone, Debug)]
pub struct SharedMemoryState {
    pub ctx: ModuleContext,
}

impl SharedMemoryState {
    #[must_use]
    pub fn new(ctx: ModuleContext) -> Self {
        Self { ctx }
    }
}

fn err_resp(status: StatusCode, msg: &str) -> Response {
    (status, Json(json!({ "error": { "message": msg } }))).into_response()
}

fn get_pool(state: &SharedMemoryState) -> Result<SqlitePool, (StatusCode, &'static str)> {
    state.ctx.pool().cloned().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Không có kết nối cơ sở dữ liệu",
    ))
}

async fn get_user(
    headers: &HeaderMap,
    pool: &SqlitePool,
) -> Result<SessionUser, (StatusCode, &'static str)> {
    session_user(pool, headers).await.ok_or((
        StatusCode::UNAUTHORIZED,
        "Yêu cầu đăng nhập để truy cập tài nguyên",
    ))
}

#[derive(Debug, Deserialize)]
pub struct ListMemoryParams {
    pub project_id: Option<String>,
    pub status: Option<String>,
    pub category: Option<String>,
}

pub fn shared_memory_router(state: SharedMemoryState) -> Router {
    Router::new()
        .route(
            "/workspaces/{ws_id}/memories",
            get(list_memories_handler).post(create_memory_handler),
        )
        .route(
            "/workspaces/{ws_id}/memories/batch-status",
            post(batch_update_status_handler),
        )
        .route(
            "/workspaces/{ws_id}/memories/{memory_id}/status",
            patch(update_memory_status_handler),
        )
        .route(
            "/workspaces/{ws_id}/memories/{memory_id}",
            delete(delete_memory_handler),
        )
        .route("/workspaces/{ws_id}/propose", post(propose_memory_handler))
        .with_state(state)
}

/// `project_id` (nếu có) phải thuộc đúng workspace; tránh gắn mục nhớ vào dự án của workspace khác.
async fn project_in_workspace(pool: &SqlitePool, project_id: Option<&str>, ws_id: &str) -> bool {
    match project_id {
        None => true,
        Some(pid) => dandori_module_api::project_workspace(pool, pid)
            .await
            .is_some_and(|w| w == ws_id),
    }
}

async fn list_memories_handler(
    State(state): State<SharedMemoryState>,
    AxumPath(ws_id): AxumPath<String>,
    Query(params): Query<ListMemoryParams>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((c, m)) => return err_resp(c, m),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((c, m)) => return err_resp(c, m),
    };
    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    let rows = sqlx::query_as::<_, MemoryEntry>(
        r#"
        SELECT id, workspace_id, project_id, category, content, source_type, source_ref,
               status, reviewed_by, reviewed_at, created_at, updated_at
        FROM m_shared_memory_entry
        WHERE workspace_id = ?
          AND (? IS NULL OR project_id = ?)
          AND (? IS NULL OR status = ?)
          AND (? IS NULL OR category = ?)
        ORDER BY created_at DESC, id ASC
        "#,
    )
    .bind(&ws_id)
    .bind(&params.project_id)
    .bind(&params.project_id)
    .bind(&params.status)
    .bind(&params.status)
    .bind(&params.category)
    .bind(&params.category)
    .fetch_all(&pool)
    .await;

    match rows {
        Ok(items) => Json(items).into_response(),
        Err(e) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "Handler tạo tri thức bao gồm xác thực quyền và ghi CSDL"
)]
async fn create_memory_handler(
    State(state): State<SharedMemoryState>,
    AxumPath(ws_id): AxumPath<String>,
    headers: HeaderMap,
    Json(payload): Json<CreateMemoryPayload>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((c, m)) => return err_resp(c, m),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((c, m)) => return err_resp(c, m),
    };
    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    let cat = payload.category.trim().to_lowercase();
    if !is_valid_category(&cat) {
        return err_resp(StatusCode::BAD_REQUEST, "Loại mục nhớ không hợp lệ");
    }

    let raw_content = payload.content.trim();
    if raw_content.is_empty() || raw_content.len() > 2000 {
        return err_resp(
            StatusCode::BAD_REQUEST,
            "Nội dung rỗng hoặc vượt quá 2000 ký tự",
        );
    }

    if !project_in_workspace(&pool, payload.project_id.as_deref(), &ws_id).await {
        return err_resp(StatusCode::BAD_REQUEST, "Dự án không thuộc workspace này");
    }

    let content = redact_secrets(raw_content);
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let source_ref = format!("user:{}", user.user_id);
    // Chống nhiễm độc bộ nhớ: chỉ quản trị viên mới tạo mục được duyệt sẵn; thành viên thường vào hàng chờ duyệt.
    let is_admin = can_administer_workspace(&pool, &user, &ws_id).await;
    let (init_status, reviewer, reviewed_at) = if is_admin {
        ("approved", Some(user.user_id.clone()), Some(now.clone()))
    } else {
        ("pending", None, None)
    };

    let query = r#"
        INSERT INTO m_shared_memory_entry (
            id, workspace_id, project_id, category, content, source_type, source_ref,
            status, reviewed_by, reviewed_at, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'human', ?, ?, ?, ?, ?, ?)
        RETURNING id, workspace_id, project_id, category, content, source_type, source_ref,
                  status, reviewed_by, reviewed_at, created_at, updated_at
    "#;

    match sqlx::query_as::<_, MemoryEntry>(query)
        .bind(id)
        .bind(&ws_id)
        .bind(&payload.project_id)
        .bind(&cat)
        .bind(content)
        .bind(source_ref)
        .bind(init_status)
        .bind(reviewer)
        .bind(reviewed_at)
        .bind(&now)
        .bind(&now)
        .fetch_one(&pool)
        .await
    {
        Ok(saved) => Json(saved).into_response(),
        Err(e) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn update_memory_status_handler(
    State(state): State<SharedMemoryState>,
    AxumPath((ws_id, memory_id)): AxumPath<(String, String)>,
    headers: HeaderMap,
    Json(payload): Json<UpdateMemoryStatusPayload>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((c, m)) => return err_resp(c, m),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((c, m)) => return err_resp(c, m),
    };
    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Chỉ quản trị viên workspace mới được duyệt hoặc từ chối mục nhớ",
        );
    }

    let status = payload.status.trim().to_lowercase();
    if !is_valid_status(&status) || status == "pending" {
        return err_resp(
            StatusCode::BAD_REQUEST,
            "Trạng thái duyệt phải là approved hoặc rejected",
        );
    }

    let now = Utc::now().to_rfc3339();
    let query = r#"
        UPDATE m_shared_memory_entry
        SET status = ?, reviewed_by = ?, reviewed_at = ?, updated_at = ?
        WHERE id = ? AND workspace_id = ?
        RETURNING id, workspace_id, project_id, category, content, source_type, source_ref,
                  status, reviewed_by, reviewed_at, created_at, updated_at
    "#;

    match sqlx::query_as::<_, MemoryEntry>(query)
        .bind(&status)
        .bind(&user.user_id)
        .bind(&now)
        .bind(&now)
        .bind(&memory_id)
        .bind(&ws_id)
        .fetch_one(&pool)
        .await
    {
        Ok(updated) => Json(updated).into_response(),
        Err(sqlx::Error::RowNotFound) => err_resp(StatusCode::NOT_FOUND, "Không tìm thấy mục nhớ"),
        Err(e) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn batch_update_status_handler(
    State(state): State<SharedMemoryState>,
    AxumPath(ws_id): AxumPath<String>,
    headers: HeaderMap,
    Json(payload): Json<BatchUpdateStatusPayload>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((c, m)) => return err_resp(c, m),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((c, m)) => return err_resp(c, m),
    };
    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Chỉ quản trị viên workspace mới được duyệt hoặc từ chối mục nhớ",
        );
    }

    let status = payload.status.trim().to_lowercase();
    if !is_valid_status(&status) || status == "pending" {
        return err_resp(
            StatusCode::BAD_REQUEST,
            "Trạng thái phải là approved hoặc rejected",
        );
    }

    let now = Utc::now().to_rfc3339();
    let mut updated_count = 0;
    for mid in &payload.memory_ids {
        let res = sqlx::query(
            r#"
            UPDATE m_shared_memory_entry
            SET status = ?, reviewed_by = ?, reviewed_at = ?, updated_at = ?
            WHERE id = ? AND workspace_id = ?
            "#,
        )
        .bind(&status)
        .bind(&user.user_id)
        .bind(&now)
        .bind(&now)
        .bind(mid)
        .bind(&ws_id)
        .execute(&pool)
        .await;

        if let Ok(r) = res {
            updated_count += r.rows_affected();
        }
    }

    Json(json!({ "updated": updated_count, "updated_count": updated_count })).into_response()
}

async fn delete_memory_handler(
    State(state): State<SharedMemoryState>,
    AxumPath((ws_id, memory_id)): AxumPath<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((c, m)) => return err_resp(c, m),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((c, m)) => return err_resp(c, m),
    };
    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Chỉ quản trị viên mới được xóa mục nhớ",
        );
    }

    let res = sqlx::query("DELETE FROM m_shared_memory_entry WHERE id = ? AND workspace_id = ?")
        .bind(&memory_id)
        .bind(&ws_id)
        .execute(&pool)
        .await;

    match res {
        Ok(r) if r.rows_affected() > 0 => Json(json!({ "deleted": true })).into_response(),
        Ok(_) => err_resp(StatusCode::NOT_FOUND, "Không tìm thấy mục nhớ để xóa"),
        Err(e) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// Kiểm tra đề xuất bộ nhớ từ agent: độ dài, dự án thuộc workspace, không chứa bí mật.
async fn proposal_error(
    pool: &SqlitePool,
    payload: &ProposeMemoryPayload,
    ws_id: &str,
) -> Option<&'static str> {
    if payload.content.trim().is_empty()
        || payload.content.trim().len() > 2000
        || payload.source_ref.len() > 200
    {
        return Some("Nội dung rỗng hoặc vượt quá 2000 ký tự, hoặc nguồn quá dài");
    }
    if !project_in_workspace(pool, payload.project_id.as_deref(), ws_id).await {
        return Some("Dự án không thuộc workspace này");
    }
    if contains_secrets(&payload.content) {
        return Some("Nội dung chứa khóa bí mật hoặc token, từ chối lưu vào bộ nhớ chung");
    }
    None
}

async fn propose_memory_handler(
    State(state): State<SharedMemoryState>,
    AxumPath(ws_id): AxumPath<String>,
    headers: HeaderMap,
    Json(payload): Json<ProposeMemoryPayload>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((c, m)) => return err_resp(c, m),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((c, m)) => return err_resp(c, m),
    };
    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    let cat = payload.category.trim().to_lowercase();
    if !is_valid_category(&cat) {
        return err_resp(StatusCode::BAD_REQUEST, "Loại mục nhớ không hợp lệ");
    }

    if let Some(msg) = proposal_error(&pool, &payload, &ws_id).await {
        return err_resp(StatusCode::BAD_REQUEST, msg);
    }

    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    let query = r#"
        INSERT INTO m_shared_memory_entry (
            id, workspace_id, project_id, category, content, source_type, source_ref,
            status, reviewed_by, reviewed_at, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'agent', ?, 'pending', NULL, NULL, ?, ?)
        RETURNING id, workspace_id, project_id, category, content, source_type, source_ref,
                  status, reviewed_by, reviewed_at, created_at, updated_at
    "#;

    match sqlx::query_as::<_, MemoryEntry>(query)
        .bind(id)
        .bind(&ws_id)
        .bind(&payload.project_id)
        .bind(&cat)
        .bind(payload.content.trim())
        .bind(payload.source_ref.trim())
        .bind(&now)
        .bind(&now)
        .fetch_one(&pool)
        .await
    {
        Ok(saved) => Json(saved).into_response(),
        Err(e) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

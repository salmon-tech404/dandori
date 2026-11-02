//! Định nghĩa các route REST API kiểm toán (P3-14).

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::Utc;
use dandori_module_api::access::{SessionUser, can_administer_workspace, session_user};
use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::hash_chain::{
    HashPayload, compute_entry_hash, redact_sensitive_strings, verify_entries,
};
use crate::model::{AppendLogRequest, AuditLogEntry, GENESIS_HASH, ListLogsQuery};

/// Trạng thái chia sẻ của module audit-log.
#[derive(Clone, Debug)]
pub struct AuditLogState {
    /// Pool kết nối cơ sở dữ liệu.
    pub pool: SqlitePool,
}

fn err_resp(code: StatusCode, msg: &str) -> Response {
    (code, Json(json!({ "error": { "message": msg } }))).into_response()
}

/// Tạo Router cho module audit-log (chỉ có route đọc và ghi thêm; tuyệt đối không có update/delete).
pub fn router(state: AuditLogState) -> Router {
    let shared = Arc::new(state);
    Router::new()
        .route("/workspaces/{ws}/logs", get(list_logs).post(append_log))
        .route("/workspaces/{ws}/verify", get(verify_chain))
        .route("/workspaces/{ws}/export", get(export_logs))
        .with_state(shared)
}

/// Nhật ký kiểm toán chứa danh tính và hành động nhạy cảm: chỉ quản trị viên workspace được đọc/ghi.
async fn check_ws_access(
    pool: &SqlitePool,
    headers: &HeaderMap,
    ws: &str,
) -> Result<SessionUser, (StatusCode, &'static str)> {
    let Some(user) = session_user(pool, headers).await else {
        return Err((StatusCode::UNAUTHORIZED, "Phiên không hợp lệ"));
    };
    if !can_administer_workspace(pool, &user, ws).await {
        return Err((
            StatusCode::FORBIDDEN,
            "Yêu cầu quyền quản trị workspace để xem nhật ký kiểm toán",
        ));
    }
    Ok(user)
}

async fn list_logs(
    State(state): State<Arc<AuditLogState>>,
    Path(ws): Path<String>,
    headers: HeaderMap,
    Query(query): Query<ListLogsQuery>,
) -> Response {
    let pool = &state.pool;
    if let Err((c, m)) = check_ws_access(pool, &headers, &ws).await {
        return err_resp(c, m);
    }

    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let offset = query.offset.unwrap_or(0).max(0);

    let rows: Result<Vec<AuditLogEntry>, _> = if let Some(act) = &query.action {
        sqlx::query_as(
            "SELECT * FROM m_audit_log WHERE workspace_id = ? AND action = ? ORDER BY sequence DESC LIMIT ? OFFSET ?",
        )
        .bind(&ws)
        .bind(act)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as(
            "SELECT * FROM m_audit_log WHERE workspace_id = ? ORDER BY sequence DESC LIMIT ? OFFSET ?",
        )
        .bind(&ws)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    };

    match rows {
        Ok(list) => Json(list).into_response(),
        Err(e) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn fetch_next_chain_link(pool: &SqlitePool, ws: &str) -> Result<(i64, String), StatusCode> {
    let last: Option<(i64, String)> = sqlx::query_as(
        "SELECT sequence, hash FROM m_audit_log WHERE workspace_id = ? ORDER BY sequence DESC LIMIT 1",
    )
    .bind(ws)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(match last {
        Some((seq, h)) => (seq + 1, h),
        None => (1, GENESIS_HASH.to_string()),
    })
}

async fn persist_audit_entry(pool: &SqlitePool, entry: &AuditLogEntry) -> Result<(), StatusCode> {
    sqlx::query(
        r#"
        INSERT INTO m_audit_log (
            id, workspace_id, sequence, actor_id, actor_name, action,
            target_type, target_id, details, source, prev_hash, hash, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&entry.id)
    .bind(&entry.workspace_id)
    .bind(entry.sequence)
    .bind(&entry.actor_id)
    .bind(&entry.actor_name)
    .bind(&entry.action)
    .bind(&entry.target_type)
    .bind(&entry.target_id)
    .bind(&entry.details)
    .bind(&entry.source)
    .bind(&entry.prev_hash)
    .bind(&entry.hash)
    .bind(&entry.created_at)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

/// Khóa tuần tự hóa việc nối chuỗi băm trong một tiến trình (tránh hai bản ghi cùng `sequence`).
static CHAIN_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Ghi một bản ghi vào chuỗi kiểm toán của workspace (dùng chung cho API và tiến trình sự kiện).
pub(crate) async fn insert_audit_entry(
    pool: &SqlitePool,
    ws: &str,
    req: &AppendLogRequest,
) -> Result<AuditLogEntry, StatusCode> {
    let _guard = CHAIN_LOCK.lock().await;
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let (sequence, prev_hash) = fetch_next_chain_link(pool, ws).await?;

    let details_str = redact_sensitive_strings(&req.details.to_string());
    let payload = HashPayload {
        prev_hash: &prev_hash,
        sequence,
        workspace_id: ws,
        actor_id: &req.actor_id,
        action: &req.action,
        target_type: &req.target_type,
        target_id: &req.target_id,
        created_at: &now,
        canonical_details: &details_str,
    };
    let hash = compute_entry_hash(&payload);

    let entry = AuditLogEntry {
        id,
        workspace_id: ws.to_string(),
        sequence,
        actor_id: req.actor_id.clone(),
        actor_name: req.actor_name.clone(),
        action: req.action.clone(),
        target_type: req.target_type.clone(),
        target_id: req.target_id.clone(),
        details: details_str,
        source: req.source.clone(),
        prev_hash,
        hash,
        created_at: now,
    };

    persist_audit_entry(pool, &entry).await?;
    Ok(entry)
}

async fn append_log(
    State(state): State<Arc<AuditLogState>>,
    Path(ws): Path<String>,
    headers: HeaderMap,
    Json(mut payload): Json<AppendLogRequest>,
) -> Response {
    let pool = &state.pool;
    let user = match check_ws_access(pool, &headers, &ws).await {
        Ok(u) => u,
        Err((c, m)) => return err_resp(c, m),
    };

    if payload.action.trim().is_empty()
        || payload.action.len() > 64
        || payload.target_type.len() > 64
        || payload.target_id.len() > 200
        || payload.details.to_string().len() > 4096
    {
        return err_resp(StatusCode::BAD_REQUEST, "Trường nhật ký rỗng hoặc quá dài");
    }
    // Danh tính người ghi luôn lấy từ phiên đăng nhập; không tin actor/source do client gửi (chống giả mạo).
    payload.actor_id = user.user_id.clone();
    payload.actor_name = user.user_id;
    payload.source = "web".to_string();

    match insert_audit_entry(pool, &ws, &payload).await {
        Ok(entry) => (StatusCode::CREATED, Json(entry)).into_response(),
        Err(code) => err_resp(code, "Lỗi ghi bản ghi kiểm toán"),
    }
}

async fn verify_chain(
    State(state): State<Arc<AuditLogState>>,
    Path(ws): Path<String>,
    headers: HeaderMap,
) -> Response {
    let pool = &state.pool;
    if let Err((c, m)) = check_ws_access(pool, &headers, &ws).await {
        return err_resp(c, m);
    }

    let rows: Result<Vec<AuditLogEntry>, _> =
        sqlx::query_as("SELECT * FROM m_audit_log WHERE workspace_id = ? ORDER BY sequence ASC")
            .bind(&ws)
            .fetch_all(pool)
            .await;

    match rows {
        Ok(entries) => Json(verify_entries(&entries)).into_response(),
        Err(e) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn export_logs(
    State(state): State<Arc<AuditLogState>>,
    Path(ws): Path<String>,
    headers: HeaderMap,
) -> Response {
    let pool = &state.pool;
    if let Err((c, m)) = check_ws_access(pool, &headers, &ws).await {
        return err_resp(c, m);
    }

    let rows: Result<Vec<AuditLogEntry>, _> =
        sqlx::query_as("SELECT * FROM m_audit_log WHERE workspace_id = ? ORDER BY sequence ASC")
            .bind(&ws)
            .fetch_all(pool)
            .await;

    let entries = match rows {
        Ok(e) => e,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let sanitized_entries: Vec<AuditLogEntry> = entries
        .into_iter()
        .map(|mut e| {
            e.details = redact_sensitive_strings(&e.details);
            e
        })
        .collect();

    Json(json!({
        "workspace_id": ws,
        "exported_at": Utc::now().to_rfc3339(),
        "total_records": sanitized_entries.len(),
        "records": sanitized_entries
    }))
    .into_response()
}

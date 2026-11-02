//! Định nghĩa các route REST API quản lý kênh thông báo (P3-12).

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::Utc;
use dandori_module_api::access::{can_access_workspace, can_administer_workspace, session_user};
use dandori_module_api::safe_http::sanitize_error_message;
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use crate::model::{
    ChannelType, CreateChannelRequest, NotificationChannel, NotificationHistoryRecord,
    PublicNotificationChannel, UpdateChannelRequest,
};
use crate::sender::dispatch_notification;
use crate::vault::{delete_secret, reveal_secret, store_secret};

/// Trạng thái chia sẻ của router module notifications.
#[derive(Clone, Debug)]
pub struct NotificationsState {
    /// Pool cơ sở dữ liệu module.
    pub pool: SqlitePool,
}

/// Tạo Router cho module notifications.
pub fn router(state: NotificationsState) -> Router {
    Router::new()
        .route(
            "/workspaces/{ws}/notifications/channels",
            get(list_channels).post(create_channel),
        )
        .route(
            "/workspaces/{ws}/notifications/channels/{id}",
            get(get_channel).put(update_channel).delete(delete_channel),
        )
        .route(
            "/workspaces/{ws}/notifications/channels/{id}/test",
            axum::routing::post(send_test_notification),
        )
        .route(
            "/workspaces/{ws}/notifications/history",
            get(get_notification_history),
        )
        .with_state(Arc::new(state))
}

fn err_resp(status: StatusCode, msg: &str) -> Response {
    (status, Json(json!({ "error": msg }))).into_response()
}

async fn list_channels(
    State(state): State<Arc<NotificationsState>>,
    Path(ws): Path<String>,
    headers: HeaderMap,
) -> Response {
    let pool = &state.pool;
    let Some(user) = session_user(pool, &headers).await else {
        return err_resp(StatusCode::UNAUTHORIZED, "Chưa đăng nhập");
    };

    if !can_access_workspace(pool, &user, &ws).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    let channels: Vec<NotificationChannel> = match sqlx::query_as(
        r#"
        SELECT id, workspace_id, name, channel_type, vault_secret_id, chat_id,
               is_enabled, notify_on_needs_human, notify_on_failed,
               notify_on_approval_requested, notify_on_limit_warning,
               created_at, updated_at
        FROM m_notifications_channel
        WHERE workspace_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(&ws)
    .fetch_all(pool)
    .await
    {
        Ok(c) => c,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let public: Vec<PublicNotificationChannel> = channels.into_iter().map(Into::into).collect();
    Json(public).into_response()
}

async fn get_channel(
    State(state): State<Arc<NotificationsState>>,
    Path((ws, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let pool = &state.pool;
    let Some(user) = session_user(pool, &headers).await else {
        return err_resp(StatusCode::UNAUTHORIZED, "Chưa đăng nhập");
    };

    if !can_access_workspace(pool, &user, &ws).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    let channel: Option<NotificationChannel> = match sqlx::query_as(
        "SELECT id, workspace_id, name, channel_type, vault_secret_id, chat_id, \
         is_enabled, notify_on_needs_human, notify_on_failed, notify_on_approval_requested, notify_on_limit_warning, \
         created_at, updated_at FROM m_notifications_channel WHERE id = ? AND workspace_id = ?",
    )
    .bind(&id)
    .bind(&ws)
    .fetch_optional(pool)
    .await
    {
        Ok(c) => c,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    match channel {
        Some(c) => {
            let pub_c: PublicNotificationChannel = c.into();
            Json(pub_c).into_response()
        }
        None => err_resp(StatusCode::NOT_FOUND, "Không tìm thấy kênh thông báo"),
    }
}

async fn insert_channel_db(pool: &SqlitePool, channel: &NotificationChannel) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO m_notifications_channel (
            id, workspace_id, name, channel_type, vault_secret_id, chat_id,
            is_enabled, notify_on_needs_human, notify_on_failed,
            notify_on_approval_requested, notify_on_limit_warning,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&channel.id)
    .bind(&channel.workspace_id)
    .bind(&channel.name)
    .bind(&channel.channel_type)
    .bind(&channel.vault_secret_id)
    .bind(&channel.chat_id)
    .bind(channel.is_enabled)
    .bind(channel.notify_on_needs_human)
    .bind(channel.notify_on_failed)
    .bind(channel.notify_on_approval_requested)
    .bind(channel.notify_on_limit_warning)
    .bind(&channel.created_at)
    .bind(&channel.updated_at)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

async fn create_channel(
    State(state): State<Arc<NotificationsState>>,
    Path(ws): Path<String>,
    headers: HeaderMap,
    Json(req): Json<CreateChannelRequest>,
) -> Response {
    let pool = &state.pool;
    let Some(user) = session_user(pool, &headers).await else {
        return err_resp(StatusCode::UNAUTHORIZED, "Chưa đăng nhập");
    };
    if !can_administer_workspace(pool, &user, &ws).await {
        return err_resp(StatusCode::FORBIDDEN, "Yêu cầu quyền admin workspace");
    }
    if ChannelType::from_str_opt(&req.channel_type).is_none() {
        return err_resp(StatusCode::BAD_REQUEST, "Loại kênh không được hỗ trợ");
    }
    if req.secret.trim().is_empty() {
        return err_resp(StatusCode::BAD_REQUEST, "Bí mật kênh không được rỗng");
    }

    let label = format!("notification:{}:{}", req.channel_type, req.name.trim());
    let vault_ref = match store_secret(pool, &label, req.secret.trim()).await {
        Ok(sid) => sid,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e),
    };

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let channel = NotificationChannel {
        id,
        workspace_id: ws,
        name: req.name.trim().to_string(),
        channel_type: req.channel_type,
        vault_secret_id: vault_ref,
        chat_id: req.chat_id.map(|s| s.trim().to_string()),
        is_enabled: true,
        notify_on_needs_human: req.notify_on_needs_human.unwrap_or(true),
        notify_on_failed: req.notify_on_failed.unwrap_or(true),
        notify_on_approval_requested: req.notify_on_approval_requested.unwrap_or(true),
        notify_on_limit_warning: req.notify_on_limit_warning.unwrap_or(true),
        created_at: now.clone(),
        updated_at: now,
    };

    if let Err(e) = insert_channel_db(pool, &channel).await {
        delete_secret(pool, &channel.vault_secret_id).await;
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e);
    }

    let pub_channel: PublicNotificationChannel = channel.into();
    (StatusCode::CREATED, Json(pub_channel)).into_response()
}

fn apply_channel_fields(channel: &mut NotificationChannel, req: &UpdateChannelRequest) {
    if let Some(new_name) = &req.name {
        channel.name = new_name.trim().to_string();
    }
    if let Some(new_chat_id) = &req.chat_id {
        channel.chat_id = Some(new_chat_id.trim().to_string());
    }
    if let Some(new_enabled) = req.is_enabled {
        channel.is_enabled = new_enabled;
    }
    if let Some(nh) = req.notify_on_needs_human {
        channel.notify_on_needs_human = nh;
    }
    if let Some(nf) = req.notify_on_failed {
        channel.notify_on_failed = nf;
    }
    if let Some(na) = req.notify_on_approval_requested {
        channel.notify_on_approval_requested = na;
    }
    if let Some(nl) = req.notify_on_limit_warning {
        channel.notify_on_limit_warning = nl;
    }
}

async fn update_channel_db(
    pool: &SqlitePool,
    channel: &NotificationChannel,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE m_notifications_channel
        SET name = ?, vault_secret_id = ?, chat_id = ?, is_enabled = ?,
            notify_on_needs_human = ?, notify_on_failed = ?,
            notify_on_approval_requested = ?, notify_on_limit_warning = ?,
            updated_at = ?
        WHERE id = ? AND workspace_id = ?
        "#,
    )
    .bind(&channel.name)
    .bind(&channel.vault_secret_id)
    .bind(&channel.chat_id)
    .bind(channel.is_enabled)
    .bind(channel.notify_on_needs_human)
    .bind(channel.notify_on_failed)
    .bind(channel.notify_on_approval_requested)
    .bind(channel.notify_on_limit_warning)
    .bind(&channel.updated_at)
    .bind(&channel.id)
    .bind(&channel.workspace_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Lưu bí mật mới vào vault (nếu có) và trả về tham chiếu bí mật cũ để xóa sau khi DB cập nhật thành công.
async fn rotate_secret(
    pool: &SqlitePool,
    channel: &mut NotificationChannel,
    new_secret: Option<String>,
) -> Result<Option<String>, String> {
    let Some(new_secret) = new_secret.filter(|s| !s.trim().is_empty()) else {
        return Ok(None);
    };
    let label = format!("notification:{}:{}", channel.channel_type, channel.name);
    let sid = store_secret(pool, &label, new_secret.trim()).await?;
    Ok(Some(std::mem::replace(&mut channel.vault_secret_id, sid)))
}

async fn update_channel(
    State(state): State<Arc<NotificationsState>>,
    Path((ws, id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(req): Json<UpdateChannelRequest>,
) -> Response {
    let pool = &state.pool;
    let Some(user) = session_user(pool, &headers).await else {
        return err_resp(StatusCode::UNAUTHORIZED, "Chưa đăng nhập");
    };
    if !can_administer_workspace(pool, &user, &ws).await {
        return err_resp(StatusCode::FORBIDDEN, "Yêu cầu quyền admin workspace");
    }

    let existing: Option<NotificationChannel> = match sqlx::query_as(
        r#"
        SELECT id, workspace_id, name, channel_type, vault_secret_id, chat_id,
               is_enabled, notify_on_needs_human, notify_on_failed,
               notify_on_approval_requested, notify_on_limit_warning,
               created_at, updated_at
        FROM m_notifications_channel
        WHERE id = ? AND workspace_id = ?
        "#,
    )
    .bind(&id)
    .bind(&ws)
    .fetch_optional(pool)
    .await
    {
        Ok(c) => c,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let Some(mut channel) = existing else {
        return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy kênh thông báo");
    };

    apply_channel_fields(&mut channel, &req);

    // Bí mật cũ chỉ bị xóa SAU khi cập nhật DB thành công (tránh kênh trỏ tới bí mật đã mất).
    let old_secret_ref = match rotate_secret(pool, &mut channel, req.secret).await {
        Ok(old) => old,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e),
    };

    channel.updated_at = Utc::now().to_rfc3339();

    if let Err(e) = update_channel_db(pool, &channel).await {
        if old_secret_ref.is_some() {
            delete_secret(pool, &channel.vault_secret_id).await;
        }
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }
    if let Some(old_ref) = old_secret_ref {
        delete_secret(pool, &old_ref).await;
    }

    let pub_channel: PublicNotificationChannel = channel.into();
    Json(pub_channel).into_response()
}

async fn delete_channel(
    State(state): State<Arc<NotificationsState>>,
    Path((ws, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let pool = &state.pool;
    let Some(user) = session_user(pool, &headers).await else {
        return err_resp(StatusCode::UNAUTHORIZED, "Chưa đăng nhập");
    };

    if !can_administer_workspace(pool, &user, &ws).await {
        return err_resp(StatusCode::FORBIDDEN, "Yêu cầu quyền admin workspace");
    }

    let existing: Option<NotificationChannel> = match sqlx::query_as(
        "SELECT id, workspace_id, name, channel_type, vault_secret_id, chat_id, is_enabled, \
         notify_on_needs_human, notify_on_failed, notify_on_approval_requested, notify_on_limit_warning, \
         created_at, updated_at FROM m_notifications_channel WHERE id = ? AND workspace_id = ?",
    )
    .bind(&id)
    .bind(&ws)
    .fetch_optional(pool)
    .await
    {
        Ok(c) => c,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let Some(channel) = existing else {
        return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy kênh để xóa");
    };

    delete_secret(pool, &channel.vault_secret_id).await;

    let res = sqlx::query("DELETE FROM m_notifications_channel WHERE id = ? AND workspace_id = ?")
        .bind(&id)
        .bind(&ws)
        .execute(pool)
        .await;

    match res {
        Ok(r) if r.rows_affected() > 0 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => err_resp(StatusCode::NOT_FOUND, "Không tìm thấy kênh để xóa"),
        Err(e) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn fetch_channel_by_id(
    pool: &SqlitePool,
    id: &str,
    ws: &str,
) -> Result<Option<NotificationChannel>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT id, workspace_id, name, channel_type, vault_secret_id, chat_id,
               is_enabled, notify_on_needs_human, notify_on_failed,
               notify_on_approval_requested, notify_on_limit_warning,
               created_at, updated_at
        FROM m_notifications_channel
        WHERE id = ? AND workspace_id = ?
        "#,
    )
    .bind(id)
    .bind(ws)
    .fetch_optional(pool)
    .await
}

async fn save_test_history(
    pool: &SqlitePool,
    ws: &str,
    ch_id: &str,
    status: &str,
    err_msg: Option<&str>,
) {
    let hist_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let _ = sqlx::query(
        r#"
        INSERT INTO m_notifications_history (
            id, workspace_id, channel_id, event_type, task_id, title, status, error_message, created_at
        ) VALUES (?, ?, ?, 'test', NULL, '[Dandori] Thử nghiệm thông báo', ?, ?, ?)
        "#,
    )
    .bind(&hist_id)
    .bind(ws)
    .bind(ch_id)
    .bind(status)
    .bind(err_msg)
    .bind(&now)
    .execute(pool)
    .await;
}

async fn perform_test_send(
    pool: &SqlitePool,
    channel: &NotificationChannel,
) -> Result<(), (StatusCode, String)> {
    let secret_value = reveal_secret(pool, &channel.vault_secret_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let ctype = ChannelType::from_str_opt(&channel.channel_type).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "Loại kênh không hợp lệ".to_string(),
        )
    })?;

    let send_result = dispatch_notification(
        ctype,
        &secret_value,
        channel.chat_id.as_deref(),
        "[Dandori] Thử nghiệm thông báo",
        "Kiểm tra kết nối thành công",
        None,
        None,
    )
    .await;

    let (status, err_msg) = match &send_result {
        Ok(()) => ("sent", None),
        Err(e) => (
            "failed",
            Some(sanitize_error_message(&e.to_string(), &secret_value)),
        ),
    };
    save_test_history(
        pool,
        &channel.workspace_id,
        &channel.id,
        status,
        err_msg.as_deref(),
    )
    .await;

    send_result.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            sanitize_error_message(&e.to_string(), &secret_value),
        )
    })
}

async fn send_test_notification(
    State(state): State<Arc<NotificationsState>>,
    Path((ws, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let pool = &state.pool;
    let Some(user) = session_user(pool, &headers).await else {
        return err_resp(StatusCode::UNAUTHORIZED, "Chưa đăng nhập");
    };
    if !can_administer_workspace(pool, &user, &ws).await {
        return err_resp(StatusCode::FORBIDDEN, "Yêu cầu quyền admin workspace");
    }

    let channel = match fetch_channel_by_id(pool, &id, &ws).await {
        Ok(Some(c)) => c,
        Ok(None) => return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy kênh thông báo"),
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    match perform_test_send(pool, &channel).await {
        Ok(()) => Json(json!({
            "success": true,
            "message": "Gửi thông báo thử nghiệm thành công"
        }))
        .into_response(),
        Err((code, safe_err)) => {
            (code, Json(json!({ "success": false, "error": safe_err }))).into_response()
        }
    }
}

async fn get_notification_history(
    State(state): State<Arc<NotificationsState>>,
    Path(ws): Path<String>,
    headers: HeaderMap,
) -> Response {
    let pool = &state.pool;
    let Some(user) = session_user(pool, &headers).await else {
        return err_resp(StatusCode::UNAUTHORIZED, "Chưa đăng nhập");
    };

    if !can_access_workspace(pool, &user, &ws).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    let records: Vec<NotificationHistoryRecord> = match sqlx::query_as(
        r#"
        SELECT id, workspace_id, channel_id, event_type, task_id, title, status, error_message, created_at
        FROM m_notifications_history
        WHERE workspace_id = ?
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(&ws)
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    Json(records).into_response()
}

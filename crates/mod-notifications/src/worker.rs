//! Tiến trình nền lắng nghe sự kiện và gửi thông báo theo kênh (P3-12).

use chrono::Utc;
use dandori_module_api::{EventBus, EventFilter};
use dandori_types::{Envelope, EventPayload};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::model::{ChannelType, NotificationChannel};
use crate::sender::dispatch_notification;
use crate::vault::reveal_secret;

fn parse_task_status_event(
    task_id: &dandori_types::TaskId,
    new_status: &str,
) -> Option<(&'static str, Option<String>, String, String)> {
    let s = new_status;
    if !matches!(s, "needs_human" | "failed" | "in_review") {
        return None;
    }
    let ev_type = match s {
        "needs_human" => "needs_human",
        "failed" => "failed",
        _ => "approval_requested",
    };
    Some((
        ev_type,
        Some(task_id.to_string()),
        format!("[Dandori] Task {task_id}"),
        s.to_string(),
    ))
}

fn parse_envelope_info(env: &Envelope) -> Option<(&'static str, Option<String>, String, String)> {
    match env.payload {
        EventPayload::TaskStatusChanged {
            ref task_id,
            ref new_status,
            ..
        } => parse_task_status_event(task_id, new_status),
        EventPayload::ApprovalRequested {
            ref task_id,
            ref reason,
            ..
        } => Some((
            "approval_requested",
            Some(task_id.to_string()),
            format!("[Dandori] Yêu cầu phê duyệt task {task_id}"),
            reason.clone(),
        )),
        EventPayload::AccountQuotaUpdated {
            ref account_id,
            ref exhausted_until,
        } => {
            exhausted_until.as_ref()?;
            Some((
                "limit_warning",
                None,
                format!("[Dandori] Cảnh báo hạn mức tài khoản {account_id}"),
                "Tài khoản chạm hạn mức sử dụng".to_string(),
            ))
        }
        EventPayload::NotificationRequested {
            ref title,
            ref message,
            ..
        } => Some((
            "notification.requested",
            env.scope.task_id.as_ref().map(ToString::to_string),
            title.clone(),
            message.clone(),
        )),
        _ => None,
    }
}

/// Xử lý một Envelope sự kiện hệ thống.
pub async fn handle_envelope(pool: &SqlitePool, env: &Envelope) {
    let Some(ws_id) = env.scope.workspace_id.as_ref() else {
        return;
    };
    let Some((event_type, task_id, title, status)) = parse_envelope_info(env) else {
        return;
    };

    let ws_str = ws_id.to_string();
    let _ = process_channels_for_event(
        pool,
        &ws_str,
        event_type,
        task_id.as_deref(),
        &title,
        &status,
    )
    .await;
}

async fn process_channels_for_event(
    pool: &SqlitePool,
    workspace_id: &str,
    event_type: &str,
    task_id: Option<&str>,
    title: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    let channels: Vec<NotificationChannel> = sqlx::query_as(
        r#"
        SELECT id, workspace_id, name, channel_type, vault_secret_id, chat_id,
               is_enabled, notify_on_needs_human, notify_on_failed,
               notify_on_approval_requested, notify_on_limit_warning,
               created_at, updated_at
        FROM m_notifications_channel
        WHERE workspace_id = ? AND is_enabled = 1
        "#,
    )
    .bind(workspace_id)
    .fetch_all(pool)
    .await?;

    for channel in channels {
        let should_send = match event_type {
            "needs_human" => channel.notify_on_needs_human,
            "failed" => channel.notify_on_failed,
            "approval_requested" => channel.notify_on_approval_requested,
            "limit_warning" => channel.notify_on_limit_warning,
            _ => true,
        };

        if !should_send {
            continue;
        }

        send_to_channel(pool, &channel, event_type, task_id, title, status).await;
    }

    Ok(())
}

async fn is_duplicate_notification(
    pool: &SqlitePool,
    channel_id: &str,
    task_id: &str,
    event_type: &str,
) -> bool {
    let row: Option<(i64,)> = sqlx::query_as(
        r#"
        SELECT 1 FROM m_notifications_history
        WHERE channel_id = ? AND task_id = ? AND event_type = ? AND status = 'sent'
          AND created_at >= datetime('now', '-300 seconds')
        LIMIT 1
        "#,
    )
    .bind(channel_id)
    .bind(task_id)
    .bind(event_type)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    row.is_some()
}

async fn is_rate_limited(pool: &SqlitePool, channel_id: &str) -> bool {
    let recent: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM m_notifications_history
        WHERE channel_id = ? AND status = 'sent'
          AND created_at >= datetime('now', '-60 seconds')
        "#,
    )
    .bind(channel_id)
    .fetch_one(pool)
    .await
    .unwrap_or((0,));

    recent.0 >= 10
}

async fn send_via_channel_provider(
    channel: &NotificationChannel,
    secret_value: &str,
    task_id: Option<&str>,
    title: &str,
    status: &str,
) -> Result<(), String> {
    let ctype = ChannelType::from_str_opt(&channel.channel_type)
        .ok_or_else(|| "Loại kênh không hợp lệ".to_string())?;
    let link = task_id.map(|tid| format!("/tasks/{tid}"));
    dispatch_notification(
        ctype,
        secret_value,
        channel.chat_id.as_deref(),
        title,
        status,
        link.as_deref(),
        None,
    )
    .await
    .map_err(|e| e.to_string())
}

async fn execute_dispatch_channel(
    pool: &SqlitePool,
    channel: &NotificationChannel,
    event_type: &str,
    task_id: Option<&str>,
    title: &str,
    status: &str,
    now: &str,
) {
    let secret_res = reveal_secret(pool, &channel.vault_secret_id).await;
    let Ok(secret_value) = secret_res else {
        record_history(
            pool,
            &channel.workspace_id,
            &channel.id,
            event_type,
            task_id,
            title,
            "failed",
            Some("Không tìm thấy bí mật trong kho khóa"),
            now,
        )
        .await;
        return;
    };

    let res = send_via_channel_provider(channel, &secret_value, task_id, title, status).await;
    let (st, err_msg) = match res {
        Ok(()) => ("sent", None),
        Err(e) => ("failed", Some(e)),
    };
    record_history(
        pool,
        &channel.workspace_id,
        &channel.id,
        event_type,
        task_id,
        title,
        st,
        err_msg.as_deref(),
        now,
    )
    .await;
}

async fn send_to_channel(
    pool: &SqlitePool,
    channel: &NotificationChannel,
    event_type: &str,
    task_id: Option<&str>,
    title: &str,
    status: &str,
) {
    let now = Utc::now().to_rfc3339();

    let is_dup = match task_id {
        Some(tid) => is_duplicate_notification(pool, &channel.id, tid, event_type).await,
        None => false,
    };

    if is_dup {
        record_history(
            pool,
            &channel.workspace_id,
            &channel.id,
            event_type,
            task_id,
            title,
            "deduplicated",
            None,
            &now,
        )
        .await;
        return;
    }

    if is_rate_limited(pool, &channel.id).await {
        record_history(
            pool,
            &channel.workspace_id,
            &channel.id,
            event_type,
            task_id,
            title,
            "rate_limited",
            None,
            &now,
        )
        .await;
        return;
    }

    execute_dispatch_channel(pool, channel, event_type, task_id, title, status, &now).await;
}

#[expect(
    clippy::too_many_arguments,
    reason = "Hàm ghi bản ghi lịch sử thông báo"
)]
async fn record_history(
    pool: &SqlitePool,
    workspace_id: &str,
    channel_id: &str,
    event_type: &str,
    task_id: Option<&str>,
    title: &str,
    status: &str,
    error_message: Option<&str>,
    created_at: &str,
) {
    let id = Uuid::new_v4().to_string();
    let _ = sqlx::query(
        r#"
        INSERT INTO m_notifications_history (
            id, workspace_id, channel_id, event_type, task_id, title, status, error_message, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(workspace_id)
    .bind(channel_id)
    .bind(event_type)
    .bind(task_id)
    .bind(title)
    .bind(status)
    .bind(error_message)
    .bind(created_at)
    .execute(pool)
    .await;
}

/// Chạy vòng lặp worker xử lý thông báo.
pub async fn run_notifications_worker(
    pool: SqlitePool,
    event_bus: Arc<EventBus>,
    cancel_token: CancellationToken,
) {
    let mut rx = event_bus.subscribe(EventFilter::all());
    loop {
        tokio::select! {
            () = cancel_token.cancelled() => break,
            res = rx.recv() => {
                let Some(env) = res else { break };
                handle_envelope(&pool, &env).await;
            }
        }
    }
}

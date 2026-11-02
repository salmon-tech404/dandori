//! Worker chạy nền tự động xử lý các yêu cầu phê duyệt đã hết hạn (RS-ASYNC-004).

use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use dandori_module_api::EventBus;
use dandori_types::{
    Envelope, EventPayload, EventScope, ProjectId, TaskId, TaskStatus, WorkspaceId,
};
use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;

async fn revert_expired_task(
    pool: &SqlitePool,
    event_bus: &Arc<EventBus>,
    ws_id_str: &str,
    prj_id_str: &str,
    task_id_str: &str,
    from_status: String,
    now_str: &str,
) {
    let _ = sqlx::query("UPDATE task SET status = ?, updated_at = ? WHERE id = ?")
        .bind(&from_status)
        .bind(now_str)
        .bind(task_id_str)
        .execute(pool)
        .await;

    let ws_id =
        WorkspaceId::from_str(ws_id_str).unwrap_or_else(|_| WorkspaceId::from(uuid::Uuid::nil()));
    let prj_id =
        ProjectId::from_str(prj_id_str).unwrap_or_else(|_| ProjectId::from(uuid::Uuid::nil()));
    let t_id = TaskId::from_str(task_id_str).unwrap_or_else(|_| TaskId::from(uuid::Uuid::nil()));

    event_bus.publish(Envelope::new(
        Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::TaskStatusChanged {
            task_id: t_id,
            old_status: TaskStatus::NeedsHuman.as_str().to_string(),
            new_status: from_status,
        },
    ));

    event_bus.publish(Envelope::new(
        Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::ApprovalDecided {
            gate_id: "approval-gate".to_string(),
            task_id: t_id,
            approved: false,
            decided_by: "system:expired".to_string(),
        },
    ));
}

/// Quét và xử lý các yêu cầu phê duyệt đã quá thời hạn.
pub async fn process_expired_requests_once(
    pool: &SqlitePool,
    event_bus: &Arc<EventBus>,
) -> Result<usize, sqlx::Error> {
    let now_str = Utc::now().to_rfc3339();

    let expired_rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT id, workspace_id, project_id, task_id, from_status \
         FROM m_approval_gate_request \
         WHERE status = 'pending' AND expires_at <= ?",
    )
    .bind(&now_str)
    .fetch_all(pool)
    .await?;

    let mut count = 0;
    for (id, ws_id_str, prj_id_str, task_id_str, from_status) in expired_rows {
        let res = sqlx::query(
            "UPDATE m_approval_gate_request \
             SET status = 'expired', decided_at = ? \
             WHERE id = ? AND status = 'pending'",
        )
        .bind(&now_str)
        .bind(&id)
        .execute(pool)
        .await?;

        if res.rows_affected() > 0 {
            count += 1;
            revert_expired_task(
                pool,
                event_bus,
                &ws_id_str,
                &prj_id_str,
                &task_id_str,
                from_status,
                &now_str,
            )
            .await;
        }
    }

    Ok(count)
}

/// Chạy vòng lặp worker kiểm tra hết hạn, hỗ trợ dừng sạch khi token bị hủy.
pub async fn run_expiry_worker(
    pool: SqlitePool,
    event_bus: Arc<EventBus>,
    cancel_token: CancellationToken,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                break;
            }
            _ = interval.tick() => {
                let _ = process_expired_requests_once(&pool, &event_bus).await;
            }
        }
    }
}

//! Tiến trình nền ghi nhật ký kiểm toán từ các sự kiện hệ thống nhạy cảm (P3-14).
//!
//! Chỉ ghi các sự kiện lõi hiện có: bật/tắt module, yêu cầu và quyết định phê duyệt, tài khoản chạm hạn mức.
//! Các sự kiện chưa có đường phát từ lõi (đăng nhập, truy cập kho khóa, ghép thiết bị) chưa được ghi ở đây.

use std::sync::Arc;

use dandori_module_api::{EventBus, EventFilter};
use dandori_types::{Envelope, EventPayload};
use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;

use crate::model::AppendLogRequest;
use crate::routes::insert_audit_entry;

fn request(
    actor: &str,
    action: &str,
    target_type: &str,
    target_id: String,
    details: serde_json::Value,
) -> AppendLogRequest {
    AppendLogRequest {
        actor_id: actor.to_string(),
        actor_name: actor.to_string(),
        action: action.to_string(),
        target_type: target_type.to_string(),
        target_id,
        details,
        source: "event".to_string(),
    }
}

/// Bản ghi do hệ thống tự phát sinh (không có người dùng cụ thể).
fn system_request(action: &str, target_type: &str, target_id: String) -> AppendLogRequest {
    request(
        "system",
        action,
        target_type,
        target_id,
        serde_json::json!({}),
    )
}

/// Chuyển sự kiện phê duyệt thành yêu cầu ghi kiểm toán.
fn approval_request(payload: &EventPayload) -> Option<AppendLogRequest> {
    match payload {
        EventPayload::ApprovalRequested {
            gate_id, task_id, ..
        } => Some(request(
            "system",
            "approval_requested",
            "task",
            task_id.to_string(),
            serde_json::json!({ "gate_id": gate_id }),
        )),
        EventPayload::ApprovalDecided {
            gate_id,
            task_id,
            approved,
            decided_by,
        } => {
            let action = if *approved {
                "approval_approved"
            } else {
                "approval_rejected"
            };
            Some(request(
                decided_by,
                action,
                "task",
                task_id.to_string(),
                serde_json::json!({ "gate_id": gate_id }),
            ))
        }
        _ => None,
    }
}

/// Chuyển một sự kiện hệ thống thành yêu cầu ghi kiểm toán (nếu là sự kiện nhạy cảm).
fn to_audit_request(env: &Envelope) -> Option<AppendLogRequest> {
    match &env.payload {
        EventPayload::ModuleEnabled { module_id } => Some(system_request(
            "module_enabled",
            "module",
            module_id.to_string(),
        )),
        EventPayload::ModuleDisabled { module_id } => Some(system_request(
            "module_disabled",
            "module",
            module_id.to_string(),
        )),
        EventPayload::AccountQuotaUpdated {
            account_id,
            exhausted_until: Some(_),
        } => Some(system_request(
            "account_quota_exhausted",
            "agent_account",
            account_id.to_string(),
        )),
        other => approval_request(other),
    }
}

/// Xử lý một sự kiện: chỉ ghi khi sự kiện thuộc một workspace cụ thể.
pub async fn handle_envelope(pool: &SqlitePool, env: &Envelope) {
    let Some(ws_id) = env.scope.workspace_id.as_ref() else {
        return;
    };
    let Some(req) = to_audit_request(env) else {
        return;
    };
    if let Err(code) = insert_audit_entry(pool, &ws_id.to_string(), &req).await {
        tracing::warn!("Không ghi được bản ghi kiểm toán từ sự kiện: {code}");
    }
}

/// Vòng lặp nền nhận sự kiện từ bus và ghi kiểm toán.
pub async fn run_audit_worker(
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

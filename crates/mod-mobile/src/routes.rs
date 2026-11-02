//! Định nghĩa các route REST API cho module mobile (P3-13).

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use chrono::{Duration, Utc};
use dandori_module_api::access::{can_administer_workspace, session_user};
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::{authenticate_mobile_device, generate_mobile_token, validate_origin};
use crate::model::{
    CreateDeviceRequest, CreateDeviceResponse, MobileApprovalItem, MobileDecisionRequest,
    MobileDevice, MobileStatusResponse, PublicMobileConfig, PublicMobileDevice,
    UpdateMobileConfigRequest,
};

/// Trạng thái chia sẻ của router module mobile.
#[derive(Clone, Debug)]
pub struct MobileState {
    /// Pool cơ sở dữ liệu.
    pub pool: SqlitePool,
}

fn err_resp(code: StatusCode, msg: &str) -> Response {
    (
        code,
        Json(json!({
            "error": {
                "message": msg
            }
        })),
    )
        .into_response()
}

/// Tạo Router cho module mobile.
pub fn router(state: MobileState) -> Router {
    let shared = Arc::new(state);
    Router::new()
        // Các route dành riêng cho thiết bị di động (xác thực qua mobile token)
        .route("/status", get(mobile_get_status))
        .route("/mobile/status", get(mobile_get_status))
        .route("/approvals", get(mobile_list_approvals))
        .route("/mobile/approvals", get(mobile_list_approvals))
        .route("/approvals/{id}/decision", post(mobile_make_decision))
        .route(
            "/mobile/approvals/{id}/decision",
            post(mobile_make_decision),
        )
        // Các route quản trị thiết bị (xác thực qua session của admin workspace)
        .route(
            "/workspaces/{ws}/devices",
            get(list_devices).post(create_device),
        )
        .route(
            "/workspaces/{ws}/mobile/devices",
            get(list_devices).post(create_device),
        )
        .route("/workspaces/{ws}/devices/{id}/revoke", post(revoke_device))
        .route(
            "/workspaces/{ws}/mobile/devices/{id}/revoke",
            post(revoke_device),
        )
        .route("/workspaces/{ws}/devices/{id}", delete(delete_device))
        .route(
            "/workspaces/{ws}/mobile/devices/{id}",
            delete(delete_device),
        )
        .route(
            "/workspaces/{ws}/config",
            get(get_config).put(update_config),
        )
        .route(
            "/workspaces/{ws}/mobile/config",
            get(get_config).put(update_config),
        )
        .route("/workspaces/{ws}/logs", get(list_action_logs))
        .route("/workspaces/{ws}/mobile/logs", get(list_action_logs))
        .with_state(shared)
}

async fn fetch_allowed_origins(pool: &SqlitePool, ws_id: &str) -> Vec<String> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT allowed_origins FROM m_mobile_config WHERE workspace_id = ?")
            .bind(ws_id)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

    if let Some((json_str,)) = row {
        serde_json::from_str(&json_str).unwrap_or_default()
    } else {
        Vec::new()
    }
}

async fn is_workspace_mobile_read_only(pool: &SqlitePool, ws_id: &str) -> bool {
    let row: Option<(bool,)> =
        sqlx::query_as("SELECT read_only FROM m_mobile_config WHERE workspace_id = ?")
            .bind(ws_id)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

    row.map(|(ro,)| ro).unwrap_or(false)
}

async fn authenticate_and_validate_mobile(
    pool: &SqlitePool,
    headers: &HeaderMap,
) -> Result<MobileDevice, (StatusCode, &'static str)> {
    let device = authenticate_mobile_device(pool, headers).await?;
    let allowed = fetch_allowed_origins(pool, &device.workspace_id).await;
    validate_origin(headers, &allowed)?;
    Ok(device)
}

async fn mobile_get_status(State(state): State<Arc<MobileState>>, headers: HeaderMap) -> Response {
    let pool = &state.pool;
    let device = match authenticate_and_validate_mobile(pool, &headers).await {
        Ok(d) => d,
        Err((code, msg)) => return err_resp(code, msg),
    };

    let ws_read_only = is_workspace_mobile_read_only(pool, &device.workspace_id).await;
    let effective_read_only = device.read_only || ws_read_only;

    let pending_approvals: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM task
        WHERE workspace_id = ? AND status IN ('needs_human', 'in_review')
        "#,
    )
    .bind(&device.workspace_id)
    .fetch_one(pool)
    .await
    .unwrap_or((0,));

    let active_runs: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM task
        WHERE workspace_id = ? AND status = 'in_progress'
        "#,
    )
    .bind(&device.workspace_id)
    .fetch_one(pool)
    .await
    .unwrap_or((0,));

    Json(MobileStatusResponse {
        pending_approvals_count: pending_approvals.0,
        active_runs_count: active_runs.0,
        is_read_only: effective_read_only,
        device_name: device.name,
    })
    .into_response()
}

async fn mobile_list_approvals(
    State(state): State<Arc<MobileState>>,
    headers: HeaderMap,
) -> Response {
    let pool = &state.pool;
    let device = match authenticate_and_validate_mobile(pool, &headers).await {
        Ok(d) => d,
        Err((code, msg)) => return err_resp(code, msg),
    };

    let tasks: Vec<(String, String, Option<String>, String, String)> = match sqlx::query_as(
        r#"
        SELECT t.id, t.title, t.description, t.status, t.created_at
        FROM task t
        JOIN project p ON t.project_id = p.id
        WHERE p.workspace_id = ? AND t.status IN ('needs_human', 'in_review')
        ORDER BY t.created_at DESC
        "#,
    )
    .bind(&device.workspace_id)
    .fetch_all(pool)
    .await
    {
        Ok(t) => t,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let items: Vec<MobileApprovalItem> = tasks
        .into_iter()
        .map(|(tid, title, desc, st, cat)| MobileApprovalItem {
            id: tid.clone(),
            task_id: tid,
            task_title: title,
            reason: desc.unwrap_or_else(|| "Yêu cầu can thiệp từ hệ thống".to_string()),
            status: st,
            created_at: cat,
        })
        .collect();

    Json(items).into_response()
}

async fn log_mobile_action(
    pool: &SqlitePool,
    ws_id: &str,
    device_id: &str,
    action: &str,
    target_id: Option<&str>,
    detail: Option<&str>,
) {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let _ = sqlx::query(
        r#"
        INSERT INTO m_mobile_action_log (
            id, workspace_id, device_id, action, target_id, detail, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(ws_id)
    .bind(device_id)
    .bind(action)
    .bind(target_id)
    .bind(detail)
    .bind(&now)
    .execute(pool)
    .await;
}

async fn apply_mobile_decision(
    pool: &SqlitePool,
    target_task_id: &str,
    device_ws: &str,
    action: &str,
) -> Result<String, StatusCode> {
    let task_ws: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT p.workspace_id
        FROM task t
        JOIN project p ON t.project_id = p.id
        WHERE t.id = ?
        "#,
    )
    .bind(target_task_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some((task_ws_id,)) = task_ws else {
        return Err(StatusCode::NOT_FOUND);
    };

    if task_ws_id != device_ws {
        return Err(StatusCode::FORBIDDEN);
    }

    let new_status = if action == "approve" {
        "planned"
    } else {
        "failed"
    };

    let now = Utc::now().to_rfc3339();
    let res = sqlx::query(
        r#"
        UPDATE task
        SET status = ?, updated_at = ?
        WHERE id = ? AND status IN ('needs_human', 'in_review')
        "#,
    )
    .bind(new_status)
    .bind(&now)
    .bind(target_task_id)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if res.rows_affected() == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(new_status.to_string())
}

async fn mobile_make_decision(
    State(state): State<Arc<MobileState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<MobileDecisionRequest>,
) -> Response {
    let pool = &state.pool;
    let device = match authenticate_and_validate_mobile(pool, &headers).await {
        Ok(d) => d,
        Err((code, msg)) => return err_resp(code, msg),
    };

    let ws_read_only = is_workspace_mobile_read_only(pool, &device.workspace_id).await;
    if device.read_only || ws_read_only {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Thiết bị đang ở chế độ chỉ xem (Read-only), không thể phê duyệt",
        );
    }

    let act = payload.action.trim().to_ascii_lowercase();
    if act != "approve" && act != "reject" {
        return err_resp(
            StatusCode::BAD_REQUEST,
            "Hành động chỉ được là approve hoặc reject",
        );
    }

    let new_status = match apply_mobile_decision(pool, &id, &device.workspace_id, &act).await {
        Ok(s) => s,
        Err(code) => {
            return err_resp(
                code,
                "Không tìm thấy yêu cầu cần phê duyệt hoặc quyền không hợp lệ",
            );
        }
    };

    log_mobile_action(
        pool,
        &device.workspace_id,
        &device.id,
        &act,
        Some(&id),
        payload.comment.as_deref(),
    )
    .await;

    Json(json!({
        "success": true,
        "action": act,
        "task_id": id,
        "new_status": new_status,
        "device_name": device.name
    }))
    .into_response()
}

// ==================== CÁC ROUTE QUẢN TRỊ THIẾT BỊ ====================

async fn list_devices(
    State(state): State<Arc<MobileState>>,
    Path(ws): Path<String>,
    headers: HeaderMap,
) -> Response {
    let pool = &state.pool;
    let Some(user) = session_user(pool, &headers).await else {
        return err_resp(StatusCode::UNAUTHORIZED, "Chưa đăng nhập");
    };
    if !can_administer_workspace(pool, &user, &ws).await {
        return err_resp(StatusCode::FORBIDDEN, "Yêu cầu quyền admin workspace");
    }

    let devices: Vec<MobileDevice> = match sqlx::query_as(
        r#"
        SELECT id, workspace_id, name, token_hash, created_by, read_only,
               is_revoked, expires_at, last_used_at, created_at, updated_at
        FROM m_mobile_device
        WHERE workspace_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(&ws)
    .fetch_all(pool)
    .await
    {
        Ok(d) => d,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let pub_devices: Vec<PublicMobileDevice> = devices.into_iter().map(Into::into).collect();
    Json(pub_devices).into_response()
}

async fn insert_device_db(pool: &SqlitePool, dev: &MobileDevice) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO m_mobile_device (
            id, workspace_id, name, token_hash, created_by, read_only,
            is_revoked, expires_at, last_used_at, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&dev.id)
    .bind(&dev.workspace_id)
    .bind(&dev.name)
    .bind(&dev.token_hash)
    .bind(&dev.created_by)
    .bind(dev.read_only)
    .bind(dev.is_revoked)
    .bind(&dev.expires_at)
    .bind(&dev.last_used_at)
    .bind(&dev.created_at)
    .bind(&dev.updated_at)
    .execute(pool)
    .await?;
    Ok(())
}

async fn create_device(
    State(state): State<Arc<MobileState>>,
    Path(ws): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<CreateDeviceRequest>,
) -> Response {
    let pool = &state.pool;
    let Some(user) = session_user(pool, &headers).await else {
        return err_resp(StatusCode::UNAUTHORIZED, "Chưa đăng nhập");
    };
    if !can_administer_workspace(pool, &user, &ws).await {
        return err_resp(StatusCode::FORBIDDEN, "Yêu cầu quyền admin workspace");
    }

    let name = payload.name.trim();
    if name.is_empty() {
        return err_resp(StatusCode::BAD_REQUEST, "Tên thiết bị không được để trống");
    }

    let id = Uuid::new_v4().to_string();
    let (raw_token, token_hash) = generate_mobile_token();
    let ttl_h = payload.ttl_hours.unwrap_or(720).clamp(1, 8760);
    let now = Utc::now();
    let expires_at = (now + Duration::hours(ttl_h)).to_rfc3339();
    let now_str = now.to_rfc3339();

    let dev = MobileDevice {
        id,
        workspace_id: ws,
        name: name.to_string(),
        token_hash,
        created_by: user.user_id.clone(),
        read_only: payload.read_only.unwrap_or(false),
        is_revoked: false,
        expires_at,
        last_used_at: None,
        created_at: now_str.clone(),
        updated_at: now_str,
    };

    if let Err(e) = insert_device_db(pool, &dev).await {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }

    let pub_dev: PublicMobileDevice = dev.into();
    (
        StatusCode::CREATED,
        Json(CreateDeviceResponse {
            device: pub_dev,
            token: raw_token,
        }),
    )
        .into_response()
}

async fn revoke_device(
    State(state): State<Arc<MobileState>>,
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

    let now_str = Utc::now().to_rfc3339();
    let res = sqlx::query(
        "UPDATE m_mobile_device SET is_revoked = 1, updated_at = ? WHERE id = ? AND workspace_id = ?",
    )
    .bind(&now_str)
    .bind(&id)
    .bind(&ws)
    .execute(pool)
    .await;

    match res {
        Ok(r) if r.rows_affected() > 0 => {
            Json(json!({ "success": true, "revoked": true })).into_response()
        }
        Ok(_) => err_resp(StatusCode::NOT_FOUND, "Không tìm thấy thiết bị"),
        Err(e) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn delete_device(
    State(state): State<Arc<MobileState>>,
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

    let res = sqlx::query("DELETE FROM m_mobile_device WHERE id = ? AND workspace_id = ?")
        .bind(&id)
        .bind(&ws)
        .execute(pool)
        .await;

    match res {
        Ok(r) if r.rows_affected() > 0 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => err_resp(StatusCode::NOT_FOUND, "Không tìm thấy thiết bị"),
        Err(e) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn get_config(
    State(state): State<Arc<MobileState>>,
    Path(ws): Path<String>,
    headers: HeaderMap,
) -> Response {
    let pool = &state.pool;
    let Some(user) = session_user(pool, &headers).await else {
        return err_resp(StatusCode::UNAUTHORIZED, "Chưa đăng nhập");
    };
    if !can_administer_workspace(pool, &user, &ws).await {
        return err_resp(StatusCode::FORBIDDEN, "Yêu cầu quyền admin workspace");
    }

    let origins = fetch_allowed_origins(pool, &ws).await;
    let ro = is_workspace_mobile_read_only(pool, &ws).await;

    Json(PublicMobileConfig {
        workspace_id: ws,
        allowed_origins: origins,
        read_only: ro,
    })
    .into_response()
}

async fn update_config(
    State(state): State<Arc<MobileState>>,
    Path(ws): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<UpdateMobileConfigRequest>,
) -> Response {
    let pool = &state.pool;
    let Some(user) = session_user(pool, &headers).await else {
        return err_resp(StatusCode::UNAUTHORIZED, "Chưa đăng nhập");
    };
    if !can_administer_workspace(pool, &user, &ws).await {
        return err_resp(StatusCode::FORBIDDEN, "Yêu cầu quyền admin workspace");
    }

    let existing_origins = fetch_allowed_origins(pool, &ws).await;
    let existing_ro = is_workspace_mobile_read_only(pool, &ws).await;

    let new_origins = payload.allowed_origins.unwrap_or(existing_origins);
    if new_origins.len() > 20
        || new_origins
            .iter()
            .any(|o| !dandori_module_api::is_valid_allowed_origin_entry(o))
    {
        return err_resp(
            StatusCode::BAD_REQUEST,
            "Danh sách Origin không hợp lệ: chỉ nhận tên máy, IP hoặc *.miền.đuôi (tối đa 20 mục)",
        );
    }
    let new_ro = payload.read_only.unwrap_or(existing_ro);
    let origins_json = serde_json::to_string(&new_origins).unwrap_or_else(|_| "[]".to_string());
    let now = Utc::now().to_rfc3339();

    let res = sqlx::query(
        r#"
        INSERT INTO m_mobile_config (workspace_id, allowed_origins, read_only, updated_at)
        VALUES (?, ?, ?, ?)
        ON CONFLICT(workspace_id) DO UPDATE SET
            allowed_origins = excluded.allowed_origins,
            read_only = excluded.read_only,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&ws)
    .bind(&origins_json)
    .bind(new_ro)
    .bind(&now)
    .execute(pool)
    .await;

    match res {
        Ok(_) => Json(PublicMobileConfig {
            workspace_id: ws,
            allowed_origins: new_origins,
            read_only: new_ro,
        })
        .into_response(),
        Err(e) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn list_action_logs(
    State(state): State<Arc<MobileState>>,
    Path(ws): Path<String>,
    headers: HeaderMap,
) -> Response {
    let pool = &state.pool;
    let Some(user) = session_user(pool, &headers).await else {
        return err_resp(StatusCode::UNAUTHORIZED, "Chưa đăng nhập");
    };
    if !can_administer_workspace(pool, &user, &ws).await {
        return err_resp(StatusCode::FORBIDDEN, "Yêu cầu quyền admin workspace");
    }

    let logs: Vec<crate::model::MobileActionLog> = match sqlx::query_as(
        r#"
        SELECT id, workspace_id, device_id, action, target_id, detail, created_at
        FROM m_mobile_action_log
        WHERE workspace_id = ?
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(&ws)
    .fetch_all(pool)
    .await
    {
        Ok(l) => l,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    Json(logs).into_response()
}

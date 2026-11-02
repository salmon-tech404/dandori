//! Định nghĩa các route HTTP cho module `overnight-queue`.

#![expect(
    clippy::too_many_lines,
    clippy::result_large_err,
    reason = "Xử lý các endpoint REST của overnight queue kèm kiểm tra BOLA và xác thực dữ liệu"
)]

use axum::Json;
use axum::Router;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use chrono::Utc;
use dandori_module_api::{
    ModuleContext, SessionUser, can_access_workspace, can_administer_workspace, session_user,
};
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use crate::model::{
    EnqueueTasksRequest, MorningSummaryDto, OvernightQueueConfig, OvernightQueueItem,
    QueueItemWithTask, ReorderQueueRequest, UpdateQueueConfigRequest,
};
use crate::worker::{Clock, RealClock};

/// Trạng thái Router của module overnight-queue.
#[derive(Clone, Debug)]
pub struct OvernightQueueState {
    /// Ngữ cảnh module.
    pub ctx: ModuleContext,
    /// Đồng hồ thời gian.
    pub clock: Arc<dyn Clock>,
}

impl OvernightQueueState {
    /// Khởi tạo trạng thái mới với đồng hồ thực tế.
    #[must_use]
    pub fn new(ctx: ModuleContext) -> Self {
        Self {
            ctx,
            clock: Arc::new(RealClock),
        }
    }

    /// Khởi tạo trạng thái với đồng hồ tùy chỉnh.
    #[must_use]
    pub fn with_clock(ctx: ModuleContext, clock: Arc<dyn Clock>) -> Self {
        Self { ctx, clock }
    }
}

fn err_resp(status: StatusCode, msg: &'static str) -> Response {
    (status, Json(json!({ "error": { "message": msg } }))).into_response()
}

fn err_resp_dyn(status: StatusCode, msg: String) -> Response {
    (status, Json(json!({ "error": { "message": msg } }))).into_response()
}

fn get_pool(state: &OvernightQueueState) -> Result<SqlitePool, Response> {
    state.ctx.pool().cloned().ok_or_else(|| {
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Không có kết nối cơ sở dữ liệu",
        )
    })
}

async fn get_user(headers: &HeaderMap, pool: &SqlitePool) -> Result<SessionUser, Response> {
    session_user(pool, headers).await.ok_or_else(|| {
        err_resp(
            StatusCode::UNAUTHORIZED,
            "Phiên đăng nhập không hợp lệ hoặc đã hết hạn",
        )
    })
}

/// Khởi tạo router HTTP cho module `overnight-queue`.
pub fn overnight_queue_router(state: OvernightQueueState) -> Router {
    Router::new()
        .route(
            "/workspaces/{ws_id}/queue",
            get(get_queue).put(update_config),
        )
        .route("/workspaces/{ws_id}/queue/items", post(enqueue_tasks))
        .route(
            "/workspaces/{ws_id}/queue/items/{item_id}",
            delete(delete_queue_item),
        )
        .route("/workspaces/{ws_id}/queue/reorder", put(reorder_queue))
        .route("/workspaces/{ws_id}/queue/pause", post(pause_queue))
        .route("/workspaces/{ws_id}/queue/resume", post(resume_queue))
        .route("/workspaces/{ws_id}/queue/start-now", post(start_queue_now))
        .route(
            "/workspaces/{ws_id}/queue/summary",
            get(get_morning_summary),
        )
        .with_state(state)
}

async fn ensure_config_exists(
    pool: &SqlitePool,
    ws_id: &str,
) -> Result<OvernightQueueConfig, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO m_overnight_queue_config (
            workspace_id, start_time, timezone, max_concurrency,
            consecutive_failure_limit, status, consecutive_failures, updated_at
        )
        VALUES (?, '23:00', 'UTC', 1, 3, 'idle', 0, ?)
        "#,
    )
    .bind(ws_id)
    .bind(&now)
    .execute(pool)
    .await?;

    sqlx::query_as(
        r#"
        SELECT workspace_id, start_time, timezone, max_concurrency,
               consecutive_failure_limit, status, consecutive_failures,
               last_run_summary, updated_at
        FROM m_overnight_queue_config
        WHERE workspace_id = ?
        "#,
    )
    .bind(ws_id)
    .fetch_one(pool)
    .await
}

/// `GET /workspaces/{ws_id}/queue` - Lấy thông tin hàng đợi và danh sách task.
async fn get_queue(
    AxumPath(ws_id): AxumPath<String>,
    State(state): State<OvernightQueueState>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return r,
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Không có quyền truy cập không gian làm việc này",
        );
    }

    let config = match ensure_config_exists(&pool, &ws_id).await {
        Ok(c) => c,
        Err(e) => return err_resp_dyn(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    let items: Vec<OvernightQueueItem> = match sqlx::query_as(
        r#"
        SELECT id, workspace_id, task_id, order_index, status, run_id, created_at, updated_at
        FROM m_overnight_queue_item
        WHERE workspace_id = ?
        ORDER BY order_index ASC
        "#,
    )
    .bind(&ws_id)
    .fetch_all(&pool)
    .await
    {
        Ok(i) => i,
        Err(e) => return err_resp_dyn(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    let mut items_with_task = Vec::with_capacity(items.len());
    for item in items {
        let task_info: Option<(String, String)> =
            sqlx::query_as("SELECT title, status FROM task WHERE id = ?")
                .bind(&item.task_id)
                .fetch_optional(&pool)
                .await
                .unwrap_or(None);

        let (title, status) =
            task_info.unwrap_or_else(|| ("Task không xác định".to_string(), "unknown".to_string()));
        items_with_task.push(QueueItemWithTask {
            item,
            task_title: title,
            task_status: status,
        });
    }

    Json(json!({
        "config": config,
        "items": items_with_task,
    }))
    .into_response()
}

/// `POST /workspaces/{ws_id}/queue/items` - Thêm các task vào hàng đợi.
async fn enqueue_tasks(
    AxumPath(ws_id): AxumPath<String>,
    State(state): State<OvernightQueueState>,
    headers: HeaderMap,
    Json(payload): Json<EnqueueTasksRequest>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return r,
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Không có quyền truy cập không gian làm việc này",
        );
    }

    let _ = ensure_config_exists(&pool, &ws_id).await;

    // Lấy max order_index hiện tại
    let max_order: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(order_index), 0) FROM m_overnight_queue_item WHERE workspace_id = ?",
    )
    .bind(&ws_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    let mut current_order = max_order;
    let now = Utc::now().to_rfc3339();
    let mut added_count = 0;

    for task_id in payload.task_ids {
        current_order += 1;
        let item_id = Uuid::new_v4().to_string();
        let res = sqlx::query(
            r#"
            INSERT OR IGNORE INTO m_overnight_queue_item (
                id, workspace_id, task_id, order_index, status, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, 'queued', ?, ?)
            "#,
        )
        .bind(&item_id)
        .bind(&ws_id)
        .bind(&task_id)
        .bind(current_order)
        .bind(&now)
        .bind(&now)
        .execute(&pool)
        .await;

        if let Ok(r) = res
            && r.rows_affected() > 0
        {
            added_count += 1;
        }
    }

    Json(json!({ "added": added_count, "success": true })).into_response()
}

/// `DELETE /workspaces/{ws_id}/queue/items/{item_id}` - Xóa một task khỏi hàng đợi.
async fn delete_queue_item(
    AxumPath((ws_id, item_id)): AxumPath<(String, String)>,
    State(state): State<OvernightQueueState>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return r,
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Không có quyền truy cập không gian làm việc này",
        );
    }

    let res =
        match sqlx::query("DELETE FROM m_overnight_queue_item WHERE id = ? AND workspace_id = ?")
            .bind(&item_id)
            .bind(&ws_id)
            .execute(&pool)
            .await
        {
            Ok(r) => r,
            Err(e) => return err_resp_dyn(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

    if res.rows_affected() == 0 {
        return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy mục trong hàng đợi");
    }

    Json(json!({ "deleted": true })).into_response()
}

/// `PUT /workspaces/{ws_id}/queue/reorder` - Sắp xếp lại thứ tự các task trong hàng đợi.
async fn reorder_queue(
    AxumPath(ws_id): AxumPath<String>,
    State(state): State<OvernightQueueState>,
    headers: HeaderMap,
    Json(payload): Json<ReorderQueueRequest>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return r,
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Không có quyền truy cập không gian làm việc này",
        );
    }

    let now = Utc::now().to_rfc3339();
    for (idx, item_id) in payload.ordered_item_ids.iter().enumerate() {
        let _ = sqlx::query(
            "UPDATE m_overnight_queue_item SET order_index = ?, updated_at = ? WHERE id = ? AND workspace_id = ?",
        )
        .bind(idx as i64)
        .bind(&now)
        .bind(item_id)
        .bind(&ws_id)
        .execute(&pool)
        .await;
    }

    Json(json!({ "success": true })).into_response()
}

/// `PUT /workspaces/{ws_id}/queue/config` - Cập nhật cấu hình hàng đợi qua đêm.
async fn update_config(
    AxumPath(ws_id): AxumPath<String>,
    State(state): State<OvernightQueueState>,
    headers: HeaderMap,
    Json(dto): Json<UpdateQueueConfigRequest>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return r,
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Cần quyền quản trị viên để cập nhật cấu hình",
        );
    }

    let current = match ensure_config_exists(&pool, &ws_id).await {
        Ok(c) => c,
        Err(e) => return err_resp_dyn(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    let start_time = dto.start_time.unwrap_or_else(|| current.start_time.clone());
    let timezone = dto.timezone.unwrap_or_else(|| current.timezone.clone());
    let max_concurrency = dto
        .max_concurrency
        .unwrap_or(current.max_concurrency)
        .max(1);
    let consecutive_limit = dto
        .consecutive_failure_limit
        .unwrap_or(current.consecutive_failure_limit)
        .max(1);
    let now = Utc::now().to_rfc3339();

    if let Err(e) = sqlx::query(
        r#"
        UPDATE m_overnight_queue_config
        SET start_time = ?, timezone = ?, max_concurrency = ?,
            consecutive_failure_limit = ?, updated_at = ?
        WHERE workspace_id = ?
        "#,
    )
    .bind(&start_time)
    .bind(&timezone)
    .bind(max_concurrency)
    .bind(consecutive_limit)
    .bind(&now)
    .bind(&ws_id)
    .execute(&pool)
    .await
    {
        return err_resp_dyn(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    let updated = ensure_config_exists(&pool, &ws_id).await.unwrap_or(current);
    Json(json!({ "config": updated })).into_response()
}

/// `POST /workspaces/{ws_id}/queue/pause` - Tạm dừng hàng đợi.
async fn pause_queue(
    AxumPath(ws_id): AxumPath<String>,
    State(state): State<OvernightQueueState>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return r,
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Cần quyền quản trị viên để tạm dừng hàng đợi",
        );
    }

    let now = Utc::now().to_rfc3339();
    let _ = sqlx::query(
        "UPDATE m_overnight_queue_config SET status = 'paused', updated_at = ? WHERE workspace_id = ?",
    )
    .bind(&now)
    .bind(&ws_id)
    .execute(&pool)
    .await;

    Json(json!({ "status": "paused", "success": true })).into_response()
}

/// `POST /workspaces/{ws_id}/queue/resume` - Tiếp tục hàng đợi.
async fn resume_queue(
    AxumPath(ws_id): AxumPath<String>,
    State(state): State<OvernightQueueState>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return r,
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Cần quyền quản trị viên để tiếp tục hàng đợi",
        );
    }

    let now = Utc::now().to_rfc3339();
    let _ = sqlx::query(
        "UPDATE m_overnight_queue_config SET status = 'running', updated_at = ? WHERE workspace_id = ?",
    )
    .bind(&now)
    .bind(&ws_id)
    .execute(&pool)
    .await;

    Json(json!({ "status": "running", "success": true })).into_response()
}

/// `POST /workspaces/{ws_id}/queue/start-now` - Khởi động hàng đợi chạy ngay lập tức.
async fn start_queue_now(
    AxumPath(ws_id): AxumPath<String>,
    State(state): State<OvernightQueueState>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return r,
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Cần quyền quản trị viên để khởi động hàng đợi",
        );
    }

    let _ = ensure_config_exists(&pool, &ws_id).await;
    let now = Utc::now().to_rfc3339();
    let _ = sqlx::query(
        "UPDATE m_overnight_queue_config SET status = 'running', consecutive_failures = 0, updated_at = ? WHERE workspace_id = ?",
    )
    .bind(&now)
    .bind(&ws_id)
    .execute(&pool)
    .await;

    Json(json!({ "status": "running", "success": true })).into_response()
}

/// `GET /workspaces/{ws_id}/queue/summary` - Xem tóm tắt sáng hôm sau ("Chuỗi để duyệt").
async fn get_morning_summary(
    AxumPath(ws_id): AxumPath<String>,
    State(state): State<OvernightQueueState>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return r,
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err(r) => return r,
    };

    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Không có quyền truy cập không gian làm việc này",
        );
    }

    let raw_summary: Option<String> = sqlx::query_scalar(
        "SELECT last_run_summary FROM m_overnight_queue_config WHERE workspace_id = ?",
    )
    .bind(&ws_id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None)
    .flatten();

    let summary: MorningSummaryDto = raw_summary
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    Json(json!({ "summary": summary })).into_response()
}

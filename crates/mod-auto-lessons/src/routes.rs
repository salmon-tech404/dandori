//! Các route HTTP cho module `auto-lessons` (P3-11).

use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::Utc;
use dandori_module_api::{
    ModuleContext, SessionUser, can_access_workspace, can_administer_workspace, session_user,
};
use serde_json::json;
use sqlx::{Row, SqlitePool};

use crate::model::{AutoLessonsConfig, AutoLessonsRunRecord, UpdateAutoLessonsConfigRequest};

/// Trạng thái chia sẻ của router `auto-lessons`.
#[derive(Clone, Debug)]
pub struct AutoLessonsState {
    /// Ngữ cảnh runtime của module.
    pub ctx: ModuleContext,
}

fn err_resp(status: StatusCode, msg: &'static str) -> Response {
    (status, Json(json!({ "error": msg }))).into_response()
}

fn get_pool(state: &AutoLessonsState) -> Result<SqlitePool, (StatusCode, &'static str)> {
    state
        .ctx
        .pool()
        .cloned()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Không kết nối được DB"))
}

async fn get_user(
    headers: &HeaderMap,
    pool: &SqlitePool,
) -> Result<SessionUser, (StatusCode, &'static str)> {
    session_user(pool, headers)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Chưa đăng nhập"))
}

async fn resolve_project_workspace(
    pool: &SqlitePool,
    headers: &HeaderMap,
    project_id: &str,
) -> Result<(SessionUser, String), (StatusCode, &'static str)> {
    let user = get_user(headers, pool).await?;
    let row = sqlx::query("SELECT workspace_id FROM project WHERE id = ?")
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Lỗi truy vấn cơ sở dữ liệu",
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Không tìm thấy dự án"))?;

    let ws_id: String = row.try_get("workspace_id").unwrap_or_default();
    if !can_access_workspace(pool, &user, &ws_id).await {
        return Err((StatusCode::FORBIDDEN, "Không có quyền truy cập workspace"));
    }

    Ok((user, ws_id))
}

async fn get_config_handler(
    State(state): State<AutoLessonsState>,
    AxumPath(project_id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((c, m)) => return err_resp(c, m),
    };
    let (_user, ws_id) = match resolve_project_workspace(&pool, &headers, &project_id).await {
        Ok(res) => res,
        Err((c, m)) => return err_resp(c, m),
    };

    let row: Option<AutoLessonsConfig> = sqlx::query_as(
        "SELECT project_id, workspace_id, enabled, trigger_on_done, trigger_on_failed, \
         max_token_budget, min_content_length, created_at, updated_at \
         FROM m_auto_lessons_config WHERE project_id = ?",
    )
    .bind(&project_id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    let config = row.unwrap_or_else(|| AutoLessonsConfig::default_for(&ws_id, &project_id));
    Json(config).into_response()
}

#[expect(
    clippy::too_many_lines,
    reason = "Xử lý kiểm tra quyền và cập nhật cấu hình dự án"
)]
async fn update_config_handler(
    State(state): State<AutoLessonsState>,
    AxumPath(project_id): AxumPath<String>,
    headers: HeaderMap,
    Json(payload): Json<UpdateAutoLessonsConfigRequest>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((c, m)) => return err_resp(c, m),
    };
    let (user, ws_id) = match resolve_project_workspace(&pool, &headers, &project_id).await {
        Ok(res) => res,
        Err((c, m)) => return err_resp(c, m),
    };

    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Chỉ quản trị viên mới được sửa cấu hình auto-lessons",
        );
    }

    let current_row: Option<AutoLessonsConfig> = sqlx::query_as(
        "SELECT project_id, workspace_id, enabled, trigger_on_done, trigger_on_failed, \
         max_token_budget, min_content_length, created_at, updated_at \
         FROM m_auto_lessons_config WHERE project_id = ?",
    )
    .bind(&project_id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    let current =
        current_row.unwrap_or_else(|| AutoLessonsConfig::default_for(&ws_id, &project_id));
    let new_enabled = payload.enabled.unwrap_or(current.enabled);
    let new_done = payload.trigger_on_done.unwrap_or(current.trigger_on_done);
    let new_failed = payload
        .trigger_on_failed
        .unwrap_or(current.trigger_on_failed);
    let new_budget = payload.max_token_budget.unwrap_or(current.max_token_budget);
    let new_min_len = payload
        .min_content_length
        .unwrap_or(current.min_content_length);
    let now = Utc::now().to_rfc3339();

    let query = r#"
        INSERT INTO m_auto_lessons_config (
            project_id, workspace_id, enabled, trigger_on_done, trigger_on_failed,
            max_token_budget, min_content_length, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(project_id) DO UPDATE SET
            enabled = excluded.enabled,
            trigger_on_done = excluded.trigger_on_done,
            trigger_on_failed = excluded.trigger_on_failed,
            max_token_budget = excluded.max_token_budget,
            min_content_length = excluded.min_content_length,
            updated_at = excluded.updated_at
        RETURNING project_id, workspace_id, enabled, trigger_on_done, trigger_on_failed,
                  max_token_budget, min_content_length, created_at, updated_at
    "#;

    match sqlx::query_as::<_, AutoLessonsConfig>(query)
        .bind(&project_id)
        .bind(&ws_id)
        .bind(new_enabled)
        .bind(new_done)
        .bind(new_failed)
        .bind(new_budget)
        .bind(new_min_len)
        .bind(&current.created_at)
        .bind(&now)
        .fetch_one(&pool)
        .await
    {
        Ok(saved) => Json(saved).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn list_runs_handler(
    State(state): State<AutoLessonsState>,
    AxumPath(project_id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((c, m)) => return err_resp(c, m),
    };
    if let Err((c, m)) = resolve_project_workspace(&pool, &headers, &project_id).await {
        return err_resp(c, m);
    }

    let runs: Vec<AutoLessonsRunRecord> = sqlx::query_as(
        "SELECT id, workspace_id, project_id, task_id, run_id, status, lesson_id, reason, created_at, finished_at \
         FROM m_auto_lessons_run WHERE project_id = ? ORDER BY created_at DESC LIMIT 50",
    )
    .bind(&project_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Json(runs).into_response()
}

/// Khởi tạo Router cho module `auto-lessons`.
pub fn create_routes(state: AutoLessonsState) -> Router {
    Router::new()
        .route(
            "/projects/{project_id}/auto-lessons/config",
            get(get_config_handler).put(update_config_handler),
        )
        .route(
            "/projects/{project_id}/auto-lessons/runs",
            get(list_runs_handler),
        )
        .with_state(state)
}

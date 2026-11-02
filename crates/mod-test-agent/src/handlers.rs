//! Các bộ xử lý HTTP request cho module `test-agent` (P2-04).

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use dandori_module_api::{ModuleContext, SessionUser, can_access_workspace, session_user};
use serde_json::json;
use sqlx::{Row, SqlitePool};

use crate::model::{TestAgentConfig, TestAgentRun, UpdateTestAgentConfigRequest};

/// Trạng thái dùng chung cho các route của module `test-agent`.
#[derive(Clone, Debug)]
pub struct TestAgentState {
    /// Ngữ cảnh runtime của module.
    pub ctx: ModuleContext,
}

fn err_resp(status: StatusCode, msg: &'static str) -> Response {
    (status, Json(json!({ "error": msg }))).into_response()
}

async fn resolve_project_workspace(
    pool: &SqlitePool,
    headers: &HeaderMap,
    project_id: &str,
) -> Result<(SessionUser, String), (StatusCode, &'static str)> {
    let user = session_user(pool, headers)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Chưa xác thực"))?;

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

async fn resolve_task_workspace(
    pool: &SqlitePool,
    headers: &HeaderMap,
    task_id: &str,
) -> Result<(SessionUser, String), (StatusCode, &'static str)> {
    let user = session_user(pool, headers)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Chưa xác thực"))?;

    let row = sqlx::query(
        "SELECT p.workspace_id FROM task t JOIN project p ON t.project_id = p.id WHERE t.id = ?",
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Lỗi truy vấn cơ sở dữ liệu",
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Không tìm thấy task"))?;

    let ws_id: String = row.try_get("workspace_id").unwrap_or_default();
    if !can_access_workspace(pool, &user, &ws_id).await {
        return Err((StatusCode::FORBIDDEN, "Không có quyền truy cập workspace"));
    }

    Ok((user, ws_id))
}

async fn fetch_or_default_config(
    pool: &SqlitePool,
    ws_id: &str,
    project_id: &str,
) -> TestAgentConfig {
    let row: Option<TestAgentConfig> = sqlx::query_as(
        "SELECT project_id, workspace_id, enabled, suggested_command, allowed_path_patterns, \
         max_test_runs_per_worker, timeout_seconds, created_at, updated_at \
         FROM m_test_agent_config WHERE project_id = ?",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    row.unwrap_or_else(|| TestAgentConfig::default_for(ws_id, project_id))
}

fn build_updated_config(
    current: TestAgentConfig,
    ws_id: String,
    project_id: String,
    payload: UpdateTestAgentConfigRequest,
    now: String,
) -> TestAgentConfig {
    let enabled = payload.enabled.unwrap_or(current.enabled);
    let cmd = payload.suggested_command.or(current.suggested_command);
    let patterns = payload
        .allowed_path_patterns
        .and_then(|p| serde_json::to_string(&p).ok())
        .unwrap_or(current.allowed_path_patterns);
    let max_runs = payload
        .max_test_runs_per_worker
        .unwrap_or(current.max_test_runs_per_worker);
    let timeout = payload.timeout_seconds.unwrap_or(current.timeout_seconds);

    TestAgentConfig {
        project_id,
        workspace_id: ws_id,
        enabled,
        suggested_command: cmd,
        allowed_path_patterns: patterns,
        max_test_runs_per_worker: max_runs,
        timeout_seconds: timeout,
        created_at: current.created_at,
        updated_at: now,
    }
}

async fn save_config(pool: &SqlitePool, cfg: &TestAgentConfig) -> Result<(), ()> {
    sqlx::query(
        "INSERT INTO m_test_agent_config (project_id, workspace_id, enabled, suggested_command, \
         allowed_path_patterns, max_test_runs_per_worker, timeout_seconds, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT (project_id) DO UPDATE SET \
         enabled = excluded.enabled, suggested_command = excluded.suggested_command, \
         allowed_path_patterns = excluded.allowed_path_patterns, \
         max_test_runs_per_worker = excluded.max_test_runs_per_worker, \
         timeout_seconds = excluded.timeout_seconds, updated_at = excluded.updated_at",
    )
    .bind(&cfg.project_id)
    .bind(&cfg.workspace_id)
    .bind(cfg.enabled)
    .bind(&cfg.suggested_command)
    .bind(&cfg.allowed_path_patterns)
    .bind(cfg.max_test_runs_per_worker)
    .bind(cfg.timeout_seconds)
    .bind(&cfg.created_at)
    .bind(&cfg.updated_at)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(|_| ())
}

/// `GET /projects/:project_id/config`: Lấy cấu hình test agent của dự án.
pub async fn get_config(
    State(state): State<TestAgentState>,
    AxumPath(project_id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có cơ sở dữ liệu"),
    };

    let (_user, ws_id) = match resolve_project_workspace(pool, &headers, &project_id).await {
        Ok(res) => res,
        Err((code, msg)) => return err_resp(code, msg),
    };

    let config = fetch_or_default_config(pool, &ws_id, &project_id).await;
    Json(json!({ "config": config })).into_response()
}

/// `PUT /projects/:project_id/config`: Cập nhật cấu hình test agent của dự án.
pub async fn update_config(
    State(state): State<TestAgentState>,
    AxumPath(project_id): AxumPath<String>,
    headers: HeaderMap,
    Json(payload): Json<UpdateTestAgentConfigRequest>,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có cơ sở dữ liệu"),
    };

    let (_user, ws_id) = match resolve_project_workspace(pool, &headers, &project_id).await {
        Ok(res) => res,
        Err((code, msg)) => return err_resp(code, msg),
    };

    let current = fetch_or_default_config(pool, &ws_id, &project_id).await;
    let now = Utc::now().to_rfc3339();
    let updated = build_updated_config(current, ws_id, project_id, payload, now);

    if save_config(pool, &updated).await.is_err() {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Lỗi cập nhật cấu hình");
    }

    Json(json!({ "config": updated })).into_response()
}

/// `GET /tasks/:task_id/results`: Danh sách các lần chạy test agent cho task.
pub async fn list_results(
    State(state): State<TestAgentState>,
    AxumPath(task_id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có cơ sở dữ liệu"),
    };

    let (_user, _ws_id) = match resolve_task_workspace(pool, &headers, &task_id).await {
        Ok(res) => res,
        Err((code, msg)) => return err_resp(code, msg),
    };

    let results: Vec<TestAgentRun> = match sqlx::query_as(
        "SELECT id, task_id, project_id, workspace_id, run_id, trigger_run_id, \
         status, exit_code, summary, flagged_files, created_at, finished_at \
         FROM m_test_agent_run WHERE task_id = ? ORDER BY created_at DESC",
    )
    .bind(&task_id)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(_) => {
            return err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Lỗi truy vấn kết quả test",
            );
        }
    };

    Json(json!({ "results": results })).into_response()
}

/// `GET /runs/:run_id/result`: Lấy kết quả test agent theo run_id.
pub async fn get_result(
    State(state): State<TestAgentState>,
    AxumPath(run_id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let pool = match state.ctx.pool() {
        Some(p) => p,
        None => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Không có cơ sở dữ liệu"),
    };

    let user = match session_user(pool, &headers).await {
        Some(u) => u,
        None => return err_resp(StatusCode::UNAUTHORIZED, "Chưa xác thực"),
    };

    let run_res: Option<TestAgentRun> = match sqlx::query_as(
        "SELECT id, task_id, project_id, workspace_id, run_id, trigger_run_id, \
         status, exit_code, summary, flagged_files, created_at, finished_at \
         FROM m_test_agent_run WHERE run_id = ?",
    )
    .bind(&run_id)
    .fetch_optional(pool)
    .await
    {
        Ok(r) => r,
        Err(_) => {
            return err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Lỗi truy vấn cơ sở dữ liệu",
            );
        }
    };

    let Some(run) = run_res else {
        return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy kết quả test");
    };

    if !can_access_workspace(pool, &user, &run.workspace_id).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    Json(json!({ "result": run })).into_response()
}

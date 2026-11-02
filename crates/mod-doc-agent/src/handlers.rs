//! Các bộ xử lý HTTP request cho module `doc-agent` (P2-05).

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use dandori_module_api::{ModuleContext, SessionUser, can_access_workspace, session_user};
use serde_json::json;
use sqlx::{Row, SqlitePool};

use crate::model::{DocAgentConfig, DocAgentRun, UpdateDocAgentConfigRequest};

/// Trạng thái dùng chung cho các route của module `doc-agent`.
#[derive(Clone, Debug)]
pub struct DocAgentState {
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
) -> DocAgentConfig {
    let row: Option<DocAgentConfig> = sqlx::query_as(
        "SELECT project_id, workspace_id, enabled, allowed_path_patterns, prompt_template, \
         timeout_seconds, created_at, updated_at \
         FROM m_doc_agent_config WHERE project_id = ?",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    row.unwrap_or_else(|| DocAgentConfig::default_for(ws_id, project_id))
}

fn build_updated_config(
    current: DocAgentConfig,
    ws_id: String,
    project_id: String,
    payload: UpdateDocAgentConfigRequest,
    now: String,
) -> DocAgentConfig {
    let enabled = payload.enabled.unwrap_or(current.enabled);
    let patterns = payload
        .allowed_path_patterns
        .and_then(|p| serde_json::to_string(&p).ok())
        .unwrap_or(current.allowed_path_patterns);
    let prompt = payload.prompt_template.or(current.prompt_template);
    let timeout = payload.timeout_seconds.unwrap_or(current.timeout_seconds);

    DocAgentConfig {
        project_id,
        workspace_id: ws_id,
        enabled,
        allowed_path_patterns: patterns,
        prompt_template: prompt,
        timeout_seconds: timeout,
        created_at: current.created_at,
        updated_at: now,
    }
}

async fn save_config(pool: &SqlitePool, cfg: &DocAgentConfig) -> Result<(), ()> {
    sqlx::query(
        "INSERT INTO m_doc_agent_config (project_id, workspace_id, enabled, \
         allowed_path_patterns, prompt_template, timeout_seconds, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT (project_id) DO UPDATE SET \
         enabled = excluded.enabled, allowed_path_patterns = excluded.allowed_path_patterns, \
         prompt_template = excluded.prompt_template, timeout_seconds = excluded.timeout_seconds, \
         updated_at = excluded.updated_at",
    )
    .bind(&cfg.project_id)
    .bind(&cfg.workspace_id)
    .bind(cfg.enabled)
    .bind(&cfg.allowed_path_patterns)
    .bind(&cfg.prompt_template)
    .bind(cfg.timeout_seconds)
    .bind(&cfg.created_at)
    .bind(&cfg.updated_at)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(|_| ())
}

/// `GET /projects/:project_id/config`: Lấy cấu hình doc agent của dự án.
pub async fn get_config(
    State(state): State<DocAgentState>,
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

/// `PUT /projects/:project_id/config`: Cập nhật cấu hình doc agent của dự án.
pub async fn update_config(
    State(state): State<DocAgentState>,
    AxumPath(project_id): AxumPath<String>,
    headers: HeaderMap,
    Json(payload): Json<UpdateDocAgentConfigRequest>,
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

/// `GET /tasks/:task_id/results`: Danh sách các lần chạy doc agent cho task.
pub async fn list_results(
    State(state): State<DocAgentState>,
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

    let results: Vec<DocAgentRun> = match sqlx::query_as(
        "SELECT id, task_id, project_id, workspace_id, run_id, status, \
         summary, changed_doc_files, flagged_files, created_at, finished_at \
         FROM m_doc_agent_run WHERE task_id = ? ORDER BY created_at DESC",
    )
    .bind(&task_id)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(_) => {
            return err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Lỗi truy vấn kết quả tài liệu",
            );
        }
    };

    Json(json!({ "results": results })).into_response()
}

/// `GET /runs/:run_id/result`: Lấy kết quả doc agent theo run_id.
pub async fn get_result(
    State(state): State<DocAgentState>,
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

    let run_res: Option<DocAgentRun> = match sqlx::query_as(
        "SELECT id, task_id, project_id, workspace_id, run_id, status, \
         summary, changed_doc_files, flagged_files, created_at, finished_at \
         FROM m_doc_agent_run WHERE run_id = ?",
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
        return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy kết quả tài liệu");
    };

    if !can_access_workspace(pool, &user, &run.workspace_id).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    Json(json!({ "result": run })).into_response()
}

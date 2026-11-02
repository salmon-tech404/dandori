//! Các bộ xử lý HTTP request cho module `diff-preview`.

use axum::Json;
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::json;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;

use crate::manager::PreviewManager;
use crate::model::{
    DiffQuery, StartPreviewRequest, validate_preview_command, validate_preview_host,
    validate_preview_port,
};
use dandori_module_api::{GitClient, ModuleContext, can_access_workspace, session_user};

/// Trạng thái chia sẻ giữa các router của `diff-preview`.
#[derive(Clone, Debug)]
pub struct DiffPreviewState {
    /// Ngữ cảnh module runtime.
    pub ctx: ModuleContext,
    /// Trình quản lý tiến trình xem trước.
    pub manager: Arc<PreviewManager>,
}

struct TaskInfo {
    workspace_id: String,
    repo_path: PathBuf,
    default_branch: String,
    workdir_path: Option<PathBuf>,
}

async fn fetch_task_info(
    pool: &SqlitePool,
    task_id: &str,
) -> Result<Option<TaskInfo>, sqlx::Error> {
    let row: Option<(Option<String>, String, String, String)> = sqlx::query_as(
        "SELECT t.workdir_path, p.repo_path, p.default_branch, p.workspace_id \
         FROM task t \
         JOIN project p ON t.project_id = p.id \
         WHERE t.id = ?",
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(
        |(workdir_path, repo_path, default_branch, workspace_id)| TaskInfo {
            workspace_id,
            repo_path: PathBuf::from(repo_path),
            default_branch,
            workdir_path: workdir_path.map(PathBuf::from),
        },
    ))
}

async fn verify_access(
    pool: &SqlitePool,
    headers: &HeaderMap,
    workspace_id: &str,
) -> Result<(), (StatusCode, &'static str)> {
    let Some(user) = session_user(pool, headers).await else {
        return Err((StatusCode::UNAUTHORIZED, "Chưa đăng nhập"));
    };

    if !can_access_workspace(pool, &user, workspace_id).await {
        return Err((StatusCode::FORBIDDEN, "Không có quyền truy cập workspace"));
    }
    Ok(())
}

fn get_pool(ctx: &ModuleContext) -> Result<SqlitePool, (StatusCode, &'static str)> {
    ctx.pool()
        .cloned()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "DB không sẵn sàng"))
}

/// Lấy diff chi tiết giữa worktree của task và nhánh cơ sở.
pub async fn get_diff(
    State(state): State<DiffPreviewState>,
    AxumPath(task_id): AxumPath<String>,
    Query(query): Query<DiffQuery>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state.ctx) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };

    let info = match fetch_task_info(&pool, &task_id).await {
        Ok(Some(i)) => i,
        Ok(None) => return (StatusCode::NOT_FOUND, "Không tìm thấy task").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    if let Err(res) = verify_access(&pool, &headers, &info.workspace_id).await {
        return res.into_response();
    }

    let workdir = match info.workdir_path {
        Some(w) if w.exists() => w,
        _ => return (StatusCode::BAD_REQUEST, "Task chưa có worktree").into_response(),
    };

    let base_ref = query.base_ref.unwrap_or(info.default_branch);
    let client = GitClient::new(&info.repo_path);
    match client.diff_structured(&workdir, &base_ref, 50_000).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn record_preview_session(pool: &SqlitePool, task_id: &str, command: &[String], port: u16) {
    let session_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let cmd_str = serde_json::to_string(command).unwrap_or_default();
    let port_i64 = i64::from(port);
    let _ = sqlx::query(
        "INSERT INTO m_diff_preview_session (id, task_id, command, port, host, status, started_at) \
         VALUES (?, ?, ?, ?, '127.0.0.1', 'running', ?)",
    )
    .bind(&session_id)
    .bind(task_id)
    .bind(&cmd_str)
    .bind(port_i64)
    .bind(&now)
    .execute(pool)
    .await;
}

fn validate_start_request(req: &StartPreviewRequest) -> Result<u16, (StatusCode, &'static str)> {
    if !req.confirmed {
        return Err((
            StatusCode::BAD_REQUEST,
            "Cần xác nhận trước khi khởi chạy bản xem trước",
        ));
    }
    let port = req.port.unwrap_or(3000);
    validate_preview_port(port).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    validate_preview_host("127.0.0.1").map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(port)
}

async fn get_workdir_and_verify(
    state: &DiffPreviewState,
    task_id: &str,
    headers: &HeaderMap,
) -> Result<(SqlitePool, PathBuf), (StatusCode, &'static str)> {
    let pool = get_pool(&state.ctx)?;
    let info = match fetch_task_info(&pool, task_id).await {
        Ok(Some(i)) => i,
        Ok(None) => return Err((StatusCode::NOT_FOUND, "Không tìm thấy task")),
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Lỗi truy vấn cơ sở dữ liệu",
            ));
        }
    };
    verify_access(&pool, headers, &info.workspace_id).await?;
    let workdir = match info.workdir_path {
        Some(w) if w.exists() => w,
        _ => return Err((StatusCode::BAD_REQUEST, "Task chưa có worktree")),
    };
    Ok((pool, workdir))
}

/// Khởi chạy tiến trình xem trước cho task.
pub async fn start_preview(
    State(state): State<DiffPreviewState>,
    AxumPath(task_id): AxumPath<String>,
    headers: HeaderMap,
    Json(req): Json<StartPreviewRequest>,
) -> Response {
    let port = match validate_start_request(&req) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };

    let (pool, workdir) = match get_workdir_and_verify(&state, &task_id, &headers).await {
        Ok(res) => res,
        Err(e) => return e.into_response(),
    };

    // Chạy một chương trình trên máy chủ là quyền của quản trị hệ thống (chủ dự án), không phải của mọi
    // thành viên workspace; đồng thời cấm shell để tham số không bị diễn giải.
    match session_user(&pool, &headers).await {
        Some(user) if user.is_system_admin() => {}
        _ => {
            return (
                StatusCode::FORBIDDEN,
                "Chỉ quản trị hệ thống được chạy lệnh xem trước",
            )
                .into_response();
        }
    }

    if let Some(ref cmd) = req.command
        && let Err(e) = validate_preview_command(cmd)
    {
        return (StatusCode::BAD_REQUEST, e).into_response();
    }

    let command = match req.command {
        Some(cmd) if !cmd.is_empty() => cmd,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                "Chưa cấu hình lệnh xem trước (preview_command)",
            )
                .into_response();
        }
    };

    match state
        .manager
        .start_preview(&task_id, &workdir, command.clone(), port)
        .await
    {
        Ok(status) => {
            record_preview_session(&pool, &task_id, &command, port).await;
            Json(status).into_response()
        }
        Err(err) if err == "Đang có bản xem trước chạy cho task này" => {
            (StatusCode::CONFLICT, err).into_response()
        }
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

/// Lấy trạng thái hiện tại của bản xem trước cho task.
pub async fn get_preview_status(
    State(state): State<DiffPreviewState>,
    AxumPath(task_id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state.ctx) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };

    let info = match fetch_task_info(&pool, &task_id).await {
        Ok(Some(i)) => i,
        Ok(None) => return (StatusCode::NOT_FOUND, "Không tìm thấy task").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    if let Err(res) = verify_access(&pool, &headers, &info.workspace_id).await {
        return res.into_response();
    }

    let status = state.manager.get_status(&task_id).await;
    Json(status).into_response()
}

/// Dừng phiên xem trước của task.
pub async fn stop_preview(
    State(state): State<DiffPreviewState>,
    AxumPath(task_id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state.ctx) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };

    let info = match fetch_task_info(&pool, &task_id).await {
        Ok(Some(i)) => i,
        Ok(None) => return (StatusCode::NOT_FOUND, "Không tìm thấy task").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    if let Err(res) = verify_access(&pool, &headers, &info.workspace_id).await {
        return res.into_response();
    }

    let stopped = state.manager.stop_preview(&task_id).await;
    let now = chrono::Utc::now().to_rfc3339();
    let _ = sqlx::query(
        "UPDATE m_diff_preview_session SET status = 'stopped', stopped_at = ? WHERE task_id = ? AND status = 'running'",
    )
    .bind(&now)
    .bind(&task_id)
    .execute(&pool)
    .await;

    Json(json!({ "stopped": stopped })).into_response()
}

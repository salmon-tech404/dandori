//! Module `worktree`: Quản lý git worktree và branch riêng cho từng nhiệm vụ (P1-04).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use dandori_module_api::{
    BoxFuture, GitClient, Manifest, Migration, Module, ModuleContext, ModuleEffort, ModuleGroup,
    ModuleRisk, ModuleScope, ModuleSubscription, ReleaseReason, UiContribution, UiDeclaration,
    Workdir, WorkdirError, WorkdirProvider, WorkerSpec, hash_session_token, sanitize_branch_name,
};
use dandori_types::Task;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool};

fn path_equals(a: &Path, b: &Path) -> bool {
    if let (Ok(ca), Ok(cb)) = (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        return ca == cb;
    }
    let sa = a.to_string_lossy().replace('/', "\\").to_lowercase();
    let sb = b.to_string_lossy().replace('/', "\\").to_lowercase();
    sa.trim_end_matches('\\') == sb.trim_end_matches('\\')
}

fn get_manifest() -> Manifest {
    Manifest {
        id: "worktree".to_string(),
        group: ModuleGroup::Runtime,
        name: "Git worktree cho task".to_string(),
        description: "Môi trường git riêng biệt cho từng nhiệm vụ".to_string(),
        version: "0.1.0".to_string(),
        requires: vec![],
        scopes: vec![ModuleScope::Workspace, ModuleScope::Project],
        effort: ModuleEffort::Medium,
        risk: ModuleRisk::Low,
        risk_note: None,
        core: false,
        requires_worker_restart: false,
        settings: std::collections::HashMap::new(),
        ui: UiDeclaration {
            contributes: vec![UiContribution {
                id: "worktree-info".to_string(),
                slot: "board.task-panel.section".to_string(),
                label: "Chạy ở đâu".to_string(),
                order: 20,
            }],
        },
    }
}

/// Hiện thực `WorkdirProvider` cho module `worktree`.
#[derive(Debug, Clone)]
pub struct GitWorkdirProvider {
    pool: Option<SqlitePool>,
    data_dir: PathBuf,
}

impl GitWorkdirProvider {
    /// Khởi tạo provider với pool và data_dir.
    #[must_use]
    pub fn new(pool: Option<SqlitePool>, data_dir: PathBuf) -> Self {
        Self { pool, data_dir }
    }

    fn resolve_worktree_dir(
        &self,
        project_id: &str,
        task_id: &str,
    ) -> Result<PathBuf, WorkdirError> {
        let base = self.data_dir.join("worktrees");
        if project_id.contains("..") || task_id.contains("..") {
            return Err(WorkdirError::PrepareFailed(
                "Đường dẫn chứa ký tự thoát thư mục không hợp lệ (RS-SEC-004)".to_string(),
            ));
        }
        let target = base.join(project_id).join(task_id);
        Ok(target)
    }
}

async fn get_project_repo_info(
    pool: &SqlitePool,
    proj_str: &str,
) -> Result<(PathBuf, String), WorkdirError> {
    let proj_row = sqlx::query("SELECT repo_path, default_branch FROM project WHERE id = ?")
        .bind(proj_str)
        .fetch_optional(pool)
        .await
        .map_err(|e| WorkdirError::PrepareFailed(e.to_string()))?
        .ok_or_else(|| WorkdirError::PrepareFailed("Không tìm thấy dự án".to_string()))?;

    let repo_path_str: String = proj_row.try_get("repo_path").unwrap_or_default();
    let default_branch: String = proj_row
        .try_get("default_branch")
        .unwrap_or_else(|_| "main".to_string());

    let repo_path = PathBuf::from(&repo_path_str);
    if !repo_path.exists() || !repo_path.join(".git").exists() {
        return Err(WorkdirError::PrepareFailed(format!(
            "Đường dẫn repo không tồn tại hoặc không phải git repo: {repo_path_str}"
        )));
    }
    Ok((repo_path, default_branch))
}

async fn determine_branch_name(
    client: &GitClient,
    task: &Task,
    task_str: &str,
) -> Result<String, WorkdirError> {
    let short_id = if task_str.len() >= 8 {
        &task_str[..8]
    } else {
        task_str
    };
    let mut branch_name = sanitize_branch_name(&task.title, short_id)
        .map_err(|e| WorkdirError::PrepareFailed(e.to_string()))?;

    if client.branch_exists(&branch_name).await.unwrap_or(false) {
        let uuid_suffix = uuid::Uuid::new_v4().to_string();
        let short_uuid = if uuid_suffix.len() >= 4 {
            &uuid_suffix[..4]
        } else {
            &uuid_suffix
        };
        branch_name = format!("{branch_name}-{short_uuid}");
    }
    Ok(branch_name)
}

async fn ensure_worktree(
    client: &GitClient,
    wt_dir: &Path,
    branch_name: &str,
    default_branch: &str,
) -> Result<(), WorkdirError> {
    if wt_dir.exists() {
        let list = client
            .worktree_list()
            .await
            .map_err(|e| WorkdirError::PrepareFailed(e.to_string()))?;
        let is_valid = list.iter().any(|w| path_equals(&w.path, wt_dir));
        if !is_valid {
            let _ = tokio::fs::remove_dir_all(wt_dir).await;
            client
                .worktree_add(wt_dir, branch_name, default_branch)
                .await
                .map_err(|e| WorkdirError::PrepareFailed(e.to_string()))?;
        }
    } else {
        client
            .worktree_add(wt_dir, branch_name, default_branch)
            .await
            .map_err(|e| WorkdirError::PrepareFailed(e.to_string()))?;
    }
    Ok(())
}

impl WorkdirProvider for GitWorkdirProvider {
    fn name(&self) -> &'static str {
        "worktree"
    }

    fn prepare(&self, task: &Task) -> BoxFuture<'static, Result<Workdir, WorkdirError>> {
        let pool = self.pool.clone();
        let task_clone = task.clone();
        let this = self.clone();

        Box::pin(async move {
            let pool = pool.ok_or_else(|| {
                WorkdirError::PrepareFailed("Thiếu kết nối cơ sở dữ liệu".to_string())
            })?;

            let proj_str = task_clone.project_id.to_string();
            let task_str = task_clone.id.to_string();

            let (repo_path, default_branch) = get_project_repo_info(&pool, &proj_str).await?;
            let client = GitClient::new(&repo_path);

            if !client
                .has_commits()
                .await
                .map_err(|e| WorkdirError::PrepareFailed(e.to_string()))?
            {
                return Err(WorkdirError::PrepareFailed(
                    "Kho git chưa có commit nào. Vui lòng tạo commit ban đầu trước khi giao việc."
                        .to_string(),
                ));
            }

            let wt_dir = this.resolve_worktree_dir(&proj_str, &task_str)?;
            let branch_name = determine_branch_name(&client, &task_clone, &task_str).await?;
            ensure_worktree(&client, &wt_dir, &branch_name, &default_branch).await?;

            Ok(Workdir::new(wt_dir))
        })
    }

    fn release(
        &self,
        task: &Task,
        reason: ReleaseReason,
    ) -> BoxFuture<'static, Result<(), WorkdirError>> {
        let pool = self.pool.clone();
        let task_clone = task.clone();
        let this = self.clone();

        Box::pin(async move {
            // Theo tiêu chí nghiệm thu P1-04: chỉ xóa khi người dùng xác nhận (ReleaseReason::Cleanup)
            if reason != ReleaseReason::Cleanup {
                return Ok(());
            }

            let pool = pool.ok_or_else(|| {
                WorkdirError::ReleaseFailed("Thiếu kết nối cơ sở dữ liệu".to_string())
            })?;

            let proj_str = task_clone.project_id.to_string();
            let task_str = task_clone.id.to_string();

            let wt_dir = this.resolve_worktree_dir(&proj_str, &task_str)?;
            if !wt_dir.exists() {
                return Ok(());
            }

            let proj_row = sqlx::query("SELECT repo_path FROM project WHERE id = ?")
                .bind(&proj_str)
                .fetch_optional(&pool)
                .await
                .map_err(|e| WorkdirError::ReleaseFailed(e.to_string()))?
                .ok_or_else(|| WorkdirError::ReleaseFailed("Không tìm thấy dự án".to_string()))?;

            let repo_path_str: String = proj_row.try_get("repo_path").unwrap_or_default();
            let client = GitClient::new(PathBuf::from(repo_path_str));

            // Kiểm tra dirty
            let dirty = client
                .status_porcelain(&wt_dir)
                .await
                .map_err(|e| WorkdirError::ReleaseFailed(e.to_string()))?;

            if !dirty.is_empty() {
                return Err(WorkdirError::ReleaseFailed(
                    "Worktree có thay đổi chưa commit. Hãy xác nhận hủy hoặc commit trước khi dọn dẹp."
                        .to_string(),
                ));
            }

            client
                .worktree_remove(&wt_dir, false)
                .await
                .map_err(|e| WorkdirError::ReleaseFailed(e.to_string()))?;

            Ok(())
        })
    }
}

/// Struct module `worktree` tích hợp vào Dandori.
#[derive(Debug, Clone, Default)]
pub struct WorktreeModule;

impl WorktreeModule {
    /// Khởi tạo module mới.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for WorktreeModule {
    fn manifest(&self) -> &Manifest {
        static MANIFEST: std::sync::OnceLock<Manifest> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(get_manifest)
    }

    fn migrations(&self) -> &'static [Migration] {
        &[]
    }

    fn routes(&self, ctx: &ModuleContext) -> Router {
        let Some(pool) = ctx.pool().cloned() else {
            return Router::new();
        };

        let data_dir = ctx.data_dir();
        let state = Arc::new(WorktreeState { pool, data_dir });

        Router::new()
            .route("/tasks/{task_id}/worktree", get(get_task_worktree))
            .route("/tasks/{task_id}/cleanup", post(cleanup_task_worktree))
            .with_state(state)
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        vec![]
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        vec![]
    }

    fn workdir_provider(&self, ctx: &ModuleContext) -> Option<Arc<dyn WorkdirProvider>> {
        let data_dir = ctx.data_dir();
        Some(Arc::new(GitWorkdirProvider::new(
            ctx.pool().cloned(),
            data_dir,
        )))
    }
}

#[derive(Clone, Debug)]
struct WorktreeState {
    pool: SqlitePool,
    data_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct CleanupRequest {
    #[serde(default)]
    force: bool,
}

async fn extract_authenticated_user(
    pool: &SqlitePool,
    headers: &HeaderMap,
) -> Result<String, (StatusCode, &'static str)> {
    let cookie_header = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let mut session_token = None;
    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if let Some(token) = cookie.strip_prefix("dandori_session=") {
            session_token = Some(token);
            break;
        }
    }

    let token = session_token.ok_or((StatusCode::UNAUTHORIZED, "Chưa đăng nhập"))?;
    let token_hash = hash_session_token(token);

    let session_row = sqlx::query("SELECT user_id, expires_at FROM session WHERE token_hash = ?")
        .bind(&token_hash)
        .fetch_optional(pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi truy vấn phiên"))?
        .ok_or((StatusCode::UNAUTHORIZED, "Phiên không hợp lệ"))?;

    let expires_at_str: String = session_row.try_get("expires_at").unwrap_or_default();
    if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(&expires_at_str)
        && exp < Utc::now()
    {
        return Err((StatusCode::UNAUTHORIZED, "Phiên đã hết hạn"));
    }

    Ok(session_row.try_get("user_id").unwrap_or_default())
}

async fn check_task_access(
    pool: &SqlitePool,
    headers: &HeaderMap,
    task_id: &str,
) -> Result<(String, String, String), (StatusCode, &'static str)> {
    let user_id = extract_authenticated_user(pool, headers).await?;

    let task_row = sqlx::query("SELECT project_id FROM task WHERE id = ?")
        .bind(task_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi truy vấn task"))?
        .ok_or((StatusCode::NOT_FOUND, "Không tìm thấy nhiệm vụ"))?;

    let project_id: String = task_row.try_get("project_id").unwrap_or_default();

    let proj_row = sqlx::query("SELECT workspace_id, repo_path FROM project WHERE id = ?")
        .bind(&project_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi truy vấn project"))?
        .ok_or((StatusCode::NOT_FOUND, "Không tìm thấy dự án"))?;

    let workspace_id: String = proj_row.try_get("workspace_id").unwrap_or_default();
    let repo_path: String = proj_row.try_get("repo_path").unwrap_or_default();

    let member =
        sqlx::query("SELECT role FROM workspace_member WHERE workspace_id = ? AND user_id = ?")
            .bind(&workspace_id)
            .bind(&user_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi kiểm tra quyền"))?;

    if member.is_none() {
        return Err((StatusCode::FORBIDDEN, "Không có quyền truy cập workspace"));
    }

    Ok((user_id, project_id, repo_path))
}

async fn get_task_worktree(
    State(state): State<Arc<WorktreeState>>,
    headers: HeaderMap,
    AxumPath(task_id): AxumPath<String>,
) -> Response {
    let (_, project_id, repo_path) = match check_task_access(&state.pool, &headers, &task_id).await
    {
        Ok(res) => res,
        Err((code, msg)) => {
            return (
                code,
                Json(json!({ "error": { "code": "forbidden", "message": msg } })),
            )
                .into_response();
        }
    };

    let wt_dir = state
        .data_dir
        .join("worktrees")
        .join(&project_id)
        .join(&task_id);

    if !wt_dir.exists() {
        return Json(json!({
            "exists": false,
            "workdir_path": null,
            "branch": null,
            "is_dirty": false,
            "dirty_files": []
        }))
        .into_response();
    }

    let client = GitClient::new(PathBuf::from(&repo_path));
    let dirty_files = client.status_porcelain(&wt_dir).await.unwrap_or_default();

    let is_dirty = !dirty_files.is_empty();

    let wt_list = client.worktree_list().await.unwrap_or_default();
    let branch = wt_list
        .iter()
        .find(|w| path_equals(&w.path, &wt_dir))
        .and_then(|w| w.branch.clone());

    Json(json!({
        "exists": true,
        "workdir_path": wt_dir.to_string_lossy(),
        "branch": branch,
        "is_dirty": is_dirty,
        "dirty_files": dirty_files
    }))
    .into_response()
}

async fn perform_worktree_cleanup(
    client: &GitClient,
    wt_dir: &Path,
    force: bool,
) -> Result<(), (StatusCode, Value)> {
    if !force {
        let dirty = client.status_porcelain(wt_dir).await.unwrap_or_default();
        if !dirty.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                json!({
                    "error": {
                        "code": "worktree_dirty",
                        "message": "Worktree có thay đổi chưa commit. Xác nhận force để dọn dẹp.",
                        "dirty_files": dirty
                    }
                }),
            ));
        }
    }

    if let Err(e) = client.worktree_remove(wt_dir, force).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({
                "error": {
                    "code": "cleanup_failed",
                    "message": format!("Không thể dọn dẹp worktree: {e}")
                }
            }),
        ));
    }

    Ok(())
}

async fn cleanup_task_worktree(
    State(state): State<Arc<WorktreeState>>,
    headers: HeaderMap,
    AxumPath(task_id): AxumPath<String>,
    Json(payload): Json<CleanupRequest>,
) -> Response {
    let (_, project_id, repo_path) = match check_task_access(&state.pool, &headers, &task_id).await
    {
        Ok(res) => res,
        Err((code, msg)) => {
            return (
                code,
                Json(json!({ "error": { "code": "forbidden", "message": msg } })),
            )
                .into_response();
        }
    };

    let wt_dir = state
        .data_dir
        .join("worktrees")
        .join(&project_id)
        .join(&task_id);

    if !wt_dir.exists() {
        return Json(json!({ "success": true, "message": "Worktree không tồn tại" }))
            .into_response();
    }

    let client = GitClient::new(PathBuf::from(&repo_path));
    if let Err((code, err_val)) = perform_worktree_cleanup(&client, &wt_dir, payload.force).await {
        return (code, Json(err_val)).into_response();
    }

    // Cập nhật workdir_path thành NULL trong DB
    let _ = sqlx::query("UPDATE task SET workdir_path = NULL WHERE id = ?")
        .bind(&task_id)
        .execute(&state.pool)
        .await;

    Json(json!({ "success": true, "message": "Đã dọn dẹp worktree thành công" })).into_response()
}

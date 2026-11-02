//! Handlers xử lý HTTP REST API cho module prompt-library.

use axum::Json;
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use dandori_module_api::{
    SessionUser, accessible_workspace_ids, can_access_workspace, session_user,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::template::{check_length, check_secrets, render_template, validate_variables};

/// Trạng thái ứng dụng của route prompt-library.
#[derive(Clone, Debug)]
pub struct AppState {
    /// Pool cơ sở dữ liệu.
    pub pool: Option<SqlitePool>,
}

fn error_response(code: &str, message: &str, status: StatusCode) -> Response {
    let req_id = Uuid::new_v4().to_string();
    (
        status,
        Json(json!({
            "error": {
                "code": code,
                "message": message,
                "request_id": req_id
            }
        })),
    )
        .into_response()
}

fn boxed_error(code: &str, message: &str, status: StatusCode) -> Box<Response> {
    Box::new(error_response(code, message, status))
}

/// Yêu cầu có phiên hợp lệ và quyền truy cập workspace.
async fn authorize_workspace(
    pool: &SqlitePool,
    headers: &HeaderMap,
    workspace_id: &str,
) -> Result<SessionUser, Box<Response>> {
    let user = session_user(pool, headers).await.ok_or_else(|| {
        boxed_error(
            "unauthorized",
            "Phiên đăng nhập không hợp lệ hoặc đã hết hạn",
            StatusCode::UNAUTHORIZED,
        )
    })?;
    if !can_access_workspace(pool, &user, workspace_id).await {
        return Err(boxed_error(
            "forbidden",
            "Không có quyền truy cập không gian làm việc này",
            StatusCode::FORBIDDEN,
        ));
    }
    Ok(user)
}

/// Yêu cầu quyền trên workspace chứa prompt `prompt_id` (404 nếu prompt không tồn tại).
async fn authorize_prompt(
    pool: &SqlitePool,
    headers: &HeaderMap,
    prompt_id: &str,
) -> Result<SessionUser, Box<Response>> {
    let ws: Option<String> =
        sqlx::query_scalar("SELECT workspace_id FROM m_prompt_library_prompt WHERE id = ?")
            .bind(prompt_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
    match ws {
        Some(ws) => authorize_workspace(pool, headers, &ws).await,
        None => {
            // Không lộ việc prompt có tồn tại hay không cho người chưa đăng nhập.
            if session_user(pool, headers).await.is_none() {
                return Err(boxed_error(
                    "unauthorized",
                    "Phiên đăng nhập không hợp lệ hoặc đã hết hạn",
                    StatusCode::UNAUTHORIZED,
                ));
            }
            Err(boxed_error(
                "not_found",
                "Không tìm thấy prompt",
                StatusCode::NOT_FOUND,
            ))
        }
    }
}

/// Phạm vi workspace được liệt kê: có `workspace_id` thì phải có quyền trên nó; không có thì chỉ
/// các workspace người dùng thuộc về (`None` = tất cả, dành cho quản trị hệ thống).
async fn list_scope(
    pool: &SqlitePool,
    headers: &HeaderMap,
    ws_id: &str,
) -> Result<Option<Vec<String>>, Box<Response>> {
    let user = if ws_id.is_empty() {
        session_user(pool, headers).await.ok_or_else(|| {
            boxed_error(
                "unauthorized",
                "Phiên đăng nhập không hợp lệ hoặc đã hết hạn",
                StatusCode::UNAUTHORIZED,
            )
        })?
    } else {
        authorize_workspace(pool, headers, ws_id).await?
    };
    Ok(accessible_workspace_ids(pool, &user).await)
}

/// Tham số lọc danh sách prompt.
#[derive(Debug, Deserialize, Default)]
pub struct ListPromptsQuery {
    /// Không gian làm việc.
    pub workspace_id: Option<String>,
    /// Dự án.
    pub project_id: Option<String>,
    /// Vai trò agent.
    pub role: Option<String>,
}

/// DTO tạo mới prompt.
#[derive(Debug, Deserialize)]
pub struct CreatePromptRequest {
    /// Không gian làm việc.
    pub workspace_id: String,
    /// Dự án (nếu có).
    pub project_id: Option<String>,
    /// Tên prompt.
    pub name: String,
    /// Vai trò agent gắn kèm (pm, worker, test, doc, hoặc để trống).
    pub role: Option<String>,
    /// Nhãn kỹ năng (skill tag).
    pub skill_tag: Option<String>,
    /// Nội dung chỉ thị ban đầu (v1).
    pub content: String,
}

/// DTO cập nhật prompt (tạo phiên bản mới).
#[derive(Debug, Deserialize)]
pub struct UpdatePromptRequest {
    /// Tên prompt mới nếu có đổi.
    pub name: Option<String>,
    /// Vai trò agent nếu có đổi.
    pub role: Option<String>,
    /// Nhãn kỹ năng nếu có đổi.
    pub skill_tag: Option<String>,
    /// Nội dung mới tạo phiên bản kế tiếp.
    pub content: String,
}

/// DTO thử kết xuất prompt không gọi agent.
#[derive(Debug, Deserialize)]
pub struct TestRenderRequest {
    /// Nội dung mẫu prompt cần thử.
    pub content: String,
    /// Tiêu đề task mẫu.
    pub task_title: Option<String>,
    /// Mô tả task mẫu.
    pub task_description: Option<String>,
    /// Tên dự án mẫu.
    pub project_name: Option<String>,
    /// Vai trò agent mẫu.
    pub role: Option<String>,
}

fn validate_prompt_content(content: &str) -> Result<(), Box<Response>> {
    if let Err(msg) = check_length(content) {
        return Err(boxed_error("length_exceeded", msg, StatusCode::BAD_REQUEST));
    }
    if let Err(msg) = check_secrets(content) {
        return Err(boxed_error("secret_detected", msg, StatusCode::BAD_REQUEST));
    }
    if let Err(msg) = validate_variables(content) {
        return Err(boxed_error(
            "invalid_variables",
            &msg,
            StatusCode::BAD_REQUEST,
        ));
    }
    Ok(())
}

/// Handler xử lý `GET /prompts`.
pub async fn list_prompts_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListPromptsQuery>,
) -> Response {
    let pool = match state.pool {
        Some(ref p) => p,
        None => return Json(json!({ "prompts": [] })).into_response(),
    };

    let ws_id = query.workspace_id.unwrap_or_default();
    let allowed = match list_scope(pool, &headers, &ws_id).await {
        Ok(a) => a,
        Err(resp) => return *resp,
    };
    let rows = match sqlx::query(
        "SELECT id, workspace_id, project_id, name, role, skill_tag, current_version, created_at, updated_at
         FROM m_prompt_library_prompt
         WHERE workspace_id = ? OR ? = ''
         ORDER BY updated_at DESC",
    )
    .bind(&ws_id)
    .bind(&ws_id)
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => return error_response("db_error", &e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    };

    let prompts: Vec<serde_json::Value> = rows
        .into_iter()
        .filter(|r| {
            allowed
                .as_ref()
                .is_none_or(|ids| ids.contains(&r.get::<String, _>(1)))
        })
        .map(|r| {
            json!({
                "id": r.get::<String, _>(0),
                "workspace_id": r.get::<String, _>(1),
                "project_id": r.get::<Option<String>, _>(2),
                "name": r.get::<String, _>(3),
                "role": r.get::<Option<String>, _>(4),
                "skill_tag": r.get::<Option<String>, _>(5),
                "current_version": r.get::<i64, _>(6),
                "created_at": r.get::<String, _>(7),
                "updated_at": r.get::<String, _>(8),
            })
        })
        .collect();

    Json(json!({ "prompts": prompts })).into_response()
}

async fn insert_prompt_records(
    pool: &SqlitePool,
    prm_id: &str,
    ver_id: &str,
    now: &str,
    payload: &CreatePromptRequest,
) -> Result<(), Box<Response>> {
    let mut tx = pool.begin().await.map_err(|e| {
        boxed_error(
            "db_error",
            &e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    sqlx::query(
        "INSERT INTO m_prompt_library_prompt (id, workspace_id, project_id, name, role, skill_tag, current_version, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?)",
    )
    .bind(prm_id)
    .bind(&payload.workspace_id)
    .bind(&payload.project_id)
    .bind(&payload.name)
    .bind(&payload.role)
    .bind(&payload.skill_tag)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| boxed_error("db_error", &e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    sqlx::query(
        "INSERT INTO m_prompt_library_version (id, prompt_id, version, content, created_at)
         VALUES (?, ?, 1, ?, ?)",
    )
    .bind(ver_id)
    .bind(prm_id)
    .bind(&payload.content)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        boxed_error(
            "db_error",
            &e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    tx.commit().await.map_err(|e| {
        boxed_error(
            "db_error",
            &e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })
}

/// Handler xử lý `POST /prompts`.
pub async fn create_prompt_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreatePromptRequest>,
) -> Response {
    if let Err(resp) = validate_prompt_content(&payload.content) {
        return *resp;
    }

    let pool = match state.pool {
        Some(ref p) => p,
        None => {
            return error_response(
                "no_db",
                "Chưa kết nối DB",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    if let Err(resp) = authorize_workspace(pool, &headers, &payload.workspace_id).await {
        return *resp;
    }

    let prompt_id = format!("prm_{}", Uuid::new_v4().simple());
    let version_id = format!("ver_{}", Uuid::new_v4().simple());
    let now = Utc::now().to_rfc3339();

    if let Err(resp) = insert_prompt_records(pool, &prompt_id, &version_id, &now, &payload).await {
        return *resp;
    }

    (
        StatusCode::CREATED,
        Json(json!({
            "id": prompt_id,
            "version": 1,
            "name": payload.name,
            "created_at": now
        })),
    )
        .into_response()
}

/// Handler xử lý `GET /prompts/{id}`.
pub async fn get_prompt_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<String>,
) -> Response {
    let pool = match state.pool {
        Some(ref p) => p,
        None => {
            return error_response(
                "no_db",
                "Chưa kết nối DB",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    if let Err(resp) = authorize_prompt(pool, &headers, &id).await {
        return *resp;
    }

    let row = match sqlx::query(
        "SELECT p.id, p.workspace_id, p.project_id, p.name, p.role, p.skill_tag, p.current_version, v.content, p.created_at, p.updated_at
         FROM m_prompt_library_prompt p
         JOIN m_prompt_library_version v ON v.prompt_id = p.id AND v.version = p.current_version
         WHERE p.id = ?",
    )
    .bind(&id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return error_response("not_found", "Không tìm thấy prompt", StatusCode::NOT_FOUND),
        Err(e) => return error_response("db_error", &e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    };

    Json(json!({
        "id": row.get::<String, _>(0),
        "workspace_id": row.get::<String, _>(1),
        "project_id": row.get::<Option<String>, _>(2),
        "name": row.get::<String, _>(3),
        "role": row.get::<Option<String>, _>(4),
        "skill_tag": row.get::<Option<String>, _>(5),
        "version": row.get::<i64, _>(6),
        "content": row.get::<String, _>(7),
        "created_at": row.get::<String, _>(8),
        "updated_at": row.get::<String, _>(9),
    }))
    .into_response()
}

async fn update_prompt_version_records(
    pool: &SqlitePool,
    id: &str,
    next_ver: i64,
    now: &str,
    payload: &UpdatePromptRequest,
) -> Result<(), Box<Response>> {
    let mut tx = pool.begin().await.map_err(|e| {
        boxed_error(
            "db_error",
            &e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    let version_id = format!("ver_{}", Uuid::new_v4().simple());
    sqlx::query(
        "INSERT INTO m_prompt_library_version (id, prompt_id, version, content, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&version_id)
    .bind(id)
    .bind(next_ver)
    .bind(&payload.content)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        boxed_error(
            "db_error",
            &e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    sqlx::query(
        "UPDATE m_prompt_library_prompt
         SET current_version = ?, name = COALESCE(?, name), role = COALESCE(?, role), skill_tag = COALESCE(?, skill_tag), updated_at = ?
         WHERE id = ?",
    )
    .bind(next_ver)
    .bind(&payload.name)
    .bind(&payload.role)
    .bind(&payload.skill_tag)
    .bind(now)
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(|e| boxed_error("db_error", &e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    tx.commit().await.map_err(|e| {
        boxed_error(
            "db_error",
            &e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })
}

/// Handler xử lý `PUT /prompts/{id}` (tạo phiên bản mới).
pub async fn update_prompt_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<String>,
    Json(payload): Json<UpdatePromptRequest>,
) -> Response {
    if let Err(resp) = validate_prompt_content(&payload.content) {
        return *resp;
    }

    let pool = match state.pool {
        Some(ref p) => p,
        None => {
            return error_response(
                "no_db",
                "Chưa kết nối DB",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    if let Err(resp) = authorize_prompt(pool, &headers, &id).await {
        return *resp;
    }

    let cur_version: i64 = match sqlx::query_scalar(
        "SELECT current_version FROM m_prompt_library_prompt WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(v)) => v,
        Ok(None) => {
            return error_response("not_found", "Không tìm thấy prompt", StatusCode::NOT_FOUND);
        }
        Err(e) => {
            return error_response(
                "db_error",
                &e.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    let next_version = cur_version + 1;
    let now = Utc::now().to_rfc3339();

    if let Err(resp) = update_prompt_version_records(pool, &id, next_version, &now, &payload).await
    {
        return *resp;
    }

    Json(json!({ "id": id, "version": next_version, "updated_at": now })).into_response()
}

/// Handler xử lý `GET /prompts/{id}/versions`.
pub async fn list_versions_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<String>,
) -> Response {
    let pool = match state.pool {
        Some(ref p) => p,
        None => {
            return error_response(
                "no_db",
                "Chưa kết nối DB",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    if let Err(resp) = authorize_prompt(pool, &headers, &id).await {
        return *resp;
    }

    let rows = match sqlx::query(
        "SELECT id, version, content, created_at
         FROM m_prompt_library_version
         WHERE prompt_id = ?
         ORDER BY version DESC",
    )
    .bind(&id)
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            return error_response(
                "db_error",
                &e.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    let versions: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.get::<String, _>(0),
                "version": r.get::<i64, _>(1),
                "content": r.get::<String, _>(2),
                "created_at": r.get::<String, _>(3),
            })
        })
        .collect();

    Json(json!({ "prompt_id": id, "versions": versions })).into_response()
}

async fn apply_version_restore(
    pool: &SqlitePool,
    id: &str,
    next_ver: i64,
    old_content: &str,
    now: &str,
) -> Result<(), Box<Response>> {
    let mut tx = pool.begin().await.map_err(|e| {
        boxed_error(
            "db_error",
            &e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    let version_id = format!("ver_{}", Uuid::new_v4().simple());
    sqlx::query(
        "INSERT INTO m_prompt_library_version (id, prompt_id, version, content, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&version_id)
    .bind(id)
    .bind(next_ver)
    .bind(old_content)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        boxed_error(
            "db_error",
            &e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    sqlx::query(
        "UPDATE m_prompt_library_prompt SET current_version = ?, updated_at = ? WHERE id = ?",
    )
    .bind(next_ver)
    .bind(now)
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        boxed_error(
            "db_error",
            &e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    tx.commit().await.map_err(|e| {
        boxed_error(
            "db_error",
            &e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })
}

async fn fetch_restore_info(
    pool: &SqlitePool,
    id: &str,
    ver: i64,
) -> Result<(String, i64), Box<Response>> {
    let old: String = sqlx::query_scalar(
        "SELECT content FROM m_prompt_library_version WHERE prompt_id = ? AND version = ?",
    )
    .bind(id)
    .bind(ver)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        boxed_error(
            "db_error",
            &e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?
    .ok_or_else(|| {
        boxed_error(
            "not_found",
            "Không tìm thấy phiên bản để khôi phục",
            StatusCode::NOT_FOUND,
        )
    })?;

    let cur: i64 =
        sqlx::query_scalar("SELECT current_version FROM m_prompt_library_prompt WHERE id = ?")
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| {
                boxed_error(
                    "db_error",
                    &e.to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            })?;

    Ok((old, cur))
}

/// Handler xử lý `POST /prompts/{id}/restore/{version}`.
pub async fn restore_version_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((id, version_num)): AxumPath<(String, i64)>,
) -> Response {
    let pool = match state.pool {
        Some(ref p) => p,
        None => {
            return error_response(
                "no_db",
                "Chưa kết nối DB",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    if let Err(resp) = authorize_prompt(pool, &headers, &id).await {
        return *resp;
    }

    let (old_content, cur_version) = match fetch_restore_info(pool, &id, version_num).await {
        Ok(info) => info,
        Err(resp) => return *resp,
    };

    let next_version = cur_version + 1;
    let now = Utc::now().to_rfc3339();

    if let Err(resp) = apply_version_restore(pool, &id, next_version, &old_content, &now).await {
        return *resp;
    }

    Json(json!({
        "id": id,
        "restored_from_version": version_num,
        "new_version": next_version,
        "content": old_content,
        "updated_at": now
    }))
    .into_response()
}

/// Handler xử lý `POST /prompts/test-render` (thử kết xuất không gọi agent).
pub async fn test_render_handler(Json(payload): Json<TestRenderRequest>) -> Response {
    if let Err(resp) = validate_prompt_content(&payload.content) {
        return *resp;
    }

    let title = payload
        .task_title
        .unwrap_or_else(|| "Nhiệm vụ thử nghiệm".into());
    let desc = payload
        .task_description
        .unwrap_or_else(|| "Mô tả thử nghiệm".into());
    let proj = payload.project_name.unwrap_or_else(|| "Dự án mẫu".into());
    let role = payload.role.unwrap_or_else(|| "worker".into());

    let rendered = render_template(&payload.content, &title, &desc, &proj, &role);

    Json(json!({
        "rendered": rendered,
        "variables_used": {
            "task.title": title,
            "task.description": desc,
            "project.name": proj,
            "role": role
        }
    }))
    .into_response()
}

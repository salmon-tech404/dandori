//! Bộ điều khiển HTTP handlers cho module `tech-profile` (P1-08).

use axum::{
    Json,
    extract::{Path as AxumPath, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use dandori_module_api::{can_access_workspace, session_user};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

#[derive(Clone, Default, Debug)]
pub struct AppState {
    pub pool: Option<SqlitePool>,
}

use crate::contributor::{TechProfileRow as ContributorRow, format_tech_profile};
use crate::validation::{check_secrets, check_value_length};

#[derive(Debug, Deserialize)]
pub struct WorkspaceQuery {
    pub workspace_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEntryRequest {
    pub workspace_id: String,
    pub category: String,
    pub key: String,
    pub value: String,
    pub sort_order: Option<i64>,
    pub updated_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEntryRequest {
    pub workspace_id: String,
    pub category: Option<String>,
    pub key: Option<String>,
    pub value: String,
    pub sort_order: Option<i64>,
    pub updated_by: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TechProfileRow {
    pub id: String,
    pub workspace_id: String,
    pub project_id: String,
    pub category: String,
    pub key: String,
    pub value: String,
    pub sort_order: i64,
    pub version: i64,
    pub updated_by: String,
    pub created_at: String,
    pub updated_at: String,
}

fn error_response(code: &'static str, message: &str, status: StatusCode) -> Response {
    (
        status,
        Json(json!({ "error": { "code": code, "message": message } })),
    )
        .into_response()
}

fn boxed_error(code: &'static str, message: &str, status: StatusCode) -> Box<Response> {
    Box::new(error_response(code, message, status))
}

fn get_pool(state: &AppState) -> Result<&SqlitePool, Box<Response>> {
    state.pool.as_ref().ok_or_else(|| {
        boxed_error(
            "no_db",
            "Chưa kết nối DB",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })
}

fn validate_entry_value(value: &str) -> Result<(), Box<Response>> {
    if let Err(msg) = check_value_length(value) {
        return Err(boxed_error("value_too_long", msg, StatusCode::BAD_REQUEST));
    }
    if let Err(msg) = check_secrets(value) {
        return Err(boxed_error("secret_detected", msg, StatusCode::BAD_REQUEST));
    }
    Ok(())
}

async fn verify_project_workspace(
    pool: &SqlitePool,
    project_id: &str,
    req_ws_id: &str,
) -> Result<(), Box<Response>> {
    let proj_ws: Option<String> =
        sqlx::query_scalar("SELECT workspace_id FROM project WHERE id = ?")
            .bind(project_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                boxed_error(
                    "db_error",
                    &e.to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            })?;

    match proj_ws {
        Some(ref ws) if ws == req_ws_id => Ok(()),
        Some(_) => Err(boxed_error(
            "workspace_isolation",
            "Bạn không có quyền truy cập vào dự án thuộc workspace khác",
            StatusCode::FORBIDDEN,
        )),
        None => Err(boxed_error(
            "project_not_found",
            "Không tìm thấy dự án",
            StatusCode::NOT_FOUND,
        )),
    }
}

/// Yêu cầu có phiên hợp lệ và quyền truy cập workspace `workspace_id`.
async fn authorize_workspace(
    pool: &SqlitePool,
    headers: &HeaderMap,
    workspace_id: &str,
) -> Result<(), Box<Response>> {
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
    Ok(())
}

/// Yêu cầu quyền trên workspace thật sự chứa dự án (không tin tham số `workspace_id` của client).
async fn authorize_project(
    pool: &SqlitePool,
    headers: &HeaderMap,
    project_id: &str,
) -> Result<(), Box<Response>> {
    let proj_ws: Option<String> =
        sqlx::query_scalar("SELECT workspace_id FROM project WHERE id = ?")
            .bind(project_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
    match proj_ws {
        Some(ws) => authorize_workspace(pool, headers, &ws).await,
        None => {
            if session_user(pool, headers).await.is_none() {
                return Err(boxed_error(
                    "unauthorized",
                    "Phiên đăng nhập không hợp lệ hoặc đã hết hạn",
                    StatusCode::UNAUTHORIZED,
                ));
            }
            Err(boxed_error(
                "project_not_found",
                "Không tìm thấy dự án",
                StatusCode::NOT_FOUND,
            ))
        }
    }
}

async fn check_workspace_access(
    pool: &SqlitePool,
    proj_id: &str,
    ws: Option<&str>,
) -> Result<(), Box<Response>> {
    if let Some(w) = ws {
        verify_project_workspace(pool, proj_id, w).await?;
    }
    Ok(())
}

async fn fetch_entry_meta(pool: &SqlitePool, id: &str) -> Result<(String, i64), Box<Response>> {
    let row: Option<(String, i64)> =
        sqlx::query_as("SELECT workspace_id, version FROM m_tech_profile_entry WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                boxed_error(
                    "db_error",
                    &e.to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            })?;

    match row {
        Some(r) => Ok(r),
        None => Err(boxed_error(
            "not_found",
            "Không tìm thấy mục hồ sơ",
            StatusCode::NOT_FOUND,
        )),
    }
}

async fn insert_entry(
    pool: &SqlitePool,
    id: &str,
    proj_id: &str,
    payload: &CreateEntryRequest,
    now: &str,
) -> Result<(), sqlx::Error> {
    let sort_order = payload.sort_order.unwrap_or(0);
    let updated_by = payload.updated_by.as_deref().unwrap_or("user");
    sqlx::query(
        "INSERT INTO m_tech_profile_entry
         (id, workspace_id, project_id, category, key, value, sort_order, version, updated_by, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, 1, ?, ?, ?)",
    )
    .bind(id)
    .bind(&payload.workspace_id)
    .bind(proj_id)
    .bind(&payload.category)
    .bind(&payload.key)
    .bind(&payload.value)
    .bind(sort_order)
    .bind(updated_by)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

async fn execute_entry_update(
    pool: &SqlitePool,
    id: &str,
    payload: &UpdateEntryRequest,
    next_ver: i64,
    now: &str,
) -> Result<(), sqlx::Error> {
    let updated_by = payload.updated_by.as_deref().unwrap_or("user");
    sqlx::query(
        "UPDATE m_tech_profile_entry
         SET category = COALESCE(?, category),
             key = COALESCE(?, key),
             value = ?,
             sort_order = COALESCE(?, sort_order),
             version = ?,
             updated_by = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(&payload.category)
    .bind(&payload.key)
    .bind(&payload.value)
    .bind(payload.sort_order)
    .bind(next_ver)
    .bind(updated_by)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn delete_entry(pool: &SqlitePool, id: &str, ws_id: &str) -> Result<(), Box<Response>> {
    let (cur_ws, _) = fetch_entry_meta(pool, id).await?;
    if cur_ws != ws_id {
        return Err(boxed_error(
            "workspace_isolation",
            "Bạn không có quyền xóa mục thuộc workspace khác",
            StatusCode::FORBIDDEN,
        ));
    }
    sqlx::query("DELETE FROM m_tech_profile_entry WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            boxed_error(
                "db_error",
                &e.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;
    Ok(())
}

/// Handler xử lý `GET /projects/{project_id}/entries`.
pub async fn list_entries_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(proj_id): AxumPath<String>,
    Query(query): Query<WorkspaceQuery>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return *r,
    };
    if let Err(resp) = authorize_project(pool, &headers, &proj_id).await {
        return *resp;
    }
    if let Err(resp) = check_workspace_access(pool, &proj_id, query.workspace_id.as_deref()).await {
        return *resp;
    }

    let rows = match sqlx::query(
        "SELECT id, workspace_id, project_id, category, key, value, sort_order, version, updated_by, created_at, updated_at
         FROM m_tech_profile_entry
         WHERE project_id = ?
         ORDER BY category ASC, sort_order ASC, key ASC",
    )
    .bind(&proj_id)
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => return error_response("db_error", &e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    };

    let entries: Vec<TechProfileRow> = rows
        .into_iter()
        .map(|r| TechProfileRow {
            id: r.get(0),
            workspace_id: r.get(1),
            project_id: r.get(2),
            category: r.get(3),
            key: r.get(4),
            value: r.get(5),
            sort_order: r.get(6),
            version: r.get(7),
            updated_by: r.get(8),
            created_at: r.get(9),
            updated_at: r.get(10),
        })
        .collect();

    Json(json!({ "entries": entries })).into_response()
}

/// Handler xử lý `POST /projects/{project_id}/entries`.
pub async fn create_entry_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(proj_id): AxumPath<String>,
    Json(payload): Json<CreateEntryRequest>,
) -> Response {
    if let Err(resp) = validate_entry_value(&payload.value) {
        return *resp;
    }
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return *r,
    };
    if let Err(resp) = authorize_project(pool, &headers, &proj_id).await {
        return *resp;
    }
    if let Err(resp) = verify_project_workspace(pool, &proj_id, &payload.workspace_id).await {
        return *resp;
    }

    let entry_id = format!("tpe_{}", Uuid::new_v4().simple());
    let now = Utc::now().to_rfc3339();
    if let Err(e) = insert_entry(pool, &entry_id, &proj_id, &payload, &now).await {
        return error_response(
            "db_error",
            &e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        );
    }

    (
        StatusCode::CREATED,
        Json(json!({ "id": entry_id, "project_id": proj_id, "key": payload.key, "version": 1 })),
    )
        .into_response()
}

/// Handler xử lý `PUT /entries/{id}`.
pub async fn update_entry_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<String>,
    Json(payload): Json<UpdateEntryRequest>,
) -> Response {
    if let Err(resp) = validate_entry_value(&payload.value) {
        return *resp;
    }
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return *r,
    };
    let (cur_ws, cur_ver) = match fetch_entry_meta(pool, &id).await {
        Ok(m) => m,
        Err(resp) => return *resp,
    };
    if let Err(resp) = authorize_workspace(pool, &headers, &cur_ws).await {
        return *resp;
    }
    if cur_ws != payload.workspace_id {
        return error_response(
            "workspace_isolation",
            "Bạn không có quyền sửa mục thuộc workspace khác",
            StatusCode::FORBIDDEN,
        );
    }

    let next_ver = cur_ver + 1;
    let now = Utc::now().to_rfc3339();
    if let Err(e) = execute_entry_update(pool, &id, &payload, next_ver, &now).await {
        return error_response(
            "db_error",
            &e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        );
    }

    Json(json!({ "id": id, "version": next_ver, "updated_at": now })).into_response()
}

/// Handler xử lý `DELETE /entries/{id}`.
pub async fn delete_entry_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<String>,
    Query(query): Query<WorkspaceQuery>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return *r,
    };
    let ws_id = match query.workspace_id {
        Some(w) => w,
        None => {
            return error_response(
                "missing_workspace",
                "Thiếu tham số workspace_id",
                StatusCode::BAD_REQUEST,
            );
        }
    };
    if let Err(resp) = authorize_workspace(pool, &headers, &ws_id).await {
        return *resp;
    }
    if let Err(resp) = delete_entry(pool, &id, &ws_id).await {
        return *resp;
    }

    Json(json!({ "deleted": true, "id": id })).into_response()
}

/// Handler xử lý `GET /projects/{project_id}/preview` (xem trước khối prompt sinh ra).
pub async fn preview_profile_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(proj_id): AxumPath<String>,
    Query(query): Query<WorkspaceQuery>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err(r) => return *r,
    };
    if let Err(resp) = authorize_project(pool, &headers, &proj_id).await {
        return *resp;
    }
    if let Err(resp) = check_workspace_access(pool, &proj_id, query.workspace_id.as_deref()).await {
        return *resp;
    }

    let rows = match sqlx::query(
        "SELECT category, key, value
         FROM m_tech_profile_entry
         WHERE project_id = ?
         ORDER BY category ASC, sort_order ASC, key ASC",
    )
    .bind(&proj_id)
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

    let entries: Vec<ContributorRow> = rows
        .into_iter()
        .map(|r| ContributorRow {
            category: r.get(0),
            key: r.get(1),
            value: r.get(2),
        })
        .collect();

    let preview = format_tech_profile(&entries);
    Json(json!({ "project_id": proj_id, "preview": preview, "total_entries": entries.len() }))
        .into_response()
}

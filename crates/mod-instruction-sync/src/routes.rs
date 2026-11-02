//! REST routes cho module `instruction-sync` (P3-09).

use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use dandori_module_api::{
    ModuleContext, SessionUser, can_access_workspace, can_administer_workspace, session_user,
};
use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::generator::{default_target_filename, generate_instruction_content};
use crate::model::{InstructionSyncConfig, PreviewPayload, PreviewResult, UpdateConfigPayload};

#[derive(Clone, Debug)]
pub struct InstructionSyncState {
    pub ctx: ModuleContext,
}

impl InstructionSyncState {
    #[must_use]
    pub fn new(ctx: ModuleContext) -> Self {
        Self { ctx }
    }
}

fn err_resp(status: StatusCode, msg: &str) -> Response {
    (status, Json(json!({ "error": { "message": msg } }))).into_response()
}

fn get_pool(state: &InstructionSyncState) -> Result<SqlitePool, (StatusCode, &'static str)> {
    state.ctx.pool().cloned().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Không có kết nối cơ sở dữ liệu",
    ))
}

async fn get_user(
    headers: &HeaderMap,
    pool: &SqlitePool,
) -> Result<SessionUser, (StatusCode, &'static str)> {
    session_user(pool, headers).await.ok_or((
        StatusCode::UNAUTHORIZED,
        "Yêu cầu đăng nhập để truy cập tài nguyên",
    ))
}

pub fn instruction_sync_router(state: InstructionSyncState) -> Router {
    Router::new()
        .route(
            "/workspaces/{ws_id}/projects/{project_id}/config",
            get(get_project_configs_handler).post(upsert_project_config_handler),
        )
        .route("/workspaces/{ws_id}/preview", post(preview_handler))
        .with_state(state)
}

async fn get_project_configs_handler(
    State(state): State<InstructionSyncState>,
    AxumPath((ws_id, project_id)): AxumPath<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((code, msg)) => return err_resp(code, msg),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((code, msg)) => return err_resp(code, msg),
    };
    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    let rows = sqlx::query_as::<_, InstructionSyncConfig>(
        r#"
        SELECT id, workspace_id, project_id, provider, target_filename, strategy, is_enabled, created_at, updated_at
        FROM m_instruction_sync_config
        WHERE workspace_id = ? AND project_id = ?
        ORDER BY provider ASC
        "#,
    )
    .bind(&ws_id)
    .bind(&project_id)
    .fetch_all(&pool)
    .await;

    match rows {
        Ok(configs) => Json(configs).into_response(),
        Err(e) => err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Lỗi truy vấn cấu hình: {}", e),
        ),
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "Hàm xử lý cấu hình đồng bộ hướng dẫn kèm xác thực quyền quản trị viên"
)]
async fn upsert_project_config_handler(
    State(state): State<InstructionSyncState>,
    AxumPath((ws_id, project_id)): AxumPath<(String, String)>,
    headers: HeaderMap,
    Json(payload): Json<UpdateConfigPayload>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((code, msg)) => return err_resp(code, msg),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((code, msg)) => return err_resp(code, msg),
    };
    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Chỉ quản trị viên workspace mới được cấu hình đồng bộ hướng dẫn",
        );
    }

    let provider = payload.provider.trim();
    if provider.is_empty() {
        return err_resp(StatusCode::BAD_REQUEST, "Nhà cung cấp không được để trống");
    }

    let target_filename = payload.target_filename.trim();
    if target_filename.is_empty()
        || target_filename.contains('/')
        || target_filename.contains('\\')
        || target_filename.contains("..")
    {
        return err_resp(
            StatusCode::BAD_REQUEST,
            "Tên tệp không hợp lệ (không được chứa đường dẫn con hoặc ..)",
        );
    }

    let allowed = ["AGENTS.md", "CLAUDE.md", ".cursorrules"];
    if !allowed.contains(&target_filename) {
        return err_resp(
            StatusCode::BAD_REQUEST,
            "Tên tệp không nằm trong danh sách cho phép (AGENTS.md, CLAUDE.md, .cursorrules)",
        );
    }

    let strategy = match payload.strategy.to_lowercase().as_str() {
        "append" => "append",
        _ => "skip",
    };

    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    let query = r#"
        INSERT INTO m_instruction_sync_config (
            id, workspace_id, project_id, provider, target_filename, strategy, is_enabled, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(workspace_id, project_id, provider) DO UPDATE SET
            target_filename = excluded.target_filename,
            strategy = excluded.strategy,
            is_enabled = excluded.is_enabled,
            updated_at = excluded.updated_at
        RETURNING id, workspace_id, project_id, provider, target_filename, strategy, is_enabled, created_at, updated_at
    "#;

    match sqlx::query_as::<_, InstructionSyncConfig>(query)
        .bind(id)
        .bind(&ws_id)
        .bind(&project_id)
        .bind(provider)
        .bind(target_filename)
        .bind(strategy)
        .bind(payload.is_enabled)
        .bind(&now)
        .bind(&now)
        .fetch_one(&pool)
        .await
    {
        Ok(saved) => Json(saved).into_response(),
        Err(e) => err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Lỗi lưu cấu hình: {}", e),
        ),
    }
}

async fn preview_handler(
    State(state): State<InstructionSyncState>,
    AxumPath(ws_id): AxumPath<String>,
    headers: HeaderMap,
    Json(payload): Json<PreviewPayload>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((code, msg)) => return err_resp(code, msg),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((code, msg)) => return err_resp(code, msg),
    };
    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    let target_filename = default_target_filename(&payload.provider).to_string();

    match generate_instruction_content(
        &pool,
        &ws_id,
        &payload.project_id,
        &payload.provider,
        payload.role.as_deref(),
    )
    .await
    {
        Ok((content, tech_count, skills_count)) => Json(PreviewResult {
            target_filename,
            content,
            tech_profile_count: tech_count,
            skills_count,
        })
        .into_response(),
        Err(e) => err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Lỗi sinh nội dung: {}", e),
        ),
    }
}

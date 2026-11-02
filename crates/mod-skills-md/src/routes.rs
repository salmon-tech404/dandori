//! Các API REST routes cho module `skills-md` (P3-08).

use std::path::PathBuf;

use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use dandori_module_api::{
    ModuleContext, SessionUser, can_access_workspace, can_administer_workspace, session_user,
};
use serde_json::json;
use sqlx::SqlitePool;

use crate::archive::{extract_zip, parse_skill_from_files, read_directory};
use crate::db::{
    approve_skill, create_binding, delete_binding, delete_skill, get_skill, list_skills,
    upsert_skill,
};
use crate::model::{CreateBindingPayload, ImportDirPayload, ImportZipPayload};

/// Trạng thái Router của module skills-md.
#[derive(Clone, Debug)]
pub struct SkillsMdState {
    pub ctx: ModuleContext,
}

impl SkillsMdState {
    #[must_use]
    pub fn new(ctx: ModuleContext) -> Self {
        Self { ctx }
    }
}

fn err_resp(status: StatusCode, msg: &str) -> Response {
    (status, Json(json!({ "error": { "message": msg } }))).into_response()
}

fn get_pool(state: &SkillsMdState) -> Result<SqlitePool, (StatusCode, &'static str)> {
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

fn decode_base64(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0;
    for &b in input.as_bytes() {
        if b.is_ascii_whitespace() || b == b'=' {
            continue;
        }
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => return None,
        };
        buf = (buf << 6) | u32::from(val);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            #[expect(
                clippy::cast_possible_truncation,
                reason = "Shifted bit buffer fits into u8"
            )]
            out.push((buf >> bits) as u8);
        }
    }
    Some(out)
}

/// Xây dựng Router cho module skills-md.
pub fn skills_md_router(state: SkillsMdState) -> Router {
    Router::new()
        .route("/workspaces/{ws_id}/skills", get(list_skills_handler))
        .route(
            "/workspaces/{ws_id}/skills/{skill_id}",
            get(get_skill_handler).delete(delete_skill_handler),
        )
        .route(
            "/workspaces/{ws_id}/skills/import-dir",
            post(import_dir_handler),
        )
        .route(
            "/workspaces/{ws_id}/skills/import-zip",
            post(import_zip_handler),
        )
        .route(
            "/workspaces/{ws_id}/skills/{skill_id}/approve",
            post(approve_skill_handler),
        )
        .route(
            "/workspaces/{ws_id}/skills/{skill_id}/bindings",
            post(create_binding_handler),
        )
        .route(
            "/workspaces/{ws_id}/skills/{skill_id}/bindings/{binding_id}",
            delete(delete_binding_handler),
        )
        .with_state(state)
}

async fn list_skills_handler(
    State(state): State<SkillsMdState>,
    AxumPath(ws_id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((st, msg)) => return err_resp(st, msg),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((st, msg)) => return err_resp(st, msg),
    };
    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    match list_skills(&pool, &ws_id).await {
        Ok(skills) => Json(json!({ "skills": skills })).into_response(),
        Err(e) => err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Lỗi truy vấn danh sách kỹ năng: {e}"),
        ),
    }
}

async fn get_skill_handler(
    State(state): State<SkillsMdState>,
    AxumPath((ws_id, skill_id)): AxumPath<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((st, msg)) => return err_resp(st, msg),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((st, msg)) => return err_resp(st, msg),
    };
    if !can_access_workspace(&pool, &user, &ws_id).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    match get_skill(&pool, &ws_id, &skill_id).await {
        Ok(Some(skill)) => Json(json!({ "skill": skill })).into_response(),
        Ok(None) => err_resp(StatusCode::NOT_FOUND, "Không tìm thấy kỹ năng"),
        Err(e) => err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Lỗi nạp kỹ năng: {e}"),
        ),
    }
}

async fn import_dir_handler(
    State(state): State<SkillsMdState>,
    AxumPath(ws_id): AxumPath<String>,
    headers: HeaderMap,
    Json(payload): Json<ImportDirPayload>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((st, msg)) => return err_resp(st, msg),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((st, msg)) => return err_resp(st, msg),
    };
    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Chỉ Quản trị viên mới được nhập kỹ năng",
        );
    }
    // Đường dẫn thư mục nằm trên máy chủ: chỉ quản trị hệ thống được chọn (không để admin workspace dò đĩa).
    if !user.is_system_admin() {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Chỉ quản trị hệ thống mới được nhập kỹ năng từ thư mục trên máy chủ",
        );
    }

    let files = match read_directory(&PathBuf::from(&payload.dir_path)) {
        Ok(f) => f,
        Err(e) => return err_resp(StatusCode::BAD_REQUEST, &e.to_string()),
    };

    let parsed = match parse_skill_from_files(&files) {
        Ok(p) => p,
        Err(e) => return err_resp(StatusCode::BAD_REQUEST, &e.to_string()),
    };

    match upsert_skill(&pool, &ws_id, &parsed).await {
        Ok(id) => Json(json!({
            "id": id,
            "name": parsed.name,
            "has_scripts": parsed.has_scripts,
            "script_files": parsed.script_files,
            "status": "pending_approval"
        }))
        .into_response(),
        Err(e) => err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Lỗi lưu kỹ năng: {e}"),
        ),
    }
}

async fn import_zip_handler(
    State(state): State<SkillsMdState>,
    AxumPath(ws_id): AxumPath<String>,
    headers: HeaderMap,
    Json(payload): Json<ImportZipPayload>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((st, msg)) => return err_resp(st, msg),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((st, msg)) => return err_resp(st, msg),
    };
    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Chỉ Quản trị viên mới được nhập kỹ năng",
        );
    }

    let raw_bytes = match decode_base64(&payload.base64_content) {
        Some(b) => b,
        None => return err_resp(StatusCode::BAD_REQUEST, "Dữ liệu base64 zip không hợp lệ"),
    };

    let files = match extract_zip(&raw_bytes) {
        Ok(f) => f,
        Err(e) => return err_resp(StatusCode::BAD_REQUEST, &e.to_string()),
    };

    let parsed = match parse_skill_from_files(&files) {
        Ok(p) => p,
        Err(e) => return err_resp(StatusCode::BAD_REQUEST, &e.to_string()),
    };

    match upsert_skill(&pool, &ws_id, &parsed).await {
        Ok(id) => Json(json!({
            "id": id,
            "name": parsed.name,
            "has_scripts": parsed.has_scripts,
            "script_files": parsed.script_files,
            "status": "pending_approval"
        }))
        .into_response(),
        Err(e) => err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Lỗi lưu kỹ năng: {e}"),
        ),
    }
}

async fn approve_skill_handler(
    State(state): State<SkillsMdState>,
    AxumPath((ws_id, skill_id)): AxumPath<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((st, msg)) => return err_resp(st, msg),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((st, msg)) => return err_resp(st, msg),
    };
    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Chỉ Quản trị viên mới được phê duyệt kỹ năng",
        );
    }

    match approve_skill(&pool, &ws_id, &skill_id).await {
        Ok(true) => Json(json!({ "success": true, "approved": true })).into_response(),
        Ok(false) => err_resp(StatusCode::NOT_FOUND, "Không tìm thấy kỹ năng để phê duyệt"),
        Err(e) => err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Lỗi phê duyệt kỹ năng: {e}"),
        ),
    }
}

async fn delete_skill_handler(
    State(state): State<SkillsMdState>,
    AxumPath((ws_id, skill_id)): AxumPath<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((st, msg)) => return err_resp(st, msg),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((st, msg)) => return err_resp(st, msg),
    };
    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Chỉ Quản trị viên mới được xóa kỹ năng",
        );
    }

    match delete_skill(&pool, &ws_id, &skill_id).await {
        Ok(true) => Json(json!({ "success": true })).into_response(),
        Ok(false) => err_resp(StatusCode::NOT_FOUND, "Không tìm thấy kỹ năng để xóa"),
        Err(e) => err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Lỗi xóa kỹ năng: {e}"),
        ),
    }
}

async fn create_binding_handler(
    State(state): State<SkillsMdState>,
    AxumPath((ws_id, skill_id)): AxumPath<(String, String)>,
    headers: HeaderMap,
    Json(payload): Json<CreateBindingPayload>,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((st, msg)) => return err_resp(st, msg),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((st, msg)) => return err_resp(st, msg),
    };
    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Chỉ Quản trị viên mới được cấu hình gắn kết",
        );
    }

    let embed_full = payload.embed_full.unwrap_or(false);
    match create_binding(
        &pool,
        &ws_id,
        &skill_id,
        payload.project_id.as_deref(),
        payload.role.as_deref(),
        embed_full,
    )
    .await
    {
        Ok(id) => Json(json!({ "id": id, "success": true })).into_response(),
        Err(e) => err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Lỗi tạo gắn kết: {e}"),
        ),
    }
}

async fn delete_binding_handler(
    State(state): State<SkillsMdState>,
    AxumPath((ws_id, _skill_id, binding_id)): AxumPath<(String, String, String)>,
    headers: HeaderMap,
) -> Response {
    let pool = match get_pool(&state) {
        Ok(p) => p,
        Err((st, msg)) => return err_resp(st, msg),
    };
    let user = match get_user(&headers, &pool).await {
        Ok(u) => u,
        Err((st, msg)) => return err_resp(st, msg),
    };
    if !can_administer_workspace(&pool, &user, &ws_id).await {
        return err_resp(
            StatusCode::FORBIDDEN,
            "Chỉ Quản trị viên mới được xóa gắn kết",
        );
    }

    match delete_binding(&pool, &ws_id, &binding_id).await {
        Ok(true) => Json(json!({ "success": true })).into_response(),
        Ok(false) => err_resp(StatusCode::NOT_FOUND, "Không tìm thấy gắn kết"),
        Err(e) => err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Lỗi xóa gắn kết: {e}"),
        ),
    }
}

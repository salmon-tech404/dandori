//! Chọn thư mục kho Git từ giao diện: liệt kê thư mục con và nhánh của một kho (chỉ quản trị hệ thống).
//!
//! Đây là khả năng đọc đĩa của máy chủ nên: chỉ quản trị hệ thống dùng được, chỉ liệt kê **thư mục** (không tên
//! tệp, không nội dung tệp), không đi theo liên kết tượng trưng/junction, giới hạn số mục.

use std::path::{Component, Path, PathBuf};

use axum::extract::Query;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use dandori_git::GitClient;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::ApiError;
use crate::middleware::{AuthUser, RequestId};

/// Số thư mục con tối đa trả về trong một lần.
const MAX_ENTRIES: usize = 500;

/// Tham số: đường dẫn thư mục cần xem (bỏ trống = danh sách ổ đĩa).
#[derive(Debug, Deserialize)]
pub struct PathQuery {
    /// Đường dẫn tuyệt đối.
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
struct DirEntryView {
    name: String,
    path: String,
    is_git: bool,
}

/// Gỡ tiền tố `\\?\` mà `canonicalize` thêm trên Windows để đường dẫn hiển thị quen thuộc.
fn display_path(p: &Path) -> String {
    let s = p.to_string_lossy().to_string();
    s.strip_prefix(r"\\?\")
        .map_or(s.clone(), ToString::to_string)
}

fn require_admin(user: &AuthUser, req_id: &RequestId) -> Option<Response> {
    if user.is_admin() {
        None
    } else {
        Some(
            ApiError::Forbidden(
                "Chỉ quản trị hệ thống được duyệt thư mục trên máy chủ".to_string(),
            )
            .into_response_with_id(&req_id.0),
        )
    }
}

/// Kiểm tra đường dẫn do người dùng gõ: tuyệt đối, không có `..`, là thư mục thật.
fn resolve_dir(raw: &str) -> Result<PathBuf, String> {
    let p = Path::new(raw.trim());
    if !p.is_absolute() {
        return Err("Đường dẫn phải là đường dẫn tuyệt đối".to_string());
    }
    if p.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("Đường dẫn không được chứa '..'".to_string());
    }
    let canonical =
        std::fs::canonicalize(p).map_err(|_| "Không đọc được thư mục này".to_string())?;
    if !canonical.is_dir() {
        return Err("Đường dẫn không phải là thư mục".to_string());
    }
    Ok(canonical)
}

fn list_roots() -> Vec<DirEntryView> {
    #[cfg(windows)]
    {
        ('A'..='Z')
            .map(|c| format!("{c}:\\"))
            .filter(|root| Path::new(root).exists())
            .map(|root| DirEntryView {
                name: root.clone(),
                path: root,
                is_git: false,
            })
            .collect()
    }
    #[cfg(not(windows))]
    {
        vec![DirEntryView {
            name: "/".to_string(),
            path: "/".to_string(),
            is_git: false,
        }]
    }
}

fn list_subdirs(dir: &Path) -> Result<(Vec<DirEntryView>, bool), String> {
    let read = std::fs::read_dir(dir).map_err(|_| "Không đọc được thư mục này".to_string())?;
    let mut out: Vec<DirEntryView> = Vec::new();
    let mut truncated = false;
    for entry in read.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        // Không đi theo symlink/junction; chỉ lấy thư mục.
        if file_type.is_symlink() || !file_type.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if out.len() >= MAX_ENTRIES {
            truncated = true;
            break;
        }
        let path = entry.path();
        out.push(DirEntryView {
            is_git: path.join(".git").exists(),
            path: display_path(&path),
            name,
        });
    }
    out.sort_by_key(|e| e.name.to_lowercase());
    Ok((out, truncated))
}

/// `GET /api/v1/fs/dirs?path=` - Liệt kê thư mục con (chỉ quản trị hệ thống).
pub async fn list_dirs(
    Extension(req_id): Extension<RequestId>,
    user: AuthUser,
    Query(q): Query<PathQuery>,
) -> Response {
    if let Some(denied) = require_admin(&user, &req_id) {
        return denied;
    }
    let raw = q.path.unwrap_or_default();
    if raw.trim().is_empty() {
        return Json(json!({ "path": "", "parent": null, "is_git": false, "entries": list_roots(), "truncated": false }))
            .into_response();
    }

    let result = tokio::task::spawn_blocking(move || {
        let dir = resolve_dir(&raw)?;
        let (entries, truncated) = list_subdirs(&dir)?;
        let parent = dir.parent().map(display_path);
        Ok::<_, String>((
            display_path(&dir),
            parent,
            dir.join(".git").exists(),
            entries,
            truncated,
        ))
    })
    .await;

    match result {
        Ok(Ok((path, parent, is_git, entries, truncated))) => {
            Json(json!({ "path": path, "parent": parent, "is_git": is_git, "entries": entries, "truncated": truncated }))
                .into_response()
        }
        Ok(Err(msg)) => ApiError::BadRequest(msg).into_response_with_id(&req_id.0),
        Err(e) => ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    }
}

/// `GET /api/v1/fs/git-branches?path=` - Danh sách nhánh cục bộ của một kho Git (chỉ quản trị hệ thống).
pub async fn git_branches(
    Extension(req_id): Extension<RequestId>,
    user: AuthUser,
    Query(q): Query<PathQuery>,
) -> Response {
    if let Some(denied) = require_admin(&user, &req_id) {
        return denied;
    }
    let raw = q.path.unwrap_or_default();
    let dir = match tokio::task::spawn_blocking(move || resolve_dir(&raw)).await {
        Ok(Ok(d)) => d,
        Ok(Err(msg)) => return ApiError::BadRequest(msg).into_response_with_id(&req_id.0),
        Err(e) => return ApiError::Internal(e.to_string()).into_response_with_id(&req_id.0),
    };
    if !dir.join(".git").exists() {
        return ApiError::BadRequest("Thư mục này không phải kho Git (thiếu .git)".to_string())
            .into_response_with_id(&req_id.0);
    }
    match GitClient::new(dir).list_branches().await {
        Ok(branches) => Json(json!({ "branches": branches })).into_response(),
        Err(e) => ApiError::BadRequest(format!("Không đọc được danh sách nhánh: {e}"))
            .into_response_with_id(&req_id.0),
    }
}

//! Bộ xử lý yêu cầu HTTP cho module `integrations` (P2-06).

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use dandori_module_api::{
    SecretBox, SqliteSecretStorage, Vault, can_access_workspace, can_administer_workspace,
    default_key_store, session_user,
};
use serde_json::json;
use sqlx::SqlitePool;

use crate::github::GitHubProvider;
use crate::gitlab::GitLabProvider;
use crate::http_client::build_safe_client;
use crate::linear::LinearClient;
use crate::model::{
    CreateConnectionRequest, IntegrationConnection, LinearImportRequest,
    PublicIntegrationConnection,
};
use crate::ssrf::validate_connection_url;

/// Trạng thái chia sẻ của module `integrations`.
#[derive(Clone, Debug)]
pub struct IntegrationsState {
    /// Kết nối cơ sở dữ liệu SQLite.
    pub pool: SqlitePool,
}

fn err_resp(status: StatusCode, msg: &str) -> axum::response::Response {
    (status, Json(json!({ "error": msg }))).into_response()
}

async fn check_ws_access(pool: &SqlitePool, headers: &HeaderMap, ws_id: &str) -> bool {
    let Some(user) = session_user(pool, headers).await else {
        return false;
    };
    can_access_workspace(pool, &user, ws_id).await
}

/// Kho khóa dùng cho token của kết nối (mã hóa AES-256-GCM, khóa chủ nằm trong kho khóa hệ điều hành).
fn token_vault(
    pool: &SqlitePool,
) -> Vault<SqliteSecretStorage, std::sync::Arc<dyn dandori_module_api::MasterKeyStore>> {
    Vault::new(SqliteSecretStorage::new(pool.clone()), default_key_store())
}

/// Tiền tố đánh dấu giá trị cột `token_encrypted` là mã bí mật trong vault (không phải token thô).
pub const VAULT_REF_PREFIX: &str = "vault:";

/// Lấy token thật từ giá trị đã lưu (`vault:<id>`). Giá trị không có tiền tố bị từ chối: không còn đường
/// đọc token lưu thô.
pub async fn reveal_token(pool: &SqlitePool, stored: &str) -> Result<String, String> {
    let id = stored
        .strip_prefix(VAULT_REF_PREFIX)
        .ok_or_else(|| "Token không nằm trong vault".to_string())?;
    let secret_id = id
        .parse()
        .map_err(|_| "Mã bí mật trong vault không hợp lệ".to_string())?;
    token_vault(pool)
        .reveal(&secret_id)
        .await
        .map(|s| s.expose_secret().to_string())
        .map_err(|e| format!("Không đọc được token từ vault: {e}"))
}

/// Chỉ quản trị workspace mới tạo/xóa kết nối ra dịch vụ ngoài; chỉ quản trị hệ thống mới được bật
/// `allow_private_network` (cho phép tới dải mạng nội bộ).
async fn authorize_connection_change(
    pool: &SqlitePool,
    headers: &HeaderMap,
    ws_id: &str,
    wants_private: bool,
) -> Result<(), Box<axum::response::Response>> {
    let Some(user) = session_user(pool, headers).await else {
        return Err(Box::new(err_resp(
            StatusCode::UNAUTHORIZED,
            "Chưa đăng nhập",
        )));
    };
    if !can_administer_workspace(pool, &user, ws_id).await {
        return Err(Box::new(err_resp(
            StatusCode::FORBIDDEN,
            "Chỉ quản trị workspace được thay đổi kết nối tích hợp",
        )));
    }
    if wants_private && !user.is_system_admin() {
        return Err(Box::new(err_resp(
            StatusCode::FORBIDDEN,
            "Chỉ quản trị hệ thống được cho phép kết nối mạng nội bộ",
        )));
    }
    Ok(())
}

fn default_base_url(provider: &str) -> &'static str {
    match provider {
        "github" => "https://api.github.com",
        "gitlab" => "https://gitlab.com",
        "linear" => "https://api.linear.app/graphql",
        _ => "https://api.github.com",
    }
}

async fn test_provider_connection(
    provider: &str,
    base_url: &url::Url,
    token: &str,
    allow_private: bool,
) -> Result<(), String> {
    let client = build_safe_client(base_url, allow_private)
        .await
        .map_err(|e| e.to_string())?;
    match provider {
        "github" => {
            let p = GitHubProvider::new(client, base_url.clone(), token.to_string());
            dandori_module_api::GitHostProvider::test_connection(&p)
                .await
                .map_err(|e| e.to_string())
        }
        "gitlab" => {
            let p = GitLabProvider::new(client, base_url.clone(), token.to_string());
            dandori_module_api::GitHostProvider::test_connection(&p)
                .await
                .map_err(|e| e.to_string())
        }
        "linear" => {
            let c = LinearClient::new(client, base_url.clone(), token.to_string());
            c.test_connection().await
        }
        _ => Err(format!("Nhà cung cấp không hỗ trợ: {provider}")),
    }
}

/// `GET /workspaces/:ws_id/connections`: Danh sách kết nối của workspace.
pub async fn list_connections(
    State(state): State<IntegrationsState>,
    Path(ws_id): Path<String>,
    headers: HeaderMap,
) -> axum::response::Response {
    if !check_ws_access(&state.pool, &headers, &ws_id).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    let rows: Vec<IntegrationConnection> = sqlx::query_as(
        "SELECT id, workspace_id, provider, name, base_url, token_encrypted, \
         allow_private_network, status, created_at, updated_at \
         FROM m_integration_connection WHERE workspace_id = ? ORDER BY created_at DESC",
    )
    .bind(&ws_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let public_list: Vec<PublicIntegrationConnection> = rows.into_iter().map(Into::into).collect();
    Json(json!({ "connections": public_list })).into_response()
}

async fn insert_connection(
    pool: &SqlitePool,
    conn: &IntegrationConnection,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO m_integration_connection \
         (id, workspace_id, provider, name, base_url, token_encrypted, allow_private_network, status, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&conn.id)
    .bind(&conn.workspace_id)
    .bind(&conn.provider)
    .bind(&conn.name)
    .bind(&conn.base_url)
    .bind(&conn.token_encrypted)
    .bind(conn.allow_private_network)
    .bind(&conn.status)
    .bind(&conn.created_at)
    .bind(&conn.updated_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Lưu token vào vault rồi ghi kết nối (cột chỉ giữ tham chiếu `vault:<id>`).
async fn save_connection(
    pool: &SqlitePool,
    ws_id: &str,
    req: &CreateConnectionRequest,
    parsed_url: &url::Url,
) -> axum::response::Response {
    // Token không bao giờ lưu thô: mã hóa vào vault, cột chỉ giữ tham chiếu `vault:<id>`.
    let secret_id = match token_vault(pool)
        .put(
            &format!("integration:{}:{}", req.provider, req.name),
            SecretBox::from(req.token.clone()),
        )
        .await
    {
        Ok(id) => id,
        Err(e) => {
            return err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Không lưu được token vào vault: {e}"),
            );
        }
    };
    let conn = IntegrationConnection::new(
        ws_id,
        &req.provider,
        &req.name,
        parsed_url.as_str(),
        &format!("{VAULT_REF_PREFIX}{secret_id}"),
        req.allow_private_network,
    );

    if let Err(e) = insert_connection(pool, &conn).await {
        let _ = token_vault(pool).delete(&secret_id).await;
        return err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Lỗi lưu kết nối: {e}"),
        );
    }

    let public_conn: PublicIntegrationConnection = conn.into();
    (
        StatusCode::CREATED,
        Json(json!({ "connection": public_conn })),
    )
        .into_response()
}

/// `POST /workspaces/:ws_id/connections`: Tạo kết nối mới.
pub async fn create_connection(
    State(state): State<IntegrationsState>,
    Path(ws_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<CreateConnectionRequest>,
) -> axum::response::Response {
    if let Err(resp) =
        authorize_connection_change(&state.pool, &headers, &ws_id, req.allow_private_network).await
    {
        return *resp;
    }

    let raw_url = req
        .base_url
        .clone()
        .unwrap_or_else(|| default_base_url(&req.provider).to_string());
    let parsed_url =
        match validate_connection_url(&req.provider, &raw_url, req.allow_private_network) {
            Ok(u) => u,
            Err(e) => return err_resp(StatusCode::BAD_REQUEST, &e.to_string()),
        };

    if let Err(e) = test_provider_connection(
        &req.provider,
        &parsed_url,
        &req.token,
        req.allow_private_network,
    )
    .await
    {
        return err_resp(
            StatusCode::BAD_REQUEST,
            &format!("Kiểm tra kết nối thất bại: {e}"),
        );
    }

    save_connection(&state.pool, &ws_id, &req, &parsed_url).await
}

/// `POST /workspaces/:ws_id/connections/test`: Kiểm tra tham số kết nối trước khi lưu.
pub async fn test_preflight_connection(
    State(state): State<IntegrationsState>,
    Path(ws_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<CreateConnectionRequest>,
) -> axum::response::Response {
    if let Err(resp) =
        authorize_connection_change(&state.pool, &headers, &ws_id, req.allow_private_network).await
    {
        return *resp;
    }

    let raw_url = req
        .base_url
        .unwrap_or_else(|| default_base_url(&req.provider).to_string());
    let parsed_url =
        match validate_connection_url(&req.provider, &raw_url, req.allow_private_network) {
            Ok(u) => u,
            Err(e) => return err_resp(StatusCode::BAD_REQUEST, &e.to_string()),
        };

    match test_provider_connection(
        &req.provider,
        &parsed_url,
        &req.token,
        req.allow_private_network,
    )
    .await
    {
        Ok(()) => Json(json!({ "success": true, "message": "Kiểm tra kết nối thành công" }))
            .into_response(),
        Err(e) => err_resp(
            StatusCode::BAD_REQUEST,
            &format!("Kiểm tra kết nối thất bại: {e}"),
        ),
    }
}

/// `POST /connections/:conn_id/test`: Kiểm tra tính hợp lệ của kết nối đã lưu.
pub async fn test_connection_handler(
    State(state): State<IntegrationsState>,
    Path(conn_id): Path<String>,
    headers: HeaderMap,
) -> axum::response::Response {
    let conn: Option<IntegrationConnection> = sqlx::query_as(
        "SELECT id, workspace_id, provider, name, base_url, token_encrypted, \
         allow_private_network, status, created_at, updated_at \
         FROM m_integration_connection WHERE id = ?",
    )
    .bind(&conn_id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let Some(c) = conn else {
        return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy kết nối");
    };

    if !check_ws_access(&state.pool, &headers, &c.workspace_id).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    let Ok(url) = url::Url::parse(&c.base_url) else {
        return err_resp(StatusCode::BAD_REQUEST, "URL cơ sở không hợp lệ");
    };

    let token = match reveal_token(&state.pool, &c.token_encrypted).await {
        Ok(t) => t,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e),
    };

    match test_provider_connection(&c.provider, &url, &token, c.allow_private_network).await {
        Ok(()) => Json(json!({ "success": true, "message": "Kết nối thành công" })).into_response(),
        Err(e) => err_resp(
            StatusCode::BAD_REQUEST,
            &format!("Kiểm tra kết nối thất bại: {e}"),
        ),
    }
}

/// `DELETE /connections/:conn_id`: Xóa kết nối.
pub async fn delete_connection(
    State(state): State<IntegrationsState>,
    Path(conn_id): Path<String>,
    headers: HeaderMap,
) -> axum::response::Response {
    let conn: Option<IntegrationConnection> = sqlx::query_as(
        "SELECT id, workspace_id, provider, name, base_url, token_encrypted, \
         allow_private_network, status, created_at, updated_at \
         FROM m_integration_connection WHERE id = ?",
    )
    .bind(&conn_id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let Some(c) = conn else {
        return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy kết nối");
    };

    if let Err(resp) =
        authorize_connection_change(&state.pool, &headers, &c.workspace_id, false).await
    {
        return *resp;
    }

    if let Some(id) = c.token_encrypted.strip_prefix(VAULT_REF_PREFIX)
        && let Ok(secret_id) = id.parse()
    {
        let _ = token_vault(&state.pool).delete(&secret_id).await;
    }

    let _ = sqlx::query("DELETE FROM m_integration_connection WHERE id = ?")
        .bind(&conn_id)
        .execute(&state.pool)
        .await;

    StatusCode::NO_CONTENT.into_response()
}

/// `GET /connections/:conn_id/linear/issues`: Lấy danh sách issues từ Linear.
pub async fn list_linear_issues(
    State(state): State<IntegrationsState>,
    Path(conn_id): Path<String>,
    headers: HeaderMap,
) -> axum::response::Response {
    let conn: Option<IntegrationConnection> = sqlx::query_as(
        "SELECT id, workspace_id, provider, name, base_url, token_encrypted, \
         allow_private_network, status, created_at, updated_at \
         FROM m_integration_connection WHERE id = ?",
    )
    .bind(&conn_id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let Some(c) = conn else {
        return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy kết nối");
    };

    if !check_ws_access(&state.pool, &headers, &c.workspace_id).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    let Ok(url) = url::Url::parse(&c.base_url) else {
        return err_resp(StatusCode::BAD_REQUEST, "URL cơ sở không hợp lệ");
    };

    let Ok(client) = build_safe_client(&url, c.allow_private_network).await else {
        return err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Lỗi khởi tạo HTTP client",
        );
    };

    let token = match reveal_token(&state.pool, &c.token_encrypted).await {
        Ok(t) => t,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e),
    };
    let linear = LinearClient::new(client, url, token);
    match linear.list_issues().await {
        Ok(issues) => Json(json!({ "issues": issues })).into_response(),
        Err(e) => err_resp(StatusCode::BAD_GATEWAY, &format!("Lỗi gọi Linear API: {e}")),
    }
}

/// `POST /connections/:conn_id/linear/import`: Nhập issue Linear vào backlog dự án.
pub async fn import_linear_issues(
    State(state): State<IntegrationsState>,
    Path(conn_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<LinearImportRequest>,
) -> axum::response::Response {
    let conn: Option<IntegrationConnection> = sqlx::query_as(
        "SELECT id, workspace_id, provider, name, base_url, token_encrypted, \
         allow_private_network, status, created_at, updated_at \
         FROM m_integration_connection WHERE id = ?",
    )
    .bind(&conn_id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let Some(c) = conn else {
        return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy kết nối");
    };

    if !check_ws_access(&state.pool, &headers, &c.workspace_id).await {
        return err_resp(StatusCode::FORBIDDEN, "Không có quyền truy cập workspace");
    }

    let Ok(url) = url::Url::parse(&c.base_url) else {
        return err_resp(StatusCode::BAD_REQUEST, "URL cơ sở không hợp lệ");
    };

    let Ok(client) = build_safe_client(&url, c.allow_private_network).await else {
        return err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Lỗi khởi tạo HTTP client",
        );
    };

    let token = match reveal_token(&state.pool, &c.token_encrypted).await {
        Ok(t) => t,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e),
    };
    let linear = LinearClient::new(client, url, token);
    match linear
        .import_issues(
            &state.pool,
            &c.workspace_id,
            &req.project_id,
            &req.issue_ids,
        )
        .await
    {
        Ok(count) => Json(json!({ "imported_count": count })).into_response(),
        Err(e) => err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Lỗi nhập issue: {e}"),
        ),
    }
}

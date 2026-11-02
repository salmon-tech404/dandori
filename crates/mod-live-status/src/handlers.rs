//! Handlers xử lý HTTP và luồng SSE cho module live-status.

use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use dandori_module_api::{EventBus, EventFilter, EventSubscription, hash_session_token};
use dandori_types::{Envelope, EventPayload, WorkspaceId};
use futures::StreamExt;
use futures::channel::mpsc;
use serde::Deserialize;
use serde_json::json;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::filter::{WorkspaceScope, event_kind_to_string, is_allowed_kind, matches_scope};
use crate::state::{ConnectionGuard, LiveStatusState, RecentEventBuffer};

/// Tham số truy vấn cho endpoint stream SSE.
#[derive(Debug, Deserialize, Default)]
pub struct StreamQuery {
    /// Định danh không gian làm việc.
    pub workspace_id: Option<String>,
    /// Định danh dự án.
    pub project_id: Option<String>,
    /// Định danh nhiệm vụ.
    pub task_id: Option<String>,
    /// Định danh lần chạy.
    pub run_id: Option<String>,
    /// Định danh sự kiện cuối cùng nhận được (dùng khi nối lại).
    pub last_event_id: Option<String>,
}

/// Trạng thái ứng dụng của route live-status.
#[derive(Clone, Debug)]
pub struct AppState {
    /// Trạng thái quản lý kết nối và buffer sự kiện.
    pub module_state: LiveStatusState,
    /// Bus sự kiện hệ thống.
    pub event_bus: Arc<EventBus>,
    /// Pool cơ sở dữ liệu nếu có.
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

fn extract_cookie(headers: &HeaderMap, cookie_name: &str) -> Option<String> {
    let cookie_header = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    for pair in cookie_header.split(';') {
        let mut parts = pair.trim().splitn(2, '=');
        let key = parts.next()?.trim();
        let val = parts.next()?.trim();
        if key == cookie_name {
            return Some(val.to_string());
        }
    }
    None
}

async fn get_user_from_session(pool: &SqlitePool, token: &str) -> Option<(String, String)> {
    let hashed = hash_session_token(token);
    let row = sqlx::query(
        "SELECT u.id, u.system_role FROM user u
         JOIN session s ON s.user_id = u.id
         WHERE s.token_hash = ? AND s.expires_at > CURRENT_TIMESTAMP",
    )
    .bind(hashed)
    .fetch_optional(pool)
    .await
    .ok()??;

    let user_id: String = row.get(0);
    let role: String = row.get(1);
    Some((user_id, role))
}

async fn check_workspace_access(
    pool: &SqlitePool,
    user_id: &str,
    system_role: &str,
    workspace_id: &str,
) -> bool {
    if system_role == "owner" || system_role == "admin" {
        return true;
    }
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM workspace_member WHERE workspace_id = ? AND user_id = ?",
    )
    .bind(workspace_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    count > 0
}

async fn authenticate_request(
    pool_opt: &Option<SqlitePool>,
    headers: &HeaderMap,
    ws_id_opt: Option<&str>,
) -> Result<(String, String), Box<Response>> {
    let pool = match pool_opt {
        Some(p) => p,
        None => return Ok(("test_user".to_string(), "admin".to_string())),
    };

    let session_token = extract_cookie(headers, "dandori_session").ok_or_else(|| {
        Box::new(error_response(
            "unauthorized",
            "Thiếu phiên đăng nhập",
            StatusCode::UNAUTHORIZED,
        ))
    })?;

    let (user_id, system_role) = get_user_from_session(pool, &session_token)
        .await
        .ok_or_else(|| {
            Box::new(error_response(
                "unauthorized",
                "Phiên không hợp lệ hoặc đã hết hạn",
                StatusCode::UNAUTHORIZED,
            ))
        })?;

    if let Some(ws_id) = ws_id_opt
        && !check_workspace_access(pool, &user_id, &system_role, ws_id).await
    {
        return Err(Box::new(error_response(
            "forbidden",
            "Không có quyền truy cập không gian làm việc này",
            StatusCode::FORBIDDEN,
        )));
    }

    Ok((user_id, system_role))
}

/// Xác định các workspace mà người dùng được xem. Không có `workspace_id` yêu cầu thì chỉ trả về
/// những workspace người đó là thành viên (quản trị hệ thống thấy tất cả), tránh lộ sự kiện chéo workspace.
async fn resolve_scope(
    pool_opt: &Option<SqlitePool>,
    user_id: &str,
    system_role: &str,
    requested_ws: Option<&str>,
) -> WorkspaceScope {
    if let Some(ws) = requested_ws {
        return WorkspaceScope::Only(ws.parse::<WorkspaceId>().into_iter().collect());
    }
    let Some(pool) = pool_opt else {
        return WorkspaceScope::All;
    };
    if system_role == "owner" || system_role == "admin" {
        return WorkspaceScope::All;
    }
    let rows: Vec<String> =
        sqlx::query_scalar("SELECT workspace_id FROM workspace_member WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
    WorkspaceScope::Only(
        rows.iter()
            .filter_map(|w| w.parse::<WorkspaceId>().ok())
            .collect(),
    )
}

fn create_sse_event(env: &Envelope) -> Event {
    let kind_str = event_kind_to_string(&env.kind);
    let data = serde_json::to_string(env).unwrap_or_else(|_| "{}".to_string());
    Event::default()
        .id(env.id.to_string())
        .event(kind_str)
        .data(data)
}

async fn run_stream_loop(
    _guard: ConnectionGuard,
    mut sub: EventSubscription,
    mut tx: mpsc::Sender<Event>,
    last_event_id: Option<String>,
    scope: WorkspaceScope,
    recent_events: Arc<RecentEventBuffer>,
) {
    if let Some(ref last_id) = last_event_id {
        let missed = recent_events.events_after(last_id, &scope);
        for env in missed {
            if tx.try_send(create_sse_event(&env)).is_err() {
                return;
            }
        }
    }

    let mut interval = tokio::time::interval(Duration::from_secs(15));
    interval.tick().await;

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let ping = Event::default().event("ping").data("{}");
                if tx.try_send(ping).is_err() {
                    break;
                }
            }
            env_opt = sub.recv() => {
                match env_opt {
                    Some(env) => {
                        if let EventPayload::ModuleDisabled { ref module_id } = env.payload
                            && module_id.as_str() == "live-status"
                        {
                            let disabled_ev = Event::default()
                                .event("module_disabled")
                                .data(r#"{"message":"module đang tắt"}"#);
                            let _ = tx.try_send(disabled_ev);
                            break;
                        }
                        if is_allowed_kind(&env.kind) && matches_scope(&env, &scope) {
                            recent_events.record(env.clone());
                            let ev = create_sse_event(&env);
                            match tx.try_send(ev) {
                                Ok(()) => {}
                                Err(e) if e.is_full() => {
                                    tracing::debug!("SSE client chậm, bỏ qua sự kiện");
                                }
                                Err(_) => break,
                            }
                        }
                    }
                    None => break,
                }
            }
        }
    }
}

/// Handler xử lý `GET /stream` (SSE).
pub async fn stream_handler(
    State(app_state): State<AppState>,
    Query(query): Query<StreamQuery>,
    headers: HeaderMap,
) -> Response {
    let ws_id = query.workspace_id.as_deref();
    let (user_id, system_role) = match authenticate_request(&app_state.pool, &headers, ws_id).await
    {
        Ok(auth) => auth,
        Err(resp) => return *resp,
    };

    let guard = match app_state.module_state.acquire_connection(&user_id) {
        Ok(g) => g,
        Err(msg) => {
            return error_response("too_many_connections", msg, StatusCode::TOO_MANY_REQUESTS);
        }
    };

    let last_event_id = headers
        .get("Last-Event-ID")
        .and_then(|h| h.to_str().ok().map(ToString::to_string))
        .or(query.last_event_id);

    let scope = resolve_scope(&app_state.pool, &user_id, &system_role, ws_id).await;
    let sub = app_state.event_bus.subscribe(EventFilter::all());
    let (tx, rx) = mpsc::channel(128);
    let recent_events = app_state.module_state.recent_events().clone();

    tokio::spawn(run_stream_loop(
        guard,
        sub,
        tx,
        last_event_id,
        scope,
        recent_events,
    ));

    let stream = rx.map(Ok::<_, std::convert::Infallible>);
    Sse::new(stream).into_response()
}

/// Handler xử lý `GET /status`.
pub async fn status_handler(State(app_state): State<AppState>) -> Response {
    Json(json!({
        "module": "live-status",
        "status": "ok",
        "total_connections": app_state.module_state.total_connections(),
        "recent_events_count": app_state.module_state.recent_events().len()
    }))
    .into_response()
}

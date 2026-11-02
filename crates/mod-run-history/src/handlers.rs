//! Handlers xử lý các yêu cầu API của module `run-history`.

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use dandori_module_api::{
    ModuleContext, SessionUser, accessible_workspace_ids, can_access_workspace,
    contains_secret_like, redact_secrets, session_user,
};
use serde_json::json;
use sqlx::SqlitePool;

use crate::model::{
    ExportQuery, ListEventsQuery, ListRunsQuery, RunDetail, RunEventItem, RunHistoryNote,
    RunSummary, UpdateNotePayload,
};

type RunDbRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    Option<String>,
    Option<i64>,
    Option<String>,
    Option<String>,
);

async fn find_run_workspace(pool: &SqlitePool, run_id: &str) -> Option<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT p.workspace_id \
         FROM run r \
         JOIN task t ON t.id = r.task_id \
         JOIN project p ON p.id = t.project_id \
         WHERE r.id = ?",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

async fn check_run_access(
    pool: &SqlitePool,
    headers: &HeaderMap,
    run_id: &str,
) -> Result<(SessionUser, String), (StatusCode, &'static str)> {
    let Some(user) = session_user(pool, headers).await else {
        return Err((StatusCode::UNAUTHORIZED, "Chưa đăng nhập"));
    };
    let Some(ws_id) = find_run_workspace(pool, run_id).await else {
        return Err((StatusCode::NOT_FOUND, "Không tìm thấy lần chạy"));
    };
    if !can_access_workspace(pool, &user, &ws_id).await {
        return Err((
            StatusCode::FORBIDDEN,
            "Không có quyền truy cập workspace này",
        ));
    }
    Ok((user, ws_id))
}

fn calculate_duration(start: &str, finished: Option<&str>) -> Option<i64> {
    let end = finished?;
    let s = chrono::DateTime::parse_from_rfc3339(start).ok()?;
    let e = chrono::DateTime::parse_from_rfc3339(end).ok()?;
    Some((e - s).num_seconds().max(0))
}

fn map_run_row(row: RunDbRow) -> RunSummary {
    let duration = calculate_duration(&row.6, row.7.as_deref());
    let tags: Vec<String> = row
        .10
        .as_deref()
        .and_then(|t| serde_json::from_str(t).ok())
        .unwrap_or_default();

    RunSummary {
        id: row.0,
        task_id: row.1,
        task_title: row.2,
        agent: row.3,
        account: row.4,
        status: row.5,
        started_at: row.6,
        finished_at: row.7,
        exit_code: row.8,
        duration_seconds: duration,
        note: row.9,
        tags,
    }
}

async fn verify_task_access(
    pool: &SqlitePool,
    user: &SessionUser,
    task_id: &str,
) -> Result<(), (StatusCode, &'static str)> {
    let ws_id: Option<String> = sqlx::query_scalar(
        "SELECT p.workspace_id FROM task t JOIN project p ON p.id = t.project_id WHERE t.id = ?",
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some(ws) = ws_id else {
        return Err((StatusCode::NOT_FOUND, "Không tìm thấy task"));
    };
    if !can_access_workspace(pool, user, &ws).await {
        return Err((
            StatusCode::FORBIDDEN,
            "Không có quyền truy cập workspace này",
        ));
    }
    Ok(())
}

fn build_list_runs_sql(query: &ListRunsQuery, accessible_ws: Option<&[String]>) -> String {
    let mut sql = String::from(
        "SELECT r.id, r.task_id, t.title as task_title, r.agent, r.account, r.status, \
                r.started_at, r.finished_at, r.exit_code, \
                n.note, n.tags \
         FROM run r \
         JOIN task t ON t.id = r.task_id \
         JOIN project p ON p.id = t.project_id \
         LEFT JOIN m_run_history_note n ON n.run_id = r.id \
         WHERE 1=1 ",
    );
    if let Some(ws_list) = accessible_ws {
        let ph = ws_list.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        sql.push_str(&format!("AND p.workspace_id IN ({ph}) "));
    }
    if query.task_id.is_some() {
        sql.push_str("AND r.task_id = ? ");
    }
    if query.agent_provider.is_some() {
        sql.push_str("AND r.agent = ? ");
    }
    if query.agent_account_id.is_some() {
        sql.push_str("AND r.account = ? ");
    }
    if query.status.is_some() {
        sql.push_str("AND r.status = ? ");
    }
    if query.from.is_some() {
        sql.push_str("AND r.started_at >= ? ");
    }
    if query.to.is_some() {
        sql.push_str("AND r.started_at <= ? ");
    }
    sql.push_str("ORDER BY r.started_at DESC, r.id DESC LIMIT ? OFFSET ?");
    sql
}

async fn execute_list_runs(
    pool: &SqlitePool,
    sql: &str,
    query: &ListRunsQuery,
    accessible_ws: Option<&[String]>,
    limit: i64,
    offset: i64,
) -> Result<Vec<RunSummary>, sqlx::Error> {
    let mut q = sqlx::query_as::<_, RunDbRow>(sql);
    if let Some(ws_list) = accessible_ws {
        for ws in ws_list {
            q = q.bind(ws);
        }
    }
    if let Some(ref task_id) = query.task_id {
        q = q.bind(task_id);
    }
    if let Some(ref agent) = query.agent_provider {
        q = q.bind(agent);
    }
    if let Some(ref account) = query.agent_account_id {
        q = q.bind(account);
    }
    if let Some(ref status) = query.status {
        q = q.bind(status);
    }
    if let Some(ref from) = query.from {
        q = q.bind(from);
    }
    if let Some(ref to) = query.to {
        q = q.bind(to);
    }
    q = q.bind(limit).bind(offset);

    let rows = q.fetch_all(pool).await?;
    Ok(rows.into_iter().map(map_run_row).collect())
}

/// Lấy danh sách lần chạy có lọc và phân trang.
pub async fn list_runs(
    State(ctx): State<ModuleContext>,
    headers: HeaderMap,
    Query(query): Query<ListRunsQuery>,
) -> Response {
    let Some(pool) = ctx.pool() else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Thiếu database pool").into_response();
    };
    let Some(user) = session_user(pool, &headers).await else {
        return (StatusCode::UNAUTHORIZED, "Chưa đăng nhập").into_response();
    };
    if let Some(ref t_id) = query.task_id {
        let access = verify_task_access(pool, &user, t_id).await;
        if let Err((code, msg)) = access {
            return (code, msg).into_response();
        }
    }
    let accessible_ws = accessible_workspace_ids(pool, &user).await;
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    let sql = build_list_runs_sql(&query, accessible_ws.as_deref());
    match execute_list_runs(pool, &sql, &query, accessible_ws.as_deref(), limit, offset).await {
        Ok(runs) => Json(json!({ "runs": runs, "limit": limit, "offset": offset })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// Lấy thông tin chi tiết một lần chạy.
pub async fn get_run(
    State(ctx): State<ModuleContext>,
    headers: HeaderMap,
    Path(run_id): Path<String>,
) -> Response {
    let Some(pool) = ctx.pool() else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Thiếu database pool").into_response();
    };
    let ws_id = match check_run_access(pool, &headers, &run_id).await {
        Ok((_, ws)) => ws,
        Err((code, msg)) => return (code, msg).into_response(),
    };
    let sql = "SELECT r.id, r.task_id, t.title as task_title, r.agent, r.account, r.status, \
                      r.started_at, r.finished_at, r.exit_code, n.note, n.tags \
               FROM run r \
               JOIN task t ON t.id = r.task_id \
               LEFT JOIN m_run_history_note n ON n.run_id = r.id \
               WHERE r.id = ?";
    let row = sqlx::query_as::<_, RunDbRow>(sql)
        .bind(&run_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

    let Some(row) = row else {
        return (StatusCode::NOT_FOUND, "Không tìm thấy lần chạy").into_response();
    };
    let reason = match row.5.as_str() {
        "success" => Some("Hoàn thành thành công".to_string()),
        "cancelled" => Some("Đã hủy bởi người dùng".to_string()),
        "failed" => Some(format!("Thất bại với mã thoát {:?}", row.8)),
        _ => None,
    };
    Json(RunDetail {
        summary: map_run_row(row),
        workspace_id: ws_id,
        termination_reason: reason,
    })
    .into_response()
}

/// Lấy danh sách sự kiện log để phát lại log của lần chạy.
pub async fn get_run_events(
    State(ctx): State<ModuleContext>,
    headers: HeaderMap,
    Path(run_id): Path<String>,
    Query(query): Query<ListEventsQuery>,
) -> Response {
    let Some(pool) = ctx.pool() else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Thiếu database pool").into_response();
    };
    if let Err((code, msg)) = check_run_access(pool, &headers, &run_id).await {
        return (code, msg).into_response();
    }
    let limit = query.limit.unwrap_or(100).clamp(1, 1000);
    let offset = query.offset.unwrap_or(0).max(0);
    let rows = sqlx::query_as::<_, (String, String, i64, String, String, String)>(
        "SELECT id, run_id, seq, event_type, data, created_at \
         FROM run_event WHERE run_id = ? ORDER BY seq ASC, id ASC LIMIT ? OFFSET ?",
    )
    .bind(&run_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(r) => {
            let events = r
                .into_iter()
                .map(|item| RunEventItem {
                    id: item.0,
                    run_id: item.1,
                    seq: item.2,
                    event_type: item.3,
                    data: item.4,
                    created_at: item.5,
                })
                .collect::<Vec<_>>();
            Json(json!({ "events": events, "limit": limit, "offset": offset })).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// Lấy ghi chú và nhãn của lần chạy.
pub async fn get_note(
    State(ctx): State<ModuleContext>,
    headers: HeaderMap,
    Path(run_id): Path<String>,
) -> Response {
    let Some(pool) = ctx.pool() else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Thiếu database pool").into_response();
    };
    if let Err((code, msg)) = check_run_access(pool, &headers, &run_id).await {
        return (code, msg).into_response();
    }
    let note_row = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT run_id, note, tags, updated_at FROM m_run_history_note WHERE run_id = ?",
    )
    .bind(&run_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    match note_row {
        Some((r_id, note, tags_raw, updated_at)) => {
            let tags: Vec<String> = serde_json::from_str(&tags_raw).unwrap_or_default();
            Json(RunHistoryNote {
                run_id: r_id,
                note,
                tags,
                updated_at,
            })
            .into_response()
        }
        None => Json(RunHistoryNote {
            run_id,
            note: String::new(),
            tags: Vec::new(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        })
        .into_response(),
    }
}

/// Cập nhật ghi chú và nhãn cho lần chạy.
pub async fn update_note(
    State(ctx): State<ModuleContext>,
    headers: HeaderMap,
    Path(run_id): Path<String>,
    Json(payload): Json<UpdateNotePayload>,
) -> Response {
    let Some(pool) = ctx.pool() else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Thiếu database pool").into_response();
    };
    if let Err((code, msg)) = check_run_access(pool, &headers, &run_id).await {
        return (code, msg).into_response();
    }
    if contains_secret_like(&payload.note) {
        return (
            StatusCode::BAD_REQUEST,
            "Ghi chú không được chứa chuỗi giống khóa bí mật hoặc token",
        )
            .into_response();
    }
    let tags_json = serde_json::to_string(&payload.tags).unwrap_or_else(|_| "[]".to_string());
    let now = chrono::Utc::now().to_rfc3339();
    let query_res = sqlx::query(
        "INSERT INTO m_run_history_note (run_id, note, tags, updated_at) VALUES (?, ?, ?, ?) \
         ON CONFLICT (run_id) DO UPDATE SET note = excluded.note, tags = excluded.tags, updated_at = excluded.updated_at",
    )
    .bind(&run_id)
    .bind(&payload.note)
    .bind(&tags_json)
    .bind(&now)
    .execute(pool)
    .await;

    if query_res.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Không thể lưu ghi chú").into_response();
    }
    Json(RunHistoryNote {
        run_id,
        note: payload.note,
        tags: payload.tags,
        updated_at: now,
    })
    .into_response()
}

fn render_export_markdown(row: &RunDbRow, events: &[RunEventItem]) -> String {
    let tags: Vec<String> = row
        .10
        .as_deref()
        .and_then(|t| serde_json::from_str(t).ok())
        .unwrap_or_default();
    let mut md = String::new();
    md.push_str(&format!("# Lịch sử lần chạy: {}\n\n", row.0));
    md.push_str(&format!(
        "- **Task:** {} ({})\n",
        redact_secrets(&row.2),
        row.1
    ));
    md.push_str(&format!("- **Agent:** {} ({})\n", row.3, row.4));
    md.push_str(&format!("- **Trạng thái:** {}\n", row.5));
    md.push_str(&format!("- **Bắt đầu:** {}\n", row.6));
    if let Some(ref fin) = row.7 {
        md.push_str(&format!("- **Kết thúc:** {fin}\n"));
    }
    if let Some(code) = row.8 {
        md.push_str(&format!("- **Mã thoát:** {code}\n"));
    }
    if let Some(ref n) = row.9 {
        md.push_str(&format!("- **Ghi chú:** {}\n", redact_secrets(n)));
    }
    if !tags.is_empty() {
        md.push_str(&format!("- **Nhãn:** {}\n", tags.join(", ")));
    }
    md.push_str("\n## Nhật ký sự kiện (Replay Log)\n\n");
    for ev in events {
        md.push_str(&format!(
            "[{}] #{} [{}] {}\n",
            ev.created_at, ev.seq, ev.event_type, ev.data
        ));
    }
    md
}

fn render_export_json(row: &RunDbRow, events: &[RunEventItem]) -> String {
    let tags: Vec<String> = row
        .10
        .as_deref()
        .and_then(|t| serde_json::from_str(t).ok())
        .unwrap_or_default();
    let export_data = json!({
        "run_id": row.0,
        "task_id": row.1,
        "task_title": redact_secrets(&row.2),
        "agent": row.3,
        "account": row.4,
        "status": row.5,
        "started_at": row.6,
        "finished_at": row.7,
        "exit_code": row.8,
        "note": row.9.as_deref().map(redact_secrets),
        "tags": tags,
        "events": events,
    });
    export_data.to_string()
}

async fn fetch_redacted_events(pool: &SqlitePool, run_id: &str) -> Vec<RunEventItem> {
    let event_rows = sqlx::query_as::<_, (String, String, i64, String, String, String)>(
        "SELECT id, run_id, seq, event_type, data, created_at \
         FROM run_event WHERE run_id = ? ORDER BY seq ASC, id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    event_rows
        .into_iter()
        .map(|r| RunEventItem {
            id: r.0,
            run_id: r.1,
            seq: r.2,
            event_type: r.3,
            data: redact_secrets(&r.4),
            created_at: r.5,
        })
        .collect()
}

/// Xuất lần chạy ra định dạng Markdown hoặc JSON đã che bí mật.
pub async fn export_run(
    State(ctx): State<ModuleContext>,
    headers: HeaderMap,
    Path(run_id): Path<String>,
    Query(query): Query<ExportQuery>,
) -> Response {
    let Some(pool) = ctx.pool() else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Thiếu database pool").into_response();
    };
    if let Err((code, msg)) = check_run_access(pool, &headers, &run_id).await {
        return (code, msg).into_response();
    }
    let sql = "SELECT r.id, r.task_id, t.title as task_title, r.agent, r.account, r.status, \
                      r.started_at, r.finished_at, r.exit_code, n.note, n.tags \
               FROM run r JOIN task t ON t.id = r.task_id LEFT JOIN m_run_history_note n ON n.run_id = r.id \
               WHERE r.id = ?";
    let Some(row) = sqlx::query_as::<_, RunDbRow>(sql)
        .bind(&run_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
    else {
        return (StatusCode::NOT_FOUND, "Không tìm thấy lần chạy").into_response();
    };
    let redacted_events = fetch_redacted_events(pool, &run_id).await;

    if query.format.as_deref() == Some("json") {
        return (
            StatusCode::OK,
            [("content-type", "application/json")],
            render_export_json(&row, &redacted_events),
        )
            .into_response();
    }
    (
        StatusCode::OK,
        [("content-type", "text/markdown; charset=utf-8")],
        render_export_markdown(&row, &redacted_events),
    )
        .into_response()
}

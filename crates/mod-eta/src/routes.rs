//! Các API routes cho module `eta` (P3-07).

use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use dandori_module_api::{ModuleContext, SessionUser, can_access_workspace, session_user};
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::calculator::evaluate_eta;
use crate::db::{
    fetch_matching_run_durations, get_accuracy_stats, get_recent_mae_ratio, record_actual_duration,
    save_prediction,
};
use crate::model::{EtaPredictionRecord, EtaResult};

/// Trạng thái Router của module eta.
#[derive(Clone, Debug)]
pub struct EtaState {
    pub ctx: ModuleContext,
}

impl EtaState {
    /// Khởi tạo trạng thái mới.
    #[must_use]
    pub fn new(ctx: ModuleContext) -> Self {
        Self { ctx }
    }
}

#[derive(Debug, Deserialize)]
pub struct RecordCompletionPayload {
    pub run_id: String,
    pub actual_duration_secs: f64,
}

fn err_resp(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

fn get_pool(state: &EtaState) -> Result<SqlitePool, (StatusCode, &'static str)> {
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

/// Xây dựng Router cho module eta.
pub fn eta_router(state: EtaState) -> Router {
    Router::new()
        .route("/workspaces/{ws_id}/eta/task/{task_id}", get(get_task_eta))
        .route("/workspaces/{ws_id}/eta/run/{run_id}", get(get_run_eta))
        .route(
            "/workspaces/{ws_id}/eta/accuracy",
            get(get_accuracy_summary),
        )
        .route(
            "/workspaces/{ws_id}/eta/record-completion",
            post(post_record_completion),
        )
        .with_state(state)
}

#[derive(sqlx::FromRow)]
struct TaskRow {
    project_id: String,
    worker_agent: String,
}

#[derive(sqlx::FromRow)]
struct ActiveRunRow {
    id: String,
    agent: String,
    started_at: String,
}

async fn resolve_task_and_run(
    pool: &SqlitePool,
    task_id: &str,
) -> Option<(TaskRow, Option<ActiveRunRow>)> {
    let task: TaskRow = sqlx::query_as("SELECT project_id, worker_agent FROM task WHERE id = ?")
        .bind(task_id)
        .fetch_optional(pool)
        .await
        .ok()??;

    let active_run: Option<ActiveRunRow> = sqlx::query_as(
        r#"
        SELECT id, agent, started_at
        FROM run
        WHERE task_id = ?
        ORDER BY started_at DESC
        LIMIT 1
        "#,
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    Some((task, active_run))
}

struct PredictionMeta<'a> {
    ws_id: &'a str,
    project_id: &'a str,
    task_id: &'a str,
    run_id: &'a str,
    provider: &'a str,
    sample_count: usize,
}

async fn persist_valid_prediction(
    pool: &SqlitePool,
    meta: PredictionMeta<'_>,
    eta_result: &EtaResult,
) {
    if let (Some(med), Some(p80), Some(conf)) = (
        eta_result.median_secs,
        eta_result.p80_secs,
        eta_result.confidence,
    ) {
        #[expect(clippy::cast_possible_wrap, reason = "Đếm số lượng mẫu nhỏ")]
        let pred_record = EtaPredictionRecord {
            id: Uuid::new_v4().to_string(),
            workspace_id: meta.ws_id.to_string(),
            project_id: meta.project_id.to_string(),
            task_id: meta.task_id.to_string(),
            run_id: meta.run_id.to_string(),
            role: "worker".to_string(),
            provider: meta.provider.to_string(),
            sample_count: meta.sample_count as i64,
            predicted_median_secs: med,
            predicted_p80_secs: p80,
            confidence: conf.as_str().to_string(),
            actual_duration_secs: None,
            absolute_error_secs: None,
            created_at: Utc::now().to_rfc3339(),
            completed_at: None,
        };
        let _ = save_prediction(pool, &pred_record).await;
    }
}

/// `GET /workspaces/{ws_id}/eta/task/{task_id}` - Tính toán hoặc lấy dự đoán ETA của một task.
async fn get_task_eta(
    State(state): State<EtaState>,
    AxumPath((ws_id, task_id)): AxumPath<(String, String)>,
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

    let Some((task, active_run)) = resolve_task_and_run(&pool, &task_id).await else {
        return err_resp(StatusCode::NOT_FOUND, "Không tìm thấy nhiệm vụ");
    };

    let (run_id, provider, start_time) = if let Some(ref r) = active_run {
        let st = DateTime::parse_from_rfc3339(&r.started_at)
            .ok()
            .map(|dt| dt.with_timezone(&Utc));
        (Some(r.id.as_str()), r.agent.as_str(), st)
    } else {
        (None, task.worker_agent.as_str(), None)
    };

    let role = "worker";
    let samples = fetch_matching_run_durations(&pool, &task.project_id, role, provider)
        .await
        .unwrap_or_default();
    let mae = get_recent_mae_ratio(&pool, &task.project_id, role, provider)
        .await
        .unwrap_or(None);

    let eta_result = evaluate_eta(&task_id, run_id, &samples, mae, start_time);
    if eta_result.has_estimate
        && let Some(r_id) = run_id
    {
        let meta = PredictionMeta {
            ws_id: &ws_id,
            project_id: &task.project_id,
            task_id: &task_id,
            run_id: r_id,
            provider,
            sample_count: samples.len(),
        };
        persist_valid_prediction(&pool, meta, &eta_result).await;
    }

    Json(json!({ "eta": eta_result })).into_response()
}

#[derive(sqlx::FromRow)]
struct PredRow {
    task_id: String,
    sample_count: i64,
    predicted_median_secs: f64,
    predicted_p80_secs: f64,
    confidence: String,
    actual_duration_secs: Option<f64>,
    absolute_error_secs: Option<f64>,
    created_at: String,
}

/// `GET /workspaces/{ws_id}/eta/run/{run_id}` - Tra cứu ETA của một lần chạy.
async fn get_run_eta(
    State(state): State<EtaState>,
    AxumPath((ws_id, run_id)): AxumPath<(String, String)>,
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

    let maybe_pred: Option<PredRow> = sqlx::query_as(
        r#"
        SELECT task_id, sample_count, predicted_median_secs, predicted_p80_secs,
               confidence, actual_duration_secs, absolute_error_secs, created_at
        FROM m_eta_prediction
        WHERE run_id = ?
        "#,
    )
    .bind(&run_id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    if let Some(pred) = maybe_pred {
        Json(json!({
            "eta": {
                "run_id": run_id,
                "task_id": pred.task_id,
                "has_estimate": true,
                "sample_count": pred.sample_count,
                "median_secs": pred.predicted_median_secs,
                "p80_secs": pred.predicted_p80_secs,
                "confidence": pred.confidence,
                "actual_duration_secs": pred.actual_duration_secs,
                "absolute_error_secs": pred.absolute_error_secs,
                "created_at": pred.created_at
            }
        }))
        .into_response()
    } else {
        err_resp(
            StatusCode::NOT_FOUND,
            "Không tìm thấy dự đoán ETA cho lần chạy này",
        )
    }
}

/// `GET /workspaces/{ws_id}/eta/accuracy` - Thống kê độ chính xác tự đo của ETA.
async fn get_accuracy_summary(
    State(state): State<EtaState>,
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

    match get_accuracy_stats(&pool, &ws_id, None).await {
        Ok(stats) => Json(json!({ "accuracy": stats })).into_response(),
        Err(e) => err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Lỗi truy vấn thống kê: {e}"),
        ),
    }
}

/// `POST /workspaces/{ws_id}/eta/record-completion` - Ghi nhận thời lượng thực tế của run.
async fn post_record_completion(
    State(state): State<EtaState>,
    AxumPath(ws_id): AxumPath<String>,
    headers: HeaderMap,
    Json(payload): Json<RecordCompletionPayload>,
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

    match record_actual_duration(&pool, &payload.run_id, payload.actual_duration_secs).await {
        Ok(true) => Json(json!({ "success": true, "recorded": true })).into_response(),
        Ok(false) => err_resp(
            StatusCode::NOT_FOUND,
            "Không tìm thấy dự đoán ETA cho run này để đối soát",
        ),
        Err(e) => err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Lỗi ghi nhận đối soát: {e}"),
        ),
    }
}

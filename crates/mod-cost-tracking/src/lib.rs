//! Module `cost-tracking` - Theo dõi token và chi phí thực tế theo nhiệm vụ và agent.

use std::sync::{Arc, LazyLock};

use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use chrono::Utc;
use dandori_module_api::{
    EventBus, Manifest, Migration, Module, ModuleContext, ModuleSubscription, WorkerSpec,
    session_user,
};
use dandori_types::{Envelope, EventPayload, EventScope, ProjectId, TaskId, WorkspaceId};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

#[expect(
    clippy::expect_used,
    reason = "embedded static module.json validated at build/test time"
)]
static MANIFEST: LazyLock<Manifest> = LazyLock::new(|| {
    let json = include_str!("../module.json");
    serde_json::from_str(json).expect("module.json của mod-cost-tracking phải hợp lệ")
});

static MIGRATIONS: [Migration; 1] = [Migration {
    id: "0001_cost_tracking",
    sql: include_str!("../migrations/0001_cost_tracking.sql"),
}];

#[derive(Clone)]
struct CostTrackingState {
    pool: Option<SqlitePool>,
    event_bus: Arc<EventBus>,
}

/// Module theo dõi token và chi phí thực tế.
#[derive(Debug, Default, Clone, Copy)]
pub struct CostTrackingModule;

impl CostTrackingModule {
    /// Tạo một thực thể CostTrackingModule mới.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for CostTrackingModule {
    fn manifest(&self) -> &Manifest {
        &MANIFEST
    }

    fn migrations(&self) -> &'static [Migration] {
        &MIGRATIONS
    }

    fn routes(&self, context: &ModuleContext) -> Router {
        let state = CostTrackingState {
            pool: context.pool().cloned(),
            event_bus: Arc::clone(context.event_bus()),
        };

        Router::new()
            .route(
                "/projects/{id}/pricing",
                get(get_project_pricing).put(set_project_pricing),
            )
            .route(
                "/projects/{id}/pricing/{pricing_id}",
                delete(delete_project_pricing),
            )
            .route("/projects/{id}/summary", get(get_project_summary))
            .route("/tasks/{id}/cost", get(get_task_cost))
            .route("/tasks/{id}/budget", put(set_task_budget))
            .route("/runs/{id}/record-tokens", post(record_run_tokens))
            .with_state(state)
    }

    fn workers(&self, _ctx: &ModuleContext) -> Vec<WorkerSpec> {
        Vec::new()
    }

    fn subscriptions(&self) -> Vec<ModuleSubscription> {
        Vec::new()
    }
}

// ==================== BẢN ĐỊNH NGHĨA DỮ LIỆU ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct PricingRule {
    pub id: String,
    pub project_id: String,
    pub provider: String,
    pub model: String,
    pub input_token_price: f64,
    pub output_token_price: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetPricingRequest {
    pub provider: String,
    pub model: String,
    pub input_token_price: f64,
    pub output_token_price: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetBudgetRequest {
    pub max_budget: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordRunTokensRequest {
    pub task_id: String,
    pub project_id: String,
    pub agent_id: String,
    pub account_id: String,
    pub provider: String,
    pub model: String,
    pub has_token_data: bool,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct RunCostView {
    pub run_id: String,
    pub agent_id: String,
    pub account_id: String,
    pub provider: String,
    pub model: String,
    pub has_token_data: bool,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub calculated_cost: Option<f64>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct TaskCostSummary {
    pub task_id: String,
    pub has_token_data: bool,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_tokens: i64,
    pub total_cost: Option<f64>,
    pub missing_runs_count: i64,
    pub budget: Option<f64>,
    pub is_over_budget: bool,
    pub runs: Vec<RunCostView>,
}

// ==================== HÀM TIỆN ÍCH PHÂN QUYỀN ====================

fn err_resp(status: StatusCode, msg: &str) -> Response {
    (status, Json(json!({ "error": msg }))).into_response()
}

async fn check_project_access(
    pool: &SqlitePool,
    headers: &HeaderMap,
    prj_id: &str,
) -> Result<String, StatusCode> {
    let Some(user) = session_user(pool, headers).await else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if user.is_system_admin() {
        return Ok(user.user_id);
    }

    let prj_row = sqlx::query("SELECT workspace_id FROM project WHERE id = ?")
        .bind(prj_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let ws_id: String = prj_row.get("workspace_id");

    let member_row =
        sqlx::query("SELECT role FROM workspace_member WHERE workspace_id = ? AND user_id = ?")
            .bind(&ws_id)
            .bind(&user.user_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if member_row.is_some() {
        Ok(user.user_id)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

async fn check_task_access(
    pool: &SqlitePool,
    headers: &HeaderMap,
    task_id: &str,
) -> Result<(String, String), StatusCode> {
    let task_row = sqlx::query("SELECT project_id FROM task WHERE id = ?")
        .bind(task_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let prj_id: String = task_row.get("project_id");

    let user_id = check_project_access(pool, headers, &prj_id).await?;
    Ok((user_id, prj_id))
}

// ==================== XỬ LÝ ĐIỂM CUỐI (HANDLERS) ====================

async fn get_project_pricing(
    State(state): State<CostTrackingState>,
    AxumPath(prj_id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let Some(ref pool) = state.pool else {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing");
    };
    if let Err(st) = check_project_access(pool, &headers, &prj_id).await {
        return err_resp(st, "Không có quyền truy cập dự án");
    }

    let rows = match sqlx::query(
        "SELECT id, project_id, provider, model, input_token_price, output_token_price, created_at, updated_at \
         FROM m_cost_tracking_pricing WHERE project_id = ? ORDER BY provider ASC, model ASC"
    )
    .bind(&prj_id)
    .fetch_all(pool)
    .await {
        Ok(r) => r,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let rules: Vec<PricingRule> = rows
        .into_iter()
        .map(|r| PricingRule {
            id: r.get("id"),
            project_id: r.get("project_id"),
            provider: r.get("provider"),
            model: r.get("model"),
            input_token_price: r.get("input_token_price"),
            output_token_price: r.get("output_token_price"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
        .collect();

    Json(json!({ "pricing": rules })).into_response()
}

async fn set_project_pricing(
    State(state): State<CostTrackingState>,
    AxumPath(prj_id): AxumPath<String>,
    headers: HeaderMap,
    Json(req): Json<SetPricingRequest>,
) -> Response {
    let Some(ref pool) = state.pool else {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing");
    };
    if let Err(st) = check_project_access(pool, &headers, &prj_id).await {
        return err_resp(st, "Không có quyền sửa cấu hình dự án");
    }

    if req.input_token_price < 0.0 || req.output_token_price < 0.0 {
        return err_resp(StatusCode::BAD_REQUEST, "Giá token không được âm");
    }

    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    let res = sqlx::query(
        "INSERT INTO m_cost_tracking_pricing \
         (id, project_id, provider, model, input_token_price, output_token_price, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(project_id, provider, model) DO UPDATE SET \
            input_token_price = excluded.input_token_price, \
            output_token_price = excluded.output_token_price, \
            updated_at = excluded.updated_at"
    )
    .bind(&id)
    .bind(&prj_id)
    .bind(&req.provider)
    .bind(&req.model)
    .bind(req.input_token_price)
    .bind(req.output_token_price)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await;

    match res {
        Ok(_) => {
            let _ = sqlx::query(
                "UPDATE m_cost_tracking_run_usage \
                 SET calculated_cost = ((COALESCE(input_tokens, 0) * ? + COALESCE(output_tokens, 0) * ?) / 1000000.0) \
                 WHERE project_id = ? AND provider = ? AND model = ? AND has_token_data = 1"
            )
            .bind(req.input_token_price)
            .bind(req.output_token_price)
            .bind(&prj_id)
            .bind(&req.provider)
            .bind(&req.model)
            .execute(pool)
            .await;

            Json(json!({ "success": true, "project_id": prj_id })).into_response()
        }
        Err(e) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn delete_project_pricing(
    State(state): State<CostTrackingState>,
    AxumPath((prj_id, pricing_id)): AxumPath<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let Some(ref pool) = state.pool else {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing");
    };
    if let Err(st) = check_project_access(pool, &headers, &prj_id).await {
        return err_resp(st, "Không có quyền sửa cấu hình dự án");
    }

    let _ = sqlx::query("DELETE FROM m_cost_tracking_pricing WHERE id = ? AND project_id = ?")
        .bind(&pricing_id)
        .bind(&prj_id)
        .execute(pool)
        .await;

    Json(json!({ "deleted": true })).into_response()
}

async fn set_task_budget(
    State(state): State<CostTrackingState>,
    AxumPath(task_id): AxumPath<String>,
    headers: HeaderMap,
    Json(req): Json<SetBudgetRequest>,
) -> Response {
    let Some(ref pool) = state.pool else {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing");
    };
    if let Err(st) = check_task_access(pool, &headers, &task_id).await {
        return err_resp(st, "Không có quyền sửa ngân sách nhiệm vụ");
    }

    if req.max_budget <= 0.0 {
        return err_resp(StatusCode::BAD_REQUEST, "Ngân sách tối đa phải lớn hơn 0");
    }

    let now = Utc::now().to_rfc3339();
    let res = sqlx::query(
        "INSERT INTO m_cost_tracking_task_budget (task_id, max_budget, alerted_over_budget, created_at, updated_at) \
         VALUES (?, ?, 0, ?, ?) \
         ON CONFLICT(task_id) DO UPDATE SET \
            max_budget = excluded.max_budget, \
            alerted_over_budget = 0, \
            updated_at = excluded.updated_at"
    )
    .bind(&task_id)
    .bind(req.max_budget)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await;

    match res {
        Ok(_) => Json(json!({ "success": true, "task_id": task_id, "max_budget": req.max_budget }))
            .into_response(),
        Err(e) => err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "Xử lý ghi nhận token, tra cứu giá và kiểm tra cảnh báo ngân sách"
)]
async fn record_run_tokens(
    State(state): State<CostTrackingState>,
    AxumPath(run_id): AxumPath<String>,
    headers: HeaderMap,
    Json(req): Json<RecordRunTokensRequest>,
) -> Response {
    let Some(ref pool) = state.pool else {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing");
    };
    if let Err(st) = check_project_access(pool, &headers, &req.project_id).await {
        return err_resp(st, "Không có quyền ghi nhận dữ liệu lần chạy");
    }

    let in_tok = req.input_tokens.unwrap_or(0);
    let out_tok = req.output_tokens.unwrap_or(0);
    let total_tok = if req.has_token_data {
        Some(in_tok + out_tok)
    } else {
        None
    };

    // Tra cứu bảng giá
    let pricing_row = sqlx::query(
        "SELECT input_token_price, output_token_price FROM m_cost_tracking_pricing \
         WHERE project_id = ? AND provider = ? AND model = ?",
    )
    .bind(&req.project_id)
    .bind(&req.provider)
    .bind(&req.model)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let calculated_cost = if req.has_token_data {
        pricing_row.map(|r| {
            let in_p: f64 = r.get("input_token_price");
            let out_p: f64 = r.get("output_token_price");
            (in_tok as f64 * in_p + out_tok as f64 * out_p) / 1_000_000.0
        })
    } else {
        None
    };

    let now = Utc::now().to_rfc3339();
    let res = sqlx::query(
        "INSERT INTO m_cost_tracking_run_usage \
         (run_id, task_id, project_id, agent_id, account_id, provider, model, has_token_data, input_tokens, output_tokens, total_tokens, calculated_cost, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(run_id) DO UPDATE SET \
            has_token_data = excluded.has_token_data, \
            input_tokens = excluded.input_tokens, \
            output_tokens = excluded.output_tokens, \
            total_tokens = excluded.total_tokens, \
            calculated_cost = excluded.calculated_cost"
    )
    .bind(&run_id)
    .bind(&req.task_id)
    .bind(&req.project_id)
    .bind(&req.agent_id)
    .bind(&req.account_id)
    .bind(&req.provider)
    .bind(&req.model)
    .bind(if req.has_token_data { 1 } else { 0 })
    .bind(if req.has_token_data { Some(in_tok) } else { None })
    .bind(if req.has_token_data { Some(out_tok) } else { None })
    .bind(total_tok)
    .bind(calculated_cost)
    .bind(&now)
    .execute(pool)
    .await;

    if let Err(e) = res {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }

    // Kiểm tra ngân sách task
    check_task_budget_crossing(pool, &state.event_bus, &req.task_id, &req.project_id).await;

    Json(json!({
        "success": true,
        "run_id": run_id,
        "has_token_data": req.has_token_data,
        "calculated_cost": calculated_cost
    }))
    .into_response()
}

#[expect(
    clippy::too_many_lines,
    reason = "Kiểm tra ngân sách nhiệm vụ và phát thông báo khi vượt"
)]
async fn check_task_budget_crossing(
    pool: &SqlitePool,
    event_bus: &EventBus,
    task_id: &str,
    project_id: &str,
) {
    let budget_row = sqlx::query(
        "SELECT max_budget, alerted_over_budget FROM m_cost_tracking_task_budget WHERE task_id = ?",
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some(b) = budget_row else {
        return;
    };
    let max_budget: f64 = b.get("max_budget");
    let alerted: i64 = b.get("alerted_over_budget");

    let sum_row = sqlx::query(
        "SELECT sum((COALESCE(u.input_tokens, 0) * p.input_token_price + COALESCE(u.output_tokens, 0) * p.output_token_price) / 1000000.0) as total_cost \
         FROM m_cost_tracking_run_usage u \
         JOIN m_cost_tracking_pricing p ON p.project_id = u.project_id AND p.provider = u.provider AND p.model = u.model \
         WHERE u.task_id = ? AND u.has_token_data = 1"
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let total_cost: f64 = sum_row
        .and_then(|r| r.try_get::<f64, _>("total_cost").ok())
        .unwrap_or(0.0);

    if total_cost >= max_budget && alerted == 0 {
        let _ = sqlx::query(
            "UPDATE m_cost_tracking_task_budget SET alerted_over_budget = 1 WHERE task_id = ?",
        )
        .bind(task_id)
        .execute(pool)
        .await;

        let ws_row = sqlx::query("SELECT workspace_id FROM project WHERE id = ?")
            .bind(project_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
        let ws_id = ws_row
            .and_then(|r| r.try_get::<String, _>("workspace_id").ok())
            .and_then(|s| s.parse::<WorkspaceId>().ok())
            .unwrap_or_else(WorkspaceId::new);

        let p_id = project_id
            .parse::<ProjectId>()
            .unwrap_or_else(|_| ProjectId::new());
        let t_id = task_id.parse::<TaskId>().unwrap_or_else(|_| TaskId::new());

        let envelope = Envelope::new(
            Utc::now(),
            EventScope::task(ws_id, p_id, t_id),
            EventPayload::NotificationRequested {
                title: "Vượt ngân sách nhiệm vụ".to_string(),
                message: format!(
                    "Nhiệm vụ {task_id} đã vượt ngân sách chi phí ({total_cost:.2}$ >= {max_budget:.2}$)"
                ),
                level: "warning".to_string(),
            },
        );
        event_bus.publish(envelope);
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "Tổng hợp chi phí và token cho nhiệm vụ kèm phân loại có/thiếu số liệu"
)]
async fn get_task_cost(
    State(state): State<CostTrackingState>,
    AxumPath(task_id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let Some(ref pool) = state.pool else {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing");
    };
    if let Err(st) = check_task_access(pool, &headers, &task_id).await {
        return err_resp(st, "Không có quyền xem chi phí nhiệm vụ");
    }

    let rows = match sqlx::query(
        "SELECT u.run_id, u.agent_id, u.account_id, u.provider, u.model, u.has_token_data, \
         u.input_tokens, u.output_tokens, u.total_tokens, u.created_at, \
         p.input_token_price, p.output_token_price \
         FROM m_cost_tracking_run_usage u \
         LEFT JOIN m_cost_tracking_pricing p \
            ON p.project_id = u.project_id AND p.provider = u.provider AND p.model = u.model \
         WHERE u.task_id = ? ORDER BY u.created_at ASC",
    )
    .bind(&task_id)
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let mut total_in = 0i64;
    let mut total_out = 0i64;
    let mut total_tok = 0i64;
    let mut missing_count = 0i64;
    let mut has_any_data = false;
    let mut total_cost: Option<f64> = Some(0.0);

    let runs: Vec<RunCostView> = rows
        .into_iter()
        .map(|r| {
            let has_data = r.get::<i64, _>("has_token_data") == 1;
            let in_t: Option<i64> = r.get("input_tokens");
            let out_t: Option<i64> = r.get("output_tokens");
            let tot_t: Option<i64> = r.get("total_tokens");
            let p_in = r
                .try_get::<Option<f64>, _>("input_token_price")
                .ok()
                .flatten();
            let p_out = r
                .try_get::<Option<f64>, _>("output_token_price")
                .ok()
                .flatten();

            let cost = if has_data {
                match (p_in, p_out) {
                    (Some(ip), Some(op)) => {
                        let it = in_t.unwrap_or(0) as f64;
                        let ot = out_t.unwrap_or(0) as f64;
                        Some((it * ip + ot * op) / 1_000_000.0)
                    }
                    _ => None,
                }
            } else {
                None
            };

            if has_data {
                has_any_data = true;
                if let Some(it) = in_t {
                    total_in += it;
                }
                if let Some(ot) = out_t {
                    total_out += ot;
                }
                if let Some(tt) = tot_t {
                    total_tok += tt;
                }

                if let Some(c) = cost {
                    if let Some(tc) = total_cost.as_mut() {
                        *tc += c;
                    }
                } else {
                    // Có lượt có token nhưng thiếu bảng giá -> tổng tiền không thể xác định
                    total_cost = None;
                }
            } else {
                missing_count += 1;
            }

            RunCostView {
                run_id: r.get("run_id"),
                agent_id: r.get("agent_id"),
                account_id: r.get("account_id"),
                provider: r.get("provider"),
                model: r.get("model"),
                has_token_data: has_data,
                input_tokens: in_t,
                output_tokens: out_t,
                total_tokens: tot_t,
                calculated_cost: cost,
                created_at: r.get("created_at"),
            }
        })
        .collect();

    if !has_any_data {
        total_cost = None;
    }

    let budget_row =
        sqlx::query("SELECT max_budget FROM m_cost_tracking_task_budget WHERE task_id = ?")
            .bind(&task_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    let budget: Option<f64> = budget_row.and_then(|r| r.try_get("max_budget").ok());
    let is_over = match (total_cost, budget) {
        (Some(c), Some(b)) => c >= b,
        _ => false,
    };

    let summary = TaskCostSummary {
        task_id,
        has_token_data: has_any_data,
        total_input_tokens: total_in,
        total_output_tokens: total_out,
        total_tokens: total_tok,
        total_cost,
        missing_runs_count: missing_count,
        budget,
        is_over_budget: is_over,
        runs,
    };

    Json(summary).into_response()
}

async fn get_project_summary(
    State(state): State<CostTrackingState>,
    AxumPath(prj_id): AxumPath<String>,
    headers: HeaderMap,
) -> Response {
    let Some(ref pool) = state.pool else {
        return err_resp(StatusCode::INTERNAL_SERVER_ERROR, "Database pool missing");
    };
    if let Err(st) = check_project_access(pool, &headers, &prj_id).await {
        return err_resp(st, "Không có quyền xem tổng hợp dự án");
    }

    let rows = match sqlx::query(
        "SELECT agent_id, sum(total_tokens) as tokens, sum(calculated_cost) as cost, \
         count(*) as total_runs, sum(case when has_token_data = 0 then 1 else 0 end) as missing_runs \
         FROM m_cost_tracking_run_usage WHERE project_id = ? GROUP BY agent_id"
    )
    .bind(&prj_id)
    .fetch_all(pool)
    .await {
        Ok(r) => r,
        Err(e) => return err_resp(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let mut project_cost = 0.0;
    let mut project_tokens = 0i64;
    let mut total_missing = 0i64;
    let mut agent_breakdown = Vec::new();

    for r in rows {
        let agent: String = r.get("agent_id");
        let toks: Option<i64> = r.get("tokens");
        let cost: Option<f64> = r.get("cost");
        let missing: i64 = r.get("missing_runs");

        let t = toks.unwrap_or(0);
        let c = cost.unwrap_or(0.0);

        project_tokens += t;
        project_cost += c;
        total_missing += missing;

        agent_breakdown.push(json!({
            "agent_id": agent,
            "tokens": t,
            "cost": c,
            "missing_runs": missing
        }));
    }

    Json(json!({
        "project_id": prj_id,
        "total_cost": project_cost,
        "total_tokens": project_tokens,
        "missing_runs_count": total_missing,
        "by_agent": agent_breakdown
    }))
    .into_response()
}

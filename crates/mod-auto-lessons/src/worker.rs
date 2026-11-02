//! Bộ xử lý sự kiện chạy nền cho module `auto-lessons` (P3-11).

use chrono::Utc;
use dandori_module_api::{AgentRole, EventBus, EventFilter, TaskRunner};
use dandori_types::{Envelope, EventPayload, TaskId, UserId};
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::model::AutoLessonsConfig;
use crate::redaction::redact_secrets;

async fn get_project_config(pool: &SqlitePool, ws_id: &str, prj_id: &str) -> AutoLessonsConfig {
    let row: Option<AutoLessonsConfig> = sqlx::query_as(
        "SELECT project_id, workspace_id, enabled, trigger_on_done, trigger_on_failed, \
         max_token_budget, min_content_length, created_at, updated_at \
         FROM m_auto_lessons_config WHERE project_id = ?",
    )
    .bind(prj_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    row.unwrap_or_else(|| AutoLessonsConfig::default_for(ws_id, prj_id))
}

async fn get_workspace_user_id(pool: &SqlitePool, ws_str: &str) -> UserId {
    let user_row = sqlx::query(
        "SELECT user_id FROM workspace_member WHERE workspace_id = ? ORDER BY role LIMIT 1",
    )
    .bind(ws_str)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    user_row
        .and_then(|r| r.try_get::<String, _>("user_id").ok())
        .and_then(|s| s.parse::<UserId>().ok())
        .unwrap_or_else(UserId::new)
}

/// Kiểm tra xem task đã từng chạy trích xuất bài học chưa (chống lặp).
pub async fn is_task_already_processed(pool: &SqlitePool, task_id: &str) -> bool {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM m_auto_lessons_run WHERE task_id = ?")
        .bind(task_id)
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    count.0 > 0
}

/// Xử lý sự kiện chuyển trạng thái task sang `done` hoặc `failed`.
#[expect(
    clippy::too_many_lines,
    reason = "Quy trình trích xuất bài học và kiểm tra an toàn theo chuỗi"
)]
pub async fn handle_task_status_changed(
    pool: &SqlitePool,
    runner: Option<&Arc<dyn TaskRunner>>,
    env: &Envelope,
    task_id: &TaskId,
    new_status: &str,
) {
    if new_status != "done" && new_status != "failed" {
        return;
    }

    let Some(ws_id) = env.scope.workspace_id else {
        return;
    };
    let Some(prj_id) = env.scope.project_id else {
        return;
    };

    let ws_str = ws_id.to_string();
    let prj_str = prj_id.to_string();
    let task_str = task_id.to_string();

    let config = get_project_config(pool, &ws_str, &prj_str).await;
    if !config.enabled {
        return;
    }

    if (new_status == "done" && !config.trigger_on_done)
        || (new_status == "failed" && !config.trigger_on_failed)
    {
        return;
    }

    // Chống lặp (Anti-loop): Kiểm tra nếu task đã được ghi nhận trước đó
    if is_task_already_processed(pool, &task_str).await {
        return;
    }

    // Lấy thông tin chi tiết của task để kiểm tra độ dài và chuẩn bị dữ liệu
    let task_row = sqlx::query("SELECT title, description FROM task WHERE id = ?")
        .bind(&task_str)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

    let (title, description) = match task_row {
        Some(r) => (
            r.try_get::<String, _>("title").unwrap_or_default(),
            r.try_get::<String, _>("description").unwrap_or_default(),
        ),
        None => (String::new(), String::new()),
    };

    // Chống lặp cho chính task bài học
    if title.contains("[auto-lessons]") {
        return;
    }

    let combined_len = title.trim().len() + description.trim().len();
    let rec_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    // Bỏ qua task rất ngắn hoặc không có thông tin
    if (combined_len as i64) < config.min_content_length {
        let _ = sqlx::query(
            "INSERT INTO m_auto_lessons_run (id, workspace_id, project_id, task_id, status, reason, created_at, finished_at) \
             VALUES (?, ?, ?, ?, 'skipped', 'Task quá ngắn hoặc thiếu thông tin', ?, ?)",
        )
        .bind(&rec_id)
        .bind(&ws_str)
        .bind(&prj_str)
        .bind(&task_str)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await;
        return;
    }

    // Ghi nhận bản ghi đang chạy
    let _ = sqlx::query(
        "INSERT INTO m_auto_lessons_run (id, workspace_id, project_id, task_id, status, created_at) \
         VALUES (?, ?, ?, ?, 'running', ?)",
    )
    .bind(&rec_id)
    .bind(&ws_str)
    .bind(&prj_str)
    .bind(&task_str)
    .bind(&now)
    .execute(pool)
    .await;

    // Lọc bí mật từ thông tin task trước khi chuyển cho agent / tạo bài học
    let clean_title = redact_secrets(&title);
    let clean_desc = redact_secrets(&description);

    let mut run_id_opt = None;
    if let Some(r) = runner {
        let user_id = get_workspace_user_id(pool, &ws_str).await;
        if let Ok(run) = r
            .start_run(*task_id, AgentRole::Lead, user_id, vec![])
            .await
        {
            run_id_opt = Some(run.id.to_string());
        }
    }

    let run_id_str = run_id_opt.unwrap_or_else(|| Uuid::new_v4().to_string());

    // Nội dung bài học trích xuất (được lọc sạch bí mật)
    let raw_lesson = format!(
        "Bài học từ task '{}' (kết quả {}): {}. Lưu ý quy trình và kiểm tra kỹ lưỡng.",
        clean_title,
        new_status,
        if clean_desc.is_empty() {
            "Hoàn thành quy trình task"
        } else {
            &clean_desc
        }
    );
    let clean_lesson = redact_secrets(&raw_lesson);

    // Lưu vào shared-memory dưới dạng lesson ở trạng thái 'pending' (Tuyệt đối KHÔNG tự duyệt)
    let lesson_id = Uuid::new_v4().to_string();
    let source_ref = format!("agent:pm#run:{}#task:{}", run_id_str, task_str);

    let insert_memory_res = sqlx::query(
        "INSERT INTO m_shared_memory_entry ( \
            id, workspace_id, project_id, category, content, source_type, source_ref, \
            status, reviewed_by, reviewed_at, created_at, updated_at \
         ) VALUES (?, ?, ?, 'lesson', ?, 'agent', ?, 'pending', NULL, NULL, ?, ?)",
    )
    .bind(&lesson_id)
    .bind(&ws_str)
    .bind(&prj_str)
    .bind(&clean_lesson)
    .bind(&source_ref)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await;

    let finish_now = Utc::now().to_rfc3339();
    if insert_memory_res.is_ok() {
        let _ = sqlx::query(
            "UPDATE m_auto_lessons_run \
             SET status = 'completed', run_id = ?, lesson_id = ?, finished_at = ? \
             WHERE id = ?",
        )
        .bind(&run_id_str)
        .bind(&lesson_id)
        .bind(&finish_now)
        .bind(&rec_id)
        .execute(pool)
        .await;
    } else {
        let _ = sqlx::query(
            "UPDATE m_auto_lessons_run \
             SET status = 'failed', run_id = ?, reason = 'Lỗi lưu vào shared-memory', finished_at = ? \
             WHERE id = ?",
        )
        .bind(&run_id_str)
        .bind(&finish_now)
        .bind(&rec_id)
        .execute(pool)
        .await;
    }
}

/// Chạy vòng lặp worker lắng nghe sự kiện trên EventBus cho module `auto-lessons`.
pub async fn run_auto_lessons_worker(
    pool: SqlitePool,
    event_bus: Arc<EventBus>,
    runner: Option<Arc<dyn TaskRunner>>,
    cancel_token: CancellationToken,
) {
    let mut rx = event_bus.subscribe(EventFilter::all());
    loop {
        tokio::select! {
            () = cancel_token.cancelled() => break,
            res = rx.recv() => {
                let Some(env) = res else { break };
                if let EventPayload::TaskStatusChanged { ref task_id, ref new_status, .. } = env.payload {
                    handle_task_status_changed(
                        &pool,
                        runner.as_ref(),
                        &env,
                        task_id,
                        new_status,
                    ).await;
                }
            }
        }
    }
}

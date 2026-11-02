//! Bộ xử lý sự kiện chạy nền cho module `test-agent` (P2-04).

use std::path::Path;
use std::sync::Arc;

use chrono::Utc;
use dandori_module_api::{AgentRole, EventBus, EventFilter, GitClient, TaskRunner};
use dandori_types::{
    Envelope, EventPayload, EventScope, ProjectId, RunId, TaskId, TaskStatus, UserId, WorkspaceId,
};
use sqlx::{Row, SqlitePool};
use tokio_util::sync::CancellationToken;

use crate::model::TestAgentConfig;
use crate::pattern::is_file_allowed;

async fn get_project_config(pool: &SqlitePool, ws_id: &str, prj_id: &str) -> TestAgentConfig {
    let row: Option<TestAgentConfig> = sqlx::query_as(
        "SELECT project_id, workspace_id, enabled, suggested_command, allowed_path_patterns, \
         max_test_runs_per_worker, timeout_seconds, created_at, updated_at \
         FROM m_test_agent_config WHERE project_id = ?",
    )
    .bind(prj_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    row.unwrap_or_else(|| TestAgentConfig::default_for(ws_id, prj_id))
}

async fn get_task_paths(
    pool: &SqlitePool,
    prj_id: &str,
    task_id: &str,
) -> Option<(String, String)> {
    let repo_row = sqlx::query("SELECT repo_path FROM project WHERE id = ?")
        .bind(prj_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()?;

    let task_row = sqlx::query("SELECT workdir_path FROM task WHERE id = ?")
        .bind(task_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()?;

    let repo_path: String = repo_row.try_get("repo_path").unwrap_or_default();
    let workdir_path: Option<String> = task_row.try_get("workdir_path").ok();
    workdir_path.map(|w| (repo_path, w))
}

async fn check_worktree_flagged_files(
    pool: &SqlitePool,
    prj_id: &str,
    task_id: &str,
    allowed_patterns: &[String],
) -> Vec<String> {
    let Some((repo_path, wt_str)) = get_task_paths(pool, prj_id, task_id).await else {
        return vec![];
    };

    let client = GitClient::new(&repo_path);
    let changed = client
        .status_porcelain(Path::new(&wt_str))
        .await
        .unwrap_or_default();

    changed
        .into_iter()
        .map(|line| {
            if line.len() > 3 {
                line[3..].trim().to_string()
            } else {
                line
            }
        })
        .filter(|file| !is_file_allowed(file, allowed_patterns))
        .collect()
}

async fn update_test_run_record(
    pool: &SqlitePool,
    rec_id: &str,
    status: &str,
    exit_code: Option<i64>,
    summary: &str,
    flagged_json: &str,
    now_str: &str,
) {
    let _ = sqlx::query(
        "UPDATE m_test_agent_run \
         SET status = ?, exit_code = ?, summary = ?, flagged_files = ?, finished_at = ? \
         WHERE id = ?",
    )
    .bind(status)
    .bind(exit_code)
    .bind(summary)
    .bind(flagged_json)
    .bind(now_str)
    .bind(rec_id)
    .execute(pool)
    .await;
}

async fn mark_task_failed(
    pool: &SqlitePool,
    event_bus: &Arc<EventBus>,
    ws_id: WorkspaceId,
    prj_id: ProjectId,
    t_id: TaskId,
    now_str: &str,
) {
    let t_id_str = t_id.to_string();
    let _ = sqlx::query("UPDATE task SET status = 'needs_human', updated_at = ? WHERE id = ?")
        .bind(now_str)
        .bind(&t_id_str)
        .execute(pool)
        .await;

    event_bus.publish(Envelope::new(
        Utc::now(),
        EventScope::task(ws_id, prj_id, t_id),
        EventPayload::TaskStatusChanged {
            task_id: t_id,
            old_status: TaskStatus::InReview.as_str().to_string(),
            new_status: TaskStatus::NeedsHuman.as_str().to_string(),
        },
    ));
}

async fn handle_test_run_finished(
    pool: &SqlitePool,
    event_bus: &Arc<EventBus>,
    run_id: &RunId,
    exit_code: Option<i64>,
    ws_id: WorkspaceId,
    prj_id: ProjectId,
    t_id: TaskId,
) -> bool {
    let test_run_row = sqlx::query(
        "SELECT id, project_id, task_id, trigger_run_id FROM m_test_agent_run WHERE run_id = ?",
    )
    .bind(run_id.to_string())
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some(row) = test_run_row else {
        return false;
    };

    let rec_id: String = row.try_get("id").unwrap_or_default();
    let p_id_str = prj_id.to_string();
    let t_id_str = t_id.to_string();
    let config = get_project_config(pool, &ws_id.to_string(), &p_id_str).await;

    let flagged =
        check_worktree_flagged_files(pool, &p_id_str, &t_id_str, &config.parsed_patterns()).await;
    let flagged_json = serde_json::to_string(&flagged).unwrap_or_else(|_| "[]".to_string());

    let (status, summary) = if exit_code == Some(0) {
        ("passed", "Kiểm thử tự động thành công (mã thoát 0)")
    } else {
        ("failed", "Kiểm thử tự động thất bại")
    };

    let now_str = Utc::now().to_rfc3339();
    update_test_run_record(
        pool,
        &rec_id,
        status,
        exit_code,
        summary,
        &flagged_json,
        &now_str,
    )
    .await;

    if status == "failed" {
        mark_task_failed(pool, event_bus, ws_id, prj_id, t_id, &now_str).await;
    }

    true
}

async fn can_trigger_test_run(
    pool: &SqlitePool,
    ws_str: &str,
    p_str: &str,
    trigger_run_id_str: &str,
) -> bool {
    let config = get_project_config(pool, ws_str, p_str).await;
    if !config.enabled {
        return false;
    }

    let count: (i64,) =
        sqlx::query_as("SELECT count(*) FROM m_test_agent_run WHERE trigger_run_id = ?")
            .bind(trigger_run_id_str)
            .fetch_one(pool)
            .await
            .unwrap_or((0,));

    count.0 < config.max_test_runs_per_worker
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

async fn dispatch_test_run(
    pool: &SqlitePool,
    runner: &Arc<dyn TaskRunner>,
    ws_str: &str,
    p_str: &str,
    t_id: TaskId,
    trigger_run_id_str: &str,
) {
    let user_id = get_workspace_user_id(pool, ws_str).await;

    match runner
        .start_run(t_id, AgentRole::Custom("test".to_string()), user_id, vec![])
        .await
    {
        Ok(test_run) => {
            let rec_id = uuid::Uuid::new_v4().to_string();
            let now_str = Utc::now().to_rfc3339();
            let _ = sqlx::query(
                "INSERT INTO m_test_agent_run (id, task_id, project_id, workspace_id, run_id, trigger_run_id, status, created_at) \
                 VALUES (?, ?, ?, ?, ?, ?, 'running', ?)",
            )
            .bind(&rec_id)
            .bind(t_id.to_string())
            .bind(p_str)
            .bind(ws_str)
            .bind(test_run.id.to_string())
            .bind(trigger_run_id_str)
            .bind(&now_str)
            .execute(pool)
            .await;
        }
        Err(e) => {
            tracing::warn!("Không thể khởi chạy test agent: {e}");
        }
    }
}

async fn handle_worker_run_finished(
    pool: &SqlitePool,
    runner: &Arc<dyn TaskRunner>,
    run_id: &RunId,
    exit_code: Option<i64>,
    ws_id: WorkspaceId,
    prj_id: ProjectId,
    t_id: TaskId,
) {
    if exit_code != Some(0) {
        return;
    }

    let ws_str = ws_id.to_string();
    let p_str = prj_id.to_string();
    let trigger_run_id_str = run_id.to_string();

    if can_trigger_test_run(pool, &ws_str, &p_str, &trigger_run_id_str).await {
        dispatch_test_run(pool, runner, &ws_str, &p_str, t_id, &trigger_run_id_str).await;
    }
}

/// Chạy vòng lặp lắng nghe sự kiện của Test Agent (P2-04).
pub async fn run_test_agent_worker(
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
                if let EventPayload::RunFinished { run_id, exit_code } = env.payload {
                    let Some(ws_id) = env.scope.workspace_id else { continue };
                    let Some(prj_id) = env.scope.project_id else { continue };
                    let Some(t_id) = env.scope.task_id else { continue };

                    let is_test_run = handle_test_run_finished(
                        &pool,
                        &event_bus,
                        &run_id,
                        exit_code,
                        ws_id,
                        prj_id,
                        t_id,
                    ).await;

                    if !is_test_run && let Some(ref r) = runner {
                        handle_worker_run_finished(
                            &pool,
                            r,
                            &run_id,
                            exit_code,
                            ws_id,
                            prj_id,
                            t_id,
                        ).await;
                    }
                }
            }
        }
    }
}

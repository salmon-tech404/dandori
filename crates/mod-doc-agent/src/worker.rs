//! Bộ xử lý sự kiện chạy nền cho module `doc-agent` (P2-05).

use std::path::Path;
use std::sync::Arc;

use chrono::Utc;
use dandori_module_api::{AgentRole, EventBus, EventFilter, GitClient, TaskRunner};
use dandori_types::{Envelope, EventPayload, ProjectId, RunId, TaskId, UserId, WorkspaceId};
use sqlx::{Row, SqlitePool};
use tokio_util::sync::CancellationToken;

use crate::model::DocAgentConfig;
use crate::pattern::is_file_allowed;

async fn get_project_config(pool: &SqlitePool, ws_id: &str, prj_id: &str) -> DocAgentConfig {
    let row: Option<DocAgentConfig> = sqlx::query_as(
        "SELECT project_id, workspace_id, enabled, allowed_path_patterns, prompt_template, \
         timeout_seconds, created_at, updated_at \
         FROM m_doc_agent_config WHERE project_id = ?",
    )
    .bind(prj_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    row.unwrap_or_else(|| DocAgentConfig::default_for(ws_id, prj_id))
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

async fn scan_worktree_files(
    pool: &SqlitePool,
    prj_id: &str,
    task_id: &str,
    allowed_patterns: &[String],
) -> (Vec<String>, Vec<String>) {
    let Some((repo_path, wt_str)) = get_task_paths(pool, prj_id, task_id).await else {
        return (vec![], vec![]);
    };

    let client = GitClient::new(&repo_path);
    let changed = client
        .status_porcelain(Path::new(&wt_str))
        .await
        .unwrap_or_default();

    let mut doc_files = Vec::new();
    let mut flagged_files = Vec::new();

    for line in changed {
        let file = if line.len() > 3 {
            line[3..].trim().to_string()
        } else {
            line
        };
        if is_file_allowed(&file, allowed_patterns) {
            doc_files.push(file);
        } else {
            flagged_files.push(file);
        }
    }

    (doc_files, flagged_files)
}

async fn update_doc_run_record(
    pool: &SqlitePool,
    rec_id: &str,
    status: &str,
    summary: &str,
    doc_json: &str,
    flagged_json: &str,
) {
    let now_str = Utc::now().to_rfc3339();
    let _ = sqlx::query(
        "UPDATE m_doc_agent_run \
         SET status = ?, summary = ?, changed_doc_files = ?, flagged_files = ?, finished_at = ? \
         WHERE id = ?",
    )
    .bind(status)
    .bind(summary)
    .bind(doc_json)
    .bind(flagged_json)
    .bind(&now_str)
    .bind(rec_id)
    .execute(pool)
    .await;
}

async fn handle_doc_run_finished(
    pool: &SqlitePool,
    run_id: &RunId,
    exit_code: Option<i64>,
    ws_id: WorkspaceId,
    prj_id: ProjectId,
    t_id: TaskId,
) -> bool {
    let doc_run_row =
        sqlx::query("SELECT id, project_id, task_id FROM m_doc_agent_run WHERE run_id = ?")
            .bind(run_id.to_string())
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    let Some(row) = doc_run_row else {
        return false;
    };

    let rec_id: String = row.try_get("id").unwrap_or_default();
    let p_id_str = prj_id.to_string();
    let t_id_str = t_id.to_string();
    let config = get_project_config(pool, &ws_id.to_string(), &p_id_str).await;

    let (doc_files, flagged) =
        scan_worktree_files(pool, &p_id_str, &t_id_str, &config.parsed_patterns()).await;
    let doc_json = serde_json::to_string(&doc_files).unwrap_or_else(|_| "[]".to_string());
    let flagged_json = serde_json::to_string(&flagged).unwrap_or_else(|_| "[]".to_string());

    let status = if exit_code == Some(0) {
        "completed"
    } else {
        "failed"
    };
    let summary = format!(
        "Tài liệu đã cập nhật: {} tệp; Tệp ngoài phạm vi: {} tệp",
        doc_files.len(),
        flagged.len()
    );

    update_doc_run_record(pool, &rec_id, status, &summary, &doc_json, &flagged_json).await;
    true
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

async fn can_trigger_doc_agent(pool: &SqlitePool, ws_str: &str, p_str: &str, t_str: &str) -> bool {
    let config = get_project_config(pool, ws_str, p_str).await;
    if !config.enabled {
        return false;
    }

    let count: (i64,) = sqlx::query_as(
        "SELECT count(*) FROM m_doc_agent_run WHERE task_id = ? AND status IN ('running', 'completed')",
    )
    .bind(t_str)
    .fetch_one(pool)
    .await
    .unwrap_or((0,));

    count.0 == 0
}

async fn dispatch_doc_run(
    pool: &SqlitePool,
    runner: &Arc<dyn TaskRunner>,
    ws_str: &str,
    p_str: &str,
    t_id: TaskId,
) {
    let user_id = get_workspace_user_id(pool, ws_str).await;

    match runner
        .start_run(t_id, AgentRole::Custom("doc".to_string()), user_id, vec![])
        .await
    {
        Ok(doc_run) => {
            let rec_id = uuid::Uuid::new_v4().to_string();
            let now_str = Utc::now().to_rfc3339();
            let _ = sqlx::query(
                "INSERT INTO m_doc_agent_run (id, task_id, project_id, workspace_id, run_id, status, created_at) \
                 VALUES (?, ?, ?, ?, ?, 'running', ?)",
            )
            .bind(&rec_id)
            .bind(t_id.to_string())
            .bind(p_str)
            .bind(ws_str)
            .bind(doc_run.id.to_string())
            .bind(&now_str)
            .execute(pool)
            .await;
        }
        Err(e) => {
            tracing::warn!("Không thể khởi chạy doc agent: {e}");
        }
    }
}

async fn handle_task_status_changed(
    pool: &SqlitePool,
    runner: &Arc<dyn TaskRunner>,
    env: &Envelope,
    new_status: &str,
) {
    if new_status != "ready_to_merge" {
        return;
    }

    let Some(ws_id) = env.scope.workspace_id else {
        return;
    };
    let Some(prj_id) = env.scope.project_id else {
        return;
    };
    let Some(t_id) = env.scope.task_id else {
        return;
    };

    let ws_str = ws_id.to_string();
    let p_str = prj_id.to_string();
    let t_str = t_id.to_string();

    if can_trigger_doc_agent(pool, &ws_str, &p_str, &t_str).await {
        dispatch_doc_run(pool, runner, &ws_str, &p_str, t_id).await;
    }
}

/// Chạy vòng lặp lắng nghe sự kiện của Doc Agent (P2-05).
pub async fn run_doc_agent_worker(
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
                match env.payload {
                    EventPayload::TaskStatusChanged { ref new_status, .. } => {
                        if let Some(ref r) = runner {
                            handle_task_status_changed(&pool, r, &env, new_status).await;
                        }
                    }
                    EventPayload::RunFinished { run_id, exit_code } => {
                        let Some(ws_id) = env.scope.workspace_id else { continue };
                        let Some(prj_id) = env.scope.project_id else { continue };
                        let Some(t_id) = env.scope.task_id else { continue };

                        handle_doc_run_finished(
                            &pool,
                            &run_id,
                            exit_code,
                            ws_id,
                            prj_id,
                            t_id,
                        ).await;
                    }
                    _ => {}
                }
            }
        }
    }
}

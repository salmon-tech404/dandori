//! Hiện thực điểm mở rộng `TransitionGuard` cho module `approval-gate` (P2-03).

use std::fmt;
use std::sync::Arc;

use chrono::{Duration, Utc};
use dandori_module_api::{
    BoxFuture, EventBus, GuardDecision, GuardError, TransitionActor, TransitionGuard,
};
use dandori_types::{Task, TaskStatus};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::model::ApprovalPolicy;

/// Bộ kiểm soát chuyển trạng thái nhiệm vụ dựa trên chính sách phê duyệt.
pub struct ApprovalGateGuard {
    pool: Option<SqlitePool>,
    _event_bus: Arc<EventBus>,
}

impl ApprovalGateGuard {
    /// Khởi tạo instance mới của `ApprovalGateGuard`.
    #[must_use]
    pub fn new(pool: Option<SqlitePool>, event_bus: Arc<EventBus>) -> Self {
        Self {
            pool,
            _event_bus: event_bus,
        }
    }

    async fn fetch_policy(pool: &SqlitePool, project_id: &str) -> ApprovalPolicy {
        let row: Option<(bool, bool, i64, String)> = sqlx::query_as(
            "SELECT require_in_review_to_merge, disallow_self_approval, expiry_hours, workspace_id \
             FROM m_approval_gate_policy \
             WHERE project_id = ?",
        )
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

        match row {
            Some((require, disallow, expiry, ws_id)) => ApprovalPolicy {
                workspace_id: ws_id,
                project_id: project_id.to_string(),
                require_in_review_to_merge: require,
                disallow_self_approval: disallow,
                expiry_hours: expiry,
            },
            None => ApprovalPolicy {
                workspace_id: String::new(),
                project_id: project_id.to_string(),
                require_in_review_to_merge: true,
                disallow_self_approval: true,
                expiry_hours: 24,
            },
        }
    }

    async fn create_request(
        pool: &SqlitePool,
        task: &Task,
        actor: &TransitionActor,
        policy: &ApprovalPolicy,
    ) -> Result<String, GuardError> {
        let req_id = Uuid::new_v4().to_string();
        let requester_id = match actor {
            TransitionActor::User(uid) => uid.clone(),
            TransitionActor::System(src) => src.to_string(),
        };

        let prj_id_str = task.project_id.to_string();
        let task_id_str = task.id.to_string();

        // Lấy workspace_id của task từ project
        let ws_row: Option<(String,)> =
            sqlx::query_as("SELECT workspace_id FROM project WHERE id = ?")
                .bind(&prj_id_str)
                .fetch_optional(pool)
                .await
                .map_err(|e| GuardError::CheckFailed(e.to_string()))?;

        let workspace_id = ws_row
            .map(|(w,)| w)
            .unwrap_or_else(|| policy.workspace_id.clone());

        let now = Utc::now();
        let expires_at = now + Duration::hours(policy.expiry_hours);
        let now_str = now.to_rfc3339();
        let expires_str = expires_at.to_rfc3339();

        sqlx::query(
            "INSERT INTO m_approval_gate_request \
             (id, workspace_id, project_id, task_id, kind, from_status, target_status, \
              requester_id, status, reason, created_at, expires_at) \
             VALUES (?, ?, ?, ?, 'transition', 'in_review', 'ready_to_merge', ?, 'pending', ?, ?, ?)",
        )
        .bind(&req_id)
        .bind(&workspace_id)
        .bind(&prj_id_str)
        .bind(&task_id_str)
        .bind(&requester_id)
        .bind("Yêu cầu duyệt chuyển từ in_review sang ready_to_merge")
        .bind(&now_str)
        .bind(&expires_str)
        .execute(pool)
        .await
        .map_err(|e| GuardError::CheckFailed(e.to_string()))?;

        Ok(req_id)
    }
}

impl fmt::Debug for ApprovalGateGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApprovalGateGuard").finish_non_exhaustive()
    }
}

impl TransitionGuard for ApprovalGateGuard {
    fn name(&self) -> &'static str {
        "approval-gate"
    }

    fn check(
        &self,
        task: &Task,
        from: TaskStatus,
        to: TaskStatus,
        actor: &TransitionActor,
    ) -> BoxFuture<'static, Result<GuardDecision, GuardError>> {
        let pool = self.pool.clone();
        let task = task.clone();
        let actor = actor.clone();

        Box::pin(async move {
            let Some(pool) = pool else {
                return Ok(GuardDecision::Allow);
            };

            // Chỉ can thiệp khi chuyển từ in_review sang ready_to_merge
            if from == TaskStatus::InReview && to == TaskStatus::ReadyToMerge {
                let prj_id_str = task.project_id.to_string();
                let policy = Self::fetch_policy(&pool, &prj_id_str).await;
                if policy.require_in_review_to_merge {
                    let _req_id = Self::create_request(&pool, &task, &actor, &policy).await?;
                    return Ok(GuardDecision::NeedsApproval {
                        kind: "approval-gate".to_string(),
                        reason: "Chuyển từ in_review sang ready_to_merge yêu cầu phê duyệt"
                            .to_string(),
                    });
                }
            }

            Ok(GuardDecision::Allow)
        })
    }
}

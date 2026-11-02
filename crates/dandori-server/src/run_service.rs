//! Lõi dịch vụ chạy task bằng agent và quản lý các lần chạy (RunService - P1-01).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use dandori_agent_port::{AgentAccountRef, AgentAdapter, AgentEvent, AgentRole, AgentRunSpec};
use dandori_db::repo::{
    AgentAccountRepo, ProjectRepo, RunEventRepo, RunRepo, TaskRepo, WorkspaceRepo,
};
use dandori_events::EventBus;
use dandori_module_api::{BoxFuture, ReleaseReason, TaskRunner, Workdir, WorkdirProvider};
use dandori_registry::lifecycle::FeatureLifecycleManager;
use dandori_types::{
    AgentAccountId, Envelope, EventPayload, EventScope, ProjectId, Run, RunEvent, RunEventId,
    RunId, Task, TaskId, TaskStatus, UserId, WorkspaceId,
};
use sqlx::{Row, SqlitePool};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// Giới hạn số ký tự tối đa của toàn bộ chuỗi prompt được lắp ráp (32KB).
pub const MAX_PROMPT_CHARS: usize = 32_000;

/// Giới hạn số lần chạy đồng thời mặc định toàn hệ thống.
pub const DEFAULT_MAX_CONCURRENT_RUNS: usize = 3;

/// Lỗi phát sinh trong quá trình vận hành `RunService`.
#[derive(Debug, thiserror::Error)]
pub enum RunServiceError {
    /// Không tìm thấy task yêu cầu.
    #[error("không tìm thấy task: {0}")]
    TaskNotFound(TaskId),

    /// Người dùng không có quyền truy cập trên workspace của task.
    #[error("truy cập bị từ chối: {0}")]
    Forbidden(String),

    /// Task đã ở trạng thái hoàn thành.
    #[error("task đã hoàn thành ({0}), không thể chạy lại")]
    TaskAlreadyCompleted(TaskId),

    /// Task đang trong quá trình chạy.
    #[error("task đang chạy ({0}), không thể bắt đầu lần chạy mới")]
    TaskAlreadyRunning(TaskId),

    /// Chuyển trạng thái task không hợp lệ.
    #[error("không thể chuyển task từ trạng thái '{0}' sang 'running'")]
    InvalidTaskTransition(String),

    /// Thiếu điểm mở rộng WorkdirProvider (cần bật module worktree).
    #[error("thiếu điểm mở rộng WorkdirProvider: cần bật module worktree")]
    WorkdirProviderMissing,

    /// Không có tài khoản agent nào sẵn sàng.
    #[error("không có tài khoản agent sẵn sàng")]
    NoAvailableAccount,

    /// Vượt quá số lần chạy đồng thời tối đa cho phép.
    #[error("vượt quá số lần chạy đồng thời tối đa ({0})")]
    ConcurrencyLimitExceeded(usize),

    /// Không tìm thấy adapter phù hợp cho nhà cung cấp.
    #[error("không tìm thấy adapter cho nhà cung cấp: {0}")]
    AdapterNotFound(String),

    /// Lỗi từ phía điểm mở rộng thư mục làm việc.
    #[error("lỗi chuẩn bị thư mục làm việc: {0}")]
    WorkdirError(String),

    /// Không tìm thấy lần chạy.
    #[error("không tìm thấy lần chạy: {0}")]
    RunNotFound(RunId),

    /// Lỗi cơ sở dữ liệu bên dưới.
    #[error("lỗi cơ sở dữ liệu: {0}")]
    Db(#[from] dandori_db::DbError),

    /// Lỗi nội bộ khác.
    #[error("lỗi hệ thống: {0}")]
    Internal(String),
}

/// Dịch vụ điều phối lần chạy task của agent (RunService).
#[derive(Clone)]
pub struct RunService {
    db: SqlitePool,
    event_bus: Arc<EventBus>,
    lifecycle_manager: Arc<FeatureLifecycleManager>,
    adapters: Arc<RwLock<HashMap<String, Arc<dyn AgentAdapter>>>>,
    max_concurrent_runs: usize,
    active_runs: Arc<RwLock<HashMap<RunId, (TaskId, CancellationToken)>>>,
}

impl std::fmt::Debug for RunService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunService")
            .field("max_concurrent_runs", &self.max_concurrent_runs)
            .finish_non_exhaustive()
    }
}

impl RunService {
    /// Khởi tạo một RunService mới.
    #[must_use]
    pub fn new(
        db: SqlitePool,
        event_bus: Arc<EventBus>,
        lifecycle_manager: Arc<FeatureLifecycleManager>,
        max_concurrent_runs: usize,
    ) -> Self {
        Self {
            db,
            event_bus,
            lifecycle_manager,
            adapters: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent_runs,
            active_runs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Đăng ký adapter xử lý cho một nhà cung cấp cụ thể (ví dụ "fake-agent", "claude").
    pub async fn register_adapter(
        &self,
        provider: impl Into<String>,
        adapter: Arc<dyn AgentAdapter>,
    ) {
        let mut lock = self.adapters.write().await;
        lock.insert(provider.into().to_lowercase(), adapter);
    }

    /// Khởi động một lần chạy mới cho task.
    #[expect(
        clippy::too_many_lines,
        reason = "Logic khởi tạo run kiểm tra đầy đủ điều kiện và quyền truy cập"
    )]
    pub async fn start_run(
        &self,
        task_id: TaskId,
        role: AgentRole,
        user_id: UserId,
        exclude_accounts: Vec<AgentAccountId>,
    ) -> Result<Run, RunServiceError> {
        // 1. Kiểm tra giới hạn chạy đồng thời
        {
            let active_count = self.active_runs.read().await.len();
            if active_count >= self.max_concurrent_runs {
                return Err(RunServiceError::ConcurrencyLimitExceeded(
                    self.max_concurrent_runs,
                ));
            }
        }

        // 2. Kiểm tra task tồn tại và quyền truy cập
        let (task, workspace_id) = self.verify_task_access(&task_id, &user_id).await?;

        // 3. Kiểm tra trạng thái task
        let current_status = task
            .status
            .parse::<TaskStatus>()
            .map_err(|_| RunServiceError::InvalidTaskTransition(task.status.clone()))?;

        if current_status == TaskStatus::Running {
            return Err(RunServiceError::TaskAlreadyRunning(task_id));
        }
        if current_status == TaskStatus::Done {
            return Err(RunServiceError::TaskAlreadyCompleted(task_id));
        }

        // 4. Kiểm tra điểm mở rộng WorkdirProvider (bắt buộc đúng 1 provider)
        let workdir_provider = self
            .lifecycle_manager
            .get_workdir_provider()
            .await
            .map_err(|e| match e {
                dandori_registry::error::RegistryError::WorkdirProviderMissing => {
                    RunServiceError::WorkdirProviderMissing
                }
                other => RunServiceError::Internal(other.to_string()),
            })?;

        // 5. Chọn tài khoản agent khả dụng dựa theo vai trò và cấu hình lead-worker
        let fallback_enabled = sqlx::query_scalar::<_, i64>(
            "SELECT fallback_enabled FROM m_api_key_fallback_project_config WHERE project_id = ?",
        )
        .bind(task.project_id.to_string())
        .fetch_optional(&self.db)
        .await
        .unwrap_or(None)
        .unwrap_or(0)
            == 1;

        let mut effective_excludes = exclude_accounts.clone();
        if !fallback_enabled
            && let Ok(all_accs) = AgentAccountRepo::new(&self.db).list_accounts().await
        {
            for a in all_accs {
                if a.auth_type == "api_key" {
                    effective_excludes.push(a.id);
                }
            }
        }

        let preferred_providers = self.resolve_preferred_providers(&task, &role).await;
        let mut account = None;
        for prov in &preferred_providers {
            if let Some(acc) = AgentAccountRepo::new(&self.db)
                .find_available_account(Some(prov), &effective_excludes)
                .await?
            {
                account = Some(acc);
                break;
            }
        }
        let account = match account {
            Some(acc) => acc,
            None => AgentAccountRepo::new(&self.db)
                .find_available_account(None, &effective_excludes)
                .await?
                .ok_or(RunServiceError::NoAvailableAccount)?,
        };

        // Lấy adapter trước khi đổi bất kỳ trạng thái nào: không có adapter thì báo lỗi ngay,
        // task không bị kẹt ở "running".
        let adapter = self.get_adapter(&account.provider).await?;

        // Chiếm task nguyên tử: hai yêu cầu đồng thời cho cùng task chỉ một bên thắng.
        let old_status = task.status.clone();
        if !TaskRepo::new(&self.db)
            .try_claim_for_run(&task.project_id, &task_id)
            .await?
        {
            return Err(RunServiceError::TaskAlreadyRunning(task_id));
        }

        // 6. Chuẩn bị thư mục làm việc qua WorkdirProvider
        let workdir = match workdir_provider.prepare(&task).await {
            Ok(w) => w,
            Err(e) => {
                self.revert_claim(&task, &old_status).await;
                return Err(RunServiceError::WorkdirError(e.to_string()));
            }
        };

        let path_str = workdir.path.to_string_lossy().to_string();
        let _ = TaskRepo::new(&self.db)
            .update_task_workdir(&task.project_id, &task.id, Some(&path_str))
            .await;

        // 7. Lắp ráp nội dung prompt từ task và các PromptContributor
        let prompt = self.assemble_prompt(&task, &role).await;

        // 8. Tạo bản ghi Run trong DB
        let run_id = RunId::new();
        let run = Run {
            id: run_id,
            task_id,
            agent: account.provider.clone(),
            account: account.label.clone(),
            status: "running".into(),
            started_at: Utc::now(),
            finished_at: None,
            exit_code: None,
        };
        if let Err(e) = RunRepo::new(&self.db).create_run(&run).await {
            self.revert_claim(&task, &old_status).await;
            return Err(e.into());
        }

        // 9. Cập nhật trạng thái task sang "running"
        let workdir_str = workdir.path.to_string_lossy().to_string();
        TaskRepo::new(&self.db)
            .update_task_workdir(&task.project_id, &task_id, Some(&workdir_str))
            .await?;

        // 10. Phát sự kiện run.started, task.assigned và task.status_changed
        self.publish_run_started(workspace_id, task.project_id, task_id, &run);
        self.publish_task_assigned(workspace_id, task.project_id, task_id, &account.provider);
        self.publish_task_status_changed(
            workspace_id,
            task.project_id,
            task_id,
            &old_status,
            "running",
        );

        // 12. Kích hoạt tác vụ nền chạy agent
        let cancel_token = CancellationToken::new();
        {
            let mut active_lock = self.active_runs.write().await;
            active_lock.insert(run_id, (task_id, cancel_token.clone()));
        }

        let svc = self.clone();
        let task_clone = task.clone();
        let account_clone = account.clone();
        tokio::spawn(async move {
            svc.execute_run_loop(
                run_id,
                task_clone,
                account_clone,
                workdir,
                workdir_provider,
                prompt,
                role,
                adapter,
                cancel_token,
                workspace_id,
            )
            .await;
        });

        Ok(run)
    }

    /// Trả task về trạng thái cũ khi việc chuẩn bị chạy thất bại sau khi đã chiếm task.
    async fn revert_claim(&self, task: &Task, old_status: &str) {
        let _ = TaskRepo::new(&self.db)
            .update_task_status(&task.project_id, &task.id, old_status)
            .await;
    }

    /// Hủy bỏ một lần chạy đang tiến hành.
    pub async fn cancel_run(&self, run_id: RunId, user_id: UserId) -> Result<(), RunServiceError> {
        let (task_id, cancel_token) = {
            let active_lock = self.active_runs.read().await;
            let (t_id, token) = active_lock
                .get(&run_id)
                .ok_or(RunServiceError::RunNotFound(run_id))?;
            (*t_id, token.clone())
        };

        // Kiểm tra quyền trên task của lần chạy
        let _ = self.verify_task_access(&task_id, &user_id).await?;

        // Kích hoạt hủy bỏ
        cancel_token.cancel();
        Ok(())
    }

    /// Lấy thông tin lần chạy theo ID.
    pub async fn get_run(&self, run_id: RunId, user_id: UserId) -> Result<Run, RunServiceError> {
        let run = RunRepo::new(&self.db)
            .get_run_by_id(&run_id)
            .await?
            .ok_or(RunServiceError::RunNotFound(run_id))?;

        // Kiểm tra quyền
        let _ = self.verify_task_access(&run.task_id, &user_id).await?;
        Ok(run)
    }

    /// Lấy danh sách sự kiện của lần chạy có phân trang.
    pub async fn list_run_events(
        &self,
        run_id: RunId,
        user_id: UserId,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<RunEvent>, i64), RunServiceError> {
        let run = self.get_run(run_id, user_id).await?;
        let (events, total) = RunEventRepo::new(&self.db)
            .list_run_events_paginated(&run.id, limit, offset)
            .await?;
        Ok((events, total))
    }

    async fn verify_task_access(
        &self,
        task_id: &TaskId,
        user_id: &UserId,
    ) -> Result<(Task, WorkspaceId), RunServiceError> {
        let task = TaskRepo::new(&self.db)
            .get_task_by_id(task_id)
            .await?
            .ok_or(RunServiceError::TaskNotFound(*task_id))?;

        let project = ProjectRepo::new(&self.db)
            .get_project_by_id(&task.project_id)
            .await?
            .ok_or_else(|| RunServiceError::Internal("Không tìm thấy dự án sở hữu task".into()))?;

        let member = WorkspaceRepo::new(&self.db)
            .get_member(&project.workspace_id, user_id)
            .await?;

        if member.is_none() {
            return Err(RunServiceError::Forbidden(
                "Người dùng không thuộc workspace chứa task này".into(),
            ));
        }

        Ok((task, project.workspace_id))
    }

    async fn assemble_prompt(&self, task: &Task, role: &AgentRole) -> String {
        let mut prompt = format!("Task: {}\n\n{}", task.title, task.description);

        let contributors = self.lifecycle_manager.get_prompt_contributors().await;
        let mut all_parts = Vec::new();
        for c in &contributors {
            let parts = c.contribute(task, role).await;
            all_parts.extend(parts);
        }

        all_parts.sort_by(|a, b| a.order.cmp(&b.order).then_with(|| a.source.cmp(&b.source)));

        for part in all_parts {
            prompt.push_str(&format!("\n\n[Nguồn: {}]\n{}", part.source, part.content));
        }

        if prompt.chars().count() > MAX_PROMPT_CHARS {
            const MARKER: &str = "\n[CẮT BỚT DO VƯỢT QUÁ GIỚI HẠN 32000 KÝ TỰ]";
            let take_len = MAX_PROMPT_CHARS.saturating_sub(MARKER.chars().count());
            let truncated: String = prompt.chars().take(take_len).collect();
            prompt = format!("{truncated}{MARKER}");
        }

        prompt
    }

    /// Kiểm tra đã có adapter đăng ký cho nhà cung cấp hay chưa.
    pub async fn has_adapter(&self, provider: &str) -> bool {
        self.get_adapter(provider).await.is_ok()
    }

    async fn get_adapter(&self, provider: &str) -> Result<Arc<dyn AgentAdapter>, RunServiceError> {
        let lock = self.adapters.read().await;
        if let Some(adapter) = lock.get(&provider.to_lowercase()) {
            return Ok(adapter.clone());
        }

        // Nếu là provider fake hoặc có fallback fake
        if (provider.eq_ignore_ascii_case("fake") || provider.eq_ignore_ascii_case("fake-agent"))
            && let Some(adapter) = lock.get("fake-agent").or_else(|| lock.get("fake"))
        {
            return Ok(adapter.clone());
        }

        Err(RunServiceError::AdapterNotFound(provider.to_string()))
    }

    #[expect(
        clippy::too_many_lines,
        clippy::too_many_arguments,
        reason = "Vòng lặp thực thi agent xử lý đầy đủ các nhánh sự kiện và kết thúc"
    )]
    async fn execute_run_loop(
        &self,
        run_id: RunId,
        task: Task,
        account: dandori_types::AgentAccount,
        workdir: Workdir,
        workdir_provider: Arc<dyn WorkdirProvider>,
        prompt: String,
        role: AgentRole,
        adapter: Arc<dyn AgentAdapter>,
        cancel_token: CancellationToken,
        workspace_id: WorkspaceId,
    ) {
        let mut spec = AgentRunSpec::new(workdir.path.clone(), prompt, role);
        spec.account = Some(AgentAccountRef {
            id: account.id.to_string(),
            provider: account.provider.clone(),
            label: account.label.clone(),
        });
        if !account.home_dir.is_empty() {
            spec.isolated_home = Some(PathBuf::from(&account.home_dir));
        }
        spec.total_timeout = Some(Duration::from_secs(600));
        spec.silent_timeout = Some(Duration::from_secs(120));

        if account.auth_type == "api_key"
            && let Some(ref sec_id) = account.secret_id
        {
            let storage = dandori_vault::SqliteSecretStorage::new(self.db.clone());
            let vault = dandori_vault::Vault::new(storage, dandori_vault::default_key_store());
            if let Ok(secret_box) = vault.reveal(sec_id).await {
                let env_var_name = match account.provider.as_str() {
                    "claude" => "ANTHROPIC_API_KEY",
                    "gemini" => "GEMINI_API_KEY",
                    "codex" => "OPENAI_API_KEY",
                    _ => "ANTHROPIC_API_KEY",
                };
                spec.env.insert(
                    env_var_name.to_string(),
                    secret_box.expose_secret().to_string(),
                );
            }
        }

        let mut seq: i64 = 0;
        let mut run_error: Option<String> = None;
        let mut final_exit_code: Option<i64> = None;
        let mut is_quota_exhausted = false;

        if account.auth_type == "api_key" {
            seq += 1;
            self.record_and_publish_output(
                run_id,
                seq,
                "stdout",
                "[Hệ thống] Nhiệm vụ đang sử dụng tài khoản API key dự phòng",
                workspace_id,
                task.project_id,
                task.id,
            )
            .await;
        }

        match adapter.start_run(spec).await {
            Ok(mut rx) => loop {
                tokio::select! {
                    () = cancel_token.cancelled() => {
                        run_error = Some("Lần chạy bị hủy bởi người dùng".into());
                        break;
                    }
                    res = rx.recv() => {
                        match res {
                            Some(event) => {
                                match event {
                                    AgentEvent::Text(txt) => {
                                        seq += 1;
                                        let clean = redact_sensitive_content(&txt);
                                        self.record_and_publish_output(run_id, seq, "stdout", &clean, workspace_id, task.project_id, task.id).await;
                                    }
                                    AgentEvent::ToolCall { id, name, arguments } => {
                                        seq += 1;
                                        let clean_args = redact_sensitive_content(&arguments);
                                        let data = serde_json::json!({
                                            "tool_id": id,
                                            "name": name,
                                            "arguments": clean_args
                                        }).to_string();
                                        self.record_and_publish_output(run_id, seq, "tool_call", &data, workspace_id, task.project_id, task.id).await;
                                    }
                                    AgentEvent::QuotaExhausted { message } => {
                                        is_quota_exhausted = true;
                                        run_error = Some(format!("Hết hạn mức: {message}"));
                                        let _ = AgentAccountRepo::new(&self.db)
                                            .update_exhausted_until(&account.id, Some(Utc::now() + chrono::Duration::hours(1)))
                                            .await;
                                        break;
                                    }
                                    AgentEvent::Finished { exit_code } => {
                                        final_exit_code = Some(i64::from(exit_code));
                                        break;
                                    }
                                    AgentEvent::Error(err_msg) => {
                                        seq += 1;
                                        let clean = redact_sensitive_content(&err_msg);
                                        self.record_and_publish_output(run_id, seq, "stderr", &clean, workspace_id, task.project_id, task.id).await;
                                    }
                                }
                            }
                            None => break,
                        }
                    }
                }
            },
            Err(e) => {
                run_error = Some(format!("Không thể khởi động agent: {e}"));
            }
        }

        // Dọn dẹp trong active_runs
        {
            let mut active_lock = self.active_runs.write().await;
            active_lock.remove(&run_id);
        }

        // Xử lý trạng thái kết thúc
        let is_cancelled = cancel_token.is_cancelled();
        let (final_run_status, final_task_status, release_reason) = if is_cancelled {
            ("cancelled", "cancelled", ReleaseReason::Cancelled)
        } else if is_quota_exhausted {
            ("failed", "needs_human", ReleaseReason::Failed)
        } else if let Some(code) = final_exit_code {
            if code == 0 {
                ("success", "in_review", ReleaseReason::Completed)
            } else {
                ("failed", "failed", ReleaseReason::Failed)
            }
        } else {
            ("failed", "failed", ReleaseReason::Failed)
        };

        // Cập nhật Run trong DB
        let now = Utc::now();
        let _ = RunRepo::new(&self.db)
            .update_run_status(
                &task.id,
                &run_id,
                final_run_status,
                Some(now),
                final_exit_code,
            )
            .await;

        // Cập nhật Task trong DB
        let _ = TaskRepo::new(&self.db)
            .update_task_status(&task.project_id, &task.id, final_task_status)
            .await;

        // Giải phóng thư mục làm việc qua WorkdirProvider
        let _ = workdir_provider.release(&task, release_reason).await;

        // Phát sự kiện kết thúc
        self.publish_task_status_changed(
            workspace_id,
            task.project_id,
            task.id,
            "running",
            final_task_status,
        );

        if final_run_status == "success" {
            let payload = EventPayload::RunFinished {
                run_id,
                exit_code: final_exit_code,
            };
            self.event_bus.publish(Envelope::new(
                Utc::now(),
                EventScope::task(workspace_id, task.project_id, task.id),
                payload,
            ));
        } else {
            let err_msg = run_error.unwrap_or_else(|| format!("Thoát với mã {final_exit_code:?}"));
            let payload = EventPayload::RunFailed {
                run_id,
                error_message: err_msg,
            };
            self.event_bus.publish(Envelope::new(
                Utc::now(),
                EventScope::task(workspace_id, task.project_id, task.id),
                payload,
            ));
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Ghi nhận và phát tán sự kiện chi tiết yêu cầu đầy đủ các trường"
    )]
    async fn record_and_publish_output(
        &self,
        run_id: RunId,
        seq: i64,
        stream: &str,
        data: &str,
        ws_id: WorkspaceId,
        proj_id: ProjectId,
        task_id: TaskId,
    ) {
        let event = RunEvent {
            id: RunEventId::new(),
            run_id,
            seq,
            event_type: stream.to_string(),
            data: data.to_string(),
            created_at: Utc::now(),
        };

        let _ = RunEventRepo::new(&self.db).create_run_event(&event).await;

        let payload = EventPayload::RunOutput {
            run_id,
            seq,
            stream: stream.to_string(),
            chunk: data.to_string(),
        };
        self.event_bus.publish(Envelope::new(
            Utc::now(),
            EventScope::task(ws_id, proj_id, task_id),
            payload,
        ));
    }

    fn publish_run_started(
        &self,
        ws_id: WorkspaceId,
        proj_id: ProjectId,
        task_id: TaskId,
        run: &Run,
    ) {
        let payload = EventPayload::RunStarted {
            run_id: run.id,
            agent: run.agent.clone(),
            account: run.account.clone(),
        };
        self.event_bus.publish(Envelope::new(
            Utc::now(),
            EventScope::task(ws_id, proj_id, task_id),
            payload,
        ));
    }

    fn publish_task_status_changed(
        &self,
        ws_id: WorkspaceId,
        proj_id: ProjectId,
        task_id: TaskId,
        old_status: &str,
        new_status: &str,
    ) {
        let payload = EventPayload::TaskStatusChanged {
            task_id,
            old_status: old_status.to_string(),
            new_status: new_status.to_string(),
        };
        self.event_bus.publish(Envelope::new(
            Utc::now(),
            EventScope::task(ws_id, proj_id, task_id),
            payload,
        ));
    }

    async fn resolve_preferred_providers(&self, task: &Task, role: &AgentRole) -> Vec<String> {
        match role {
            AgentRole::Worker => {
                if !task.worker_agent.is_empty() {
                    return vec![task.worker_agent.clone()];
                }
                self.get_default_provider_order(
                    &task.project_id,
                    "worker",
                    &["gemini", "claude", "codex"],
                )
                .await
            }
            AgentRole::Lead => {
                if !task.lead_agent.is_empty() {
                    return vec![task.lead_agent.clone()];
                }
                self.get_default_provider_order(
                    &task.project_id,
                    "pm",
                    &["claude", "gemini", "codex"],
                )
                .await
            }
            AgentRole::Custom(r) => {
                self.get_default_provider_order(&task.project_id, r, &["gemini", "claude", "codex"])
                    .await
            }
        }
    }

    async fn get_default_provider_order(
        &self,
        project_id: &ProjectId,
        role_name: &str,
        fallback: &[&str],
    ) -> Vec<String> {
        let proj_str = project_id.to_string();
        let row = sqlx::query(
            "SELECT provider_order FROM m_lead_worker_default WHERE project_id = ? AND role = ?",
        )
        .bind(&proj_str)
        .bind(role_name)
        .fetch_optional(&self.db)
        .await;

        if let Ok(Some(r)) = row {
            let order_json: String = r.try_get("provider_order").unwrap_or_default();
            if let Some(list) = serde_json::from_str::<Vec<String>>(&order_json)
                .ok()
                .filter(|l| !l.is_empty())
            {
                return list;
            }
        }

        fallback.iter().map(|s| s.to_string()).collect()
    }

    fn publish_task_assigned(
        &self,
        workspace_id: WorkspaceId,
        project_id: ProjectId,
        task_id: TaskId,
        agent: &str,
    ) {
        self.event_bus.publish(Envelope::new(
            Utc::now(),
            EventScope::task(workspace_id, project_id, task_id),
            EventPayload::TaskAssigned {
                task_id,
                agent: agent.to_string(),
            },
        ));
    }
}

/// Che giấu các chuỗi nhạy cảm giống API key hoặc token trước khi lưu vào DB hoặc phát sự kiện.
#[must_use]
pub fn redact_sensitive_content(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut i = 0;
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();

    while i < len {
        let matched = dandori_module_api::matched_secret_prefix(&chars, i);
        let matched_prefix_len = matched.map_or(0, |p| p.chars().count());
        let is_bearer = matched == Some("bearer ");

        if matched_prefix_len > 0 {
            let start_token = i + matched_prefix_len;
            let end_token = dandori_module_api::secret_token_end(&chars, start_token);
            if end_token - start_token >= dandori_module_api::MIN_SECRET_TOKEN_LEN {
                if is_bearer {
                    result.push_str("Bearer [REDACTED]");
                } else {
                    result.push_str("[REDACTED]");
                }
                i = end_token;
                continue;
            }
        }

        if let Some(&c) = chars.get(i) {
            result.push(c);
        }
        i += 1;
    }

    result
}

impl TaskRunner for RunService {
    fn start_run(
        &self,
        task_id: TaskId,
        role: AgentRole,
        user_id: UserId,
        exclude_accounts: Vec<AgentAccountId>,
    ) -> BoxFuture<'static, Result<Run, String>> {
        let this = self.clone();
        Box::pin(async move {
            this.start_run(task_id, role, user_id, exclude_accounts)
                .await
                .map_err(|e| e.to_string())
        })
    }
}

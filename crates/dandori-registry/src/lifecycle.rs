//! Quản lý vòng đời thực thi thật sự của các module (T07 & 01_KIEN_TRUC.md mục 4.3, 4.4).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Utc;
use dandori_events::EventBus;
use dandori_module_api::{
    GitHostProvider, Module, ModuleContext, PromptContributor, TaskRunner, TransitionGuard,
    WorkdirProvider,
};
use dandori_types::{Envelope, EventPayload, EventScope, FeatureModuleId};
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;

use crate::error::RegistryError;
use crate::registry::FeatureRegistry;
use crate::types::{ChangeRequest, RegistryRole, RegistryScope};

/// Đại diện cho một phiên bản module đang chạy trong bộ nhớ.
#[derive(Debug)]
struct ActiveModuleInstance {
    cancel_token: CancellationToken,
    worker_handles: Vec<tokio::task::JoinHandle<()>>,
    subscription_handles: Vec<tokio::task::JoinHandle<()>>,
    workdir_provider: Option<Arc<dyn WorkdirProvider>>,
    prompt_contributor: Option<Arc<dyn PromptContributor>>,
    transition_guard: Option<Arc<dyn TransitionGuard>>,
    git_host_provider: Option<Arc<dyn GitHostProvider>>,
}

/// Trình quản lý vòng đời thực thi (bật/tắt thật) cho các module Dandori.
#[derive(Clone, Debug)]
pub struct FeatureLifecycleManager {
    registry: FeatureRegistry,
    pool: SqlitePool,
    event_bus: Arc<EventBus>,
    modules: Arc<RwLock<HashMap<String, Arc<dyn Module>>>>,
    active_modules: Arc<RwLock<HashMap<String, ActiveModuleInstance>>>,
    test_workdir_providers: Arc<RwLock<Vec<Arc<dyn WorkdirProvider>>>>,
    test_prompt_contributors: Arc<RwLock<Vec<Arc<dyn PromptContributor>>>>,
    test_transition_guards: Arc<RwLock<Vec<Arc<dyn TransitionGuard>>>>,
    test_git_host_providers: Arc<RwLock<Vec<Arc<dyn GitHostProvider>>>>,
    task_runner: Arc<std::sync::RwLock<Option<Arc<dyn TaskRunner>>>>,
}

impl FeatureLifecycleManager {
    /// Khởi tạo FeatureLifecycleManager mới.
    #[must_use]
    pub fn new(registry: FeatureRegistry, pool: SqlitePool, event_bus: Arc<EventBus>) -> Self {
        Self {
            registry,
            pool,
            event_bus,
            modules: Arc::new(RwLock::new(HashMap::new())),
            active_modules: Arc::new(RwLock::new(HashMap::new())),
            test_workdir_providers: Arc::new(RwLock::new(Vec::new())),
            test_prompt_contributors: Arc::new(RwLock::new(Vec::new())),
            test_transition_guards: Arc::new(RwLock::new(Vec::new())),
            test_git_host_providers: Arc::new(RwLock::new(Vec::new())),
            task_runner: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    /// Thiết lập bộ điều phối TaskRunner dùng chung cho các module.
    pub fn set_task_runner(&self, runner: Arc<dyn TaskRunner>) {
        if let Ok(mut lock) = self.task_runner.write() {
            *lock = Some(runner);
        }
    }

    /// Đăng ký một định nghĩa module có sẵn trong hệ thống.
    pub async fn register_module(&self, module: Arc<dyn Module>) {
        let mut lock = self.modules.write().await;
        lock.insert(module.manifest().id.clone(), module);
    }

    /// Kiểm tra xem module có đang chạy trong tiến trình hay không.
    pub async fn is_module_active(&self, module_id: &str) -> bool {
        let lock = self.active_modules.read().await;
        lock.contains_key(module_id)
    }

    /// Tham chiếu tới FeatureRegistry bên dưới.
    #[must_use]
    pub fn registry(&self) -> &FeatureRegistry {
        &self.registry
    }

    /// Lấy điểm mở rộng `WorkdirProvider` duy nhất đang hoạt động.
    ///
    /// # Errors
    /// Trả về `RegistryError::WorkdirProviderMissing` nếu không có provider nào hoạt động.
    /// Trả về `RegistryError::AmbiguousWorkdirProvider` nếu có nhiều hơn 1 provider.
    pub async fn get_workdir_provider(&self) -> Result<Arc<dyn WorkdirProvider>, RegistryError> {
        let mut providers = Vec::new();
        {
            let lock = self.active_modules.read().await;
            for inst in lock.values() {
                if let Some(ref wp) = inst.workdir_provider {
                    providers.push(wp.clone());
                }
            }
        }
        {
            let test_lock = self.test_workdir_providers.read().await;
            for wp in test_lock.iter() {
                providers.push(wp.clone());
            }
        }

        match providers.len() {
            0 => Err(RegistryError::WorkdirProviderMissing),
            1 => Ok(providers.remove(0)),
            count => Err(RegistryError::AmbiguousWorkdirProvider(count)),
        }
    }

    /// Lấy danh sách toàn bộ các điểm mở rộng `PromptContributor` đang hoạt động.
    pub async fn get_prompt_contributors(&self) -> Vec<Arc<dyn PromptContributor>> {
        let mut contributors = Vec::new();
        {
            let lock = self.active_modules.read().await;
            for inst in lock.values() {
                if let Some(ref pc) = inst.prompt_contributor {
                    contributors.push(pc.clone());
                }
            }
        }
        {
            let test_lock = self.test_prompt_contributors.read().await;
            for pc in test_lock.iter() {
                contributors.push(pc.clone());
            }
        }
        contributors
    }

    /// Lấy danh sách toàn bộ các điểm mở rộng `TransitionGuard` đang hoạt động (P2-03).
    pub async fn get_transition_guards(&self) -> Vec<Arc<dyn TransitionGuard>> {
        let mut guards = Vec::new();
        {
            let lock = self.active_modules.read().await;
            for inst in lock.values() {
                if let Some(ref tg) = inst.transition_guard {
                    guards.push(tg.clone());
                }
            }
        }
        {
            let test_lock = self.test_transition_guards.read().await;
            for tg in test_lock.iter() {
                guards.push(tg.clone());
            }
        }
        guards
    }

    /// Lấy provider `GitHostProvider` theo tên nếu đang hoạt động (P2-06).
    pub async fn get_git_host_provider(&self, name: &str) -> Option<Arc<dyn GitHostProvider>> {
        {
            let lock = self.active_modules.read().await;
            for inst in lock.values() {
                if let Some(ref ghp) = inst.git_host_provider
                    && ghp.name() == name
                {
                    return Some(ghp.clone());
                }
            }
        }
        {
            let test_lock = self.test_git_host_providers.read().await;
            for ghp in test_lock.iter() {
                if ghp.name() == name {
                    return Some(ghp.clone());
                }
            }
        }
        None
    }

    /// Lấy danh sách toàn bộ các điểm mở rộng `GitHostProvider` đang hoạt động (P2-06).
    pub async fn get_git_host_providers(&self) -> Vec<Arc<dyn GitHostProvider>> {
        let mut providers = Vec::new();
        {
            let lock = self.active_modules.read().await;
            for inst in lock.values() {
                if let Some(ref ghp) = inst.git_host_provider {
                    providers.push(ghp.clone());
                }
            }
        }
        {
            let test_lock = self.test_git_host_providers.read().await;
            for ghp in test_lock.iter() {
                providers.push(ghp.clone());
            }
        }
        providers
    }

    /// Đăng ký một `WorkdirProvider` phục vụ kiểm thử đơn vị hoặc giả lập.
    pub async fn register_workdir_provider_for_test(&self, provider: Arc<dyn WorkdirProvider>) {
        let mut lock = self.test_workdir_providers.write().await;
        lock.push(provider);
    }

    /// Đăng ký một `PromptContributor` phục vụ kiểm thử đơn vị hoặc giả lập.
    pub async fn register_prompt_contributor_for_test(
        &self,
        contributor: Arc<dyn PromptContributor>,
    ) {
        let mut lock = self.test_prompt_contributors.write().await;
        lock.push(contributor);
    }

    /// Đăng ký một `TransitionGuard` phục vụ kiểm thử đơn vị hoặc giả lập (P2-03).
    pub async fn register_transition_guard_for_test(&self, guard: Arc<dyn TransitionGuard>) {
        let mut lock = self.test_transition_guards.write().await;
        lock.push(guard);
    }

    /// Đăng ký một `GitHostProvider` phục vụ kiểm thử đơn vị hoặc giả lập (P2-06).
    pub async fn register_git_host_provider_for_test(&self, provider: Arc<dyn GitHostProvider>) {
        let mut lock = self.test_git_host_providers.write().await;
        lock.push(provider);
    }

    /// Đếm số tác vụ worker hoặc subscription đang hoạt động (phục vụ kiểm tra RS-ASYNC-004).
    pub async fn active_task_count(&self) -> usize {
        let lock = self.active_modules.read().await;
        let mut count = 0;
        for inst in lock.values() {
            for h in &inst.worker_handles {
                if !h.is_finished() {
                    count += 1;
                }
            }
            for h in &inst.subscription_handles {
                if !h.is_finished() {
                    count += 1;
                }
            }
        }
        count
    }

    /// Dừng êm toàn bộ các module đang hoạt động, hủy bỏ các tác vụ nền (worker & subscription).
    pub async fn shutdown(&self) {
        let mut lock = self.active_modules.write().await;
        for (_, instance) in lock.drain() {
            instance.cancel_token.cancel();
            for handle in instance.worker_handles {
                let _ = tokio::time::timeout(Duration::from_millis(500), handle).await;
            }
            for handle in instance.subscription_handles {
                let _ = tokio::time::timeout(Duration::from_millis(500), handle).await;
            }
        }
    }

    /// Bật module và các module phụ thuộc theo thứ tự topo.
    ///
    /// # Errors
    /// Trả về `RegistryError` nếu quyền hạn không đủ, migration lỗi hoặc lưu DB thất bại.
    pub async fn enable(
        &self,
        module_id: &str,
        scope: &RegistryScope,
        role: RegistryRole,
        user_id: &str,
        risk_ack: bool,
    ) -> Result<Vec<String>, RegistryError> {
        let plan = self.registry.plan_enable(module_id, scope).await?;
        if plan.needs_risk_ack && !risk_ack {
            return Err(RegistryError::MissingRiskAcknowledgment(
                module_id.to_string(),
            ));
        }

        let mut started_now = Vec::new();
        for mid in &plan.will_enable_with {
            if self.is_module_active(mid).await {
                continue;
            }

            let module = {
                let lock = self.modules.read().await;
                lock.get(mid).cloned()
            };

            if let Some(module) = module {
                if let Err(e) = self.start_module_runtime(mid, &module).await {
                    self.rollback_runtime(&started_now).await;
                    return Err(e);
                }
                started_now.push(mid.clone());
            }
        }

        self.apply_enable_db_and_events(&plan.will_enable_with, scope, role, user_id, risk_ack)
            .await?;

        Ok(plan.will_enable_with)
    }

    async fn start_module_runtime(
        &self,
        module_id: &str,
        module: &Arc<dyn Module>,
    ) -> Result<(), RegistryError> {
        // 1. Chạy migration của module
        for mig in module.migrations() {
            sqlx::query(mig.sql)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    RegistryError::MigrationFailed(format!(
                        "Migration '{}' cho module '{}' thất bại: {e}",
                        mig.id, module_id
                    ))
                })?;
        }

        // 2. Khởi tạo workers & subscriptions & extension points
        let cancel_token = CancellationToken::new();
        let mut ctx = ModuleContext::new(
            module_id.to_string(),
            self.event_bus.clone(),
            serde_json::json!({}),
        )
        .with_pool(self.pool.clone());

        if let Ok(lock) = self.task_runner.read()
            && let Some(ref runner) = *lock
        {
            ctx = ctx.with_task_runner(runner.clone());
        }

        let workdir_provider = module.workdir_provider(&ctx);
        let prompt_contributor = module.prompt_contributor(&ctx);
        let transition_guard = module.transition_guard(&ctx);
        let git_host_provider = module.git_host_provider(&ctx);
        let worker_handles = self.spawn_workers(module, &ctx, &cancel_token);
        let subscription_handles = self.spawn_subscriptions(module, &cancel_token);

        let mut active_lock = self.active_modules.write().await;
        active_lock.insert(
            module_id.to_string(),
            ActiveModuleInstance {
                cancel_token,
                worker_handles,
                subscription_handles,
                workdir_provider,
                prompt_contributor,
                transition_guard,
                git_host_provider,
            },
        );

        Ok(())
    }

    fn spawn_workers(
        &self,
        module: &Arc<dyn Module>,
        ctx: &ModuleContext,
        cancel_token: &CancellationToken,
    ) -> Vec<tokio::task::JoinHandle<()>> {
        let mut handles = Vec::new();
        for spec in module.workers(ctx) {
            let token = cancel_token.child_token();
            let run_fn = spec.run;
            let handle = tokio::spawn(async move {
                run_fn(token).await;
            });
            handles.push(handle);
        }
        handles
    }

    fn spawn_subscriptions(
        &self,
        module: &Arc<dyn Module>,
        cancel_token: &CancellationToken,
    ) -> Vec<tokio::task::JoinHandle<()>> {
        let mut handles = Vec::new();
        for sub in module.subscriptions() {
            let mut rx = self.event_bus.subscribe(sub.filter);
            let token = cancel_token.child_token();
            let handler = sub.handler;
            let handle = tokio::spawn(async move {
                loop {
                    tokio::select! {
                        () = token.cancelled() => break,
                        res = rx.recv() => {
                            match res {
                                Some(env) => handler(env).await,
                                None => break,
                            }
                        }
                    }
                }
            });
            handles.push(handle);
        }
        handles
    }

    async fn rollback_runtime(&self, module_ids: &[String]) {
        let mut lock = self.active_modules.write().await;
        for mid in module_ids {
            if let Some(instance) = lock.remove(mid) {
                instance.cancel_token.cancel();
                for handle in instance.worker_handles {
                    handle.abort();
                }
                for handle in instance.subscription_handles {
                    handle.abort();
                }
            }
        }
    }

    async fn apply_enable_db_and_events(
        &self,
        module_ids: &[String],
        scope: &RegistryScope,
        role: RegistryRole,
        user_id: &str,
        risk_ack: bool,
    ) -> Result<(), RegistryError> {
        for mid in module_ids {
            self.registry
                .change_state(ChangeRequest {
                    module_id: mid.clone(),
                    scope: scope.clone(),
                    enabled: true,
                    locked: false,
                    config: None,
                    reason: "Module enabled via lifecycle".into(),
                    performed_by: user_id.to_string(),
                    actor_role: role,
                    risk_acknowledged: risk_ack,
                })
                .await?;

            let payload = EventPayload::ModuleEnabled {
                module_id: FeatureModuleId::new(mid),
            };
            let env = Envelope::new(Utc::now(), EventScope::empty(), payload);
            self.event_bus.publish(env);
        }
        Ok(())
    }

    /// Tắt module và các module phụ thuộc theo thứ tự topo đảo ngược.
    ///
    /// # Errors
    /// Trả về `RegistryError` nếu có module phụ thuộc cần nó mà `disable_dependents` là false.
    pub async fn disable(
        &self,
        module_id: &str,
        scope: &RegistryScope,
        role: RegistryRole,
        user_id: &str,
        disable_dependents: bool,
    ) -> Result<Vec<String>, RegistryError> {
        let plan = self.registry.plan_disable(module_id, scope).await?;
        if !plan.will_break.is_empty() && !disable_dependents {
            return Err(RegistryError::DependentModulesActive(plan.will_break));
        }

        let mut to_disable = plan.will_break;
        to_disable.push(module_id.to_string());

        // Dừng theo thứ tự ngược: module phụ thuộc dừng trước, module bị yêu cầu dừng sau cùng
        for mid in to_disable.iter().rev() {
            self.stop_single_module(mid, scope, role, user_id).await?;
        }

        Ok(to_disable)
    }

    async fn stop_single_module(
        &self,
        mid: &str,
        scope: &RegistryScope,
        role: RegistryRole,
        user_id: &str,
    ) -> Result<(), RegistryError> {
        self.registry
            .change_state(ChangeRequest {
                module_id: mid.to_string(),
                scope: scope.clone(),
                enabled: false,
                locked: false,
                config: None,
                reason: "Module disabled via lifecycle".into(),
                performed_by: user_id.to_string(),
                actor_role: role,
                risk_acknowledged: false,
            })
            .await?;

        let instance = {
            let mut lock = self.active_modules.write().await;
            lock.remove(mid)
        };

        if let Some(instance) = instance {
            instance.cancel_token.cancel();
            for handle in instance.worker_handles {
                let _ = tokio::time::timeout(Duration::from_millis(500), handle).await;
            }
            for handle in instance.subscription_handles {
                let _ = tokio::time::timeout(Duration::from_millis(500), handle).await;
            }
        }

        let payload = EventPayload::ModuleDisabled {
            module_id: FeatureModuleId::new(mid),
        };
        let env = Envelope::new(Utc::now(), EventScope::empty(), payload);
        self.event_bus.publish(env);

        Ok(())
    }

    /// Khởi động lại worker của module khi có cập nhật cấu hình cần restart (requires_worker_restart).
    ///
    /// # Errors
    /// Trả về `RegistryError` nếu module không tìm thấy hoặc đang không hoạt động.
    pub async fn restart_module_workers(&self, module_id: &str) -> Result<(), RegistryError> {
        let module = {
            let lock = self.modules.read().await;
            lock.get(module_id)
                .cloned()
                .ok_or_else(|| RegistryError::ModuleNotFound(module_id.to_string()))?
        };

        let mut lock = self.active_modules.write().await;
        let instance = lock
            .get_mut(module_id)
            .ok_or_else(|| RegistryError::ModuleNotFound(module_id.to_string()))?;

        instance.cancel_token.cancel();
        for handle in instance.worker_handles.drain(..) {
            let _ = tokio::time::timeout(Duration::from_millis(500), handle).await;
        }

        let new_token = CancellationToken::new();
        instance.cancel_token = new_token.clone();

        let ctx = ModuleContext::new(
            module_id.to_string(),
            self.event_bus.clone(),
            serde_json::json!({}),
        )
        .with_pool(self.pool.clone());

        instance.worker_handles = self.spawn_workers(&module, &ctx, &new_token);
        Ok(())
    }

    /// Khôi phục các module đang bật khi máy chủ khởi động lại dựa trên cài đặt cơ sở dữ liệu.
    ///
    /// # Errors
    /// Trả về `RegistryError` nếu đọc cơ sở dữ liệu hoặc nạp module thất bại.
    pub async fn restore_active_modules(&self) -> Result<(), RegistryError> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT DISTINCT module_id FROM feature_setting WHERE enabled = 1")
                .fetch_all(&self.pool)
                .await?;

        for (mid,) in rows {
            let module = {
                let lock = self.modules.read().await;
                lock.get(&mid).cloned()
            };

            if let Some(module) = module
                && !self.is_module_active(&mid).await
            {
                self.start_module_runtime(&mid, &module).await?;
            }
        }

        Ok(())
    }

    /// Tạo axum Router bảo vệ các route `/api/v1/m/<id>/...` bằng cổng kiểm tra trạng thái bật/tắt.
    pub fn create_router(&self) -> axum::Router {
        let lifecycle = self.clone();
        axum::Router::new().fallback(move |req: axum::extract::Request| {
            let lc = lifecycle.clone();
            async move { lc.handle_gated_request(req).await }
        })
    }

    async fn handle_gated_request(&self, req: axum::extract::Request) -> axum::response::Response {
        let req_id = uuid::Uuid::new_v4().to_string();
        let session_token = extract_session_token(&req);

        // 1. Kiểm tra xác thực phiên trước khi kiểm tra trạng thái module (T10 AC4 / T14B mục 2)
        if !verify_token_session(&self.pool, session_token.as_deref()).await {
            return (
                StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({
                    "error": {
                        "code": "unauthorized",
                        "message": "Phiên đăng nhập không hợp lệ hoặc đã hết hạn",
                        "request_id": req_id
                    }
                })),
            )
                .into_response();
        }

        let path = req.uri().path().to_string();
        let rest = path
            .strip_prefix("/api/v1/m/")
            .unwrap_or_else(|| path.trim_start_matches('/'));

        let parts: Vec<&str> = rest.splitn(2, '/').collect();
        let module_id = match parts.first() {
            Some(id) if !id.is_empty() => *id,
            _ => return (StatusCode::NOT_FOUND, "Not found").into_response(),
        };

        if !self.is_module_active(module_id).await {
            return (
                StatusCode::FORBIDDEN,
                axum::Json(serde_json::json!({
                    "error": {
                        "code": "module_disabled",
                        "message": "module đang tắt",
                        "request_id": req_id
                    }
                })),
            )
                .into_response();
        }

        let module = {
            let lock = self.modules.read().await;
            lock.get(module_id).cloned()
        };

        match module {
            Some(m) => {
                self.forward_to_module(&m, module_id, parts.get(1).copied(), req)
                    .await
            }
            None => (StatusCode::NOT_FOUND, "Module not found").into_response(),
        }
    }

    async fn forward_to_module(
        &self,
        module: &Arc<dyn Module>,
        module_id: &str,
        subpath_opt: Option<&str>,
        mut req: axum::extract::Request,
    ) -> axum::response::Response {
        let ctx = ModuleContext::new(
            module_id.to_string(),
            self.event_bus.clone(),
            serde_json::json!({}),
        )
        .with_pool(self.pool.clone());

        let module_router = module.routes(&ctx);
        let subpath = match (subpath_opt, req.uri().query()) {
            (Some(p), Some(q)) => format!("/{p}?{q}"),
            (Some(p), None) => format!("/{p}"),
            (None, Some(q)) => format!("/?{q}"),
            (None, None) => "/".to_string(),
        };

        if let Ok(uri) = subpath.parse::<axum::http::Uri>() {
            *req.uri_mut() = uri;
        }

        match module_router.oneshot(req).await {
            Ok(res) => res.into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal module error").into_response(),
        }
    }
}

fn extract_session_token(req: &axum::extract::Request) -> Option<String> {
    let cookie_header = req
        .headers()
        .get(axum::http::header::COOKIE)?
        .to_str()
        .ok()?;
    for pair in cookie_header.split(';') {
        let mut parts = pair.trim().splitn(2, '=');
        let key = parts.next()?.trim();
        let val = parts.next()?.trim();
        if key == "dandori_session" && !val.is_empty() {
            return Some(val.to_string());
        }
    }
    None
}

async fn verify_token_session(pool: &SqlitePool, token: Option<&str>) -> bool {
    let Some(token) = token else {
        return false;
    };
    let token_hash = dandori_db::hash_session_token(token);
    let now_str = Utc::now().to_rfc3339();
    let row = sqlx::query("SELECT id FROM session WHERE token_hash = ? AND expires_at > ?")
        .bind(token_hash)
        .bind(now_str)
        .fetch_optional(pool)
        .await;

    matches!(row, Ok(Some(_)))
}

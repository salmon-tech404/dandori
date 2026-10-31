//! Quản lý vòng đời thực thi thật sự của các module (T07 & 01_KIEN_TRUC.md mục 4.3, 4.4).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Utc;
use dandori_events::EventBus;
use dandori_module_api::{Module, ModuleContext};
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
}

/// Trình quản lý vòng đời thực thi (bật/tắt thật) cho các module Dandori.
#[derive(Clone, Debug)]
pub struct FeatureLifecycleManager {
    registry: FeatureRegistry,
    pool: SqlitePool,
    event_bus: Arc<EventBus>,
    modules: Arc<RwLock<HashMap<String, Arc<dyn Module>>>>,
    active_modules: Arc<RwLock<HashMap<String, ActiveModuleInstance>>>,
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

        // 2. Khởi tạo workers & subscriptions
        let cancel_token = CancellationToken::new();
        let ctx = ModuleContext::new(
            module_id.to_string(),
            self.event_bus.clone(),
            serde_json::json!({}),
        )
        .with_pool(self.pool.clone());

        let worker_handles = self.spawn_workers(module, &ctx, &cancel_token);
        let subscription_handles = self.spawn_subscriptions(module, &cancel_token);

        let mut active_lock = self.active_modules.write().await;
        active_lock.insert(
            module_id.to_string(),
            ActiveModuleInstance {
                cancel_token,
                worker_handles,
                subscription_handles,
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
            let req_id = uuid::Uuid::new_v4().to_string();
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
        let subpath = match subpath_opt {
            Some(p) => format!("/{p}"),
            None => "/".to_string(),
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

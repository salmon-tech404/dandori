//! Hiện thực Feature Registry quản lý trạng thái, tính hiệu lực và audit (01_KIEN_TRUC.md mục 5 & T06).

use std::collections::{HashMap, HashSet, VecDeque};

use chrono::Utc;
use sqlx::SqlitePool;

use crate::error::RegistryError;
use crate::types::{
    ChangeRequest, DisablePlan, EffectiveState, EnablePlan, RegistryRole, RegistryScope,
};
use dandori_db::repo::FeatureRepo;
use dandori_types::{
    FeatureAudit, FeatureAuditId, FeatureModule, FeatureModuleId, FeaturePack, FeaturePackId,
    FeatureSetting,
};

/// Quản lý trạng thái, tính hiệu lực, quyền hạn, audit và các gói mẫu tính năng.
#[derive(Debug, Clone)]
pub struct FeatureRegistry {
    pool: SqlitePool,
}

impl FeatureRegistry {
    /// Khởi tạo FeatureRegistry với pool cơ sở dữ liệu.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    fn repo(&self) -> FeatureRepo<'_> {
        FeatureRepo::new(&self.pool)
    }

    /// Khởi tạo dữ liệu các module chuẩn và gói mẫu ban đầu.
    ///
    /// # Errors
    /// Trả về `RegistryError` nếu thao tác cơ sở dữ liệu thất bại.
    pub async fn seed_modules_and_packs(&self) -> Result<(), RegistryError> {
        self.seed_default_modules().await?;
        self.seed_default_packs().await?;
        Ok(())
    }

    async fn seed_default_modules(&self) -> Result<(), RegistryError> {
        let sample_mod = FeatureModule {
            id: FeatureModuleId::new("sample-hello"),
            group_name: "safety".into(),
            name: "Module mẫu kiểm thử".into(),
            description: "Module mẫu phục vụ kiểm thử và thử cổng P0".into(),
            version: "0.1.0".into(),
            requires: vec![],
            scopes: vec!["workspace".into(), "project".into()],
            risk: "low".into(),
            effort: "light".into(),
            core: false,
        };
        self.repo().upsert_module(&sample_mod).await?;
        Ok(())
    }

    async fn seed_default_packs(&self) -> Result<(), RegistryError> {
        let repo = self.repo();
        let packs = [
            FeaturePack {
                id: FeaturePackId::new("solo-light"),
                name: "Solo nhẹ".into(),
                modules: vec![
                    "sample-hello".into(),
                    "auth-login".into(),
                    "kanban".into(),
                    "live-status".into(),
                ],
            },
            FeaturePack {
                id: FeaturePackId::new("small-team"),
                name: "Nhóm nhỏ".into(),
                modules: vec![
                    "sample-hello".into(),
                    "auth-login".into(),
                    "kanban".into(),
                    "live-status".into(),
                    "multi-account".into(),
                    "approval-gate".into(),
                ],
            },
            FeaturePack {
                id: FeaturePackId::new("full"),
                name: "Đầy đủ".into(),
                modules: vec![
                    "sample-hello".into(),
                    "auth-login".into(),
                    "kanban".into(),
                    "live-status".into(),
                    "multi-account".into(),
                    "approval-gate".into(),
                    "calendar".into(),
                ],
            },
        ];

        for p in &packs {
            repo.upsert_pack(p).await?;
        }
        Ok(())
    }

    /// Đăng ký nhanh một module cho mục đích kiểm thử.
    ///
    /// # Errors
    /// Trả về `RegistryError` nếu không thể lưu dữ liệu.
    pub async fn register_test_module(
        &self,
        id: &str,
        requires: Vec<String>,
        core: bool,
        high_risk: bool,
    ) -> Result<(), RegistryError> {
        let m = FeatureModule {
            id: FeatureModuleId::new(id),
            group_name: "test".into(),
            name: format!("Test module {id}"),
            description: format!("Description for {id}"),
            version: "0.1.0".into(),
            requires,
            scopes: vec!["system".into(), "workspace".into(), "project".into()],
            risk: if high_risk {
                "high".into()
            } else {
                "low".into()
            },
            effort: "light".into(),
            core,
        };
        self.repo().upsert_module(&m).await?;
        Ok(())
    }

    /// Tính toán trạng thái hiệu lực của module tại một dự án theo 4 bước ở `01_KIEN_TRUC.md` mục 5.
    /// Không lưu kết quả vào cơ sở dữ liệu.
    ///
    /// # Errors
    /// Trả về `RegistryError` nếu truy vấn thất bại.
    pub async fn effective_state(
        &self,
        module_id: &str,
        workspace_id: &str,
        project_id: &str,
    ) -> Result<EffectiveState, RegistryError> {
        let repo = self.repo();
        let mid = FeatureModuleId::new(module_id);

        let sys = repo.get_setting(&mid, "system", "system").await?;
        let ws = repo.get_setting(&mid, "workspace", workspace_id).await?;
        let proj = repo.get_setting(&mid, "project", project_id).await?;

        Ok(Self::resolve_effective(module_id, sys, ws, proj))
    }

    fn resolve_effective(
        module_id: &str,
        sys: Option<FeatureSetting>,
        ws: Option<FeatureSetting>,
        proj: Option<FeatureSetting>,
    ) -> EffectiveState {
        // 1. Nếu cấp hệ thống hoặc workspace đặt locked: dùng giá trị của cấp đó
        if let Some(s) = sys.as_ref().filter(|s| s.locked) {
            return Self::build_state(module_id, s, "system", true);
        }
        if let Some(w) = ws.as_ref().filter(|w| w.locked) {
            return Self::build_state(module_id, w, "workspace", true);
        }

        // 2. Nếu không: giá trị ở cấp dự án nếu có
        if let Some(p) = proj.as_ref() {
            return Self::build_state(module_id, p, "project", p.locked);
        }

        // 3. Nếu không: cấp workspace, rồi cấp hệ thống
        if let Some(w) = ws.as_ref() {
            return Self::build_state(module_id, w, "workspace", false);
        }
        if let Some(s) = sys.as_ref() {
            return Self::build_state(module_id, s, "system", false);
        }

        // 4. Nếu không có giá trị nào: tắt
        EffectiveState {
            module_id: module_id.to_string(),
            enabled: false,
            source_scope: "default".into(),
            locked: false,
            config: None,
        }
    }

    fn build_state(
        module_id: &str,
        s: &FeatureSetting,
        source: &str,
        locked: bool,
    ) -> EffectiveState {
        EffectiveState {
            module_id: module_id.to_string(),
            enabled: s.enabled,
            source_scope: source.into(),
            locked,
            config: serde_json::from_str(&s.config).ok(),
        }
    }

    /// Lập kế hoạch kích hoạt module với danh sách phụ thuộc bắc cầu được sắp xếp theo thứ tự topo.
    ///
    /// # Errors
    /// Trả về `RegistryError` nếu module không tồn tại hoặc có chu trình phụ thuộc.
    pub async fn plan_enable(
        &self,
        module_id: &str,
        scope: &RegistryScope,
    ) -> Result<EnablePlan, RegistryError> {
        let repo = self.repo();
        let all_modules = repo.list_modules().await?;
        let mod_map: HashMap<String, FeatureModule> = all_modules
            .into_iter()
            .map(|m| (m.id.to_string(), m))
            .collect();

        if !mod_map.contains_key(module_id) {
            return Err(RegistryError::ModuleNotFound(module_id.to_string()));
        }

        let (order, needs_risk_ack, risk_note) =
            Self::collect_topo_dependencies(module_id, &mod_map)?;

        let mut will_enable = Vec::new();
        for m_id in order {
            if !self.is_enabled_in_scope(&m_id, scope).await? || m_id == module_id {
                will_enable.push(m_id);
            }
        }

        Ok(EnablePlan {
            module_id: module_id.to_string(),
            will_enable_with: will_enable,
            needs_risk_ack,
            risk_note,
        })
    }

    fn collect_topo_dependencies(
        module_id: &str,
        mod_map: &HashMap<String, FeatureModule>,
    ) -> Result<(Vec<String>, bool, Option<String>), RegistryError> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut order = Vec::new();
        let mut needs_risk = false;
        let mut risk_msg = None;

        Self::dfs_topo(
            module_id,
            mod_map,
            &mut visited,
            &mut rec_stack,
            &mut order,
            &mut needs_risk,
            &mut risk_msg,
        )?;

        Ok((order, needs_risk, risk_msg))
    }

    fn dfs_topo(
        cur: &str,
        mod_map: &HashMap<String, FeatureModule>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        order: &mut Vec<String>,
        needs_risk: &mut bool,
        risk_msg: &mut Option<String>,
    ) -> Result<(), RegistryError> {
        visited.insert(cur.to_string());
        rec_stack.insert(cur.to_string());

        let m = mod_map
            .get(cur)
            .ok_or_else(|| RegistryError::ModuleNotFound(cur.to_string()))?;

        if m.risk != "low" {
            *needs_risk = true;
            if risk_msg.is_none() {
                *risk_msg = Some(format!("Module '{cur}' có mức độ rủi ro {}", m.risk));
            }
        }

        for dep in &m.requires {
            if rec_stack.contains(dep) {
                return Err(RegistryError::DependencyCycle(format!("{cur} -> {dep}")));
            }
            if !visited.contains(dep) {
                Self::dfs_topo(
                    dep, mod_map, visited, rec_stack, order, needs_risk, risk_msg,
                )?;
            }
        }

        rec_stack.remove(cur);
        order.push(cur.to_string());
        Ok(())
    }

    async fn is_enabled_in_scope(
        &self,
        module_id: &str,
        scope: &RegistryScope,
    ) -> Result<bool, RegistryError> {
        match scope {
            RegistryScope::System => {
                let s = self
                    .repo()
                    .get_setting(&FeatureModuleId::new(module_id), "system", "system")
                    .await?;
                Ok(s.is_some_and(|st| st.enabled))
            }
            RegistryScope::Workspace(ws_id) => {
                let st = self.effective_state(module_id, ws_id, "default").await?;
                Ok(st.enabled)
            }
            RegistryScope::Project {
                workspace_id,
                project_id,
            } => {
                let st = self
                    .effective_state(module_id, workspace_id, project_id)
                    .await?;
                Ok(st.enabled)
            }
        }
    }

    /// Lập kế hoạch tắt module, liệt kê mọi module đang bật khác phụ thuộc vào nó (bắc cầu).
    ///
    /// # Errors
    /// Trả về `RegistryError` nếu truy vấn thất bại.
    pub async fn plan_disable(
        &self,
        module_id: &str,
        scope: &RegistryScope,
    ) -> Result<DisablePlan, RegistryError> {
        let repo = self.repo();
        let all_modules = repo.list_modules().await?;
        let mod_map: HashMap<String, FeatureModule> = all_modules
            .into_iter()
            .map(|m| (m.id.to_string(), m))
            .collect();

        if !mod_map.contains_key(module_id) {
            return Err(RegistryError::ModuleNotFound(module_id.to_string()));
        }

        let mut will_break = Vec::new();
        let mut queue = VecDeque::from([module_id.to_string()]);
        let mut seen = HashSet::from([module_id.to_string()]);

        while let Some(current) = queue.pop_front() {
            for (cid, cmod) in &mod_map {
                if cmod.requires.contains(&current)
                    && seen.insert(cid.clone())
                    && self.is_enabled_in_scope(cid, scope).await?
                {
                    will_break.push(cid.clone());
                    queue.push_back(cid.clone());
                }
            }
        }

        will_break.sort();
        Ok(DisablePlan {
            module_id: module_id.to_string(),
            will_break,
        })
    }

    /// Thay đổi cấu hình bật/tắt hoặc settings của module với kiểm tra quyền hạn và khóa phân cấp.
    ///
    /// # Errors
    /// Trả về `RegistryError` nếu vi phạm quyền hạn, khóa cấp trên, module core, hoặc lỗi DB.
    pub async fn change_state(&self, req: ChangeRequest) -> Result<(), RegistryError> {
        let repo = self.repo();
        let mid = FeatureModuleId::new(&req.module_id);

        Self::check_permission(&req)?;

        let mod_info = repo
            .get_module(&mid)
            .await?
            .ok_or_else(|| RegistryError::ModuleNotFound(req.module_id.clone()))?;

        if mod_info.core && !req.enabled {
            return Err(RegistryError::CoreModuleCannotBeDisabled(
                req.module_id.clone(),
            ));
        }

        self.check_upper_locks(&mid, &req).await?;

        if req.enabled && mod_info.risk != "low" && !req.risk_acknowledged {
            return Err(RegistryError::MissingRiskAcknowledgment(
                req.module_id.clone(),
            ));
        }

        self.persist_change(&mid, &req).await?;
        Ok(())
    }

    fn check_permission(req: &ChangeRequest) -> Result<(), RegistryError> {
        if !req.actor_role.can_manage_scope(&req.scope) {
            let required = match req.scope {
                RegistryScope::System => "SystemOwner",
                RegistryScope::Workspace(_) => "WorkspaceAdmin",
                RegistryScope::Project { .. } => "ProjectOwner",
            };
            return Err(RegistryError::PermissionDenied {
                required_role: required.into(),
                actual_role: req.actor_role.as_str().into(),
            });
        }
        Ok(())
    }

    async fn check_upper_locks(
        &self,
        mid: &FeatureModuleId,
        req: &ChangeRequest,
    ) -> Result<(), RegistryError> {
        let repo = self.repo();
        match &req.scope {
            RegistryScope::Project { workspace_id, .. } => {
                let sys = repo.get_setting(mid, "system", "system").await?;
                if sys.is_some_and(|s| s.locked) {
                    return Err(RegistryError::ScopeLocked {
                        locked_at_scope: "system".into(),
                        module_id: req.module_id.clone(),
                    });
                }
                let ws = repo.get_setting(mid, "workspace", workspace_id).await?;
                if ws.is_some_and(|w| w.locked) {
                    return Err(RegistryError::ScopeLocked {
                        locked_at_scope: "workspace".into(),
                        module_id: req.module_id.clone(),
                    });
                }
            }
            RegistryScope::Workspace(_) => {
                let sys = repo.get_setting(mid, "system", "system").await?;
                if sys.is_some_and(|s| s.locked) {
                    return Err(RegistryError::ScopeLocked {
                        locked_at_scope: "system".into(),
                        module_id: req.module_id.clone(),
                    });
                }
            }
            RegistryScope::System => {}
        }
        Ok(())
    }

    async fn persist_change(
        &self,
        mid: &FeatureModuleId,
        req: &ChangeRequest,
    ) -> Result<(), RegistryError> {
        let repo = self.repo();
        let config_str = req
            .config
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?
            .unwrap_or_else(|| "{}".into());

        let setting = FeatureSetting {
            module_id: mid.clone(),
            scope_kind: req.scope.scope_kind().into(),
            scope_id: req.scope.scope_id().into(),
            enabled: req.enabled,
            locked: req.locked,
            config: config_str,
            changed_by: req.performed_by.clone(),
            changed_at: Utc::now(),
        };
        repo.upsert_setting(&setting).await?;

        let action = if req.config.is_some() {
            "configure"
        } else if req.enabled {
            "enable"
        } else {
            "disable"
        };

        let audit = FeatureAudit {
            id: FeatureAuditId::new(),
            module_id: mid.clone(),
            scope_kind: req.scope.scope_kind().into(),
            scope_id: req.scope.scope_id().into(),
            action: action.into(),
            performed_by: req.performed_by.clone(),
            reason: req.reason.clone(),
            created_at: Utc::now(),
        };
        repo.create_audit(&audit).await?;
        Ok(())
    }

    /// Hoàn tác thay đổi gần nhất của module tại phạm vi chỉ định.
    ///
    /// # Errors
    /// Trả về `RegistryError` nếu không có lịch sử hoặc quyền hạn không đủ.
    pub async fn undo_last_change(
        &self,
        module_id: &str,
        scope: &RegistryScope,
        performed_by: &str,
        actor_role: RegistryRole,
    ) -> Result<(), RegistryError> {
        if !actor_role.can_manage_scope(scope) {
            return Err(RegistryError::PermissionDenied {
                required_role: "AppropriateRole".into(),
                actual_role: actor_role.as_str().into(),
            });
        }

        let repo = self.repo();
        let mid = FeatureModuleId::new(module_id);
        let audits = repo.list_audits(&mid).await?;

        let scope_kind = scope.scope_kind();
        let scope_id = scope.scope_id();
        let relevant: Vec<_> = audits
            .into_iter()
            .filter(|a| {
                a.scope_kind == scope_kind && a.scope_id == scope_id && a.action != "rollback"
            })
            .collect();

        let target_enabled = if relevant.len() < 2 {
            false
        } else {
            let prev = relevant
                .get(relevant.len().saturating_sub(2))
                .ok_or_else(|| RegistryError::NoAuditHistoryToUndo(module_id.to_string()))?;
            prev.action == "enable" || prev.action == "configure"
        };

        let setting = FeatureSetting {
            module_id: mid.clone(),
            scope_kind: scope_kind.into(),
            scope_id: scope_id.into(),
            enabled: target_enabled,
            locked: false,
            config: "{}".into(),
            changed_by: performed_by.into(),
            changed_at: Utc::now(),
        };
        repo.upsert_setting(&setting).await?;

        let undo_audit = FeatureAudit {
            id: FeatureAuditId::new(),
            module_id: mid,
            scope_kind: scope_kind.into(),
            scope_id: scope_id.into(),
            action: "rollback".into(),
            performed_by: performed_by.into(),
            reason: "Hoàn tác thay đổi gần nhất".into(),
            created_at: Utc::now(),
        };
        repo.create_audit(&undo_audit).await?;
        Ok(())
    }

    /// Lấy danh sách lịch sử audit của một module.
    ///
    /// # Errors
    /// Trả về `RegistryError` nếu truy vấn thất bại.
    pub async fn list_audits(&self, module_id: &str) -> Result<Vec<FeatureAudit>, RegistryError> {
        let mid = FeatureModuleId::new(module_id);
        let audits = self.repo().list_audits(&mid).await?;
        Ok(audits)
    }
}

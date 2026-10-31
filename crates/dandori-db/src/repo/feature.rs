//! Repository cho 4 bảng tính năng (`feature_module`, `feature_setting`, `feature_pack`, `feature_audit`).

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::error::DbError;
use dandori_types::{
    FeatureAudit, FeatureAuditId, FeatureModule, FeatureModuleId, FeaturePack, FeaturePackId,
    FeatureSetting,
};

/// Thao tác dữ liệu với 4 bảng tính năng của Dandori.
#[derive(Debug, Clone, Copy)]
pub struct FeatureRepo<'a> {
    pool: &'a SqlitePool,
}

impl<'a> FeatureRepo<'a> {
    /// Tạo mới một `FeatureRepo`.
    #[must_use]
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    // --- 1. feature_module ---

    /// Thêm hoặc cập nhật khai báo module tính năng.
    ///
    /// # Errors
    /// Trả về `DbError` nếu không thể lưu dữ liệu.
    pub async fn upsert_module(&self, m: &FeatureModule) -> Result<(), DbError> {
        let id_str = m.id.as_str();
        let requires_json = serde_json::to_string(&m.requires).map_err(|e| {
            DbError::ConstraintViolation(format!("Requires JSON không hợp lệ: {e}"))
        })?;
        let scopes_json = serde_json::to_string(&m.scopes)
            .map_err(|e| DbError::ConstraintViolation(format!("Scopes JSON không hợp lệ: {e}")))?;
        let core_int = if m.core { 1 } else { 0 };

        sqlx::query!(
            r#"
            INSERT INTO feature_module (
                id, group_name, name, description, version,
                requires, scopes, risk, effort, core
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                group_name = excluded.group_name,
                name = excluded.name,
                description = excluded.description,
                version = excluded.version,
                requires = excluded.requires,
                scopes = excluded.scopes,
                risk = excluded.risk,
                effort = excluded.effort,
                core = excluded.core
            "#,
            id_str,
            m.group_name,
            m.name,
            m.description,
            m.version,
            requires_json,
            scopes_json,
            m.risk,
            m.effort,
            core_int
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Lấy thông tin module tính năng theo ID.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn get_module(&self, id: &FeatureModuleId) -> Result<Option<FeatureModule>, DbError> {
        let id_str = id.as_str();

        let row = sqlx::query!(
            r#"
            SELECT
                id, group_name, name, description, version,
                requires, scopes, risk, effort, core
            FROM feature_module
            WHERE id = ?
            "#,
            id_str
        )
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| {
            let requires: Vec<String> = serde_json::from_str(&r.requires)
                .map_err(|e| DbError::ConstraintViolation(format!("Requires JSON lỗi: {e}")))?;
            let scopes: Vec<String> = serde_json::from_str(&r.scopes)
                .map_err(|e| DbError::ConstraintViolation(format!("Scopes JSON lỗi: {e}")))?;

            Ok(FeatureModule {
                id: FeatureModuleId::new(r.id),
                group_name: r.group_name,
                name: r.name,
                description: r.description,
                version: r.version,
                requires,
                scopes,
                risk: r.risk,
                effort: r.effort,
                core: r.core == 1,
            })
        })
        .transpose()
    }

    /// Danh sách toàn bộ các module tính năng đã đăng ký.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn list_modules(&self) -> Result<Vec<FeatureModule>, DbError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, group_name, name, description, version,
                requires, scopes, risk, effort, core
            FROM feature_module
            ORDER BY group_name ASC, id ASC
            "#
        )
        .fetch_all(self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            let requires: Vec<String> = serde_json::from_str(&r.requires)
                .map_err(|e| DbError::ConstraintViolation(format!("Requires JSON lỗi: {e}")))?;
            let scopes: Vec<String> = serde_json::from_str(&r.scopes)
                .map_err(|e| DbError::ConstraintViolation(format!("Scopes JSON lỗi: {e}")))?;

            list.push(FeatureModule {
                id: FeatureModuleId::new(r.id),
                group_name: r.group_name,
                name: r.name,
                description: r.description,
                version: r.version,
                requires,
                scopes,
                risk: r.risk,
                effort: r.effort,
                core: r.core == 1,
            });
        }

        Ok(list)
    }

    // --- 2. feature_setting ---

    /// Thêm hoặc cập nhật cấu hình bật/tắt của module theo phạm vi.
    ///
    /// # Errors
    /// Trả về `DbError` nếu module không tồn tại hoặc lỗi ghi dữ liệu.
    pub async fn upsert_setting(&self, s: &FeatureSetting) -> Result<(), DbError> {
        let mod_str = s.module_id.as_str();
        let enabled_int = if s.enabled { 1 } else { 0 };
        let locked_int = if s.locked { 1 } else { 0 };
        let changed_at_str = s.changed_at.to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO feature_setting (
                module_id, scope_kind, scope_id, enabled, locked,
                config, changed_by, changed_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(module_id, scope_kind, scope_id) DO UPDATE SET
                enabled = excluded.enabled,
                locked = excluded.locked,
                config = excluded.config,
                changed_by = excluded.changed_by,
                changed_at = excluded.changed_at
            "#,
            mod_str,
            s.scope_kind,
            s.scope_id,
            enabled_int,
            locked_int,
            s.config,
            s.changed_by,
            changed_at_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Lấy cấu hình của module tại một phạm vi cụ thể.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn get_setting(
        &self,
        module_id: &FeatureModuleId,
        scope_kind: &str,
        scope_id: &str,
    ) -> Result<Option<FeatureSetting>, DbError> {
        let mod_str = module_id.as_str();

        let row = sqlx::query!(
            r#"
            SELECT
                module_id, scope_kind, scope_id, enabled, locked,
                config, changed_by, changed_at
            FROM feature_setting
            WHERE module_id = ? AND scope_kind = ? AND scope_id = ?
            "#,
            mod_str,
            scope_kind,
            scope_id
        )
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| {
            let changed_at = DateTime::parse_from_rfc3339(&r.changed_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&Utc);

            Ok(FeatureSetting {
                module_id: FeatureModuleId::new(r.module_id),
                scope_kind: r.scope_kind,
                scope_id: r.scope_id,
                enabled: r.enabled == 1,
                locked: r.locked == 1,
                config: r.config,
                changed_by: r.changed_by,
                changed_at,
            })
        })
        .transpose()
    }

    /// Lấy toàn bộ cấu hình theo các phạm vi của một module.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn list_settings(
        &self,
        module_id: &FeatureModuleId,
    ) -> Result<Vec<FeatureSetting>, DbError> {
        let mod_str = module_id.as_str();

        let rows = sqlx::query!(
            r#"
            SELECT
                module_id, scope_kind, scope_id, enabled, locked,
                config, changed_by, changed_at
            FROM feature_setting
            WHERE module_id = ?
            "#,
            mod_str
        )
        .fetch_all(self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            let changed_at = DateTime::parse_from_rfc3339(&r.changed_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&Utc);

            list.push(FeatureSetting {
                module_id: FeatureModuleId::new(r.module_id),
                scope_kind: r.scope_kind,
                scope_id: r.scope_id,
                enabled: r.enabled == 1,
                locked: r.locked == 1,
                config: r.config,
                changed_by: r.changed_by,
                changed_at,
            });
        }

        Ok(list)
    }

    // --- 3. feature_pack ---

    /// Thêm hoặc cập nhật gói mẫu tính năng.
    ///
    /// # Errors
    /// Trả về `DbError` nếu không thể lưu dữ liệu.
    pub async fn upsert_pack(&self, p: &FeaturePack) -> Result<(), DbError> {
        let id_str = p.id.as_str();
        let modules_json = serde_json::to_string(&p.modules)
            .map_err(|e| DbError::ConstraintViolation(format!("Modules JSON lỗi: {e}")))?;

        sqlx::query!(
            r#"
            INSERT INTO feature_pack (id, name, modules)
            VALUES (?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                modules = excluded.modules
            "#,
            id_str,
            p.name,
            modules_json
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Lấy thông tin gói mẫu tính năng theo ID.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn get_pack(&self, id: &FeaturePackId) -> Result<Option<FeaturePack>, DbError> {
        let id_str = id.as_str();

        let row = sqlx::query!(
            r#"
            SELECT id, name, modules
            FROM feature_pack
            WHERE id = ?
            "#,
            id_str
        )
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| {
            let modules: Vec<String> = serde_json::from_str(&r.modules)
                .map_err(|e| DbError::ConstraintViolation(format!("Modules JSON lỗi: {e}")))?;

            Ok(FeaturePack {
                id: FeaturePackId::new(r.id),
                name: r.name,
                modules,
            })
        })
        .transpose()
    }

    /// Danh sách toàn bộ các gói mẫu tính năng có trong hệ thống.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn list_packs(&self) -> Result<Vec<FeaturePack>, DbError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, name, modules
            FROM feature_pack
            ORDER BY id ASC
            "#
        )
        .fetch_all(self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            let modules: Vec<String> = serde_json::from_str(&r.modules)
                .map_err(|e| DbError::ConstraintViolation(format!("Modules JSON lỗi: {e}")))?;

            list.push(FeaturePack {
                id: FeaturePackId::new(r.id),
                name: r.name,
                modules,
            });
        }

        Ok(list)
    }

    // --- 4. feature_audit ---

    /// Ghi nhận một sự kiện nhật ký kiểm toán thay đổi trạng thái tính năng.
    ///
    /// # Errors
    /// Trả về `DbError` nếu không thể lưu dữ liệu.
    pub async fn create_audit(&self, a: &FeatureAudit) -> Result<(), DbError> {
        let id_str = a.id.to_string();
        let mod_str = a.module_id.as_str();
        let created_at_str = a.created_at.to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO feature_audit (
                id, module_id, scope_kind, scope_id,
                action, performed_by, reason, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id_str,
            mod_str,
            a.scope_kind,
            a.scope_id,
            a.action,
            a.performed_by,
            a.reason,
            created_at_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Lấy danh sách nhật ký kiểm toán của một module tính năng theo thứ tự thời gian.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn list_audits(
        &self,
        module_id: &FeatureModuleId,
    ) -> Result<Vec<FeatureAudit>, DbError> {
        let mod_str = module_id.as_str();

        let rows = sqlx::query!(
            r#"
            SELECT
                id, module_id, scope_kind, scope_id,
                action, performed_by, reason, created_at
            FROM feature_audit
            WHERE module_id = ?
            ORDER BY created_at ASC
            "#,
            mod_str
        )
        .fetch_all(self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            let audit_id = FeatureAuditId::from_str(&r.id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let created_at = DateTime::parse_from_rfc3339(&r.created_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&Utc);

            list.push(FeatureAudit {
                id: audit_id,
                module_id: FeatureModuleId::new(r.module_id),
                scope_kind: r.scope_kind,
                scope_id: r.scope_id,
                action: r.action,
                performed_by: r.performed_by,
                reason: r.reason,
                created_at,
            });
        }

        Ok(list)
    }

    /// Lấy danh sách kiểm toán gần nhất trên toàn hệ thống.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn list_all_audits(&self, limit: i64) -> Result<Vec<FeatureAudit>, DbError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, module_id, scope_kind, scope_id,
                action, performed_by, reason, created_at
            FROM feature_audit
            ORDER BY created_at DESC
            LIMIT ?
            "#,
            limit
        )
        .fetch_all(self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            let audit_id = FeatureAuditId::from_str(&r.id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let created_at = DateTime::parse_from_rfc3339(&r.created_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&Utc);

            list.push(FeatureAudit {
                id: audit_id,
                module_id: FeatureModuleId::new(r.module_id),
                scope_kind: r.scope_kind,
                scope_id: r.scope_id,
                action: r.action,
                performed_by: r.performed_by,
                reason: r.reason,
                created_at,
            });
        }

        Ok(list)
    }
}

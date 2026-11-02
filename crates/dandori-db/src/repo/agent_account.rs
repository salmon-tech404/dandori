//! Repository cho thực thể `AgentAccount`.

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::error::DbError;
use dandori_types::{AgentAccount, AgentAccountId, SecretId};

struct AgentAccountRow {
    id: String,
    provider: String,
    label: String,
    home_dir: String,
    auth_type: String,
    secret_id: Option<String>,
    status: String,
    enabled: i64,
    priority: i64,
    exhausted_until: Option<String>,
    last_checked_at: Option<String>,
}

impl TryFrom<AgentAccountRow> for AgentAccount {
    type Error = DbError;

    fn try_from(r: AgentAccountRow) -> Result<Self, Self::Error> {
        let acc_id = AgentAccountId::from_str(&r.id)
            .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
        let secret_id = r
            .secret_id
            .map(|s| {
                SecretId::from_str(&s)
                    .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))
            })
            .transpose()?;
        let exhausted_until = r
            .exhausted_until
            .map(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|t| t.with_timezone(&Utc))
                    .map_err(|e| {
                        DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}"))
                    })
            })
            .transpose()?;
        let last_checked_at = r
            .last_checked_at
            .map(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|t| t.with_timezone(&Utc))
                    .map_err(|e| {
                        DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}"))
                    })
            })
            .transpose()?;

        Ok(Self {
            id: acc_id,
            provider: r.provider,
            label: r.label,
            home_dir: r.home_dir,
            auth_type: r.auth_type,
            secret_id,
            status: r.status,
            enabled: r.enabled == 1,
            priority: r.priority,
            exhausted_until,
            last_checked_at,
        })
    }
}

/// Thao tác dữ liệu với bảng `agent_account`.
#[derive(Debug, Clone, Copy)]
pub struct AgentAccountRepo<'a> {
    pool: &'a SqlitePool,
}

impl<'a> AgentAccountRepo<'a> {
    /// Tạo mới một `AgentAccountRepo`.
    #[must_use]
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Thêm tài khoản agent mới.
    ///
    /// # Errors
    /// Trả về `DbError` nếu không thể lưu dữ liệu hoặc secret tham chiếu không tồn tại.
    pub async fn create_account(&self, acc: &AgentAccount) -> Result<(), DbError> {
        let id_str = acc.id.to_string();
        let secret_str = acc.secret_id.map(|s| s.to_string());
        let enabled_int = if acc.enabled { 1 } else { 0 };
        let exhausted_str = acc.exhausted_until.map(|t| t.to_rfc3339());
        let last_checked_str = acc.last_checked_at.map(|t| t.to_rfc3339());

        sqlx::query(
            r#"
            INSERT INTO agent_account (
                id, provider, label, home_dir, auth_type, secret_id,
                status, enabled, priority, exhausted_until, last_checked_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id_str)
        .bind(&acc.provider)
        .bind(&acc.label)
        .bind(&acc.home_dir)
        .bind(&acc.auth_type)
        .bind(secret_str)
        .bind(&acc.status)
        .bind(enabled_int)
        .bind(acc.priority)
        .bind(exhausted_str)
        .bind(last_checked_str)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Lấy tài khoản agent theo ID.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn get_account(&self, id: &AgentAccountId) -> Result<Option<AgentAccount>, DbError> {
        let id_str = id.to_string();

        let row = sqlx::query(
            r#"
            SELECT
                id, provider, label, home_dir, auth_type, secret_id,
                status, enabled, priority, exhausted_until, last_checked_at
            FROM agent_account
            WHERE id = ?
            "#,
        )
        .bind(&id_str)
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| {
            let r_id: String = sqlx::Row::try_get(&r, "id")?;
            let r_provider: String = sqlx::Row::try_get(&r, "provider")?;
            let r_label: String = sqlx::Row::try_get(&r, "label")?;
            let r_home_dir: String = sqlx::Row::try_get(&r, "home_dir")?;
            let r_auth_type: String = sqlx::Row::try_get(&r, "auth_type")?;
            let r_secret_id: Option<String> = sqlx::Row::try_get(&r, "secret_id")?;
            let r_status: String = sqlx::Row::try_get(&r, "status")?;
            let r_enabled: i64 = sqlx::Row::try_get(&r, "enabled")?;
            let r_priority: i64 = sqlx::Row::try_get(&r, "priority")?;
            let r_exhausted_until: Option<String> = sqlx::Row::try_get(&r, "exhausted_until")?;
            let r_last_checked_at: Option<String> = sqlx::Row::try_get(&r, "last_checked_at")?;

            AgentAccount::try_from(AgentAccountRow {
                id: r_id,
                provider: r_provider,
                label: r_label,
                home_dir: r_home_dir,
                auth_type: r_auth_type,
                secret_id: r_secret_id,
                status: r_status,
                enabled: r_enabled,
                priority: r_priority,
                exhausted_until: r_exhausted_until,
                last_checked_at: r_last_checked_at,
            })
        })
        .transpose()
    }

    /// Danh sách toàn bộ tài khoản agent kết nối.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn list_accounts(&self) -> Result<Vec<AgentAccount>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, provider, label, home_dir, auth_type, secret_id,
                status, enabled, priority, exhausted_until, last_checked_at
            FROM agent_account
            ORDER BY priority ASC, id ASC
            "#,
        )
        .fetch_all(self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            let r_id: String = sqlx::Row::try_get(&r, "id")?;
            let r_provider: String = sqlx::Row::try_get(&r, "provider")?;
            let r_label: String = sqlx::Row::try_get(&r, "label")?;
            let r_home_dir: String = sqlx::Row::try_get(&r, "home_dir")?;
            let r_auth_type: String = sqlx::Row::try_get(&r, "auth_type")?;
            let r_secret_id: Option<String> = sqlx::Row::try_get(&r, "secret_id")?;
            let r_status: String = sqlx::Row::try_get(&r, "status")?;
            let r_enabled: i64 = sqlx::Row::try_get(&r, "enabled")?;
            let r_priority: i64 = sqlx::Row::try_get(&r, "priority")?;
            let r_exhausted_until: Option<String> = sqlx::Row::try_get(&r, "exhausted_until")?;
            let r_last_checked_at: Option<String> = sqlx::Row::try_get(&r, "last_checked_at")?;

            list.push(AgentAccount::try_from(AgentAccountRow {
                id: r_id,
                provider: r_provider,
                label: r_label,
                home_dir: r_home_dir,
                auth_type: r_auth_type,
                secret_id: r_secret_id,
                status: r_status,
                enabled: r_enabled,
                priority: r_priority,
                exhausted_until: r_exhausted_until,
                last_checked_at: r_last_checked_at,
            })?);
        }

        Ok(list)
    }

    /// Tìm tài khoản khả dụng đầu tiên thỏa mãn tiêu chí:
    /// - `enabled = true`
    /// - `status = "ready"`
    /// - `exhausted_until` không tồn tại hoặc đã qua thời điểm hiện tại
    /// - Không thuộc `exclude_ids`
    /// - Nếu có `provider`: khớp nhà cung cấp
    /// - Sắp xếp theo `priority` tăng dần, rồi đến `id`.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn find_available_account(
        &self,
        provider: Option<&str>,
        exclude_ids: &[AgentAccountId],
    ) -> Result<Option<AgentAccount>, DbError> {
        let all = self.list_accounts().await?;
        let now = Utc::now();
        let found = all.into_iter().find(|acc| {
            if !acc.enabled || acc.status != "ready" {
                return false;
            }
            if let Some(exhausted) = acc.exhausted_until
                && exhausted > now
            {
                return false;
            }
            if let Some(p) = provider
                && !acc.provider.eq_ignore_ascii_case(p)
            {
                return false;
            }
            if exclude_ids.contains(&acc.id) {
                return false;
            }
            true
        });

        Ok(found)
    }

    /// Cập nhật thời điểm hết hạn mức `exhausted_until`.
    ///
    /// # Errors
    /// Trả về `DbError` nếu không tìm thấy tài khoản hoặc cập nhật thất bại.
    pub async fn update_exhausted_until(
        &self,
        id: &AgentAccountId,
        exhausted_until: Option<DateTime<Utc>>,
    ) -> Result<(), DbError> {
        let id_str = id.to_string();
        let exh_str = exhausted_until.map(|t| t.to_rfc3339());

        let result = sqlx::query(
            r#"
            UPDATE agent_account
            SET exhausted_until = ?
            WHERE id = ?
            "#,
        )
        .bind(exh_str)
        .bind(&id_str)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound {
                entity: "agent_account",
                id: id_str,
            });
        }

        Ok(())
    }

    /// Cập nhật trạng thái và thời gian kiểm tra của tài khoản.
    ///
    /// # Errors
    /// Trả về `DbError` nếu không tìm thấy tài khoản hoặc cập nhật thất bại.
    pub async fn update_account_status(
        &self,
        id: &AgentAccountId,
        status: &str,
        last_checked_at: Option<DateTime<Utc>>,
    ) -> Result<(), DbError> {
        let id_str = id.to_string();
        let last_checked_str = last_checked_at.map(|t| t.to_rfc3339());

        let result = sqlx::query(
            r#"
            UPDATE agent_account
            SET status = ?, last_checked_at = ?
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(last_checked_str)
        .bind(&id_str)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound {
                entity: "agent_account",
                id: id_str,
            });
        }

        Ok(())
    }

    /// Xóa tài khoản agent theo ID.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn delete_account(&self, id: &AgentAccountId) -> Result<(), DbError> {
        let id_str = id.to_string();

        sqlx::query(
            r#"
            DELETE FROM agent_account
            WHERE id = ?
            "#,
        )
        .bind(&id_str)
        .execute(self.pool)
        .await?;

        Ok(())
    }
}

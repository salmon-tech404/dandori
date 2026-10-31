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

        sqlx::query!(
            r#"
            INSERT INTO agent_account (
                id, provider, label, home_dir, auth_type, secret_id,
                status, enabled, priority, exhausted_until, last_checked_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id_str,
            acc.provider,
            acc.label,
            acc.home_dir,
            acc.auth_type,
            secret_str,
            acc.status,
            enabled_int,
            acc.priority,
            exhausted_str,
            last_checked_str
        )
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

        let row = sqlx::query!(
            r#"
            SELECT
                id, provider, label, home_dir, auth_type, secret_id,
                status, enabled, priority, exhausted_until, last_checked_at
            FROM agent_account
            WHERE id = ?
            "#,
            id_str
        )
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| {
            AgentAccount::try_from(AgentAccountRow {
                id: r.id,
                provider: r.provider,
                label: r.label,
                home_dir: r.home_dir,
                auth_type: r.auth_type,
                secret_id: r.secret_id,
                status: r.status,
                enabled: r.enabled,
                priority: r.priority,
                exhausted_until: r.exhausted_until,
                last_checked_at: r.last_checked_at,
            })
        })
        .transpose()
    }

    /// Danh sách toàn bộ tài khoản agent kết nối.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn list_accounts(&self) -> Result<Vec<AgentAccount>, DbError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, provider, label, home_dir, auth_type, secret_id,
                status, enabled, priority, exhausted_until, last_checked_at
            FROM agent_account
            ORDER BY priority ASC
            "#
        )
        .fetch_all(self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            list.push(AgentAccount::try_from(AgentAccountRow {
                id: r.id,
                provider: r.provider,
                label: r.label,
                home_dir: r.home_dir,
                auth_type: r.auth_type,
                secret_id: r.secret_id,
                status: r.status,
                enabled: r.enabled,
                priority: r.priority,
                exhausted_until: r.exhausted_until,
                last_checked_at: r.last_checked_at,
            })?);
        }

        Ok(list)
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

        let result = sqlx::query!(
            r#"
            UPDATE agent_account
            SET status = ?, last_checked_at = ?
            WHERE id = ?
            "#,
            status,
            last_checked_str,
            id_str
        )
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

        sqlx::query!(
            r#"
            DELETE FROM agent_account
            WHERE id = ?
            "#,
            id_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }
}

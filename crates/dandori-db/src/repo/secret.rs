//! Repository cho thực thể `SecretRecord`.

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::error::DbError;
use dandori_types::{SecretId, SecretRecord};

/// Thao tác dữ liệu với bảng `secret`.
#[derive(Debug, Clone, Copy)]
pub struct SecretRepo<'a> {
    pool: &'a SqlitePool,
}

impl<'a> SecretRepo<'a> {
    /// Tạo mới một `SecretRepo`.
    #[must_use]
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Lưu trữ bản ghi khóa bí mật đã mã hóa.
    ///
    /// # Errors
    /// Trả về `DbError` nếu không thể lưu dữ liệu.
    pub async fn create_secret(&self, secret: &SecretRecord) -> Result<(), DbError> {
        let id_str = secret.id.to_string();
        let created_at_str = secret.created_at.to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO secret (id, label, nonce, ciphertext, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
            id_str,
            secret.label,
            secret.nonce,
            secret.ciphertext,
            created_at_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Đọc bản ghi khóa bí mật đã mã hóa theo ID.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn get_secret(&self, id: &SecretId) -> Result<Option<SecretRecord>, DbError> {
        let id_str = id.to_string();

        let row = sqlx::query!(
            r#"
            SELECT id, label, nonce, ciphertext, created_at
            FROM secret
            WHERE id = ?
            "#,
            id_str
        )
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| {
            let secret_id = SecretId::from_str(&r.id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let created_at = DateTime::parse_from_rfc3339(&r.created_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&Utc);

            Ok(SecretRecord {
                id: secret_id,
                label: r.label,
                nonce: r.nonce,
                ciphertext: r.ciphertext,
                created_at,
            })
        })
        .transpose()
    }

    /// Xóa bản ghi khóa bí mật. Nếu có bảng con tham chiếu (`agent_account`), khóa ngoại RESTRICT sẽ chặn lại.
    ///
    /// # Errors
    /// Trả về `DbError` nếu có ràng buộc khóa ngoại vi phạm hoặc lỗi truy vấn.
    pub async fn delete_secret(&self, id: &SecretId) -> Result<(), DbError> {
        let id_str = id.to_string();

        sqlx::query!(
            r#"
            DELETE FROM secret
            WHERE id = ?
            "#,
            id_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Liệt kê các khóa bí mật hiện có, chỉ trả về ID và nhãn (không trả ciphertext).
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn cơ sở dữ liệu thất bại.
    pub async fn list_secrets(&self) -> Result<Vec<(SecretId, String)>, DbError> {
        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT id, label FROM secret ORDER BY created_at DESC")
                .fetch_all(self.pool)
                .await?;

        let mut list = Vec::with_capacity(rows.len());
        for (id_str, label) in rows {
            let secret_id = SecretId::from_str(&id_str)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            list.push((secret_id, label));
        }
        Ok(list)
    }
}

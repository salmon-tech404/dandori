//! Repository cho thực thể `Session`.

use chrono::DateTime;
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::error::DbError;
use dandori_types::{Session, SessionId, UserId};

/// Thao tác dữ liệu với bảng `session`.
#[derive(Debug, Clone, Copy)]
pub struct SessionRepo<'a> {
    pool: &'a SqlitePool,
}

impl<'a> SessionRepo<'a> {
    /// Tạo mới một `SessionRepo`.
    #[must_use]
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Thêm phiên đăng nhập mới.
    ///
    /// # Errors
    /// Trả về `DbError` nếu khóa ngoại user_id không tồn tại hoặc lỗi ghi dữ liệu.
    pub async fn create_session(&self, session: &Session) -> Result<(), DbError> {
        let id_str = session.id.to_string();
        let user_id_str = session.user_id.to_string();
        let expires_at_str = session.expires_at.to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO session (id, user_id, expires_at, token_hash)
            VALUES (?, ?, ?, ?)
            "#,
            id_str,
            user_id_str,
            expires_at_str,
            session.token_hash
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Lấy phiên đăng nhập theo ID.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn get_session(&self, id: &SessionId) -> Result<Option<Session>, DbError> {
        let id_str = id.to_string();

        let row = sqlx::query!(
            r#"
            SELECT id, user_id, expires_at, token_hash
            FROM session
            WHERE id = ?
            "#,
            id_str
        )
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| {
            let session_id = SessionId::from_str(&r.id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let user_id = UserId::from_str(&r.user_id).map_err(|e| {
                DbError::ConstraintViolation(format!("User UUID không hợp lệ: {e}"))
            })?;
            let expires_at = DateTime::parse_from_rfc3339(&r.expires_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&chrono::Utc);

            Ok(Session {
                id: session_id,
                user_id,
                expires_at,
                token_hash: r.token_hash,
            })
        })
        .transpose()
    }

    /// Xóa phiên đăng nhập theo ID.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn delete_session(&self, id: &SessionId) -> Result<(), DbError> {
        let id_str = id.to_string();

        sqlx::query!(
            r#"
            DELETE FROM session
            WHERE id = ?
            "#,
            id_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }
}

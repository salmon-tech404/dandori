//! Repository cho thực thể `User`.

use chrono::DateTime;
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::error::DbError;
use dandori_types::{User, UserId};

/// Thao tác dữ liệu với bảng `user`.
#[derive(Debug, Clone, Copy)]
pub struct UserRepo<'a> {
    pool: &'a SqlitePool,
}

impl<'a> UserRepo<'a> {
    /// Tạo mới một `UserRepo`.
    #[must_use]
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Thêm người dùng mới.
    ///
    /// # Errors
    /// Trả về `DbError` nếu tên đăng nhập đã tồn tại hoặc thao tác cơ sở dữ liệu thất bại.
    pub async fn create_user(&self, user: &User) -> Result<(), DbError> {
        let id_str = user.id.to_string();
        let created_at_str = user.created_at.to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO user (id, username, password_hash, system_role, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
            id_str,
            user.username,
            user.password_hash,
            user.system_role,
            created_at_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Lấy thông tin người dùng theo ID.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn get_user_by_id(&self, id: &UserId) -> Result<Option<User>, DbError> {
        let id_str = id.to_string();

        let row = sqlx::query!(
            r#"
            SELECT id, username, password_hash, system_role, created_at
            FROM user
            WHERE id = ?
            "#,
            id_str
        )
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| {
            let user_id = UserId::from_str(&r.id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let created_at = DateTime::parse_from_rfc3339(&r.created_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&chrono::Utc);

            Ok(User {
                id: user_id,
                username: r.username,
                password_hash: r.password_hash,
                system_role: r.system_role,
                created_at,
            })
        })
        .transpose()
    }

    /// Lấy thông tin người dùng theo tên đăng nhập (username).
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, DbError> {
        let row = sqlx::query!(
            r#"
            SELECT id, username, password_hash, system_role, created_at
            FROM user
            WHERE username = ?
            "#,
            username
        )
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| {
            let user_id = UserId::from_str(&r.id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let created_at = DateTime::parse_from_rfc3339(&r.created_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&chrono::Utc);

            Ok(User {
                id: user_id,
                username: r.username,
                password_hash: r.password_hash,
                system_role: r.system_role,
                created_at,
            })
        })
        .transpose()
    }

    /// Cập nhật vai trò hệ thống của người dùng.
    ///
    /// # Errors
    /// Trả về `DbError` nếu người dùng không tồn tại hoặc truy vấn lỗi.
    pub async fn update_user_role(&self, id: &UserId, role: &str) -> Result<(), DbError> {
        let id_str = id.to_string();

        let result = sqlx::query!(
            r#"
            UPDATE user
            SET system_role = ?
            WHERE id = ?
            "#,
            role,
            id_str
        )
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound {
                entity: "user",
                id: id_str,
            });
        }

        Ok(())
    }

    /// Xóa người dùng theo ID.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn delete_user(&self, id: &UserId) -> Result<(), DbError> {
        let id_str = id.to_string();

        sqlx::query!(
            r#"
            DELETE FROM user
            WHERE id = ?
            "#,
            id_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }
}

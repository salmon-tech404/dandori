//! Repository cho thực thể `Workspace` và `WorkspaceMember`.

use chrono::DateTime;
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::error::DbError;
use dandori_types::{UserId, Workspace, WorkspaceId, WorkspaceMember};

/// Thao tác dữ liệu với bảng `workspace` và `workspace_member`.
#[derive(Debug, Clone, Copy)]
pub struct WorkspaceRepo<'a> {
    pool: &'a SqlitePool,
}

impl<'a> WorkspaceRepo<'a> {
    /// Tạo mới một `WorkspaceRepo`.
    #[must_use]
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Tạo không gian làm việc mới.
    ///
    /// # Errors
    /// Trả về `DbError` nếu không thể ghi vào cơ sở dữ liệu.
    pub async fn create_workspace(&self, ws: &Workspace) -> Result<(), DbError> {
        let id_str = ws.id.to_string();
        let created_at_str = ws.created_at.to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO workspace (id, name, created_at)
            VALUES (?, ?, ?)
            "#,
            id_str,
            ws.name,
            created_at_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Lấy thông tin không gian làm việc theo ID.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn get_workspace(&self, id: &WorkspaceId) -> Result<Option<Workspace>, DbError> {
        let id_str = id.to_string();

        let row = sqlx::query!(
            r#"
            SELECT id, name, created_at
            FROM workspace
            WHERE id = ?
            "#,
            id_str
        )
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| {
            let ws_id = WorkspaceId::from_str(&r.id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let created_at = DateTime::parse_from_rfc3339(&r.created_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&chrono::Utc);

            Ok(Workspace {
                id: ws_id,
                name: r.name,
                created_at,
            })
        })
        .transpose()
    }

    /// Cập nhật tên không gian làm việc.
    ///
    /// # Errors
    /// Trả về `DbError` nếu workspace không tồn tại hoặc lỗi cập nhật.
    pub async fn update_workspace_name(&self, id: &WorkspaceId, name: &str) -> Result<(), DbError> {
        let id_str = id.to_string();

        let result = sqlx::query!(
            r#"
            UPDATE workspace
            SET name = ?
            WHERE id = ?
            "#,
            name,
            id_str
        )
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound {
                entity: "workspace",
                id: id_str,
            });
        }

        Ok(())
    }

    /// Xóa không gian làm việc theo ID.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn delete_workspace(&self, id: &WorkspaceId) -> Result<(), DbError> {
        let id_str = id.to_string();

        sqlx::query!(
            r#"
            DELETE FROM workspace
            WHERE id = ?
            "#,
            id_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Thêm thành viên vào không gian làm việc.
    ///
    /// # Errors
    /// Trả về `DbError` nếu khóa ngoại không tồn tại hoặc lỗi ghi dữ liệu.
    pub async fn add_member(&self, member: &WorkspaceMember) -> Result<(), DbError> {
        let ws_str = member.workspace_id.to_string();
        let user_str = member.user_id.to_string();

        sqlx::query!(
            r#"
            INSERT INTO workspace_member (workspace_id, user_id, role)
            VALUES (?, ?, ?)
            ON CONFLICT(workspace_id, user_id) DO UPDATE SET role = excluded.role
            "#,
            ws_str,
            user_str,
            member.role
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Lấy thông tin thành viên của một workspace.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn get_member(
        &self,
        ws_id: &WorkspaceId,
        user_id: &UserId,
    ) -> Result<Option<WorkspaceMember>, DbError> {
        let ws_str = ws_id.to_string();
        let user_str = user_id.to_string();

        let row = sqlx::query!(
            r#"
            SELECT workspace_id, user_id, role
            FROM workspace_member
            WHERE workspace_id = ? AND user_id = ?
            "#,
            ws_str,
            user_str
        )
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| {
            let ws = WorkspaceId::from_str(&r.workspace_id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let u = UserId::from_str(&r.user_id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;

            Ok(WorkspaceMember {
                workspace_id: ws,
                user_id: u,
                role: r.role,
            })
        })
        .transpose()
    }

    /// Danh sách toàn bộ thành viên của một workspace.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn list_members(&self, ws_id: &WorkspaceId) -> Result<Vec<WorkspaceMember>, DbError> {
        let ws_str = ws_id.to_string();

        let rows = sqlx::query!(
            r#"
            SELECT workspace_id, user_id, role
            FROM workspace_member
            WHERE workspace_id = ?
            "#,
            ws_str
        )
        .fetch_all(self.pool)
        .await?;

        let mut members = Vec::with_capacity(rows.len());
        for r in rows {
            let ws = WorkspaceId::from_str(&r.workspace_id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let u = UserId::from_str(&r.user_id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;

            members.push(WorkspaceMember {
                workspace_id: ws,
                user_id: u,
                role: r.role,
            });
        }

        Ok(members)
    }

    /// Lấy danh sách các workspace mà người dùng là thành viên.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn list_user_workspaces(&self, user_id: &UserId) -> Result<Vec<Workspace>, DbError> {
        let u_str = user_id.to_string();

        let rows = sqlx::query!(
            r#"
            SELECT w.id, w.name, w.created_at
            FROM workspace w
            INNER JOIN workspace_member wm ON w.id = wm.workspace_id
            WHERE wm.user_id = ?
            ORDER BY w.created_at ASC
            "#,
            u_str
        )
        .fetch_all(self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            let ws_id = WorkspaceId::from_str(&r.id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let created_at = DateTime::parse_from_rfc3339(&r.created_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&chrono::Utc);

            list.push(Workspace {
                id: ws_id,
                name: r.name,
                created_at,
            });
        }

        Ok(list)
    }

    /// Xóa thành viên khỏi không gian làm việc.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn remove_member(
        &self,
        ws_id: &WorkspaceId,
        user_id: &UserId,
    ) -> Result<(), DbError> {
        let ws_str = ws_id.to_string();
        let user_str = user_id.to_string();

        sqlx::query!(
            r#"
            DELETE FROM workspace_member
            WHERE workspace_id = ? AND user_id = ?
            "#,
            ws_str,
            user_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }
}

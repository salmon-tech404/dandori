//! Repository cho thực thể `Project`.

use chrono::DateTime;
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::error::DbError;
use dandori_types::{Project, ProjectId, WorkspaceId};

/// Thao tác dữ liệu với bảng `project` có phạm vi theo `workspace_id` (RS-DB-002).
#[derive(Debug, Clone, Copy)]
pub struct ProjectRepo<'a> {
    pool: &'a SqlitePool,
}

impl<'a> ProjectRepo<'a> {
    /// Tạo mới một `ProjectRepo`.
    #[must_use]
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Tạo dự án mới trong không gian làm việc.
    ///
    /// # Errors
    /// Trả về `DbError` nếu không thể ghi dữ liệu hoặc workspace không tồn tại.
    pub async fn create_project(&self, project: &Project) -> Result<(), DbError> {
        let id_str = project.id.to_string();
        let ws_str = project.workspace_id.to_string();
        let created_at_str = project.created_at.to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            id_str,
            ws_str,
            project.name,
            project.repo_path,
            project.default_branch,
            created_at_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Lấy thông tin dự án theo ID, bắt buộc chỉ định `workspace_id` sở hữu (RS-DB-002).
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn get_project(
        &self,
        ws_id: &WorkspaceId,
        id: &ProjectId,
    ) -> Result<Option<Project>, DbError> {
        let id_str = id.to_string();
        let ws_str = ws_id.to_string();

        let row = sqlx::query!(
            r#"
            SELECT id, workspace_id, name, repo_path, default_branch, created_at
            FROM project
            WHERE id = ? AND workspace_id = ?
            "#,
            id_str,
            ws_str
        )
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| {
            let proj_id = ProjectId::from_str(&r.id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let workspace_id = WorkspaceId::from_str(&r.workspace_id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let created_at = DateTime::parse_from_rfc3339(&r.created_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&chrono::Utc);

            Ok(Project {
                id: proj_id,
                workspace_id,
                name: r.name,
                repo_path: r.repo_path,
                default_branch: r.default_branch,
                created_at,
            })
        })
        .transpose()
    }

    /// Lấy thông tin dự án theo ID không lọc workspace (dùng để xác định workspace sở hữu).
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn get_project_by_id(&self, id: &ProjectId) -> Result<Option<Project>, DbError> {
        let id_str = id.to_string();

        let row = sqlx::query!(
            r#"
            SELECT id, workspace_id, name, repo_path, default_branch, created_at
            FROM project
            WHERE id = ?
            "#,
            id_str
        )
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| {
            let proj_id = ProjectId::from_str(&r.id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let workspace_id = WorkspaceId::from_str(&r.workspace_id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let created_at = DateTime::parse_from_rfc3339(&r.created_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&chrono::Utc);

            Ok(Project {
                id: proj_id,
                workspace_id,
                name: r.name,
                repo_path: r.repo_path,
                default_branch: r.default_branch,
                created_at,
            })
        })
        .transpose()
    }

    /// Lấy danh sách dự án thuộc một không gian làm việc.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn list_projects(&self, ws_id: &WorkspaceId) -> Result<Vec<Project>, DbError> {
        let ws_str = ws_id.to_string();

        let rows = sqlx::query!(
            r#"
            SELECT id, workspace_id, name, repo_path, default_branch, created_at
            FROM project
            WHERE workspace_id = ?
            ORDER BY created_at ASC
            "#,
            ws_str
        )
        .fetch_all(self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            let proj_id = ProjectId::from_str(&r.id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let workspace_id = WorkspaceId::from_str(&r.workspace_id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let created_at = DateTime::parse_from_rfc3339(&r.created_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&chrono::Utc);

            list.push(Project {
                id: proj_id,
                workspace_id,
                name: r.name,
                repo_path: r.repo_path,
                default_branch: r.default_branch,
                created_at,
            });
        }

        Ok(list)
    }

    /// Cập nhật tên và nhánh mặc định của dự án, bắt buộc chỉ định `workspace_id` sở hữu (RS-DB-002).
    ///
    /// # Errors
    /// Trả về `DbError` nếu dự án không tồn tại trong workspace hoặc cập nhật thất bại.
    pub async fn update_project(
        &self,
        ws_id: &WorkspaceId,
        id: &ProjectId,
        name: &str,
        default_branch: &str,
    ) -> Result<(), DbError> {
        let id_str = id.to_string();
        let ws_str = ws_id.to_string();

        let result = sqlx::query!(
            r#"
            UPDATE project
            SET name = ?, default_branch = ?
            WHERE id = ? AND workspace_id = ?
            "#,
            name,
            default_branch,
            id_str,
            ws_str
        )
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound {
                entity: "project",
                id: id_str,
            });
        }

        Ok(())
    }

    /// Xóa dự án, bắt buộc chỉ định `workspace_id` sở hữu (RS-DB-002).
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn delete_project(&self, ws_id: &WorkspaceId, id: &ProjectId) -> Result<(), DbError> {
        let id_str = id.to_string();
        let ws_str = ws_id.to_string();

        sqlx::query!(
            r#"
            DELETE FROM project
            WHERE id = ? AND workspace_id = ?
            "#,
            id_str,
            ws_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }
}

//! Repository cho thực thể `Task`.

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::error::DbError;
use dandori_types::{ProjectId, Task, TaskId};

/// Thao tác dữ liệu với bảng `task` có phạm vi theo `project_id` (RS-DB-002).
#[derive(Debug, Clone, Copy)]
pub struct TaskRepo<'a> {
    pool: &'a SqlitePool,
}

struct RawTaskRow {
    id: String,
    project_id: String,
    title: String,
    description: String,
    status: String,
    lead_agent: String,
    worker_agent: String,
    workdir_path: Option<String>,
    created_at: String,
    updated_at: String,
}

impl TryFrom<RawTaskRow> for Task {
    type Error = DbError;

    fn try_from(r: RawTaskRow) -> Result<Self, Self::Error> {
        let task_id = TaskId::from_str(&r.id)
            .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
        let p_id = ProjectId::from_str(&r.project_id)
            .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
        let created_at = DateTime::parse_from_rfc3339(&r.created_at)
            .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&r.updated_at)
            .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
            .with_timezone(&Utc);

        Ok(Self {
            id: task_id,
            project_id: p_id,
            title: r.title,
            description: r.description,
            status: r.status,
            lead_agent: r.lead_agent,
            worker_agent: r.worker_agent,
            workdir_path: r.workdir_path,
            created_at,
            updated_at,
        })
    }
}

impl<'a> TaskRepo<'a> {
    /// Tạo mới một `TaskRepo`.
    #[must_use]
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Tạo task mới thuộc dự án.
    ///
    /// # Errors
    /// Trả về `DbError` nếu không thể ghi dữ liệu hoặc dự án không tồn tại.
    pub async fn create_task(&self, task: &Task) -> Result<(), DbError> {
        let id_str = task.id.to_string();
        let proj_str = task.project_id.to_string();
        let created_at_str = task.created_at.to_rfc3339();
        let updated_at_str = task.updated_at.to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO task (
                id, project_id, title, description, status,
                lead_agent, worker_agent, workdir_path, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id_str,
            proj_str,
            task.title,
            task.description,
            task.status,
            task.lead_agent,
            task.worker_agent,
            task.workdir_path,
            created_at_str,
            updated_at_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Lấy thông tin task theo ID, bắt buộc chỉ định `project_id` sở hữu (RS-DB-002).
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn get_task(
        &self,
        project_id: &ProjectId,
        id: &TaskId,
    ) -> Result<Option<Task>, DbError> {
        let id_str = id.to_string();
        let proj_str = project_id.to_string();

        let row = sqlx::query!(
            r#"
            SELECT
                id, project_id, title, description, status,
                lead_agent, worker_agent, workdir_path, created_at, updated_at
            FROM task
            WHERE id = ? AND project_id = ?
            "#,
            id_str,
            proj_str
        )
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| {
            let task_id = TaskId::from_str(&r.id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let p_id = ProjectId::from_str(&r.project_id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let created_at = DateTime::parse_from_rfc3339(&r.created_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&Utc);
            let updated_at = DateTime::parse_from_rfc3339(&r.updated_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&Utc);

            Ok(Task {
                id: task_id,
                project_id: p_id,
                title: r.title,
                description: r.description,
                status: r.status,
                lead_agent: r.lead_agent,
                worker_agent: r.worker_agent,
                workdir_path: r.workdir_path,
                created_at,
                updated_at,
            })
        })
        .transpose()
    }

    /// Lấy danh sách task thuộc một dự án.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn list_tasks(&self, project_id: &ProjectId) -> Result<Vec<Task>, DbError> {
        let proj_str = project_id.to_string();

        let rows = sqlx::query!(
            r#"
            SELECT
                id, project_id, title, description, status,
                lead_agent, worker_agent, workdir_path, created_at, updated_at
            FROM task
            WHERE project_id = ?
            ORDER BY created_at ASC
            "#,
            proj_str
        )
        .fetch_all(self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            list.push(
                RawTaskRow {
                    id: r.id,
                    project_id: r.project_id,
                    title: r.title,
                    description: r.description,
                    status: r.status,
                    lead_agent: r.lead_agent,
                    worker_agent: r.worker_agent,
                    workdir_path: r.workdir_path,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                }
                .try_into()?,
            );
        }

        Ok(list)
    }

    /// Lấy thông tin task theo ID không lọc project (dùng để xác định dự án và workspace sở hữu).
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn get_task_by_id(&self, id: &TaskId) -> Result<Option<Task>, DbError> {
        let id_str = id.to_string();

        let row = sqlx::query!(
            r#"
            SELECT
                id, project_id, title, description, status,
                lead_agent, worker_agent, workdir_path, created_at, updated_at
            FROM task
            WHERE id = ?
            "#,
            id_str
        )
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| {
            let task_id = TaskId::from_str(&r.id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let p_id = ProjectId::from_str(&r.project_id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let created_at = DateTime::parse_from_rfc3339(&r.created_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&Utc);
            let updated_at = DateTime::parse_from_rfc3339(&r.updated_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&Utc);

            Ok(Task {
                id: task_id,
                project_id: p_id,
                title: r.title,
                description: r.description,
                status: r.status,
                lead_agent: r.lead_agent,
                worker_agent: r.worker_agent,
                workdir_path: r.workdir_path,
                created_at,
                updated_at,
            })
        })
        .transpose()
    }

    /// Cập nhật tiêu đề và mô tả của task.
    ///
    /// # Errors
    /// Trả về `DbError` nếu task không tồn tại hoặc cập nhật thất bại.
    pub async fn update_task(
        &self,
        project_id: &ProjectId,
        id: &TaskId,
        title: &str,
        description: &str,
    ) -> Result<(), DbError> {
        let id_str = id.to_string();
        let proj_str = project_id.to_string();
        let now_str = Utc::now().to_rfc3339();

        let result = sqlx::query!(
            r#"
            UPDATE task
            SET title = ?, description = ?, updated_at = ?
            WHERE id = ? AND project_id = ?
            "#,
            title,
            description,
            now_str,
            id_str,
            proj_str
        )
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound {
                entity: "task",
                id: id_str,
            });
        }

        Ok(())
    }

    /// Cập nhật thông tin task bao gồm phân công lead và worker.
    ///
    /// # Errors
    /// Trả về `DbError` nếu task không tồn tại hoặc cập nhật thất bại.
    pub async fn update_task_full(
        &self,
        project_id: &ProjectId,
        id: &TaskId,
        title: &str,
        description: &str,
        lead_agent: &str,
        worker_agent: &str,
    ) -> Result<(), DbError> {
        let id_str = id.to_string();
        let proj_str = project_id.to_string();
        let now_str = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            UPDATE task
            SET title = ?, description = ?, lead_agent = ?, worker_agent = ?, updated_at = ?
            WHERE id = ? AND project_id = ?
            "#,
        )
        .bind(title)
        .bind(description)
        .bind(lead_agent)
        .bind(worker_agent)
        .bind(now_str)
        .bind(&id_str)
        .bind(&proj_str)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound {
                entity: "task",
                id: id_str,
            });
        }

        Ok(())
    }

    /// Lấy danh sách task phân trang kèm tổng số lượng.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn list_tasks_paginated(
        &self,
        project_id: &ProjectId,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Task>, i64), DbError> {
        let proj_str = project_id.to_string();

        let count_row = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM task
            WHERE project_id = ?
            "#,
            proj_str
        )
        .fetch_one(self.pool)
        .await?;

        let total = count_row.count;

        let rows = sqlx::query!(
            r#"
            SELECT
                id, project_id, title, description, status,
                lead_agent, worker_agent, workdir_path, created_at, updated_at
            FROM task
            WHERE project_id = ?
            ORDER BY created_at ASC
            LIMIT ? OFFSET ?
            "#,
            proj_str,
            limit,
            offset
        )
        .fetch_all(self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            list.push(
                RawTaskRow {
                    id: r.id,
                    project_id: r.project_id,
                    title: r.title,
                    description: r.description,
                    status: r.status,
                    lead_agent: r.lead_agent,
                    worker_agent: r.worker_agent,
                    workdir_path: r.workdir_path,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                }
                .try_into()?,
            );
        }

        Ok((list, total))
    }

    /// Cập nhật workdir_path của task, bắt buộc chỉ định `project_id` sở hữu (RS-DB-002).
    ///
    /// # Errors
    /// Trả về `DbError` nếu task không thuộc dự án hoặc cập nhật thất bại.
    pub async fn update_task_workdir(
        &self,
        project_id: &ProjectId,
        id: &TaskId,
        workdir_path: Option<&str>,
    ) -> Result<(), DbError> {
        let id_str = id.to_string();
        let proj_str = project_id.to_string();
        let now_str = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            UPDATE task
            SET workdir_path = ?1, updated_at = ?2
            WHERE id = ?3 AND project_id = ?4
            "#,
        )
        .bind(workdir_path)
        .bind(now_str)
        .bind(&id_str)
        .bind(proj_str)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound {
                entity: "task",
                id: id_str,
            });
        }

        Ok(())
    }

    /// Cập nhật trạng thái task, bắt buộc chỉ định `project_id` sở hữu (RS-DB-002).
    ///
    /// Chiếm task để chạy một cách nguyên tử: chỉ đổi sang `running` nếu task chưa `running`/`done`.
    /// Trả về `false` nếu đã có người chiếm (hai yêu cầu đồng thời chỉ một bên thắng).
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn try_claim_for_run(
        &self,
        project_id: &ProjectId,
        id: &TaskId,
    ) -> Result<bool, DbError> {
        let result = sqlx::query(
            "UPDATE task SET status = 'running', updated_at = ?              WHERE id = ? AND project_id = ? AND status NOT IN ('running', 'done')",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(id.to_string())
        .bind(project_id.to_string())
        .execute(self.pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }

    /// # Errors
    /// Trả về `DbError` nếu task không thuộc dự án hoặc cập nhật thất bại.
    pub async fn update_task_status(
        &self,
        project_id: &ProjectId,
        id: &TaskId,
        status: &str,
    ) -> Result<(), DbError> {
        let id_str = id.to_string();
        let proj_str = project_id.to_string();
        let now_str = Utc::now().to_rfc3339();

        let result = sqlx::query!(
            r#"
            UPDATE task
            SET status = ?, updated_at = ?
            WHERE id = ? AND project_id = ?
            "#,
            status,
            now_str,
            id_str,
            proj_str
        )
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound {
                entity: "task",
                id: id_str,
            });
        }

        Ok(())
    }

    /// Xóa task, bắt buộc chỉ định `project_id` sở hữu (RS-DB-002).
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn delete_task(&self, project_id: &ProjectId, id: &TaskId) -> Result<(), DbError> {
        let id_str = id.to_string();
        let proj_str = project_id.to_string();

        sqlx::query!(
            r#"
            DELETE FROM task
            WHERE id = ? AND project_id = ?
            "#,
            id_str,
            proj_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }
}

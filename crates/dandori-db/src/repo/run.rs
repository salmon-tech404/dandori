//! Repository cho thực thể `Run` và `RunEvent`.

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::error::DbError;
use dandori_types::{Run, RunEvent, RunEventId, RunId, TaskId};

/// Thao tác dữ liệu với bảng `run` có phạm vi theo `task_id` (RS-DB-002).
#[derive(Debug, Clone, Copy)]
pub struct RunRepo<'a> {
    pool: &'a SqlitePool,
}

impl<'a> RunRepo<'a> {
    /// Tạo mới một `RunRepo`.
    #[must_use]
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Tạo một lần chạy mới cho task.
    ///
    /// # Errors
    /// Trả về `DbError` nếu không thể ghi dữ liệu hoặc task không tồn tại.
    pub async fn create_run(&self, run: &Run) -> Result<(), DbError> {
        let id_str = run.id.to_string();
        let task_str = run.task_id.to_string();
        let started_at_str = run.started_at.to_rfc3339();
        let finished_at_str = run.finished_at.map(|t| t.to_rfc3339());

        sqlx::query!(
            r#"
            INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id_str,
            task_str,
            run.agent,
            run.account,
            run.status,
            started_at_str,
            finished_at_str,
            run.exit_code
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Lấy thông tin lần chạy theo ID, bắt buộc chỉ định `task_id` sở hữu (RS-DB-002).
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn get_run(&self, task_id: &TaskId, id: &RunId) -> Result<Option<Run>, DbError> {
        let id_str = id.to_string();
        let task_str = task_id.to_string();

        let row = sqlx::query!(
            r#"
            SELECT id, task_id, agent, account, status, started_at, finished_at, exit_code
            FROM run
            WHERE id = ? AND task_id = ?
            "#,
            id_str,
            task_str
        )
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| {
            let run_id = RunId::from_str(&r.id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let t_id = TaskId::from_str(&r.task_id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let started_at = DateTime::parse_from_rfc3339(&r.started_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&Utc);
            let finished_at = r
                .finished_at
                .map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .map(|t| t.with_timezone(&Utc))
                        .map_err(|e| {
                            DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}"))
                        })
                })
                .transpose()?;

            Ok(Run {
                id: run_id,
                task_id: t_id,
                agent: r.agent,
                account: r.account,
                status: r.status,
                started_at,
                finished_at,
                exit_code: r.exit_code,
            })
        })
        .transpose()
    }

    /// Danh sách các lần chạy của một task.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn list_runs(&self, task_id: &TaskId) -> Result<Vec<Run>, DbError> {
        let task_str = task_id.to_string();

        let rows = sqlx::query!(
            r#"
            SELECT id, task_id, agent, account, status, started_at, finished_at, exit_code
            FROM run
            WHERE task_id = ?
            ORDER BY started_at ASC
            "#,
            task_str
        )
        .fetch_all(self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            let run_id = RunId::from_str(&r.id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let t_id = TaskId::from_str(&r.task_id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let started_at = DateTime::parse_from_rfc3339(&r.started_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&Utc);
            let finished_at = r
                .finished_at
                .map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .map(|t| t.with_timezone(&Utc))
                        .map_err(|e| {
                            DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}"))
                        })
                })
                .transpose()?;

            list.push(Run {
                id: run_id,
                task_id: t_id,
                agent: r.agent,
                account: r.account,
                status: r.status,
                started_at,
                finished_at,
                exit_code: r.exit_code,
            });
        }

        Ok(list)
    }

    /// Cập nhật trạng thái và kết quả lần chạy.
    ///
    /// # Errors
    /// Trả về `DbError` nếu không tìm thấy run hoặc cập nhật thất bại.
    pub async fn update_run_status(
        &self,
        task_id: &TaskId,
        id: &RunId,
        status: &str,
        finished_at: Option<DateTime<Utc>>,
        exit_code: Option<i64>,
    ) -> Result<(), DbError> {
        let id_str = id.to_string();
        let task_str = task_id.to_string();
        let finished_at_str = finished_at.map(|t| t.to_rfc3339());

        let result = sqlx::query!(
            r#"
            UPDATE run
            SET status = ?, finished_at = ?, exit_code = ?
            WHERE id = ? AND task_id = ?
            "#,
            status,
            finished_at_str,
            exit_code,
            id_str,
            task_str
        )
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound {
                entity: "run",
                id: id_str,
            });
        }

        Ok(())
    }

    /// Xóa lần chạy.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn delete_run(&self, task_id: &TaskId, id: &RunId) -> Result<(), DbError> {
        let id_str = id.to_string();
        let task_str = task_id.to_string();

        sqlx::query!(
            r#"
            DELETE FROM run
            WHERE id = ? AND task_id = ?
            "#,
            id_str,
            task_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }
}

/// Thao tác dữ liệu với bảng `run_event`.
#[derive(Debug, Clone, Copy)]
pub struct RunEventRepo<'a> {
    pool: &'a SqlitePool,
}

impl<'a> RunEventRepo<'a> {
    /// Tạo mới một `RunEventRepo`.
    #[must_use]
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Ghi nhận sự kiện phát sinh từ lần chạy.
    ///
    /// # Errors
    /// Trả về `DbError` nếu không thể ghi dữ liệu hoặc run không tồn tại.
    pub async fn create_run_event(&self, event: &RunEvent) -> Result<(), DbError> {
        let id_str = event.id.to_string();
        let run_str = event.run_id.to_string();
        let created_at_str = event.created_at.to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO run_event (id, run_id, seq, event_type, data, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            id_str,
            run_str,
            event.seq,
            event.event_type,
            event.data,
            created_at_str
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Lấy danh sách sự kiện của một lần chạy theo thứ tự tuần tự `seq`.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn list_run_events(&self, run_id: &RunId) -> Result<Vec<RunEvent>, DbError> {
        let run_str = run_id.to_string();

        let rows = sqlx::query!(
            r#"
            SELECT id, run_id, seq, event_type, data, created_at
            FROM run_event
            WHERE run_id = ?
            ORDER BY seq ASC
            "#,
            run_str
        )
        .fetch_all(self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            let event_id = RunEventId::from_str(&r.id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let r_id = RunId::from_str(&r.run_id)
                .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
            let created_at = DateTime::parse_from_rfc3339(&r.created_at)
                .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
                .with_timezone(&Utc);

            list.push(RunEvent {
                id: event_id,
                run_id: r_id,
                seq: r.seq,
                event_type: r.event_type,
                data: r.data,
                created_at,
            });
        }

        Ok(list)
    }
}

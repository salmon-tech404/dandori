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

        sqlx::query(
            r#"
            INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id_str)
        .bind(&task_str)
        .bind(&run.agent)
        .bind(&run.account)
        .bind(&run.status)
        .bind(&started_at_str)
        .bind(finished_at_str)
        .bind(run.exit_code)
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

        let row = sqlx::query(
            r#"
            SELECT id, task_id, agent, account, status, started_at, finished_at, exit_code
            FROM run
            WHERE id = ? AND task_id = ?
            "#,
        )
        .bind(&id_str)
        .bind(&task_str)
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| self.map_run_row(&r)).transpose()
    }

    /// Lấy thông tin lần chạy theo RunId duy nhất.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn get_run_by_id(&self, id: &RunId) -> Result<Option<Run>, DbError> {
        let id_str = id.to_string();

        let row = sqlx::query(
            r#"
            SELECT id, task_id, agent, account, status, started_at, finished_at, exit_code
            FROM run
            WHERE id = ?
            "#,
        )
        .bind(&id_str)
        .fetch_optional(self.pool)
        .await?;

        row.map(|r| self.map_run_row(&r)).transpose()
    }

    fn map_run_row(&self, r: &sqlx::sqlite::SqliteRow) -> Result<Run, DbError> {
        let r_id: String = sqlx::Row::try_get(r, "id")?;
        let r_task_id: String = sqlx::Row::try_get(r, "task_id")?;
        let r_agent: String = sqlx::Row::try_get(r, "agent")?;
        let r_account: String = sqlx::Row::try_get(r, "account")?;
        let r_status: String = sqlx::Row::try_get(r, "status")?;
        let r_started_at: String = sqlx::Row::try_get(r, "started_at")?;
        let r_finished_at: Option<String> = sqlx::Row::try_get(r, "finished_at")?;
        let r_exit_code: Option<i64> = sqlx::Row::try_get(r, "exit_code")?;

        let run_id = RunId::from_str(&r_id)
            .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
        let t_id = TaskId::from_str(&r_task_id)
            .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
        let started_at = DateTime::parse_from_rfc3339(&r_started_at)
            .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
            .with_timezone(&Utc);
        let finished_at = r_finished_at
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
            agent: r_agent,
            account: r_account,
            status: r_status,
            started_at,
            finished_at,
            exit_code: r_exit_code,
        })
    }

    /// Danh sách các lần chạy của một task.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn list_runs(&self, task_id: &TaskId) -> Result<Vec<Run>, DbError> {
        let task_str = task_id.to_string();

        let rows = sqlx::query(
            r#"
            SELECT id, task_id, agent, account, status, started_at, finished_at, exit_code
            FROM run
            WHERE task_id = ?
            ORDER BY started_at ASC
            "#,
        )
        .bind(&task_str)
        .fetch_all(self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            list.push(self.map_run_row(&r)?);
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

        let result = sqlx::query(
            r#"
            UPDATE run
            SET status = ?, finished_at = ?, exit_code = ?
            WHERE id = ? AND task_id = ?
            "#,
        )
        .bind(status)
        .bind(finished_at_str)
        .bind(exit_code)
        .bind(&id_str)
        .bind(&task_str)
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

        sqlx::query(
            r#"
            DELETE FROM run
            WHERE id = ? AND task_id = ?
            "#,
        )
        .bind(&id_str)
        .bind(&task_str)
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

        sqlx::query(
            r#"
            INSERT INTO run_event (id, run_id, seq, event_type, data, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id_str)
        .bind(&run_str)
        .bind(event.seq)
        .bind(&event.event_type)
        .bind(&event.data)
        .bind(&created_at_str)
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

        let rows = sqlx::query(
            r#"
            SELECT id, run_id, seq, event_type, data, created_at
            FROM run_event
            WHERE run_id = ?
            ORDER BY seq ASC
            "#,
        )
        .bind(&run_str)
        .fetch_all(self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            list.push(self.map_event_row(&r)?);
        }

        Ok(list)
    }

    /// Lấy danh sách sự kiện phân trang cùng tổng số sự kiện của một lần chạy.
    ///
    /// # Errors
    /// Trả về `DbError` nếu truy vấn thất bại.
    pub async fn list_run_events_paginated(
        &self,
        run_id: &RunId,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<RunEvent>, i64), DbError> {
        let run_str = run_id.to_string();

        let count_row = sqlx::query("SELECT COUNT(*) as total FROM run_event WHERE run_id = ?")
            .bind(&run_str)
            .fetch_one(self.pool)
            .await?;
        let total: i64 = sqlx::Row::try_get(&count_row, "total")?;

        let rows = sqlx::query(
            r#"
            SELECT id, run_id, seq, event_type, data, created_at
            FROM run_event
            WHERE run_id = ?
            ORDER BY seq ASC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&run_str)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            list.push(self.map_event_row(&r)?);
        }

        Ok((list, total))
    }

    fn map_event_row(&self, r: &sqlx::sqlite::SqliteRow) -> Result<RunEvent, DbError> {
        let id_val: String = sqlx::Row::try_get(r, "id")?;
        let r_id_val: String = sqlx::Row::try_get(r, "run_id")?;
        let seq_val: i64 = sqlx::Row::try_get(r, "seq")?;
        let event_type_val: String = sqlx::Row::try_get(r, "event_type")?;
        let data_val: String = sqlx::Row::try_get(r, "data")?;
        let created_at_val: String = sqlx::Row::try_get(r, "created_at")?;

        let event_id = RunEventId::from_str(&id_val)
            .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
        let r_id = RunId::from_str(&r_id_val)
            .map_err(|e| DbError::ConstraintViolation(format!("UUID không hợp lệ: {e}")))?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_val)
            .map_err(|e| DbError::ConstraintViolation(format!("Timestamp không hợp lệ: {e}")))?
            .with_timezone(&Utc);

        Ok(RunEvent {
            id: event_id,
            run_id: r_id,
            seq: seq_val,
            event_type: event_type_val,
            data: data_val,
            created_at,
        })
    }
}

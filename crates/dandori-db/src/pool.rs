//! Quản lý kết nối và vòng đời SQLite pool của Dandori (RS-DB-001).

use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

use crate::error::DbError;
use crate::repo::{
    AgentAccountRepo, FeatureRepo, ProjectRepo, RunEventRepo, RunRepo, SecretRepo, SessionRepo,
    TaskRepo, UserRepo, WorkspaceRepo,
};

/// Bộ quản lý pool kết nối cơ sở dữ liệu SQLite của Dandori.
#[derive(Debug, Clone)]
pub struct DbPool {
    pool: SqlitePool,
}

impl DbPool {
    /// Khởi tạo kết nối tới tệp SQLite trên đĩa, tự động áp dụng migrations.
    ///
    /// # Errors
    /// Trả về `DbError` nếu không thể kết nối hoặc migration thất bại.
    pub async fn from_path(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_millis(5000));

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect_with(options)
            .await?;

        let db = Self { pool };
        db.migrate().await?;
        Ok(db)
    }

    /// Khởi tạo SQLite trong bộ nhớ (in-memory) cho kiểm thử hoặc môi trường tạm thời.
    ///
    /// # Errors
    /// Trả về `DbError` nếu không thể tạo pool hoặc migration thất bại.
    pub async fn in_memory() -> Result<Self, DbError> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:?cache=shared")?
            .foreign_keys(true)
            .busy_timeout(Duration::from_millis(5000));

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let db = Self { pool };
        db.migrate().await?;
        Ok(db)
    }

    /// Áp dụng các migrations SQL tiến duy nhất.
    ///
    /// # Errors
    /// Trả về `DbError` nếu xảy ra lỗi trong quá trình thực thi migrations.
    pub async fn migrate(&self) -> Result<(), DbError> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Trả về tham chiếu tới `SqlitePool` nội bộ của sqlx.
    #[must_use]
    pub fn inner(&self) -> &SqlitePool {
        &self.pool
    }

    /// Truy cập kho dữ liệu `User`.
    #[must_use]
    pub fn user_repo(&self) -> UserRepo<'_> {
        UserRepo::new(&self.pool)
    }

    /// Truy cập kho dữ liệu `Session`.
    #[must_use]
    pub fn session_repo(&self) -> SessionRepo<'_> {
        SessionRepo::new(&self.pool)
    }

    /// Truy cập kho dữ liệu `Workspace`.
    #[must_use]
    pub fn workspace_repo(&self) -> WorkspaceRepo<'_> {
        WorkspaceRepo::new(&self.pool)
    }

    /// Truy cập kho dữ liệu `Project`.
    #[must_use]
    pub fn project_repo(&self) -> ProjectRepo<'_> {
        ProjectRepo::new(&self.pool)
    }

    /// Truy cập kho dữ liệu `Task`.
    #[must_use]
    pub fn task_repo(&self) -> TaskRepo<'_> {
        TaskRepo::new(&self.pool)
    }

    /// Truy cập kho dữ liệu `Run`.
    #[must_use]
    pub fn run_repo(&self) -> RunRepo<'_> {
        RunRepo::new(&self.pool)
    }

    /// Truy cập kho dữ liệu `RunEvent`.
    #[must_use]
    pub fn run_event_repo(&self) -> RunEventRepo<'_> {
        RunEventRepo::new(&self.pool)
    }

    /// Truy cập kho dữ liệu `Secret`.
    #[must_use]
    pub fn secret_repo(&self) -> SecretRepo<'_> {
        SecretRepo::new(&self.pool)
    }

    /// Truy cập kho dữ liệu `AgentAccount`.
    #[must_use]
    pub fn agent_account_repo(&self) -> AgentAccountRepo<'_> {
        AgentAccountRepo::new(&self.pool)
    }

    /// Truy cập kho dữ liệu các bảng tính năng (`feature_*`).
    #[must_use]
    pub fn feature_repo(&self) -> FeatureRepo<'_> {
        FeatureRepo::new(&self.pool)
    }
}

use std::path::Path;

use dandori_module_api::{
    BoxFuture, DecoratorContext, DecoratorError, DecoratorResult, WorkdirDecorator,
    safe_write_worktree_file,
};
use sqlx::SqlitePool;

use crate::generator::{default_target_filename, generate_instruction_content};

pub const ALLOWED_FILENAMES: &[&str] = &["AGENTS.md", "CLAUDE.md", ".cursorrules"];

#[derive(Debug)]
pub struct InstructionSyncDecorator {
    pool: SqlitePool,
}

impl InstructionSyncDecorator {
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ConfigRow {
    target_filename: String,
    strategy: String,
    is_enabled: i64,
}

impl WorkdirDecorator for InstructionSyncDecorator {
    fn allowed_filenames(&self) -> &[&str] {
        ALLOWED_FILENAMES
    }

    #[expect(
        clippy::too_many_lines,
        reason = "Hàm decorate worktree thực hiện phân nhánh chiến lược và xử lý kết quả"
    )]
    fn decorate(
        &self,
        workdir: &Path,
        context: &DecoratorContext,
    ) -> BoxFuture<'static, Result<DecoratorResult, DecoratorError>> {
        let pool = self.pool.clone();
        let workdir = workdir.to_path_buf();
        let ctx = context.clone();

        Box::pin(async move {
            let provider_str = ctx.provider.as_deref().unwrap_or("codex");
            let row_opt = sqlx::query_as::<_, ConfigRow>(
                r#"
                SELECT target_filename, strategy, is_enabled
                FROM m_instruction_sync_config
                WHERE workspace_id = ? AND project_id = ? AND provider = ?
                "#,
            )
            .bind(&ctx.workspace_id)
            .bind(&ctx.project_id)
            .bind(provider_str)
            .fetch_optional(&pool)
            .await
            .map_err(|e| DecoratorError::Other(e.to_string()))?;

            let (target_filename, strategy, is_enabled) = match row_opt {
                Some(row) => (row.target_filename, row.strategy, row.is_enabled != 0),
                None => (
                    default_target_filename(provider_str).to_string(),
                    "skip".to_string(),
                    true,
                ),
            };

            if !is_enabled {
                return Ok(DecoratorResult {
                    created_files: Vec::new(),
                    skipped_files: vec![target_filename],
                    appended_files: Vec::new(),
                });
            }

            let append_mode = strategy.eq_ignore_ascii_case("append");

            let role_name = format!("{:?}", ctx.role);
            let (content, _tech_count, _skills_count) = generate_instruction_content(
                &pool,
                &ctx.workspace_id,
                &ctx.project_id,
                provider_str,
                Some(&role_name),
            )
            .await
            .map_err(|e| DecoratorError::Other(e.to_string()))?;

            let target_exists = workdir.join(&target_filename).exists();

            let wrote = safe_write_worktree_file(
                &workdir,
                &target_filename,
                &content,
                ALLOWED_FILENAMES,
                append_mode,
                Some("DANDORI INSTRUCTION SYNC"),
            )
            .await?;

            let result = if wrote {
                if target_exists {
                    DecoratorResult {
                        created_files: Vec::new(),
                        skipped_files: Vec::new(),
                        appended_files: vec![target_filename],
                    }
                } else {
                    DecoratorResult {
                        created_files: vec![target_filename],
                        skipped_files: Vec::new(),
                        appended_files: Vec::new(),
                    }
                }
            } else {
                DecoratorResult {
                    created_files: Vec::new(),
                    skipped_files: vec![target_filename],
                    appended_files: Vec::new(),
                }
            };

            Ok(result)
        })
    }
}
